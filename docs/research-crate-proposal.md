# Research Crate Proposal: X Bookmarks → Knowledge Repository

**Date**: December 5, 2025  
**Status**: Proposal  
**Inspired by**: [elizaOS/knowledge](https://github.com/elizaOS/knowledge)

---

## Executive Summary

Create a Rust crate that automatically ingests X bookmarks, converts them to structured Markdown, and commits them to a knowledge repository. This enables continuous research aggregation from social media.

---

## Reference Architecture: elizaOS/knowledge

The [elizaOS/knowledge](https://github.com/elizaOS/knowledge) repository provides an excellent blueprint:

### Their Pipeline

```
01:00 UTC  External Data Ingestion (sync.yml)
01:15 UTC  Daily Fact Extraction (extract_daily_facts.yml)
01:30 UTC  Context Aggregation (aggregate-daily-sources.yml)
02:00 UTC  Council Briefing Generation (generate-council-briefing.yml)
02:30 UTC  HackMD Note Updates (update_hackmd_notes.yml)
04:00 UTC  Poster Generation (generate-posters.yml)
04:30 UTC  Discord Briefing (daily_discord_briefing.yml)
```

### Key Components

1. **Data Sources**: Discord, GitHub, AI news aggregators
2. **Processing**: Python scripts with LLM integration (OpenRouter)
3. **Storage**: JSON + Markdown files in git
4. **Distribution**: GitHub Pages, HackMD, Discord webhooks

---

## Proposed Architecture: cto-research

### Option A: Playwright-based Scraper (Recommended)

Use Playwright for browser automation with persistent authentication.

```
┌─────────────────────────────────────────────────────────┐
│                    cto-research crate                   │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌─────────────┐    ┌──────────────┐    ┌───────────┐  │
│  │  Playwright │───▶│ X Bookmarks  │───▶│  Parser   │  │
│  │  Browser    │    │  Scraper     │    │           │  │
│  └─────────────┘    └──────────────┘    └─────┬─────┘  │
│         │                                      │        │
│         ▼                                      ▼        │
│  ┌─────────────┐    ┌──────────────┐    ┌───────────┐  │
│  │  Persistent │    │   LLM        │◀───│  Markdown │  │
│  │  Session    │    │   Enrichment │    │  Generator│  │
│  │  (cookies)  │    │  (optional)  │    │           │  │
│  └─────────────┘    └──────────────┘    └─────┬─────┘  │
│                                                │        │
│                                                ▼        │
│                                         ┌───────────┐  │
│                                         │   Git     │  │
│                                         │   Commit  │  │
│                                         └───────────┘  │
└─────────────────────────────────────────────────────────┘
```

### Option B: MCP Server Approach

Create an MCP server that exposes X bookmarks as a resource.

```rust
// mcp/src/tools/x_bookmarks.rs
pub struct XBookmarksTool {
    browser: Browser,
    session_path: PathBuf,  // Persistent cookies
}

impl Tool for XBookmarksTool {
    async fn list_bookmarks(&self) -> Result<Vec<Bookmark>> { ... }
    async fn scrape_bookmark(&self, url: &str) -> Result<Post> { ... }
    async fn save_to_markdown(&self, post: &Post) -> Result<PathBuf> { ... }
}
```

### Option C: Hybrid (Browser Extension + Backend)

1. Browser extension captures bookmarks as you add them
2. Sends to backend service via webhook
3. Backend processes and commits to git

---

## Data Model

### Raw Bookmark (JSON)

```json
{
  "id": "1980629163976675779",
  "author": {
    "handle": "omarsar0",
    "name": "elvis",
    "verified": true
  },
  "content": "People are sleeping on Deep Agents...",
  "media": [
    {"type": "image", "url": "..."}
  ],
  "links": ["https://arxiv.org/..."],
  "metrics": {
    "replies": 3,
    "reposts": 16,
    "likes": 156,
    "views": 7500
  },
  "created_at": "2025-06-17T12:00:00Z",
  "bookmarked_at": "2025-12-05T01:00:00Z",
  "tags": ["agents", "mcp", "enterprise"]  // LLM-generated
}
```

### Processed Markdown

```markdown
# Deep Agents for Enterprise

**Author**: @omarsar0 (elvis) ✓  
**Date**: June 17, 2025  
**URL**: https://x.com/omarsar0/status/1980629163976675779  
**Tags**: #agents #mcp #enterprise

---

> People are sleeping on Deep Agents. Start using them now. This is a fun paper showcasing how to put together advanced deep agents for enterprise use cases. Uses the best techniques: task decomposition, planning, specialized subagents, MCP for NL2SQL, file analysis, and more.

## Key Takeaways

- Task decomposition is essential for complex agents
- Planning layers improve agent reliability
- Specialized subagents for different tasks
- MCP for tool integration

## References

- [Linked Paper](https://arxiv.org/...)

---

*Metrics: 3 replies • 16 reposts • 156 likes • 7.5K views*
```

---

## Implementation Plan

### Phase 1: Browser Automation (Week 1-2)

```rust
// crates/research/src/scraper.rs
use playwright::Playwright;

pub struct XScraper {
    browser: Browser,
    context: BrowserContext,  // Persistent session
}

impl XScraper {
    pub async fn new(session_dir: &Path) -> Result<Self> {
        let playwright = Playwright::initialize().await?;
        let browser = playwright.chromium().launch(LaunchOptions {
            headless: true,
            ..Default::default()
        }).await?;
        
        // Load persistent context with cookies
        let context = browser.new_context(ContextOptions {
            storage_state: Some(session_dir.join("state.json")),
            ..Default::default()
        }).await?;
        
        Ok(Self { browser, context })
    }
    
    pub async fn scrape_bookmarks(&self) -> Result<Vec<Bookmark>> {
        let page = self.context.new_page().await?;
        page.goto("https://x.com/i/bookmarks").await?;
        
        // Scroll and collect bookmarks
        let mut bookmarks = Vec::new();
        loop {
            let posts = page.query_selector_all("article").await?;
            for post in posts {
                bookmarks.push(self.parse_post(&post).await?);
            }
            
            // Check for new content after scroll
            if !self.scroll_and_wait(&page).await? {
                break;
            }
        }
        
        Ok(bookmarks)
    }
}
```

### Phase 2: LLM Enrichment (Week 2-3)

```rust
// crates/research/src/enrichment.rs
pub struct Enricher {
    client: OpenRouterClient,
}

impl Enricher {
    pub async fn enrich(&self, bookmark: &Bookmark) -> Result<EnrichedBookmark> {
        let prompt = format!(
            "Analyze this X post and extract:\n\
             1. Key topics/tags (3-5)\n\
             2. Brief summary (1-2 sentences)\n\
             3. Key takeaways (bullet points)\n\
             4. Related concepts\n\n\
             Post: {}\n\
             Author: @{}\n\
             Links: {:?}",
            bookmark.content,
            bookmark.author.handle,
            bookmark.links
        );
        
        let response = self.client.complete(&prompt).await?;
        // Parse LLM response into structured data
        Ok(EnrichedBookmark::from_llm_response(bookmark, response)?)
    }
}
```

### Phase 3: Git Integration (Week 3-4)

```rust
// crates/research/src/repository.rs
pub struct ResearchRepo {
    repo_path: PathBuf,
}

impl ResearchRepo {
    pub async fn save_bookmark(&self, bookmark: &EnrichedBookmark) -> Result<PathBuf> {
        let date = bookmark.bookmarked_at.format("%Y-%m-%d");
        let filename = format!("{}-{}.md", date, bookmark.id);
        let path = self.repo_path.join("xposts").join(&filename);
        
        let markdown = bookmark.to_markdown()?;
        tokio::fs::write(&path, markdown).await?;
        
        // Git add + commit
        self.git_commit(&path, &format!(
            "research: add bookmark from @{}", 
            bookmark.author.handle
        )).await?;
        
        Ok(path)
    }
    
    pub async fn sync(&self) -> Result<SyncReport> {
        let scraper = XScraper::new(&self.session_path).await?;
        let existing = self.get_existing_ids().await?;
        
        let bookmarks = scraper.scrape_bookmarks().await?;
        let new_bookmarks: Vec<_> = bookmarks
            .into_iter()
            .filter(|b| !existing.contains(&b.id))
            .collect();
        
        let enricher = Enricher::new()?;
        let mut saved = Vec::new();
        
        for bookmark in new_bookmarks {
            let enriched = enricher.enrich(&bookmark).await?;
            let path = self.save_bookmark(&enriched).await?;
            saved.push(path);
        }
        
        Ok(SyncReport { new_count: saved.len(), paths: saved })
    }
}
```

### Phase 4: Automation (Week 4)

```yaml
# .github/workflows/sync-bookmarks.yml
name: Sync X Bookmarks

on:
  schedule:
    - cron: '0 */6 * * *'  # Every 6 hours
  workflow_dispatch:

jobs:
  sync:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Playwright
        run: npx playwright install chromium
      
      - name: Restore Session
        uses: actions/cache@v4
        with:
          path: .session/
          key: x-session-${{ github.repository }}
      
      - name: Sync Bookmarks
        env:
          OPENROUTER_API_KEY: ${{ secrets.OPENROUTER_API_KEY }}
        run: cargo run -p research -- sync
      
      - name: Commit Changes
        run: |
          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"
          git add xposts/
          git diff --staged --quiet || git commit -m "research: sync bookmarks $(date +%Y-%m-%d)"
          git push
```

---

## Authentication Strategy

### Option 1: Persistent Browser State (Recommended)

```bash
# Initial setup (one-time, manual)
cargo run -p research -- login

# Opens browser, you log in manually
# Saves cookies to .session/state.json
# Subsequent runs use saved session
```

### Option 2: Environment Variables

```bash
# Store encrypted cookies in GitHub Secrets
X_AUTH_TOKEN=...
X_CT0=...
```

### Option 3: OAuth App (Most Robust)

Register as X developer and use official OAuth flow.

---

## Directory Structure

```
crates/
└── research/
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        ├── scraper.rs      # Playwright-based scraping
        ├── parser.rs       # HTML → structured data
        ├── enrichment.rs   # LLM-based tagging/summarization
        ├── markdown.rs     # Structured data → Markdown
        ├── repository.rs   # Git operations
        └── cli.rs          # Command-line interface

docs/
└── xposts/
    ├── 2025-12-05-1980629163976675779.md
    ├── 2025-12-05-1933652486520242421.md
    └── index.md            # Auto-generated index
```

---

## CLI Interface

```bash
# Initial login (opens browser for manual auth)
cto-research login

# Sync new bookmarks
cto-research sync

# Sync with LLM enrichment
cto-research sync --enrich

# Generate index/summary
cto-research index

# Export to JSON
cto-research export --format json > bookmarks.json
```

---

## Integration Points

### With CTO Platform

1. **MCP Tool**: Expose as `research_bookmarks` tool for agents
2. **Knowledge Base**: Feed into RAG system for agent context
3. **Workflow Trigger**: New bookmarks trigger research workflows

### With Existing Tools

1. **Firecrawl**: For scraping linked URLs
2. **OpenRouter**: For LLM enrichment
3. **GitHub Actions**: For automation

---

## Security Considerations

1. **Session Storage**: Encrypt browser state at rest
2. **API Keys**: Use secrets management (GitHub Secrets, Vault)
3. **Rate Limiting**: Respect X's rate limits (avoid bans)
4. **Private Data**: Don't commit DMs or private bookmarks

---

## References

- [elizaOS/knowledge](https://github.com/elizaOS/knowledge) - Knowledge aggregation system
- [elizaOS/eliza](https://github.com/elizaOS/eliza) - Autonomous agent framework
- [Playwright Rust](https://github.com/nickelc/playwright-rust) - Browser automation
- [madjin/daily-silk](https://github.com/madjin/daily-silk) - Discord AI news scraper

---

## Next Steps

1. [ ] Create `crates/research/` scaffold
2. [ ] Implement Playwright-based scraper
3. [ ] Add LLM enrichment with OpenRouter
4. [ ] Set up GitHub Actions automation
5. [ ] Test with your X account
6. [ ] Document authentication flow









