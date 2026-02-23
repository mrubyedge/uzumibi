# Architecture

Uzumibi's architecture consists of several key components:

### Core Components

```
┌─────────────────────────────────────────┐
│         Edge Platform                   │
│  (Cloudflare Workers, Fastly, Spin)    │
└─────────────┬───────────────────────────┘
              │
              │ HTTP Request
              ▼
┌─────────────────────────────────────────┐
│     Platform-Specific Runtime           │
│  (WASM Host Environment)                │
└─────────────┬───────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────┐
│      Uzumibi WASM Module                │
│  ┌───────────────────────────────────┐  │
│  │    mruby/edge Runtime             │  │
│  │  ┌─────────────────────────────┐  │  │
│  │  │   Your Ruby Application     │  │  │
│  │  │   (Uzumibi::Router)         │  │  │
│  │  └─────────────────────────────┘  │  │
│  └───────────────────────────────────┘  │
└─────────────┬───────────────────────────┘
              │
              │ HTTP Response
              ▼
```

### Component Layers

1. **uzumibi-cli**: Command-line tool for generating project scaffolds
2. **uzumibi-gem**: Core framework providing the Router class and request/response handling
3. **uzumibi-art-router**: Lightweight router library for path matching and parameter extraction
4. **mruby/edge**: Ruby runtime optimized for edge computing
5. **Platform Adapters**: Platform-specific code for each edge provider

### Request Flow

1. HTTP request arrives at the edge platform
2. Platform routes to WASM module
3. Request data is serialized and passed to mruby/edge
4. Your Router class processes the request
5. Response is generated and serialized
6. Platform sends HTTP response to client
