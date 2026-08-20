# Available Services

## Cloudflare Workers

Generate an HTTP application with these APIs using:

~~~bash
uzumibi new --template cloudflare --features enable-external my-app
~~~

### Outbound HTTP

~~~ruby
Uzumibi::Fetch.fetch(url, method = "GET", body = "", headers = {})
~~~

Returns an `Uzumibi::Response` with `status_code`, `headers`, and `body`.

### Workers KV

~~~ruby
Uzumibi::KV.get(key)          # String or nil
Uzumibi::KV.set(key, value)   # true
~~~

The generated host uses the `UZUMIBI_KV` binding. Only `get` and `set` are currently implemented.

### Durable Object storage

~~~ruby
Uzumibi::LegacyKV.get(key)
Uzumibi::LegacyKV.set(key, value)
~~~

The generated project defines a single `UzumibiKVObject` instance named `default`. This compatibility API is separate from Workers KV.

### Environment bindings and secrets

~~~ruby
Uzumibi::Secret.get(name)     # String or nil
~~~

The host looks up `env[name]`. Configure sensitive values with Wrangler secrets rather than committing them.

### Queue producer

~~~ruby
Uzumibi::Queue.send(binding_name, message)
~~~

`binding_name` is the producer binding in `wrangler.jsonc`, for example `"UZUMIBI_QUEUE"`. The message is converted to a String.

### Cloudflare Access identity

~~~ruby
Uzumibi::Access.team = "my-team"
identity = Uzumibi::Access.get_identity(token)
~~~

The result is an `Uzumibi::AccessIdentity` with `user_uuid`, `email`, and `raw_data`.

### Static assets

`fetch_assets` is available in every Cloudflare build. It exits Ruby routing and delegates the original request to the generated `ASSETS` binding.

## Queue consumer

Generate with:

~~~bash
uzumibi new --template cloudflare --features queue my-consumer
~~~

The Queue API adds:

- `Uzumibi::Consumer#on_receive(message)`
- `Uzumibi::Message#id`
- `Uzumibi::Message#timestamp`
- `Uzumibi::Message#body`
- `Uzumibi::Message#attempts`
- `Uzumibi::Message#ack!`
- `Uzumibi::Message#nack!`
- `Uzumibi::Message#retry(delay_seconds: N)`

## Not currently implemented by the Cloudflare adapter

The current adapter does not define general-purpose `Cache`, `ObjectStore`, or `SQL` Ruby classes. It also does not implement Workers KV delete, list, or metadata operations.
