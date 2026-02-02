# PRD: LSP-Based Refactoring MCP Server

## Summary

Build an MCP server that provides semantic code refactoring capabilities by integrating with language server protocols (rust-analyzer, ts-language-server, gopls, pyright). This enables agents to perform safe, semantics-aware code modifications beyond simple text replacement.

## Problem Statement

Current agent code generation suffers from:
1. **Brittle refactoring** - Regex/string replacement breaks on edge cases
2. **No semantic awareness** - Agents don't understand AST structure
3. **Manual intervention required** - Humans must fix refactoring bugs
4. **Inconsistent code style** - Each agent generates different conventions

Agents can generate code but struggle to maintain/evolve existing codebases safely.

## Proposed Solution

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    LSP Refactoring MCP Server               │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │  Language   │  │   Refactor  │  │   Code      │         │
│  │  Detection  │  │   Engine    │  │   Quality   │         │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘         │
│         │                │                │                  │
│         └────────────────┼────────────────┘                  │
│                          ▼                                   │
│                 ┌─────────────────┐                          │
│                 │   LSP Server    │                          │
│                 │   Integration   │                          │
│                 └────────┬────────┘                          │
│                          │                                   │
│         ┌────────────────┼────────────────┐                  │
│         ▼                ▼                ▼                  │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐              │
│  │ rust-    │    │ ts-      │    │ gopls/   │              │
│  │ analyzer │    │ language │    │ pyright  │              │
│  │          │    │ server   │    │          │              │
│  └──────────┘    └──────────┘    └──────────┘              │
└─────────────────────────────────────────────────────────────┘
```

### MCP Tools Provided

| Tool | Description |
|------|-------------|
| `detect_language` | Identify language and available LSP for a codebase |
| `list_refactorings` | Get available refactoring operations at a position |
| `apply_refactoring` | Execute a refactoring operation |
| `rename_symbol` | Safe rename across entire codebase |
| `extract_function` | Pull code into a new function |
| `move_symbol` | Move function/class between modules |
| `inline_symbol` | Inline a function/variable |
| `find_usages` | Find all references to a symbol |

### Integration Points

- **Rex** - Use for architecture refactoring during implementation
- **Cleo** - Quality checks via semantic analysis
- **Nova** - Frontend refactoring (TypeScript/JavaScript)
- **Rex** - Rust backend refactoring

## Technical Implementation

### LSP Server Management

```rust
// Pseudocode for LSP server pool
struct LspServerPool {
    servers: HashMap<Language, LspServerInstance>,
}

impl LspServerPool {
    fn get_server(&mut self, language: &str) -> Result<&mut LspServer> {
        // Spawn LSP server if not exists
        // Cache and reuse for performance
    }
}
```

### Refactoring Operations

1. **Rename Symbol** - Cross-file rename with proper AST updates
2. **Extract Function** - Create new function, update call sites
3. **Move Symbol** - Relocate between files with import updates
4. **Inline Function/Variable** - Replace calls with body/content
5. **Change Signature** - Add/remove/reorder parameters

### Safety Guarantees

- All operations use LSP protocol (battle-tested)
- Preview mode available before applying changes
- Atomic operations - roll back on failure
- Preserve comments and formatting where possible

## Success Criteria

- [ ] MCP server exposes 8+ refactoring tools
- [ ] Supports Rust (rust-analyzer), TypeScript (ts-language-server), Go (gopls), Python (pyright)
- [ ] Refactoring operations complete in <5s for 1000-file codebase
- [ ] Zero data loss incidents in production use
- [ ] All agents (Rex, Blaze, Cleo) can invoke via MCP

## Effort Estimate

**Medium (3-4 weeks)**
- Week 1: LSP server pool, language detection
- Week 2-3: Core refactoring operations
- Week 4: Testing, error handling, docs

## Open Questions

- Should we support VS Code refactoring extensions?
- How to handle LSP servers that aren't installed on the system?
- What's the fallback if LSP server crashes mid-operation?

## References

- rust-analyzer: https://rust-analyzer.github.io/
- LSP Specification: https://microsoft.github.io/language-server-protocol/
