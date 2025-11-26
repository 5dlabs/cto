# pg-aiguide PostgreSQL Tools

You have access to **pg-aiguide** for AI-optimized PostgreSQL expertise.

## Available Tools

| Tool | Use Case |
|------|----------|
| `semantic_search_postgres_docs` | Search PostgreSQL manual (version-aware) |
| `semantic_search_tiger_docs` | Search TimescaleDB and ecosystem docs |
| `view_skill` | Get curated best practices for schema design, indexing |

## When to Use pg-aiguide

**Always use pg-aiguide when:**

- Designing database schemas (tables, constraints, indexes)
- Writing SQL queries or migrations
- Choosing data types or indexing strategies
- Implementing parameterized queries with SQLx
- Reviewing code for SQL injection vulnerabilities

## Example Usage

**Schema design best practices:**

```javascript
view_skill({ skill: "schema design" })
```

**Search for constraint patterns:**

```javascript
semantic_search_postgres_docs({
  query: "CHECK constraints partial indexes",
  version: "17"
})
```

**Modern PostgreSQL features:**

```javascript
semantic_search_postgres_docs({
  query: "GENERATED ALWAYS AS IDENTITY vs SERIAL"
})
```

**TimescaleDB time-series patterns:**

```javascript
semantic_search_tiger_docs({
  query: "hypertable compression continuous aggregates"
})
```

## Best Practices

✅ **DO:**

- Query pg-aiguide BEFORE designing any database schema
- Use `view_skill` for best practices on common patterns
- Specify PostgreSQL version when relevant (e.g., PG17 features)
- Check for proper constraints, indexes, and data types

❌ **DON'T:**

- Skip pg-aiguide when writing database code
- Use Context7 for PostgreSQL-specific questions (pg-aiguide is better)
- Ignore constraint and index recommendations
- Use outdated patterns like SERIAL when IDENTITY is available
