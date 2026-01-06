---
name: prisma-llm-docs
description: Fetch LLM-optimized documentation for Prisma, the next-generation TypeScript ORM.
agents: [nova, rex, grizz]
triggers: [prisma, prisma docs, orm, database, typescript orm]
llm_docs:
  - prisma
---

# Prisma Documentation (llms.txt)

Prisma is a next-generation ORM for Node.js & TypeScript that provides type-safe database access with auto-generated queries. This skill allows agents to fetch its LLM-optimized documentation.

## Usage

To get detailed documentation for Prisma, use the `firecrawl_scrape` tool:

```
firecrawl_scrape({ url: "https://www.prisma.io/llms.txt", formats: ["markdown"] })
```

## Key Features

- **Prisma Client** - Type-safe database queries with auto-completion
- **Prisma Migrate** - Declarative database migrations
- **Prisma Studio** - Visual database browser
- **Multi-database support** - PostgreSQL, MySQL, SQLite, MongoDB, SQL Server

## Related Skills

- `effect-patterns` - Effect TypeScript patterns (often used with Prisma)
- `drizzle-llm-docs` - Alternative TypeScript ORM
