# Running Locally

## Cloudflare Workers

From a generated Cloudflare project:

~~~bash
pnpm install
pnpm run dev
~~~

`pnpm run dev` runs the template’s Wasm build script and then starts Wrangler. The default local URL is printed by Wrangler, normally `http://localhost:8787`.

After changing `lib/app.rb`, `lib/consumer.rb`, a Rust dependency, or `package.json` build configuration, stop and rerun `pnpm run dev` so the Wasm module is rebuilt.

Use `pnpm start` only when the Wasm output already exists and you intentionally want to start Wrangler without rebuilding.

The `enable-external` and `queue` variants require `wasm-opt`. They may also require valid KV, Durable Object, or Queue bindings in `wrangler.jsonc`.

## Other templates

Generated commands differ by platform. Run `uzumibi new` and follow the “Next steps” printed by the CLI, then consult the generated configuration files.
