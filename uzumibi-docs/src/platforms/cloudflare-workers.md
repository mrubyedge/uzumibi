# Cloudflare Workers

Cloudflare Workers is a serverless platform that runs JavaScript and WebAssembly at the edge.

### Features

- **Global Distribution**: Runs in 300+ data centers worldwide
- **V8 Engine**: Fast JavaScript runtime with WebAssembly support
- **KV Storage**: Built-in key-value storage (TBA)
- **Durable Objects**: Stateful objects (TBA)
- **Low Cold Start**: Minimal startup latency

### Project Setup

Generate a new Cloudflare Workers project:

```bash
uzumibi new --template cloudflare my-app
cd my-app
```

### Configuration

Edit `wrangler.jsonc` to configure your Worker:

```jsonc
{
  "name": "my-app",
  "main": "src/index.js",
  "compatibility_date": "2024-01-01"
}
```

### Local Development

```bash
# Build WASM module
cd wasm-app
cargo build --target wasm32-unknown-unknown --release
cd ..

# Copy WASM file
cp target/wasm32-unknown-unknown/release/*.wasm public/app.wasm

# Start dev server
npx wrangler dev
```

### Deployment

```bash
npx wrangler login
npx wrangler deploy
```

### Limitations

- **CPU Time**: 50ms on free plan, 50ms-30s on paid plans
- **Memory**: 128MB
- **Request Size**: 100MB
- **Response Size**: Unlimited

### Platform-Specific Features

- Access to Cloudflare KV (TBA)
- Access to Cloudflare R2 (TBA)
- Access to Durable Objects (TBA)
