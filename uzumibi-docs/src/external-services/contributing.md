# Contributing Platform Integrations

Platform service APIs span multiple layers. A Cloudflare change may require coordinated updates to:

- `uzumibi-cloudflare-ext` for Ruby classes and Wasm imports
- `uzumibi-cli/templates/cloudflare` for JavaScript host functions and bindings
- the `enable-external` or `queue` feature overlay
- unit and runn integration tests
- this documentation

When adding an API, document the exact Ruby signature, required template feature, binding name, return value, buffer or encoding constraints, and failure behavior. Avoid documenting a planned API as available before its adapter and generated template are both implemented.

Use [GitHub issues](https://github.com/mrubyedge/uzumibi/issues) to discuss API design and compatibility.
