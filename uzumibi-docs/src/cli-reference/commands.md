# Commands

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
