<!-- Implementation playbook for Factory droid CLI integration -->

# Factory CLI Integration Playbook

This document captures the working agreement for bringing Factory’s `droid` CLI into the 5D Labs multi-CLI controller. It distills what we learned from the Codex and Cursor enablement plus nuances from the Factory documentation we scraped into `docs/factory-cli/**`. Treat it as the source-of-truth checklist while implementing the adapter, templates, images, and Helm wiring.

## 1. Product & Capability Snapshot

- **CLI entrypoint**: `factory-droid` (placeholder; confirm actual binary name once we pull the tool). Supports *Auto-Run* mode for unattended execution (`docs/factory-cli/user-guides/auto-run.md`).
- **Configuration surfaces**:
  - Project-level `.factory/cli.json` (maps to Factory’s permissions + droid customisation). Analogue to Cursor’s `.cursor/cli.json`.
  - Global config under `$HOME/.factory/cli-config.json`; must be provisioned from template (see “Configuration → settings/byok/agents-md/mcp”).
  - `AGENTS.md` is first-class for prompt specialisation (`docs/factory-cli/configuration/agents-md.md`).
- **Model routing**: “Choosing your model” doc spells out per-task model selection and BYOK support. The CLI accepts direct model IDs and can route to enterprise-hosted LLMs.
- **Automation**: `droid exec` supports headless pipelines, including GitHub Actions cookbook (`docs/factory-cli/droid-exec/cookbook/github-actions.md`). Auto Run mode disables interactive confirmations.
- **Tooling**: MCP integration page shows how to register external servers, so ToolMan remains viable.

## 2. Core Parity Goals (Codex & Cursor lessons applied)

| Area | Codex/Cursor Baseline | Factory Target | Action |
| --- | --- | --- | --- |
| Templates | Handlebars under `agent-templates/code/<cli>/` with base + per-agent wrappers, config, MCP JSON | Same structure: create `factory` subtree mirroring Cursor (containers, agents docs, CLI configs) | Duplicate layout, reusing partials + shared logic (no hard-coded tool lists). |
| Controller | `cursor.rs` adapter builds template context, passes model/approval/sandbox, writes CLI configs, mounts MCP | New `factory.rs` adapter replicates pattern and plumbs Factory-specific flags (auto-run, spec mode toggles as needed) | Use new `CliType::Factory` and extend enums in template selection + tests. |
| Config delivery | ConfigMap built from `generate-agent-templates-configmap.sh` | Add Factory templates to same generator (just new files) | Already handled once templates exist; ensure file names conform to sort order. |
| CLI invocation | Cursor container wraps binary, ensures Node shim, passes `--model`, `--print --force --output-format stream-json` | Factory script must pass the equivalents: identify `--auto-run`, `--non-interactive`, `--model <id>`, `--output json/stream` flags from docs | ✅ Container now wires `droid exec --auto <level> --model <id> --output-format <format>` based on Helm + CLI config, with retries + completion probe. |
| Completion probes | Cursor reruns CLI with summarised question | Implement same logic; confirm Factory supports stream output for quick yes/no (may use `--output text`). | ✅ Completion probe implemented via follow-up `droid exec --auto low --output-format text`. |
| Secret injection | ExternalSecret supplies API key, exported as env var (Cursor: `CURSOR_API_KEY`) | Determine Factory’s token env (`FACTORY_API_KEY` or similar). Update `infra/charts/controller/templates/secret.yaml` + values to mount. | ✅ Helm values expose `cliApiKeys.factory.secretKey = FACTORY_API_KEY`; controller maps it automatically. |
| Image build | Cursor image had Node path mismatch fixed via symlink + wrapper | Build `ghcr.io/5dlabs/factory:latest` based on upstream requirements. Expose binary at `/usr/local/bin`, include Node/Python if CLI scripts depend on them (docs mention spec mode requiring Node?). Validate via test pod before shipping. |
| GitHub behavior | Auto PR logic, labels creation, workspace cleanup | Reuse same auto-PR script. Ensure instructions emphasise “never push to main”. |
| RBAC/apply | ConfigMap updates handled via server-side apply + RBAC | No change. Just confirm runners can access config in `agent-platform`. |

## 3. Required Templates & Assets

Create the following under `infra/charts/controller/agent-templates/`:

- `code/factory/`
  - `container-base.sh.hbs` – shell wrapper invoking `factory-droid`. Must:
    - Bootstrap runtime (Rust toolchain, Node if required).
    - Install global config (`$HOME/.factory/cli-config.json`) from `/task-files/factory-cli-config.json` if present.
    - Install project config `.factory/cli.json`.
    - Copy MCP config to `/workspace/.factory/mcp.json` or equivalent (confirm path).
    - Run droid with forced model + auto-run flags, log to `/tmp/factory-run-*.jsonl` (for debugging).
    - Implement retry loop + completion probe.
    - Honour `outputFormat` and `autoLevel` from CLI config.
  - `container.sh.hbs`, `container-rex.sh.hbs`, `container-cleo.sh.hbs`, `container-tess.sh.hbs`, `container-rex-remediation.sh.hbs` – thin wrappers referencing the base partial with agent-specific banners/prompts.
  - `agents.md.hbs` + per-agent variants if Factory requires separate memos (check docs; may leverage spec mode instructions).
  - `factory-cli-config.json.hbs` – global CLI config template exposing:
    - `version`, `hasChangedDefaultModel`, `model.default`, optional `temperature`, `maxOutputTokens`.
    - `autoRun`/`approval` toggles (needs mapping from docs – likely `"autoRun": true`, `"confirmations": false`).
    - Permissions (Factory uses allow/deny tokens similar to Cursor? Validate from docs; default to allow all until we see restrictions).
    - MCP server entry.
  - `factory-cli.json.hbs` – project-level permissions.
  - `mcp.json.hbs` – ensure path/format matches Factory expectation.
  - `client-config.json.hbs`, `coding-guidelines.md.hbs`, etc. – reuse cross-CLI templates where identical.
