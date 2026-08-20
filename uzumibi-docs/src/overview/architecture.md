# Architecture

Uzumibi separates the Ruby application from the platform-specific host.

~~~text
HTTP request or platform event
            |
            v
Platform adapter (JavaScript or Rust)
            |
     compact byte buffer
            |
            v
Wasm module
  + mruby/edge VM
  + uzumibi-gem
  + embedded Ruby bytecode
            |
            v
Ruby Router or Queue Consumer
~~~

## Build time

The generated Rust build script compiles the Ruby source into mruby bytecode and embeds it in the Wasm module or native application. Changing Ruby code therefore requires a rebuild.

## Runtime

For HTTP applications, the platform adapter serializes the request method, path, query string, selected headers, and body into a buffer. `uzumibi-gem` constructs an `Uzumibi::Request`, dispatches the matching route, and serializes the returned `Uzumibi::Response`.

The transport is implemented separately for each template:

- Cloudflare Workers uses a JavaScript Worker around a `wasm32-unknown-unknown` module.
- Fastly and Spin run WASI-oriented Rust adapters.
- Cloud Run runs a native Rust HTTP server.
- Service Worker and Web Worker templates use browser JavaScript hosts.

## Optional host calls

Some operations require calling back from Wasm into the host. On Cloudflare, the `enable-external` and `queue` features build the Wasm module with Asyncify so Ruby code can wait for asynchronous Workers APIs such as `fetch`, KV, and Queues.

These APIs are platform adapter features, not universal `uzumibi-gem` APIs.
