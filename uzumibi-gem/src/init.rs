use std::rc::Rc;

use mrubyedge::{
    Error,
    yamrb::{
        helpers::{mrb_define_class_cmethod, mrb_define_cmethod, mrb_funcall},
        prelude::shared_memory::mrb_shared_memory_new,
        value::{RObject, RValue},
        vm::VM,
    },
};

use crate::{request::*, response::*};

extern crate mrubyedge;
extern crate uzumibi_art_router;

///
/// init_uzumibi() defines Uzumibi module and Router class.
/// Signatures are as follows:
///
/// ```rbs
/// @rbs!
///   module Uzumibi
///     class Router
///       def self.routes() -> Hash
///       def self.get(path: String, handler: Proc) -> String
///       def initialize_request(size: Integer) -> SharedMemory
///       def set_request(request: Request)
///       def start_request() -> Response
/// ```
///
pub fn init_uzumibi(vm: &mut VM) {
    let uzumibi = vm.define_module("Uzumibi", None);
    let router_class = vm.define_class("Router", None, Some(uzumibi.clone()));

    mrb_define_class_cmethod(
        vm,
        router_class.clone(),
        "routes",
        Box::new(uzumibi_router_routes),
    );
    // FIXME: other http methods...
    mrb_define_class_cmethod(
        vm,
        router_class.clone(),
        "get",
        Box::new(uzumibi_router_set_route),
    );

    mrb_define_cmethod(
        vm,
        router_class.clone(),
        "initialize_request",
        Box::new(uzumibi_initialize_request),
    );

    mrb_define_cmethod(
        vm,
        router_class.clone(),
        "set_request",
        Box::new(uzumibi_set_request),
    );
    mrb_define_cmethod(
        vm,
        router_class.clone(),
        "start_request",
        Box::new(uzumibi_start_request),
    );
    mrb_define_cmethod(
        vm,
        router_class,
        "start_request_and_return_shared_memory",
        Box::new(uzumibi_start_request_and_return_shared_memory),
    );

    init_uzumibi_response(vm);
    init_uzumibi_request(vm);

    uzumibi_art_router::init_uzumibi_art_router(vm);
}

const ROUTES_KEY: &str = "@_art_router";
const REQUEST_KEY: &str = "@_request";
const REQUEST_BUF_KEY: &str = "@_request_buf";

fn uzumibi_router_routes(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let klass = vm.getself()?;
    let router = if klass.get_ivar(ROUTES_KEY).is_falsy() {
        // Create an ArtRouter instance
        let uzumibi = vm
            .get_const_by_name("Uzumibi")
            .ok_or_else(|| Error::RuntimeError("Uzumibi module not found".to_string()))?;
        let uzumibi_module = match &uzumibi.as_ref().value {
            RValue::Module(m) => m.clone(),
            _ => return Err(Error::RuntimeError("Uzumibi must be a module".to_string())),
        };
        let art_router_class = uzumibi_module
            .get_const_by_name("ArtRouter")
            .ok_or_else(|| Error::RuntimeError("ArtRouter class not found".to_string()))?;
        let router = mrb_funcall(vm, Some(art_router_class), "new", &[])?;
        klass.set_ivar(ROUTES_KEY, router.clone());
        router
    } else {
        klass.get_ivar(ROUTES_KEY)
    };

    Ok(router)
}

fn uzumibi_router_set_route(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    if args.len() != 2 || args[args.len() - 1].is_falsy() {
        return Err(Error::ArgumentError(
            "Expected 2 arguments: path, handler".to_string(),
        ));
    }
    let art_router = uzumibi_router_routes(vm, &[])?;
    let path = args[0].clone();
    let handler = args[1].clone();

    // Call ArtRouter's set_route method
    mrb_funcall(vm, Some(art_router), "set_route", &[path.clone(), handler])?;

    Ok(path)
}

fn uzumibi_initialize_request(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let shared_memory = mrb_shared_memory_new(vm, args)?;
    vm.getself()?
        .set_ivar(REQUEST_BUF_KEY, shared_memory.clone());
    Ok(shared_memory)
}

fn uzumibi_set_request(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let request_obj = args
        .first()
        .ok_or_else(|| Error::ArgumentError("Expected 1 argument: request object".to_string()))?;
    vm.getself()?.set_ivar(REQUEST_KEY, request_obj.clone());
    Ok(RObject::nil().to_refcount_assigned())
}

fn uzumibi_start_request(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let app = vm.getself()?;
    let request_obj = app.get_ivar(REQUEST_KEY);
    let mut request = match &request_obj.value {
        RValue::Nil => {
            let request_buf = vm.getself()?.get_ivar(REQUEST_BUF_KEY);
            uzumibi_construct_request(request_buf)?
        }
        RValue::Instance(_) => Request::from_robject(vm, request_obj.clone())?,
        _ => {
            return Err(Error::ArgumentError("Invalid request object".to_string()));
        }
    };

    let self_class = mrb_funcall(vm, app.into(), "class", &[])?;
    let art_router = self_class.get_ivar(ROUTES_KEY);
    if art_router.is_falsy() {
        return Err(Error::RuntimeError("Router is not initialized".to_string()));
    }

    // Get route and params from ArtRouter's get_route method
    let path_obj = RObject::string(request.path.clone()).to_refcount_assigned();
    let result = mrb_funcall(vm, Some(art_router), "get_route", &[path_obj])?;

    // result is an array [route, params] or an empty array
    match &result.value {
        RValue::Array(arr) => {
            let arr = arr.borrow();
            if arr.len() == 2 {
                let route = arr[0].clone();
                let params_hash = arr[1].clone();

                // Merge params into request
                if let RValue::Hash(h) = &params_hash.value {
                    let params_h = h.borrow();
                    for (_, (key_obj, value_obj)) in params_h.iter() {
                        let key: String = key_obj.as_ref().try_into()?;
                        let value: String = value_obj.as_ref().try_into()?;
                        request.params.insert(key, value);
                    }
                }

                let request = request.into_robject(vm);
                let response = uzumibi_response_new(vm);

                mrb_funcall(vm, Some(route), "call", &[request, response.clone()])?;

                Ok(response)
            } else {
                // Route not found
                uzumibi_return_notfound(vm)
            }
        }
        _ => uzumibi_return_notfound(vm),
    }
}

fn uzumibi_start_request_and_return_shared_memory(
    vm: &mut VM,
    _args: &[Rc<RObject>],
) -> Result<Rc<RObject>, Error> {
    let response = uzumibi_start_request(vm, &[])?;
    let response_sm = mrb_funcall(vm, response.into(), "to_shared_memory", &[])?;
    Ok(response_sm)
}

#[allow(dead_code)]
fn uzumibi_class(vm: &mut VM) -> Rc<RObject> {
    vm.get_const_by_name("Uzumibi").unwrap()
}

fn uzumibi_construct_request(request_buf: Rc<RObject>) -> Result<Request, Error> {
    let sm = match &request_buf.value {
        RValue::SharedMemory(sm) => Ok(sm.clone()),
        _ => Err(Error::ArgumentError(
            "request buffer must be SharedMemory".to_string(),
        )),
    }?;
    let sm = sm.borrow();
    let buf = sm.memory.as_ref();
    let request = Request::new_from_buffer(&buf);

    Ok(request)
}
