# Creating a Cloudflare Workers Project

Let's create a new Uzumibi application for Cloudflare Workers:

```bash
uzumibi new --template cloudflare my-uzumibi-app
cd my-uzumibi-app
```

This generates a new project with the following structure:

```
my-uzumibi-app/
├── Cargo.toml
├── build.rs
├── wrangler.jsonc
├── lib/
│   └── app.rb          # Your Ruby application
├── src/
│   └── index.js        # JavaScript entry point
└── wasm-app/
    ├── Cargo.toml
    └── src/
        └── lib.rs      # Rust WASM module
```

### Available Templates

The CLI supports the following templates:

- `cloudflare`: Cloudflare Workers
- `fastly`: Fastly Compute@Edge
- `spin`: Spin (Fermyon Cloud)
- `cloudrun`: Google Cloud Run (experimental)
