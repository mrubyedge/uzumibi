use std::{fs, path::Path};

extern crate mruby_compiler2_sys;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let http_max_bytes = std::env::var("UZUMIBI_HTTP_MAX_BYTES")
        .unwrap_or_else(|_| "65536".to_string())
        .parse::<u32>()
        .expect("UZUMIBI_HTTP_MAX_BYTES must be a positive 32-bit integer");
    assert!(
        http_max_bytes > 0 && http_max_bytes <= i32::MAX as u32,
        "UZUMIBI_HTTP_MAX_BYTES must be between 1 and {}",
        i32::MAX
    );
    fs::write(
        Path::new(&out_dir).join("uzumibi_config.rs"),
        format!("pub const HTTP_MAX_BYTES: u32 = {http_max_bytes};\n"),
    )
    .expect("failed to write Uzumibi build configuration");
    println!("cargo:rerun-if-env-changed=UZUMIBI_HTTP_MAX_BYTES");

    let mrb_path = Path::new(&out_dir).join("consumer.mrb");
    let code = include_str!("../lib/consumer.rb");
    println!("cargo:rerun-if-changed=../lib/consumer.rb");

    unsafe {
        let mut ctx = mruby_compiler2_sys::MRubyCompiler2Context::new();
        ctx.compile_to_file(code, &mrb_path)
            .expect("failed to compile mruby script");
    }
}
