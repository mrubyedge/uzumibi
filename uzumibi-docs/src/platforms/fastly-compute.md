# Fastly Compute

Fastly Compute@Edge runs WebAssembly workloads at the edge using the WASI interface.

### Features

- **Global CDN**: Runs on Fastly's global edge network
- **WASI Support**: Standard WebAssembly System Interface
- **High Performance**: Near-native execution speed
- **Edge Dictionary**: Configuration storage (TBA)
- **KV Store**: Key-value storage (TBA)

### Project Setup

Generate a new Fastly Compute project:

```bash
uzumibi new --template fastly my-app
cd my-app
```

### Configuration

Edit `fastly.toml`:

```toml
name = "my-app"
description = "Uzumibi application on Fastly Compute"
authors = ["Your Name <your.email@example.com>"]
language = "rust"

[local_server]
  [local_server.backends]
    [local_server.backends.backend_name]
      url = "http://httpbin.org"
```

### Local Development

```bash
# Build
cargo build --target wasm32-wasi --release

# Run locally
fastly compute serve
```

### Deployment

```bash
fastly compute deploy
```

### Limitations

- **Execution Time**: Up to 60 seconds
- **Memory**: Configurable, typically 128MB-512MB
- **Request Size**: 8KB headers, unlimited body
- **Response Size**: Unlimited

### Platform-Specific Features

- Access to Fastly KV Store (TBA)
- Access to Edge Dictionary (TBA)
- Backend requests configuration (TBA)
