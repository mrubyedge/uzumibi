# Deploying

## Cloudflare Workers

Authenticate Wrangler:

~~~bash
pnpm exec wrangler login
~~~

Then build and deploy:

~~~bash
pnpm run deploy
~~~

The deploy script selects the correct build mode for the generated template:

- base template: vanilla Wasm
- `enable-external`: Asyncify-enabled Wasm
- `queue`: Queue consumer Wasm

Before deploying a feature variant, replace placeholder resource IDs and create any Queue or KV resources referenced by `wrangler.jsonc`.

Cloudflare account limits and resource configuration can change. Refer to the [Cloudflare Workers documentation](https://developers.cloudflare.com/workers/) for platform policy and Wrangler configuration.

## Other templates

Deployment commands are platform-specific. Use the next steps printed by the CLI and the configuration generated for that template.
