#![allow(static_mut_refs)]
extern crate mrubyedge;
extern crate uzumibi_cloudflare_ext;
extern crate uzumibi_gem;

use std::mem::MaybeUninit;

use mrubyedge::{
    rite::rite,
    yamrb::{
        helpers::mrb_funcall,
        value::{RObject, RValue},
        vm::VM,
    },
};

#[cfg(not(feature = "queue"))]
static MRB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/app.mrb"));
#[cfg(feature = "queue")]
static MRB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/consumer.mrb"));

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

// ---- VM initialization ----

fn init_vm() -> Result<VM, mrubyedge::Error> {
    let mut rite = rite::load(MRB)
        .map_err(|e| mrubyedge::Error::RuntimeError(format!("Failed to load mruby: {:?}", e)))?;
    let mut vm = VM::open(&mut rite);
    mrubyedge_serde_json::init_json(&mut vm);
    uzumibi_gem::init::init_uzumibi(&mut vm);
    uzumibi_cloudflare_ext::init_cloudflare_ext(&mut vm);

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
    uzumibi_cloudflare_ext::debug_console_log_internal("uzumibi_start_request called");
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
        Err(mrubyedge::Error::TaggedError("UzumibiPassAssets", _)) => {
            uzumibi_cloudflare_ext::PASS_ASSETS << 32
        }
        Err(e) => {
            let err_buf = set_error_to_buf(format!("Error in start_request: {}", e));
            ((err_buf as u32) as u64) << 32
        }
    }
}

// ---- Queue message handling (only when queue feature is active) ----

#[cfg(feature = "queue")]
static mut MESSAGE_BUF: Option<Vec<u8>> = None;

#[cfg(feature = "queue")]
fn do_uzumibi_initialize_message(size: i32) -> Result<*mut u8, mrubyedge::Error> {
    let _ = assume_init_vm()?;
    unsafe {
        MESSAGE_BUF = Some(vec![0u8; size as usize]);
        Ok(MESSAGE_BUF.as_mut().unwrap().as_mut_ptr())
    }
}

#[cfg(feature = "queue")]
fn do_uzumibi_start_message() -> Result<(), mrubyedge::Error> {
    uzumibi_cloudflare_ext::debug_console_log_internal("uzumibi_start_message called");
    let vm = assume_init_vm()?;

    let buf = unsafe {
        MESSAGE_BUF.as_ref().ok_or_else(|| {
            mrubyedge::Error::RuntimeError("Message buffer not initialized".to_string())
        })?
    };

    uzumibi_cloudflare_ext::dispatch_queue_message(vm, buf)
}

#[cfg(feature = "queue")]
#[unsafe(export_name = "uzumibi_initialize_message")]
unsafe extern "C" fn uzumibi_initialize_message(size: i32) -> u64 {
    match do_uzumibi_initialize_message(size) {
        Ok(ptr) => (ptr as u32) as u64,
        Err(e) => {
            let err_buf = set_error_to_buf(format!("Error in initialize_message: {}", e));
            ((err_buf as u32) as u64) << 32
        }
    }
}

#[cfg(feature = "queue")]
#[unsafe(export_name = "uzumibi_start_message")]
unsafe extern "C" fn uzumibi_start_message() -> u32 {
    match do_uzumibi_start_message() {
        Ok(()) => 0,
        Err(e) => set_error_to_buf(format!("Error in start_message: {}", e)) as u32,
    }
}
