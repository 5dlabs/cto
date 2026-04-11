---
name: turborepo-llm-docs
description: Fetch LLM-optimized documentation for Turborepo, the high-performance monorepo build system.
agents: [blaze, nova, bolt]
triggers: [turborepo, turbo, monorepo, build system, workspace]
llm_docs:
  - turborepo
---

# Turborepo Documentation (llms.txt)

Turborepo is a high-performance build system for JavaScript and TypeScript monorepos. This skill allows agents to fetch its LLM-optimized documentation.

## Usage

To get detailed documentation for Turborepo, use the `firecrawl_scrape` tool:

```
firecrawl_scrape({ url: "https://turbo.build/llms.txt", formats: ["markdown"] })
```

## Key Features

- **Incremental builds** - Only rebuild what changed
- **Remote caching** - Share build cache across team/CI
- **Parallel execution** - Maximize CPU utilization
- **Task pipelines** - Define task dependencies
- **Pruning** - Generate minimal subsets for deployment

## Related Skills

- `bolt` - Infrastructure setup (often initializes Turborepo)
- `shadcn-stack` - Frontend stack that works well in monorepos
