#![allow(static_mut_refs)]
extern crate mrubyedge;
extern crate spin_sdk;
extern crate uzumibi_gem;

use std::{cell::RefCell, mem::MaybeUninit, rc::Rc};

use mrubyedge::{
    rite::rite,
    yamrb::{
        helpers::{mrb_define_cmethod, mrb_funcall},
        shared_memory::SharedMemory,
        value::{RObject, RValue},
        vm::VM,
    },
};
use spin_sdk::http::{HeaderValue, Request, Response};

static MRB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/app.mrb"));

static mut MRUBY_VM: MaybeUninit<VM> = MaybeUninit::uninit();
static mut MRUBY_VM_LOADED: bool = false;

fn debug_console_log_internal(message: &str) {
    println!("{}", message);
}

fn uzumibi_kernel_debug_console_log(
    vm: &mut VM,
    args: &[Rc<RObject>],
) -> Result<Rc<RObject>, mrubyedge::Error> {
    let msg_obj = &args[0];
    let msg = mrb_funcall(vm, msg_obj.clone().into(), "to_s", &[])?;
    let msg: String = msg.as_ref().try_into()?;
    debug_console_log_internal(&msg);
    Ok(RObject::nil().to_refcount_assigned())
}

fn init_vm() -> VM {
    println!("Initializing MRuby VM");

    let mut rite = rite::load(MRB).expect("failed to load");
    let mut vm = VM::open(&mut rite);
    uzumibi_gem::init::init_uzumibi(&mut vm);
    let object = vm.object_class.clone();
    mrb_define_cmethod(
        &mut vm,
        object,
        "debug_console",
        Box::new(uzumibi_kernel_debug_console_log),
    );

    vm.run().expect("failed to run");

    vm
}

fn assume_init_vm() -> &'static mut VM {
    unsafe {
        if !MRUBY_VM_LOADED {
            MRUBY_VM = MaybeUninit::new(init_vm());
            MRUBY_VM_LOADED = true;
        }
        MRUBY_VM.assume_init_mut()
    }
}

pub fn uzumibi_initialize_request(size: i32) -> Rc<RefCell<SharedMemory>> {
    let vm = assume_init_vm();
    let size = RObject::integer(size as i64).to_refcount_assigned();
    let app = vm
        .globals
        .get("$APP")
        .or_else(|| {
            debug_console_log_internal("$APP is not defined");
            None
        })
        .unwrap();
    let ret = mrb_funcall(vm, app.clone().into(), "initialize_request", &[size])
        .map_err(|e| {
            debug_console_log_internal(&format!("Error in initialize_request: {}", e));
            e
        })
        .unwrap();
    robject_as_shared_memory(ret)
}

fn robject_as_shared_memory(obj: Rc<RObject>) -> Rc<RefCell<SharedMemory>> {
    match obj.value {
        RValue::SharedMemory(ref sm) => sm.clone(),
        _ => panic!("Expected SharedMemory object"),
    }
}

pub fn pack_request_data(request: &Request) -> Vec<u8> {
    let mut data = Vec::new();
    let method = request.method().to_string();
    let path = request.path().as_bytes();
    let headers = request.headers().collect::<Vec<(&str, &HeaderValue)>>();

    // Method (6 bytes, padded with \0)
    let mut method_buf = [0u8; 6];
    method_buf[..method.len().min(6)].copy_from_slice(&method.as_bytes()[..method.len().min(6)]);
    data.extend_from_slice(&method_buf);

    // Path size (u16 little-endian)
    let path_size = path.len() as u16;
    data.extend_from_slice(&path_size.to_le_bytes());

    // Path
    data.extend_from_slice(path);

    // Headers count (u16 little-endian)
    let headers_count = headers.len() as u16;
    data.extend_from_slice(&headers_count.to_le_bytes());

    // Headers (key size, key, value size, value)
    for (key, value) in headers.iter() {
        let key_bytes = key.as_bytes();
        let value_bytes = value.as_bytes();

        let key_size = key_bytes.len() as u16;
        data.extend_from_slice(&key_size.to_le_bytes());
        data.extend_from_slice(key_bytes);

        let value_size = value_bytes.len() as u16;
        data.extend_from_slice(&value_size.to_le_bytes());
        data.extend_from_slice(value_bytes);
    }

    data
}

pub fn uzumibi_start_request() -> Response {
    let vm = assume_init_vm();
    let app = vm
        .globals
        .get("$APP")
        .or_else(|| {
            debug_console_log_internal("$APP is not defined");
            None
        })
        .unwrap();
    let ret = mrb_funcall(vm, app.clone().into(), "start_request", &[])
        .map_err(|e| {
            debug_console_log_internal(&format!("Error in start_request: {}", e));
            e
        })
        .unwrap();
    robject_as_response(ret)
}

fn robject_as_response(obj: Rc<RObject>) -> Response {
    let vm = assume_init_vm();
    let status_code: u32 = {
        let status_obj = mrb_funcall(vm, obj.clone().into(), "status_code", &[]).unwrap();
        status_obj.as_ref().try_into().expect("Invalid status code")
    };
    let headers: Vec<_> = {
        let headers_obj = mrb_funcall(vm, obj.clone().into(), "headers", &[]).unwrap();
        headers_obj.as_ref().try_into().expect("Invalid headers")
    };
    let body = {
        let body_obj = mrb_funcall(vm, obj.clone().into(), "body", &[]).unwrap();
        let body_str: String = body_obj.as_ref().try_into().expect("Invalid body");
        body_str.into_bytes()
    };

    let mut builder = Response::builder();
    let mut response = builder.status(status_code as u16);
    for (key, value) in headers {
        let key: String = key.as_ref().try_into().expect("Invalid header key");
        let value: String = value.as_ref().try_into().expect("Invalid header value");
        response = response.header(&key, &value);
    }
    response.body(body).build()
}
