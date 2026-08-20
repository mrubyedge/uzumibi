# Platform Comparison

This table describes the current Uzumibi templates. It does not attempt to reproduce provider pricing or runtime limits, which change independently of Uzumibi.

| Template | Generated host | Ruby HTTP routing | Optional service APIs | Event consumer |
| --- | --- | --- | --- | --- |
| `cloudflare` | JavaScript Worker + Wasm | Yes | `enable-external` | Cloudflare Queues with `queue` |
| `cloudrun` | Native Rust service + Docker | Yes | `enable-external` | Pub/Sub push with `queue` |
| `fastly` | Fastly Compute Rust app | Yes | No feature overlay | No |
| `spin` | Spin component | Yes | No feature overlay | No |
| `serviceworker` | Browser Service Worker + Wasm | Yes | No feature overlay | No |
| `webworker` | Browser Web Worker + Wasm | Yes | No feature overlay | No |

“Optional service APIs” means APIs implemented by that template’s adapter. Identical Ruby class names on different platforms can map to different provider services and are not a promise of full portability.

For provider limits and availability, consult the provider’s current documentation.
