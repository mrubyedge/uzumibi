# Spin (Fermyon Cloud)

Spin is an open-source framework for building and running serverless WebAssembly applications.

### Features

- **Component Model**: Uses WebAssembly Component Model
- **Open Source**: Run anywhere that supports Spin
- **Fermyon Cloud**: Managed hosting platform
- **Key-Value Store**: Built-in KV storage (TBA)
- **SQLite**: Embedded database (TBA)

### Project Setup

Generate a new Spin project:

```bash
uzumibi new --template spin my-app
cd my-app
```

### Configuration

Edit `spin.toml`:

```toml
spin_manifest_version = 2

[application]
name = "my-app"
version = "0.1.0"
authors = ["Your Name <your.email@example.com>"]

[[trigger.http]]
route = "/..."
component = "my-app"

[component.my-app]
source = "target/wasm32-wasi/release/my_app.wasm"
allowed_outbound_hosts = []
[component.my-app.build]
command = "cargo build --target wasm32-wasi --release"
```

### Local Development

```bash
# Build and run
spin build
spin up
```

### Deployment

Deploy to Fermyon Cloud:

```bash
spin login
spin deploy
```

Or run on your own infrastructure using any Spin-compatible runtime.

### Limitations

- **Execution Time**: Platform-dependent
- **Memory**: Platform-dependent
- **Component Model**: Uses newer WASI preview 2 (compatibility varies)

### Platform-Specific Features

- Access to Spin KV Store (TBA)
- Access to SQLite (TBA)
- Redis integration (TBA)
