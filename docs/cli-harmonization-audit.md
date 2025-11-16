# CLI Harmonization Audit (2025-11-16)

## Executive Summary
- The controller already fans every agent through the same adapter pipeline, so business rules (branch discipline, quality gates, tool lists) remain identical regardless of CLI choice.
- Six CLIs (`claude`, `codex`, `factory`, `opencode`, `cursor`, `gemini`) now ship template families and production-ready adapters. Deprecated CLIs (Grok/Qwen/OpenHands) were removed from the enum to match the supported surface area.
- Model validation inside the MCP server is CLI-agnostic, so the reference config you shared will parse without manual tweaks for Gemini-specific settings.
- To keep CLIs interchangeable per agent, we now need broader smoke tests so every CLI/container script pair runs at least once before a workflow is dispatched.

## Reference Config Touchpoints
The repo’s baseline `cto-config.json` already mixes different CLIs across workflows (`codex` for `defaults.code`, `factory` for `defaults.play`, `factory` for Rex, `codex` for Cipher, `claude` for the rest), so your reference config is consistent with today’s assumptions.

```1:59:cto-config.json
{
  "version": "1.0",
  "defaults": {
    "docs": {
      "model": "claude-opus-4-1-20250805",
      "githubApp": "5DLabs-Morgan",
      "includeCodebase": false,
      "sourceBranch": "main"
    },
    "code": {
      "model": "gpt-4o",
      "githubApp": "5DLabs-Rex",
      "continueSession": false,
      "workingDirectory": ".",
      "overwriteMemory": false,
      "repository": "5dlabs/cto",
      "docsRepository": "5dlabs/cto",
      "docsProjectDirectory": "docs",
      "service": "cto",
      "cli": "codex",
      "maxRetries": 10
    },
    ...
    "play": {
      "model": "claude-sonnet-4-5-20250929",
      "cli": "factory",
      "implementationAgent": "5DLabs-Rex",
      "frontendAgent": "5DLabs-Blaze",
      "qualityAgent": "5DLabs-Cleo",
      "securityAgent": "5DLabs-Cipher",
      "testingAgent": "5DLabs-Tess",
      ...
    }
  },
  "agents": {
    "rex": {
      "githubApp": "5DLabs-Rex",
      "cli": "factory",
      "model": "claude-sonnet-4-5-20250929",
      ...
    },
    "cipher": {
      "githubApp": "5DLabs-Cipher",
      "cli": "codex",
      "model": "gpt-4o",
      ...
    }
  }
}
```

## Architecture Parity Check

### Adapter coverage
The adapter factory now instantiates six adapters at startup (`claude`, `codex`, `cursor`, `factory`, `opencode`, `gemini`), so every CLI that exists in `CLIType` is wired into the shared lifecycle (metrics, health checks, template rendering).

```177:201:controller/src/cli/adapter_factory.rs
async fn register_default_adapters(&self) -> AdapterResult<()> {
    let claude_adapter = Arc::new(ClaudeAdapter::new().await?);
    self.register_adapter(CLIType::Claude, claude_adapter).await?;

    let codex_adapter = Arc::new(CodexAdapter::new().await?);
    self.register_adapter(CLIType::Codex, codex_adapter).await?;

    let cursor_adapter = Arc::new(CursorAdapter::new().await?);
    self.register_adapter(CLIType::Cursor, cursor_adapter).await?;

    let factory_adapter = Arc::new(FactoryAdapter::new().await?);
    self.register_adapter(CLIType::Factory, factory_adapter).await?;

    let opencode_adapter = Arc::new(OpenCodeAdapter::new().await?);
    self.register_adapter(CLIType::OpenCode, opencode_adapter)
        .await?;

    let gemini_adapter = Arc::new(GeminiAdapter::new().await?);
    self.register_adapter(CLIType::Gemini, gemini_adapter).await?;

    Ok(())
}
```

### Template families stay in sync
Every CLI gets its own container script, config, and memory templates, and the Helm script emits separate ConfigMaps so they ship in lockstep.

