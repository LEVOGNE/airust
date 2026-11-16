# Migrate from AIRust 0.1.8 to 0.1.9

Version 0.1.9 introduces the English-language and hardening changes described here. Set the dependency to `airust = "0.1.9"` to select this release. The published 0.1.8 package and tag remain the historical baseline. This release does not integrate an external DenkMaps application or MCP transport.

## Documentation and language

Documentation files now have English names:

| Previous path | Current path |
| --- | --- |
| `docs/ARCHITEKTUR.md` | `docs/ARCHITECTURE.md` |
| `docs/ANALYSE-0.1.7.md` | `docs/ANALYSIS-0.1.7.md` |
| `docs/AENDERUNGEN-0.1.8.md` | `docs/CHANGES-0.1.8.md` |

Update any application links to these repository documents. Source comments have been removed. Read [API.md](API.md) for behavioral contracts and [ARCHITECTURE.md](ARCHITECTURE.md) for design details.

English is the project and interface language. Language-neutral Unicode tokenization and matching remain part of the engine. Existing application content is data; switching project language does not imply translating a caller's stored knowledge.

## Runtime state

The CLI runtime directory is selected by `AIRUST_RUNTIME_DIR`. Its default is `$HOME/.local/state/airust`. It must be absolute and private to the current user; the Unix directory mode is `0700`. Existing symlink or insecure directories are rejected. `server.json` records PID, executable, and process-start identity; `stop` verifies that identity before signaling. `airust.log` is private.

PID and log state move out of globally shared `/tmp` paths. Existing background processes started by older versions must be handled with their original installation before starting a new process. Review strict command-line parsing when updating scripts: malformed options should produce a failure rather than being silently ignored.

## PDF output

`pdf2kb` rejects unknown flags and identical input and output paths. Replacing an existing output requires `--overwrite`. Configuration is validated before output directories are created. Use `--` to end option parsing when a filename begins with a dash. Unicode preview truncation keeps character boundaries intact.

## Knowledge persistence

The hardening pass validates example weights on load and save. Nonfinite and negative weights are invalid. A rejected save must leave the destination intact.

Saving a training collection uses an exclusively created private temporary file, flush and synchronization, followed by replacement. Symlink destinations are rejected rather than treated as an ambiguous replacement target. Callers should pass the intended regular file path and handle the returned error.

## Web authentication and origins

`AIRUST_API_TOKEN` is optional for a loopback-only listener and required for a non-loopback listener. Tokens must have 32 through 256 ASCII characters drawn from letters, digits, `-`, and `_`. A configured token protects the API and console WebSocket.

The browser prompts for the token and retains it in JavaScript memory. HTTP clients use `Authorization: Bearer ...`. WebSocket clients offer `airust` and `token.<token>` as subprotocols. The credential is not placed in a URL.

`AIRUST_ALLOWED_HOSTS` supplies explicit comma-separated hostnames. The default allows localhost, loopback addresses, and the selected bind IP. Do not include ports or wildcard entries in this setting. Requests must use the correct service port. Browser origins must match an allowed host and the main or console port. Cross-site requests are rejected. Command-line clients without an Origin header remain supported.

Reverse-proxy deployments must account for these host and port checks. A proxy configuration is not automatically compatible merely because it forwards a TCP connection.

## Explicit administration

Administrative filesystem endpoints, knowledge load/save operations, and console reset are disabled by default. `AIRUST_ENABLE_ADMIN=1` enables them. Without it, attempts return HTTP 403 or the equivalent console rejection.

Enabled filesystem operations stay within the server's working directory. Absolute paths, parent traversal, hidden components, and symlink components are rejected. Use explicit relative paths for the intended administrative files.

These controls do not create a multi-user authorization model or an operating-system sandbox. See [SECURITY.md](SECURITY.md) before deploying the optional administration surface.

## Settings commands and sample data

Ordinary chat queries no longer change interface settings by keyword detection. Settings commands require the explicit `/settings` prefix. The `/api/settings` endpoint remains available.

The embedded sample dataset contains 54 English questions. Translated duplicates were replaced as part of the English project surface. Existing local database content and custom rows are retained.

## Resource limits and crawling

HTTP request bodies are limited to 8 MiB. Query input must contain non-whitespace text and use at most 4,096 UTF-8 bytes. Training input and serialized output are limited to 1 MiB each, with nonempty input. Imports allow at most 1,000 items, validate the entire batch, and write through a SQL transaction.

Filesystem reads are limited to 1 MiB. Console WebSocket frames and messages are limited to 4 KiB; trimmed console commands are limited to 1 KiB. SQL query pagination allows 1 through 1,000 rows and a nonnegative offset.

The crawler accepts public HTTP and HTTPS destinations without embedded credentials. It validates resolved addresses, pins the selected DNS result, ignores proxies, and disables redirects. Response bodies are limited to 2 MiB. Traversal is limited to depth five and 100 pages.

Localhost, private-network, and redirect-based crawling workflows must be reconsidered when upgrading. These restrictions apply to the optional crawler; the in-memory knowledge engine still operates without networking.

## Future integrations

Test the 0.1.9 engine before implementing any external application adapter. Generic semantic providers, typed relation evidence, persistent indexes, and structural context are future work listed in [ROADMAP.md](../ROADMAP.md).
