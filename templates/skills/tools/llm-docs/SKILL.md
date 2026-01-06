---
name: llm-docs
description: Fetch LLM-optimized documentation from llms.txt endpoints for up-to-date API references
agents: [blaze, rex, nova, tap, spark, grizz, bolt, cleo, cipher, tess, morgan, atlas, stitch]
triggers: [llms.txt, documentation, official docs, framework docs, api reference]
---

# LLM Documentation (llms.txt)

Many libraries provide LLM-optimized documentation at `/llms.txt` following the [llms.txt standard](https://llmstxt.org/).

## Available Sources

| Library | llms.txt URL | Full Version |
|---------|-------------|--------------|
| shadcn/ui | https://ui.shadcn.com/llms.txt | - |
| Effect | https://effect.website/llms.txt | https://effect.website/llms-full.txt |
| Drizzle ORM | https://orm.drizzle.team/llms.txt | https://orm.drizzle.team/llms-full.txt |
| TanStack | https://tanstack.com/llms.txt | - |
| Better Auth | https://www.better-auth.com/llms.txt | - |

## When to Use llms.txt

Use llms.txt when:
- **Starting work with a library** - Get the architectural overview first
- **Context7 lacks recent updates** - llms.txt is always current
- **You need official API reference links** - llms.txt links to authoritative docs
- **Understanding library structure** - See what sections/features exist

## llms.txt vs Context7

| Aspect | llms.txt | Context7 |
|--------|----------|----------|
| **Source** | Official project files | Indexed documentation |
| **Granularity** | Full overview + links | Query-based chunks |
| **Freshness** | Always current | Depends on indexing |
| **Best for** | Architecture overview | Specific API questions |

**Recommended workflow:** Start with llms.txt for overview, then use Context7 for specific implementation details.

---

## Workflow

### 1. Fetch llms.txt for Overview

Use Firecrawl to fetch the llms.txt file:

```
firecrawl_scrape({ 
  url: "https://effect.website/llms.txt",
  formats: ["markdown"]
})
```

This returns a structured overview with:
- Project description
- Key documentation sections
- Links to important resources

### 2. Fetch Full Documentation (if available)

For deeper context, some libraries provide `llms-full.txt`:

```
firecrawl_scrape({ 
  url: "https://effect.website/llms-full.txt",
  formats: ["markdown"]
})
```

**Note:** Full versions can be large. Only fetch when you need comprehensive documentation.

### 3. Follow Up with Context7

After understanding the structure from llms.txt, query specific topics:

```
context7_resolve_library_id({ libraryName: "effect typescript" })
→ /effect-ts/effect

context7_get_library_docs({ 
  libraryId: "/effect-ts/effect",
  topic: "schema validation with branded types"
})
```

---

## Quick Reference

### shadcn/ui
```
firecrawl_scrape({ url: "https://ui.shadcn.com/llms.txt" })
```
- Component library with Radix UI + Tailwind CSS
- Installation guides, component docs, theming

### Effect TypeScript
```
firecrawl_scrape({ url: "https://effect.website/llms.txt" })
```
- Type-safe error handling, concurrency, Schema validation
- Full version available for comprehensive docs

### Drizzle ORM
```
firecrawl_scrape({ url: "https://orm.drizzle.team/llms.txt" })
```
- TypeScript ORM for PostgreSQL, MySQL, SQLite
- Migrations, queries, relations

### TanStack
```
firecrawl_scrape({ url: "https://tanstack.com/llms.txt" })
```
- Router, Query, Table, Form, Virtual
- Framework-agnostic with React/Vue/Solid/Svelte support

### Better Auth
```
firecrawl_scrape({ url: "https://www.better-auth.com/llms.txt" })
```
- Authentication framework for TypeScript
- Social providers, sessions, plugins

---

## Best Practices

1. **Check llms.txt first** - Before diving into code, understand the library structure
2. **Use the registry** - Reference `llm-docs-registry.yaml` for known URLs
3. **Prefer llms.txt over scraping random pages** - It's curated for LLM consumption
4. **Combine sources** - llms.txt → Context7 → specific doc pages
5. **Cache when appropriate** - For repeated work, save llms.txt content locally
