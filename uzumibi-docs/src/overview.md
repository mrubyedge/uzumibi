# Overview

## What is Uzumibi?

Uzumibi is a lightweight web application framework designed for edge computing platforms. The name "Uzumibi" (うずみび) is a Japanese term that refers to live embers buried under a layer of ash to keep the fire from going out. This metaphor represents how Uzumibi keeps Ruby alive in the constrained environments of edge computing.

The framework enables developers to:

- Write serverless applications in Ruby for edge platforms
- Deploy to multiple edge providers (Cloudflare Workers, Fastly Compute, Spin, etc.)
- Build high-performance applications optimized for WebAssembly
- Use a familiar Sinatra-like routing DSL

## What is mruby/edge?

[mruby/edge](https://github.com/mrubyedge/mrubyedge) is a specialized implementation of mruby, optimized specifically for edge computing scenarios. It's designed to run efficiently in WebAssembly environments with limited resources.

Key features of mruby/edge:

- **Optimized for WebAssembly**: Compiled to WASM for fast startup and low memory footprint
- **Minimal Runtime**: Stripped-down Ruby implementation suitable for edge environments
- **No GC Overhead**: Carefully managed memory allocation for predictable performance
- **Edge-Optimized**: Built specifically for constrained computing environments

## Architecture

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

## Project Structure

The Uzumibi project consists of several crates:

- **uzumibi-cli**: CLI tool for project generation
- **uzumibi-gem**: Core framework functionality
- **uzumibi-art-router**: Routing library
- **uzumibi-on-*-spike**: Example implementations for each platform

Each spike project demonstrates how to integrate Uzumibi with a specific edge platform.
