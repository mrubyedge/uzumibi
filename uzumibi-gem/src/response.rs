use std::{collections::HashMap, rc::Rc};

use mrubyedge::{
    Error,
    yamrb::{
        helpers::mrb_funcall,
        prelude::hash::{mrb_hash_new, mrb_hash_set_index},
        value::{RObject, RSym, RValue},
        vm::VM,
    },
};

pub struct Response {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

const RESPONSE_STATUS_CODE_KEY: &str = "status_code";
const RESPONSE_HEADERS_KEY: &str = "headers";
const RESPONSE_BODY_KEY: &str = "body";

const RESPONSE_STATUS_CODE_IVAR_KEY: &str = "@status_code";
const RESPONSE_HEADERS_IVAR_KEY: &str = "@headers";
const RESPONSE_BODY_IVAR_KEY: &str = "@body";

pub(crate) fn init_uzumibi_response(vm: &mut VM) {
    let uzumibi = vm
        .get_const_by_name("Uzumibi")
        .expect("Uzumibi module must be defined beforehand");
    let uzumibi_module = match &uzumibi.as_ref().value {
        RValue::Module(m) => m.clone(),
        _ => panic!("Uzumibi must be a module"),
    };
    let response_class = vm.define_class("Response", None, Some(uzumibi_module));
    let response_class = RObject::class(response_class, vm);

    mrb_funcall(
        vm,
        Some(response_class.clone()),
        "attr_writer",
        &[as_sym(RESPONSE_STATUS_CODE_KEY)],
    )
    .expect("attr_writer failed");
    mrb_funcall(
        vm,
        Some(response_class.clone()),
        "attr_writer",
        &[as_sym(RESPONSE_HEADERS_KEY)],
    )
    .expect("attr_writer failed");
    mrb_funcall(
        vm,
        Some(response_class.clone()),
        "attr_writer",
        &[as_sym(RESPONSE_BODY_KEY)],
    )
    .expect("attr_writer failed");
}

fn as_sym(name: impl Into<String>) -> Rc<RObject> {
    let sym = RSym::new(name.into());
    RObject::symbol(sym).to_refcount_assigned()
}

fn as_string(value: impl Into<String>) -> Rc<RObject> {
    RObject::string(value.into()).to_refcount_assigned()
}

pub(crate) fn uzumibi_response_new(vm: &mut VM) -> Rc<RObject> {
    let uzumibi = vm
        .get_const_by_name("Uzumibi")
        .expect("Uzumibi module must be defined beforehand");
    let uzumibi_module = match &uzumibi.as_ref().value {
        RValue::Module(m) => m.clone(),
        _ => panic!("Uzumibi must be a module"),
    };
    let response_class = uzumibi_module.get_const_by_name("Response");
    if response_class.is_falsy() {
        panic!("Response class must be defined beforehand");
    }
    mrb_funcall(vm, Some(response_class), "new", &[]).expect("Failed to create Response instance")
}

pub(crate) fn uzumibi_return_notfound(vm: &mut VM) -> Result<Rc<RObject>, Error> {
    let response = uzumibi_response_new(vm);
    response.set_ivar(
        RESPONSE_STATUS_CODE_IVAR_KEY,
        RObject::integer(404).to_refcount_assigned(),
    );
    let response_body = "Not Found";
    response.set_ivar(
        RESPONSE_BODY_IVAR_KEY,
        RObject::string(response_body.to_string()).to_refcount_assigned(),
    );

    let response_headers = mrb_hash_new(vm, &[])?;
    mrb_hash_set_index(
        response_headers.clone(),
        as_string("Content-Type"),
        as_string("text/plain; charset=utf-8"),
    )?;
    mrb_hash_set_index(
        response_headers.clone(),
        as_string("Content-Length"),
        as_string(response_body.len().to_string()),
    )?;
    mrb_hash_set_index(
        response_headers.clone(),
        as_string("Cache-Control"),
        as_string("no-cache"),
    )?;
    response.set_ivar(RESPONSE_HEADERS_IVAR_KEY, response_headers);

    Ok(response)
}
