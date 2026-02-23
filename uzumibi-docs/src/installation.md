# Installation and Getting Started

This guide will walk you through installing Uzumibi and creating your first edge application.

## Prerequisites

Before you begin, make sure you have:

- Rust toolchain (1.70 or later)
- `wasm32-unknown-unknown` target installed
- Platform-specific tools (e.g., `wrangler` for Cloudflare Workers)

## Installing via cargo

Install the Uzumibi CLI tool using cargo:

```bash
cargo install uzumibi-cli
```

This will install the `uzumibi` command-line tool, which you can use to generate new projects.

To verify the installation:

```bash
uzumibi --version
```

## Creating a Cloudflare Workers Project

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

## Editing Ruby Files

Open `lib/app.rb` and define your routes:

```ruby
class App < Uzumibi::Router
  get "/" do |req, res|
    res.status_code = 200
    res.headers = {
      "Content-Type" => "text/plain",
      "X-Powered-By" => "#{RUBY_ENGINE} #{RUBY_VERSION}"
    }
    res.body = "Hello from Uzumibi on the edge!\n"
    res
  end

  get "/hello/:name" do |req, res|
    res.status_code = 200
    res.headers = { "Content-Type" => "text/plain" }
    res.body = "Hello, #{req.params[:name]}!\n"
    res
  end

  post "/data" do |req, res|
    res.status_code = 200
    res.headers = { "Content-Type" => "application/json" }
    res.body = JSON.generate({ received: req.body })
    res
  end
end

$APP = App.new
```

The Ruby code is compiled to mruby bytecode during the build process and embedded into the WASM module.

## Running Locally

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

## Deploying

### Cloudflare Workers

Make sure you have a Cloudflare account and `wrangler` configured:

```bash
npx wrangler login
```

Deploy your application:

```bash
npx wrangler deploy
```

### Fastly Compute

Deploy to Fastly:

```bash
fastly compute deploy
```

### Spin (Fermyon Cloud)

Deploy to Fermyon Cloud:

```bash
spin deploy
```

## Next Steps

- Learn about the [Ruby API](./ruby-api.md) for routing and request/response handling
- Explore [supported platforms](./platforms.md) and platform-specific features
- Check out [external service abstractions](./external-services.md) for KV stores, caching, etc.

## Troubleshooting

### Build Errors

If you encounter build errors, make sure:

1. The `wasm32-unknown-unknown` target is installed:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

2. For Fastly, install the `wasm32-wasi` target:
   ```bash
   rustup target add wasm32-wasi
   ```

### Ruby Code Not Updating

The Ruby code is compiled at build time. After changing `lib/app.rb`, you need to rebuild the WASM module:

```bash
cargo build --target wasm32-unknown-unknown --release
```

### WASM Module Too Large

To reduce WASM module size:

1. Use release builds (already configured)
2. Strip debug symbols (already configured)
3. Minimize Ruby code and dependencies
