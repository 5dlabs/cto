# Repository Guidelines

## Project Structure & Module Organization
- `mcp/` — Rust MCP server (`cto-mcp`).
- `controller/` — Rust controllers and binaries (e.g., `agent-controller`).
- `sidecar/` — Rust sidecar utilities.
- `infra/` — Helm charts, GitOps manifests, images, and scripts.
- `scripts/` — Bash helpers and validation utilities.
- `docs/` — Architecture, examples, and references.

## Build, Test, and Development Commands
- Build MCP server: `cd mcp && cargo build --release`.
- Build controller: `cd controller && cargo build --release --bin agent-controller`.
- Run tests (Rust): `cargo test -p cto-mcp` and `cargo test -p controller`.
- Lint/format (Rust): `cargo fmt --all --check` and `cargo clippy --all-targets -- -D warnings`.
- GitOps validation: `make -C infra/gitops validate` (or `lint`, `test`).
- Pre-commit checks: `pre-commit install && pre-commit run --all-files`.

## Coding Style & Naming Conventions
- Rust: rustfmt (Edition 2021, `max_width=100`); prefer `tracing::*` over `println!` (enforced by Clippy). Binary names kebab-case (e.g., `agent-controller`); files/modules snake_case (e.g., `src/bin/agent_controller.rs`).
- YAML: 2-space indent; begin docs with `---`; follow `.yamllint.yaml`.
- Markdown: follow `.markdownlint.yaml` (incremental headings, no trailing spaces).
- Shell: keep `bash -euo pipefail`; validate with ShellCheck where applicable.

## Testing Guidelines
- Unit tests live alongside code (`mod tests { ... }`) and in Rust integration tests when needed. Run `cargo test` in crate roots.
- For controllers and workflows, add small, deterministic tests; prefer fixtures under `controller/src/**/tests/`.
- Validate infrastructure changes with `make -C infra/gitops validate` and attach output to PRs when material.

## Commit & Pull Request Guidelines
- Use Conventional Commits: `feat:`, `fix:`, `chore:`, `refactor:`, etc. Keep commits focused and descriptive.
- Before pushing: `cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `pre-commit run --all-files`.
- PRs must include: summary, rationale, scope of impact, and verification steps; link issues (e.g., `Closes #123`). Include logs/screenshots for infra or CLI-facing changes.

## Security & Configuration Tips
- Never commit secrets. Use Kubernetes secrets/Helm values under `infra/secret-store/` and local env vars.
- Use `cto-config.json` (see `cto-config-example.json`) and `.cursor/mcp.json` to configure local runs; avoid committing user-specific tokens.
- Large files and generated artifacts should not be committed unless explicitly required.

