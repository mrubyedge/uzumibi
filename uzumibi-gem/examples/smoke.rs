use mrubyedge::yamrb::vm::VM;

extern crate mruby_compiler2_sys;
extern crate mrubyedge;
extern crate uzumibi_gem;

const SCRIPT: &str = r#"
class App < Uzumibi::Router
  get "/" do |req, res|
    p req.method
    p req.path
    p req.headers
    res.status_code = 200
    res.headers = { "Content-Type" => "text/plain" }
    res.body = "Hello, Uzumibi!"
  end

  get "/users/:id" do |req, res|
    p req.path
    res.status_code = 200
    res.headers = { "Content-Type" => "text/plain" }
    res.body = "Hello, User #{req.params[:id]}! param: #{req.params[:foo]}, #{req.params[:baz]}"
  end
end

router = App.new
p router

sm = router.initialize_request(1024)
p sm
path = "/users/42"
query_string = "foo=bar&baz=qux"
buf = "GET"
buf += [0, 0, 0].pack("CCC")  # buffer
buf += [path.size].pack("S")  # path size
buf += path
buf += [query_string.size].pack("S")  # query string size
buf += query_string
buf += [2].pack("S")  # headers size
buf += [4].pack("S")  # header 1 key size
buf += "Host"
buf += [11].pack("S") # header 1 value size
buf += "example.com"
buf += [6].pack("S")  # header 2 key size
buf += "Accept"
buf += [3].pack("S")  # header 2 value size
buf += "*/*"
buf += [0].pack("L")  # body size
sm.replace(buf)

response = router.start_request
puts response.body
"#;

fn main() -> Result<(), mrubyedge::Error> {
    //unsafe {
    // std::env::set_var("MRUBYEDGE_DEBUG", "2");
    //}

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
