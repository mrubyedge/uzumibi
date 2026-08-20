# CLI Troubleshooting

## `uzumibi: command not found`

Confirm that Cargo’s binary directory is on `PATH`:

~~~bash
cargo install uzumibi-cli
uzumibi --version
~~~

## Template not found

Template names are lowercase:

~~~text
cloudflare, cloudrun, fastly, spin, serviceworker, webworker
~~~

The CLI prints the available template names when a requested template does not exist.

## Existing files

The CLI can generate into an existing directory. Without `--force`, it displays a diff and asks whether to overwrite, skip, or abort for each conflict.

Use `--dest-dir` to choose a separate destination, or `--force` only when replacing existing files is intentional.

## A feature appears to have no effect

Feature names select template overlay directories. Use only features documented for the selected template; an unknown name does not create an overlay.
