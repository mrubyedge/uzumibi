# CLI Reference

The Uzumibi CLI (`uzumibi`) is a command-line tool for scaffolding new edge application projects.

## Installation

Install via cargo:

```bash
cargo install uzumibi-cli
```

Verify installation:

```bash
uzumibi --version
```

## Commands

### `uzumibi new`

Create a new edge application project from a template.

#### Synopsis

```bash
uzumibi new --template <TEMPLATE> <PROJECT_NAME>
```

#### Arguments

- `<PROJECT_NAME>`: The name of your project. This will be used as the directory name.

#### Options

- `-t, --template <TEMPLATE>`: The platform template to use. **Required.**

#### Available Templates

| Template | Description | Status |
|----------|-------------|--------|
| `cloudflare` | Cloudflare Workers | Stable |
| `fastly` | Fastly Compute@Edge | Stable |
| `spin` | Spin (Fermyon) | Stable |
| `cloudrun` | Google Cloud Run | Experimental |

#### Examples

Create a Cloudflare Workers project:

```bash
uzumibi new --template cloudflare my-worker
```

Create a Fastly Compute project:

```bash
uzumibi new --template fastly my-compute-app
```

Create a Spin project:

```bash
uzumibi new --template spin my-spin-app
```

Create a Cloud Run project:

```bash
uzumibi new --template cloudrun my-cloudrun-app
```

#### What Gets Created

The `uzumibi new` command generates a complete project structure including:

- **Cargo.toml**: Rust workspace configuration
- **build.rs**: Build script that compiles Ruby to mruby bytecode
- **lib/app.rb**: Your Ruby application code (main entry point)
- **src/**: Platform-specific Rust/JavaScript code
- Platform-specific configuration files:
  - `wrangler.jsonc` (Cloudflare)
  - `fastly.toml` (Fastly)
  - `spin.toml` (Spin)
  - `Dockerfile` (Cloud Run)

Example project structure for Cloudflare Workers:

```
my-worker/
├── Cargo.toml
├── package.json
├── pnpm-lock.yaml
├── wrangler.jsonc
├── lib/
│   └── app.rb
├── src/
│   └── index.js
└── wasm-app/
    ├── Cargo.toml
    ├── build.rs
    └── src/
        └── lib.rs
```

### `uzumibi --help`

Display help information:

```bash
uzumibi --help
```

Output:

```
Uzumibi CLI - Create a new edge application project powered by Ruby

Usage: uzumibi <COMMAND>

Commands:
  new   Create a new edge application project
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### `uzumibi --version`

Display the CLI version:

```bash
uzumibi --version
```

## Project Templates

### Cloudflare Workers Template

Files included:
- `wrangler.jsonc`: Wrangler configuration
- `package.json`: Node.js dependencies (for Wrangler)
- `src/index.js`: JavaScript entry point
- `wasm-app/`: Rust WASM module source

Build with:
```bash
cd wasm-app
cargo build --target wasm32-unknown-unknown --release
```

Run locally:
```bash
npx wrangler dev
```

Deploy:
```bash
npx wrangler deploy
```

### Fastly Compute Template

Files included:
- `fastly.toml`: Fastly service configuration
- `Cargo.toml`: Rust project configuration
- `src/main.rs`: Application entry point
- `src/lib.rs`: WASM module

Build with:
```bash
cargo build --target wasm32-wasi --release
```

Run locally:
```bash
fastly compute serve
```

Deploy:
```bash
fastly compute deploy
```

### Spin Template

Files included:
- `spin.toml`: Spin application manifesthown
- `Cargo.toml`: Rust project configuration
- `src/lib.rs`: Application entry point

Build with:
```bash
spin build
```

Run locally:
```bash
spin up
```

Deploy:
```bash
spin deploy
```

### Cloud Run Template

Files included:
- `Dockerfile`: Container image definition
- `Cargo.toml`: Rust project configuration
- `src/main.rs`: HTTP server entry point
- `src/uzumibi.rs`: Uzumibi integration

Build with:
```bash
cargo build --release
```

Run locally:
```bash
cargo run
```

Deploy:
```bash
gcloud builds submit --tag gcr.io/PROJECT_ID/app
gcloud run deploy --image gcr.io/PROJECT_ID/app
```

## Common Workflows

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

## Troubleshooting

### "uzumibi: command not found"

Make sure `~/.cargo/bin` is in your PATH:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Add this to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.) to make it permanent.

### "Invalid template"

Check that you're using a valid template name:
- `cloudflare`
- `fastly`
- `spin`
- `cloudrun`

Template names are case-sensitive and must be lowercase.

### "Directory already exists"

The CLI won't overwrite existing directories. Either:
1. Choose a different project name
2. Remove the existing directory
3. Use a different location

## Environment Variables

### `CARGO_TARGET_DIR`

Override the default Cargo target directory:

```bash
export CARGO_TARGET_DIR=/path/to/target
uzumibi new --template cloudflare my-app
```

## Future Commands

Planned additions to the CLI:

- `uzumibi build`: Build the project for the current platform
- `uzumibi dev`: Start local development server
- `uzumibi deploy`: Deploy to the configured platform
- `uzumibi init`: Initialize Uzumibi in an existing project
- `uzumibi add-service`: Add external service integration

These commands are not yet implemented but are planned for future releases.

## Updating the CLI

Update to the latest version:

```bash
cargo install uzumibi-cli --force
```

## Getting Help

- Run `uzumibi --help` for command help
- Visit the [GitHub repository](https://github.com/mrubyedge/uzumibi)
- Check the [documentation](https://uzumibi.example.com) (TBA)
- Open an issue for bugs or feature requests
