pub mod firestore;
pub mod jwt;
pub mod meta;
pub mod pubsub;

use std::rc::Rc;

use mrubyedge::{
    Error,
    yamrb::{
        helpers::{
            mrb_define_class_cmethod, mrb_define_cmethod, mrb_define_module_cmethod, mrb_funcall,
        },
        prelude::hash::{mrb_hash_new, mrb_hash_set_index},
        value::{RObject, RSym, RValue},
        vm::VM,
    },
};
use reqwest::blocking::Client;

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
///     class Fetch
///       def self.fetch(url: String, method: String = "GET", body: String = "", headers: Hash[String, String] = {}) -> Response
///     end
///     class Queue
///       def self.send(topic_name: String, message: String) -> bool
///     end
///     class Message
///       attr_accessor id: String
///       attr_accessor timestamp: String
///       attr_accessor body: String
///       def ack!() -> bool
///       def nack!() -> bool
///       def retry!(delay_seconds: Integer) -> bool
///     end
///     class Consumer
///       def on_receive(message: Message) -> untyped
///     end
///     class Access
///       def self.get_identity(jwt_token: String, expected_audience: String?) -> Identity
///     end
///     class Identity
///       attr_accessor user_uuid: String
///       attr_accessor email: String
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

    // Uzumibi::Fetch class
    let fetch_class = vm.define_class("Fetch", None, Some(uzumibi.clone()));
    mrb_define_class_cmethod(
        vm,
        fetch_class,
        "fetch",
        Box::new(uzumibi_fetch_class_fetch),
    );

    // Uzumibi::Queue class
    let queue_class = vm.define_class("Queue", None, Some(uzumibi.clone()));
    mrb_define_class_cmethod(vm, queue_class, "send", Box::new(uzumibi_queue_class_send));

    // Uzumibi::Message class
    let message_class = vm.define_class("Message", None, Some(uzumibi.clone()));
    let message_class_obj = RObject::class(message_class.clone(), vm);
    for attr in ["id", "timestamp", "body"] {
        mrb_funcall(
            vm,
            Some(message_class_obj.clone()),
            "attr_accessor",
            &[RObject::symbol(RSym::new(attr.to_string())).to_refcount_assigned()],
        )
        .expect("attr_accessor failed");
    }
    mrb_define_cmethod(
        vm,
        message_class.clone(),
        "ack!",
        Box::new(uzumibi_message_ack),
    );
    mrb_define_cmethod(
        vm,
        message_class.clone(),
        "nack!",
        Box::new(uzumibi_message_nack),
    );
    mrb_define_cmethod(vm, message_class, "retry!", Box::new(uzumibi_message_retry));

    #[cfg(feature = "queue")]
    {
        // Uzumibi::Consumer (base class for user-defined consumers)
        let consumer_class = vm.define_class("Consumer", None, Some(uzumibi.clone()));
        mrb_define_cmethod(
            vm,
            consumer_class,
            "on_receive",
            Box::new(uzumibi_consumer_on_receive),
        );
    }

    // Uzumibi::Access class
    let access_class = vm.define_class("Access", None, Some(uzumibi.clone()));
    mrb_define_class_cmethod(
        vm,
        access_class,
        "get_identity",
        Box::new(uzumibi_access_get_identity),
    );

    // Uzumibi::Identity class
    let identity_class = vm.define_class("Identity", None, Some(uzumibi));
    let identity_class_obj = RObject::class(identity_class, vm);
    for attr in ["user_uuid", "email"] {
        mrb_funcall(
            vm,
            Some(identity_class_obj.clone()),
            "attr_accessor",
            &[RObject::symbol(RSym::new(attr.to_string())).to_refcount_assigned()],
        )
        .expect("attr_accessor failed");
    }
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
    if token.is_falsy() {
        return uzumibi_google_fetch_token(vm, _args);
    }
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
    if project_id.is_falsy() {
        return uzumibi_google_fetch_project_id(vm, _args);
    }
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

    let token_obj = mrb_funcall(vm, Some(google_obj.clone()), "token", &[])?;
    let token: String = token_obj
        .as_ref()
        .try_into()
        .map_err(|e| Error::RuntimeError(format!("Invalid token: {}", e)))?;

    let project_id_obj = mrb_funcall(vm, Some(google_obj), "project_id", &[])?;
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