```3:37:controller/src/tasks/template_paths.rs
pub const CODE_CLAUDE_CONTAINER_TEMPLATE: &str = "code/claude/container.sh.hbs";
pub const CODE_CLAUDE_MEMORY_TEMPLATE: &str = "code/claude/memory.md.hbs";
...
pub const CODE_CODEX_CONFIG_TEMPLATE: &str = "code/codex/config.toml.hbs";
...
pub const CODE_CURSOR_CONTAINER_TEMPLATE: &str = "code/cursor/container.sh.hbs";
...
pub const CODE_FACTORY_GLOBAL_CONFIG_TEMPLATE: &str = "code/factory/factory-cli-config.json.hbs";
...
pub const CODE_OPENCODE_CONFIG_TEMPLATE: &str = "code/opencode/config.json.hbs";
```

```110:119:infra/charts/controller/scripts/generate-agent-templates-configmap.sh
create_configmap "shared" "agents/"
create_configmap "claude" "code/claude/"
create_configmap "cursor" "code/cursor/"
create_configmap "codex" "code/codex/"
create_configmap "factory" "code/factory/"
create_configmap "gemini" "code/gemini/"
create_configmap "opencode" "code/opencode/"
create_configmap "code-shared" "code/[^/]+\\.(hbs|md|sh)$"
create_configmap "docs" "docs/"
```

### Shared prompt scaffolds
Gemini and OpenCode used to carry identical prompt preambles inline; those blocks now live in a single shared partial so we only edit the guidance once per policy change.

```1:68:infra/charts/controller/agent-templates/code/shared/prompt-scaffold.sh.hbs
if [ ! -f "{{work_dir}}/{{guidance_filename}}" ]; then
  echo "⚠️ No {{guidance_filename}} guidance file detected; creating placeholder"
  ...
PROMPT_PREFIX="${PROMPT_PREFIX}- {{memory_reference}}\n"
...
if attempt_task_recovery "{{#if docs_source}}{{docs_source}}{{else}}/tmp/docs-repo{{/if}}" "{{work_dir}}/task" "{{task_id}}"; then
...
```

```2179:2213:controller/src/tasks/code/templates.rs
obj.insert(
    "prompt_scaffold".to_string(),
    json!({
        "work_dir": "$GEMINI_WORK_DIR",
        "guidance_filename": "GEMINI.md",
        "memory_reference": "@GEMINI.md — Repository guidelines and workflow",
        "docs_source": "/tmp/docs-repo",
        "cli_display_name": "Gemini"
    }),
);
```

That partial now renders inside both container scripts:

```1418:1424:infra/charts/controller/agent-templates/code/gemini/container-base.sh.hbs
# =========================================================================
# Prompt assembly and Gemini execution
# =========================================================================

{{> code_shared_prompt-scaffold prompt_scaffold}}
```

### Capability registry feeds templates
A new `cli_capabilities(CLIType)` registry keeps the streaming/function-call flags and context windows for every CLI in one place. Each adapter now defers to that helper instead of duplicating values, and the template generator injects the serialized capability map into the `cli` context so AGENTS.md/container scripts can branch on what the selected CLI actually supports.【1:33:controller/src/cli/capabilities.rs】【360:402:controller/src/tasks/code/templates.rs】

### Automated CLI/agent validation matrix
`cargo test -p controller` now runs `test_cli_agent_validation_matrix`, which iterates every CLI type against every GitHub app (Rex/Cleo/Tess/Blaze/Cipher/Atlas/Bolt) and asserts their container + memory templates exist under `infra/charts/controller/agent-templates`. Missing ConfigMap entries now fail in seconds during unit tests instead of during multi-hour workflows.【3170:3179:controller/src/tasks/code/templates.rs】

### Shared business logic per agent
All adapters inherit the same validation, telemetry, and template rendering guarantees through `BaseAdapter`, so instructions such as “always create a PR” or “run clippy pedantic” live in the rendered agent markdown rather than the CLI implementation.

```196:244:controller/src/cli/base_adapter.rs
pub fn validate_base_config(&self, config: &AgentConfig) -> AdapterResult<()> {
    if config.github_app.trim().is_empty() {
        return Err(AdapterError::ValidationError(
            "GitHub app cannot be empty".to_string(),
        ));
    }
    ...
    if config.cli != expected_cli {
        return Err(AdapterError::ValidationError(format!(
            "CLI type mismatch: expected '{}', got '{}'",
            expected_cli, config.cli
        )));
    }
    ...
    Ok(())
}
```

