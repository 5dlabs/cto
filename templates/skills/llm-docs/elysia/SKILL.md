---
name: elysia-llm-docs
description: Fetch Elysia framework documentation via llms.txt for up-to-date API references
agents: [nova]
triggers: [elysia, bun server, typescript backend, eden, end-to-end type safety]
llm_docs_url: https://elysiajs.com/llms.txt
---

# Elysia LLM Documentation

Elysia provides LLM-optimized documentation at `https://elysiajs.com/llms.txt`.

## When to Use

Fetch this documentation when:
- Setting up a new Elysia project
- Implementing route handlers, plugins, or lifecycle hooks
- Working with Eden for end-to-end type safety
- Integrating with Better Auth, Drizzle, or other services
- Configuring OpenTelemetry, CORS, JWT, or other plugins

## Key Topics Covered

- **Essential**: Routes, Handlers, Plugins, Lifecycle, Validation, Best Practices
- **Patterns**: Configuration, Cookies, Error Handling, WebSocket, TypeScript
- **Eden**: End-to-end type safety, Treaty, Fetch, Unit Testing
- **Plugins**: Bearer, CORS, Cron, GraphQL, HTML, JWT, OpenAPI, Static
- **Integrations**: AI SDK, Astro, Better Auth, Cloudflare, Drizzle, Expo, Next.js, Prisma

## Quick Reference

```typescript
// Fetch Elysia docs via Firecrawl
const docs = await firecrawl.scrape({
  url: "https://elysiajs.com/llms.txt",
  formats: ["markdown"]
});
```

## Related Skills

- `effect-patterns` - Effect TypeScript integration
- `better-auth` - Authentication framework
- `bun-llm-docs` - Bun runtime documentation
