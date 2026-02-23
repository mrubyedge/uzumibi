# Troubleshooting

### Build Errors

If you encounter build errors, make sure:

1. The `wasm32-unknown-unknown` target is installed:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

2. For Fastly, install the `wasm32-wasi` target:
   ```bash
   rustup target add wasm32-wasi
   ```

### Ruby Code Not Updating

The Ruby code is compiled at build time. After changing `lib/app.rb`, you need to rebuild the WASM module:

```bash
cargo build --target wasm32-unknown-unknown --release
```

### WASM Module Too Large

To reduce WASM module size:

1. Use release builds (already configured)
2. Strip debug symbols (already configured)
3. Minimize Ruby code and dependencies
