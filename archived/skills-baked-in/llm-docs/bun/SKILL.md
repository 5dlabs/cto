---
name: bun-llm-docs
description: Fetch Bun runtime documentation via llms.txt for up-to-date API references
agents: [nova, blaze]
triggers: [bun, bun runtime, bun install, bunx, bun test, bun build]
llm_docs_url: https://bun.sh/llms.txt
---

# Bun LLM Documentation

Bun provides comprehensive LLM-optimized documentation at `https://bun.sh/llms.txt`.

## When to Use

Fetch this documentation when:
- Setting up a new Bun project or migrating from Node.js
- Working with Bun-specific APIs (SQLite, S3, Redis, FFI)
- Configuring the bundler, minifier, or bytecode caching
- Writing tests with `bun test`
- Using Bun's shell scripting API
- Deploying to cloud platforms (AWS Lambda, Vercel, Railway)

## Key Topics Covered

- **Runtime**: File I/O, HTTP Server, WebSockets, Workers, Streams
- **Bundler**: CSS, HTML, Hot Reloading, Macros, Plugins, Executables
- **Package Manager**: Install, Add, Update, Workspaces, Lockfile
- **Testing**: Coverage, Mocks, Snapshots, DOM testing, Jest migration
- **Ecosystem**: Elysia, Express, Hono, Drizzle, Prisma, Next.js, Remix

## Quick Reference

```typescript
// Fetch Bun docs via Firecrawl
const docs = await firecrawl.scrape({
  url: "https://bun.sh/llms.txt",
  formats: ["markdown"]
});
```

## Key APIs Documented

| Module | Description |
|--------|-------------|
| `Bun.serve` | HTTP server with WebSocket support |
| `Bun.file` | File I/O with lazy loading |
| `Bun.build` | JavaScript/TypeScript bundler |
| `Bun.$` | Shell scripting API |
| `Bun.sql` | SQL database client |
| `bun:sqlite` | SQLite database |
| `bun:ffi` | Foreign function interface |

## Related Skills

- `elysia-llm-docs` - Elysia web framework
- `effect-patterns` - Effect TypeScript patterns
