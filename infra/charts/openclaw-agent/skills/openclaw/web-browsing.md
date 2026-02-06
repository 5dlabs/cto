# Web Browsing Skill

You have four distinct web tools. Choose the right one for the job.

## Tool Selection Guide

| Tool | Use When | JS Support | Speed |
|------|----------|------------|-------|
| `web_search` | Finding URLs, quick answers, current events | N/A | Fast |
| `web_fetch` | Reading articles, docs, APIs (static HTML) | No | Fast |
| `browser` | JS-heavy sites, logins, interactive pages | Yes (Playwright) | Slow |
| Firecrawl | `web_fetch` fails due to anti-bot protection | Partial | Medium |

## web_search (Brave/Perplexity)

Structured search results with snippets. Use for discovery — finding the right URLs before fetching.

```
Query: "Kubernetes PVC resize documentation"
→ Returns: titles, URLs, snippets
```

## web_fetch (HTTP GET + Readability)

Direct HTTP fetch with automatic content extraction. No JavaScript execution. Best for:
- Documentation pages
- Blog posts and articles
- API responses (JSON/XML)
- GitHub raw files

## browser (Playwright)

Full browser automation. Use only when `web_fetch` fails or you need interaction:
- Pages that require JavaScript rendering (SPAs)
- Sites behind login forms
- Pages with dynamic content loading
- Taking screenshots for visual verification

**Cost:** Slow and resource-heavy. Always try `web_fetch` first.

## Firecrawl (Anti-Bot Fallback)

When `web_fetch` returns empty/blocked content, retry with Firecrawl. It handles:
- Cloudflare challenges
- Rate limiting with retries
- JavaScript-rendered content

## Decision Flow

1. Need to find something? → `web_search`
2. Know the URL, static page? → `web_fetch`
3. `web_fetch` blocked/empty? → Firecrawl
4. Need JS, login, or interaction? → `browser`
