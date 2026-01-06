---
name: zod-llm-docs
description: Fetch Zod schema validation documentation via llms.txt for up-to-date API references
agents: [nova, blaze, rex]
triggers: [zod, schema validation, type inference, z.object, z.string, z.number]
llm_docs_url: https://zod.dev/llms.txt
---

# Zod LLM Documentation

Zod provides comprehensive LLM-optimized documentation at `https://zod.dev/llms.txt`.

## When to Use

Fetch this documentation when:
- Defining TypeScript-first schemas for data validation
- Working with Zod 4's new features (codecs, metadata, registries)
- Converting schemas to/from JSON Schema
- Customizing or formatting validation errors
- Integrating with form libraries or API validation
- Using Zod Mini for bundle size optimization

## Key Topics Covered

- **Schema Types**: Primitives, Objects, Arrays, Unions, Records, Tuples
- **String Validation**: Email, UUID, URL, ISO datetime, IP addresses, JWT
- **Transformations**: Refinements, Transforms, Pipes, Codecs
- **Error Handling**: Custom errors, Formatting, Internationalization
- **Advanced**: Recursive objects, Discriminated unions, Branded types
- **JSON Schema**: Conversion, Metadata, Registries

## Quick Reference

```typescript
// Fetch Zod docs via Firecrawl
const docs = await firecrawl.scrape({
  url: "https://zod.dev/llms.txt",
  formats: ["markdown"]
});
```

## Common Schema Patterns

```typescript
import { z } from "zod";

// Basic object schema
const userSchema = z.object({
  id: z.string().uuid(),
  email: z.string().email(),
  age: z.number().int().positive(),
  role: z.enum(["admin", "user"]),
});

// Infer TypeScript type
type User = z.infer<typeof userSchema>;
```

## Zod 4 Features

| Feature | Description |
|---------|-------------|
| Codecs | Bidirectional encode/decode |
| Metadata | Attach metadata to schemas |
| Registries | Organize and reference schemas |
| Zod Mini | Tree-shakable minimal build |
| JSON Schema | Native conversion support |

## Related Skills

- `effect-patterns` - Effect TypeScript (uses Zod-like Schema)
- `better-auth` - Uses Zod for validation
