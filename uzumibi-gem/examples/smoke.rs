use std::env;

use mrubyedge::yamrb::vm::VM;

extern crate mruby_compiler2_sys;
extern crate mrubyedge;
extern crate uzumibi_gem;

const SCRIPT: &str = r#"
class App < Uzumibi::Router
  get "/" do
    "Hello, Uzumibi!"
  end
end

router = App.new
p router
"#;

fn main() -> Result<(), mrubyedge::Error> {
    unsafe {
        // env::set_var("MRUBYEDGE_DEBUG", "2");
    }

    let mrb_bin = unsafe {
        mruby_compiler2_sys::MRubyCompiler2Context::new()
            .dump_bytecode(SCRIPT)
            .expect("failed to compile...");

        mruby_compiler2_sys::MRubyCompiler2Context::new()
            .compile(SCRIPT)
            .map_err(|e| {
                mrubyedge::Error::RuntimeError(format!("Failed to compile script: {}", e))
            })?
    };
    let mut rite = mrubyedge::rite::load(&mrb_bin)
        .map_err(|e| mrubyedge::Error::RuntimeError(format!("Failed to load rite: {}", e)))?;
    let mut vm = VM::open(&mut rite);
    uzumibi_gem::init::init_uzumibi(&mut vm);
    vm.run()
        .map_err(|e| mrubyedge::Error::RuntimeError(format!("Failed to run script: {}", e)))?;
    Ok(())
}
