use std::path::Path;

extern crate mruby_compiler2_sys;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let mrb_path = Path::new(&out_dir).join("consumer.mrb");
    let code = include_str!("lib/consumer.rb");
    println!("cargo:rerun-if-changed=lib/consumer.rb");

    unsafe {
        let mut ctx = mruby_compiler2_sys::MRubyCompiler2Context::new();
        ctx.compile_to_file(code, &mrb_path)
            .expect("failed to compile mruby script");
    }
}
