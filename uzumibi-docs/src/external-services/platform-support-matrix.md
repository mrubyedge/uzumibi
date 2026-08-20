# Feature Support Matrix

This matrix describes the generated Cloudflare variants.

| Capability | Base | `enable-external` | `queue` |
| --- | --- | --- | --- |
| HTTP `Uzumibi::Router` application | Yes | Yes | No; HTTP returns 400 |
| `debug_console` | Yes | Yes | Yes |
| `fetch_assets` | Yes | Yes | Not used by the event consumer |
| `Uzumibi::Fetch.fetch` | No | Yes | Yes |
| `Uzumibi::KV.get/set` | No | Yes | Yes |
| `Uzumibi::LegacyKV.get/set` | No | Yes | Yes |
| `Uzumibi::Secret.get` | No | Yes | Yes |
| `Uzumibi::Queue.send` | No | Yes, with a producer binding | Yes |
| `Uzumibi::Consumer` and `Message` | No | No | Yes |
| Asyncify / `wasm-opt` required | No | Yes | Yes |

Other platform adapters have their own feature sets. Refer to their generated files and platform guides rather than inferring support from this table.
