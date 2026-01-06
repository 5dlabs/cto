---
name: clerk-llm-docs
description: Fetch LLM-optimized documentation for Clerk, the complete user management platform.
agents: [blaze, nova, tap, spark]
triggers: [clerk, clerk docs, user management, authentication, identity]
llm_docs:
  - clerk
---

# Clerk Documentation (llms.txt)

Clerk is a complete user management platform providing authentication, user profiles, and organization management. This skill allows agents to fetch its LLM-optimized documentation.

## Usage

To get detailed documentation for Clerk, use the `firecrawl_scrape` tool:

```
firecrawl_scrape({ url: "https://clerk.com/llms.txt", formats: ["markdown"] })
```

## Key Features

- **Authentication** - Email, social, phone, Web3 login
- **User Management** - Pre-built UI components
- **Organizations** - Multi-tenant support
- **Session Management** - Secure session handling
- **Webhooks** - Real-time user events

## Related Skills

- `better-auth` - Alternative TypeScript-first auth framework
- `shadcn-stack` - Next.js stack (works well with Clerk)
