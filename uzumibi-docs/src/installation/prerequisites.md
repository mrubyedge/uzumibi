# Prerequisites

All generated projects require a current stable Rust toolchain.

For Cloudflare Workers, install:

- Rust and Cargo
- the `wasm32-unknown-unknown` Rust target
- Node.js
- pnpm

~~~bash
rustup target add wasm32-unknown-unknown
npm install --global pnpm
~~~

The generated project installs Wrangler as a development dependency, so use it through `pnpm` or `pnpm exec`.

Projects generated with Cloudflare’s `enable-external` or `queue` feature also require Binaryen’s `wasm-opt`, because those builds apply Asyncify:

~~~bash
brew install binaryen
~~~

On other operating systems, install Binaryen using the packages or binaries listed by the [Binaryen project](https://github.com/WebAssembly/binaryen).

A Cloudflare account and Wrangler login are required for deployment, but not for creating a project.
