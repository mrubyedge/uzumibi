#![allow(static_mut_refs)]
extern crate mrubyedge;
extern crate uzumibi_gem;

use std::{collections::HashMap, rc::Rc};

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{body::Incoming as IncomingBody, Request, Response};
use mrubyedge::{
    rite::rite,
    yamrb::{
        helpers::{mrb_define_cmethod, mrb_funcall},
        value::RObject,
        vm::VM,
    },
};

static MRB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/app.mrb"));

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

pub fn uzumibi_handle_request(
    request: &Request<IncomingBody>,
) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let mut vm = init_vm();
    let app = vm
        .globals
        .get("$APP")
        .or_else(|| {
            debug_console_log_internal("$APP is not defined");
            None
        })
        .unwrap()
        .clone();

    let request_robject = build_request_as_robject(&mut vm, request);
    mrb_funcall(
        &mut vm,
        Some(app.clone()),
        "set_request",
        &[request_robject],
    )
    .map_err(|e| e.to_string())?;
    let response_robject =
        mrb_funcall(&mut vm, Some(app.clone()), "start_request", &[]).map_err(|e| e.to_string())?;
    build_response_from_robject(&mut vm, response_robject)
}

pub fn build_request_as_robject(vm: &mut VM, request: &Request<IncomingBody>) -> Rc<RObject> {
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let headers = request
        .headers()
        .iter()
        .map(|(k, v)| {
            let value_str = v.to_str().unwrap_or_default();
            (k.as_str().to_string(), value_str.to_string())
        })
        .collect::<HashMap<String, String>>();

    let request = uzumibi_gem::request::Request {
        method,
        path,
        headers,
    };

    request.into_robject(vm)
}

pub fn build_response_from_robject(
    vm: &mut VM,
    response: Rc<RObject>,
) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let status_code: u32 = {
        let status_obj = mrb_funcall(vm, response.clone().into(), "status_code", &[])
            .map_err(|e| e.to_string())?;
        status_obj
            .as_ref()
            .try_into()
            .map_err(|e: mrubyedge::Error| e.to_string())?
    };
    let headers: Vec<_> = {
        let headers_obj =
            mrb_funcall(vm, response.clone().into(), "headers", &[]).map_err(|e| e.to_string())?;
        headers_obj
            .as_ref()
            .try_into()
            .map_err(|e: mrubyedge::Error| e.to_string())?
    };
    let body = {
        let body_obj =
            mrb_funcall(vm, response.clone().into(), "body", &[]).map_err(|e| e.to_string())?;
        let body_str: String = body_obj
            .as_ref()
            .try_into()
            .map_err(|e: mrubyedge::Error| e.to_string())?;
        body_str.into_bytes()
    };

    let builder = Response::builder();
    let mut response = builder.status(status_code as u16);
    for (key, value) in headers {
        let key: String = key
            .as_ref()
            .try_into()
            .map_err(|e: mrubyedge::Error| e.to_string())?;
        let value: String = value
            .as_ref()
            .try_into()
            .map_err(|e: mrubyedge::Error| e.to_string())?;
        response = response.header(&key, &value);
    }
    let res = response.body(Full::new(Bytes::from(body)))?;
    Ok(res)
}
