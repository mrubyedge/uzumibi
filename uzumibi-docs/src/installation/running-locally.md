# Running Locally

### Cloudflare Workers

First, build the WASM module:

```bash
cd wasm-app
cargo build --target wasm32-unknown-unknown --release
cd ..
```

Copy the WASM file to the appropriate location:

```bash
cp target/wasm32-unknown-unknown/release/uzumibi_cloudflare_app.wasm public/app.wasm
```

Then start the development server:

```bash
npx wrangler dev
```

Your application will be available at `http://localhost:8787`.

### Fastly Compute

Build the project:

```bash
cargo build --target wasm32-wasi --release
```

Run locally:

```bash
fastly compute serve
```

### Spin

Build and run:

```bash
spin build
spin up
```
