# Choosing a Platform

Choose a template based first on its execution host and the integration you need:

- Choose `cloudflare` for a Workers application, Workers KV or Durable Objects access, Cloudflare Access identity, static assets, or a Cloudflare Queues consumer.
- Choose `cloudrun` for a containerized Rust service on Google Cloud or its supported Google service integrations.
- Choose `fastly` for Fastly Compute.
- Choose `spin` for a Spin component.
- Choose `serviceworker` or `webworker` for browser-hosted experiments.

Also check:

1. whether the template has the required host-service adapter;
2. whether its Wasm target and build tools fit your environment;
3. whether the provider’s current limits, regions, and pricing fit the workload;
4. whether the template is covered by the repository’s integration tests.

The templates are independent adapters. Generating the same Ruby router for another template does not automatically make platform-specific service calls portable.
