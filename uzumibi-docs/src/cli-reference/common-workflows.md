# Common Workflows

## Create a Cloudflare HTTP application

~~~bash
uzumibi new --template cloudflare my-app
cd my-app
pnpm install
pnpm run dev
~~~

Edit `lib/app.rb`, then restart `pnpm run dev` to rebuild the embedded Ruby bytecode.

## Enable Cloudflare host APIs

~~~bash
uzumibi new --template cloudflare --features enable-external my-app
cd my-app
pnpm install
~~~

Install `wasm-opt`, configure the bindings in `wrangler.jsonc`, then run `pnpm run dev`.

## Create a Cloudflare Queue consumer

~~~bash
uzumibi new --template cloudflare --features queue my-consumer
cd my-consumer
pnpm install
pnpm exec wrangler queues create my-consumer-queue
pnpm run dev
~~~

Implement `Consumer#on_receive` in `lib/consumer.rb`. Ensure the queue name and bindings in `wrangler.jsonc` match the resource you created.

## Update an existing generated project

Templates are copied at generation time; upgrading `uzumibi-cli` does not rewrite an existing application. Generate a temporary project with the same template and feature, compare it with your application, and apply the changes you need.
