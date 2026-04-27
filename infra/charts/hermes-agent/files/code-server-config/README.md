# shared/code-server-config

Canonical code-server configuration shared between the persistent Helm
deployment (`infra/charts/openclaw-agent`) and the ephemeral controller-spawned
CodeRun jobs (`crates/controller/src/tasks/code/resources.rs`).

Both consumers **must** use these files so VS Code settings and workbench
state stay in lock-step.

## Files

- `settings.json` — VS Code user settings written to
  `$USER_DATA_DIR/User/settings.json`.
- `storage.json` — VS Code global workbench state written to
  `$USER_DATA_DIR/User/globalStorage/storage.json`.

## Consumers

- **Helm** — included via a directory-level symlink at
  `infra/charts/openclaw-agent/files/code-server-config` so Helm's
  `.Files.Get "files/code-server-config/<file>"` works despite the Helm
  restriction on paths that traverse `..`.
- **Rust controller** — embedded with
  `include_str!("../../../../../shared/code-server-config/<file>")` at
  compile time into `build_job_spec()`.

## Parity check

CI job `.github/workflows/parity-check.yaml` fails the build if either
consumer stops referencing this directory. See that workflow for the
exact asserts.

## Editing

Edit the files here. Both consumers pick up the change on next
build/render — no duplicated JSON to keep in sync.