- `docs/factory/` – if Factory expects documentation assets akin to Cursor’s docs agent, mirror necessary files (memory, prompt, settings).

## 4. Controller & Code Changes

1. **Introduce `CliType::Factory`**
   - `controller/src/cli/adapters/mod.rs`, `factory.rs` (new file), update `factory::FactoryAdapter` with `build_template_context` mirroring `CursorAdapter`.
   - Extend `controller/src/tasks/template_paths.rs` to discover Factory template root.
   - Update task template tests (`controller/src/bin/test_templates.rs`) to run Factory rendering and shell syntax validation.
2. **Template context keys**
   - Add Helm-configured fields (`factory.model`, `factory.reasoning_effort`, etc.).
   - Ensure `model` is passed to container partial (for `--model` flag) and to CLI config JSON.
   - If Factory CLI needs workspace service name, propagate `service`, `working_directory` as we do for Cursor.
3. **MCP + ToolMan**
   - Confirm Factory expects `.factory/mcp.json` or `.factory/mcp/*.json`. Adjust path and mount in adapter accordingly.
4. **Secret wiring**
   - Update `controller/src/tasks/config.rs` if we need additional config values (e.g., Factory-specific toggles).

## 5. Helm & GitOps Updates

1. **Values schema**: extend `infra/charts/controller/values.schema.json` & `values.yaml` with `agent.cliImages.factory`, `agent.cliConfigs.factory` (model defaults, flags, doc limits).
2. **Secret template**: update `infra/charts/controller/templates/secret.yaml` to mount `FACTORY_API_KEY` (assuming env var; confirm from docs). Use `external-secrets` to source value via `cto-config`.
3. **Task controller config**: ensure `task-controller-config.yaml` passes CLI options for Factory (images, config map references).
4. **RBAC/ConfigMaps**: no new RBAC expected, but verify `agent-templates-static.yaml` size stays under 1MB after adding files.
5. **Image references**: add Factory image to `agents-build` workflow matrix so GitHub builds/pushes `ghcr.io/5dlabs/factory:<tag>` along with other CLIs.

## 6. Factory CLI Image Build Plan

1. **Base image**: start from `node:20-bullseye` (docs mention specification mode using Node-based tooling) or the official Factory image if provided.
2. **Install**: scripted install for Factory CLI (likely via `curl | bash` similar to Cursor). Capture exact version tag for reproducibility.
3. **Runtime extras**: include Rust toolchain, git, Python, jq—matching other CLI containers so workflow logic works.
4. **Binary location**: ensure final executable lives at `/usr/local/bin/factory-droid`. Provide wrapper if CLI expects Node modules in specific path.
5. **Permissions**: align UID/GID with other agent images to avoid PVC permission issues.
6. **Validation**: create a `factory-cli-smoke-test.sh` (analogous to Cursor’s) to run inside CI verifying `factory-droid --version`, `factory-droid help`, auto-run dry run, and MCP connectivity stub.

## 7. Testing & Verification Checklist

- `cargo fmt`, `cargo clippy`, `cargo test` on `controller/` and `mcp/` crates.
- `controller/src/bin/test_templates.rs` should render Factory templates and run `bash -n` on generated scripts.
- `scripts/test-all-agents-json.sh` – extend to include Factory output sample if applicable.
- Helm lint/`make -C infra/gitops validate` after template/config additions.
- Spin up a dry-run workflow (like rust-basic-api) with `CliType::Factory` to ensure:
  - global & project configs land in correct paths;
  - CLI respects `--model` flag and auto-run (no prompts);
  - MCP handshake with ToolMan succeeds;
  - auto PR stage works, no labels missing.

## 8. Edge Cases & Open Questions

1. **CLI flag parity**: need to confirm exact flags for auto-run, output modes, model selection. Update this doc once verified against the binary.
2. **Permission schema**: docs show custom commands and agents; confirm whether project config mandates `permissions.allow/deny` or different schema. Adjust templates accordingly.
3. **Spec mode**: Factory emphasises “Specification Mode”; determine if we should default Rex to spec mode vs implementation mode. Possibly controlled via template flag or additional CLI config entry.
4. **Observability**: confirm CLI emits structured logs similar to Cursor. If not, adapt completion parsing logic.
5. **Licensing throttles**: ensure our automation respects concurrency/policy limits (not documented publicly; coordinate with Factory support if needed).

## 9. Next Steps

1. Verify CLI download/install process and enumerate required environment variables (API key name, workspace IDs).
2. Draft initial templates + adapter scaffolding in a feature branch (`feature/factory-cli-support`).
3. Build and test Factory runner image locally using the plan above, then push to GHCR.
4. Wire Helm values, regenerate ConfigMap, and run integration tests on staging repo (e.g., `5dlabs/rust-basic-api`).
5. Iterate on doc updates with any newly discovered nuances.

---

**Document changelog**
- 2025-09-27 – Initial draft based on Codex/Cursor retrospectives and Factory CLI docs scrape.
