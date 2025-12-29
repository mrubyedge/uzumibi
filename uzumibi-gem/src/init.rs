use std::rc::Rc;

use mrubyedge::{
    Error,
    yamrb::{
        helpers::mrb_define_class_cmethod,
        prelude::hash::{mrb_hash_new, mrb_hash_set_index},
        value::RObject,
        vm::VM,
    },
};

extern crate mrubyedge;

pub fn init_uzumibi(vm: &mut VM) {
    let uzumibi = vm.define_module("Uzumibi", None);
    let router_class = vm.define_class("Router", None, Some(uzumibi.clone()));

    mrb_define_class_cmethod(
        vm,
        router_class.clone(),
        "routes",
        Box::new(uzumibi_router_routes),
    );
    // FIXME: other methods...
    mrb_define_class_cmethod(
        vm,
        router_class.clone(),
        "get",
        Box::new(uzumibi_router_set_route),
    );
}

const ROUTES_KEY: &str = "@_routes";

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
