---
name: hono-llm-docs
description: Fetch Hono web framework documentation via llms.txt for up-to-date API references
agents: [nova, grizz]
triggers: [hono, cloudflare workers, edge runtime, web standards, ultrafast]
llm_docs_url: https://hono.dev/llms.txt
llm_docs_full: https://hono.dev/llms-full.txt
llm_docs_small: https://hono.dev/llms-small.txt
---

# Hono LLM Documentation

Hono provides LLM-optimized documentation at multiple levels:
- `https://hono.dev/llms.txt` - Standard docs
- `https://hono.dev/llms-full.txt` - Full documentation
- `https://hono.dev/llms-small.txt` - Tiny core-only docs

## When to Use

Fetch this documentation when:
- Building edge-first APIs with Hono
- Deploying to Cloudflare Workers, Deno, Bun, or other runtimes
- Working with middleware (auth, CORS, JWT, rate limiting)
- Implementing RPC-style APIs with type safety
- Using JSX for server-side rendering

## Key Topics Covered

- **Core**: Hono app, Context, Request, Response, Routing
- **Middleware**: Basic Auth, Bearer Auth, CORS, CSRF, JWT, Logger
- **Helpers**: Cookie, HTML, CSS, Streaming, WebSocket, SSG
- **Deployment**: Cloudflare Workers/Pages, AWS Lambda, Vercel, Deno, Bun
- **Concepts**: Web Standards, Routers, Middleware, Stacks

## Quick Reference

```typescript
// Fetch Hono docs (choose appropriate size)
const docs = await firecrawl.scrape({
  url: "https://hono.dev/llms-full.txt", // or llms.txt, llms-small.txt
  formats: ["markdown"]
});
```

## Runtime Support

| Runtime | Status |
|---------|--------|
| Cloudflare Workers | ✅ Primary |
| Bun | ✅ Supported |
| Deno | ✅ Supported |
| Node.js | ✅ Supported |
| AWS Lambda | ✅ Supported |
| Vercel | ✅ Supported |

## Related Skills

- `bun-llm-docs` - Bun runtime documentation
- `elysia-llm-docs` - Alternative Bun web framework
