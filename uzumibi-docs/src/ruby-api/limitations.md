# Limitations

Current core and adapter constraints include:

- Ruby code is compiled and embedded at build time.
- Available Ruby features are those implemented by mruby/edge and initialized crates, not the full CRuby standard library.
- Native CRuby extensions cannot be loaded into the Wasm runtime.
- Query parsing does not currently perform URL decoding.
- JSON and form parsing require exact supported content-type values.
- Response headers use 16-bit lengths and response bodies use a 32-bit length in the transport format.
- Platform service APIs are adapter-specific and often require a feature overlay.
- The Cloudflare adapter has its own configurable encoded-request limit and currently text-decodes response bodies.

See the selected [platform guide](../platforms.md) for build tools, bindings, and host-specific constraints.
