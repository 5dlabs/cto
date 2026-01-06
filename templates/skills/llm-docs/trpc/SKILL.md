---
name: trpc-llm-docs
description: Fetch LLM-optimized documentation for tRPC, the end-to-end typesafe API framework.
agents: [blaze, nova]
triggers: [trpc, trpc docs, typesafe api, api framework, rpc]
llm_docs:
  - trpc
---

# tRPC Documentation (llms.txt)

tRPC allows you to easily build & consume fully typesafe APIs without schemas or code generation. This skill allows agents to fetch its LLM-optimized documentation.

## Usage

To get detailed documentation for tRPC, use the `firecrawl_scrape` tool:

```
firecrawl_scrape({ url: "https://trpc.io/llms.txt", formats: ["markdown"] })
```

## Key Features

- **End-to-end typesafety** - Full type inference between client and server
- **Framework agnostic** - Works with Next.js, Express, Fastify, and more
- **Subscriptions** - Real-time support with WebSockets
- **Procedure batching** - Automatic request batching

## Related Skills

- `effect-patterns` - Effect TypeScript patterns
- `shadcn-stack` - Next.js + shadcn/ui stack (often used with tRPC)
