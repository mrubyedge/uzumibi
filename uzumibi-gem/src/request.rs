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

use mrubyedge::{
    Error,
    yamrb::{
        helpers::mrb_funcall,
        prelude::hash::{mrb_hash_new, mrb_hash_set_index},
        value::{RObject, RSym, RValue},
        vm::VM,
    },
};

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub params: HashMap<String, String>,
}

const REQUEST_METHOD_KEY: &str = "method";
const REQUEST_PATH_KEY: &str = "path";
const REQUEST_HEADERS_KEY: &str = "headers";
const REQUEST_PARAMS_KEY: &str = "params";

const REQUEST_METHOD_IVAR_KEY: &str = "@method";
const REQUEST_PATH_IVAR_KEY: &str = "@path";
const REQUEST_HEADERS_IVAR_KEY: &str = "@headers";
const REQUEST_PARAMS_IVAR_KEY: &str = "@params";

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
        "attr_accessor",
        &[as_sym(REQUEST_METHOD_KEY)],
    )
    .expect("attr_accessor failed");
    mrb_funcall(
        vm,
        Some(request_class.clone()),
        "attr_accessor",
        &[as_sym(REQUEST_PATH_KEY)],
    )
    .expect("attr_accessor failed");
    mrb_funcall(
        vm,
        Some(request_class.clone()),
        "attr_accessor",
        &[as_sym(REQUEST_HEADERS_KEY)],
    )
    .expect("attr_accessor failed");
    mrb_funcall(
        vm,
        Some(request_class.clone()),
        "attr_accessor",
        &[as_sym(REQUEST_PARAMS_KEY)],
    )
    .expect("attr_accessor failed");
}

fn as_sym(name: impl Into<String>) -> Rc<RObject> {
    let sym = RSym::new(name.into());
    RObject::symbol(sym).to_refcount_assigned()
}

impl Request {
    pub fn new_from_buffer(buf: &[u8]) -> Self {
        let mut method = String::new();
        for &b in &buf[..6] {
            if b == 0 {
                break;
            }
            method.push(b as char);
        }
        let buf = &buf[6..];
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

        let headers_data = buf;
        let mut pos = 0;
        for _ in 0..headers_size {
            let name_size = u16::from_le_bytes([headers_data[pos], headers_data[pos + 1]]) as usize;
            pos += 2;
            let name: String = headers_data[pos..pos + name_size]
                .iter()
                .map(|&b| b as char)
                .collect();
            pos += name_size;
            let value_size =
                u16::from_le_bytes([headers_data[pos], headers_data[pos + 1]]) as usize;
            pos += 2;
            let value: String = headers_data[pos..pos + value_size]
                .iter()
                .map(|&b| b as char)
                .collect();
            pos += value_size;

            headers.insert(name, value);
        }

        Self {
            method,
            path,
            headers,
            params: HashMap::new(),
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
        let params_hash = mrb_hash_new(vm, &[]).expect("Failed to create params hash");
        for (key, value) in self.params {
            mrb_hash_set_index(
                params_hash.clone(),
                RObject::symbol(RSym::new(key)).to_refcount_assigned(),
                RObject::string(value).to_refcount_assigned(),
            )
            .expect("Failed to set param");
        }
        request_obj.set_ivar(REQUEST_PARAMS_IVAR_KEY, params_hash);

        request_obj
    }

    pub fn from_robject(_vm: &mut VM, obj: Rc<RObject>) -> Result<Self, Error> {
        let method_obj = obj.get_ivar(REQUEST_METHOD_IVAR_KEY);
        let method: String = method_obj.as_ref().try_into()?;

        let path_obj = obj.get_ivar(REQUEST_PATH_IVAR_KEY);
        let path: String = path_obj.as_ref().try_into()?;

        let headers_obj = obj.get_ivar(REQUEST_HEADERS_IVAR_KEY);
        let mut headers = HashMap::new();
        match &headers_obj.value {
            RValue::Hash(h) => {
                let headers_hash = h.borrow();
                for (_, (key_obj, value_obj)) in headers_hash.iter() {
                    let key: String = key_obj.as_ref().try_into()?;
                    let value: String = value_obj.as_ref().try_into()?;
                    headers.insert(key, value);
                }
            }
            _ => {
                return Err(Error::RuntimeError("headers must be a Hash".to_string()));
            }
        };

        let params_obj = obj.get_ivar(REQUEST_PARAMS_IVAR_KEY);
        let mut params = HashMap::new();
        match &params_obj.value {
            RValue::Hash(h) => {
                let params_hash = h.borrow();
                for (_, (key_obj, value_obj)) in params_hash.iter() {
                    let key: String = key_obj.as_ref().try_into()?;
                    let value: String = value_obj.as_ref().try_into()?;
                    params.insert(key, value);
                }
            }
            RValue::Nil => {}
            _ => {
                return Err(Error::RuntimeError("params must be a Hash".to_string()));
            }
        };

        Ok(Self {
            method,
            path,
            headers,
            params,
        })
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
