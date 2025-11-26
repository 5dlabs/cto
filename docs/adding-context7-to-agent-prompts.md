# Adding Context7 Instructions to Agent System Prompts

## Overview

Context7 documentation tools should be included in agent system prompts so agents know how to use them effectively. This guide explains how to add Context7 instructions to agent configurations.

## Where Agent System Prompts Are Stored

Agent system prompts are defined in:
```
infra/charts/controller/values.yaml
```

Under each agent's configuration, there's a `systemPrompt` field that contains the agent's instructions.

## How to Add Context7 Instructions

### Option 1: Include Full Instructions (Recommended for Complex Agents)

For agents like Rex, Blaze, or Morgan who frequently need library documentation:

```yaml
agents:
  rex:
    systemPrompt: |
      You are Rex, a Senior Backend Architect...
      
      [existing prompt content]
      
      ## Context7 Documentation Tools
      
      You have access to Context7 for real-time library documentation.
      
      **Usage Workflow:**
      1. Resolve library: `resolve_library_id({ libraryName: "tokio" })`
      2. Get docs: `get_library_docs({ context7CompatibleLibraryID: "/websites/rs_tokio_tokio", topic: "async runtime" })`
      
      **Best Practices:**
      - Always resolve the library name first to get the correct ID
      - Choose libraries with high benchmark scores and reputation
      - Be specific in your topic queries
      - Use Context7 before implementing unfamiliar features
      
      **Example for Rust:**
      ```
      resolve_library_id({ libraryName: "tokio" })
      get_library_docs({
        context7CompatibleLibraryID: "/websites/rs_tokio_tokio",
        topic: "async runtime setup and basic usage"
      })
      ```
```

### Option 2: Brief Reference (For Specialized Agents)

For agents like Cleo or Tess who have specific, focused roles:

```yaml
agents:
  cleo:
    systemPrompt: |
      You are Cleo, a code quality specialist...
      
      [existing prompt content]
      
      **Context7 Tools:** Use `resolve_library_id` then `get_library_docs` for current library documentation.
```

## Template Snippet

A reusable snippet is available at:
```
infra/charts/controller/agent-templates/context7-instructions-snippet.md
```

You can copy this content into agent system prompts as needed.

## Agent-Specific Recommendations

### Rex (Implementation Agent)
**Priority:** HIGH  
**Why:** Frequently implements features with various Rust libraries  
**Suggested Libraries:** tokio, serde, axum, sqlx, anyhow

Add full Context7 instructions with Rust-specific examples.

### Blaze (Frontend Agent)
**Priority:** HIGH  
**Why:** Works with React, Next.js, and UI libraries  
**Suggested Libraries:** react, next.js, shadcn/ui, react-query

Add full Context7 instructions with TypeScript/React examples.

### Morgan (PM/Docs Agent)
**Priority:** MEDIUM  
**Why:** May need to reference libraries when creating documentation  
**Suggested Libraries:** Various, depending on project

Add brief Context7 reference.

### Cleo (Code Quality Agent)
**Priority:** LOW  
**Why:** Focuses on linting and formatting, less on implementation  
**Suggested Libraries:** clippy, rustfmt

Add brief Context7 reference for linting tool documentation.

### Tess (QA/Testing Agent)
**Priority:** MEDIUM  
**Why:** Needs testing framework documentation  
**Suggested Libraries:** pytest, jest, tokio-test

Add Context7 instructions with testing framework examples.

### Cipher (Security Agent)
**Priority:** MEDIUM  
**Why:** Needs security library documentation  
**Suggested Libraries:** jsonwebtoken, bcrypt, security-related crates

Add Context7 instructions with security library examples.

## Configuration in cto-config.json

Ensure agents have both Context7 tools in their remote tools list:

```json
{
  "agents": {
    "rex": {
      "tools": {
        "remote": [
          "brave_search_brave_web_search",
          "context7_resolve_library_id",
          "context7_get_library_docs"
        ]
      }
    }
  }
}
```

## Testing Agent Understanding

After adding Context7 instructions, test that agents use them correctly:

1. **Ask agent to implement a feature with an unfamiliar library**
2. **Verify agent:**
   - Calls `resolve_library_id` first
   - Chooses appropriate library from results
   - Calls `get_library_docs` with specific topic
   - Uses documentation in implementation

Example test:
```
"Implement JWT authentication in Rust using the jsonwebtoken library"
```

Expected behavior:
1. Agent calls `resolve_library_id({ libraryName: "jsonwebtoken" })`
2. Agent reviews options and selects best match
3. Agent calls `get_library_docs` with specific topic
4. Agent implements based on current documentation

## Deployment

After updating agent system prompts in `values.yaml`:

1. **Commit changes:**
   ```bash
   git add infra/charts/controller/values.yaml
   git commit -m "feat: Add Context7 instructions to agent system prompts"
   ```

2. **Push and sync:**
   ```bash
   git push origin main
   ```

3. **ArgoCD will automatically sync** the changes to the cluster

4. **Restart affected agents** (optional, for immediate effect):
   ```bash
   kubectl rollout restart deployment -n cto -l app.kubernetes.io/component=agent
   ```

## Documentation References

- [Context7 Setup Guide](context7-setup-complete.md)
- [Context7 Agent Usage Guide](context7-agent-usage-guide.md)
- [Context7 Prompt Instructions](context7-prompt-instructions.md)
- [Cursor CLI Models](cursor-cli-available-models.md)

## Summary

1. ✅ Add Context7 tools to `cto-config.json` remote tools
2. ✅ Add Context7 instructions to agent system prompts in `values.yaml`
3. ✅ Prioritize full instructions for implementation agents (Rex, Blaze)
4. ✅ Test agents use Context7 correctly
5. ✅ Deploy via GitOps (ArgoCD)

---

**Status:** Ready for Implementation  
**Next Steps:** Update `values.yaml` with Context7 instructions for priority agents

