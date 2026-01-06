---
name: firecrawl
description: Web scraping and content extraction from URLs for research and documentation.
agents: [blaze, rex, nova, tap, spark, grizz, morgan, cleo]
triggers: [scrape, crawl, website, url, web content, research, external docs]
---

# Firecrawl (Web Scraping)

Use Firecrawl to extract content from websites for research, documentation, and context gathering.

## Tools

| Tool | Purpose |
|------|---------|
| `firecrawl_scrape` | Extract content from a single URL |
| `firecrawl_crawl` | Crawl multiple pages from a domain |
| `firecrawl_map` | Discover all URLs on a website |
| `firecrawl_search` | Search the web and extract results |

## Single Page Scraping

```
firecrawl_scrape({
  url: "https://docs.example.com/api/auth",
  formats: ["markdown"]
})
```

Returns clean markdown content from the page.

## Website Discovery

```
# First, map the site to find relevant pages
firecrawl_map({
  url: "https://docs.example.com",
  limit: 50
})

# Then scrape specific pages
firecrawl_scrape({ url: "https://docs.example.com/guides/quickstart" })
```

## Web Search

```
firecrawl_search({
  query: "Effect TypeScript error handling patterns",
  limit: 5
})
```

## Best Practices

1. **Map before crawl** - Discover URLs first, then selectively scrape
2. **Use markdown format** - Cleaner for LLM consumption
3. **Limit crawl depth** - Avoid token overflow with `limit` parameter
4. **Be specific with search** - Include library names and versions

## Common Use Cases

| Task | Tool | Example |
|------|------|---------|
| Read external docs | `scrape` | API documentation not in Context7 |
| Research patterns | `search` | Find implementation examples |
| Gather context | `map` + `scrape` | Understand a new library |
| PRD enrichment | `scrape` | Extract requirements from linked docs |
