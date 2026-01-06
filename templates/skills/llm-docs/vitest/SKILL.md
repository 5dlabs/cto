---
name: vitest-llm-docs
description: Fetch LLM-optimized documentation for Vitest, the Vite-native testing framework.
agents: [blaze, nova, tess]
triggers: [vitest, vitest docs, testing, unit tests, test framework, vite testing]
llm_docs:
  - vitest
---

# Vitest Documentation (llms.txt)

Vitest is a Vite-native testing framework that provides a fast and feature-rich testing experience. This skill allows agents to fetch its LLM-optimized documentation.

## Usage

To get detailed documentation for Vitest, use the `firecrawl_scrape` tool:

```
firecrawl_scrape({ url: "https://vitest.dev/llms.txt", formats: ["markdown"] })
```

## Key Features

- **Vite-native** - Instant test runs with Vite's transform pipeline
- **Jest compatible** - Drop-in replacement API
- **ESM first** - Native ES modules support
- **TypeScript support** - Out-of-box TypeScript
- **UI mode** - Interactive test interface
- **Coverage** - Built-in code coverage

## Related Skills

- `tess` - Testing agent (uses Vitest for frontend tests)
- `shadcn-stack` - Frontend stack that uses Vitest
