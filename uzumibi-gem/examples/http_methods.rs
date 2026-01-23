use mrubyedge::yamrb::vm::VM;

extern crate mruby_compiler2_sys;
extern crate mrubyedge;
extern crate uzumibi_gem;

fn main() -> Result<(), mrubyedge::Error> {
    let script = r#"
class App < Uzumibi::Router
  get "/users/:id" do |req, res|
    res.status_code = 200
    res.headers = { "Content-Type" => "text/plain" }
    res.body = "GET User " + req.params[:id].to_s
  end

  post "/users" do |req, res|
    res.status_code = 201
    res.headers = { "Content-Type" => "text/plain" }
    res.body = "POST: User created"
  end

  put "/users/:id" do |req, res|
    res.status_code = 200
    res.headers = { "Content-Type" => "text/plain" }
    res.body = "PUT: User " + req.params[:id].to_s + " updated"
  end

  delete "/users/:id" do |req, res|
    res.status_code = 200
    res.headers = { "Content-Type" => "text/plain" }
    res.body = "DELETE: User " + req.params[:id].to_s + " deleted"
  end

  head "/users/:id" do |req, res|
    res.status_code = 200
    res.headers = { "Content-Type" => "text/plain" }
    res.body = "This should not be returned for HEAD"
  end
end

router = App.new

puts "Testing GET /users/123"
sm = router.initialize_request(1024)
buf = "GET"
buf += [0, 0, 0].pack("CCC")
buf += ["/users/123".size].pack("S")
buf += "/users/123"
buf += [0, 0, 0].pack("SSL")
sm.replace(buf)
response = router.start_request
puts "Status: " + response.status_code.to_s
puts "Body: " + response.body.inspect
puts ""

puts "Testing POST /users"
sm = router.initialize_request(1024)
buf = "POST"
buf += [0, 0].pack("CC")
buf += ["/users".size].pack("S")
buf += "/users"
buf += [0, 0, 0].pack("SSL")
sm.replace(buf)
response = router.start_request
puts "Status: " + response.status_code.to_s
puts "Body: " + response.body.inspect
puts ""

puts "Testing PUT /users/456"
sm = router.initialize_request(1024)
buf = "PUT"
buf += [0, 0, 0].pack("CCC")
buf += ["/users/456".size].pack("S")
buf += "/users/456"
buf += [0, 0, 0].pack("SSL")
sm.replace(buf)
response = router.start_request
puts "Status: " + response.status_code.to_s
puts "Body: " + response.body.inspect
puts ""

puts "Testing DELETE /users/789"
sm = router.initialize_request(1024)
buf = "DELETE"
buf += ["/users/789".size].pack("S")
buf += "/users/789"
buf += [0, 0, 0].pack("SSL")
sm.replace(buf)
response = router.start_request
puts "Status: " + response.status_code.to_s
puts "Body: " + response.body.inspect
puts ""

puts "Testing HEAD /users/999"
sm = router.initialize_request(1024)
buf = "HEAD"
buf += [0, 0].pack("CC")
buf += ["/users/999".size].pack("S")
buf += "/users/999"
buf += [0, 0, 0].pack("SSL")
sm.replace(buf)
response = router.start_request
puts "Status: " + response.status_code.to_s
puts "Body: " + response.body.inspect
puts ""
"#;

    let mrb_bin = unsafe {
        mruby_compiler2_sys::MRubyCompiler2Context::new()
            .compile(script)
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
