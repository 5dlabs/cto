---
name: repomix
description: Pack codebases into context-optimized formats for analysis and understanding.
agents: [morgan, cleo, atlas]
triggers: [codebase, repository, pack, analyze codebase, understand repo]
---

# Repomix (Codebase Packaging)

Use Repomix to pack entire codebases into a single, context-optimized format for analysis.

## Tools

| Tool | Purpose |
|------|---------|
| `repomix_pack_codebase` | Pack local codebase |
| `repomix_pack_remote_repository` | Pack a GitHub repository |
| `repomix_grep_repomix_output` | Search within packed output |
| `repomix_read_repomix_output` | Read sections of packed output |

## Packing a Remote Repository

```
repomix_pack_remote_repository({
  remote: "https://github.com/5dlabs/my-project",
  branch: "develop"
})
```

Returns a packed representation of the entire codebase.

## Searching Packed Output

```
# After packing, search for specific patterns
repomix_grep_repomix_output({
  pattern: "authentication",
  output_id: "abc123"
})
```

## Reading Specific Sections

```
repomix_read_repomix_output({
  output_id: "abc123",
  start_line: 100,
  end_line: 200
})
```

## Use Cases

| Task | Approach |
|------|----------|
| **PRD Analysis** | Pack repo to understand existing architecture |
| **Code Review** | Pack to see full context of changes |
| **Task Planning** | Pack to understand dependencies |
| **Integration** | Pack multiple repos for merge planning |

## Best Practices

1. **Pack before planning** - Understand the codebase first
2. **Use grep for navigation** - Don't read everything
3. **Focus on relevant sections** - Use start/end lines
4. **Cache outputs** - Reuse packed output for multiple queries
