# Introduction

Uzumibi lets you write request handlers in Ruby for WebAssembly-based edge and serverless runtimes.

An Uzumibi application has three main parts:

1. Ruby application code, normally `lib/app.rb`
2. The mruby/edge runtime and Uzumibi framework, compiled into a Wasm module
3. A platform adapter that transfers requests, responses, and optional host services between the platform and Wasm

Ruby code is compiled to mruby bytecode during the application build. It is not loaded from the filesystem at request time.

## Minimal application

~~~ruby
class App < Uzumibi::Router
  get "/" do |req, res|
    res.return(
      200,
      { "content-type" => "text/plain" },
      "Hello from Uzumibi!\n"
    )
  end
end

$APP = App.new
~~~

`Uzumibi::Router` dispatches by HTTP method and path. A handler receives an `Uzumibi::Request` and `Uzumibi::Response` and mutates the response. Ending the handler with `res` is the conventional style.

## What the CLI provides

The `uzumibi` CLI generates complete, platform-specific projects. The generated build and development commands differ by template; the CLI itself currently only provides the `new` command.

For Cloudflare Workers, the generated project uses pnpm and Wrangler:

~~~bash
uzumibi new --template cloudflare my-app
cd my-app
pnpm install
pnpm run dev
~~~

Continue with [Installation and Getting Started](./installation.md), or see the [Cloudflare Workers guide](./platforms/cloudflare-workers.md).
