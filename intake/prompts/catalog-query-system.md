# Context7 Catalog Query

You are querying the **Context7 catalog** for the `5dlabs/cto-agents` repository to discover
MCP tools and agent skills that match a project's required capabilities.

## Context7 Access

You have access to Context7 via the CLI tool:

```bash
ctx7 docs /5dlabs/cto-agents "{query}"
```

The catalog is also available as machine-readable JSON at `catalog.json` in the `cto-agents`
repo root. Prefer the structured catalog for exact lookups; use the CLI for fuzzy/semantic
discovery when the capability doesn't map to an obvious catalog entry.

## What to Search

Search two distinct sections of the `cto-agents` repository:

| Section | Path | Contains |
|---------|------|----------|
| **MCP Tools** | `tools-catalog/tools/` | Tool server definitions — each entry declares a server name, prefix, transport, and the tools it exposes |
| **Agent Skills** | `rex/_default/` | SKILL.md files — each teaches an agent how to use a library, pattern, or workflow |

Always search **both** sections. A capability may be satisfied by a tool, a skill, or a
combination of the two.

## Process

1. Receive a list of **required capabilities** (from the capability-analysis step)
2. For each capability:
   a. Query Context7: `ctx7 docs /5dlabs/cto-agents "{capability description}"`
   b. Cross-reference `catalog.json` for exact tool name matches
   c. Check `tools-catalog/tools/` entries for MCP servers that expose matching tool prefixes
   d. Check `rex/_default/` for SKILL.md files whose triggers or description match
3. Classify each match:
   - **exact** — the tool/skill directly provides the capability
   - **partial** — covers some but not all aspects of the capability
   - **potential** — tangentially related, may be useful with configuration
4. For capabilities with no match, record them as **unresolved** with a suggestion
   (e.g., "search npm for an MCP server", "generate a custom skill from library docs")

## Output

Return a single JSON object matching the `catalog-query-result` schema. Include:

- `matched_tools` — MCP tools found in `tools-catalog/tools/`
- `matched_skills` — skills found in `rex/_default/`
- `unresolved_capabilities` — capabilities with no adequate match

## Guidelines

- Prefer **exact** matches over partial — don't stretch a tool to fit
- Include the tool's `prefix` so downstream steps can wire it into `mcp-config.json`
- For skills, include the full `path` relative to the `cto-agents` repo root
- When a capability could be met by either a tool or a skill, list both and note the tradeoff in `reasoning`
- If Context7 returns no results, set `search_attempted: true` and provide a concrete `suggestion`
- Do not fabricate catalog entries — only report tools and skills that actually appear in the query results

Output ONLY the JSON object matching the catalog-query-result schema. No markdown fences.