// --- Uzumibi::Fetch methods ---

/// Fetch.fetch(url, method="GET", body="", headers={}) -> Response
fn uzumibi_fetch_class_fetch(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    if args.is_empty() {
        return Err(Error::ArgumentError("Expected 1 argument: url".to_string()));
    }

    let url_obj = &args[0];
    let url = mrb_funcall(vm, url_obj.clone().into(), "to_s", &[])?;
    let url: String = url.as_ref().try_into()?;

    let method = if args.len() > 1 {
        let m = mrb_funcall(vm, args[1].clone().into(), "to_s", &[])?;
        let m: String = m.as_ref().try_into()?;
        m
    } else {
        "GET".to_string()
    };

    let body = if args.len() > 2 {
        let b = mrb_funcall(vm, args[2].clone().into(), "to_s", &[])?;
        let b: String = b.as_ref().try_into()?;
        b
    } else {
        String::new()
    };

    // Parse headers from Hash (4th argument)
    let mut headers_map = std::collections::HashMap::new();
    if args.len() > 3
        && let RValue::Hash(h) = &args[3].as_ref().value
    {
        let hash = h.borrow();
        for (_, (key_obj, value_obj)) in hash.iter() {
            let key = mrb_funcall(vm, key_obj.clone().into(), "to_s", &[])?;
            let key: String = key.as_ref().try_into()?;
            let value = mrb_funcall(vm, value_obj.clone().into(), "to_s", &[])?;
            let value: String = value.as_ref().try_into()?;
            headers_map.insert(key, value);
        }
    }

    // Make HTTP request
    let client = Client::new();
    let mut request = match method.as_str() {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "PATCH" => client.patch(&url),
        "DELETE" => client.delete(&url),
        "HEAD" => client.head(&url),
        _ => {
            return Err(Error::RuntimeError(format!(
                "Unsupported HTTP method: {}",
                method
            )));
        }
    };

    // Add headers
    for (key, value) in headers_map {
        request = request.header(key, value);
    }

    // Add body for methods that support it
    if !body.is_empty() && matches!(method.as_str(), "POST" | "PUT" | "PATCH") {
        request = request.body(body);
    }

    let response = request
        .send()
        .map_err(|e| Error::RuntimeError(format!("HTTP request failed: {}", e)))?;

    let status_code = response.status().as_u16();
    let response_headers = response.headers().clone();
    let response_body = response
        .text()
        .map_err(|e| Error::RuntimeError(format!("Failed to read response body: {}", e)))?;

    // Create Uzumibi::Response instance
    let uzumibi = vm
        .get_const_by_name("Uzumibi")
        .ok_or_else(|| Error::RuntimeError("Uzumibi module not found".to_string()))?;
    let uzumibi_module = match &uzumibi.as_ref().value {
        RValue::Module(m) => m.clone(),
        _ => return Err(Error::RuntimeError("Uzumibi must be a module".to_string())),
    };
    let response_class = uzumibi_module
        .get_const_by_name("Response")
        .ok_or_else(|| Error::RuntimeError("Uzumibi::Response class not found".to_string()))?;
    let response_obj = mrb_funcall(vm, Some(response_class), "new", &[])?;

    response_obj.set_ivar(
        "@status_code",
        RObject::integer(status_code as i64).to_refcount_assigned(),
    );

    let headers_hash = mrb_hash_new(vm, &[])?;
    for (key, value) in response_headers.iter() {
        let key_str = key.to_string();
        let value_str = value.to_str().unwrap_or("").to_string();
        mrb_hash_set_index(
            headers_hash.clone(),
            RObject::string(key_str).to_refcount_assigned(),
            RObject::string(value_str).to_refcount_assigned(),
        )?;
    }
    response_obj.set_ivar("@headers", headers_hash);

    response_obj.set_ivar(
        "@body",
        RObject::string(response_body).to_refcount_assigned(),
    );

    Ok(response_obj)
}

// --- Uzumibi::Queue methods ---

