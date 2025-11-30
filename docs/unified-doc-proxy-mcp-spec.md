# Unified Doc Proxy MCP Tool Specification

**Date:** November 26, 2025  
**Purpose:** Specification for a unified MCP tool that proxies documentation requests to Context7 or Firecrawl based on document type.

---

## Overview

Create a single MCP tool that provides a unified interface for documentation retrieval, automatically routing to the appropriate backend:

- **`repo`** → Context7 (for Git-hosted library documentation)
- **`scrape`** → Firecrawl (for arbitrary website documentation)

This replaces the need for users to know which backend to call - they just provide a URL and type.

---

## Tool Interface

### Tool Name
```
add_docs
```

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `url` | string | ✅ | The documentation source URL |
| `type` | enum | ✅ | `"repo"` or `"scrape"` |
| `query` | string | ❌ | Optional search query/topic to focus on |

### Example Calls

**For a GitHub repository (library docs):**
```json
{
  "name": "add_docs",
  "arguments": {
    "url": "https://github.com/solana-labs/solana",
    "type": "repo",
    "query": "transactions"
  }
}
```

**For a website (API docs, etc.):**
```json
{
  "name": "add_docs",
  "arguments": {
    "url": "https://docs.birdeye.so",
    "type": "scrape",
    "query": "wallet API"
  }
}
```

---

## Routing Logic

```
┌─────────────────────────────────────────────────────────────┐
│                      add_docs(url, type)                    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │  type == "repo" │
                    └─────────────────┘
                      │           │
                     YES          NO
                      │           │
                      ▼           ▼
            ┌─────────────┐  ┌─────────────┐
            │  Context7   │  │  Firecrawl  │
            │   (repo)    │  │  (scrape)   │
            └─────────────┘  └─────────────┘
```

---

## Backend Integration

### For `type: "repo"` → Context7

**Step 1:** Resolve the library ID from the URL

```
Input URL: https://github.com/solana-labs/solana
Extract:   org = "solana-labs", project = "solana"
Library ID: /solana-labs/solana
```

**Step 2:** Call Context7 API

```bash
# Resolve library ID
GET https://context7.com/api/v2/search?query=solana-labs/solana

# Get documentation
GET https://context7.com/api/v2/docs/code/{library_id}?topic={query}
Authorization: Bearer {CONTEXT7_API_KEY}
```

**API Reference:**
- Endpoint: `https://context7.com/api/v2/docs/code/{library_id}`
- Auth: `Authorization: Bearer ctx7sk-...`
- Params: `topic` (optional), `tokens` (optional, default 10000)

---

### For `type: "scrape"` → Firecrawl

**Option A: Single page scrape**
```json
{
  "tool": "firecrawl_scrape",
  "arguments": {
    "url": "https://docs.birdeye.so/docs/overview",
    "formats": ["markdown"],
    "onlyMainContent": true
  }
}
```

**Option B: Search within scraped content**
```json
{
  "tool": "firecrawl_search",
  "arguments": {
    "query": "site:docs.birdeye.so {user_query}",
    "limit": 5,
    "scrapeOptions": {
      "formats": ["markdown"],
      "onlyMainContent": true
    }
  }
}
```

**Option C: Map site then scrape relevant pages**
```json
// Step 1: Map
{
  "tool": "firecrawl_map",
  "arguments": {
    "url": "https://docs.birdeye.so",
    "search": "{query}",
    "limit": 10
  }
}

// Step 2: Scrape matched URLs
{
  "tool": "firecrawl_scrape",
  "arguments": {
    "url": "{matched_url}",
    "formats": ["markdown"]
  }
}
```

**API Credentials:**
- API Key: `FIRECRAWL_API_KEY=fc-...`

---

## Response Format

Return a unified response regardless of backend:

```json
{
  "success": true,
  "source": "context7" | "firecrawl",
  "url": "https://...",
  "type": "repo" | "scrape",
  "content": [
    {
      "title": "Section Title",
      "url": "https://source-url",
      "content": "Markdown content...",
      "relevance": 0.95
    }
  ],
  "metadata": {
    "snippets_count": 10,
    "tokens_used": 5000
  }
}
```

