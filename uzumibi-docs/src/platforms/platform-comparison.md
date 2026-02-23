# Platform Comparison

| Feature | Cloudflare Workers | Fastly Compute | Spin | Cloud Run | Service Worker |
|---------|-------------------|----------------|------|-----------|----------------|
| **Execution Model** | V8 Isolates | WASI | WASI | Container | Browser |
| **Cold Start** | Very Fast | Very Fast | Fast | Slower | N/A |
| **Max Execution Time** | 50ms-30s | 60s | Varies | 60min | Varies |
| **Memory Limit** | 128MB | 128-512MB | Varies | 4GB+ | Browser |
| **Global Distribution** | Yes | Yes | Platform-dependent | Regional | N/A |
| **Cost Model** | Per-request | Per-request | Platform-dependent | Per-request + compute | Free |
| **Maturity** | Stable | Stable | Stable | Experimental | Experimental |
