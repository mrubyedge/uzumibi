pub mod store;
pub mod vendor_art_tree;

use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

use mrubyedge::{
    Error,
    yamrb::{
        helpers::{mrb_define_class_cmethod, mrb_define_cmethod},
        prelude::hash::{mrb_hash_new, mrb_hash_set_index},
        value::{RClass, RData, RObject, RSym, RType, RValue},
        vm::VM,
    },
};

const UNSET_OBJECT_ID: u64 = u64::MAX;

fn get_uzumibi_art_router_class(vm: &mut VM) -> Rc<RClass> {
    let uzumibi = vm
        .get_const_by_name("Uzumibi")
        .expect("Uzumibi module must be defined beforehand");
    let uzumibi_module = match &uzumibi.as_ref().value {
        RValue::Module(m) => m.clone(),
        _ => panic!("Uzumibi must be a module"),
    };
    let class = uzumibi_module
        .get_const_by_name("ArtRouter")
        .expect("ArtRouter class must be defined");
    match &class.as_ref().value {
        RValue::Class(c) => c.clone(),
        _ => panic!("ArtRouter must be a class"),
    }
}

pub fn init_uzumibi_art_router(vm: &mut VM) {
    let uzumibi = vm
        .get_const_by_name("Uzumibi")
        .expect("Uzumibi module must be defined beforehand");
    let uzumibi_module = match &uzumibi.as_ref().value {
        RValue::Module(m) => m.clone(),
        _ => panic!("Uzumibi must be a module"),
    };
    let art_router_class = vm.define_class("ArtRouter", None, Some(uzumibi_module));

    mrb_define_class_cmethod(
        vm,
        art_router_class.clone(),
        "new",
        Box::new(uzumibi_art_router_new),
    );
    mrb_define_cmethod(
        vm,
        art_router_class.clone(),
        "set_route",
        Box::new(uzumibi_art_router_set_route),
    );
    mrb_define_cmethod(
        vm,
        art_router_class.clone(),
        "get_route",
        Box::new(uzumibi_art_router_get_route),
    );
}

fn uzumibi_art_router_new(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let store: store::RouteStore<Rc<RObject>> = store::RouteStore::new();
    let class = get_uzumibi_art_router_class(vm);
    let data = RData {
        class,
        data: RefCell::new(Some(Rc::new(Box::new(store)))),
        ref_count: 0,
    };
    let instance = RObject {
        tt: RType::Data,
        value: RValue::Data(Rc::new(data)),
        object_id: Cell::new(UNSET_OBJECT_ID),
        singleton_class: RefCell::new(None),
        ivar: RefCell::new(HashMap::new()),
    }
    .to_refcount_assigned();

    Ok(instance)
}

fn uzumibi_art_router_set_route(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let path_obj = match args.first() {
        Some(p) if p.is_truthy() => p.clone(),
        _ => {
            return Err(Error::ArgumentError(
                "Path argument is required".to_string(),
            ));
        }
    };
    let path: String = path_obj.as_ref().try_into()?;
    let route_proc = match args.get(1) {
        Some(proc) if proc.is_truthy() => proc.clone(),
        _ => {
            return Err(Error::ArgumentError(
                "A block must be given to set route".to_string(),
            ));
        }
    };
    let self_obj = vm.getself()?;
    let data = match &self_obj.value {
        RValue::Data(d) => d,
        _ => {
            return Err(Error::RuntimeError(
                "ArtRouter instance must have data".to_string(),
            ));
        }
    };
    let borrowed = data.data.borrow();
    let store_any = borrowed
        .as_ref()
        .ok_or_else(|| Error::RuntimeError("RouteStore is already taken".to_string()))?;
    let store = store_any
        .downcast_ref::<store::RouteStore<Rc<RObject>>>()
        .ok_or_else(|| Error::RuntimeError("Failed to downcast RouteStore".to_string()))?;
    store.insert(&path, store::Route::Handler(route_proc));

    Ok(RObject::boolean(true).to_refcount_assigned())
}

fn uzumibi_art_router_get_route(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let path_obj = match args.first() {
        Some(p) if p.is_truthy() => p.clone(),
        _ => {
            return Err(Error::ArgumentError(
                "Path argument is required".to_string(),
            ));
        }
    };
    let path: String = path_obj.as_ref().try_into()?;
    let self_obj = vm.getself()?;
    let data = match &self_obj.value {
        RValue::Data(d) => d,
        _ => {
            return Err(Error::RuntimeError(
                "ArtRouter instance must have data".to_string(),
            ));
        }
    };
    let borrowed = data.data.borrow();
    let store_any = borrowed
        .as_ref()
        .ok_or_else(|| Error::RuntimeError("RouteStore is already taken".to_string()))?;
    let store = store_any
        .downcast_ref::<store::RouteStore<Rc<RObject>>>()
        .ok_or_else(|| Error::RuntimeError("Failed to downcast RouteStore".to_string()))?;
    if let (Some(route), params) = store.get_with_params(&path) {
        match &route.value {
            RValue::Proc(_) => {
                let hash = mrb_hash_new(vm, &[])?;
                for (k, v) in params.iter() {
                    let key = RObject::symbol(RSym::new(k.to_owned())).to_refcount_assigned();
                    let value = RObject::string(v.to_owned()).to_refcount_assigned();
                    mrb_hash_set_index(hash.clone(), key, value)?;
                }
                Ok(RObject::array(vec![route.clone(), hash]).to_refcount_assigned())
            }
            _ => Err(Error::RuntimeError(
                "Route is not a callable Proc".to_string(),
            )),
        }
    } else {
        Ok(RObject::array(vec![]).to_refcount_assigned())
    }
}
