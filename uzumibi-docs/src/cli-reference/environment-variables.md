# Environment Variables

The `uzumibi` CLI does not define custom environment variables for project generation.

Generated projects may read platform-specific or build-specific variables. The Cloudflare Wasm build recognizes:

| Variable | Purpose |
| --- | --- |
| `UZUMIBI_HTTP_MAX_BYTES` | One-build override for the maximum encoded HTTP request size |
| `CARGO_TARGET_DIR` | Standard Cargo target-directory override |

For a persistent HTTP-size setting, edit `uzumibi.httpMaxBytes` in the generated `package.json`. The precedence is:

1. `--http-max-bytes` passed to `scripts/build-wasm.mjs`
2. `UZUMIBI_HTTP_MAX_BYTES`
3. `package.json`
4. the built-in default of 65536
