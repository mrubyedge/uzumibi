# What are External Service Abstractions?

External Service Abstractions are unified APIs that allow your Uzumibi application to access platform-specific services (like key-value stores, caches, databases, etc.) through a common interface. This abstraction layer enables you to write code once and deploy to multiple platforms without platform-specific modifications.

Each edge platform provides different services with different APIs:
- Cloudflare Workers has KV, R2, Durable Objects
- Fastly Compute has KV Store, Edge Dictionary
- Spin has Key-Value Store, SQLite
- Cloud Run can access Google Cloud services

External Service Abstractions provide a unified Ruby API that translates to the appropriate platform-specific implementation at runtime.

### Benefits

- **Write Once, Deploy Anywhere**: Same code works across platforms
- **Platform Independence**: Switch platforms without rewriting service access code
- **Consistent API**: Familiar Ruby interface regardless of underlying platform
- **Type Safety**: Well-defined interfaces reduce errors

### Status

**Current Status**: TBA (To Be Announced)

The External Service Abstractions layer is currently under development. The following sections describe the planned architecture and APIs.
