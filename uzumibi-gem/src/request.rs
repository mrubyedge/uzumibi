//! This module defines Uzumibi::Request class.
//! `init_uzumibi_request()` defines internally.
//! Signatures are as follows:
//!
//! ```rbs
//! @rbs!
//!   module Uzumibi
//!     class Request
//!       def method: String
//!       def path: String
//!       def headers: Hash<String, String>
//! ```
//!
use std::{collections::HashMap, rc::Rc};

use mrubyedge::yamrb::{
    helpers::mrb_funcall,
    prelude::hash::{mrb_hash_new, mrb_hash_set_index},
    value::{RObject, RSym, RValue},
    vm::VM,
};

pub struct Request {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
}

const REQUEST_METHOD_KEY: &str = "method";
const REQUEST_PATH_KEY: &str = "path";
const REQUEST_HEADERS_KEY: &str = "headers";

const REQUEST_METHOD_IVAR_KEY: &str = "@method";
const REQUEST_PATH_IVAR_KEY: &str = "@path";
const REQUEST_HEADERS_IVAR_KEY: &str = "@headers";

pub(crate) fn init_uzumibi_request(vm: &mut VM) {
    let uzumibi = vm
        .get_const_by_name("Uzumibi")
        .expect("Uzumibi module must be defined beforehand");
    let uzumibi_module = match &uzumibi.as_ref().value {
        RValue::Module(m) => m.clone(),
        _ => panic!("Uzumibi must be a module"),
    };
    let request_class = vm.define_class("Request", None, Some(uzumibi_module));
    let request_class = RObject::class(request_class, vm);

    mrb_funcall(
        vm,
        Some(request_class.clone()),
        "attr_writer",
        &[as_sym(REQUEST_METHOD_KEY)],
    )
    .expect("attr_writer failed");
    mrb_funcall(
        vm,
        Some(request_class.clone()),
        "attr_writer",
        &[as_sym(REQUEST_PATH_KEY)],
    )
    .expect("attr_writer failed");
    mrb_funcall(
        vm,
        Some(request_class.clone()),
        "attr_writer",
        &[as_sym(REQUEST_HEADERS_KEY)],
    )
    .expect("attr_writer failed");
}

fn as_sym(name: impl Into<String>) -> Rc<RObject> {
    let sym = RSym::new(name.into());
    RObject::symbol(sym).to_refcount_assigned()
}

impl Request {
    pub fn new_from_buffer(buf: &[u8]) -> Self {
        let method: String = buf[0..4].iter().map(|&b| b as char).collect();
        let buf = &buf[4..];
        let path_size = u16::from_le_bytes([buf[0], buf[1]]);
        let buf = &buf[2..];
        let path: String = buf[0..path_size as usize]
            .iter()
            .map(|&b| b as char)
            .collect();
        let buf = &buf[path_size as usize..];

        let headers_size = u16::from_le_bytes([buf[0], buf[1]]);
        let buf = &buf[2..];
        let mut headers = HashMap::new();

        let headers_data = &buf[0..headers_size as usize];
        let mut pos = 0;
        while pos < headers_data.len() {
            // Read header name until \0
            let name_end = headers_data[pos..]
                .iter()
                .position(|&b| b == 0)
                .unwrap_or(headers_data.len() - pos);
            let name = String::from_utf8_lossy(&headers_data[pos..pos + name_end]).to_string();
            pos += name_end + 1; // Skip \0

            if pos >= headers_data.len() {
                break;
            }

            // Read header value until \0
            let value_end = headers_data[pos..]
                .iter()
                .position(|&b| b == 0)
                .unwrap_or(headers_data.len() - pos);
            let value = String::from_utf8_lossy(&headers_data[pos..pos + value_end]).to_string();
            pos += value_end + 1; // Skip \0

            headers.insert(name, value);
        }

        Self {
            method,
            path,
            headers,
        }
    }

    pub fn into_robject(self, vm: &mut VM) -> Rc<RObject> {
        let request_obj = uzumibi_request_new(vm);

        request_obj.set_ivar(
            REQUEST_METHOD_IVAR_KEY,
            RObject::string(self.method).to_refcount_assigned(),
        );
        request_obj.set_ivar(
            REQUEST_PATH_IVAR_KEY,
            RObject::string(self.path).to_refcount_assigned(),
        );
        let headers_hash = mrb_hash_new(vm, &[]).expect("Failed to create headers hash");
        for (key, value) in self.headers {
            mrb_hash_set_index(
                headers_hash.clone(),
                RObject::string(key).to_refcount_assigned(),
                RObject::string(value).to_refcount_assigned(),
            )
            .expect("Failed to set header");
        }
        request_obj.set_ivar(REQUEST_HEADERS_IVAR_KEY, headers_hash);

        request_obj
    }
}

pub(crate) fn uzumibi_request_new(vm: &mut VM) -> Rc<RObject> {
    let uzumibi = vm
        .get_const_by_name("Uzumibi")
        .expect("Uzumibi module must be defined beforehand");
    let uzumibi_module = match &uzumibi.as_ref().value {
        RValue::Module(m) => m.clone(),
        _ => panic!("Uzumibi must be a module"),
    };
    let request_class = uzumibi_module.get_const_by_name("Request");
    match request_class.as_ref() {
        Some(request) if request.is_truthy() => mrb_funcall(vm, Some(request.clone()), "new", &[])
            .expect("Failed to create Request instance"),
        _ => panic!("Request class must be defined beforehand"),
    }
}
