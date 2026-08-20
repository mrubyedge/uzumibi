# Project Structure

The repository is a Cargo workspace containing the framework, platform adapters, CLI, and integration projects.

| Path | Purpose |
| --- | --- |
| `uzumibi-cli` | CLI and embedded project templates |
| `uzumibi-gem` | Core Ruby request, response, and router API |
| `uzumibi-art-router` | Route matching implementation |
| `uzumibi-cloudflare-ext` | Cloudflare-specific Ruby host APIs |
| `uzumibi-google` | Google Cloud integrations |
| `uzumibi-docs` | This mdBook |
| `uzumibi-on-*-spike` | Platform integration and development examples |

Generated application layouts are platform-specific. A generated Cloudflare project contains a JavaScript Worker at the root and a Rust Wasm crate under `wasm-app`; other templates may be a single Rust crate.

Use the generated project’s own scripts and configuration as the source of truth. The spike directories are useful for development, but applications should normally be created with `uzumibi new`.
