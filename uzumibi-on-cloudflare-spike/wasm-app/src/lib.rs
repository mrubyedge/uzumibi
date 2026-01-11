#![allow(static_mut_refs)]
extern crate mrubyedge;
extern crate uzumibi_gem;

use std::{mem::MaybeUninit, rc::Rc};

use mrubyedge::{
    Error,
    rite::rite,
    yamrb::{
        helpers::{mrb_define_cmethod, mrb_funcall},
        value::{RObject, RValue},
        vm::VM,
    },
};

static MRB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/app.mrb"));

static mut MRUBY_VM: MaybeUninit<VM> = MaybeUninit::uninit();
static mut MRUBY_VM_LOADED: bool = false;

unsafe extern "C" {
    unsafe fn debug_console_log(ptr: *const u8, len: usize);
}

fn debug_console_log_internal(message: &str) {
    unsafe {
        debug_console_log(message.as_ptr(), message.len());
    }
}

fn uzumibi_kernel_debug_console_log(
    vm: &mut VM,
    args: &[Rc<RObject>],
) -> Result<Rc<RObject>, mrubyedge::Error> {
    let msg_obj = &args[0];
    let msg = mrb_funcall(vm, msg_obj.clone().into(), "to_s", &[])?;
    let msg: String = msg.as_ref().try_into()?;
    unsafe {
        debug_console_log(msg.as_ptr(), msg.len());
    }
    Ok(RObject::nil().to_refcount_assigned())
}

fn init_vm() -> VM {
    let mut rite = rite::load(MRB).expect("failed to load");
    let mut vm = VM::open(&mut rite);
    uzumibi_gem::init::init_uzumibi(&mut vm);
    let object = vm.object_class.clone();
    mrb_define_cmethod(
        &mut vm,
        object,
        "debug_console",
        Box::new(uzumibi_kernel_debug_console_log),
    );

    vm.run().expect("failed to run");

    vm
}

fn assume_init_vm() -> &'static mut VM {
    unsafe {
        if !MRUBY_VM_LOADED {
            MRUBY_VM = MaybeUninit::new(init_vm());
            MRUBY_VM_LOADED = true;
        }
        MRUBY_VM.assume_init_mut()
    }
}

#[unsafe(export_name = "uzumibi_initialize_request")]
unsafe extern "C" fn uzumibi_initialize_request(size: i32) -> *mut u8 {
    let vm = assume_init_vm();
    let size = RObject::integer(size as i64).to_refcount_assigned();
    let app = vm
        .globals
        .get("$APP")
        .or_else(|| {
            debug_console_log_internal("$APP is not defined");
            None
        })
        .unwrap();
    let ret = mrb_funcall(vm, app.clone().into(), "initialize_request", &[size])
        .map_err(|e| {
            debug_console_log_internal(&format!("Error in initialize_request: {}", e));
            e
        })
        .unwrap();
    ret.as_ref()
        .try_into()
        .map_err(|e: Error| {
            debug_console_log_internal(&format!("Error converting to pointer: {}", e));
            e
        })
        .unwrap()
}

#[unsafe(export_name = "uzumibi_start_request")]
unsafe extern "C" fn uzumibi_start_request() -> *mut u8 {
    debug_console_log_internal("uzumibi_start_request called");
    let vm = assume_init_vm();
    let app = vm
        .globals
        .get("$APP")
        .or_else(|| {
            debug_console_log_internal("$APP is not defined");
            None
        })
        .unwrap();
    let ret = mrb_funcall(
        vm,
        app.clone().into(),
        "start_request_and_return_shared_memory",
        &[],
    )
    .map_err(|e| {
        debug_console_log_internal(&format!("Error in start_request: {}", e));
        e
    })
    .unwrap();
    match &ret.as_ref().value {
        RValue::SharedMemory(sm) => sm.borrow_mut().leak(),
        _ => {
            debug_console_log_internal("Error: Returned value is not SharedMemory");
            std::ptr::null_mut()
        }
    }
}
