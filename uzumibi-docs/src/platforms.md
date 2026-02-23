# Supported Platforms

Uzumibi supports deployment to multiple edge computing platforms. Each platform has its own characteristics, deployment process, and limitations.

## Cloudflare Workers

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

## Fastly Compute

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

## Spin (Fermyon Cloud)

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

## Cloud Run

Google Cloud Run is a managed compute platform that automatically scales your containers.

**Status**: Experimental

### Features

- **Container-Based**: Runs standard OCI containers
- **Auto-Scaling**: Scales to zero and up based on traffic
- **HTTP/2**: Full HTTP/2 support
- **Long-Running**: Supports long execution times
- **Google Cloud Integration**: Access to GCP services

### Project Setup

Generate a new Cloud Run project:

```bash
uzumibi new --template cloudrun my-app
cd my-app
```

### Configuration

The project includes a `Dockerfile` for containerization:

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/my-app /usr/local/bin/my-app
CMD ["my-app"]
```

### Local Development

```bash
# Build and run locally
cargo run
```

The server will start on `http://localhost:8080`.

### Deployment

```bash
# Build container
gcloud builds submit --tag gcr.io/PROJECT_ID/my-app

# Deploy to Cloud Run
gcloud run deploy my-app \
  --image gcr.io/PROJECT_ID/my-app \
  --platform managed \
  --region us-central1 \
  --allow-unauthenticated
```

### Limitations

- **Cold Start**: Higher cold start latency compared to edge platforms
- **Cost**: Billed per request and compute time
- **Not Edge**: Runs in regional data centers, not at the edge

### Platform-Specific Features

- Access to Google Cloud Storage (TBA)
- Access to Cloud SQL (TBA)
- Access to Firestore (TBA)

## Service Worker/Web Worker (Experimental)

Run Uzumibi directly in the browser using Service Workers or Web Workers.

**Status**: Experimental - For demonstration and testing purposes

### Features

- **Browser-Based**: Runs entirely in the browser
- **Offline Support**: Service Workers enable offline functionality
- **Client-Side Routing**: Handle requests without a server
- **Development Tool**: Useful for testing and development

### Project Structure

The Service Worker spike project demonstrates:

- Loading WASM in a Service Worker
- Intercepting fetch requests
- Processing requests through Uzumibi
- Returning responses to the browser

### Use Cases

- **Offline-First Apps**: Progressive Web Apps with offline routing
- **Development/Testing**: Test Uzumibi logic in the browser
- **Client-Side APIs**: Mock APIs or client-side data processing
- **Educational**: Learn how Uzumibi works

### Limitations

- **Browser Only**: Not suitable for production server workloads
- **Security Restrictions**: Subject to browser security policies
- **Limited Storage**: Browser storage APIs only
- **Performance**: May be slower than server-side execution

### How It Works

1. Register Service Worker
2. Service Worker loads WASM module
3. Intercept fetch events
4. Route through Uzumibi Router
5. Return response to page

See the [uzumibi-on-serviceworker-spike](https://github.com/mrubyedge/uzumibi/tree/main/uzumibi-on-serviceworker-spike) directory for the complete implementation.

## Platform Comparison

| Feature | Cloudflare Workers | Fastly Compute | Spin | Cloud Run | Service Worker |
|---------|-------------------|----------------|------|-----------|----------------|
| **Execution Model** | V8 Isolates | WASI | WASI | Container | Browser |
| **Cold Start** | Very Fast | Very Fast | Fast | Slower | N/A |
| **Max Execution Time** | 50ms-30s | 60s | Varies | 60min | Varies |
| **Memory Limit** | 128MB | 128-512MB | Varies | 4GB+ | Browser |
| **Global Distribution** | Yes | Yes | Platform-dependent | Regional | N/A |
| **Cost Model** | Per-request | Per-request | Platform-dependent | Per-request + compute | Free |
| **Maturity** | Stable | Stable | Stable | Experimental | Experimental |

## Choosing a Platform

Consider these factors when choosing a platform:

- **Global Performance**: Cloudflare Workers or Fastly Compute for worldwide low latency
- **Execution Time**: Cloud Run if you need longer execution times
- **Open Source**: Spin for self-hosted or cloud-agnostic deployments
- **Cost**: Compare pricing for your expected traffic patterns
- **Integration**: Choose based on existing cloud provider relationships
- **Development**: Service Worker for local testing and development

## Next Steps

- Learn about [External Service Abstractions](./external-services.md) for accessing platform-specific features
- Check out [Examples](./examples.md) for platform-specific example code
