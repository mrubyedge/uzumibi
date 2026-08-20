# Uzumibi

![Uzumibi's Logo](./logo.png)

Uzumibi is a Ruby web framework and project generator for WebAssembly-based edge and serverless runtimes. Ruby application code is compiled to mruby bytecode at build time and executed by [mruby/edge](https://github.com/mrubyedge/mrubyedge) inside a platform-specific host.

The `uzumibi` CLI currently provides templates for:

- Cloudflare Workers
- Fastly Compute
- Spin
- Google Cloud Run
- Browser Service Workers
- Browser Web Workers

## Documentation

- [Beginning Uzumibi](https://mrubyedge.github.io/beginning-uzumibi/)
- [Uzumibi documentation](https://mrubyedge.github.io/uzumibi/)

## Quick start with Cloudflare Workers

Install the CLI and the WebAssembly target:

~~~bash
cargo install uzumibi-cli
rustup target add wasm32-unknown-unknown
~~~

Create and run a project:

~~~bash
uzumibi new --template cloudflare my-app
cd my-app
pnpm install
pnpm run dev
~~~

Edit `lib/app.rb` to define routes:

~~~ruby
class App < Uzumibi::Router
  get "/" do |req, res|
    res.return(
      200,
      { "content-type" => "text/plain" },
      "Hello from #{RUBY_ENGINE} #{RUBY_VERSION}\n"
    )
  end

  get "/hello/:name" do |req, res|
    res.return(
      200,
      { "content-type" => "text/plain" },
      "Hello, #{req.params[:name]}!\n"
    )
  end
end

$APP = App.new
~~~

For Cloudflare Workers, `pnpm run dev` rebuilds the Wasm module and starts Wrangler. See the [Cloudflare Workers guide](https://mrubyedge.github.io/uzumibi/platforms/cloudflare-workers.html) for request-size configuration, external services, static assets, and Queue consumers.

## Workspace components

- [`uzumibi-cli`](./uzumibi-cli/) — generates platform-specific application projects
- [`uzumibi-gem`](./uzumibi-gem/) — defines `Uzumibi::Router`, `Request`, and `Response`
- [`uzumibi-art-router`](./uzumibi-art-router/) — route matching and path-parameter extraction
- [`uzumibi-cloudflare-ext`](./uzumibi-cloudflare-ext/) — Cloudflare host APIs exposed to Ruby
- [`uzumibi-google`](./uzumibi-google/) — Google Cloud integrations used by the Cloud Run template
- `uzumibi-on-*-spike` directories — development and integration examples for individual runtimes

## How to pronounce “Uzumibi”

Uzumibi (うずみび) is pronounced roughly as “oo-zoo-mee-bee.” The Japanese word refers to live embers kept under ash so that the fire does not go out.
