# Cloud Run

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
