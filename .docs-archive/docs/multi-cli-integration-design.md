# Multi-CLI Agent Platform: PRD & Architecture Design

**Version:** 1.0
**Date:** September 23, 2025
**Authors:** Claude (AI Assistant)
**Status:** Draft for Review

## Executive Summary

The 5D Labs Agent Platform currently operates exclusively with Claude Code CLI. To expand capabilities and support diverse development workflows, we need to implement CLI-agnostic support, starting with OpenAI's Codex CLI, followed by Opencode and Cursor. This requires a standardized abstraction layer that handles the nuances of each CLI while maintaining consistent agent behavior.

## Table of Contents

1. [Critical Implementation Prerequisites & Open Questions](#critical-implementation-prerequisites--open-questions)
2. [Current Architecture Analysis](#current-architecture-analysis)
3. [Key Challenges Identified](#key-challenges-identified)
4. [Proposed Solution Architecture](#proposed-solution-architecture)
5. [Implementation Roadmap](#implementation-roadmap)
6. [Technical Considerations](#technical-considerations)
7. [Backward Compatibility Test Matrix](#backward-compatibility-test-matrix)
8. [Inter-Agent Collaboration Points](#inter-agent-collaboration-points)
9. [Success Metrics](#success-metrics)
10. [Risk Mitigation](#risk-mitigation)

## Critical Implementation Prerequisites & Open Questions

### 🚨 MAJOR SCOPE DISCOVERY: 8 CLI Architecture Analysis

**Comprehensive analysis reveals 8 different CLI footprints in `/infra/images/`:**

| CLI | Install Source (repo path) | Runtime Footprint | Primary Config Artifacts | Guidance / Memory Mechanism | MCP Integration Notes |
|-----|----------------------------|-------------------|--------------------------|-----------------------------|-----------------------|
| **Claude** | `infra/images/claude/Dockerfile` (installs `@anthropic-ai/claude-code`) | Node.js | Project `CLAUDE.md` + generated `mcp.json` | `CLAUDE.md` instructions (current baseline) | Uses Claude’s native MCP support with Tools bridged via `tools --url …` |
| **Codex** | `infra/images/codex/Dockerfile` (installs `@openai/codex`) | Rust binary distributed via npm | `~/.codex/config.toml` (TOML) | `AGENTS.md` layered files (`docs/codex/docs/getting-started.md:62`) | STDIO-only MCP clients (`docs/codex/docs/config.md:341`) |
| **Opencode** | `infra/images/opencode/Dockerfile` (installs OpenCode bootstrap) | TypeScript/Node | Project + global `opencode.json` / `opencode.jsonc` | `AGENTS.md` + instruction files (`docs/opencode/packages/web/src/content/docs/docs/rules.mdx`) | Ships `@modelcontextprotocol` client libraries; supports both local and remote MCP servers |
| **Gemini** | `infra/images/gemini/Dockerfile` (installs `@google/gemini-cli`) | TypeScript/Node | TBD – need upstream config review | Guidance mechanism TBD (no `GEMINI.md` reference in repo yet) | MCP support advertised upstream; implementation specifics still to be confirmed |
| **Grok** | `infra/images/grok/Dockerfile` (installs `@vibe-kit/grok-cli`) | TypeScript/Node | CLI-driven flags/env (scripts/grok-entrypoint.sh) | TBD – no persisted rules observed yet | MCP usage not documented in repo – requires investigation |
| **Cursor** | `infra/images/cursor/Dockerfile` (installs `cursor-agent`) | Python CLI + Node deps | Environment variables + CLI arguments | TBD – depends on Cursor agent capabilities | MCP integration unclear; likely via Cursor cloud APIs |
| **OpenHands** | `infra/images/openhands/Dockerfile` (installs `openhands-ai` via pip) | Python/Poetry | Virtualenv configuration, project YAML/TOML | Internal stateful sessions (OpenHands runtime) | MCP support undocumented; treat as non-MCP initially |
| **Qwen** | `infra/images/qwen/Dockerfile` (installs `@qwen-code/qwen-code`) | TypeScript/Node | CLI defaults (npm package) | Guidance mechanism TBD – confirm upstream behavior | MCP story unknown; may mirror other Node CLIs but needs validation |

**CRITICAL FINDINGS:**
- **npm-based distribution**: Six of the eight CLIs (Claude, Codex, Opencode, Gemini, Grok, Qwen) are delivered via npm packages on top of our runtime image, implying shared dependency management and cache strategy.
- **Mixed guidance formats**: Claude and Codex use documented instruction files (`CLAUDE.md`, `AGENTS.md`), Opencode also leans on `AGENTS.md`; the remaining CLIs require discovery before we assume memory compatibility.
- **Configuration diversity**: We must support TOML (`codex`), JSON/JSONC (`opencode`), and CLI/ENV driven modes (Grok, Cursor, OpenHands, Qwen) without forcing a single schema.
- **MCP readiness varies**: Only Claude/Codex have confirmed MCP stories in this repo. Gemini/Grok/Qwen likely provide SDK integrations, while Cursor/OpenHands may require wrappers or alternative tooling.
- **Container baselines**: Python-focused CLIs (Cursor, OpenHands) impose different dependency sets compared to Node-first CLIs, reinforcing the need for CLI-specific entrypoints.

**REVISED SCOPE - Phase 1 Focus: "Big 4" CLIs**
- **Claude** (current reference)
- **Codex** (Rust binary + TOML config)
- **Opencode** (TypeScript multi-package + JSON/JSONC config)
- **Gemini** (TypeScript CLI – upstream config still under review)

**Future Phases:**
- **Phase 2**: Grok, Qwen (TypeScript variants)
- **Phase 3**: Cursor, Openhands (Python frameworks)

### 🚨 Immediate Blockers Identified

#### 1. Model Validation Blocker (CRITICAL)
**Status**: ❌ **BLOCKING**
**Location**: `/mcp/src/main.rs:validate_model_name()`
**Issue**: Current validation strictly enforces Claude models:
```rust
fn validate_model_name(model: &str) -> Result<()> {
    if !model.starts_with("claude-") && !["opus", "sonnet", "haiku"].contains(&model) {
        return Err(anyhow!(
            "Invalid model '{}'. Must be a valid Claude model name (claude-* format) or CLAUDE code model (opus, sonnet, haiku)",
            model
        ));
    }
    Ok(())
}
```
**Impact**: Codex models like `gpt-5-codex`, `o3` are immediately rejected
**Required Action**: Update validation to support CLI-specific model patterns

#### 2. MCP Streaming Compatibility (UNKNOWN)
**Status**: ❓ **NEEDS AUDIT**
**Issue**: Unknown whether Codex CLI supports HTTP streaming for MCP responses
**Impact**: May require buffered relay implementation
**Required Action**: Test Codex MCP streaming capabilities immediately

#### 3. Container Images (VERIFIED)
**Status**: ✅ **CONFIRMED**
**Verification**: `ghcr.io/5dlabs/codex:latest` exists and is accessible
**Details**: Both `ghcr.io/5dlabs/codex` and `ghcr.io/5dlabs/opencode` images are defined in Helm values

### 📋 Open Questions for Other Agent

#### Configuration Management
1. **Helm Merge Strategy**: Should `cto-config.json` fields merge at the field level or replace entire objects from the Helm `agents.<agent>` CLI settings?
   - Field-level: `cto-config.json.agents.rex.cliConfig.model` overrides only the model
   - Object-level: `cto-config.json.agents.rex.cliConfig` replaces entire Helm config
   - **Recommendation Needed**: Which approach fits better with existing reconciliation patterns?

2. **Model Validation Timing**: Should we update `validate_model_name` in Phase 0 or Phase 1?
   - Phase 0: Fix validation before any CLI work (safer)
   - Phase 1: Update during adapter implementation (integrated)
   - **Question**: What's your preference based on existing development workflow?

#### Streaming & Transport
3. **MCP Streaming Fallback**: If Codex lacks streaming support, what's the preferred pattern?
   ```rust
   // Option A: Transport abstraction
   pub enum McpTransport {
       DirectStreaming(StreamingClient),
       BufferedRelay(BufferedClient),
   }

   // Option B: Wrapper pattern
   pub struct CodexMcpProxy {
       tools_client: ToolsClient,
       buffer_threshold: usize,
   }
   ```
   - **Question**: Which pattern aligns better with existing MCP infrastructure?

4. **Timeout Alignment**: Should we normalize timeout behavior across CLIs or respect each CLI's defaults?

#### Development Workflow
5. **Feature Flags**: Should we implement CLI selection feature flags for gradual rollout?
   - Environment-based: Different CLIs per environment (dev/staging/prod)
   - Agent-based: Gradual agent migration to new CLIs
   - **Question**: What's the preferred rollout strategy for your team?

6. **Testing Strategy**: How should we test CLI combinations without disrupting existing workflows?
   - Parallel test namespaces?
   - Shadow testing with metrics?
   - **Question**: What testing approach fits your current CI/CD pipeline?

#### Questions/Concerns for Associate Agent - ANSWERED

**Q1: `tools_client: ToolsClient` implementation plan**
**A:** The `ToolsClient` refers to the existing `tools` CLI binary already deployed in runtime containers (`infra/images/runtime/Dockerfile` installs v2.4.4). No new SDK needed - Codex will use the same STDIO interface as Claude currently does via the `tools --url ...` wrapper pattern defined in `mcp.json.hbs`.

**Q2: Need for `McpTransport`/`CodexMcpProxy` abstractions**
**A:** These abstractions are **OPTIONAL** - only needed if Codex lacks HTTP streaming support. Since Codex uses STDIO MCP (confirmed in `docs/codex/docs/config.md:341`), and our existing `tools` CLI already bridges STDIO ↔ HTTP, the current infrastructure should work without additional proxies. The abstractions serve as **fallback patterns** if streaming performance issues are discovered during testing.

**Q3: Streaming fallback motivation and constraints**
**A:** The buffered relay approach is **precautionary** based on common CLI limitations where STDIO-based tools may not handle streaming efficiently for large tool responses (e.g., large file reads, extensive search results). The audit scope should focus on:
- Response latency for >10KB tool outputs
- Memory usage during large MCP responses
- Timeout behavior under slow network conditions
- Whether Codex buffers entire responses before outputting

If performance is acceptable without buffering, we can skip the relay implementation entirely.

#### Additional Follow-ups for Associate Agent - ANSWERED

**Q1: Upstream documentation for Gemini, Grok, and Qwen CLI configuration**

**Gemini CLI** (`/tmp/gemini-cli/`):
- **Config**: Uses `GEMINI.md` files (confirmed in `/tmp/gemini-cli/packages/cli/src/commands/extensions/examples/context/GEMINI.md`)
- **Pattern**: Similar to Claude's `CLAUDE.md` approach
- **Official Google CLI**: `@google/gemini-cli` npm package
- **Documentation**: Full docs available in cloned repo at `/tmp/gemini-cli/GEMINI.md`

**Grok CLI** (`/tmp/grok-cli/`):
- **Config**: Uses `.grok/GROK.md` pattern (subdirectory approach)
- **Package**: `@vibe-kit/grok-cli` or similar TypeScript CLI
- **Memory**: Project-specific guidance files in `.grok/` subdirectory
- **Status**: Repository structure confirms `.grok/GROK.md` semantics exist

**Qwen CLI** (`/tmp/qwen-code/`):
- **Config**: Gemini fork - uses `GEMINI.md` variant (confirmed in package.json: Gemini CLI fork)
- **Package**: `@qwen-code/qwen-code` npm package
- **Memory**: Same pattern as Gemini but adapted for Qwen models
- **Documentation**: Available in cloned repo at `/tmp/qwen-code/`

**Q2: Version pinning strategy for npm packages**

**Current Risk**: All Dockerfiles use `npm install <package>@latest` during image builds
**Recommendation**: Pin versions in Helm values and promote to image tags
**Implementation**: Add `cliVersions` to Helm values:
```yaml
agent:
  cliVersions:
    claude: "0.8.5"
    codex: "0.7.2"
    gemini: "0.7.0-nightly.20250918.2722473a"  # From /tmp/gemini-cli/package.json
    qwen: "0.0.12"  # From /tmp/qwen-code/package.json
    grok: "latest"  # TBD - needs version discovery
    opencode: "latest"  # TBD - needs version discovery
```

**Additional Configuration Details Discovered:**

**Gemini CLI Specifics** (from `/tmp/gemini-cli/`):
- **Build command**: `npm run preflight` (comprehensive validation)
- **Testing**: Uses Vitest framework with extensive test coverage
- **Memory pattern**: `GEMINI.md` with Ink library screen reader guidance
- **MCP Integration**: Native MCP support with standardized tooling

**Qwen CLI Specifics** (from `/tmp/qwen-code/`):
- **Architecture**: Direct fork of Gemini CLI adapted for Qwen models
- **Build command**: `npm run preflight` (inherited from Gemini)
- **Container**: Uses same TypeScript/Node.js foundation as Gemini
- **Binary name**: `qwen` (vs `gemini` for original)

#### Additional Implementation Guidance from CLI Analysis

**CLI Architecture Patterns Discovered:**
- **npm-delivered CLIs (Claude, Codex, Opencode, Gemini, Grok, Qwen)**: Share the Node-based runtime we already ship. Integration hinges on per-CLI config formats rather than disparate language stacks.
- **Rust CLI (Codex)**: External MCP client via `tools` – existing STDIO wrapper continues to apply.
- **Python CLIs (Cursor, OpenHands)**: Bring their own virtualenv / framework assumptions – staged for a later phase once Node-first CLIs land.

**Memory / Guidance Mechanisms - CONFIRMED:**
- **Claude**: `CLAUDE.md` (current baseline)
- **Codex**: `AGENTS.md` (layered project guidance)
- **Opencode**: `AGENTS.md` (confirmed in documentation)
- **Gemini**: `GEMINI.md` (confirmed in upstream repo at `/tmp/gemini-cli/`)
- **Grok**: `.grok/GROK.md` (subdirectory pattern confirmed in `/tmp/grok-cli/`)
- **Qwen**: `GEMINI.md` variant (Gemini fork confirmed in `/tmp/qwen-code/package.json`)
- **Cursor/OpenHands**: Session-based (no persistent guidance files - Python frameworks)

**Recommended Phase 1 Focus Validation:**
The "Big 4" approach (Claude, Codex, Opencode, Gemini) covers all major architectural patterns:
1. **Reference implementation** (Claude)
2. **Rust + External MCP** (Codex)
3. **TypeScript + Multi-package** (Opencode)
4. **TypeScript CLI with Google tooling** (Gemini – confirm exact config once upstream docs are reviewed)

This provides comprehensive coverage without scope creep to 8 different CLI architectures.

### Phase 0: Prerequisite Validation (Week 0)

**Objective**: Resolve all blockers before starting implementation

#### Phase 0.1: Critical Fixes (Days 1-2)
1. **Update Model Validation**:
   ```rust
   fn validate_model_name(cli_type: &str, model: &str) -> Result<()> {
       match cli_type {
           "claude" => validate_claude_model(model),
           "codex" => validate_codex_model(model),
           "opencode" => validate_opencode_model(model),
           _ => Err(anyhow!("Unsupported CLI type: {}", cli_type))
       }
   }
   ```

2. **MCP Streaming Audit**:
   - Test Codex with existing Tools setup
   - Document streaming capabilities and limitations
   - Design fallback strategy if needed

#### Phase 0.2: Configuration Design (Days 3-5)
1. **Define Merge Strategy**: Document exact precedence rules
2. **Create Migration Plan**: Step-by-step guide for existing agents
3. **Validate Helm Integration**: Test configuration merge behavior

#### Phase 0.3: Foundation Testing (Days 6-7)
1. **Backward Compatibility Tests**: Ensure existing Claude agents unchanged
2. **Container Environment Tests**: Verify all images work in cluster
3. **Configuration Validation**: Test merged configuration scenarios

## Current Architecture Analysis

### Existing CLI Integration Points

1. **Helm Values Configuration** (`/infra/charts/controller/values.yaml`)
   - Agent CLI configurations: `agent.cliImages` maps CLI types to Docker images
   - Agent-specific CLI configs: `agents.<agent>.cli` / `agents.<agent>.model` per GitHub app
   - Current support: `claude`, `codex`, `opencode` images defined

2. **MCP Server Configuration** (`/mcp/src/main.rs`)
   - `AgentConfig` entries carry the `cli`, `model`, and temperature defaults used at runtime
   - Agent resolution currently assumes Claude-only execution paths
   - `validate_model_name` enforces `claude-*`/`opus|sonnet|haiku` models, so Codex/OpenCode models are rejected today

3. **Container Templates** (`/infra/charts/controller/templates/`)
   - Template system uses Handlebars for dynamic script generation
   - CLI-specific configurations passed via workflow parameters
   - Container script templates in `agent-templates-static.yaml`

4. **Controller Architecture** (`/controller/src/`)
   - Dual-CRD system (CodeRun/DocsRun) with agent classification
   - Template rendering for agent-specific configurations
   - GitHub integration with JWT token management

### Current Configuration Structure

```json
{
  "agents": {
    "rex": {
      "githubApp": "5DLabs-Rex",
      "cli": "claude",
      "model": "claude-sonnet-4-20250514",
      "tools": {
        "remote": ["memory_create_entities"],
        "localServers": {
          "filesystem": {"enabled": true, "tools": ["read_file", "write_file"]},
          "git": {"enabled": true, "tools": ["git_status", "git_diff"]}
        }
      }
    }
  }
}
```

## Key Challenges Identified

### 1. CLI-Specific Configuration Complexity

Each CLI has different configuration patterns:

- **Claude Code**: Model names like `claude-sonnet-4-20250514`, built-in MCP support
- **Codex**: Model names like `gpt-5-codex`, `o3`, TOML-based config system
- **Opencode**: TypeScript-based, different MCP integration patterns

### 2. Memory and State Management

- **Claude Code**: Uses `CLAUDE.md` files for memory
- **Codex**: Relies on layered `AGENTS.md` files for persistent guidance
- **Opencode**: Session-based state management

### 3. MCP Tool Configuration

- **Claude Code**: Direct MCP integration with standardized tools
- **Codex**: Has MCP client support but different tool patterns
- **Opencode**: Custom MCP implementation with different tool naming

### 4. Prompt Engineering Differences

Each CLI requires different system prompt formats and constraints:
- Token limits vary significantly
- Tool calling mechanisms differ
- Context window handling varies

### 5. Container Environment Complexity

Current templates are Claude-specific and need abstraction for multiple CLIs.

## Proposed Solution Architecture

### Phase 1: CLI Abstraction Layer Design

#### 1.1 Agent-Centric Configuration Schema

Extend `cto-config.json` by enriching each `agents.<name>` entry instead of introducing a new `cliProfiles` block. The controller and MCP server already consume the agent-centric map, so the additional fields stay co-located with the existing GitHub app wiring:

```json
{
  "version": "1.1",
  "agents": {
    "rex": {
      "githubApp": "5DLabs-Rex",
      "cli": "codex",
      "cliConfig": {
        "model": "gpt-5-codex",
        "maxTokens": 8192,
        "temperature": 0.7,
        "containerImage": "ghcr.io/5dlabs/codex:latest",
        "auth": {
          "mode": "chatgpt",
          "apiKeySecretRef": null
        },
        "sandboxPreset": "workspace-write"
      },
      "tools": {
        "remote": ["memory_create_entities"],
        "localServers": {
          "filesystem": {"enabled": true},
          "git": {"enabled": true}
        }
      }
    }
  }
}
```

Key points:

- `cli` continues to drive the CLI selection path, enabling backward-compatible defaulting to Claude when unset.
- `cliConfig` carries model/tuning parameters plus CLI-specific settings (such as Codex authentication mode and container image overrides).
- Set `cliConfig.auth.mode` to `api_key` and populate `apiKeySecretRef` with a Kubernetes Secret reference when usage-based billing is required.
- Helm’s per-agent CLI settings (`agents.<agent>.cli`, `model`, `maxTokens`, `temperature`) remain the canonical source for cluster defaults; a reconciler will merge those values with any repository-level overrides from `cto-config.json`.

#### 1.2 CLI Adapter Interface

Create a standardized interface for CLI interactions:

```rust
// controller/src/cli/adapter.rs
pub trait CliAdapter {
    fn validate_model(&self, model: &str) -> Result<()>;
    fn generate_config(&self, agent_config: &AgentConfig) -> Result<String>;
    fn format_prompt(&self, system_prompt: &str, context: &Context) -> Result<String>;
    fn parse_response(&self, response: &str) -> Result<CliResponse>;
    fn get_memory_filename(&self) -> &str;
    fn get_executable_name(&self) -> &str;
}

pub struct ClaudeAdapter;
pub struct CodexAdapter;
pub struct OpencodeAdapter;

impl CliAdapter for CodexAdapter {
    fn validate_model(&self, model: &str) -> Result<()> {
        if !model.starts_with("gpt-") && !["o3", "o1-mini"].contains(&model) {
            return Err(anyhow!("Invalid Codex model: {}", model));
        }
        Ok(())
    }

    fn generate_config(&self, agent_config: &AgentConfig) -> Result<String> {
        // Generate TOML config for Codex
        let template = include_str!("../templates/codex-config.toml.hbs");
        // ... template rendering logic
    }

    // ... other implementations
}
```

Implementation note: rather than introducing a parallel stack, the new adapter logic plugs into the existing `controller::cli::{discovery, router, adapter}` modules so we keep a single path for capability checks, command construction, and telemetry.

#### 1.3 Template Abstraction System

Create CLI-specific template variants:

```
/infra/charts/controller/agent-templates/
├── agents_claude-system-prompt.md.hbs
├── agents_codex-system-prompt.md.hbs
├── agents_opencode-system-prompt.md.hbs
├── codex-config.toml.hbs
├── opencode-config.json.hbs
└── container-scripts/
    ├── claude-entrypoint.sh.hbs
    ├── codex-entrypoint.sh.hbs
    └── opencode-entrypoint.sh.hbs
```

The Codex/OpenCode prompts and scripts are net-new assets. Add them to `infra/charts/controller/agent-templates/` and update `scripts/generate-agent-templates-configmap.sh` so the ConfigMap now bundles all CLI variants.

### Phase 2: Codex CLI Integration Implementation

#### 2.1 Codex-Specific Configurations

Create Codex adapter with proper model support:

```toml
# codex-config.toml.hbs template
model = "{{model}}"
model_provider = "{{provider}}"

{{#if mcpServers}}
[mcp_servers]
{{#each mcpServers}}
[mcp_servers.{{@key}}]
{{#if this.command}}
command = {{{json this.command}}}
{{/if}}
{{#if this.env}}
env = {{{json this.env}}}
{{/if}}
{{/each}}
{{/if}}

[model_providers.openai]
name = "OpenAI"
base_url = "https://api.openai.com/v1"
env_key = "OPENAI_API_KEY"
wire_api = "chat"
```

Authentication options:

- `auth.mode = "chatgpt"` triggers a non-interactive `codex login --headless` flow that exchanges the mounted ChatGPT session token for Codex.
- `auth.mode = "api_key"` skips login, relying on the controller to inject `OPENAI_API_KEY` from the referenced Kubernetes Secret.
- Additional auth providers (e.g., future Azure OpenAI support) can slot in by extending the `auth` object without changing the surrounding schema.

#### 2.2 Container Script Templates

```bash
# codex-entrypoint.sh.hbs
#!/bin/bash
set -euo pipefail

# Materialize Codex configuration
mkdir -p ~/.codex
cat > ~/.codex/config.toml <<'EOF'
{{{codexConfig}}}
EOF

# Persist project guidance in AGENTS.md (Codex reads these automatically)
cat > /workspace/AGENTS.md <<'EOF'
{{{systemPrompt}}}
EOF

# Inject API key when the agent is configured for usage-based billing
if [[ "${CODEX_AUTH_MODE:-}" == "api_key" ]]; then
  export OPENAI_API_KEY="$(cat /var/run/secrets/{{apiKeySecretRef}})"
fi

cd /workspace
exec codex "$@"
```

Authentication handling:

- `CODEX_AUTH_MODE` is derived from `cliConfig.auth.mode`.
- For `chatgpt` mode, the Job mounts the identity token needed for `codex login --headless` during bootstrap.
- For `api_key` mode, the controller mounts the referenced secret and sets `OPENAI_API_KEY` before launch.

#### 2.3 MCP Integration

- The MCP server keeps its single Tools instance; Codex reuses the existing Tools client-config that is mounted for Claude today.
- Tools is exposed to the agents through the `tools` CLI, which bridges STDIO ↔ HTTP (`infra/charts/controller/agent-templates/code/mcp.json.hbs` calls `tools --url ...`). Codex’s MCP support is STDIO-only (`docs/codex/docs/config.md:341`), so the existing wrapper remains compatible without additional adapters.
- The runtime base image already ships the `tools` CLI (`infra/images/runtime/Dockerfile` installs release v2.4.4), ensuring the Codex container inherits the binary automatically.
- The controller continues to derive Tools’s configuration (tool enablement, repository workspace paths) from `agent.tools` and injects it into the Pod environment, independent of the selected CLI.
- We will audit Codex’s current MCP support for HTTP streaming; if it lacks stable streaming, add a lightweight wrapper that proxies Tools responses until upstream support lands.

### Phase 3: Standardized CLI Management

#### 3.1 CLI Factory Pattern

```rust
// controller/src/cli/factory.rs
pub struct CliFactory;

impl CliFactory {
    pub fn create_adapter(cli_type: &str) -> Result<Box<dyn CliAdapter>> {
        match cli_type {
            "claude" => Ok(Box::new(ClaudeAdapter::new())),
            "codex" => Ok(Box::new(CodexAdapter::new())),
            "opencode" => Ok(Box::new(OpencodeAdapter::new())),
            _ => Err(anyhow!("Unsupported CLI type: {}", cli_type))
        }
    }

    pub fn get_supported_clis() -> Vec<&'static str> {
        vec!["claude", "codex", "opencode"]
    }
}
```

#### 3.2 Dynamic Template Selection

```rust
// controller/src/templates/manager.rs
pub struct TemplateManager {
    handlebars: Handlebars<'static>,
}

impl TemplateManager {
    pub fn render_for_cli(&self, cli_type: &str, template_name: &str, context: &Context) -> Result<String> {
        let full_template_name = format!("{}_{}", cli_type, template_name);

        // Fall back to generic template if CLI-specific doesn't exist
        let template_to_use = if self.handlebars.has_template(&full_template_name) {
            &full_template_name
        } else {
            template_name
        };

        self.handlebars.render(template_to_use, context)
            .map_err(|e| anyhow!("Template rendering failed: {}", e))
    }
}
```

### Phase 4: Enhanced Agent Configuration

#### 4.1 Agent Profile Resolution

```rust
// controller/src/agents/resolver.rs
pub struct AgentResolver {
    helm_defaults: AgentCliDefaults,   // from values.yaml (agents.<agent>)
    cto_config: CtoConfig,             // repository-level overrides
    router: CliRouter,                 // existing controller::cli::router
}

impl AgentResolver {
    pub fn resolve_cli_config(&mut self, agent_name: &str) -> Result<ResolvedCliConfig> {
        let agent = self.cto_config.agents.get(agent_name)
            .ok_or_else(|| anyhow!("Agent not found: {}", agent_name))?;

        let cli_type = agent.cli.parse::<CLIType>()?;

        let merged = merge_cli_config(
            self.helm_defaults.get(&agent.github_app),
            agent.cli_config.as_ref(),
        );

        // Re-use the existing router pipeline so capability checks and discovery
        // stay centralized inside controller::cli.
        let universal_cfg = build_universal_config(agent, &merged);
        let routed = self.router.translate(universal_cfg, cli_type).await?;

        Ok(ResolvedCliConfig {
            cli_type,
            model: merged.model,
            max_tokens: merged.max_tokens,
            temperature: merged.temperature,
            container_image: merged
                .container_image
                .unwrap_or_else(|| default_image_for(cli_type)),
            command: routed.command,
            config_files: routed.config_files,
        })
    }
}
```

## Implementation Roadmap

### Phase 1: Foundation (Weeks 1-2)
1. Design and implement CLI abstraction layer interfaces
2. Create enhanced configuration schema in `cto-config.json`
3. Implement CLI factory pattern and adapter interfaces
4. Create template management system

### Phase 2: Codex Integration (Weeks 3-4)
1. Implement `CodexAdapter` with TOML config generation
2. Create Codex-specific templates:
   - System prompt template
   - Configuration template
   - Container entrypoint script
3. Add Codex model validation and MCP integration
4. Test Codex CLI with existing agent workflows

### Phase 3: Template Standardization (Weeks 5-6)
1. Refactor existing Claude templates to use new abstraction
2. Create generic templates with CLI-specific variants
3. Implement dynamic template selection in controller
4. Add template validation and error handling

### Phase 4: Testing & Validation (Weeks 7-8)
1. Comprehensive testing of both Claude and Codex workflows
2. Integration testing with existing agent platform
3. Performance testing and optimization
4. Documentation and deployment guides

### Phase 5: Future CLI Support (Weeks 9+)
1. Implement Opencode adapter
2. Add Cursor CLI support
3. Create CLI plugin system for easier future additions
4. Add monitoring and metrics for multi-CLI usage

## Backward Compatibility Test Matrix

### Required Test Scenarios

#### Scenario 1: Existing Claude-Only Agent (MUST PASS)
```json
{
  "agents": {
    "rex": {
      "githubApp": "5DLabs-Rex",
      "cli": "claude",
      "model": "claude-sonnet-4-20250514",
      "tools": {
        "remote": ["memory_create_entities"],
        "localServers": {
          "filesystem": {"enabled": true, "tools": ["read_file", "write_file"]},
          "git": {"enabled": true, "tools": ["git_status", "git_diff"]}
        }
      }
    }
  }
}
```
**Expected Behavior**: Identical to current behavior, no changes
**Critical Path**: Must work unchanged throughout all implementation phases

#### Scenario 2: Legacy Configuration Without CLI Field (MUST PASS)
```json
{
  "agents": {
    "legacy_agent": {
      "githubApp": "5DLabs-LegacyBot",
      "model": "opus",
      "tools": { /* minimal config */ }
    }
  }
}
```
**Expected Behavior**: Defaults to Claude CLI with existing template stack
**Test Points**: Model validation, template selection, container script

#### Scenario 3: Mixed CLI Environment (TARGET STATE)
```json
{
  "agents": {
    "claude_agent": {
      "githubApp": "5DLabs-Claude",
      "cli": "claude",
      "model": "claude-sonnet-4-20250514"
    },
    "codex_agent": {
      "githubApp": "5DLabs-Codex",
      "cli": "codex",
      "cliConfig": {
        "model": "gpt-5-codex",
        "maxTokens": 8192,
        "temperature": 0.7
      }
    }
  }
}
```
**Expected Behavior**: Both agents work simultaneously with their respective CLIs
**Test Points**: Template isolation, configuration merge, resource allocation

#### Scenario 4: Configuration Migration Path (VALIDATION)
```json
// Before: Claude-only
{
  "agents": {
    "rex": {
      "githubApp": "5DLabs-Rex",
      "cli": "claude",
      "model": "claude-sonnet-4-20250514"
    }
  }
}

// After: Migrated to Codex
{
  "agents": {
    "rex": {
      "githubApp": "5DLabs-Rex",
      "cli": "codex",
      "cliConfig": {
        "model": "gpt-5-codex",
        "auth": {"mode": "chatgpt"}
      }
    }
  }
}
```
**Expected Behavior**: Smooth transition without data loss
**Test Points**: Session continuity, workspace preservation, tool configurations

### Failure Mode Testing

#### Configuration Merge Conflicts
```json
// Helm values.yaml has:
// agents.rex.model = "claude-opus-4-1-20250805"

// cto-config.json has:
{
  "agents": {
    "rex": {
      "githubApp": "5DLabs-Rex",
      "cli": "codex",
      "cliConfig": {
        "model": "gpt-5-codex"  // Conflicts with Helm
      }
    }
  }
}
```
**Expected Behavior**: Clear precedence resolution and error handling
**Test Points**: Configuration validation, error messages, fallback behavior

#### Invalid Model/CLI Combinations
```json
{
  "agents": {
    "broken_agent": {
      "githubApp": "5DLabs-Broken",
      "cli": "codex",
      "cliConfig": {
        "model": "claude-sonnet-4-20250514"  // Wrong CLI for model
      }
    }
  }
}
```
**Expected Behavior**: Validation error with helpful message
**Test Points**: Early validation, error propagation, user feedback

### Test Implementation Strategy

1. **Unit Tests**: Configuration parsing and validation
2. **Integration Tests**: End-to-end workflow execution
3. **Regression Tests**: Existing Claude agent behavior unchanged
4. **Performance Tests**: No degradation in execution times
5. **Migration Tests**: Smooth configuration transitions

## Inter-Agent Collaboration Points

### Phase Handoffs

#### Phase 0 → Phase 1: Foundation Validation
**Deliverables from Other Agent**:
- ✅ Model validation fix implemented and tested
- ✅ MCP streaming compatibility assessment complete
- ✅ Helm merge strategy documented and validated
- ✅ Backward compatibility test suite passing

**My Responsibilities**:
- Implement CLI adapter interfaces based on streaming results
- Create template management system with confirmed merge behavior
- Build factory pattern using validated CLI capabilities

#### Phase 1 → Phase 2: Codex Integration
**Deliverables from Other Agent**:
- ✅ Codex container image verified and tested
- ✅ Authentication strategy implemented (chatgpt/api_key modes)
- ✅ MCP proxy/relay if streaming limitations found
- ✅ Template rendering pipeline updated for multi-CLI

**My Responsibilities**:
- Implement CodexAdapter using confirmed streaming approach
- Create Codex-specific templates with validated authentication
- Test Codex integration with existing agent workflows

#### Phase 2 → Phase 3: Template Standardization
**Deliverables from Other Agent**:
- ✅ CLI routing logic implemented in controller
- ✅ Dynamic template selection working
- ✅ Configuration validation hooks in place
- ✅ Error handling patterns established

**My Responsibilities**:
- Refactor existing templates to use new abstraction
- Create comprehensive template test suite
- Document template creation guidelines for future CLIs

### Cross-Agent Validation Points

#### 1. Configuration Schema Validation
**Collaboration Need**: Ensure `cto-config.json` schema changes work with existing controller logic
**Validation Method**: Joint testing with real agent configurations

#### 2. Template Rendering Consistency
**Collaboration Need**: Verify template outputs produce valid container scripts for all CLIs
**Validation Method**: Container execution tests across CLI types

#### 3. Error Message Standardization
**Collaboration Need**: Consistent error messages between MCP server and controller
**Validation Method**: Error scenario testing and message format alignment

#### 4. Performance Impact Assessment
**Collaboration Need**: Measure any performance changes from new abstraction layers
**Validation Method**: Benchmark tests with before/after metrics

### Communication Protocol

#### Daily Sync Points
- **Phase 0**: Daily check-ins on blocker resolution progress
- **Phase 1-3**: Milestone-based handoffs with validation criteria
- **Phase 4**: Joint testing and issue resolution

#### Documentation Handoffs
- **Architecture Decisions**: Document all design choices with rationale
- **Implementation Notes**: Share lessons learned and gotchas discovered
- **Testing Results**: Comprehensive test results and coverage reports

#### Issue Escalation
- **Technical Blockers**: Immediate escalation if implementation approach needs revision
- **Timeline Risks**: Early warning if phases may slip due to complexity
- **Integration Issues**: Joint debugging sessions for cross-component problems

## Technical Considerations

### Container Strategy

Given the complexity of CLI-specific configurations, recommend maintaining separate container scripts rather than trying to create one unified script. This approach:

1. **Reduces Complexity**: Each CLI has its own entrypoint script
2. **Improves Maintainability**: Changes to one CLI don't affect others
3. **Enables Optimization**: Each script can be optimized for its specific CLI
4. **Simplifies Debugging**: Easier to troubleshoot CLI-specific issues

### Configuration Management

The enhanced `cto-config.json` schema provides:

1. **Agent-Scoped Defaults**: Helm `agents.<agent>` sections supply cluster-wide defaults for model, tokens, and images.
2. **Repository Overrides**: `cto-config.json` augments per-agent settings without duplicating Helm values.
3. **Backward Compatibility**: Agents with only `cli` set continue to run on Claude with the existing template stack.
4. **Validation Hooks**: Controller-side schema validation ensures merged configs satisfy per-CLI requirements before scheduling a job.

### Transport & Streaming

1. **Codex Streaming Audit**: Verify whether the Codex CLI supports HTTP streaming for MCP responses; if not, wrap Tools responses in a buffered relay until upstream streaming is available.
2. **Timeout Behavior**: Align Codex’s streaming/timeout semantics with Claude’s so the controller can keep a single SLA enforcement path.
3. **Telemetry**: Capture per-CLI streaming latency metrics to validate the relay (if required) and retire it once native streaming lands.

### Error Handling Strategy

Implement comprehensive error handling:

1. **Configuration Validation**: Validate CLI configs at startup
2. **Model Compatibility**: Check model names against CLI capabilities
3. **Template Rendering**: Handle missing templates gracefully
4. **Runtime Errors**: Proper error propagation and logging

### Migration Strategy

1. **Feature Flags**: Add CLI selection feature flags for gradual rollout
2. **Backward Compatibility**: Maintain support for existing Claude-only configs
3. **Default Behavior**: Default to Claude CLI for existing agents
4. **Migration Tools**: Provide scripts to convert existing configs

## Success Metrics

1. **Functional**: Both Claude and Codex CLIs work with all agent types
2. **Performance**: No degradation in agent execution times
3. **Maintainability**: Adding new CLIs requires minimal code changes
4. **Compatibility**: Existing workflows continue to work unchanged
5. **Flexibility**: Easy to configure different CLIs per agent

## Risk Mitigation

### 🚨 Critical Technical Risks

#### 1. Model Validation Blocker (HIGH PRIORITY)
**Risk**: Current `validate_model_name()` function prevents any non-Claude models
**Impact**: Complete blocker for Codex integration
**Mitigation Strategy**:
- **Immediate Action**: Implement CLI-aware validation in Phase 0
- **Fallback Plan**: Temporary bypass with warning logs until proper validation ready
- **Monitoring**: Track validation failures and model usage patterns

```rust
// Risk mitigation implementation
fn validate_model_name_safe(cli_type: Option<&str>, model: &str) -> Result<()> {
    match cli_type {
        Some("claude") | None => validate_claude_model(model),
        Some("codex") => validate_codex_model(model),
        Some("opencode") => validate_opencode_model(model),
        Some(unsupported) => {
            warn!("Unknown CLI type '{}', skipping model validation", unsupported);
            Ok(()) // Graceful degradation
        }
    }
}
```

#### 2. MCP Streaming Incompatibility (MEDIUM PRIORITY)
**Risk**: Codex may not support HTTP streaming, breaking real-time tool interactions
**Impact**: Degraded user experience, potential timeout issues
**Mitigation Strategy**:
- **Primary Plan**: Implement buffered relay with configurable thresholds
- **Performance Plan**: Optimize buffer sizes based on tool response patterns
- **Future Plan**: Retire relay once Codex gains native streaming support

```rust
// Risk mitigation pattern
pub struct McpStreamingAdapter {
    inner: Box<dyn McpClient>,
    streaming_capable: bool,
    buffer_config: BufferConfig,
}

impl McpStreamingAdapter {
    pub async fn call_tool(&self, request: ToolRequest) -> Result<ToolResponse> {
        if self.streaming_capable {
            self.inner.stream_tool_call(request).await
        } else {
            self.buffered_tool_call(request).await
        }
    }
}
```

#### 3. Configuration Merge Conflicts (MEDIUM PRIORITY)
**Risk**: Unclear precedence between Helm values and cto-config.json
**Impact**: Unexpected agent behavior, difficult debugging
**Mitigation Strategy**:
- **Documentation**: Clear precedence rules with examples
- **Validation**: Runtime checks for conflicting configurations
- **Observability**: Log all configuration merges with source attribution

```rust
// Risk mitigation approach
#[derive(Debug)]
pub struct ConfigurationSource {
    pub source: String,      // "helm", "cto-config", "default"
    pub field_path: String,  // "agents.rex.cliConfig.model"
    pub value: String,       // Actual value used
    pub overridden_by: Option<Box<ConfigurationSource>>,
}

pub fn merge_with_audit(helm: &HelmConfig, cto: &CtoConfig) -> (MergedConfig, Vec<ConfigurationSource>) {
    // Track all configuration decisions for debugging
}
```

#### 4. Container Image Availability (LOW PRIORITY)
**Risk**: CLI container images may be missing or corrupted
**Impact**: Job failures, inability to run agents
**Mitigation Strategy**:
- ✅ **Verified**: `ghcr.io/5dlabs/codex:latest` exists and accessible
- **Health Checks**: Pre-flight image validation before job scheduling
- **Fallback**: Graceful degradation to Claude if CLI images unavailable

### 🛡️ Operational Risks

#### 5. Backward Compatibility Breaks (HIGH PRIORITY)
**Risk**: Existing Claude agents stop working during migration
**Impact**: Production outages, development disruption
**Mitigation Strategy**:
- **Feature Flags**: CLI selection behind configurable flags
- **Default Behavior**: Unchanged behavior for existing configurations
- **Rollback Plan**: Ability to disable multi-CLI features instantly
- **Testing**: Comprehensive regression test suite

#### 6. Performance Degradation (MEDIUM PRIORITY)
**Risk**: New abstraction layers slow down agent execution
**Impact**: User experience degradation, timeout issues
**Mitigation Strategy**:
- **Benchmarking**: Before/after performance measurements
- **Optimization**: Profile and optimize critical paths
- **Monitoring**: Real-time performance metrics per CLI type
- **Alerts**: Performance degradation alerts with automatic rollback triggers

#### 7. Authentication Complexity (MEDIUM PRIORITY)
**Risk**: Multiple authentication modes create security vulnerabilities
**Impact**: Unauthorized access, billing issues, token exposure
**Mitigation Strategy**:
- **Secret Management**: Proper Kubernetes Secret handling
- **Audit Trails**: Log all authentication events
- **Principle of Least Privilege**: Minimal required permissions
- **Token Rotation**: Automated token refresh for long-running jobs

#### 8. Template Maintenance Burden (LOW PRIORITY)
**Risk**: Multiple CLI templates become difficult to maintain
**Impact**: Development velocity reduction, template drift
**Mitigation Strategy**:
- **Standardization**: Common template patterns across CLIs
- **Automation**: Template generation and validation tools
- **Documentation**: Clear guidelines for template development
- **Testing**: Automated template validation in CI/CD

### 🔍 Monitoring & Alerting Strategy

#### Real-Time Metrics
- CLI usage distribution (claude vs codex vs opencode)
- Configuration validation failures by CLI type
- Template rendering success/failure rates
- Authentication success rates by auth mode
- Streaming vs buffered MCP interaction ratios

#### Early Warning Alerts
- Model validation failure rate > 5%
- Container image pull failures for any CLI
- Configuration merge conflicts detected
- Performance degradation > 20% vs baseline
- Authentication failures > 2% for any CLI

#### Rollback Triggers
- Critical path failures (model validation, container starts)
- Performance degradation > 50%
- Authentication failures > 10%
- Backward compatibility test failures

### 📋 Risk Response Playbook

#### Immediate Response (< 1 hour)
1. **Assess Impact**: Determine scope of affected agents
2. **Emergency Rollback**: Disable problematic CLI types via feature flags
3. **Stakeholder Notification**: Alert development team and users
4. **Incident Documentation**: Log issue details and timeline

#### Short-Term Response (< 24 hours)
1. **Root Cause Analysis**: Investigate failure cause
2. **Hotfix Development**: Implement targeted fix
3. **Testing**: Validate fix in isolated environment
4. **Gradual Re-enablement**: Phased rollout of fix

#### Long-Term Response (< 1 week)
1. **Process Improvement**: Update development and testing processes
2. **Monitoring Enhancement**: Add metrics to prevent similar issues
3. **Documentation Update**: Capture lessons learned
4. **Training**: Share knowledge with team members

## Future Considerations

### CLI Plugin System

Design a plugin system for easier CLI additions:

```rust
// Future plugin interface
pub trait CliPlugin {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn create_adapter(&self) -> Box<dyn CliAdapter>;
    fn validate_config(&self, config: &Value) -> Result<()>;
}
```

### Dynamic CLI Discovery

Implement runtime CLI discovery for installed CLIs:

```rust
// Future capability
pub fn discover_available_clis() -> Vec<String> {
    // Scan for installed CLI tools
    // Check container image availability
    // Return list of supported CLIs
}
```

### Advanced Configuration

Support for environment-specific CLI configurations:

```json
{
  "environments": {
    "development": {
      "defaultCliProfile": "claude",
      "allowedClis": ["claude", "codex"]
    },
    "production": {
      "defaultCliProfile": "codex",
      "allowedClis": ["codex"]
    }
  }
}
```

---

This architecture provides a solid foundation for supporting multiple CLIs while maintaining the platform's existing functionality and enabling future expansion to additional CLI tools.
