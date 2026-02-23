# Common Workflows

### Creating and Deploying a New App

```bash
# 1. Create project
uzumibi new --template cloudflare my-app

# 2. Navigate to project
cd my-app

# 3. Edit your Ruby code
vim lib/app.rb

# 4. Build WASM module
cd wasm-app
cargo build --target wasm32-unknown-unknown --release
cd ..

# 5. Test locally
npx wrangler dev

# 6. Deploy
npx wrangler deploy
```

### Updating Ruby Code

After modifying `lib/app.rb`, rebuild the WASM module:

```bash
cd wasm-app
cargo build --target wasm32-unknown-unknown --release
cd ..
```

The `build.rs` script automatically compiles your Ruby code to mruby bytecode during the Cargo build process.

### Adding Dependencies

To add Rust crates to your project:

```bash
cd wasm-app  # or project root for non-Cloudflare projects
cargo add <crate-name>
```

Note: mruby/Ruby dependencies are limited to what's available in the mruby/edge runtime.
