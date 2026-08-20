# How Platform Service APIs Work

WebAssembly code cannot directly call JavaScript promises or provider SDKs. Uzumibi adapter crates define Ruby methods and Wasm imports; the generated host implements those imports with the platform’s native APIs.

For Cloudflare Workers:

~~~text
Ruby API
   |
uzumibi-cloudflare-ext
   |
Wasm import
   |
generated src/index.js
   |
Workers API or binding
~~~

Cloudflare operations such as outbound fetch, KV access, and Queue sends are asynchronous. The `enable-external` build applies Asyncify with `wasm-opt` and uses `asyncify-wasm` so the Ruby call can suspend while JavaScript awaits the Workers API.

The base Cloudflare HTTP template does not enable those asynchronous imports. The `queue` feature enables the external Rust feature as part of the Queue build.

API names can be shared by another adapter, but their exact provider semantics and configuration may differ. Treat each platform guide as the compatibility contract.
