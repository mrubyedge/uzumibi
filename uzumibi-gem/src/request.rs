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

use crate::helpers;

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub query_string: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub params: HashMap<String, String>,
}

unsafe impl Send for Request {}
unsafe impl Sync for Request {}

const REQUEST_METHOD_KEY: &str = "method";
const REQUEST_PATH_KEY: &str = "path";
const REQUEST_HEADERS_KEY: &str = "headers";
const REQUEST_PARAMS_KEY: &str = "params";
const REQUEST_BODY_KEY: &str = "body";
const REQUEST_RAW_BODY_KEY: &str = "raw_body";

const REQUEST_METHOD_IVAR_KEY: &str = "@method";
const REQUEST_PATH_IVAR_KEY: &str = "@path";
const REQUEST_HEADERS_IVAR_KEY: &str = "@headers";
const REQUEST_PARAMS_IVAR_KEY: &str = "@params";
const REQUEST_BODY_IVAR_KEY: &str = "@body";
const REQUEST_RAW_BODY_IVAR_KEY: &str = "@raw_body";

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
    mrb_funcall(
        vm,
        Some(request_class.clone()),
        "attr_accessor",
        &[as_sym(REQUEST_BODY_KEY)],
    )
    .expect("attr_accessor failed");
    mrb_funcall(
        vm,
        Some(request_class.clone()),
        "attr_accessor",
        &[as_sym(REQUEST_RAW_BODY_KEY)],
    )
    .expect("attr_accessor failed");
}

fn as_sym(name: impl Into<String>) -> Rc<RObject> {
    let sym = RSym::new(name.into());
    RObject::symbol(sym).to_refcount_assigned()
}

impl Request {
    pub fn new_from_buffer(buf: &[u8]) -> Self {
        // Parse Method (6 bytes)
        let mut method = String::new();
        for &b in &buf[..6] {
            if b == 0 {
                break;
            }
            method.push(b as char);
        }
        let mut offset = 6;

        // Parse Path size (u16) + Path
        let path_size = u16::from_le_bytes([buf[offset], buf[offset + 1]]) as usize;
        offset += 2;
        let path: String = buf[offset..offset + path_size]
            .iter()
            .map(|&b| b as char)
            .collect();
        offset += path_size;

        // Parse Query String size (u16) + Query String
        let query_size = u16::from_le_bytes([buf[offset], buf[offset + 1]]) as usize;
        offset += 2;
        let query_string: String = buf[offset..offset + query_size]
            .iter()
            .map(|&b| b as char)
            .collect();
        offset += query_size;

        // Parse Headers count (u16) + Headers
        let headers_count = u16::from_le_bytes([buf[offset], buf[offset + 1]]) as usize;
        offset += 2;
        let mut headers = HashMap::new();
        for _ in 0..headers_count {
            let name_size = u16::from_le_bytes([buf[offset], buf[offset + 1]]) as usize;
            offset += 2;
            let name: String = buf[offset..offset + name_size]
                .iter()
                .map(|&b| b as char)
                .collect();
            offset += name_size;

            let value_size = u16::from_le_bytes([buf[offset], buf[offset + 1]]) as usize;
            offset += 2;
            let value: String = buf[offset..offset + value_size]
                .iter()
                .map(|&b| b as char)
                .collect();
            offset += value_size;

            headers.insert(name, value);
        }

        // Parse Request body size (u32) + Request body
        let body_size = u32::from_le_bytes([
            buf[offset],
            buf[offset + 1],
            buf[offset + 2],
            buf[offset + 3],
        ]) as usize;
        offset += 4;
        let body = buf[offset..offset + body_size].to_vec();

        Self {
            method,
            path,
            query_string,
            headers,
            body,
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
        let mut content_type: &'static str = "";
        for (key, value) in self.headers {
            if key.to_lowercase() == "content-type" {
                if value.to_lowercase() == "application/x-www-form-urlencoded" {
                    content_type = "application/x-www-form-urlencoded";
                } else if value.to_lowercase() == "application/json" {
                    content_type = "application/json";
                }
            }

            mrb_hash_set_index(
                headers_hash.clone(),
                RObject::string(key).to_refcount_assigned(),
                RObject::string(value).to_refcount_assigned(),
            )
            .expect("Failed to set header");
        }
        request_obj.set_ivar(REQUEST_HEADERS_IVAR_KEY, headers_hash);
        let params_hash = mrb_hash_new(vm, &[]).expect("Failed to create params hash");

        // Merge route params
        for (key, value) in self.params {
            mrb_hash_set_index(
                params_hash.clone(),
                RObject::symbol(RSym::new(key)).to_refcount_assigned(),
                RObject::string(value).to_refcount_assigned(),
            )
            .expect("Failed to set param");
        }

        // Parse and merge query string params
        if !self.query_string.is_empty() {
            for pair in self.query_string.split('&') {
                if let Some((key, value)) = pair.split_once('=') {
                    mrb_hash_set_index(
                        params_hash.clone(),
                        RObject::symbol(RSym::new(key.to_string())).to_refcount_assigned(),
                        RObject::string(value.to_string()).to_refcount_assigned(),
                    )
                    .expect("Failed to set query param");
                }
            }
        }

        let mut json_body = false;
        if !self.body.is_empty() {
            match content_type {
                "application/x-www-form-urlencoded" => {
                    for (key, value) in helpers::parse_x_www_form_urlencoded(&self.body) {
                        mrb_hash_set_index(
                            params_hash.clone(),
                            RObject::symbol(RSym::new(key)).to_refcount_assigned(),
                            RObject::string(value).to_refcount_assigned(),
                        )
                        .expect("Failed to set form param");
                    }
                }
                "application/json" => {
                    #[cfg(feature = "use-json")]
                    {
                        let body_rstr =
                            RObject::string_from_vec(self.body.clone()).to_refcount_assigned();
                        if let Ok(json_value) = mrubyedge_serde_json::mrb_json_class_load(vm, &[body_rstr])
                        {
                            // If json_value is a Hash, set key-value pairs to params
                            if let RValue::Hash(h) = &json_value.value {
                                let json_hash = h.borrow();
                                for (_, (key_obj, value_obj)) in json_hash.iter() {
                                    if let Ok(key) = TryInto::<String>::try_into(key_obj.as_ref()) {
                                        mrb_hash_set_index(
                                            params_hash.clone(),
                                            RObject::symbol(RSym::new(key)).to_refcount_assigned(),
                                            value_obj.clone(),
                                        )
                                        .expect("Failed to set json param");
                                    }
                                }
                            }

                            request_obj.set_ivar(REQUEST_BODY_IVAR_KEY, json_value);
                            json_body = true;
                        } else {
                            // Ignore JSON parse error
                        }
                    }
                }
                _ => {}
            }
        }
        // TODO: Parse json

        request_obj.set_ivar(REQUEST_PARAMS_IVAR_KEY, params_hash);
        let raw_body = RObject::string_from_vec(self.body).to_refcount_assigned();

        if !json_body {
            request_obj.set_ivar(REQUEST_BODY_IVAR_KEY, raw_body.clone());
        }
        request_obj.set_ivar(REQUEST_RAW_BODY_IVAR_KEY, raw_body);

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

        let body_obj = obj.get_ivar(REQUEST_BODY_IVAR_KEY);
        let body: Vec<u8> = match &body_obj.value {
            RValue::String(s, _) => s.borrow().to_vec(),
            RValue::Nil => Vec::new(),
            _ => {
                return Err(Error::RuntimeError("body must be a String".to_string()));
            }
        };

        Ok(Self {
            method,
            path,
            query_string: String::new(),
            headers,
            body,
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
