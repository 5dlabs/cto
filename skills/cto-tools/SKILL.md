---
name: cto-tools
description: Discover and invoke 150+ MCP tools via code execution or CLI. Write TypeScript scripts for multi-step workflows; use CLI for single operations.
triggers:
  - "need a tool"
  - "don't have a tool for"
  - "discover tools"
  - "search tools"
  - "mcp"
  - "cto-tools"
  - "tool not available"
  - "escalate"
  - "request capability"
---

# cto-tools — MCP Tool Discovery, Invocation & Composition

## 1. When to Use This Skill

| Situation | Action |
|-----------|--------|
| You need a capability not in your eager toolset | **Discover** → `cto-tools mcp list` |
| You're about to say "I don't have a tool for X" | **Stop.** You probably do — 150+ tools available. Search first. |
| You need to chain multiple tool calls where intermediate results should NOT enter your context (e.g., search 10K results → filter to 5) | **Code execution** → `cto-tools exec` |
| You need access to a tool not pre-loaded | **Escalate** → `cto-tools mcp escalate` |

---

## 2. Quick Reference

| Task | Command |
|------|---------|
| List all tools | `cto-tools mcp list` |
| List tools by category | `cto-tools mcp list --category github` |
| Describe a tool | `cto-tools mcp describe github_search_code` |
| Call a tool | `cto-tools mcp call github_search_code --json '{"q":"test","repo":"5dlabs/cto"}'` |
| Request new capability | `cto-tools mcp escalate terraform_plan --reason "Need infra preview for PR #42"` |
| Run a TypeScript script | `cto-tools exec /tmp/my-script.ts` |
| Run inline TypeScript | `cto-tools exec -e 'import {...} from "/.cto-tools/servers/github/search_code.ts"; ...'` |

---

## 3. Discovery Flow

**Step 1 — Browse the catalog:**

```bash
cto-tools mcp list
```

This prints all 150+ tools grouped by category. Current categories include:

| Category | Tool Count | Examples |
|----------|-----------|----------|
| github | 26 | `search_code`, `get_file_contents`, `create_pull_request` |
| linear | 187 | `create_issue`, `search_issues`, `update_issue` |
| grafana | 56 | `query_prometheus`, `list_dashboards`, `get_annotations` |
| playwright | 22 | `navigate`, `screenshot`, `click` |
| argocd | 14 | `get_application`, `sync_application`, `get_logs` |
| _13+ more_ | — | loki, k8s, slack, notion, sentry, … |

**Step 2 — Narrow by category** (when you know the domain):

```bash
cto-tools mcp list --category github
```

**Step 3 — Inspect a candidate:**

```bash
cto-tools mcp describe github_search_code
```

Returns the full JSON Schema for the tool's inputs and a description of its output.

**Step 4 — For Claude Code agents:**
You can also use `ToolSearch` for on-demand schema loading — it queries the same catalog under the hood.

> **Tip:** Cache tool names mentally after first discovery. Don't `list` on every turn.

---

## 4. Simple Invocation (Single Tool Call)

Use `cto-tools mcp call` when you need a single tool call with a straightforward result.

### Example 1 — Search code

```bash
cto-tools mcp call github_search_code \
  --json '{"q": "EscalationPolicy lang:rust", "repo": "5dlabs/cto"}'
```

### Example 2 — Create a Linear issue

```bash
cto-tools mcp call linear_create_issue \
  --json '{
    "teamId": "CTOPA",
    "title": "Fix escalation timeout",
    "description": "The escalation handler times out after 30s",
    "priority": 2
  }'
```

### Example 3 — Query Grafana

```bash
cto-tools mcp call grafana_query_prometheus \
  --json '{
    "query": "rate(http_requests_total{namespace=\"cto\"}[5m])",
    "start": "now-1h",
    "end": "now"
  }'
```

### Example 4 — Check ArgoCD sync status

```bash
cto-tools mcp call argocd_get_application \
  --json '{"name": "cto"}'
```

All calls return JSON to stdout. Exit code 0 means success; non-zero means error (see §7).

---

## 5. Code Execution Pattern (Multi-Step Workflows)

### When to use which

| Scenario | Use |
|----------|-----|
| Single tool call, simple result | `cto-tools mcp call` (CLI) |
| Multiple tool calls, filtering/aggregation | `cto-tools exec` (TypeScript) |
| Loops, conditional logic | `cto-tools exec` (TypeScript) |
| Intermediate data that shouldn't enter your context | `cto-tools exec` (TypeScript) |

### Example — Search and filter

```typescript
// /tmp/find-recent-prs.ts
import { search_code } from "/.cto-tools/servers/github/search_code.ts";
import { get_file_contents } from "/.cto-tools/servers/github/get_file_contents.ts";

// Search for recent escalation changes
const hits = await search_code({ q: "EscalationPolicy", repo: "5dlabs/cto" });

// Filter to only Rust files, get first 3
const rustFiles = hits.items
  .filter(h => h.path.endsWith(".rs"))
  .slice(0, 3);

// Fetch content for each
for (const file of rustFiles) {
  const content = await get_file_contents({
    owner: "5dlabs",
    repo: "cto",
    path: file.path,
  });
  console.log(`--- ${file.path} ---`);
  console.log(content.content.slice(0, 500)); // Only first 500 chars
}
```

Run it:

```bash
cto-tools exec /tmp/find-recent-prs.ts
```

