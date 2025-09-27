<!-- Master checklist for onboarding new CLIs into the multi-CLI controller -->

# CLI Integration Playbook

This document captures the end-to-end checklist for introducing a new CLI into the multi-CLI controller. Follow these steps—in order—to reach feature parity with existing adapters (Claude, Codex, Cursor, Factory). Each section highlights required code paths, common pitfalls, and the nuances we encountered while integrating recent CLIs.

---

## 1. Planning & Discovery

- **Baseline research**
  - Scrape/collect the vendor documentation into `docs/<cli-name>/` for offline reference.
  - Identify authentication model (API key, OAuth, session token) and whether secrets can be reused from existing ExternalSecrets.
- **Capabilities profile**
  - Update `controller/src/cli/discovery.rs` with the new CLI’s configuration format, required env vars, and capability matrix (`CLICapabilities`, `CLIConfiguration`).
  - Add the CLI to `CLIType` (`controller/src/cli/types.rs`) and ensure display/serde helpers cover the new variant.
- **Image strategy**
  - Decide on base image requirements (runtime, additional tooling). If we publish our own image, add a Dockerfile under `infra/images/<cli>/`.

> **Gotcha:** Discovery defaults feed into health checks and logging. Missing entries here produce confusing controller warnings later—always add the CLI to `CLIType::from_str_ci` and `Display` implementations.

---

## 2. Adapter Implementation

- **Adapter skeleton**
  - Create `controller/src/cli/adapters/<cli>.rs` mirroring the Codex/Factory structure.
  - Use `BaseAdapter` helpers: `render_template_file`, `validate_base_config`, `base_health_check` to keep behaviour consistent.
  - Implement:
    - `validate_model` – even if permissive, reject empty strings.
    - `generate_config` – render CLI configuration via Handlebars templates (`infra/charts/controller/agent-templates/code/<cli>/`).
    - `format_prompt` – ensure newline termination to avoid shell quoting issues.
    - `parse_response` – consume CLI output (JSONL, structured logs). Return `ParsedResponse` with tool calls/metadata populated.
    - `health_check` – call `base_health_check`, render a small mock config, parse a canned response, and capture success/failure in `health.details`.
- **Adapter tests**
  - Place focussed tests at the bottom of the adapter file (see Factory/Codex). Cover config rendering, response parsing, and health status.

> **Gotcha:** Log streams often mix JSON and plain text. Use a tolerant parser (line-by-line `serde_json::from_str`) and accumulate plain segments so we never drop vital messages.

---

## 3. Template Plumbing

- **Handlebars templates**
  - Add CLI-specific templates under `infra/charts/controller/agent-templates/code/<cli>/`:
    - `container-base.sh.hbs` + per-agent wrappers.
    - CLI config (`<cli>-cli-config.json.hbs`) and project permissions (`<cli>-cli.json.hbs`).
    - Agent memory files (`agents*.md.hbs`).
- **Template selector**
  - Update `controller/src/tasks/template_paths.rs` with constants for new templates.
  - Extend `CodeTemplateGenerator` (`controller/src/tasks/code/templates.rs`) to:
    - Inject CLI-specific render settings (e.g., auto-run level, output format).
    - Register container partials and choose the right template per agent (Rex/Cleo/Tess/remediation).
    - Pass additional context (e.g., `auto_level`, `output_format`) so templates avoid hard-coded defaults.
- **Template smoke tests**
  - Add sample data to `controller/src/bin/test_templates.rs` for each new template to catch missing fields early.

> **Gotcha:** Handlebars drops raw JSON strings by default. Wrap arbitrary JSON additions in triple braces (`{{{raw_additional_json}}}`) to avoid HTML escaping.

---

## 4. Configuration Bridge

- **Bridge adapter**
  - Implement a CLI translator in `controller/src/cli/bridge.rs`:
    - Emit CLI config files (`TranslationResult.config_files`) matching what the runtime container expects (home vs workspace paths).
    - Gather toolman/MCP metadata from `UniversalConfig` (tool list, server URLs) and fold into the config JSON.
    - Generate default `droid` (or equivalent) command arguments with sane defaults (`--auto`, `--output-format`).
  - Update bridge tests to confirm file paths, env vars, and command lines.
- **Controller wiring**
  - Ensure the new CLI appears in `ConfigurationBridge::new` and the fallback tests pass.

> **Gotcha:** `UniversalConfig.tools` may include duplicates; dedupe before writing arrays to avoid spurious diffs in rendered configs.

---

## 5. Helm & Secrets

- **Values**
  - Add a CLI-specific image entry to `infra/charts/controller/values.yaml` under `agent.cliImages`.
  - Ensure `secrets.cliApiKeys.<cli>` exists with the desired secret key/env var mapping.
- **Templates**
  - If additional secrets are needed, update `infra/charts/controller/templates/secret.yaml` and `task-controller-config.yaml`.
- **Static ConfigMap**
  - Regenerate `infra/charts/controller/templates/agent-templates-static.yaml` using `./scripts/generate-templates-configmap.sh` once templates change.

> **Gotcha:** CI validates the static ConfigMap. Forgetting to regenerate it after template tweaks sends PRs red immediately.

---

## 6. Documentation & Records

- **README updates**
  - Mention the new CLI wherever we describe multi-CLI support or default models.
- **CLI-specific docs**
  - Expand `docs/<cli>/` with scraped references and any runbook notes (auth, flags, troubleshooting).
- **Integration playbook**
  - Add lessons learned to the CLI-specific playbook (e.g., `docs/factory-cli/factory-integration-playbook.md`) so future iterations improve.

> **Gotcha:** Keeping README language up to date prevents confusion for operators; ensure every CLI reference covers all supported variants.

---

## 7. Testing & Validation

- **Rust quality gates**
  - `cargo fmt/clippy/test` for `controller/` and `mcp/` crates.
- **Helm + GitOps**
  - `./scripts/generate-templates-configmap.sh`
  - `make -C infra/gitops validate`
- **Adapter-specific checks**
  - Run the template smoke binary (`cargo run --bin test_templates`) if you touch Handlebars.
  - (Optional) Execute the controller against a sandbox cluster to verify end-to-end job success.

> **Gotcha:** Clippy is unforgiving on bool comparisons (`assert!(...)` vs `assert_eq!(.., true)`). Follow existing patterns to keep CI green.

---

## 8. Pull Request Checklist

- Diff review: confirm only expected files changed (adapters, templates, Helm values, docs).
- Ensure docs and README mention the new CLI.
- Provide summary with test matrix in PR description (use `--fill` or manual template).
- Watch the `Validate Agent Templates` workflow for ConfigMap drift.

Following this playbook keeps new CLIs aligned with our existing abstractions and dramatically shortens integration time. Update this document with new gotchas whenever we onboard another CLI.