---

## Environment Variables Required

```bash
# Context7 (for repo type)
CONTEXT7_API_KEY=ctx7sk-27071219-bd21-4645-9b01-2de3d517f08b

# Firecrawl (for scrape type)
FIRECRAWL_API_KEY=fc-283175e66d0a47168f6f6083fb5e9831
```

---

## URL Parsing Logic

For `type: "repo"`, extract library info from URL:

```python
def parse_repo_url(url: str) -> dict:
    """
    Parse GitHub/GitLab/Bitbucket URL to extract org and project.
    
    Examples:
    - https://github.com/solana-labs/solana → /solana-labs/solana
    - https://github.com/cilium/cilium → /cilium/cilium
    - https://gitlab.com/org/project → /org/project
    """
    # Strip protocol and host
    path = url.replace("https://", "").replace("http://", "")
    
    # Remove host (github.com, gitlab.com, etc.)
    parts = path.split("/")
    if len(parts) >= 3:
        org = parts[1]
        project = parts[2].replace(".git", "")
        return {
            "org": org,
            "project": project,
            "library_id": f"/{org}/{project}"
        }
    
    raise ValueError(f"Could not parse repo URL: {url}")
```

For `type: "scrape"`, use the URL directly with Firecrawl.

---

## Error Handling

```json
// Library not found in Context7
{
  "success": false,
  "error": "library_not_found",
  "message": "Library '/org/project' not found in Context7. Try type='scrape' instead.",
  "suggestion": "Submit this repo at https://context7.com/add"
}

// Scrape failed
{
  "success": false,
  "error": "scrape_failed",
  "message": "Could not scrape URL: connection timeout",
  "url": "https://..."
}
```

---

## Implementation Notes

### For CTO MCP Server (Rust)

1. Add new tool handler in `tools.rs`
2. Create HTTP clients for Context7 and Firecrawl APIs
3. Implement URL parsing logic
4. Add environment variable handling for API keys

### Dependencies

```toml
[dependencies]
reqwest = { version = "0.11", features = ["json"] }
serde_json = "1.0"
url = "2.4"
```

### Suggested File Structure

```
cto/mcp/src/
├── doc_proxy/
│   ├── mod.rs           # Module entry
│   ├── context7.rs      # Context7 API client
│   ├── firecrawl.rs     # Firecrawl API client
│   ├── router.rs        # Type-based routing logic
│   └── url_parser.rs    # URL parsing utilities
└── tools.rs             # Register add_docs tool
```

---

## Testing

### Test Cases

1. **Repo - Known library:**
   ```json
   {"url": "https://github.com/solana-labs/solana", "type": "repo", "query": "transactions"}
   ```
   Expected: Context7 returns Solana transaction docs

2. **Repo - Unknown library:**
   ```json
   {"url": "https://github.com/unknown/unknown", "type": "repo"}
   ```
   Expected: Error with suggestion to submit or use scrape

3. **Scrape - API docs:**
   ```json
   {"url": "https://docs.birdeye.so", "type": "scrape", "query": "wallet API"}
   ```
   Expected: Firecrawl returns scraped content

4. **Scrape - Single page:**
   ```json
   {"url": "https://docs.birdeye.so/reference/get-wallet-v2-pnl", "type": "scrape"}
   ```
   Expected: Firecrawl returns single page content

---

## Migration Path

This tool replaces the existing agent-docs PostgreSQL-based document retrieval:

| Before | After |
|--------|-------|
| Custom ingestion jobs | No ingestion needed |
| PostgreSQL vector store | Context7 (managed) |
| Custom scraping | Firecrawl (managed) |
| `doc_query` tool | `add_docs` tool |

---

## Summary

**Two parameters, two backends, one interface:**

```
add_docs(url, type)
  │
  ├── type: "repo"   → Context7 API
  │
  └── type: "scrape" → Firecrawl API
```

No more maintaining ingestion pipelines, embeddings, or infrastructure. Just point and retrieve.