### Example — Cross-tool aggregation (metrics + logs → issue)

```typescript
// /tmp/incident-summary.ts
import { query_prometheus } from "/.cto-tools/servers/grafana/query_prometheus.ts";
import { query_loki } from "/.cto-tools/servers/loki/query_loki.ts";
import { create_issue } from "/.cto-tools/servers/linear/create_issue.ts";

// Gather metrics and logs
const errorRate = await query_prometheus({
  query: 'rate(http_errors_total{namespace="cto"}[5m])',
});
const recentErrors = await query_loki({
  query: '{namespace="cto"} |= "ERROR"',
  limit: 10,
});

// Summarize — intermediate data stays in sandbox, only summary enters context
const summary = [
  `Error rate: ${errorRate.data?.result?.[0]?.value?.[1] ?? "N/A"}/s`,
  `Recent errors: ${recentErrors.length}`,
].join("\n");

console.log(summary);
```

### Example — Inline execution (no temp file)

```bash
cto-tools exec -e '
import { search_code } from "/.cto-tools/servers/github/search_code.ts";
const hits = await search_code({ q: "todo fixme", repo: "5dlabs/cto" });
console.log(`Found ${hits.total_count} TODOs/FIXMEs`);
'
```

> **Key principle:** Use `console.log()` to emit only the compact summary you need. Everything else stays inside the script sandbox and never bloats your context window.

---

## 6. Escalation Flow

When you need a tool that's **not** in your pre-loaded eager set:

```bash
cto-tools mcp escalate terraform_plan \
  --reason "PR #42 modifies infra/charts — need to preview Terraform changes before merge"
```

### Escalation outcomes

| Result | Meaning | What to do |
|--------|---------|------------|
| **Granted** | Tool is now available for the rest of your session | Proceed with `cto-tools mcp call <tool>` |
| **Denied — matches a deny pattern** | Blocked by escalation policy | Try a different approach; this tool is intentionally restricted |
| **Denied — not in catalog** | Tool doesn't exist | Run `cto-tools mcp list` to find the correct name |
| **Denied — review mode** | Requires human approval | Wait or ask the operator to approve |

### Escalation guidelines

- **Write a clear, specific reason** — it's logged for audit and shown to human reviewers.
- **Don't escalate for tools already in your eager set** — check with `cto-tools mcp list` first.
- **Don't spam escalations** — if denied, read the reason before retrying.

---

## 7. Error Handling

| Exit Code | Meaning | Recovery |
|-----------|---------|----------|
| `0` | Success | — |
| `1` | Tool returned an error | Check args format. Run `cto-tools mcp describe <tool>` to verify the input schema. |
| `2` | Policy denied | Tool is blocked by escalation policy. Try a different approach. |
| `3` | Not in catalog | Tool doesn't exist. Run `cto-tools mcp list` to find the correct name. |
| `4` | Server unreachable | MCP server may be starting up. Wait 5–10s and retry (up to 3 times). |

### Common mistakes

```bash
# ❌ Wrong: passing args as flags
cto-tools mcp call github_search_code --q "test"

# ✅ Right: pass args as JSON
cto-tools mcp call github_search_code --json '{"q": "test"}'
```

```bash
# ❌ Wrong: unescaped quotes in JSON
cto-tools mcp call grafana_query_prometheus --json '{"query": "rate(http_requests_total{namespace="cto"}[5m])"}'

# ✅ Right: escaped inner quotes
cto-tools mcp call grafana_query_prometheus --json '{"query": "rate(http_requests_total{namespace=\"cto\"}[5m])"}'
```

---

## 8. Anti-Patterns

| | Pattern |
|---|---------|
| ❌ | Don't `list` on every turn — cache tool names mentally after first discovery |
| ❌ | Don't `describe` all 150+ tools — only describe candidates you'll actually use |
| ❌ | Don't escalate for tools already in your eager set |
| ❌ | Don't write TypeScript for single tool calls — use `cto-tools mcp call` instead |
| ❌ | Don't pass huge intermediate results back to your context — filter in code first |
| ✅ | Do write TypeScript when you need loops, filtering, or multi-tool composition |
| ✅ | Do use `console.log()` for compact summaries that enter your context |
| ✅ | Do check exit codes and handle errors before retrying blindly |
| ✅ | Do prefer `cto-tools exec` over raw shell pipelines for multi-step MCP workflows |

---

## 9. Runtime Compatibility

| Runtime | Discovery | Invocation | Code Execution |
|---------|-----------|------------|----------------|
| **Claude Code** | `ToolSearch` (native) or `cto-tools mcp list` | Native tool calls or `cto-tools mcp call` | `cto-tools exec` |
| **Codex** | `cto-tools mcp list` | `cto-tools mcp call` | `cto-tools exec` |
| **OpenCode** | `cto-tools mcp list` | `cto-tools mcp call` | `cto-tools exec` |
| **Factory** | `cto-tools mcp list` | `cto-tools mcp call` | `cto-tools exec` |
| **Gemini** | `cto-tools mcp list` | `cto-tools mcp call` | `cto-tools exec` |
| **Cursor** | `cto-tools mcp list` | `cto-tools mcp call` | `cto-tools exec` |

The CLI path (`cto-tools mcp *` and `cto-tools exec`) works identically across all runtimes. Claude Code agents can additionally use native `ToolSearch` for on-demand schema loading but all other commands remain the same.
