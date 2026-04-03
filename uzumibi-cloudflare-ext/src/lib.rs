#![allow(static_mut_refs)]
extern crate mrubyedge;
extern crate uzumibi_gem;

use std::rc::Rc;

#[cfg(feature = "queue")]
use mrubyedge::yamrb::value::RSym;
#[expect(unused_imports)]
use mrubyedge::yamrb::{
    helpers::{mrb_define_class_cmethod, mrb_define_cmethod, mrb_funcall},
    prelude::hash::{mrb_hash_new, mrb_hash_set_index},
    value::{RObject, RValue},
    vm::VM,
};

/// Special return value indicating that the request should be passed through to static assets.
pub const PASS_ASSETS: u64 = 0xFEFFFFFF;

// ---- Cloudflare-specific extern C declarations ----

unsafe extern "C" {
    unsafe fn debug_console_log(ptr: *const u8, len: usize);
}

#[cfg(feature = "queue")]
unsafe extern "C" {
    unsafe fn uzumibi_cf_message_ack(message_id_ptr: *const u8, message_id_size: usize) -> i32;
    unsafe fn uzumibi_cf_message_retry(
        message_id_ptr: *const u8,
        message_id_size: usize,
        delay_seconds: i32,
    ) -> i32;
}

#[cfg(feature = "enable-external")]
unsafe extern "C" {
    unsafe fn uzumibi_cf_fetch(
        url_ptr: *const u8,
        url_size: usize,
        method_ptr: *const u8,
        method_size: usize,
        body_ptr: *const u8,
        body_size: usize,
        headers_ptr: *const u8,
        headers_size: usize,
        result_ptr: *mut u8,
        result_max_size: usize,
    ) -> i32;
    unsafe fn uzumibi_cf_durable_object_get(
        key_ptr: *const u8,
        key_size: usize,
        result_ptr: *mut u8,
        result_max_size: usize,
    ) -> i32;
    unsafe fn uzumibi_cf_durable_object_set(
        key_ptr: *const u8,
        key_size: usize,
        value_ptr: *const u8,
        value_size: usize,
    ) -> i32;
    unsafe fn uzumibi_cf_queue_send(
        queue_name_ptr: *const u8,
        queue_name_size: usize,
        message_ptr: *const u8,
        message_size: usize,
    ) -> i32;
}

// ---- Debug console ----

pub fn debug_console_log_internal(message: &str) {
    unsafe {
        debug_console_log(message.as_ptr(), message.len());
    }
}

// ---- External API wrappers (only when enable-external feature is active) ----

/// Packed response format (same as Uzumibi::Response#to_shared_memory):
///   u16 LE status_code
///   u16 LE headers_count
///   (u16 LE key_size, key bytes, u16 LE value_size, value bytes) * headers_count
///   u32 LE body_size
///   body bytes
#[cfg(feature = "enable-external")]
fn cf_fetch(url: &str, method: &str, body: &str, headers: &[u8]) -> Result<Vec<u8>, String> {
    const BUFFER_SIZE: usize = 65536;
    let mut buffer = vec![0u8; BUFFER_SIZE];

    unsafe {
        let result = uzumibi_cf_fetch(
            url.as_ptr(),
            url.len(),
            method.as_ptr(),
            method.len(),
            body.as_ptr(),
            body.len(),
            headers.as_ptr(),
            headers.len(),
            buffer.as_mut_ptr(),
            BUFFER_SIZE,
        );
        match result {
            len if len >= 0 => {
                let len = len as usize;
                Ok(buffer[..len].to_vec())
            }
            _ => Err(format!("Fetch failed with return code: {}", result)),
        }
    }
}

#[cfg(feature = "enable-external")]
fn cf_durable_object_get(key: &str) -> Result<Option<String>, String> {
    const BUFFER_SIZE: usize = 65536;
    let mut buffer = vec![0u8; BUFFER_SIZE];

    unsafe {
        let result = uzumibi_cf_durable_object_get(
            key.as_ptr(),
            key.len(),
            buffer.as_mut_ptr(),
            BUFFER_SIZE,
        );
        match result {
            -1 => Ok(None),
            len if len >= 0 => {
                let len = len as usize;
                let value = String::from_utf8(buffer[..len].to_vec())
                    .map_err(|e| format!("Failed to decode UTF-8: {}", e))?;
                Ok(Some(value))
            }
            _ => Err(format!(
                "Unexpected return value from durable_object_get: {}",
                result
            )),
        }
    }
}

