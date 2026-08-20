# Commands

The current CLI has one subcommand: `uzumibi new`.

## `uzumibi new`

~~~text
uzumibi new [OPTIONS] --template <TEMPLATE> <PROJECT_NAME>
~~~

### Arguments and options

| Argument or option | Description |
| --- | --- |
| `<PROJECT_NAME>` | Project name used in generated files and, by default, as the destination directory |
| `-t, --template <TEMPLATE>` | Required template name |
| `-d, --dest-dir <DEST_DIR>` | Write to a directory other than `PROJECT_NAME` |
| `--force` | Overwrite existing files without prompting |
| `--features <FEATURES>` | Comma-separated feature overlays |

Available templates are `cloudflare`, `cloudrun`, `fastly`, `spin`, `serviceworker`, and `webworker`.

Currently defined feature overlays are:

| Template | Feature | Purpose |
| --- | --- | --- |
| `cloudflare` | `enable-external` | Async Cloudflare host APIs from Ruby |
| `cloudflare` | `queue` | Cloudflare Queues consumer; includes external APIs |
| `cloudrun` | `enable-external` | Google Cloud external-service APIs |
| `cloudrun` | `queue` | Pub/Sub push consumer |

### Examples

~~~bash
uzumibi new --template cloudflare my-worker
uzumibi new -t cloudflare --features enable-external my-worker
uzumibi new -t cloudflare --features queue queue-consumer
uzumibi new -t cloudflare --dest-dir ./apps/worker my-worker
~~~

When files already exist and `--force` is not supplied, the CLI shows a diff and prompts for each conflicting file.

## Help and version

~~~bash
uzumibi --help
uzumibi new --help
uzumibi --version
~~~
