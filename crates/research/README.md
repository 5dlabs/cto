# Research Pipeline

Twitter/X bookmark monitoring and research curation pipeline for the CTO platform.

## Overview

The Research crate monitors your Twitter/X bookmarks, analyzes them for relevance using AI, enriches linked content with Firecrawl, and stores curated research as markdown with YAML frontmatter.

## Features

- **Bookmark Monitoring**: Polls Twitter/X bookmarks using headless browser automation
- **AI Analysis**: Uses Claude/GPT to score relevance and categorize content
- **Link Enrichment**: Scrapes linked URLs with Firecrawl to extract additional context
- **Markdown Storage**: Saves research entries as markdown with YAML frontmatter
- **Kubernetes CronJob**: Runs on a 5-minute schedule as a Kubernetes CronJob

## Architecture

```
Twitter Bookmarks → Browser Automation → AI Analysis → Firecrawl Enrichment → Markdown Storage
```

### Components

| Module | Description |
|--------|-------------|
| `auth` | Twitter session management and browser-based login |
| `twitter` | Bookmark polling and HTML parsing |
| `analysis` | AI-powered relevance scoring and categorization |
| `enrichment` | Firecrawl client for link content extraction |
| `storage` | Markdown generation with YAML frontmatter |
| `pipeline` | Full orchestration: poll → analyze → enrich → store |

## Categories

Content is automatically categorized into:

- **agents**: AI/LLM agent development patterns
- **rust**: Rust language, crates, and tooling
- **infrastructure**: Kubernetes, cloud, DevOps
- **tooling**: Developer tools, MCP, IDEs
- **architecture**: Software design patterns
- **devops**: CI/CD, automation, deployment
- **security**: Security practices, vulnerabilities
- **research**: Academic papers and studies
- **announcements**: Product launches, releases

## CLI Usage

```bash
# Run a single poll cycle
research poll --output=/data/research --state=/data/state.json --min-relevance=0.5

# Interactive authentication
research auth --output=.twitter-session.json
research auth --export-to-vault

# List recent entries
research list --limit=20
research list --category=rust

# Search entries
research search "async runtime"

# Process a specific tweet
research process https://x.com/user/status/123456
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `TWITTER_AUTH_TOKEN` | Long-lived auth_token cookie from Twitter |
| `TWITTER_CT0` | CSRF token (optional, regenerates automatically) |
| `ANTHROPIC_API_KEY` | Anthropic API key for Claude |
| `OPENAI_API_KEY` | OpenAI API key (alternative provider) |
| `FIRECRAWL_API_KEY` | Firecrawl API key for link scraping |

## Kubernetes Deployment

The research pipeline is deployed as a CronJob in the `cto` namespace:

```yaml
# Run every 5 minutes
schedule: "*/5 * * * *"
```

### Secrets

Store credentials in Vault:

```bash
# Twitter session cookies
vault kv put secret/research-twitter TWITTER_AUTH_TOKEN=<token> TWITTER_CT0=<ct0>

# AI provider
vault kv put secret/research-anthropic ANTHROPIC_API_KEY=<key>

# Firecrawl
vault kv put secret/research-firecrawl FIRECRAWL_API_KEY=<key>
```

### Storage

Research entries are stored on a PVC at `/data/research`:

```
/data/
├── state.json              # Tracks processed bookmarks
└── research/
    ├── index.json          # Entry catalog
    └── 2025/12/05/         # Date-based organization
        └── 123456.md       # Tweet ID as filename
```

## Output Format

Each research entry is saved as markdown with YAML frontmatter:

```markdown
---
id: "123456789"
author: "@rustlang"
posted_at: "2025-12-05T10:30:00Z"
processed_at: "2025-12-05T10:35:22Z"
relevance_score: 0.85
categories:
  - rust
  - tooling
topics:
  - async
  - tokio
tags:
  - rust
  - async
  - tokio
---

# Research: Async Runtime

## Original Tweet

> New tokio release with improved scheduler...

**Author**: @rustlang (Rust Language)  
**Posted**: December 05, 2025 at 10:30 AM

## Analysis

Highly relevant - covers async runtime improvements directly applicable to platform...

**Relevance**: High (0.85)

## Enriched Content

### Tokio Release Notes

**Source**: https://tokio.rs/blog/...

> Key improvements include...
```

## Development

### Build

```bash
cd crates/research
cargo build --release
```

### Test

```bash
cargo test -p research
```

### Lint

```bash
cargo clippy -p research -- -D warnings
```

### Run Locally

```bash
# Set up environment
export TWITTER_AUTH_TOKEN=<your-token>
export ANTHROPIC_API_KEY=<your-key>

# Run poll cycle
cargo run -p research -- poll --output=./research --state=./state.json
```

## Smoke Tests

Browser automation smoke tests are available in `examples/`:

```bash
# Test chromiumoxide (requires running Chrome)
cargo run --example smoke_chromiumoxide

# Test cookie-only approach (baseline)
cargo run --example smoke_cookies
```

## Troubleshooting

### Session Expired

If you see 401/403 errors, the Twitter session has expired:

1. Run `research auth` locally to re-authenticate
2. Export new cookies to Vault:
   ```bash
   vault kv put secret/research-twitter TWITTER_AUTH_TOKEN=<new-token>
   ```

### No Bookmarks Found

This usually means JavaScript didn't render properly:

1. Check browser automation logs
2. Verify Chromium is installed in the container
3. Ensure enough memory (2Gi recommended)

### Firecrawl Errors

Link enrichment failures are non-fatal. Check:

1. `FIRECRAWL_API_KEY` is set
2. Firecrawl rate limits not exceeded
3. Target URLs are accessible

## License

See repository LICENSE file.