#[cfg(feature = "enable-external")]
fn cf_durable_object_set(key: &str, value: &str) -> Result<(), String> {
    unsafe {
        let result =
            uzumibi_cf_durable_object_set(key.as_ptr(), key.len(), value.as_ptr(), value.len());
        match result {
            0 => Ok(()),
            _ => Err(format!("Failed to set value: return code {}", result)),
        }
    }
}

#[cfg(feature = "enable-external")]
fn cf_queue_send(queue_name: &str, message: &str) -> Result<(), String> {
    unsafe {
        let result = uzumibi_cf_queue_send(
            queue_name.as_ptr(),
            queue_name.len(),
            message.as_ptr(),
            message.len(),
        );
        match result {
            0 => Ok(()),
            _ => Err(format!(
                "Failed to send queue message: return code {}",
                result
            )),
        }
    }
}

// ---- mruby gem method implementations ----

fn uzumibi_kernel_debug_console_log(
    vm: &mut VM,
    args: &[Rc<RObject>],
) -> Result<Rc<RObject>, mrubyedge::Error> {
    let msg_obj = &args[0];
    let msg = mrb_funcall(vm, msg_obj.clone().into(), "to_s", &[])?;
    let msg: String = msg.as_ref().try_into()?;
    unsafe {
        debug_console_log(msg.as_ptr(), msg.len());
    }
    Ok(RObject::nil().to_refcount_assigned())
}

/// Fetch.fetch(url, method="GET", body="", headers={}) -> Uzumibi::Response
#[cfg(feature = "enable-external")]
fn uzumibi_fetch_class_fetch(
    vm: &mut VM,
    args: &[Rc<RObject>],
) -> Result<Rc<RObject>, mrubyedge::Error> {
    let url_obj = &args[0];
    let url = mrb_funcall(vm, url_obj.clone().into(), "to_s", &[])?;
    let url: String = url.as_ref().try_into()?;

    let method = if args.len() > 1 {
        let m = mrb_funcall(vm, args[1].clone().into(), "to_s", &[])?;
        let m: String = m.as_ref().try_into()?;
        m
    } else {
        "GET".to_string()
    };

    let body = if args.len() > 2 {
        let b = mrb_funcall(vm, args[2].clone().into(), "to_s", &[])?;
        let b: String = b.as_ref().try_into()?;
        b
    } else {
        String::new()
    };

    // Pack request headers from Hash (4th argument)
    let packed_headers = if args.len() > 3 {
        pack_headers_from_hash(vm, &args[3])?
    } else {
        vec![0u8; 2] // u16 LE count = 0
    };

    let packed = cf_fetch(&url, &method, &body, &packed_headers)
        .map_err(|e| mrubyedge::Error::RuntimeError(format!("Fetch failed: {}", e)))?;

    // Unpack the packed response into Uzumibi::Response
    unpack_response_to_robject(vm, &packed)
}

/// Pack a mruby Hash into binary format for request headers:
///   u16 LE headers_count
///   (u16 LE key_size, key bytes, u16 LE value_size, value bytes) * count
#[cfg(feature = "enable-external")]
fn pack_headers_from_hash(
    vm: &mut VM,
    hash_obj: &Rc<RObject>,
) -> Result<Vec<u8>, mrubyedge::Error> {
    match &hash_obj.as_ref().value {
        RValue::Hash(h) => {
            let hash = h.borrow();
            let mut buf = Vec::new();
            let count = hash.len() as u16;
            buf.extend_from_slice(&count.to_le_bytes());
            for (_, (key_obj, value_obj)) in hash.iter() {
                let key = mrb_funcall(vm, key_obj.clone().into(), "to_s", &[])?;
                let key: String = key.as_ref().try_into()?;
                let value = mrb_funcall(vm, value_obj.clone().into(), "to_s", &[])?;
                let value: String = value.as_ref().try_into()?;
                buf.extend_from_slice(&(key.len() as u16).to_le_bytes());
                buf.extend_from_slice(key.as_bytes());
                buf.extend_from_slice(&(value.len() as u16).to_le_bytes());
                buf.extend_from_slice(value.as_bytes());
            }
            Ok(buf)
        }
        RValue::Nil => {
            Ok(vec![0u8; 2]) // u16 LE count = 0
        }
        _ => Err(mrubyedge::Error::RuntimeError(
            "headers argument must be a Hash".to_string(),
        )),
    }
}

