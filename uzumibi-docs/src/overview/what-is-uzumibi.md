# What is Uzumibi?

Uzumibi is a small Ruby HTTP framework plus a set of adapters and project templates for edge and serverless platforms.

The core framework provides:

- routing for `GET`, `POST`, `PUT`, `DELETE`, `HEAD`, and `OPTIONS`
- named path parameters and wildcard routes
- request objects containing method, path, headers, parameters, cookies, and body data
- response objects containing a status code, headers, and a String body
- a compact binary protocol used by platform adapters to exchange HTTP data with Wasm

Platform-specific functionality is provided by the generated template and adapter crates. It is not guaranteed to be portable between templates. For example, the Cloudflare template can optionally expose Workers KV, Durable Objects, outbound fetch, secrets, Access identity, and Queues.

The name “Uzumibi” (うずみび) refers to live embers kept under ash so that the fire does not go out.