```1:72:infra/charts/controller/agent-templates/code/factory/agents-rex.md.hbs
# Factory Project Memory — Implementation Agent (Rex)
...
- **Stay on the feature branch.** The controller has already checked out `feature/task-{{task_id}}-implementation`...
- **Operate without supervision.** Do not pause to ask for permission...
- **Create the PR** ... add labels (`task-{{task_id}}`, `service-{{service}}`, `run-{{workflow_name}}`) and include "Closes #ISSUE_NUMBER".
```

### Model validation no longer blocks non-Claude CLIs
The MCP server now accepts any non-empty model string, so Codex/Factory/OpenAI/Gemini identifiers pass validation and defer to the target CLI.

```172:180:mcp/src/main.rs
fn validate_model_name(model: &str) -> Result<()> {
    if model.trim().is_empty() {
        return Err(anyhow!("Model name cannot be empty"));
    }
    // Allow any non-empty model name - let the CLI handle model-specific validation
    Ok(())
}
```

## Local CLI documentation snapshots
To keep pace with rapidly changing CLI switches, we vend the upstream docs inside this repo. Each directory contains the exact files pulled on 2025-11-16 so reviewers can diff future updates:

- `docs/claude-code/` – [Anthropic Claude Code docs](https://code.claude.com/docs/)
- `docs/cursor-cli/` – [Cursor headless CLI](https://cursor.com/docs/cli/headless) (HTML snapshot)
- `docs/codex-cli/` – [OpenAI Codex docs](https://github.com/openai/codex/tree/main/docs)
- `docs/factory-cli/` – [Factory CLI docs](https://github.com/Factory-AI/factory/tree/main/docs)
- `docs/opencode-cli/` – [OpenCode CLI docs](https://opencode.ai/docs)
- `docs/gemini-cli/` – [Gemini CLI docs](https://github.com/google-gemini/gemini-cli/tree/main/docs)

These folders are intentionally vendor-specific so we can link to concrete examples while keeping the controller templates DRY.

## CLI + agent validation plan
Instead of waiting for end-to-end workflow flakes, we can exercise each CLI/agent pair deterministically:

| CLI | Agents exercised | Automated checks | Manual spot-check |
| --- | --- | --- | --- |
| Claude | Rex, Cleo, Tess, Cipher | `cargo test -p controller` already covers Claude adapter parsing + template selection; add nightly `cursor-agent`-style smoke script that renders Rex container and runs `claude --version` | Run `docs/claude-code` quickstart in the seeded PVC |
| Codex | Rex, Cleo, Tess, Cipher | Existing adapter tests + `test_templates` binary; add `scripts/validate-cli codex` to invoke `codex --version --config generated` | Manual: `docker run codex-agent` with generated config |
| Cursor | Rex, Cleo, Tess, Blaze | Unit tests validate stream-json parsing; add `make validate-cursor` that executes `cursor-agent --print --output-format stream-json --force "noop"` inside the container image | Validate headless auth by replaying `docs/cursor-cli/headless.html` instructions |
| Factory | Rex, Blaze | Keep current `factory --version` smoke step and assert `factory-cli-config.json` from ConfigMap matches schema extracted from `docs/factory-cli/README.md` | Run `factory droid plan` locally with generated config |
| OpenCode | Rex | New prompt scaffold ensures deterministic prompt preview; add `scripts/validate-cli opencode` to run `opencode --version` and parse `--dry-run` output | Launch `opencode run --dry-run` using `docs/opencode-cli` reference |
| Gemini | Rex | Adapter tests + prompt scaffold; add `scripts/validate-cli gemini` to call `gemini --version` and `gemini run prompt.md --dry-run` with generated settings | Follow `docs/gemini-cli/README.md` quickstart inside container |

Per agent gating:

- **Rex** – requires PR automation + task file presence. Validation uses `code/shared/prompt-scaffold` plus `scripts/validate-cli <cli> --agent rex`.
- **Cleo/Tess** – there is no separate CLI; they inherit Rex containers but different AGENTS.md. Validation ensures `agents/<name>-system-prompt` renders and `task/acceptance-criteria.md` is referenced.
- **Cipher** – same as Cleo/Tess but ensures `github-guidelines.md` includes security hooks; tests already read that file.

Once `scripts/validate-cli` exists (tracked in TODO-5/6), we can hook it into CI to give us a deterministic “ready” bit before dispatching expensive workflows.

## CLI Readiness Matrix

| CLI       | Adapter status | Template set | Notes |
|-----------|----------------|--------------|-------|
| Claude    | ✅ Mature       | ✅ (`code/claude/*`) | Production baseline; business prompts live in agent markdown. |
| Codex     | ✅ Mature       | ✅ (`code/codex/*`)  | TOML config + AGENTS.md generated; ready for all agents. |
| Factory   | ✅ Mature       | ✅ (`code/factory/*`)| JSON config emitted; used for play workflows. |
| OpenCode  | ✅ Mature       | ✅ (`code/opencode/*`)| Supports provider overrides + remote tools. |
| Cursor    | ✅ Mature       | ✅ (`code/cursor/*`) | Adapter renders config/memory templates, parses stream-json output, and carries dedicated unit tests. |
| Gemini    | ✅ Mature       | ✅ (`code/gemini/*`) | Adapter + templates land in this PR; emits `GEMINI.md`, user/workspace `settings.json`, and stream-json parsing. |

## Gaps & Risks to Address Pre-Test

### Gemini adapter + templates implemented
Gemini now follows the exact same contract as the other CLIs: the adapter renders user/workspace `settings.json`, emits `GEMINI.md`, and parses `--output-format stream-json` events. The implementation mirrors the upstream CLI contract so we can drop it into production as soon as Gemini 3 ships.  Supporting references:

- Adapter + unit tests: `controller/src/cli/adapters/gemini.rs`
- Bridge generation + CLI command wiring: `controller/src/cli/bridge.rs`
- Template set (`container.sh`, `GEMINI.md`, `settings.*.json`): `controller/src/tasks/code/templates.rs` and `infra/charts/controller/agent-templates/code/gemini/*`

### Config split includes Gemini
`scripts/generate-agent-templates-configmap.sh` now emits a dedicated `agent-templates-gemini.yaml`, so Helm gets the Gemini prompts without bloating the shared ConfigMaps.

### Remaining validation
1. Re-run `./scripts/generate-agent-templates-configmap.sh` and commit the regenerated YAML so ArgoCD ships the new Gemini templates alongside the existing ones.
2. Execute the usual adapter/unit suites (`cargo test -p controller`) plus at least one dry-run workflow per CLI (e.g., `./scripts/test-play-project.sh --agent rex --cli gemini`) to prove cross-CLI parity.
3. Keep the vendored upstream docs (`docs/gemini-cli`) synced with [google-gemini/gemini-cli](https://github.com/google-gemini/gemini-cli) so we can track the Gemini 3 schema and CLI surface area.

## Preemptive Validation Plan
1. **Unit smoke for each adapter**: `cargo test -p controller cli::adapters::codex` (repeat for factory/opencode/claude) to catch template regressions.
2. **Template checksum regeneration**: `./scripts/generate-templates-configmap.sh` followed by `git diff --stat` whenever you modify any CLI-specific prompts to guarantee all variants stay in sync.
3. **End-to-end dry runs** (once Cursor/others are implemented):
   - Implementation stage: `./scripts/test-play-project.sh --agent rex --cli codex`.
   - Quality stage: `./scripts/test-play-project.sh --agent cleo --cli factory` (verifies cross-CLI PVC + resume logic).
4. **Config parsing guard**: `cargo test -p mcp model_validation_tests` after updating `cto-config.json` to ensure new model strings stay accepted.
5. **Static lint**: `cargo clippy --all-targets -- -D warnings -W clippy::pedantic` plus `cargo test -p controller` before switching CLI assignments, so regressions are caught prior to a multi-agent play run.

The remaining work is purely procedural (regenerate ConfigMaps + run the multi-CLI smoke tests). The adapter stack, templates, PVC orchestration, and MCP validation are already harmonized across all six supported CLIs.

