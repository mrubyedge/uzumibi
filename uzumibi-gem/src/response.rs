//! This module defines Uzumibi::Response class.
//! `init_uzumibi_response()` should be called on prelude process.
//! Signatures are as follows:
//!
//! ```rbs
//! @rbs!
//!   module Uzumibi
//!     class Response
//!       def status_code: Integer # u16
//!       def headers: Hash<String, String>
//!       def body: String
//!       def to_shared_memory() -> SharedMemory
//! ```
//!
use std::{collections::HashMap, rc::Rc};

use mrubyedge::{
    Error,
    yamrb::{
        helpers::{mrb_define_cmethod, mrb_funcall},
        prelude::{
            hash::{mrb_hash_new, mrb_hash_set_index},
            shared_memory::mrb_shared_memory_new,
        },
        value::{RObject, RSym, RValue},
        vm::VM,
    },
};

#[derive(Debug)]
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
    let response_class_ = vm.define_class("Response", None, Some(uzumibi_module));
    let response_class = RObject::class(response_class_.clone(), vm);

    mrb_funcall(
        vm,
        Some(response_class.clone()),
        "attr_accessor",
        &[as_sym(RESPONSE_STATUS_CODE_KEY)],
    )
    .expect("attr_accessor failed");
    mrb_funcall(
        vm,
        Some(response_class.clone()),
        "attr_accessor",
        &[as_sym(RESPONSE_HEADERS_KEY)],
    )
    .expect("attr_accessor failed");
    mrb_funcall(
        vm,
        Some(response_class.clone()),
        "attr_accessor",
        &[as_sym(RESPONSE_BODY_KEY)],
    )
    .expect("attr_accessor failed");

    mrb_define_cmethod(
        vm,
        response_class_.clone(),
        "to_shared_memory",
        Box::new(uzumibi_response_to_shared_memory),
    );
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
    match response_class.as_ref() {
        Some(response) if response.is_truthy() => {
            mrb_funcall(vm, Some(response.clone()), "new", &[])
                .expect("Failed to create Response instance")
        }
        _ => panic!("Response class must be defined beforehand"),
    }
}

fn uzumibi_response_to_shared_memory(
    vm: &mut VM,
    _args: &[Rc<RObject>],
) -> Result<Rc<RObject>, Error> {
    let response = vm.getself()?;
    let status_code: u32 = response
        .get_ivar(RESPONSE_STATUS_CODE_IVAR_KEY)
        .as_ref()
        .try_into()?;
    let headers: Vec<(Rc<RObject>, Rc<RObject>)> = response
        .get_ivar(RESPONSE_HEADERS_IVAR_KEY)
        .as_ref()
        .try_into()?;
    let body: String = response
        .get_ivar(RESPONSE_BODY_IVAR_KEY)
        .as_ref()
        .try_into()?;

    let mut buf: Vec<u8> = Vec::with_capacity(65536);
    let mut status_code_buf = [0u8; 2];
    status_code_buf[..2].copy_from_slice(&(status_code as u16).to_le_bytes()[..2]);
    buf.extend_from_slice(&status_code_buf);

    let headers_count = headers.len() as u16;
    let mut headers_count_buf = [0u8; 2];
    headers_count_buf[..2].copy_from_slice(&headers_count.to_le_bytes()[..2]);
    buf.extend_from_slice(&headers_count_buf);

    for (key, value) in headers.iter() {
        let key_str: String = key.as_ref().try_into()?;
        let value_str: String = value.as_ref().try_into()?;

        let key_bytes = key_str.as_bytes();
        let key_size = key_bytes.len() as u16;
        let mut key_size_buf = [0u8; 2];
        key_size_buf[..2].copy_from_slice(&key_size.to_le_bytes()[..2]);
        buf.extend_from_slice(&key_size_buf);
        buf.extend_from_slice(key_bytes);

        let value_bytes = value_str.as_bytes();
        let value_size = value_bytes.len() as u16;
        let mut value_size_buf = [0u8; 2];
        value_size_buf[..2].copy_from_slice(&value_size.to_le_bytes()[..2]);
        buf.extend_from_slice(&value_size_buf);
        buf.extend_from_slice(value_bytes);
    }

    let body_bytes = body.as_bytes();
    let body_size = body_bytes.len() as u32;
    let mut body_size_buf = [0u8; 4];
    body_size_buf[..4].copy_from_slice(&body_size.to_le_bytes()[..4]);
    buf.extend_from_slice(&body_size_buf);
    buf.extend_from_slice(body_bytes);

    let memory = mrb_shared_memory_new(vm, &[])?;
    let buf = RObject::string_from_vec(buf).to_refcount_assigned();
    mrb_funcall(vm, Some(memory.clone()), "replace", &[buf])?;
    Ok(memory)
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