/// Unpack packed binary response into Uzumibi::Response mruby object
#[cfg(feature = "enable-external")]
fn unpack_response_to_robject(vm: &mut VM, buf: &[u8]) -> Result<Rc<RObject>, mrubyedge::Error> {
    let mut offset = 0;

    // Status code (u16 LE)
    let status_code = u16::from_le_bytes([buf[offset], buf[offset + 1]]);
    offset += 2;

    // Headers count (u16 LE)
    let headers_count = u16::from_le_bytes([buf[offset], buf[offset + 1]]) as usize;
    offset += 2;

    // Parse headers
    let headers_hash = mrb_hash_new(vm, &[])?;
    for _ in 0..headers_count {
        let key_size = u16::from_le_bytes([buf[offset], buf[offset + 1]]) as usize;
        offset += 2;
        let key = String::from_utf8_lossy(&buf[offset..offset + key_size]).to_string();
        offset += key_size;

        let value_size = u16::from_le_bytes([buf[offset], buf[offset + 1]]) as usize;
        offset += 2;
        let value = String::from_utf8_lossy(&buf[offset..offset + value_size]).to_string();
        offset += value_size;

        mrb_hash_set_index(
            headers_hash.clone(),
            RObject::string(key).to_refcount_assigned(),
            RObject::string(value).to_refcount_assigned(),
        )?;
    }

    // Body size (u32 LE)
    let body_size = u32::from_le_bytes([
        buf[offset],
        buf[offset + 1],
        buf[offset + 2],
        buf[offset + 3],
    ]) as usize;
    offset += 4;

    // Body
    let body = String::from_utf8_lossy(&buf[offset..offset + body_size]).to_string();

    // Create Uzumibi::Response instance
    let uzumibi = vm
        .get_const_by_name("Uzumibi")
        .ok_or_else(|| mrubyedge::Error::RuntimeError("Uzumibi module not found".to_string()))?;
    let uzumibi_module = match &uzumibi.as_ref().value {
        RValue::Module(m) => m.clone(),
        _ => {
            return Err(mrubyedge::Error::RuntimeError(
                "Uzumibi must be a module".to_string(),
            ));
        }
    };
    let response_class = uzumibi_module
        .get_const_by_name("Response")
        .ok_or_else(|| {
            mrubyedge::Error::RuntimeError("Uzumibi::Response class not found".to_string())
        })?;
    let response = mrb_funcall(vm, Some(response_class), "new", &[])?;

    response.set_ivar(
        "@status_code",
        RObject::integer(status_code as i64).to_refcount_assigned(),
    );
    response.set_ivar("@headers", headers_hash);
    response.set_ivar("@body", RObject::string(body).to_refcount_assigned());

    Ok(response)
}

/// KV.get(key)
#[cfg(feature = "enable-external")]
fn uzumibi_kv_class_get(
    vm: &mut VM,
    args: &[Rc<RObject>],
) -> Result<Rc<RObject>, mrubyedge::Error> {
    let key_obj = &args[0];
    let key = mrb_funcall(vm, key_obj.clone().into(), "to_s", &[])?;
    let key: String = key.as_ref().try_into()?;

    match cf_durable_object_get(&key) {
        Ok(Some(value)) => Ok(RObject::string(value).to_refcount_assigned()),
        Ok(None) => Ok(RObject::nil().to_refcount_assigned()),
        Err(e) => Err(mrubyedge::Error::RuntimeError(format!(
            "Failed to access storage value: {}",
            e
        ))),
    }
}

/// KV.set(key, value)
#[cfg(feature = "enable-external")]
fn uzumibi_kv_class_set(
    vm: &mut VM,
    args: &[Rc<RObject>],
) -> Result<Rc<RObject>, mrubyedge::Error> {
    let key_obj = &args[0];
    let key = mrb_funcall(vm, key_obj.clone().into(), "to_s", &[])?;
    let key: String = key.as_ref().try_into()?;

    let value_obj = &args[1];
    let value = mrb_funcall(vm, value_obj.clone().into(), "to_s", &[])?;
    let value: String = value.as_ref().try_into()?;

    cf_durable_object_set(&key, &value).map_err(|e| {
        mrubyedge::Error::RuntimeError(format!("Failed to set storage value: {}", e))
    })?;

    Ok(RObject::boolean(true).to_refcount_assigned())
}

