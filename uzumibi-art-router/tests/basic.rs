use mrubyedge::yamrb::vm::VM;

extern crate mruby_compiler2_sys;
extern crate mrubyedge;
extern crate uzumibi_art_router;
extern crate uzumibi_gem;

fn compile_vm(code: &'static str) -> Result<VM, mrubyedge::Error> {
    // unsafe {
    //     std::env::set_var("MRUBYEDGE_DEBUG", "2");
    // }

    let mrb_bin = unsafe {
        mruby_compiler2_sys::MRubyCompiler2Context::new()
            .compile(code)
            .map_err(|e| {
                mrubyedge::Error::RuntimeError(format!("Failed to compile script: {}", e))
            })?
    };
    let mut rite = mrubyedge::rite::load(&mrb_bin)
        .map_err(|e| mrubyedge::Error::RuntimeError(format!("Failed to load rite: {}", e)))?;
    let mut vm = VM::open(&mut rite);
    uzumibi_gem::init::init_uzumibi(&mut vm);
    uzumibi_art_router::init_uzumibi_art_router(&mut vm);
    Ok(vm)
}

#[test]
fn test_art_router_smoke() -> Result<(), mrubyedge::Error> {
    let code = r#"
    art = Uzumibi::ArtRouter.new
    art.inspect
    "#;
    let mut vm = compile_vm(code)?;
    let ret = vm
        .run()
        .map_err(|e| mrubyedge::Error::RuntimeError(format!("Failed to run script: {}", e)))?;
    let ret: String = ret.as_ref().try_into()?;
    assert_eq!(&ret[0..21], "#<Uzumibi::ArtRouter:");
    Ok(())
}

#[test]
fn test_art_router_match_simple() -> Result<(), mrubyedge::Error> {
    let code = r#"
    art = Uzumibi::ArtRouter.new
    art.set_route "/hello" do |i|
      i + 5470
    end
    route, _hash = *art.get_route("/hello")
    route.call(1)
    "#;
    let mut vm = compile_vm(code)?;
    let ret = vm
        .run()
        .map_err(|e| mrubyedge::Error::RuntimeError(format!("Failed to run script: {}", e)))?;
    let ret: i32 = ret.as_ref().try_into()?;
    assert_eq!(ret, 5471);
    Ok(())
}

#[test]
fn test_art_router_match_params() -> Result<(), mrubyedge::Error> {
    let code = r#"
    art = Uzumibi::ArtRouter.new
    art.set_route "/users/:id" do |uid|
      "User id: " + uid
    end
    route, params = *art.get_route("/users/23")
    route.call(params[:id])
    "#;
    let mut vm = compile_vm(code)?;
    let ret = vm
        .run()
        .map_err(|e| mrubyedge::Error::RuntimeError(format!("Failed to run script: {}", e)))?;
    let ret: String = ret.as_ref().try_into()?;
    assert_eq!(ret, "User id: 23");
    Ok(())
}

#[test]
fn test_art_router_match_params_nest() -> Result<(), mrubyedge::Error> {
    let code = r#"
    art = Uzumibi::ArtRouter.new
    art.set_route "/users/:id/posts/:title" do |uid, title|
      "User #{uid} wrote #{title}"
    end
    route, params = *art.get_route("/users/23/posts/hello-world")
    route.call(params[:id], params[:title])
    "#;
    let mut vm = compile_vm(code)?;
    let ret = vm
        .run()
        .map_err(|e| mrubyedge::Error::RuntimeError(format!("Failed to run script: {}", e)))?;
    let ret: String = ret.as_ref().try_into()?;
    assert_eq!(ret, "User 23 wrote hello-world");
    Ok(())
}
