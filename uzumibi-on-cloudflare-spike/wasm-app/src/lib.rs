#![allow(static_mut_refs)]
extern crate mrubyedge;
extern crate uzumibi_gem;

use std::{mem::MaybeUninit, rc::Rc};

use mrubyedge::{
    rite::rite,
    yamrb::{
        helpers::{mrb_define_class_cmethod, mrb_define_cmethod, mrb_funcall},
        value::{RObject, RValue},
        vm::VM,
    },
};

static MRB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/app.mrb"));

static mut MRUBY_VM: MaybeUninit<VM> = MaybeUninit::uninit();
static mut MRUBY_VM_LOADED: bool = false;

static mut ERROR_BUF: [u8; 4096] = [0; 4096];

fn set_error_to_buf(message: impl AsRef<str>) -> *const u8 {
    unsafe {
        let bytes = message.as_ref().as_bytes();
        let len = bytes.len().min(ERROR_BUF.len() - 1);
        ERROR_BUF[..len].copy_from_slice(&bytes[..len]);
        ERROR_BUF[len] = 0;
        ERROR_BUF.as_ptr()
    }
}

unsafe extern "C" {
    unsafe fn debug_console_log(ptr: *const u8, len: usize);
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

fn debug_console_log_internal(message: &str) {
    unsafe {
        debug_console_log(message.as_ptr(), message.len());
    }
}

// ---- External API wrappers (only when enable-external feature is active) ----

#[cfg(feature = "enable-external")]
fn cf_fetch(url: &str, method: &str, body: &str) -> Result<String, String> {
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
            buffer.as_mut_ptr(),
            BUFFER_SIZE,
        );
        match result {
            len if len >= 0 => {
                let len = len as usize;
                String::from_utf8(buffer[..len].to_vec())
                    .map_err(|e| format!("Failed to decode UTF-8: {}", e))
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

/// Fetch.fetch(url, method="GET", body="")
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

    match cf_fetch(&url, &method, &body) {
        Ok(response) => Ok(RObject::string(response).to_refcount_assigned()),
        Err(e) => Err(mrubyedge::Error::RuntimeError(format!(
            "Fetch failed: {}",
            e
        ))),
    }
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

// ---- VM initialization ----

fn init_vm() -> Result<VM, mrubyedge::Error> {
    let mut rite = rite::load(MRB)
        .map_err(|e| mrubyedge::Error::RuntimeError(format!("Failed to load mruby: {:?}", e)))?;
    let mut vm = VM::open(&mut rite);
    uzumibi_gem::init::init_uzumibi(&mut vm);
    let object = vm.object_class.clone();
    mrb_define_cmethod(
        &mut vm,
        object,
        "debug_console",
        Box::new(uzumibi_kernel_debug_console_log),
    );

    #[cfg(feature = "enable-external")]
    {
        let uzumibi_module = vm.get_module_by_name("Uzumibi");

        // Uzumibi::Fetch.fetch(url, method="GET", body="")
        let fetch_class = vm.define_class("Fetch", None, Some(uzumibi_module.clone()));
        mrb_define_class_cmethod(
            &mut vm,
            fetch_class,
            "fetch",
            Box::new(uzumibi_fetch_class_fetch),
        );

        // Uzumibi::KV.get(key) / Uzumibi::KV.set(key, value)
        let kv_class = vm.define_class("KV", None, Some(uzumibi_module.clone()));
        mrb_define_class_cmethod(
            &mut vm,
            kv_class.clone(),
            "get",
            Box::new(uzumibi_kv_class_get),
        );
        mrb_define_class_cmethod(&mut vm, kv_class, "set", Box::new(uzumibi_kv_class_set));

        // Uzumibi::Queue.send(queue_name, message)
        let queue_class = vm.define_class("Queue", None, Some(uzumibi_module));
        mrb_define_class_cmethod(
            &mut vm,
            queue_class,
            "send",
            Box::new(uzumibi_queue_class_send),
        );
    }

    vm.run()
        .map_err(|e| mrubyedge::Error::RuntimeError(format!("Failed to init VM: {:?}", e)))?;

    Ok(vm)
}

fn assume_init_vm() -> Result<&'static mut VM, mrubyedge::Error> {
    unsafe {
        if !MRUBY_VM_LOADED {
            MRUBY_VM = MaybeUninit::new(init_vm()?);
            MRUBY_VM_LOADED = true;
        }
        Ok(MRUBY_VM.assume_init_mut())
    }
}

fn do_uzumibi_initialize_request(size: i32) -> Result<*mut u8, mrubyedge::Error> {
    let vm = assume_init_vm()?;
    let size = RObject::integer(size as i64).to_refcount_assigned();
    let app = vm
        .globals
        .get("$APP")
        .ok_or_else(|| mrubyedge::Error::RuntimeError("$APP is not defined".to_string()))?;
    let ret = mrb_funcall(vm, app.clone().into(), "initialize_request", &[size])?;
    ret.as_ref().try_into()
}

fn do_uzumibi_start_request() -> Result<*mut u8, mrubyedge::Error> {
    debug_console_log_internal("uzumibi_start_request called");
    let vm = assume_init_vm()?;
    let app = vm
        .globals
        .get("$APP")
        .ok_or_else(|| mrubyedge::Error::RuntimeError("$APP is not defined".to_string()))?;
    let ret = mrb_funcall(
        vm,
        app.clone().into(),
        "start_request_and_return_shared_memory",
        &[],
    )?;
    match &ret.as_ref().value {
        RValue::SharedMemory(sm) => Ok(sm.borrow_mut().leak()),
        _ => Err(mrubyedge::Error::RuntimeError(
            "Returned value is not SharedMemory".to_string(),
        )),
    }
}

#[unsafe(export_name = "uzumibi_initialize_request")]
unsafe extern "C" fn uzumibi_initialize_request(size: i32) -> u64 {
    match do_uzumibi_initialize_request(size) {
        Ok(ptr) => (ptr as u32) as u64,
        Err(e) => {
            let err_buf = set_error_to_buf(format!("Error in initialize_request: {}", e));
            ((err_buf as u32) as u64) << 32
        }
    }
}

#[unsafe(export_name = "uzumibi_start_request")]
unsafe extern "C" fn uzumibi_start_request() -> u64 {
    match do_uzumibi_start_request() {
        Ok(ptr) => (ptr as u32) as u64,
        Err(e) => {
            let err_buf = set_error_to_buf(format!("Error in start_request: {}", e));
            ((err_buf as u32) as u64) << 32
        }
    }
}