/// Queue.send(queue_name, message)
#[cfg(feature = "enable-external")]
fn uzumibi_queue_class_send(
    vm: &mut VM,
    args: &[Rc<RObject>],
) -> Result<Rc<RObject>, mrubyedge::Error> {
    let queue_name_obj = &args[0];
    let queue_name = mrb_funcall(vm, queue_name_obj.clone().into(), "to_s", &[])?;
    let queue_name: String = queue_name.as_ref().try_into()?;

    let message_obj = &args[1];
    let message = mrb_funcall(vm, message_obj.clone().into(), "to_s", &[])?;
    let message: String = message.as_ref().try_into()?;

    cf_queue_send(&queue_name, &message).map_err(|e| {
        mrubyedge::Error::RuntimeError(format!("Failed to send queue message: {}", e))
    })?;

    Ok(RObject::boolean(true).to_refcount_assigned())
}

// ---- Queue consumer support (only when queue feature is active) ----

/// Message.ack! -> delegates to JS
#[cfg(feature = "queue")]
fn uzumibi_message_ack(
    vm: &mut VM,
    _args: &[Rc<RObject>],
) -> Result<Rc<RObject>, mrubyedge::Error> {
    let self_obj = vm.getself()?;
    let id_obj = self_obj.get_ivar("@id");
    if matches!(id_obj.as_ref().value, RValue::Nil) {
        return Err(mrubyedge::Error::RuntimeError(
            "Message object does not have @id".to_string(),
        ));
    }
    let id = mrb_funcall(vm, id_obj.into(), "to_s", &[])?;
    let id: String = id.as_ref().try_into()?;

    unsafe {
        let result = uzumibi_cf_message_ack(id.as_ptr(), id.len());
        if result != 0 {
            return Err(mrubyedge::Error::RuntimeError(format!(
                "Failed to ack message: return code {}",
                result
            )));
        }
    }
    Ok(RObject::boolean(true).to_refcount_assigned())
}

/// Message.retry(delay_seconds: N) -> delegates to JS
#[cfg(feature = "queue")]
fn uzumibi_message_retry(
    vm: &mut VM,
    _args: &[Rc<RObject>],
) -> Result<Rc<RObject>, mrubyedge::Error> {
    let self_obj = vm.getself()?;
    let id_obj = self_obj.get_ivar("@id");
    if matches!(id_obj.as_ref().value, RValue::Nil) {
        return Err(mrubyedge::Error::RuntimeError(
            "Message object does not have @id".to_string(),
        ));
    }
    let id = mrb_funcall(vm, id_obj.into(), "to_s", &[])?;
    let id: String = id.as_ref().try_into()?;

    let delay_seconds: i32 = match vm.get_kwargs() {
        Some(kwargs) => match kwargs.get("delay_seconds") {
            Some(val) => {
                let v: i64 = val.as_ref().try_into()?;
                v as i32
            }
            None => 0,
        },
        None => 0,
    };

    unsafe {
        let result = uzumibi_cf_message_retry(id.as_ptr(), id.len(), delay_seconds);
        if result != 0 {
            return Err(mrubyedge::Error::RuntimeError(format!(
                "Failed to retry message: return code {}",
                result
            )));
        }
    }
    Ok(RObject::boolean(true).to_refcount_assigned())
}

/// Message.nack! -> retry with delay_seconds=0
#[cfg(feature = "queue")]
fn uzumibi_message_nack(
    vm: &mut VM,
    _args: &[Rc<RObject>],
) -> Result<Rc<RObject>, mrubyedge::Error> {
    let self_obj = vm.getself()?;
    let id_obj = self_obj.get_ivar("@id");
    if matches!(id_obj.as_ref().value, RValue::Nil) {
        return Err(mrubyedge::Error::RuntimeError(
            "Message object does not have @id".to_string(),
        ));
    }
    let id = mrb_funcall(vm, id_obj.into(), "to_s", &[])?;
    let id: String = id.as_ref().try_into()?;

    unsafe {
        let result = uzumibi_cf_message_retry(id.as_ptr(), id.len(), 0);
        if result != 0 {
            return Err(mrubyedge::Error::RuntimeError(format!(
                "Failed to nack message: return code {}",
                result
            )));
        }
    }
    Ok(RObject::boolean(true).to_refcount_assigned())
}

