extern crate mrubyedge;
extern crate uzumibi_gem;

use std::{collections::HashMap, rc::Rc};

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response, body::Incoming as IncomingBody};
use mrubyedge::{
    error::StaticError,
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

pub(crate) fn uzumibi_handle_request(
    request: uzumibi_gem::request::Request,
) -> Result<Response<Full<Bytes>>, mrubyedge::error::StaticError> {
    let mut vm = init_vm();
    let app = vm
        .globals
        .get("$APP")
        .ok_or_else(|| {
            debug_console_log_internal("$APP is not defined");
            mrubyedge::error::StaticError::General("$APP is not defined".into())
        })?
        .clone();
    let request_robject = request.into_robject(&mut vm);
    mrb_funcall(
        &mut vm,
        Some(app.clone()),
        "set_request",
        &[request_robject],
    )?;
    let response_robject = mrb_funcall(&mut vm, Some(app.clone()), "start_request", &[])?;
    build_response_from_robject(&mut vm, response_robject)
}

pub(crate) fn build_uzumibi_request(
    request: &Request<IncomingBody>,
) -> uzumibi_gem::request::Request {
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
    let query_string = request.uri().query().unwrap_or_default().to_string();

    uzumibi_gem::request::Request {
        method,
        path,
        headers,
        query_string,
        body: Vec::new(),
        params: HashMap::new(),
    }
}

pub(crate) fn build_response_from_robject(
    vm: &mut VM,
    response: Rc<RObject>,
) -> Result<Response<Full<Bytes>>, mrubyedge::error::StaticError> {
    let status_code: u32 = {
        let status_obj = mrb_funcall(vm, response.clone().into(), "status_code", &[])?;
        status_obj.as_ref().try_into()?
    };
    let headers: Vec<_> = {
        let headers_obj = mrb_funcall(vm, response.clone().into(), "headers", &[])?;
        headers_obj.as_ref().try_into()?
    };
    let body = {
        let body_obj = mrb_funcall(vm, response.clone().into(), "body", &[])?;
        let body_str: String = body_obj.as_ref().try_into()?;
        body_str.into_bytes()
    };

    let builder = Response::builder();
    let mut response = builder.status(status_code as u16);
    for (key, value) in headers {
        let key: String = key.as_ref().try_into()?;
        let value: String = value.as_ref().try_into()?;
        response = response.header(&key, &value);
    }
    let res = response
        .body(Full::new(Bytes::from(body)))
        .map_err(|e| StaticError::General(format!("{}", e)))?;
    Ok(res)
}
