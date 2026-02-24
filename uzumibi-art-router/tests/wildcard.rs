use mrubyedge::yamrb::vm::VM;

extern crate mruby_compiler2_sys;
extern crate mrubyedge;
extern crate uzumibi_art_router;
extern crate uzumibi_gem;

fn compile_vm(code: &'static str) -> Result<VM, mrubyedge::Error> {
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
fn test_wildcard_simple() -> Result<(), mrubyedge::Error> {
    let code = r#"
    art = Uzumibi::ArtRouter.new
    art.set_route "/api/*" do |rest|
      "API path: " + rest
    end
    route, params = *art.get_route("/api/users/123")
    route.call(params[:*])
    "#;
    let mut vm = compile_vm(code)?;
    let ret = vm
        .run()
        .map_err(|e| mrubyedge::Error::RuntimeError(format!("Failed to run script: {}", e)))?;
    let ret: String = ret.as_ref().try_into()?;
    assert_eq!(ret, "API path: users/123");
    Ok(())
}

#[test]
fn test_wildcard_single_segment() -> Result<(), mrubyedge::Error> {
    let code = r#"
    art = Uzumibi::ArtRouter.new
    art.set_route "/static/*" do |rest|
      "Static: " + rest
    end
    route, params = *art.get_route("/static/style.css")
    route.call(params[:*])
    "#;
    let mut vm = compile_vm(code)?;
    let ret = vm
        .run()
        .map_err(|e| mrubyedge::Error::RuntimeError(format!("Failed to run script: {}", e)))?;
    let ret: String = ret.as_ref().try_into()?;
    assert_eq!(ret, "Static: style.css");
    Ok(())
}

#[test]
fn test_wildcard_deep_path() -> Result<(), mrubyedge::Error> {
    let code = r#"
    art = Uzumibi::ArtRouter.new
    art.set_route "/assets/*" do |rest|
      "Asset: " + rest
    end
    route, params = *art.get_route("/assets/images/icons/logo.png")
    route.call(params[:*])
    "#;
    let mut vm = compile_vm(code)?;
    let ret = vm
        .run()
        .map_err(|e| mrubyedge::Error::RuntimeError(format!("Failed to run script: {}", e)))?;
    let ret: String = ret.as_ref().try_into()?;
    assert_eq!(ret, "Asset: images/icons/logo.png");
    Ok(())
}

#[test]
fn test_wildcard_with_exact_route() -> Result<(), mrubyedge::Error> {
    let code = r#"
    art = Uzumibi::ArtRouter.new
    art.set_route "/api/*" do |rest|
      "Wildcard: " + rest
    end
    art.set_route "/about" do
      "About page"
    end

    route1, params1 = *art.get_route("/api/v1/users")
    result1 = route1.call(params1[:*])

    route2, _params2 = *art.get_route("/about")
    result2 = route2.call

    [result1, result2]
    "#;
    let mut vm = compile_vm(code)?;
    let ret = vm
        .run()
        .map_err(|e| mrubyedge::Error::RuntimeError(format!("Failed to run script: {}", e)))?;

    match &ret.value {
        mrubyedge::yamrb::value::RValue::Array(arr) => {
            let arr = arr.borrow();
            let result1: String = arr[0].as_ref().try_into()?;
            let result2: String = arr[1].as_ref().try_into()?;
            assert_eq!(result1, "Wildcard: v1/users");
            assert_eq!(result2, "About page");
        }
        _ => panic!("Expected array result"),
    }
    Ok(())
}

#[test]
fn test_wildcard_not_match_parent() -> Result<(), mrubyedge::Error> {
    let code = r#"
    art = Uzumibi::ArtRouter.new
    art.set_route "/api/*" do |rest|
      "API: " + rest
    end

    # /api itself should not match /api/*
    result = art.get_route("/other/path")
    result.empty?
    "#;
    let mut vm = compile_vm(code)?;
    let ret = vm
        .run()
        .map_err(|e| mrubyedge::Error::RuntimeError(format!("Failed to run script: {}", e)))?;
    let ret: bool = ret.as_ref().try_into()?;
    assert!(ret, "Non-matching path should return empty array");
    Ok(())
}

#[test]
fn test_wildcard_vs_param() -> Result<(), mrubyedge::Error> {
    // Test using Rust API directly to verify behavior
    use uzumibi_art_router::store::{Route, RouteStore};

    let store: RouteStore<&str> = RouteStore::new();

    // Register both :id param route and * wildcard route
    store.insert("/api/:id", Route::new("param_handler"));
    store.insert("/api/*", Route::new("wildcard_handler"));

    // /api/foo - single segment, should match :id (param has priority over wildcard)
    let (handler1, params1) = store.get_with_params("/api/foo");
    assert_eq!(handler1.unwrap(), "param_handler");
    assert_eq!(params1.get("id").unwrap(), "foo");

    // /api/foo/bar - multiple segments, should match wildcard
    let (handler2, params2) = store.get_with_params("/api/foo/bar");
    assert_eq!(handler2.unwrap(), "wildcard_handler");
    assert_eq!(params2.get("*").unwrap(), "foo/bar");

    Ok(())
}

#[test]
fn test_wildcard_vs_exact_match() -> Result<(), mrubyedge::Error> {
    let code = r#"
    art = Uzumibi::ArtRouter.new
    art.set_route "/api/*" do |rest|
      "Wildcard: " + rest
    end
    art.set_route "/api/foo" do
      "Exact: foo"
    end

    # /api/foo should match exact route, not wildcard
    route1, _params1 = *art.get_route("/api/foo")
    result1 = route1.call

    # /api/bar should match wildcard
    route2, params2 = *art.get_route("/api/bar")
    result2 = route2.call(params2[:*])

    # /api/foo/baz should match wildcard (deeper path)
    route3, params3 = *art.get_route("/api/foo/baz")
    result3 = route3.call(params3[:*])

    [result1, result2, result3]
    "#;
    let mut vm = compile_vm(code)?;
    let ret = vm
        .run()
        .map_err(|e| mrubyedge::Error::RuntimeError(format!("Failed to run script: {}", e)))?;

    match &ret.value {
        mrubyedge::yamrb::value::RValue::Array(arr) => {
            let arr = arr.borrow();
            let result1: String = arr[0].as_ref().try_into()?;
            let result2: String = arr[1].as_ref().try_into()?;
            let result3: String = arr[2].as_ref().try_into()?;
            assert_eq!(result1, "Exact: foo", "/api/foo should match exact route");
            assert_eq!(result2, "Wildcard: bar", "/api/bar should match wildcard");
            assert_eq!(
                result3, "Wildcard: foo/baz",
                "/api/foo/baz should match wildcard"
            );
        }
        _ => panic!("Expected array result"),
    }
    Ok(())
}

#[test]
fn test_multiple_wildcards() -> Result<(), mrubyedge::Error> {
    let code = r#"
    art = Uzumibi::ArtRouter.new
    art.set_route "/api/*" do |rest|
      "API: " + rest
    end
    art.set_route "/static/*" do |rest|
      "Static: " + rest
    end

    route1, params1 = *art.get_route("/api/users")
    result1 = route1.call(params1[:*])

    route2, params2 = *art.get_route("/static/js/app.js")
    result2 = route2.call(params2[:*])

    [result1, result2]
    "#;
    let mut vm = compile_vm(code)?;
    let ret = vm
        .run()
        .map_err(|e| mrubyedge::Error::RuntimeError(format!("Failed to run script: {}", e)))?;

    match &ret.value {
        mrubyedge::yamrb::value::RValue::Array(arr) => {
            let arr = arr.borrow();
            let result1: String = arr[0].as_ref().try_into()?;
            let result2: String = arr[1].as_ref().try_into()?;
            assert_eq!(result1, "API: users");
            assert_eq!(result2, "Static: js/app.js");
        }
        _ => panic!("Expected array result"),
    }
    Ok(())
}