/// Consumer.on_receive(message) - abstract method, must be overridden
#[cfg(feature = "queue")]
fn uzumibi_consumer_on_receive(
    _vm: &mut VM,
    _args: &[Rc<RObject>],
) -> Result<Rc<RObject>, mrubyedge::Error> {
    Err(mrubyedge::Error::RuntimeError(
        "on_receive must be implemented by subclass of Uzumibi::Consumer".to_string(),
    ))
}

// ---- Cloudflare Access ----

#[cfg(feature = "enable-external")]
static mut ACCESS_TEAM: Option<String> = None;

/// Extract body string from packed response buffer
#[cfg(feature = "enable-external")]
fn unpack_response_body(buf: &[u8]) -> Result<String, mrubyedge::Error> {
    let mut offset = 0;
    // Skip status code (u16)
    offset += 2;
    // Headers count (u16)
    let headers_count = u16::from_le_bytes([buf[offset], buf[offset + 1]]) as usize;
    offset += 2;
    // Skip headers
    for _ in 0..headers_count {
        let key_size = u16::from_le_bytes([buf[offset], buf[offset + 1]]) as usize;
        offset += 2 + key_size;
        let value_size = u16::from_le_bytes([buf[offset], buf[offset + 1]]) as usize;
        offset += 2 + value_size;
    }
    // Body size (u32)
    let body_size = u32::from_le_bytes([
        buf[offset],
        buf[offset + 1],
        buf[offset + 2],
        buf[offset + 3],
    ]) as usize;
    offset += 4;
    // Body
    Ok(String::from_utf8_lossy(&buf[offset..offset + body_size]).to_string())
}

/// Access.team=(name)
#[cfg(feature = "enable-external")]
fn uzumibi_access_set_team(
    vm: &mut VM,
    args: &[Rc<RObject>],
) -> Result<Rc<RObject>, mrubyedge::Error> {
    let team = mrb_funcall(vm, args[0].clone().into(), "to_s", &[])?;
    let team: String = team.as_ref().try_into()?;
    unsafe {
        ACCESS_TEAM = Some(team);
    }
    Ok(args[0].clone())
}

/// Access.get_identity(cf_authorization_token) -> Identity
#[cfg(feature = "enable-external")]
fn uzumibi_access_get_identity(
    vm: &mut VM,
    args: &[Rc<RObject>],
) -> Result<Rc<RObject>, mrubyedge::Error> {
    let token = mrb_funcall(vm, args[0].clone().into(), "to_s", &[])?;
    let token: String = token.as_ref().try_into()?;

    let team = unsafe {
        ACCESS_TEAM.as_ref().ok_or_else(|| {
            mrubyedge::Error::RuntimeError("Uzumibi::Access.team is not set".to_string())
        })?
    };

    let url = format!(
        "https://{}.cloudflareaccess.com/cdn-cgi/access/get-identity",
        team
    );

    // Pack Cookie header
    let cookie_value = format!("CF_Authorization={}", token);
    let mut headers_buf = Vec::new();
    let count: u16 = 1;
    headers_buf.extend_from_slice(&count.to_le_bytes());
    let key = b"cookie";
    headers_buf.extend_from_slice(&(key.len() as u16).to_le_bytes());
    headers_buf.extend_from_slice(key);
    let val = cookie_value.as_bytes();
    headers_buf.extend_from_slice(&(val.len() as u16).to_le_bytes());
    headers_buf.extend_from_slice(val);

    let packed = cf_fetch(&url, "GET", "", &headers_buf)
        .map_err(|e| mrubyedge::Error::RuntimeError(format!("Access fetch failed: {}", e)))?;

    let body = unpack_response_body(&packed)?;

    // Parse JSON body using mrubyedge-serde-json
    let body_robj = RObject::string(body).to_refcount_assigned();
    let json_value = mrubyedge_serde_json::mrb_json_class_load(vm, &[body_robj])?;

    // Create Identity and set fields from parsed JSON hash
    let uzumibi = vm
        .get_const_by_name("Uzumibi")
        .ok_or_else(|| mrubyedge::Error::RuntimeError("Uzumibi module not found".to_string()))?;
    let uzumibi_module = match &uzumibi.as_ref().value {
        RValue::Module(m) => m.clone(),
        _ => {
            return Err(mrubyedge::Error::RuntimeError(
                "Uzumibi must be a module".to_string(),
            ));
        }
    };
    let identity_class = uzumibi_module
        .get_const_by_name("Identity")
        .ok_or_else(|| {
            mrubyedge::Error::RuntimeError("Uzumibi::Identity class not found".to_string())
        })?;
    let identity = mrb_funcall(vm, Some(identity_class), "new", &[])?;

    // Extract known fields from JSON hash
    let field_mappings = [("user_uuid", "@user_uuid"), ("email", "@email")];
    for (json_key, ivar_key) in &field_mappings {
        let val = mrb_funcall(
            vm,
            Some(json_value.clone()),
            "[]",
            &[RObject::string(json_key.to_string()).to_refcount_assigned()],
        )?;
        identity.set_ivar(ivar_key, val);
    }

    // Store raw data hash
    identity.set_ivar("@raw_data", json_value);

    Ok(identity)
}

