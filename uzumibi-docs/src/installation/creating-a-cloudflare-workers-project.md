# Creating a Cloudflare Workers Project

Generate the base HTTP template:

~~~bash
uzumibi new --template cloudflare my-uzumibi-app
cd my-uzumibi-app
pnpm install
~~~

The generated project has this structure:

~~~text
my-uzumibi-app/
├── Cargo.toml
├── package.json
├── pnpm-lock.yaml
├── wrangler.jsonc
├── lib/
│   └── app.rb
├── public/
│   └── assets/
├── scripts/
│   └── build-wasm.mjs
├── src/
│   ├── index.js
│   └── request-buffer.js
├── test/
│   └── request-buffer.spec.js
└── wasm-app/
    ├── Cargo.toml
    ├── build.rs
    └── src/
        └── lib.rs
~~~

- `lib/app.rb` is the Ruby application.
- `wasm-app/build.rs` compiles and embeds the Ruby bytecode.
- `src/index.js` is the Workers entry point and Wasm host.
- `scripts/build-wasm.mjs` selects the build mode and embeds configuration.
- `wrangler.jsonc` configures the Worker and static-assets binding.

## Feature variants

Enable asynchronous Cloudflare host APIs:

~~~bash
uzumibi new --template cloudflare --features enable-external my-app
~~~

Create a Cloudflare Queues consumer:

~~~bash
uzumibi new --template cloudflare --features queue my-consumer
~~~

The Queue variant uses `lib/consumer.rb` and `$CONSUMER` instead of `lib/app.rb` and `$APP`. The `queue` feature includes the external-service APIs.

See [Cloudflare Workers](../platforms/cloudflare-workers.md) for configuration and feature details.
