use std::rc::Rc;

use mrubyedge::{
    Error,
    yamrb::{
        helpers::{mrb_define_class_cmethod, mrb_define_cmethod, mrb_funcall},
        prelude::{
            hash::{mrb_hash_get_index, mrb_hash_new, mrb_hash_set_index},
            shared_memory::mrb_shared_memory_new,
        },
        value::{RObject, RValue},
        vm::VM,
    },
};

use crate::{request::*, response::*};

extern crate mrubyedge;

///
/// init_uzumibi() defines Uzumibi module and Router class.
/// Signatures are as follows:
///
/// ```rbs
/// @rbs!
///   module Uzumibi
///     class Router
///       def self.routes() -> Hash
///       def self.get(path, handler) -> path
///       def initialize_request(size) -> SharedMemory
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
        "start_request",
        Box::new(uzumibi_start_request),
    );

    init_uzumibi_response(vm);
    init_uzumibi_request(vm);
}

const ROUTES_KEY: &str = "@_routes";
const REQUEST_BUF_KEY: &str = "@_request_buf";

fn uzumibi_router_routes(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let klass = vm.getself()?;
    let routes = if klass.get_ivar(ROUTES_KEY).is_falsy() {
        let hash = mrb_hash_new(vm, &[RObject::integer(0).to_refcount_assigned()])?;
        klass.set_ivar(ROUTES_KEY, hash.clone());
        hash
    } else {
        klass.get_ivar(ROUTES_KEY)
    };

    Ok(routes)
}

fn uzumibi_router_set_route(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    if args.len() != 2 || args[args.len() - 1].is_falsy() {
        return Err(Error::ArgumentError(
            "Expected 2 arguments: path, handler".to_string(),
        ));
    }
    let routes = uzumibi_router_routes(vm, &[])?;
    let path = args[0].clone();
    let handler = args[1].clone();

    mrb_hash_set_index(routes, path.clone(), handler.clone())?;

    Ok(path)
}

fn uzumibi_initialize_request(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let shared_memory = mrb_shared_memory_new(vm, args)?;
    shared_memory.set_ivar(REQUEST_BUF_KEY, shared_memory.clone());
    Ok(shared_memory)
}

fn uzumibi_start_request(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let request_buf = vm.getself()?.get_ivar(REQUEST_BUF_KEY);
    let request = uzumibi_construct_request(request_buf)?;

    let uzumibi = uzumibi_class(vm);
    let router_hash = uzumibi.get_ivar(ROUTES_KEY);
    if router_hash.is_falsy() {
        return Err(Error::RuntimeError("Router is not initialized".to_string()));
    }

    let key = RObject::string(request.path.clone()).to_refcount_assigned();
    let route = mrb_hash_get_index(router_hash, key)?;
    if matches!(route.as_ref().value, RValue::Proc(_)) {
        let request = request.into_robject(vm);
        let response = uzumibi_response_new(vm);

        mrb_funcall(vm, Some(route), "call", &[request, response.clone()])?;

        Ok(response)
    } else {
        uzumibi_return_notfound(vm)
    }
}

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