// ---- Assets pass-through ----

fn uzumibi_fetch_assets(
    _vm: &mut VM,
    _args: &[Rc<RObject>],
) -> Result<Rc<RObject>, mrubyedge::Error> {
    Err(mrubyedge::Error::TaggedError(
        "UzumibiPassAssets",
        "pass assets to platform".to_string(),
    ))
}

// ---- VM initialization ----

/// Initialize Cloudflare-specific mruby classes and methods on the given VM.
/// This should be called after `uzumibi_gem::init::init_uzumibi(&mut vm)`.
pub fn init_cloudflare_ext(vm: &mut VM) {
    // Define UzumibiPassAssets exception class
    let runtime_error = vm.get_class_by_name("RuntimeError");
    vm.define_class("UzumibiPassAssets", Some(runtime_error), None);

    // Kernel-level methods
    let object = vm.object_class.clone();
    mrb_define_cmethod(
        vm,
        object.clone(),
        "debug_console",
        Box::new(uzumibi_kernel_debug_console_log),
    );
    mrb_define_cmethod(vm, object, "fetch_assets", Box::new(uzumibi_fetch_assets));

    #[cfg(feature = "enable-external")]
    {
        let uzumibi_module = vm.get_module_by_name("Uzumibi");

        // Uzumibi::Fetch.fetch(url, method="GET", body="")
        let fetch_class = vm.define_class("Fetch", None, Some(uzumibi_module.clone()));
        mrb_define_class_cmethod(
            vm,
            fetch_class,
            "fetch",
            Box::new(uzumibi_fetch_class_fetch),
        );

        // Uzumibi::KV.get(key) / Uzumibi::KV.set(key, value)
        let kv_class = vm.define_class("KV", None, Some(uzumibi_module.clone()));
        mrb_define_class_cmethod(vm, kv_class.clone(), "get", Box::new(uzumibi_kv_class_get));
        mrb_define_class_cmethod(vm, kv_class, "set", Box::new(uzumibi_kv_class_set));

        // Uzumibi::Queue.send(queue_name, message)
        let queue_class = vm.define_class("Queue", None, Some(uzumibi_module.clone()));
        mrb_define_class_cmethod(vm, queue_class, "send", Box::new(uzumibi_queue_class_send));

        // Uzumibi::Access.team= / Uzumibi::Access.get_identity(token)
        let access_class = vm.define_class("Access", None, Some(uzumibi_module.clone()));
        mrb_define_class_cmethod(
            vm,
            access_class.clone(),
            "team=",
            Box::new(uzumibi_access_set_team),
        );
        mrb_define_class_cmethod(
            vm,
            access_class,
            "get_identity",
            Box::new(uzumibi_access_get_identity),
        );

        // Uzumibi::Identity with attr_accessor for common fields
        let identity_class = vm.define_class("Identity", None, Some(uzumibi_module));
        let identity_class_obj = RObject::class(identity_class, vm);
        for attr in ["user_uuid", "email", "raw_data"] {
            mrb_funcall(
                vm,
                Some(identity_class_obj.clone()),
                "attr_accessor",
                &[
                    RObject::symbol(mrubyedge::yamrb::value::RSym::new(attr.to_string()))
                        .to_refcount_assigned(),
                ],
            )
            .expect("attr_accessor failed");
        }
    }

    #[cfg(feature = "queue")]
    {
        let uzumibi_module = vm.get_module_by_name("Uzumibi");

        // Uzumibi::Consumer (base class for user-defined consumers)
        let consumer_class = vm.define_class("Consumer", None, Some(uzumibi_module.clone()));
        mrb_define_cmethod(
            vm,
            consumer_class,
            "on_receive",
            Box::new(uzumibi_consumer_on_receive),
        );

        // Uzumibi::Message with ack! and retry methods
        let message_class = vm.define_class("Message", None, Some(uzumibi_module));
        let message_class_obj = RObject::class(message_class.clone(), vm);
        for attr in ["id", "timestamp", "body", "attempts"] {
            mrb_funcall(
                vm,
                Some(message_class_obj.clone()),
                "attr_accessor",
                &[RObject::symbol(RSym::new(attr.to_string())).to_refcount_assigned()],
            )
            .expect("attr_accessor failed");
        }
        mrb_define_cmethod(
            vm,
            message_class.clone(),
            "ack!",
            Box::new(uzumibi_message_ack),
        );
        mrb_define_cmethod(
            vm,
            message_class.clone(),
            "nack!",
            Box::new(uzumibi_message_nack),
        );
        mrb_define_cmethod(vm, message_class, "retry", Box::new(uzumibi_message_retry));
    }
}