/// Queue.send(topic_name, message) -> bool
fn uzumibi_queue_class_send(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    if args.len() < 2 {
        return Err(Error::ArgumentError(
            "Expected 2 arguments: topic_name, message".to_string(),
        ));
    }

    let topic_obj = &args[0];
    let topic = mrb_funcall(vm, topic_obj.clone().into(), "to_s", &[])?;
    let topic: String = topic.as_ref().try_into()?;

    let message_obj = &args[1];
    let message = mrb_funcall(vm, message_obj.clone().into(), "to_s", &[])?;
    let message: String = message.as_ref().try_into()?;

    let (token, _project_id) = get_google_credentials(vm)?;

    let pubsub_message = pubsub::PubsubMessage {
        data: Some(message),
        attributes: None,
        message_id: None,
        publish_time: None,
        ordering_key: None,
    };

    match pubsub::publish(&token, &topic, vec![pubsub_message]) {
        Ok(_) => Ok(RObject::boolean(true).to_refcount_assigned()),
        Err(e) => Err(Error::RuntimeError(format!(
            "Failed to send message: {}",
            e
        ))),
    }
}

// --- Uzumibi::Message methods ---

/// Message#ack! -> bool
fn uzumibi_message_ack(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let self_obj = vm.getself()?;
    let id_obj = self_obj.get_ivar("@id");
    if matches!(id_obj.as_ref().value, RValue::Nil) {
        return Err(Error::RuntimeError(
            "Message object does not have @id".to_string(),
        ));
    }

    let id = mrb_funcall(vm, id_obj.into(), "to_s", &[])?;
    let id: String = id.as_ref().try_into()?;

    let (token, _project_id) = get_google_credentials(vm)?;

    // Get subscription from message or use default
    let subscription_obj = self_obj.get_ivar("@subscription");
    let subscription = if matches!(subscription_obj.as_ref().value, RValue::Nil) {
        return Err(Error::RuntimeError(
            "Message object does not have @subscription".to_string(),
        ));
    } else {
        let sub = mrb_funcall(vm, subscription_obj.into(), "to_s", &[])?;
        let sub: String = sub.as_ref().try_into()?;
        sub
    };

    match pubsub::acknowledge(&token, &subscription, vec![id]) {
        Ok(_) => Ok(RObject::boolean(true).to_refcount_assigned()),
        Err(e) => Err(Error::RuntimeError(format!("Failed to ack message: {}", e))),
    }
}

/// Message#nack! -> bool (modifyAckDeadline with 0 seconds)
fn uzumibi_message_nack(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let self_obj = vm.getself()?;
    let id_obj = self_obj.get_ivar("@id");
    if matches!(id_obj.as_ref().value, RValue::Nil) {
        return Err(Error::RuntimeError(
            "Message object does not have @id".to_string(),
        ));
    }

    let id = mrb_funcall(vm, id_obj.into(), "to_s", &[])?;
    let id: String = id.as_ref().try_into()?;

    let (token, _project_id) = get_google_credentials(vm)?;

    let subscription_obj = self_obj.get_ivar("@subscription");
    let subscription = if matches!(subscription_obj.as_ref().value, RValue::Nil) {
        return Err(Error::RuntimeError(
            "Message object does not have @subscription".to_string(),
        ));
    } else {
        let sub = mrb_funcall(vm, subscription_obj.into(), "to_s", &[])?;
        let sub: String = sub.as_ref().try_into()?;
        sub
    };

    match pubsub::modify_ack_deadline(&token, &subscription, vec![id], 0) {
        Ok(_) => Ok(RObject::boolean(true).to_refcount_assigned()),
        Err(e) => Err(Error::RuntimeError(format!(
            "Failed to nack message: {}",
            e
        ))),
    }
}

