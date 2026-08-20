# Cloudflare Workers

The Cloudflare template runs an Uzumibi Wasm module inside a JavaScript Worker. The generated project owns both sides of the boundary: Rust and embedded Ruby in `wasm-app`, and the Workers host in `src/index.js`.

## Create a project

For an HTTP application without asynchronous host calls:

~~~bash
uzumibi new --template cloudflare my-app
cd my-app
pnpm install
pnpm run dev
~~~

The generated scripts are:

| Command | Purpose |
| --- | --- |
| `pnpm run dev` | Build vanilla Wasm and start Wrangler |
| `pnpm run deploy` | Build vanilla Wasm and deploy |
| `pnpm start` | Start Wrangler without rebuilding Wasm |
| `pnpm test` | Run JavaScript tests |

Edit `lib/app.rb` and restart `pnpm run dev` after a change.

## Runtime architecture

For each HTTP request:

1. `src/index.js` reads the Workers `Request`.
2. `src/request-buffer.js` encodes the method, pathname, query string, selected headers, and body.
3. The Rust Wasm export allocates a shared-memory region of the exact encoded size.
4. `uzumibi-gem` constructs an `Uzumibi::Request` and dispatches `$APP`.
5. The returned `Uzumibi::Response` is packed into Wasm memory.
6. JavaScript reads the status, headers, and body and creates a Workers `Response`.

The mruby/edge VM is initialized lazily and retained by the Wasm instance.

## HTTP request size

The adapter has its own encoded-request limit in addition to Cloudflare’s account and platform limits. The default is 65,536 bytes.

The limit is stored persistently in `package.json`:

~~~json
{
  "uzumibi": {
    "httpMaxBytes": 65536
  }
}
~~~

The value covers the complete encoded request: framing, method, path, query string, included headers, and body. A request over the configured value receives HTTP 413 before Ruby routing begins.

The build script validates a positive integer up to 2,147,483,647 and embeds it in the Wasm module. Rebuild after changing it:

~~~bash
pnpm run dev
~~~

For a one-off build, use either an environment variable or a script option:

~~~bash
UZUMIBI_HTTP_MAX_BYTES=1048576 pnpm run build:wasm:vanilla
node scripts/build-wasm.mjs vanilla --http-max-bytes=1048576
~~~

The option has precedence over the environment variable, which has precedence over `package.json`. Increasing the limit permits a larger allocation; it does not change Cloudflare’s own request or memory limits.

## Request and response behavior

- `req.params` combines path parameters, query parameters, and supported parsed body parameters.
- An exact `application/json` content type parses a JSON object into `req.body` and merges its top-level fields into `req.params`.
- `req.raw_body` preserves the original request body as a Ruby String.
- An exact `application/x-www-form-urlencoded` content type merges form fields into `req.params`.
- The current Workers adapter omits `cf-connecting-ip`, `cf-ray`, and headers beginning with `x-` before passing headers to Ruby.
- A response body is a Ruby String. The current JavaScript adapter decodes it as text when constructing the Workers response; arbitrary binary response bytes are not yet preserved transparently.

Consult [Cloudflare Workers limits](https://developers.cloudflare.com/workers/platform/limits/) for current platform limits.

## Static assets

The base `wrangler.jsonc` binds the generated `public` directory as `ASSETS`. Call `fetch_assets` from a route to delegate the original request to `env.ASSETS.fetch(request)`:

~~~ruby
get "/assets/*" do |req, res|
  fetch_assets
end
~~~

Cloudflare may serve a matching static asset before invoking the Worker depending on the current `assets` routing configuration. See [Workers Static Assets configuration](https://developers.cloudflare.com/workers/static-assets/binding/).

## External-service feature

Generate an HTTP application with asynchronous Workers APIs:

~~~bash
uzumibi new --template cloudflare --features enable-external my-app
~~~

This variant:

- enables `uzumibi-cloudflare-ext/enable-external`
- uses `asyncify-wasm` at runtime
- runs `wasm-opt --asyncify` during the build
- includes KV and Durable Object binding examples in `wrangler.jsonc`

Install Binaryen before building:

~~~bash
brew install binaryen
~~~

The following Ruby APIs are currently defined:

| API | Workers operation |
| --- | --- |
| `Uzumibi::Fetch.fetch(url, method = "GET", body = "", headers = {})` | outbound `fetch` |
| `Uzumibi::KV.get(key)` / `.set(key, value)` | `UZUMIBI_KV` Workers KV binding |
| `Uzumibi::LegacyKV.get(key)` / `.set(key, value)` | generated `UzumibiKVObject` Durable Object |
| `Uzumibi::Secret.get(name)` | Worker environment binding with that name |
| `Uzumibi::Queue.send(binding_name, message)` | Queue producer binding |
| `Uzumibi::Access.team=` / `.get_identity(token)` | Cloudflare Access identity endpoint |

See [Cloudflare Access identity](../external-services/cloudflare-access.md) for setup and request handling.

Example outbound request:

~~~ruby
response = Uzumibi::Fetch.fetch(
  "https://example.com/api",
  "POST",
  JSON.generate({ "hello" => "world" }),
  { "content-type" => "application/json" }
)
~~~

Example KV access:

~~~ruby
Uzumibi::KV.set("greeting", "hello")
value = Uzumibi::KV.get("greeting")
~~~

Create a KV namespace and replace `<YOUR_KV_NAMESPACE_ID>` in `wrangler.jsonc`:

~~~bash
pnpm exec wrangler kv namespace create UZUMIBI_KV
~~~

For Queue producers, `Uzumibi::Queue.send` takes the Wrangler binding name, such as `"UZUMIBI_QUEUE"`, rather than the Cloudflare resource name.

## Queue consumer feature

Generate a Queue consumer:

~~~bash
uzumibi new --template cloudflare --features queue my-consumer
cd my-consumer
pnpm install
pnpm exec wrangler queues create my-consumer-queue
pnpm run dev
~~~

The generated `wrangler.jsonc` uses `my-consumer-queue` as both a producer and consumer resource. The Queue feature includes the external-service feature.

Implement the consumer in `lib/consumer.rb`:

~~~ruby
class Consumer < Uzumibi::Consumer
  def on_receive(message)
    debug_console("received #{message.id}: #{message.body}")

    if message.attempts > 3
      message.ack!
    else
      message.retry(delay_seconds: 3)
    end
  end
end

$CONSUMER = Consumer.new
~~~

`Uzumibi::Message` exposes `id`, `timestamp`, `body`, and `attempts`, plus `ack!`, `nack!`, and `retry(delay_seconds: N)`.

The Queue template is event-oriented. Ordinary HTTP requests are intentionally rejected with HTTP 400.

See the [Cloudflare Queues Wrangler commands](https://developers.cloudflare.com/queues/reference/wrangler-commands/) for current resource-management commands.

## Current adapter constraints

- The base HTTP build cannot call asynchronous external Workers APIs; use `enable-external`.
- External fetch and KV reads currently use fixed 64 KiB host-call result buffers.
- Secret reads currently use an 8 KiB result buffer.
- Responses are text-decoded by the JavaScript adapter.
- The Queue consumer processes messages one at a time inside each delivered batch.

These are Uzumibi adapter constraints and are separate from Cloudflare account limits.
