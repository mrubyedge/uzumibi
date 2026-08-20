# Project Templates

Each template is a complete platform adapter, not only a deployment configuration.

| Template | Host |
| --- | --- |
| `cloudflare` | Cloudflare Workers JavaScript host plus a Rust Wasm crate |
| `cloudrun` | Native Rust HTTP server packaged with Docker |
| `fastly` | Fastly Compute Rust application |
| `spin` | Spin component |
| `serviceworker` | Browser Service Worker example |
| `webworker` | Browser Web Worker example |

The CLI replaces project-name placeholders while copying the selected template. A feature is an overlay that replaces or adds files after the base template is copied.

## Cloudflare build scripts

A generated Cloudflare project provides:

| Script | Behavior |
| --- | --- |
| `pnpm run dev` | Build the selected Wasm mode and run Wrangler |
| `pnpm run deploy` | Build the selected Wasm mode and deploy with Wrangler |
| `pnpm start` | Run Wrangler without rebuilding |
| `pnpm test` | Run the JavaScript tests with Vitest |

The exact Wasm build script name depends on the selected feature: `build:wasm:vanilla`, `build:wasm:asyncify`, or `build:wasm:queue`.

See [Cloudflare Workers](../platforms/cloudflare-workers.md) for the generated layout and configuration.
