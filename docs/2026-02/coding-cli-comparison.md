# Coding CLI Sub-Agent Capabilities Comparison

Research completed: 2026-01-31

## Executive Summary

| CLI | Sub-Agent Support | Status | How to Delegate |
|-----|-------------------|--------|-----------------|
| **Claude Code** | Native | GA | `--agents`, `.claude/agents/` files, SDK |
| **Codex CLI** | Experimental | `child_agents_md` flag | Unknown mechanism |
| **Pi Coding Agent** | Via Extensions | User-built | Extensions API, spawn instances |
| **OpenCode/Crush** | None | N/A | MCP tools only |

## Detailed Analysis

### Claude Code (Anthropic)

**Version:** 2.1.25+

**Sub-Agent Status:** Native support, production-ready

**Mechanisms:**
1. **CLI Flags:**
   - `--agent <agent>` - Select agent for session
   - `--agents <json>` - Define custom agents inline

2. **Agent Definition Files:**
   - Location: `.claude/agents/` (project) or `~/.claude/agents/` (user)
   - Format: Markdown with YAML frontmatter
   - Fields: `name`, `description`, `tools`, `model`, `permissionMode`, `skills`

3. **SDK (Programmatic):**
   - TeammateTool for spawning sub-agents
   - Task queues for parallel work
   - Background execution with inbox messaging

**Example Agent Definition:**
```markdown
---
name: implementer
description: Code implementation specialist
tools: Read, Write, Edit, Bash, Grep, Glob
model: sonnet
permissionMode: acceptEdits
---

Your implementation instructions here...
```

**Key Features:**
- Agents can spawn other agents
- Model selection per agent (opus, sonnet, haiku)
- Permission modes control autonomy
- Tool restrictions per agent
- Skill preloading

---

### Codex CLI (OpenAI)

**Version:** Latest (2026)

**Sub-Agent Status:** Experimental (`child_agents_md` feature flag)

**Mechanisms:**
1. **Feature Flag:**
   - `child_agents_md` - Under development
   - `collab` - Under development
   - `collaboration_modes` - Under development

2. **AGENTS.md:**
   - Similar to Claude Code's approach
   - Details not publicly documented yet

**Enabling:**
```bash
codex --enable child_agents_md
# or in ~/.codex/config.toml
[features]
child_agents_md = true
```

**Notes:**
- Feature is "under development"
- No public documentation yet
- Likely similar to Claude Code's `.claude/agents/` pattern

---

### Pi Coding Agent (@mariozechner)

**Version:** Latest

**Sub-Agent Status:** Explicitly NOT built-in; available via extensions

**Philosophy (from docs):**
> "Pi ships with powerful defaults but skips features like sub agents and plan mode."
> "No sub-agents. There's many ways to do this. Spawn pi instances via tmux, or build your own with extensions, or install a package that does it your way."

**Mechanisms:**
1. **Extensions API:**
   - `pi.registerTool()` - Add custom tools
   - Full programmatic control
   - Can spawn child Pi instances

2. **Manual Spawning:**
   - Use tmux to run multiple Pi instances
   - Each instance is independent

3. **Pi Packages:**
   - Third-party packages can add sub-agent support
   - Check npm/Discord for community packages

**Example Extension Stub:**
```typescript
export default function (pi: ExtensionAPI) {
  pi.registerTool({
    name: "spawn_agent",
    description: "Spawn a sub-agent for parallel work",
    handler: async (params) => {
      // Spawn Pi instance in background
      // ...
    }
  });
}
```

---

### OpenCode / Crush (Charm)

**Version:** Crush (successor to OpenCode)

**Sub-Agent Status:** None

**Focus:** Single-agent TUI with rich context via:
- MCP servers (stdio, http, sse)
- LSP integration
- Multi-model switching mid-session

**Workaround:**
- Use MCP to add tool capabilities
- No native way to spawn parallel agents

---

## Abstraction Layer Design

Given the differences, our `intake-agent` should use an **abstraction layer** that:

1. **Detects available CLI** and its capabilities
2. **Falls back gracefully** when sub-agents aren't available
3. **Uses native mechanisms** where supported

### Proposed Interface

```typescript
interface SubAgentProvider {
  // Check if provider supports sub-agents
  supportsSubAgents(): boolean;
  
  // Spawn a sub-agent with configuration
  spawn(config: SubAgentConfig): Promise<SubAgentHandle>;
  
  // Send message to sub-agent
  sendMessage(handle: SubAgentHandle, message: string): Promise<void>;
  
  // Get sub-agent output
  getOutput(handle: SubAgentHandle): Promise<string>;
  
  // Wait for completion
  await(handle: SubAgentHandle): Promise<SubAgentResult>;
  
  // Kill sub-agent
  kill(handle: SubAgentHandle): Promise<void>;
}

interface SubAgentConfig {
  name: string;
  type: 'implementer' | 'reviewer' | 'tester' | 'documenter' | 'researcher' | 'debugger';
  prompt: string;
  workdir: string;
  model?: string;
  tools?: string[];
  timeout?: number;
}
```

### Implementation Strategy

| CLI | Implementation |
|-----|----------------|
| **Claude Code** | Use native `--agents` or SDK |
| **Codex CLI** | Enable `child_agents_md` flag, use AGENTS.md |
| **Pi** | Spawn separate instances via extensions or tmux |
| **Crush** | Fall back to sequential execution |

### Fallback: Sequential Execution

For CLIs without sub-agent support:
1. Execute subtasks sequentially
2. Pass output from one to the next
3. No parallelization benefit but still works

---

## Recommendations

1. **Primary Target:** Claude Code - most mature sub-agent support
2. **Secondary Target:** Codex CLI - watch for `child_agents_md` graduation
3. **Pi Support:** Build extension or use tmux spawning
4. **Crush:** Sequential fallback only

For the `intake-agent` MVP, focus on:
1. **Claude Code native integration** (priority)
2. **Process-based spawning** as universal fallback
3. **Codex experimental support** (optional)
