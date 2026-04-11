# Dynamic MCP Tool SDK — Implementation Prompt

> **Methodology**: Structured using the [Anthropic Prompt Generator](https://docs.anthropic.com/en/docs/build-with-claude/prompt-engineering/prompt-generator) pattern.
> Variables use `{$VAR}` syntax. Inject actual values before use. Default values provided in the Appendix.

---

## Inputs

```
{$ESCALATION_RS}
{$HTTP_SERVER_RS}
{$BRIDGE_STATE_DEFINITION}
{$CODERUN_CRD_SPEC}
{$TOOL_CATALOG_SUMMARY}
{$MODEL_PROVIDERS}
{$EXISTING_CLI_TEMPLATE}
{$ESCALATION_POLICY_JSON}
{$TOOLS_SHARED_PARTIALS}
```

---

## Instructions

You are a senior platform engineer building a **TypeScript SDK for MCP tool access** — a code execution layer that lets AI agents import MCP tools as typed functions, write scripts, and run them in a Deno sandbox. This directly implements the architecture described in Anthropic's "Code Execution with MCP" engineering article (reproduced in full below).

Your task: design and implement this **dynamic MCP tool loading system** for a multi-agent Kubernetes platform so agents can discover, request, and invoke MCP tools mid-session without pod restarts or static tool filtering.

The system must support multiple agent runtimes (Claude Code, Codex, Gemini, OpenCode, Cursor, Factory) with different tool invocation capabilities. You will produce the implementation for **PR 3 — Code Execution Mode**, write the code, and validate each component.

---

### Primary Design Reference — Anthropic "Code Execution with MCP"

This is the **design bible** for the SDK. Read it carefully before writing any code.

<anthropic_research_article>
#### Problem: Excessive Token Consumption

1. **Tool definitions overload context.** Loading 150+ full tool schemas upfront consumes hundreds of thousands of tokens before the agent even reads the user's request.
2. **Intermediate results bloat context.** Every tool call result passes through the model. A 2-hour meeting transcript flowing through two tool calls = ~50,000 wasted tokens.

#### Solution: Code Execution with MCP

Present MCP servers as **code APIs** rather than direct tool calls. The agent writes TypeScript that imports only the tools it needs. Data flows through the execution environment, not the model.

**File tree structure (filesystem-as-discovery):**
```
servers/
├── google-drive/
│   ├── getDocument.ts
│   └── index.ts
├── salesforce/
│   ├── updateRecord.ts
│   └── index.ts
└── ...
```

**Each tool file is a typed wrapper:**
```typescript
// ./servers/google-drive/getDocument.ts
import { callMCPTool } from "../../../client.js";

interface GetDocumentInput {
  documentId: string;
}

interface GetDocumentResponse {
  content: string;
}

export async function getDocument(input: GetDocumentInput): Promise<GetDocumentResponse> {
  return callMCPTool<GetDocumentResponse>('google_drive__get_document', input);
}
```

**Agent writes code, not individual tool calls:**
```typescript
import * as gdrive from './servers/google-drive';
import * as salesforce from './servers/salesforce';

const transcript = (await gdrive.getDocument({ documentId: 'abc123' })).content;
await salesforce.updateRecord({
  objectType: 'SalesMeeting',
  recordId: '00Q5f000001abcXYZ',
  data: { Notes: transcript }
});
```

#### Key Benefits

- **Progressive disclosure.** Agents navigate the filesystem to discover tools on demand (`ls servers/`, `cat servers/github/search_code.ts`). Only the tools needed for the current task enter context.
- **Context-efficient results.** Filter 10,000 rows down to 5 in code before returning to the model. Intermediate data never touches the context window.
- **Powerful control flow.** Loops, conditionals, error handling in code — not chained tool calls. A conditional tree executes in the sandbox, not as N model turns.
- **Privacy-preserving.** Intermediate results stay in the execution environment. PII can be tokenized before reaching the model and untokenized on the way to downstream tools.
- **State persistence and skills.** Agents write intermediate results to files, save reusable functions as skills (`.ts` files with `SKILL.md`), and build a toolbox of higher-level capabilities over time.

#### Important Caveat

> "Code execution introduces its own complexity. Running agent-generated code requires a secure execution environment with appropriate sandboxing, resource limits, and monitoring. These infrastructure requirements add operational overhead and security considerations that direct tool calls avoid."

We address this with **Deno's built-in permission model** (`--allow-net`, `--allow-read`, `--deny-all`), execution timeouts, and script size limits.
</anthropic_research_article>

---

### Existing System Context

Here is the existing escalation engine (Rust, pure logic, already implemented in PR 1). This is the decision logic your TypeScript SDK calls via the HTTP proxy when an agent requests a capability:

<escalation_engine>
{$ESCALATION_RS}
</escalation_engine>

Here is the HTTP server module that hosts the MCP proxy. Your `mcp.ts` runtime library makes `fetch()` calls to this server's `/mcp` endpoint:

<http_server>
{$HTTP_SERVER_RS}
</http_server>

Here is the `BridgeState` struct. Note the `session_states` and `default_escalation_policy` fields added in PR 1 — your SDK interacts with these via `X-Agent-Id` and `X-Agent-Prewarm` headers:

<bridge_state>
{$BRIDGE_STATE_DEFINITION}
</bridge_state>

Here is the CodeRun CRD specification. The `escalation_policy`, `remote_tools`, and `local_tools` fields determine what tools an agent pod can access:

<coderun_crd>
{$CODERUN_CRD_SPEC}
</coderun_crd>

Here is the tool catalog — 150+ tools across 23 categories served via MCP JSON-RPC:

<tool_catalog>
{$TOOL_CATALOG_SUMMARY}
</tool_catalog>

Here are the model providers and agent runtimes. Each CLI runtime has different capabilities — some support native deferred tools (Claude Code), others rely entirely on CLI/filesystem discovery:

<model_providers>
{$MODEL_PROVIDERS}
</model_providers>

Here is the existing CLI template for agent pods. The `cto-tools-setup` partial should be invoked after the existing CLI setup:

<cli_template>
{$EXISTING_CLI_TEMPLATE}
</cli_template>

Here is the default escalation policy:

<escalation_policy>
{$ESCALATION_POLICY_JSON}
</escalation_policy>

Here are the existing shared template partials — your new `cto-tools-setup.sh.hbs` partial fits alongside these:

<shared_partials>
{$TOOLS_SHARED_PARTIALS}
</shared_partials>

---

### Core Design Principles

Follow these principles throughout your design and implementation:

1. **Token cost scales with usage, not catalog size.** Agents must not load hundreds of full tool schemas into context. Native-deferred runtimes (Claude Code) carry only tool names until needed; non-native runtimes (Codex, Gemini, OpenCode) discover tools on demand via CLI or filesystem.
2. **No restarts for capability expansion.** An agent that realises mid-task it needs Grafana, Terraform, or any other tool should be able to acquire it in-session through an auditable escalation path.
3. **Least privilege by default.** Pre-warmed tools are an allowlist for eager loading. Everything else goes through an `EscalationPolicy` (allow/deny globs + modes: auto, allowlist, review). Grants and denials are logged to the CRD status for audit.
4. **Runtime-agnostic backend.** Adding a new agent runtime should only require teaching its adapter to mount the CLI / inject the prompt block — the HTTP proxy, escalation engine, and tool catalog remain unchanged.
5. **Code execution over round-tripping.** For multi-step tool workflows, agents should write TypeScript scripts that import typed MCP tool bindings and run in a Deno sandbox, collapsing N model turns into 1 script execution.

---

### Architecture — What to Build

You are implementing **PR 3 — Code Execution Mode**. PRs 1 (escalation engine) and 2 (CRD + controller integration) are complete. The escalation engine exists in `crates/tools/src/escalation.rs`, the CRD has `escalation_policy: Option<EscalationPolicy>`, and `X-Agent-Id` / `X-Agent-Prewarm` headers are already threaded through the controller.

#### Component 1: TypeScript SDK Codegen (`codegen.ts`)

Build a code generator that reads the tool catalog from the MCP server at pod startup and emits the `/.cto-tools/` directory tree:

```
/.cto-tools/
├── servers/
│   ├── github/
│   │   ├── search_code.ts      ← one file per tool, typed wrapper
│   │   ├── get_file_contents.ts
│   │   └── index.ts            ← re-exports all tools in server
│   ├── grafana/
│   │   ├── query_prometheus.ts
│   │   ├── query_loki.ts
│   │   └── index.ts
│   ├── linear/
│   │   ├── create_issue.ts
│   │   └── index.ts
│   ├── filesystem/              ← local tool, same namespace
│   └── terraform/
├── mcp.ts                      ← routing helper (local vs remote)
├── codegen.ts                  ← generates the tree from catalog
├── deno.json                   ← import map + permissions
└── README.md                   ← agent-readable: "how to write a script"
```

**How codegen works:**
1. Call `tools/list` on the MCP server (`fetch()` to `TOOLS_SERVER_URL`).
2. Parse each tool name to extract the server prefix (e.g. `github_search_code` → server `github`, function `search_code`).
3. For each tool, emit a `.ts` file with a typed wrapper function. Derive TypeScript interface types from the tool's `inputSchema` (JSON Schema → TypeScript interfaces).
4. For each server directory, emit an `index.ts` that re-exports all tool functions.
5. Write `deno.json` with the import map and permission configuration.
6. Write `README.md` with agent-readable instructions.

Each tool file follows this pattern:
```typescript
// /.cto-tools/servers/github/search_code.ts
import { callTool } from "../../mcp.ts";

/** Search code across GitHub repositories.
 *
 * @example
 * const results = await search_code({ q: "EscalationPolicy", repo: "5dlabs/cto" });
 * console.log(results.total_count);
 */
export async function search_code(
  args: { q: string; repo?: string; per_page?: number }
): Promise<{ total_count: number; items: Array<{ path: string; sha: string }> }> {
  return callTool("github_search_code", args);
}
```

#### Component 2: `mcp.ts` Runtime Library (~150 lines)

The SDK core. Exports:

| Function | Purpose |
|----------|---------|
| `listTools()` | Returns tool names grouped by server (for discovery) |
| `describeTool(name: string)` | Returns a tool's input schema and description (for understanding what a tool needs) |
| `callTool<T>(name: string, args: Record<string, unknown>)` | Calls a tool via JSON-RPC and returns typed result |
| `escalate(toolName: string, reason: string)` | Requests a capability via the escalation engine |

**Internal implementation:**
- `rpc(method: string, params: Record<string, unknown>)` — handles JSON-RPC 2.0 framing with `fetch()`. Sends `X-Agent-Id` and `X-Agent-Prewarm` headers.
- Routes local tools to `localhost:3001` (local MCP sidecar), remote tools to the cluster service.
- Environment variables consumed:
  - `TOOLS_SERVER_URL` — remote MCP HTTP endpoint (default: `http://cto-tools.cto.svc.cluster.local:3000/mcp`)
  - `LOCAL_TOOLS_URL` — local MCP sidecar (default: `http://localhost:3001/mcp`)
  - `CTO_AGENT_ID` — agent identity for `X-Agent-Id` header
  - `CTO_AGENT_PREWARM` — comma-separated pre-warmed tool list for `X-Agent-Prewarm` header

**Error handling:**
- Wrap JSON-RPC errors in a `ToolError` class with `code`, `message`, and `data` fields.
- Distinguish: tool not in catalog (`code: -32601`), policy denied (`code: -32403`), upstream server down (`code: -32000`).
- Include retry logic for transient failures (503, connection refused) with exponential backoff (max 3 retries).

#### Component 3: Agent Script Execution Pattern

Agents write scripts that import tools and execute complex workflows in a single sandbox run:

```typescript
// Agent writes to /tmp/find-escalation.ts
import { search_code } from "/.cto-tools/servers/github/search_code.ts";
import { write } from "/.cto-tools/servers/filesystem/write.ts";

const hits = await search_code({ q: "EscalationPolicy", repo: "5dlabs/cto" });
const top5 = hits.items.slice(0, 5).map(h => ({ path: h.path }));
await write({ path: "/tmp/findings.json", content: JSON.stringify(top5, null, 2) });
console.log(`kept ${top5.length} of ${hits.total_count}`);
// Agent context gets ONE line: "kept 5 of 1247". The 1247 raw results never touch the context window.
```

**Deno sandbox command:**
```bash
deno run \
  --allow-net=cto-tools.cto.svc.cluster.local,localhost \
  --allow-read=/workspace,/.cto-tools \
  --allow-write=/tmp \
  --allow-env=TOOLS_SERVER_URL,LOCAL_TOOLS_URL,CTO_AGENT_ID,CTO_AGENT_PREWARM \
  /tmp/find-escalation.ts
```

**Timeout enforcement:** Wrap `deno run` with `timeout 120` (configurable via `CTO_TOOLS_TIMEOUT` env var, default 120s).
**Script size limit:** Reject scripts > 100KB before execution.

#### Component 4: Bash CLI Shim (`cto-tools`)

A shell script wrapping `curl` + `jq` for simple operations. Must work without Deno installed (fallback for constrained environments).

```bash
# List all tools (names only, grouped by server)
cto-tools mcp list
cto-tools mcp list --category github    # filter by server/category

# Describe a specific tool (show input schema)
cto-tools mcp describe github_search_code

# Call a tool with JSON args
cto-tools mcp call github_search_code --json '{"q": "EscalationPolicy", "repo": "5dlabs/cto"}'

# Escalate to request a new capability
cto-tools mcp escalate terraform_plan --reason "Need to preview infra changes for PR #42"

# Execute a TypeScript snippet inline (like python -c but for Deno)
cto-tools exec -e 'import { search_code } from "/.cto-tools/servers/github/search_code.ts"; console.log(await search_code({q: "test"}))'

# Execute a .ts file in the Deno sandbox
cto-tools exec /tmp/my-script.ts

# Show version and environment
cto-tools version
```

**Implementation details:**
- JSON-RPC calls via `curl -s -X POST $TOOLS_SERVER_URL -H "Content-Type: application/json" -H "X-Agent-Id: $CTO_AGENT_ID"`
- `jq` for JSON formatting and field extraction
- `exec` subcommand wraps `deno run` with the correct permission flags
- Exit codes: 0 = success, 1 = tool error, 2 = policy denied, 3 = not in catalog, 4 = server unreachable

#### Component 5: Agent Skill (`cto-tools` skill)

Create a `SKILL.md` file that teaches agents how to use the SDK. This skill is loaded into the agent's context at session start.

**Trigger conditions:**
- "Use this skill any time you need a capability not in your eager toolset."
- "Use it when you're about to give up because 'I don't have a tool for X' — you probably do."
- "Use it for any multi-step tool workflow where intermediate results don't need to be in your context."

**Sections to include:**
1. **Discovery flow** — `cto-tools mcp list` (or `ToolSearch` for Claude), with category filtering. Scan names first, then `describe` only candidates.
2. **Simple invocation** — `cto-tools mcp call <tool> --json '{...}'` with realistic examples for github, linear, grafana, argocd.
3. **Code execution pattern** — When to write `.ts` files vs. using CLI: multi-step workflows with filtering/aggregation → write TypeScript; single call → use CLI.
4. **Escalation flow** — When to call `cto-tools mcp escalate`, how to write a useful reason, how to interpret grant vs deny.
5. **Error handling** — "tool not in catalog" vs "policy denied" vs "upstream MCP server down" — with recovery steps for each.
6. **Anti-patterns** — Don't `list` on every turn (cache mentally), don't escalate for tools already in your eager set, don't describe all 150 tools.
7. **Dual-runtime** — If `cto-tools` is on PATH, use Bash examples; if `ToolSearch` is a native tool, use those. Claude Code agents can use both.

#### Component 6: Container & Deployment

- **Deno in Dockerfile:** Add Deno to the agent base Dockerfile:
  ```dockerfile
  RUN curl -fsSL https://deno.land/install.sh | DENO_INSTALL=/usr/local sh
  ```

- **Handlebars partial** (`templates/_shared/partials/cto-tools-setup.sh.hbs`):
  - Copies `mcp.ts`, `codegen.ts`, and `deno.json` from `/task-files/` to `/.cto-tools/`
  - Runs `codegen.ts` to generate the `servers/` tree from the live tool catalog
  - Copies the `cto-tools` CLI to `/usr/local/bin/` and `chmod +x`
  - Exports `CTO_AGENT_ID` from `{{github_app_name}}`
  - Exports `CTO_AGENT_PREWARM` from the CodeRun's `remoteTools` field
  - Creates `README.md` in `/.cto-tools/`

- **Wire into container script:** Add `{{> cto-tools-setup}}` to the shared container script template, after the existing `{{> tools-config}}` partial.

- **Skill registration:** Add `cto-tools` to every agent's default skills list in `cto-config.json`.

---

### Runtime-Specific Behavior

<runtime_behavior>
**Claude Code (native deferred tools):**
- At pod startup, `.mcp.json` declares the full catalog with most tools `deferred: true`.
- Agent sees tool names in a system reminder (minimal tokens). Uses `ToolSearch` to fetch schemas on demand.
- `tools_request_capability` only fires for tools outside the startup catalog.
- Claude Code can ALSO use `cto-tools exec` for multi-step workflows where code execution is more token-efficient than sequential tool calls.
- Skill instructions should include both native tool examples AND `cto-tools exec` examples.

**Codex / OpenCode / Factory (CLI runtimes):**
- Agent gets a core toolset (filesystem, shell, git) + a system prompt block saying: "You have `cto-tools` on your PATH. Run `cto-tools mcp list` to discover 150+ additional tools."
- For simple ops: `cto-tools mcp call github_search_code --json '{...}'`
- For multi-step workflows: write a `.ts` script importing from `/.cto-tools/servers/`, run with `cto-tools exec`.
- Escalation: `cto-tools mcp escalate <name> --reason '...'`

**Cursor / Gemini:**
- Same as Codex path, but may also have native tool support.
- The `cto-tools` CLI is the universal fallback that works regardless of runtime capabilities.
</runtime_behavior>

---

### Environment Variables

These are available inside agent pods. Reference these in your implementation:

```bash
# ── Core Platform
NAMESPACE="cto"
TOOLS_SERVER_URL="http://cto-tools.cto.svc.cluster.local:3000/mcp"
PORT="3000"
SYSTEM_CONFIG_PATH="./infra/charts/cto/templates/tools"
AGENT_TEMPLATES_PATH="./templates"

# ── Agent Identity (injected per-pod by controller)
CTO_AGENT_ID=""                    # set by cto-tools-setup partial from GITHUB_APP_NAME
GITHUB_APP_NAME=""                 # e.g. "5DLabs-Rex", "5DLabs-Bolt"

# ── AI Provider Keys (available but not used by SDK directly)
ANTHROPIC_API_KEY=""
OPENAI_API_KEY=""

# ── MCP Server
MCP_SERVER_URL="http://cto-tools.cto.svc.cluster.local:3000/mcp"

# ── SCM
GITHUB_TOKEN=""
GH_TOKEN=""

# ── SDK-Specific
CTO_TOOLS_TIMEOUT="120"           # script execution timeout in seconds
CTO_TOOLS_MAX_SCRIPT_SIZE="102400" # max script size in bytes (100KB)
LOCAL_TOOLS_URL="http://localhost:3001/mcp" # local MCP sidecar
```

---

### PR Context

| PR | Scope | Status |
|----|-------|--------|
| **PR 1** | Escalation engine (`escalation.rs`) + HTTP proxy (`tools_request_capability` handler) | ✅ Complete |
| **PR 2** | CRD (`EscalationPolicy` in CodeRunSpec) + controller (header injection, backward-compat shim) | ✅ Complete |
| **PR 3** | **Code execution mode (YOU ARE HERE):** `mcp.ts`, `codegen.ts`, `cto-tools` CLI, Deno in Dockerfile, setup partial, agent skill | 🔨 Implementing |
| **PR 4** | Local tool unification: narrow `tools-client` to local-only, `localhost:3001` sidecar, codegen routing | ⏳ Future |

---

### Output Format

For each file you create or modify, provide:

1. The full file path relative to the repository root.
2. The complete implementation code — no placeholders, no `// ...rest of implementation`.
3. Unit/integration tests where applicable (use `Deno.test()` for TypeScript, `bats` or inline test functions for bash).
4. A brief rationale for key design decisions.

Structure your output as:
```
## File: <path>
### Rationale
<brief explanation>
### Code
<full implementation>
### Tests
<test code if applicable>
```

---

### Thinking Process

Before writing any code, think through your approach in a <scratchpad>. Consider:

- How to parse tool names into server/function structure (e.g. `github_search_code` → server `github`, function `search_code`; `modelcontextprotocol_perplexity_ask` → server `modelcontextprotocol`, function `perplexity_ask`). What's the delimiter? The first `_` segment? What about tools with multi-segment server names?
- How to derive TypeScript interfaces from JSON Schema `inputSchema`. Handle: required vs optional fields, nested objects, arrays, enums, `$ref` (if any).
- How `mcp.ts` should determine local vs remote routing. The tool name prefix alone may not be enough — use the codegen output or a routing map.
- The Deno permission model: exactly which `--allow-*` flags, and how to prevent agents from loosening permissions.
- How `codegen.ts` should be resilient: what if the MCP server is slow to start? Retry with backoff. What if a tool has no `inputSchema`? Emit `args: Record<string, unknown>`.
- Backward compatibility: agents with only `remoteTools` filtering (legacy) should still work. The codegen just generates fewer files.
- The skill must work for both Claude Code (which has native `ToolSearch`) and CLI runtimes (which only have `cto-tools` on PATH). Use conditional language: "If you have ToolSearch, use it. Otherwise, use `cto-tools mcp list`."

<scratchpad>
[Your reasoning here]
</scratchpad>

Begin implementing PR 3. Write production-quality TypeScript and Bash. Validate each component.

---

## Appendix: Default Variable Values

Use these values when injecting variables. For `{$ESCALATION_RS}`, `{$HTTP_SERVER_RS}`, `{$BRIDGE_STATE_DEFINITION}`, `{$CODERUN_CRD_SPEC}`, and `{$EXISTING_CLI_TEMPLATE}`, paste the actual file contents from the repository.

### `{$TOOL_CATALOG_SUMMARY}`
```
150+ tools across 23 categories, served via MCP JSON-RPC at /mcp endpoint:

| Category      | Pattern                | Count | Purpose                              |
|--------------|------------------------|-------|--------------------------------------|
| linear        | linear_*               | 187   | Issues, projects, cycles, teams      |
| grafana       | grafana_*              | 56    | Dashboards, alerts, Loki/Prometheus  |
| github        | github_*               | 26    | PRs, issues, code, branches         |
| playwright    | playwright_*           | 22    | Browser automation                   |
| argocd        | argocd_*               | 14    | GitOps sync, rollback, resources     |
| octocode      | octocode_*             | 6     | GitHub code search, repo structure   |
| openmemory    | openmemory_*           | 6     | Persistent cross-session memory      |
| loki          | loki_*                 | 7     | Log search, patterns, correlations   |
| tavily        | tavily_*               | 5     | Web search, crawl, extract           |
| perplexity    | modelcontextprotocol_* | 4     | AI search, reasoning                 |
| exa           | exa_*                  | 2     | Web search, code context             |
| terraform     | terraform_*            | 9     | Provider/module lookup               |
| solana        | solana_*               | 3     | Anchor framework, docs               |
| context7      | context7_*             | 2     | Library documentation                |
| graphql       | graphql_*              | 2     | Schema introspection, queries        |
| better_auth   | better_auth_*          | 4     | Auth docs search                     |
| pg_aiguide    | pg_aiguide_*           | 2     | Postgres/TimescaleDB docs            |
| ai_elements   | ai_elements_*          | 2     | AI UI components                     |

Tool naming convention: {server_prefix}_{tool_name}
Examples: github_search_code, linear_create_issue, grafana_query_prometheus
Server prefix extraction: split on first underscore for single-word servers;
  for multi-word servers (modelcontextprotocol), use the MCP server name from tools/list response.
```

### `{$MODEL_PROVIDERS}`
```json
{
  "cliModels": {
    "claude": "claude-opus-4-6-20260205",
    "codex": "gpt-5.2-codex",
    "cursor": "opus-4.6",
    "gemini": "gemini-3.1-pro-preview",
    "factory": "gpt-5.2",
    "opencode": "gpt-5.2",
    "dexter": "claude-opus-4-6-20260205"
  },
  "providers": {
    "Anthropic": { "enabled": true, "baseUrl": "https://api.anthropic.com" },
    "OpenAI": { "enabled": true, "baseUrl": "https://api.openai.com/v1" },
    "Fireworks": { "enabled": true, "baseUrl": "https://api.fireworks.ai/inference" },
    "ZhipuAI": { "enabled": true },
    "Cursor": { "enabled": true }
  },
  "agentRuntimes": [
    {
      "cli": "claude",
      "capabilities": ["native_deferred_tools", "tool_search", "mcp_json", "bash", "code_execution"],
      "notes": "Full MCP support via .mcp.json with deferred: true. ToolSearch for on-demand schema loading. Can also use cto-tools CLI for code execution workflows."
    },
    {
      "cli": "codex",
      "capabilities": ["bash", "code_execution", "filesystem"],
      "notes": "CLI-only runtime. Discovers tools via cto-tools CLI or /.cto-tools/ filesystem exploration."
    },
    {
      "cli": "opencode",
      "capabilities": ["bash", "code_execution", "filesystem"],
      "notes": "Same capability profile as codex."
    },
    {
      "cli": "cursor",
      "capabilities": ["native_tools", "bash", "code_execution"],
      "notes": "Has native tool support but also supports CLI fallback via cto-tools."
    },
    {
      "cli": "gemini",
      "capabilities": ["bash", "code_execution"],
      "notes": "CLI-based tool discovery via cto-tools."
    },
    {
      "cli": "factory",
      "capabilities": ["bash", "code_execution"],
      "notes": "CLI-based tool discovery via ZhipuAI runtime."
    }
  ]
}
```

### `{$ESCALATION_POLICY_JSON}`
```json
{
  "mode": "allowlist",
  "allow": [
    "github_*",
    "linear_*",
    "grafana_*",
    "octocode_*",
    "openmemory_*",
    "firecrawl_*",
    "context7_*",
    "tavily_*",
    "exa_*",
    "loki_*",
    "playwright_*"
  ],
  "deny": [
    "terraform_destroy*",
    "argocd_delete*"
  ]
}
```

### `{$ESCALATION_RS}` (key types)
```rust
pub enum EscalationMode { Auto, Allowlist, Review }

pub struct EscalationPolicy {
    pub mode: EscalationMode,
    pub allow: Vec<String>,    // glob patterns
    pub deny: Vec<String>,     // glob patterns, deny always beats allow
}

pub enum EscalationDecision { Grant, Deny { reason: String } }

pub struct EscalationRecord {
    pub tool_name: String,
    pub reason: String,
    pub decision: String,       // "grant" or "deny"
    pub policy_reason: Option<String>,
    pub at: String,             // RFC 3339 timestamp
}

pub struct SessionState {
    pub prewarm: HashSet<String>,    // tools from X-Agent-Prewarm header
    pub granted: HashSet<String>,    // tools granted via escalation
    pub escalations: Vec<EscalationRecord>,  // audit log
}

// Pure function — no I/O
pub fn evaluate(
    policy: &EscalationPolicy,
    session: &SessionState,
    tool: &str,
    in_catalog: bool
) -> EscalationDecision
```

### `{$BRIDGE_STATE_DEFINITION}`
```rust
pub struct BridgeState {
    pub system_config_manager: Arc<RwLock<ConfigManager>>,
    pub available_tools: Arc<RwLock<HashMap<String, Tool>>>,
    pub connection_pool: Arc<ServerConnectionPool>,
    pub current_working_dir: Arc<RwLock<Option<PathBuf>>>,
    pub http_sessions: Arc<RwLock<HashMap<String, String>>>,
    pub health_monitor: Arc<Mutex<HealthMonitor>>,
    pub session_states: Arc<RwLock<SessionMap>>,            // X-Agent-Id → SessionState
    pub default_escalation_policy: Arc<EscalationPolicy>,   // from TOOLS_ESCALATION_POLICY env
}

// HTTP Routes:
// POST /mcp         — Main MCP JSON-RPC endpoint (tools/list, tools/call, initialize)
// GET  /client-config — Client configuration
// GET  /health       — Liveness
// GET  /health/servers — Per-server health
// GET  /ready        — Readiness (waits for tool discovery)
// GET  /metrics      — Prometheus metrics
```

### `{$CODERUN_CRD_SPEC}` (key fields)
```rust
pub struct CodeRunSpec {
    pub cli: CLIType,                              // claude, codex, cursor, gemini, factory, opencode
    pub provider: Option<Provider>,                // anthropic, openai, fireworks, etc.
    pub model: Option<String>,                     // model identifier
    pub github_app: String,                        // "5DLabs-Rex", "5DLabs-Bolt", etc.
    pub prompt: String,                            // task instructions
    pub remote_tools: Option<String>,              // comma-separated tool filter (LEGACY — PR 2 synthesizes EscalationPolicy from this)
    pub local_tools: Option<String>,               // local MCP servers to spawn (e.g. "filesystem,memory")
    pub escalation_policy: Option<EscalationPolicy>, // PR 2 addition — overrides default
    pub implementation_agent: Option<String>,       // e.g. "rex", "blaze"
    pub quality: bool,
    pub security: bool,
    pub testing: bool,
    pub deployment: bool,
    pub acp: Option<Vec<ACPEntry>>,                // AI-CLI-Provider entries
    pub openclaw: Option<OpenClawEntry>,            // OpenClaw provider config
    pub watcher_config: Option<WatcherConfig>,
}
```

### `{$TOOLS_SHARED_PARTIALS}`
```
templates/_shared/partials/
├── tools-config.sh.hbs          ← existing: copies client-config, merges project config, displays tool summary
├── git-setup.sh.hbs             ← git config
├── github-auth.sh.hbs           ← GitHub App auth
├── scm-auth.sh.hbs              ← SCM provider auth
├── skills-setup.sh.hbs          ← installs agent skills
├── mcp-check.sh.hbs             ← verifies MCP connectivity
├── node-env.sh.hbs              ← Node.js environment
├── rust-env.sh.hbs              ← Rust environment
├── go-env.sh.hbs                ← Go environment
├── coordinator.md.hbs           ← multi-agent coordinator prompt
├── subagent-dispatch.md.hbs     ← subagent routing
├── watcher-coordination.md.hbs  ← watcher agent prompts
└── cto-tools-setup.sh.hbs      ← NEW (you create this): SDK + CLI + codegen setup
```

### `{$EXISTING_CLI_TEMPLATE}`
Source the contents of `templates/clis/openclaw.sh.hbs` — this is the Handlebars template that generates the agent pod's entrypoint script. Your `cto-tools-setup` partial should be invoked after the existing tools-config setup within this template.