/// Message#retry!(delay_seconds: N) -> bool
fn uzumibi_message_retry(vm: &mut VM, _args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    let self_obj = vm.getself()?;
    let id_obj = self_obj.get_ivar("@id");
    if matches!(id_obj.as_ref().value, RValue::Nil) {
        return Err(Error::RuntimeError(
            "Message object does not have @id".to_string(),
        ));
    }

    let id = mrb_funcall(vm, id_obj.into(), "to_s", &[])?;
    let id: String = id.as_ref().try_into()?;

    let delay_seconds: i32 = match vm.get_kwargs() {
        Some(kwargs) => match kwargs.get("delay_seconds") {
            Some(val) => {
                let v: i64 = val.as_ref().try_into()?;
                v as i32
            }
            None => 0,
        },
        None => 0,
    };

    let (token, _project_id) = get_google_credentials(vm)?;

    let subscription_obj = self_obj.get_ivar("@subscription");
    let subscription = if matches!(subscription_obj.as_ref().value, RValue::Nil) {
        return Err(Error::RuntimeError(
            "Message object does not have @subscription".to_string(),
        ));
    } else {
        let sub = mrb_funcall(vm, subscription_obj.into(), "to_s", &[])?;
        let sub: String = sub.as_ref().try_into()?;
        sub
    };

    match pubsub::modify_ack_deadline(&token, &subscription, vec![id], delay_seconds) {
        Ok(_) => Ok(RObject::boolean(true).to_refcount_assigned()),
        Err(e) => Err(Error::RuntimeError(format!(
            "Failed to retry message: {}",
            e
        ))),
    }
}

#[cfg(feature = "queue")]
fn uzumibi_consumer_on_receive(
    _vm: &mut VM,
    _args: &[Rc<RObject>],
) -> Result<Rc<RObject>, Error> {
    Err(Error::RuntimeError(
        "on_receive must be implemented by subclass of Uzumibi::Consumer".to_string(),
    ))
}

// --- Uzumibi::Access methods ---

/// Access.get_identity(jwt_token) -> Identity
fn uzumibi_access_get_identity(vm: &mut VM, args: &[Rc<RObject>]) -> Result<Rc<RObject>, Error> {
    if args.is_empty() {
        return Err(Error::ArgumentError(
            "Expected 1 argument: jwt_token".to_string(),
        ));
    }

    let uzumibi = vm
        .get_const_by_name("Uzumibi")
        .ok_or_else(|| Error::RuntimeError("Uzumibi module not found".to_string()))?;
    let uzumibi_module = match &uzumibi.as_ref().value {
        RValue::Module(m) => m.clone(),
        _ => return Err(Error::RuntimeError("Uzumibi must be a module".to_string())),
    };

    let token_obj = &args[0];
    let token = mrb_funcall(vm, token_obj.clone().into(), "to_s", &[])?;
    let token: String = token.as_ref().try_into()?;

    let expected_audience = if args.len() > 1 {
        let aud = mrb_funcall(vm, args[1].clone().into(), "to_s", &[])?;
        let aud: String = aud.as_ref().try_into()?;
        aud
    } else {
        let google_const = uzumibi_module
            .get_const_by_name("Google")
            .ok_or_else(|| Error::RuntimeError("Uzumibi::Google module not found".to_string()))?;
        let google_mod = match &google_const.as_ref().value {
            RValue::Module(m) => m.clone(),
            _ => {
                return Err(Error::RuntimeError(
                    "Uzumibi::Google must be a module".to_string(),
                ));
            }
        };
        let google_obj = RObject::module(google_mod).to_refcount_assigned();
        let project_id_obj = mrb_funcall(vm, Some(google_obj), "project_id", &[])?;
        let project_id: String = project_id_obj.as_ref().try_into()?;
        format!("/projects/{}/apps/default", project_id)
    };

    let claims = jwt::validate_iap_jwt(&token, &expected_audience)
        .map_err(|e| Error::RuntimeError(format!("Failed to validate JWT: {}", e)))?;

    // Create Uzumibi::Identity instance
    let identity_class = uzumibi_module
        .get_const_by_name("Identity")
        .ok_or_else(|| Error::RuntimeError("Uzumibi::Identity class not found".to_string()))?;

    let identity = mrb_funcall(vm, Some(identity_class), "new", &[])?;

    identity.set_ivar(
        "@user_uuid",
        RObject::string(claims.sub).to_refcount_assigned(),
    );
    identity.set_ivar(
        "@email",
        RObject::string(claims.email).to_refcount_assigned(),
    );

    Ok(identity)
}
