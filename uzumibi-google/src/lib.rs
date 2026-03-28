pub mod firestore;
pub mod meta;

use std::rc::Rc;

use mrubyedge::{
    Error,
    yamrb::{
        helpers::{mrb_define_class_cmethod, mrb_define_module_cmethod},
        value::RObject,
        vm::VM,
    },
};

const TOKEN_IVAR_KEY: &str = "@token";
const PROJECT_ID_IVAR_KEY: &str = "@project_id";

/// init_google() defines Uzumibi::Google module and Uzumibi::KV class.
///
/// ```rbs
/// @rbs!
///   module Uzumibi
///     module Google
///       def self.fetch_token() -> String
///       def self.token() -> String
///       def self.token=(value: String) -> String
///       def self.fetch_project_id() -> String
///       def self.project_id() -> String
///       def self.project_id=(value: String) -> String
///     end
///     class KV
///       def self.get(key: String) -> String?
///       def self.set(key: String, value: String) -> bool
///     end
///   end
/// ```
pub fn init_google(vm: &mut VM) {
    let uzumibi = vm.define_module("Uzumibi", None);

    // Uzumibi::Google module
    let google_mod = vm.define_module("Google", Some(uzumibi.clone()));

    mrb_define_module_cmethod(
        vm,
        google_mod.clone(),
        "fetch_token",
        Box::new(uzumibi_google_fetch_token),
    );
    mrb_define_module_cmethod(
        vm,
        google_mod.clone(),
        "token",
        Box::new(uzumibi_google_token),
    );
    mrb_define_module_cmethod(
        vm,
        google_mod.clone(),
        "token=",
        Box::new(uzumibi_google_set_token),
    );
    mrb_define_module_cmethod(
        vm,
        google_mod.clone(),
        "fetch_project_id",
        Box::new(uzumibi_google_fetch_project_id),
    );
    mrb_define_module_cmethod(
        vm,
        google_mod.clone(),
        "project_id",
        Box::new(uzumibi_google_project_id),
    );
    mrb_define_module_cmethod(
        vm,
        google_mod.clone(),
        "project_id=",
        Box::new(uzumibi_google_set_project_id),
    );

    // Uzumibi::KV class
    let kv_class = vm.define_class("KV", None, Some(uzumibi.clone()));

    mrb_define_class_cmethod(vm, kv_class.clone(), "get", Box::new(uzumibi_kv_get));
    mrb_define_class_cmethod(vm, kv_class.clone(), "set", Box::new(uzumibi_kv_set));
}

// --- Uzumibi::Google methods ---

fn uzumibi_google_fetch_token(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let token = meta::get_authorization_token_from_metadata()
        .map_err(|e| Error::RuntimeError(format!("Failed to fetch token: {}", e)))?;

    let token_obj = RObject::string(token).to_refcount_assigned();
    let klass = vm.getself()?;
    klass.set_ivar(TOKEN_IVAR_KEY, token_obj.clone());
    Ok(token_obj)
}

fn uzumibi_google_token(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let klass = vm.getself()?;
    let token = klass.get_ivar(TOKEN_IVAR_KEY);
    Ok(token)
}

fn uzumibi_google_set_token(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    if args.is_empty() {
        return Err(Error::ArgumentError(
            "Expected 1 argument: value".to_string(),
        ));
    }
    let klass = vm.getself()?;
    klass.set_ivar(TOKEN_IVAR_KEY, args[0].clone());
    Ok(args[0].clone())
}

fn uzumibi_google_fetch_project_id(
    vm: &mut VM,
    _args: &[Rc<RObject>],
) -> Result<Rc<RObject>, Error> {
    let project_id = meta::get_project_id_from_metadata()
        .map_err(|e| Error::RuntimeError(format!("Failed to fetch project_id: {}", e)))?;

    let project_id_obj = RObject::string(project_id).to_refcount_assigned();
    let klass = vm.getself()?;
    klass.set_ivar(PROJECT_ID_IVAR_KEY, project_id_obj.clone());
    Ok(project_id_obj)
}

fn uzumibi_google_project_id(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let klass = vm.getself()?;
    let project_id = klass.get_ivar(PROJECT_ID_IVAR_KEY);
    Ok(project_id)
}

fn uzumibi_google_set_project_id(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    if args.is_empty() {
        return Err(Error::ArgumentError(
            "Expected 1 argument: value".to_string(),
        ));
    }
    let klass = vm.getself()?;
    klass.set_ivar(PROJECT_ID_IVAR_KEY, args[0].clone());
    Ok(args[0].clone())
}

// --- Uzumibi::KV methods ---

fn get_google_credentials(vm: &mut VM) -> Result<(String, String), Error> {
    let google_mod = vm.get_module_by_name("Google");

    let google_obj = RObject::module(google_mod).to_refcount_assigned();

    let token_obj = google_obj.get_ivar(TOKEN_IVAR_KEY);
    if token_obj.is_falsy() {
        return Err(Error::RuntimeError(
            "Token not set. Call Uzumibi::Google.fetch_token or Uzumibi::Google.token= first"
                .to_string(),
        ));
    }
    let token: String = token_obj
        .as_ref()
        .try_into()
        .map_err(|e| Error::RuntimeError(format!("Invalid token: {}", e)))?;

    let project_id_obj = google_obj.get_ivar(PROJECT_ID_IVAR_KEY);
    if project_id_obj.is_falsy() {
        return Err(Error::RuntimeError(
            "Project ID not set. Call Uzumibi::Google.fetch_project_id or Uzumibi::Google.project_id= first"
                .to_string(),
        ));
    }
    let project_id: String = project_id_obj
        .as_ref()
        .try_into()
        .map_err(|e| Error::RuntimeError(format!("Invalid project_id: {}", e)))?;

    Ok((token, project_id))
}

fn uzumibi_kv_get(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    if args.is_empty() {
        return Err(Error::ArgumentError("Expected 1 argument: key".to_string()));
    }
    let key: String = args[0]
        .as_ref()
        .try_into()
        .map_err(|e| Error::RuntimeError(format!("Invalid key: {}", e)))?;

    let (token, project_id) = get_google_credentials(vm)?;

    match firestore::get_document(&project_id, &token, &key) {
        Ok(value) => Ok(RObject::string(value).to_refcount_assigned()),
        Err(firestore::FirestoreError::DocumentNotFound(_)) => {
            Ok(RObject::nil().to_refcount_assigned())
        }
        Err(e) => Err(Error::RuntimeError(format!("KV get failed: {}", e))),
    }
}

fn uzumibi_kv_set(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    if args.len() < 2 {
        return Err(Error::ArgumentError(
            "Expected 2 arguments: key, value".to_string(),
        ));
    }
    let key: String = args[0]
        .as_ref()
        .try_into()
        .map_err(|e| Error::RuntimeError(format!("Invalid key: {}", e)))?;
    let value: String = args[1]
        .as_ref()
        .try_into()
        .map_err(|e| Error::RuntimeError(format!("Invalid value: {}", e)))?;

    let (token, project_id) = get_google_credentials(vm)?;

    match firestore::set_document(&project_id, &token, &key, &value) {
        Ok(_) => Ok(RObject::boolean(true).to_refcount_assigned()),
        Err(e) => Err(Error::RuntimeError(format!("KV set failed: {}", e))),
    }
}
