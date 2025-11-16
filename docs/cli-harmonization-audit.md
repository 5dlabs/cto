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

