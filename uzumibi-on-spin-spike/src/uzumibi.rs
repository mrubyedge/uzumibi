#![allow(static_mut_refs)]
extern crate mrubyedge;
extern crate spin_sdk;
extern crate uzumibi_gem;

use std::{collections::HashMap, mem::MaybeUninit, rc::Rc};

use mrubyedge::{
    rite::rite,
    yamrb::{
        helpers::{mrb_define_cmethod, mrb_funcall},
        value::RObject,
        vm::VM,
    },
};
use spin_sdk::http::{Request, Response};

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

pub fn uzumibi_initialize_request(request: Request) -> Result<(), mrubyedge::Error> {
    let vm = assume_init_vm();
    let method = request.method().to_string();
    let path = request.path().to_string();
    let headers = request
        .headers()
        .map(|(k, v)| {
            let value_str = v.as_str().unwrap_or("");
            (k.to_string(), value_str.to_string())
        })
        .collect::<HashMap<String, String>>();
    let query_string = request.query().to_string();
    let body = request.into_body();

    let request = uzumibi_gem::request::Request {
        method,
        path,
        headers,
        query_string,
        body,
        params: HashMap::new(),
    };

    let app = vm
        .globals
        .get("$APP")
        .cloned()
        .ok_or_else(|| mrubyedge::Error::RuntimeError("Failed to get $APP".to_string()))?;
    let request = request.into_robject(vm);
    mrb_funcall(vm, Some(app.clone()), "set_request", &[request])?;
    Ok(())
}

pub fn uzumibi_start_request() -> Result<Response, mrubyedge::Error> {
    let vm = assume_init_vm();
    let app = vm
        .globals
        .get("$APP")
        .cloned()
        .ok_or_else(|| mrubyedge::Error::RuntimeError("Failed to get $APP".to_string()))?;
    let ret = mrb_funcall(vm, app.clone().into(), "start_request", &[]).map_err(|e| {
        debug_console_log_internal(&format!("Error in start_request: {}", e));
        e
    })?;
    robject_as_response(ret)
}

fn robject_as_response(obj: Rc<RObject>) -> Result<Response, mrubyedge::Error> {
    let vm = assume_init_vm();
    let status_code: u32 = {
        let status_obj = mrb_funcall(vm, obj.clone().into(), "status_code", &[])?;
        status_obj.as_ref().try_into()?
    };
    let headers: Vec<_> = {
        let headers_obj = mrb_funcall(vm, obj.clone().into(), "headers", &[])?;
        headers_obj.as_ref().try_into()?
    };
    let body = {
        let body_obj = mrb_funcall(vm, obj.clone().into(), "body", &[])?;
        let body_str: String = body_obj.as_ref().try_into()?;
        body_str.into_bytes()
    };

    let mut builder = Response::builder();
    let mut response = builder.status(status_code as u16);
    for (key, value) in headers {
        let key: String = key.as_ref().try_into()?;
        let value: String = value.as_ref().try_into()?;
        response = response.header(&key, &value);
    }
    Ok(response.body(body).build())
}
