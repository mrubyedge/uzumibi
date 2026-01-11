Uzumibi
==========

Uzumibi is a lightweight web application framework for embedding MRuby into edge computing platforms like Cloudflare Workers, Fastly Compute@Edge, Spin and so on. It allows developers to write serverless applications using Ruby, leveraging the power of MRuby for efficient execution in constrained environments.

Uzumibi uses a specialized mruby implementation [mruby/edge](https://github.com/mrubyedge/mrubyedge), which is optimized for edge computing scenarios - WebAssembly environments with limited resources.

### tl;dr

Ruby code example for Uzumibi:

```ruby
class App < Uzumibi::Router
  get "/" do |req, res|
    debug_console("[Uzumibi] Received request at /")
    debug_console("[Uzumibi] Requested UA: #{req.headers["user-agent"]}")

    res.status_code = 200
    res.headers = {
      "content-type" => "text/plain",
      "x-powered-by" => "#{RUBY_ENGINE} #{RUBY_VERSION}"
    }
    res.body = "Hello from edges!\n"
    res
  end

  get "/description" do |req, res|
    res.status_code = 200
    res.headers = {
      "content-type" => "text/plain",
    }
    res.body = 
      "\"Uzumibi\" is a Japanese term that refers\n" +
      "to live embers buried under a layer of ash\n" +
      "to keep the fire from going out.\n"
    res
  end
end

$APP = App.new
```

...that runs on various edge platforms!!

Crates and projects
-----------------

- [**uzumibi-cli**](./uzumibi-cli/) - A command-line interface tool to generate Uzumibi application scaffolds to various edge platforms.
    - ![crates.io](https://img.shields.io/crates/v/uzumibi-cli.svg)
- [**uzumibi-gem**](./uzumibi-gem/) - The mruby/edge gem that provides the core Uzumibi framework functionality.
    - ![crates.io](https://img.shields.io/crates/v/uzumibi-gem.svg)

### Spike codes

- [**uzumibi-on-cloudflare-spike**](./uzumibi-on-cloudflare-spike/) - An Uzumibi application scaffold for Cloudflare Workers (using Spin).
- [**uzumibi-on-cloudrun-spike**](./uzumibi-on-cloudrun-spike/) - An Uzumibi application scaffold for Google Cloud Run. Experimental.
- [**uzumibi-on-fastly-spike**](./uzumibi-on-fastly-spike/) - An Uzumibi application scaffold for Fastly Compute@Edge.
- [**uzumibi-on-spin-spike**](./uzumibi-on-spin-spike/) - An Uzumibi application scaffold for Spin using Fermyon Cloud.

### ToDos

- Support of wasmCloud

## How to pronounce "Uzumibi"

Uzumibi(うずみび) is pronounced as /`oo-zóo-mi-bì`/, which sounds natural when you  pronounce in relaxed oo - `ʊ`
