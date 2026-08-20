# Troubleshooting

## Cloudflare Wasm target is missing

~~~bash
rustup target add wasm32-unknown-unknown
~~~

## `wasm-opt` is not found

The `enable-external` and `queue` variants apply Asyncify and require Binaryen:

~~~bash
brew install binaryen
wasm-opt --version
~~~

## Ruby changes are not visible

Ruby bytecode is embedded at build time. Stop Wrangler and rerun:

~~~bash
pnpm run dev
~~~

## Wrangler rejects a placeholder binding

Feature templates include values such as `<YOUR_KV_NAMESPACE_ID>`. Create or select the Cloudflare resource and replace the placeholder in `wrangler.jsonc`.

## Requests return HTTP 413

The Cloudflare adapter rejects an encoded request larger than `uzumibi.httpMaxBytes` in `package.json`. Increase the value and rebuild the Wasm module. The value applies to the complete encoded request, not only the body.

## A route returns 404

Check the HTTP method and normalized path. The base router returns `404 Not Found` when no route matches. Cloudflare also returns 404 for `/favicon.ico` before invoking Ruby.