/// Unpack a queue message from a binary buffer and call `$CONSUMER.on_receive(message)`.
///
/// Message binary format:
///   u16 LE id_size, id bytes,
///   u16 LE timestamp_size, timestamp bytes,
///   u32 LE body_size, body bytes,
///   u32 LE attempts
#[cfg(feature = "queue")]
pub fn dispatch_queue_message(vm: &mut VM, buf: &[u8]) -> Result<(), mrubyedge::Error> {
    let mut offset = 0;

    // id (u16 LE size + bytes)
    let id_size = u16::from_le_bytes([buf[offset], buf[offset + 1]]) as usize;
    offset += 2;
    let id = String::from_utf8_lossy(&buf[offset..offset + id_size]).to_string();
    offset += id_size;

    // timestamp (u16 LE size + bytes)
    let ts_size = u16::from_le_bytes([buf[offset], buf[offset + 1]]) as usize;
    offset += 2;
    let timestamp = String::from_utf8_lossy(&buf[offset..offset + ts_size]).to_string();
    offset += ts_size;

    // body (u32 LE size + bytes)
    let body_size = u32::from_le_bytes([
        buf[offset],
        buf[offset + 1],
        buf[offset + 2],
        buf[offset + 3],
    ]) as usize;
    offset += 4;
    let body = String::from_utf8_lossy(&buf[offset..offset + body_size]).to_string();
    offset += body_size;

    // attempts (u32 LE)
    let attempts = u32::from_le_bytes([
        buf[offset],
        buf[offset + 1],
        buf[offset + 2],
        buf[offset + 3],
    ]) as i64;

    // Create Uzumibi::Message instance
    let uzumibi = vm
        .get_const_by_name("Uzumibi")
        .ok_or_else(|| mrubyedge::Error::RuntimeError("Uzumibi module not found".to_string()))?;
    let uzumibi_module = match &uzumibi.as_ref().value {
        RValue::Module(m) => m.clone(),
        _ => {
            return Err(mrubyedge::Error::RuntimeError(
                "Uzumibi must be a module".to_string(),
            ));
        }
    };
    let message_class = uzumibi_module.get_const_by_name("Message").ok_or_else(|| {
        mrubyedge::Error::RuntimeError("Uzumibi::Message class not found".to_string())
    })?;
    let message = mrb_funcall(vm, Some(message_class), "new", &[])?;

    message.set_ivar("@id", RObject::string(id).to_refcount_assigned());
    message.set_ivar(
        "@timestamp",
        RObject::string(timestamp).to_refcount_assigned(),
    );
    message.set_ivar("@body", RObject::string(body).to_refcount_assigned());
    message.set_ivar(
        "@attempts",
        RObject::integer(attempts).to_refcount_assigned(),
    );

    // Call $CONSUMER.on_receive(message)
    let consumer = vm
        .globals
        .get("$CONSUMER")
        .ok_or_else(|| mrubyedge::Error::RuntimeError("$CONSUMER is not defined".to_string()))?;
    mrb_funcall(vm, consumer.clone().into(), "on_receive", &[message])?;

    Ok(())
}
