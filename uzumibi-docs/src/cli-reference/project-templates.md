# Project Templates

### Cloudflare Workers Template

Files included:
- `wrangler.jsonc`: Wrangler configuration
- `package.json`: Node.js dependencies (for Wrangler)
- `src/index.js`: JavaScript entry point
- `wasm-app/`: Rust WASM module source

Build with:
```bash
cd wasm-app
cargo build --target wasm32-unknown-unknown --release
```

Run locally:
```bash
npx wrangler dev
```

Deploy:
```bash
npx wrangler deploy
```

### Fastly Compute Template

Files included:
- `fastly.toml`: Fastly service configuration
- `Cargo.toml`: Rust project configuration
- `src/main.rs`: Application entry point
- `src/lib.rs`: WASM module

Build with:
```bash
cargo build --target wasm32-wasi --release
```

Run locally:
```bash
fastly compute serve
```

Deploy:
```bash
fastly compute deploy
```

### Spin Template

Files included:
- `spin.toml`: Spin application manifesthown
- `Cargo.toml`: Rust project configuration
- `src/lib.rs`: Application entry point

Build with:
```bash
spin build
```

Run locally:
```bash
spin up
```

Deploy:
```bash
spin deploy
```

### Cloud Run Template

Files included:
- `Dockerfile`: Container image definition
- `Cargo.toml`: Rust project configuration
- `src/main.rs`: HTTP server entry point
- `src/uzumibi.rs`: Uzumibi integration

Build with:
```bash
cargo build --release
```

Run locally:
```bash
cargo run
```

Deploy:
```bash
gcloud builds submit --tag gcr.io/PROJECT_ID/app
gcloud run deploy --image gcr.io/PROJECT_ID/app
```
