use std::path::Path;

extern crate mruby_compiler2_sys;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    println!("cargo:rerun-if-changed=lib/app.rb");
    println!("cargo:rerun-if-changed=lib/consumer.rb");

    let is_queue = std::env::var("CARGO_FEATURE_QUEUE").is_ok();
    let (mrb_path, code) = if is_queue {
        (
            Path::new(&out_dir).join("consumer.mrb"),
            include_str!("lib/consumer.rb"),
        )
    } else {
        (
            Path::new(&out_dir).join("app.mrb"),
            include_str!("lib/app.rb"),
        )
    };

    unsafe {
        let mut ctx = mruby_compiler2_sys::MRubyCompiler2Context::new();
        ctx.compile_to_file(code, &mrb_path)
            .expect("failed to compile mruby script");
    }
}
