# AGENTS.md - Intake Agent Workspace

You are an AI agent working on the **intake-agent** for the CTO platform.

## First Run

Read `CONTEXT.md` - it contains extracted knowledge from a previous agent session.

## Your Mission

You build and improve the intake-agent - the TypeScript binary that:
- Parses PRDs into structured tasks
- Expands tasks into subtasks
- Analyzes complexity
- Performs research via MCP servers

## Workspace Structure

```
/Users/jonathonfritz/clawd-intake/          # Agent workspace (you are here)
├── CONTEXT.md                               # Previous session context
├── AGENTS.md                                # This file
├── SOUL.md                                  # Your personality
├── TOOLS.md                                 # Local tool notes
└── cto/                                     # Symlink to CTO worktree

/Users/jonathonfritz/.cursor/worktrees/cto/intake-agent/  # CTO worktree
└── tools/intake-agent/                      # The actual code
    ├── src/                                 # TypeScript source
    ├── dist/                                # Compiled binary
    ├── tests/                               # Test data
    └── docs/                                # Documentation
```

## Memory

- **Daily notes:** `memory/YYYY-MM-DD.md`
- Capture decisions, progress, blockers
- Update CONTEXT.md with significant findings

## Git Workflow

- **Branch:** `feat/intake-agent`
- **Base:** `origin/main`
- **Remote:** https://github.com/5dlabs/cto.git

Commit often. Push when you have working changes.

## Safety

- Don't modify files outside `tools/intake-agent/` or `crates/intake/`
- Ask before making breaking API changes
- Test before pushing

## Quick Commands

```bash
# Navigate to code
cd /Users/jonathonfritz/.cursor/worktrees/cto/intake-agent/tools/intake-agent

# Build
bun install && bun run build

# Test
echo '{"operation":"ping"}' | ./dist/intake-agent

# Run intake
echo '{"operation":"parse_prd","model":"claude-sonnet-4-20250514","payload":{"prd_content":"..."}}' | ./dist/intake-agent
```


---

## UI Automation (Peekaboo)

When automating macOS UI:
1. Always run `peekaboo see --annotate --path /tmp/ui-state.png` first
2. Use element IDs from the annotated image (e.g., B1, T2)
3. Target by app + window when possible: `--app "App Name" --window-title "Window"`
4. Peekaboo requires Screen Recording + Accessibility permissions (already granted)
---

## Long-Term Memory (Open Memory) - MANDATORY USAGE

**You MUST use Open Memory to maintain continuity. Your context gets compacted. Memories persist.**

### Available Tools
```
openmemory_store     - Save information
openmemory_query     - Semantic search  
openmemory_list      - Recent memories
openmemory_get       - Fetch by ID
openmemory_reinforce - Boost importance
openmemory_delete    - Remove outdated
```

---

### 🟢 ON EVERY SESSION START (do this FIRST)

Before responding to ANY user message, run:
```
openmemory_query({ query: "intake current work outstanding tasks context", k: 8 })
openmemory_list({ limit: 5 })
```

Read the results. Understand what you were working on. THEN respond.

---

### 🔵 DURING WORK (store as you go)

**After completing a significant task:**
```
openmemory_store({
  content: "Completed: [what you did]. Result: [outcome]. Next: [what's remaining]",
  tags: ["intake", "project-name", "progress"]
})
```

**When you make a decision:**
```
openmemory_store({
  content: "Decision: [what]. Reason: [why]. Alternative considered: [what else]",
  tags: ["intake", "decision", "project-name"]
})
```

**When you hit a blocker:**
```
openmemory_store({
  content: "Blocker: [issue]. Tried: [what]. Need: [what's required to proceed]",
  tags: ["intake", "blocker", "project-name"]
})
```

---

### 🟡 BEFORE COMPACTION (when context is getting full)

When you notice context is high (>70%) or get a compaction warning:

```
openmemory_store({
  content: `SESSION SUMMARY [date]:
  
COMPLETED THIS SESSION:
- [task 1]
- [task 2]

STILL OUTSTANDING:
- [remaining task 1]
- [remaining task 2]

CURRENT STATE:
- [where things are at]

BLOCKERS/NEEDS:
- [what's blocking progress]

KEY CONTEXT FOR NEXT SESSION:
- [critical info to remember]`,
  tags: ["intake", "session-summary", "YYYY-MM-DD"]
})
```

Then reinforce it:
```
openmemory_reinforce({ id: "[memory-id]", boost: 0.5 })
```

---

### 🔴 AFTER COMPACTION (context was reset)

If your context seems empty or you don't remember recent work:

```
openmemory_query({ query: "intake session summary recent work", k: 5 })
openmemory_list({ limit: 10 })
```

Read everything. Rebuild context. Continue where you left off.

---

### Memory Hygiene

**Reinforce** memories you keep referencing:
```
openmemory_reinforce({ id: "[id]", boost: 0.3 })
```

**Delete** outdated memories (completed tasks, old blockers):
```
openmemory_delete({ id: "[id]" })
```

---

### Network Access

Open Memory is accessed **directly via Twingate VPN** at ClusterIP:
```
http://10.105.155.160:8080/mcp
```

**No port-forward needed!** Just ensure Twingate is connected.

If connection fails:
1. Check Twingate is connected
2. Fallback to port-forward: `kubectl -n cto port-forward svc/cto-openmemory 8765:8080`

---

### Fallback (if MCP tools unavailable)

Use exec to call directly:
```bash
node -e "
fetch('http://10.105.155.160:8080/mcp', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json', 'Accept': 'application/json, text/event-stream' },
  body: JSON.stringify({
    jsonrpc: '2.0', method: 'tools/call', id: 1,
    params: { name: 'openmemory_query', arguments: { query: 'your query here', k: 5 }}
  })
}).then(r => r.json()).then(d => console.log(JSON.stringify(d, null, 2)));
"
```
