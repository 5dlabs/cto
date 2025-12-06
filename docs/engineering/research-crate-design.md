# Research Crate Design Document

**Date:** December 5, 2025  
**Status:** Draft  
**Author:** CTO Platform Team

---

## 1. Overview

### 1.1 Purpose

Create a new `research` crate that monitors Twitter/X bookmarks for relevant content, analyzes it using LLMs, and enriches it with linked documentation. This provides a semi-automated research pipeline for discovering and curating technical knowledge that may be useful for the CTO platform.

### 1.2 Problem Statement

Valuable technical insights are often discovered on Twitter/X but get lost in the feed. Manually curating, analyzing, and storing relevant tweets is time-consuming. We need an automated system that:

1. Monitors bookmarked tweets for new content
2. Analyzes tweets for platform relevance
3. Extracts and enriches content from linked resources
4. Stores curated research in a searchable format

### 1.3 Goals

- **Passive Collection**: Poll bookmarks every 5 minutes without triggering rate limits
- **Intelligent Filtering**: Use LLMs to determine relevance to CTO platform goals
- **Content Enrichment**: Automatically scrape linked resources for context
- **Structured Storage**: Store research as markdown with metadata for future retrieval

---

## 2. Architecture

### 2.1 High-Level Flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           Research Pipeline                              │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Twitter/X     │───▶│   Bookmark      │───▶│   Content       │
│   Auth          │    │   Poller        │    │   Extractor     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                                      │
                                                      ▼
                       ┌─────────────────┐    ┌─────────────────┐
                       │   AI Analyzer   │◀───│   Tweet Parser  │
                       │   (Relevance)   │    │                 │
                       └─────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌─────────────────┐
                       │  Is Relevant?   │
                       └─────────────────┘
                          │         │
                         YES        NO
                          │         │
                          ▼         ▼
┌─────────────────┐    ┌────────┐  ┌────────┐
│   Link          │    │ Store  │  │ Skip   │
│   Enricher      │───▶│ MD     │  │        │
│   (Firecrawl)   │    │        │  │        │
└─────────────────┘    └────────┘  └────────┘
```

### 2.2 Component Breakdown

| Component | Responsibility |
|-----------|---------------|
| **Twitter Auth** | Manage browser-based authentication using stored credentials |
| **Bookmark Poller** | Check for new bookmarks every 5 minutes |
| **Content Extractor** | Parse tweet content, media, and metadata |
| **Tweet Parser** | Structure raw content into analyzable format |
| **AI Analyzer** | Determine relevance using LLM (Claude/GPT) |
| **Link Enricher** | Use Firecrawl to extract content from linked URLs |
| **Storage** | Write markdown files with YAML frontmatter |

---

## 3. Technical Design

### 3.1 Crate Structure

```
crates/research/
├── Cargo.toml
├── clippy.toml
├── README.md
├── src/
│   ├── lib.rs                  # Public API exports
│   ├── main.rs                 # CLI binary
│   │
│   ├── auth/
│   │   ├── mod.rs
│   │   ├── browser.rs          # Playwright/headless browser auth
│   │   ├── session.rs          # Session management/cookies
│   │   └── credentials.rs      # Secure credential storage
│   │
│   ├── twitter/
│   │   ├── mod.rs
│   │   ├── poller.rs           # Bookmark polling logic
│   │   ├── parser.rs           # Tweet content parsing
│   │   └── types.rs            # Tweet/Bookmark structs
│   │
│   ├── analysis/
│   │   ├── mod.rs
│   │   ├── relevance.rs        # Relevance scoring
│   │   ├── prompts.rs          # LLM prompt templates
│   │   └── categories.rs       # Topic categorization
│   │
│   ├── enrichment/
│   │   ├── mod.rs
│   │   ├── firecrawl.rs        # Firecrawl API client
│   │   ├── scraper.rs          # Content extraction
│   │   └── links.rs            # URL detection/processing
│   │
│   └── storage/
│       ├── mod.rs
│       ├── markdown.rs         # Markdown generation
│       ├── index.rs            # Research index/catalog
│       └── dedup.rs            # Duplicate detection
│
└── prompts/
    ├── relevance.hbs           # Relevance analysis prompt
    ├── categorize.hbs          # Topic categorization prompt
    └── summarize.hbs           # Content summary prompt
```

### 3.2 Core Types

```rust
/// A bookmarked tweet with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    /// Unique tweet ID
    pub id: String,
    /// Tweet author
    pub author: Author,
    /// Tweet text content
    pub text: String,
    /// When the tweet was posted
    pub posted_at: DateTime<Utc>,
    /// When it was bookmarked
    pub bookmarked_at: DateTime<Utc>,
    /// Attached media
    pub media: Vec<Media>,
    /// URLs found in the tweet
    pub urls: Vec<String>,
    /// Quote tweet if present
    pub quote: Option<Box<Bookmark>>,
    /// Thread context (if part of thread)
    pub thread: Option<Vec<String>>,
}

/// Author information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub handle: String,
    pub name: String,
    pub verified: bool,
    pub bio: Option<String>,
}

/// Relevance analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelevanceResult {
    /// Is this relevant to the platform? (0.0-1.0)
    pub score: f32,
    /// Reasoning for the score
    pub reasoning: String,
    /// Detected categories
    pub categories: Vec<Category>,
    /// Key topics/technologies mentioned
    pub topics: Vec<String>,
    /// Should we enrich with linked content?
    pub should_enrich: bool,
}

/// Content category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Category {
    /// Agent/AI development patterns
    Agents,
    /// Rust ecosystem updates
    Rust,
    /// Kubernetes/infrastructure
    Infrastructure,
    /// MCP/tool integrations
    Tooling,
    /// Software architecture
    Architecture,
    /// DevOps/CI-CD
    DevOps,
    /// Security practices
    Security,
    /// Research papers/academic
    Research,
    /// Product launches/announcements
    Announcements,
    /// Other/general
    Other,
}

/// Enriched research entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchEntry {
    /// Original bookmark
    pub bookmark: Bookmark,
    /// Relevance analysis
    pub relevance: RelevanceResult,
    /// Enriched content from links
    pub enriched: Vec<EnrichedLink>,
    /// Generated summary
    pub summary: String,
    /// Tags for search
    pub tags: Vec<String>,
    /// When this was processed
    pub processed_at: DateTime<Utc>,
}

/// Enriched link content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedLink {
    /// Original URL
    pub url: String,
    /// Resolved title
    pub title: Option<String>,
    /// Markdown content
    pub content: String,
    /// Key excerpts
    pub excerpts: Vec<String>,
}
```

### 3.3 Authentication Strategy

Since the Twitter API has become expensive/restrictive, we'll use browser automation:

**Option A: Playwright-based (Recommended)**
```rust
/// Browser-based authentication using headless browser
pub struct BrowserAuth {
    /// Browser instance
    browser: Browser,
    /// Persistent session context
    context: BrowserContext,
    /// Session cookies
    cookies: Vec<Cookie>,
}

impl BrowserAuth {
    /// Initialize with stored credentials
    pub async fn new(config: &AuthConfig) -> Result<Self>;
    
    /// Login if needed (handles 2FA prompt)
    pub async fn ensure_logged_in(&mut self) -> Result<()>;
    
    /// Get authenticated page for scraping
    pub async fn get_page(&self) -> Result<Page>;
    
    /// Save session for reuse
    pub async fn save_session(&self) -> Result<()>;
}
```

**Option B: Cookie-based Session**
```rust
/// Reuse exported browser session cookies
pub struct CookieAuth {
    cookies: CookieJar,
    client: reqwest::Client,
}
```

**Recommendation**: Start with cookie-based auth (simpler), with Playwright as fallback for re-authentication.

### 3.4 Polling Strategy

```rust
/// Bookmark poller with rate limiting
pub struct BookmarkPoller {
    auth: Arc<dyn Auth>,
    config: PollConfig,
    state: PollState,
}

#[derive(Debug, Clone)]
pub struct PollConfig {
    /// Interval between polls (default: 5 minutes)
    pub interval: Duration,
    /// Max bookmarks to process per poll
    pub batch_size: usize,
    /// Backoff multiplier on errors
    pub backoff_multiplier: f32,
    /// Max backoff duration
    pub max_backoff: Duration,
}

impl Default for PollConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(5 * 60), // 5 minutes
            batch_size: 10,
            backoff_multiplier: 2.0,
            max_backoff: Duration::from_secs(30 * 60), // 30 minutes
        }
    }
}

/// Tracks polling state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollState {
    /// Last successful poll timestamp
    pub last_poll: Option<DateTime<Utc>>,
    /// Last seen bookmark ID (for incremental fetching)
    pub cursor: Option<String>,
    /// Consecutive failure count
    pub failures: u32,
    /// IDs of already processed bookmarks
    pub processed: HashSet<String>,
}
```

### 3.5 AI Analysis Integration

Reuse the existing `tasks` crate AI provider abstraction:

```rust
use tasks::ai::{AIProvider, AIMessage, GenerateOptions, parse_ai_response};

/// Relevance analyzer using LLM
pub struct RelevanceAnalyzer {
    provider: Arc<dyn AIProvider>,
    prompts: PromptManager,
}

impl RelevanceAnalyzer {
    /// Analyze bookmark for relevance to CTO platform
    pub async fn analyze(&self, bookmark: &Bookmark) -> Result<RelevanceResult> {
        let prompt = self.prompts.render("relevance", &json!({
            "tweet_text": bookmark.text,
            "author": bookmark.author.handle,
            "urls": bookmark.urls,
            "context": PLATFORM_CONTEXT,
        }))?;
        
        let messages = vec![
            AIMessage::system(ANALYSIS_SYSTEM_PROMPT),
            AIMessage::user(prompt),
        ];
        
        let options = GenerateOptions {
            temperature: Some(0.3),
            max_tokens: Some(1000),
            json_mode: true,
            ..Default::default()
        };
        
        let response = self.provider.generate_text("claude-sonnet-4-20250514", &messages, &options).await?;
        parse_ai_response(&response)
    }
}
```

**Relevance Prompt (relevance.hbs)**:
```handlebars
Analyze this tweet for relevance to a multi-agent software development platform.

## Platform Context
The CTO platform orchestrates AI coding agents (Rex, Blaze, Tess, etc.) for automated
software development, using Kubernetes, Argo Workflows, MCP tools, and Rust.

Key areas of interest:
- AI/LLM agent development patterns
- MCP (Model Context Protocol) tools and integrations  
- Kubernetes operators and orchestration
- Rust best practices and ecosystem
- GitOps and CI/CD automation
- Software architecture patterns
- Developer tooling innovations

## Tweet to Analyze
Author: @{{author}}
Content: {{tweet_text}}
{{#if urls}}
Links: {{#each urls}}
- {{this}}
{{/each}}
{{/if}}

## Task
Respond with JSON:
{
  "score": <0.0-1.0 relevance score>,
  "reasoning": "<why this is/isn't relevant>",
  "categories": ["<matching categories>"],
  "topics": ["<key topics/technologies>"],
  "should_enrich": <true if links should be scraped>
}
```

### 3.6 Link Enrichment with Firecrawl

Leverage existing Firecrawl patterns from `crates/mcp/src/doc_proxy.rs`:

```rust
use crate::enrichment::firecrawl::{FirecrawlClient, ScrapeOptions};

/// Enrich research entry with linked content
pub struct LinkEnricher {
    client: FirecrawlClient,
    config: EnrichConfig,
}

#[derive(Debug, Clone)]
pub struct EnrichConfig {
    /// Max links to enrich per tweet
    pub max_links: usize,
    /// Timeout per scrape
    pub timeout: Duration,
    /// Content length limit
    pub max_content_length: usize,
}

impl LinkEnricher {
    /// Enrich bookmark with scraped link content
    pub async fn enrich(&self, bookmark: &Bookmark) -> Result<Vec<EnrichedLink>> {
        let mut enriched = Vec::new();
        
        for url in bookmark.urls.iter().take(self.config.max_links) {
            // Skip Twitter internal links
            if url.contains("twitter.com") || url.contains("x.com") {
                continue;
            }
            
            match self.scrape_url(url).await {
                Ok(content) => enriched.push(content),
                Err(e) => {
                    tracing::warn!("Failed to enrich {url}: {e}");
                }
            }
        }
        
        Ok(enriched)
    }
    
    async fn scrape_url(&self, url: &str) -> Result<EnrichedLink> {
        let response = self.client.scrape(url, ScrapeOptions {
            formats: vec!["markdown"],
            only_main_content: true,
            timeout: self.config.timeout,
        }).await?;
        
        Ok(EnrichedLink {
            url: url.to_string(),
            title: response.metadata.and_then(|m| m.title),
            content: truncate(&response.markdown, self.config.max_content_length),
            excerpts: extract_key_excerpts(&response.markdown),
        })
    }
}
```

### 3.7 Storage Format

Store research as markdown files with YAML frontmatter:

```
research/
├── index.json                    # Catalog of all entries
├── 2025/
│   └── 12/
│       └── 05/
│           ├── 1865432109.md     # Tweet ID as filename
│           └── 1865432110.md
└── by-topic/
    ├── agents/
    │   └── 1865432109.md         # Symlinks for browsing
    └── rust/
        └── 1865432110.md
```

**Markdown Format**:
```markdown
---
id: "1865432109"
author: "@karpathy"
posted_at: "2025-12-05T14:30:00Z"
bookmarked_at: "2025-12-05T15:00:00Z"
processed_at: "2025-12-05T15:05:00Z"
relevance_score: 0.92
categories:
  - agents
  - research
topics:
  - llm
  - reasoning
  - chain-of-thought
tags:
  - ai-agents
  - research-paper
  - reasoning
---

# Research: LLM Reasoning Patterns

## Original Tweet

> New paper on chain-of-thought reasoning in LLMs. Key finding: structured 
> decomposition beats free-form reasoning by 15% on complex tasks.
> 
> Paper: https://arxiv.org/abs/2025.12345

**Author**: @karpathy (Andrej Karpathy)  
**Posted**: December 5, 2025 at 2:30 PM

## Analysis

This tweet discusses advancements in LLM reasoning that could improve our agent 
decision-making. The structured decomposition approach aligns with our 
task-oriented workflow design.

**Relevance**: High (0.92)

## Enriched Content

### Paper: Structured Decomposition for LLM Reasoning

[Content from arXiv page...]

## Key Takeaways

1. Structured task decomposition improves reasoning accuracy
2. Applicable to complex multi-step workflows
3. Could enhance Rex's code analysis capabilities

---

*Curated by CTO Research Pipeline*
```

### 3.8 CLI Interface

```rust
/// Research crate CLI commands
#[derive(Parser)]
#[command(name = "research")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the research monitor daemon
    Watch {
        /// Poll interval in seconds
        #[arg(long, default_value = "300")]
        interval: u64,
        
        /// Output directory for research
        #[arg(long, default_value = "research")]
        output: PathBuf,
        
        /// AI provider (anthropic, openai)
        #[arg(long, default_value = "anthropic")]
        provider: String,
        
        /// Model to use for analysis
        #[arg(long, default_value = "claude-sonnet-4-20250514")]
        model: String,
        
        /// Minimum relevance score (0.0-1.0)
        #[arg(long, default_value = "0.5")]
        min_relevance: f32,
    },
    
    /// Manually process a specific tweet URL
    Process {
        /// Tweet URL to process
        url: String,
        
        /// Force processing even if already seen
        #[arg(long)]
        force: bool,
    },
    
    /// List recent research entries
    List {
        /// Filter by category
        #[arg(long)]
        category: Option<Category>,
        
        /// Limit results
        #[arg(long, default_value = "20")]
        limit: usize,
    },
    
    /// Search research entries
    Search {
        /// Search query
        query: String,
    },
    
    /// Authenticate with Twitter
    Auth {
        /// Re-authenticate even if session exists
        #[arg(long)]
        force: bool,
    },
}
```

---

## 4. Kubernetes Deployment

### 4.1 Architecture: CronJob-Based

Instead of a long-running daemon, we'll use a Kubernetes CronJob that runs every 5 minutes. This is cleaner because:
- No persistent process to manage
- Automatic restart on failure
- Built-in scheduling via Kubernetes
- Plays well with GitOps (ArgoCD)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         Kubernetes Cluster                               │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
        ┌───────────────────────────┼───────────────────────────┐
        │                           │                           │
        ▼                           ▼                           ▼
┌───────────────┐          ┌───────────────┐          ┌───────────────┐
│    Vault      │          │   CronJob     │          │  PVC: research│
│  (Secrets)    │─────────▶│  (5 min)      │─────────▶│  (storage)    │
└───────────────┘          └───────────────┘          └───────────────┘
        │                           │
        │                           ▼
        │                  ┌───────────────┐
        │                  │   research    │
        │                  │   container   │
        │                  │   (headless)  │
        │                  └───────────────┘
        │                           │
        └───────────────────────────┤
          Secrets mounted:          │
          - TWITTER_AUTH_TOKEN      │
          - ANTHROPIC_API_KEY       ▼
          - FIRECRAWL_API_KEY  ┌───────────────┐
                               │  Firecrawl    │
                               │  (MCP/API)    │
                               └───────────────┘
```

### 4.2 Vault Secrets Configuration

Create a new secrets file for research:

```yaml
# infra/vault/secrets/research.yaml
---
# Research Crate Secrets - cto namespace
# 
# REQUIRED: Populate these secrets in Vault:
#
# 1. Twitter session cookies (get from browser DevTools after login):
#    vault kv put secret/research-twitter \
#      TWITTER_AUTH_TOKEN=<auth_token cookie value> \
#      TWITTER_CT0=<ct0 cookie value>
#
# 2. AI Provider API key (Anthropic recommended):
#    vault kv put secret/research-ai ANTHROPIC_API_KEY=<your-api-key>
#
# 3. Firecrawl (reuse from tools if available):
#    vault kv put secret/research-firecrawl FIRECRAWL_API_KEY=<your-api-key>
#
# Note: auth_token is long-lived (months/years) until revoked.
# ct0 regenerates automatically from auth_token.
#
---
apiVersion: secrets.hashicorp.com/v1beta1
kind: VaultStaticSecret
metadata:
  name: research-twitter-secrets
  namespace: cto
  labels:
    app.kubernetes.io/name: research-twitter-secrets
    app.kubernetes.io/part-of: platform
spec:
  vaultAuthRef: infra/vault-auth
  mount: secret
  path: research-twitter
  type: kv-v2
  refreshAfter: 30s
  destination:
    create: true
    name: research-twitter-secrets

---
apiVersion: secrets.hashicorp.com/v1beta1
kind: VaultStaticSecret
metadata:
  name: research-ai-secrets
  namespace: cto
  labels:
    app.kubernetes.io/name: research-ai-secrets
    app.kubernetes.io/part-of: platform
spec:
  vaultAuthRef: infra/vault-auth
  mount: secret
  path: research-ai
  type: kv-v2
  refreshAfter: 30s
  destination:
    create: true
    name: research-ai-secrets

---
apiVersion: secrets.hashicorp.com/v1beta1
kind: VaultStaticSecret
metadata:
  name: research-firecrawl-secrets
  namespace: cto
  labels:
    app.kubernetes.io/name: research-firecrawl-secrets
    app.kubernetes.io/part-of: platform
spec:
  vaultAuthRef: infra/vault-auth
  mount: secret
  path: research-firecrawl
  type: kv-v2
  refreshAfter: 30s
  destination:
    create: true
    name: research-firecrawl-secrets
```

### 4.3 CronJob Manifest

```yaml
# infra/research/research-cronjob.yaml
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: research-storage
  namespace: cto
  labels:
    app.kubernetes.io/name: research-storage
    app.kubernetes.io/part-of: platform
spec:
  accessModes:
    - ReadWriteOnce
  storageClassName: local-path
  resources:
    requests:
      storage: 5Gi

---
apiVersion: v1
kind: ServiceAccount
metadata:
  name: research-poller
  namespace: cto
  labels:
    app.kubernetes.io/name: research-poller
    app.kubernetes.io/part-of: platform

---
apiVersion: batch/v1
kind: CronJob
metadata:
  name: research-poller
  namespace: cto
  labels:
    app.kubernetes.io/name: research-poller
    app.kubernetes.io/part-of: platform
spec:
  # Run every 5 minutes
  schedule: "*/5 * * * *"
  concurrencyPolicy: Forbid  # Don't overlap runs
  failedJobsHistoryLimit: 3
  successfulJobsHistoryLimit: 1
  startingDeadlineSeconds: 300
  jobTemplate:
    spec:
      activeDeadlineSeconds: 240  # 4 min max runtime (leave buffer)
      template:
        metadata:
          labels:
            app.kubernetes.io/name: research-poller
        spec:
          serviceAccountName: research-poller
          securityContext:
            runAsNonRoot: true
            runAsUser: 1001
            runAsGroup: 1001
            fsGroup: 1001
            seccompProfile:
              type: RuntimeDefault
          restartPolicy: OnFailure
          
          # Init container to ensure Chrome is available
          initContainers:
            - name: browser-check
              image: ghcr.io/5dlabs/research:latest
              command: ["chromium", "--version"]
              securityContext:
                allowPrivilegeEscalation: false
                capabilities:
                  drop: ["ALL"]
          
          containers:
            - name: research
              image: ghcr.io/5dlabs/research:latest
              imagePullPolicy: Always
              securityContext:
                allowPrivilegeEscalation: false
                capabilities:
                  drop: ["ALL"]
              
              command: ["research"]
              args:
                - "poll"
                - "--output=/data/research"
                - "--state=/data/state.json"
                - "--min-relevance=0.5"
              
              env:
                # Twitter auth from Vault
                - name: TWITTER_AUTH_TOKEN
                  valueFrom:
                    secretKeyRef:
                      name: research-twitter-secrets
                      key: TWITTER_AUTH_TOKEN
                - name: TWITTER_CT0
                  valueFrom:
                    secretKeyRef:
                      name: research-twitter-secrets
                      key: TWITTER_CT0
                      optional: true  # Auto-regenerates
                
                # AI provider
                - name: ANTHROPIC_API_KEY
                  valueFrom:
                    secretKeyRef:
                      name: research-ai-secrets
                      key: ANTHROPIC_API_KEY
                
                # Firecrawl for link enrichment
                - name: FIRECRAWL_API_KEY
                  valueFrom:
                    secretKeyRef:
                      name: research-firecrawl-secrets
                      key: FIRECRAWL_API_KEY
                
                # Chrome flags for headless
                - name: CHROME_FLAGS
                  value: "--headless --no-sandbox --disable-gpu --disable-dev-shm-usage"
              
              volumeMounts:
                - name: research-data
                  mountPath: /data
              
              resources:
                requests:
                  memory: "512Mi"
                  cpu: "250m"
                limits:
                  memory: "1Gi"
                  cpu: "1000m"
          
          volumes:
            - name: research-data
              persistentVolumeClaim:
                claimName: research-storage
```

### 4.4 Container Image (Dockerfile)

```dockerfile
# infra/images/research/Dockerfile
FROM rust:1.83-bookworm AS builder

WORKDIR /app
COPY . .
RUN cargo build --release -p research

# Runtime image with Chrome
FROM debian:bookworm-slim

# Install Chromium and dependencies
RUN apt-get update && apt-get install -y \
    chromium \
    ca-certificates \
    fonts-liberation \
    libasound2 \
    libatk-bridge2.0-0 \
    libatk1.0-0 \
    libcups2 \
    libdbus-1-3 \
    libdrm2 \
    libgbm1 \
    libgtk-3-0 \
    libnspr4 \
    libnss3 \
    libxcomposite1 \
    libxdamage1 \
    libxfixes3 \
    libxkbcommon0 \
    libxrandr2 \
    xdg-utils \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -g 1001 research && \
    useradd -u 1001 -g research -s /bin/bash -m research

# Copy binary
COPY --from=builder /app/target/release/research /usr/local/bin/research

# Set up data directory
RUN mkdir -p /data && chown research:research /data

USER research
WORKDIR /data

ENV CHROME_BIN=/usr/bin/chromium
ENV CHROME_FLAGS="--headless --no-sandbox --disable-gpu --disable-dev-shm-usage"

ENTRYPOINT ["research"]
CMD ["--help"]
```

### 4.5 One-Time Auth Setup

Since the CronJob runs headless, we need a one-time interactive setup:

```bash
#!/bin/bash
# scripts/research-auth.sh
# Run this locally to export Twitter cookies to Vault

set -euo pipefail

echo "🔐 Research Crate - Twitter Authentication Setup"
echo ""
echo "This will:"
echo "1. Open a browser for you to log into Twitter/X"
echo "2. Extract the auth_token cookie"
echo "3. Store it in Vault for the CronJob to use"
echo ""

# Run the auth command locally (requires display)
cargo run -p research -- auth --export-to-vault

echo ""
echo "✅ Authentication complete!"
echo "   The CronJob will now be able to poll your bookmarks."
```

**Alternative: Manual cookie extraction**

```bash
# If browser automation doesn't work, manually extract:
# 1. Log into x.com in your browser
# 2. Open DevTools > Application > Cookies > x.com
# 3. Copy auth_token value
# 4. Store in Vault:

vault kv put secret/research-twitter \
  TWITTER_AUTH_TOKEN="your-auth-token-here"
```

### 4.6 CLI Changes for Container Mode

The CLI needs a `poll` subcommand (single-run, no loop):

```rust
#[derive(Subcommand)]
pub enum Commands {
    /// Run a single poll cycle (for CronJob use)
    Poll {
        /// Output directory for research
        #[arg(long, default_value = "/data/research")]
        output: PathBuf,
        
        /// State file path (tracks processed bookmarks)
        #[arg(long, default_value = "/data/state.json")]
        state: PathBuf,
        
        /// Minimum relevance score
        #[arg(long, default_value = "0.5")]
        min_relevance: f32,
        
        /// Max bookmarks to process per run
        #[arg(long, default_value = "10")]
        batch_size: usize,
    },
    
    /// Interactive auth setup (run locally, not in container)
    Auth {
        /// Export cookies directly to Vault
        #[arg(long)]
        export_to_vault: bool,
        
        /// Output file for cookies (if not using Vault)
        #[arg(long)]
        output: Option<PathBuf>,
    },
    
    // ... other commands unchanged
}
```

---

## 5. Configuration

### 5.1 Config File (research-config.json)

```json
{
  "twitter": {
    "session_path": "~/.config/cto/twitter-session.json",
    "poll_interval_secs": 300,
    "batch_size": 10,
    "max_backoff_secs": 1800
  },
  "analysis": {
    "provider": "anthropic",
    "model": "claude-sonnet-4-20250514",
    "min_relevance_score": 0.5,
    "temperature": 0.3
  },
  "enrichment": {
    "enabled": true,
    "max_links_per_tweet": 3,
    "timeout_secs": 30,
    "max_content_length": 50000
  },
  "storage": {
    "output_dir": "./research",
    "organize_by_date": true,
    "create_topic_symlinks": true
  },
  "categories_of_interest": [
    "agents",
    "rust",
    "infrastructure",
    "tooling",
    "architecture"
  ]
}
```

### 5.2 Environment Variables (Container Mode)

All secrets are injected from Vault via Kubernetes secrets:

| Variable | Source | Description |
|----------|--------|-------------|
| `TWITTER_AUTH_TOKEN` | `research-twitter-secrets` | Long-lived auth cookie (required) |
| `TWITTER_CT0` | `research-twitter-secrets` | CSRF token (optional, auto-regenerates) |
| `ANTHROPIC_API_KEY` | `research-ai-secrets` | Claude API key for analysis |
| `FIRECRAWL_API_KEY` | `research-firecrawl-secrets` | For link enrichment |
| `CHROME_BIN` | Container default | Path to Chromium binary |
| `CHROME_FLAGS` | Container default | Headless mode flags |

---

## 6. Dependencies

### 6.1 New Dependencies

```toml
[dependencies]
# Browser automation
chromiumoxide = "0.7"
tokio = { version = "1.40", features = ["full"] }

# Existing workspace deps
serde = { workspace = true }
serde_json = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
chrono = { workspace = true }
reqwest = { workspace = true }
handlebars = { workspace = true }
clap = { workspace = true }

# Local crates
tasks = { path = "../tasks" }  # For AI provider abstraction
```

### 6.2 Optional: rettiwt-api Alternative

If browser automation proves unreliable, consider the `rettiwt-api` approach:

```toml
# Alternative: Use rettiwt-api Node.js bridge
# Requires: npx rettiwt-api server
```

---

## 7. Security Considerations

1. **Credential Storage**: Never log or expose Twitter credentials
2. **Session Security**: Encrypt stored session cookies at rest
3. **Rate Limiting**: Implement conservative backoff to avoid account flags
4. **API Keys**: Store Firecrawl/LLM keys in environment or secure vault
5. **Content Filtering**: Don't store sensitive/private content

---

## 8. Testing Strategy

### 7.1 Unit Tests

- Tweet parsing from HTML/JSON fixtures
- Relevance score calculation
- Markdown generation
- Category detection

### 7.2 Integration Tests

- Mock Twitter responses for poller
- Mock Firecrawl responses for enricher
- Full pipeline with fixture data

### 7.3 Manual Testing

- Initial auth flow with real credentials
- Live bookmark monitoring
- Link enrichment validation

---

## 9. Implementation Phases

### Phase 0: Smoke Test (Day 1-2)
- [ ] Run smoke tests to choose browser automation approach
- [ ] Test cookie persistence and Twitter auth
- [ ] Validate headless mode works in container

### Phase 1: Foundation (Week 1)
- [ ] Create crate structure with workspace integration
- [ ] Define core types (Bookmark, Author, etc.)
- [ ] Implement cookie-based auth with Vault integration
- [ ] Basic bookmark parsing from HTML

### Phase 2: Analysis (Week 2)
- [ ] Integrate AI provider from tasks crate
- [ ] Create relevance/categorization prompts
- [ ] Implement markdown storage format
- [ ] Add state tracking (processed bookmarks)

### Phase 3: Enrichment (Week 3)
- [ ] Firecrawl integration for link scraping
- [ ] Content summarization
- [ ] Full pipeline integration (poll → analyze → enrich → store)

### Phase 4: Kubernetes Deployment (Week 4)
- [ ] Dockerfile with Chromium
- [ ] CronJob manifest
- [ ] Vault secrets configuration
- [ ] ArgoCD Application manifest
- [ ] PVC for research storage

### Phase 5: Polish (Week 5)
- [ ] One-time auth script (`research auth`)
- [ ] Error handling and alerting
- [ ] Documentation and runbook

---

## 10. Open Questions

1. ~~**Authentication Robustness**: How often will Twitter invalidate sessions?~~
   - **RESOLVED**: `auth_token` is long-lived (months/years), only invalidated on password change or explicit revocation

2. ~~**Rate Limits**: What's the safe polling frequency?~~
   - **RESOLVED**: 5-minute CronJob schedule, batch limited to 10 bookmarks per run

3. ~~**Model Selection**: Claude vs GPT for analysis?~~
   - **RESOLVED**: Start with Claude via existing tasks crate AI provider

4. **Content Volume**: How many bookmarks/day expected?
   - Design for 50/day max, with dedup via state file

5. **Search/Retrieval**: Local index vs vector DB?
   - Start simple with JSON index, consider embeddings later

6. ~~**Browser Automation**: Playwright-Rust vs Chromiumoxide?~~
   - **PENDING**: Run smoke tests to determine (see Phase 0)

7. ~~**Deployment**: Daemon vs CronJob?~~
   - **RESOLVED**: Kubernetes CronJob every 5 minutes

8. ~~**Notifications**: Alert on high-relevance finds?~~
   - **DEFERRED**: Stretch goal, monitor manually for now

---

## 11. Success Metrics

- **Coverage**: % of bookmarks successfully processed
- **Accuracy**: Manual spot-check of relevance scoring
- **Enrichment**: % of links successfully scraped
- **Latency**: Time from bookmark to stored research
- **Uptime**: Poller availability (should run 24/7)

---

## 12. Smoke Test Plan

Before committing to a browser automation approach, we'll build a minimal smoke test to compare options.

### 11.1 Twitter/X Token Validity (Research Findings)

**Key insight from research:**
- `auth_token` cookie is **long-lived** (persists until explicit revocation or password change)
- `ct0` cookie is short-lived (~6 hours) but **auto-regenerates** when you have a valid `auth_token`
- Access tokens don't expire unless you explicitly revoke them in Twitter settings

**Implication**: We only need to authenticate once manually, save the `auth_token` cookie, and it should work indefinitely. The app can detect when re-auth is needed (401/403 responses) and prompt.

### 11.2 Automation Options Comparison

| Feature | Playwright-Rust | Chromiumoxide | Cookie-Only (reqwest) |
|---------|-----------------|---------------|----------------------|
| **Language** | Rust (wraps Node.js) | Pure Rust | Pure Rust |
| **Dependency** | Requires Node.js | Chrome/Chromium binary | None (HTTP only) |
| **Cookie API** | `context.add_cookies()` | Via DevTools Protocol | Native `CookieStore` |
| **Session Persist** | Built-in storage_state | Manual cookie export | Manual JSON file |
| **Anti-bot Detection** | Best (full browser) | Good (full browser) | Poor (may get blocked) |
| **Complexity** | Medium | Medium-High | Low |
| **Benchmark Score** | 87.9 | High reputation | N/A |

### 11.3 Smoke Test Structure

```
crates/research/
└── examples/
    ├── smoke_playwright.rs    # Playwright-rust approach
    ├── smoke_chromiumoxide.rs # Pure Rust approach  
    └── smoke_cookies.rs       # Cookie-only approach (baseline)
```

### 11.4 Test Cases

Each smoke test will attempt:

```rust
/// Smoke test interface
trait SmokeTest {
    /// 1. Load saved session (if exists)
    async fn load_session(&self) -> Result<Option<Session>>;
    
    /// 2. Check if session is still valid
    async fn validate_session(&self) -> Result<bool>;
    
    /// 3. If invalid, prompt for manual login (one-time)
    async fn manual_login(&self) -> Result<Session>;
    
    /// 4. Navigate to bookmarks page
    async fn fetch_bookmarks_page(&self) -> Result<String>;
    
    /// 5. Parse first 5 bookmarks
    async fn parse_bookmarks(&self, html: &str) -> Result<Vec<Bookmark>>;
    
    /// 6. Save session for reuse
    async fn save_session(&self) -> Result<()>;
}
```

### 11.5 Smoke Test: Playwright-Rust

```rust
// examples/smoke_playwright.rs
use playwright::Playwright;
use playwright::api::Cookie;
use std::path::Path;

const SESSION_FILE: &str = ".twitter-session.json";
const BOOKMARKS_URL: &str = "https://x.com/i/bookmarks";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎭 Playwright-Rust Smoke Test\n");
    
    let playwright = Playwright::initialize().await?;
    playwright.prepare()?;
    
    let chromium = playwright.chromium();
    let browser = chromium.launcher()
        .headless(false) // Show browser for manual login
        .launch()
        .await?;
    
    let context = browser.context_builder()
        .user_agent(Some("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36"))
        .build()
        .await?;
    
    // Try to load existing session
    if Path::new(SESSION_FILE).exists() {
        println!("📂 Loading saved session...");
        let cookies: Vec<Cookie> = serde_json::from_str(
            &std::fs::read_to_string(SESSION_FILE)?
        )?;
        context.add_cookies(&cookies).await?;
    }
    
    let page = context.new_page().await?;
    
    // Navigate to bookmarks
    println!("🔖 Navigating to bookmarks...");
    page.goto_builder(BOOKMARKS_URL).goto().await?;
    
    // Check if we landed on login page
    let url = page.url()?;
    if url.contains("login") || url.contains("flow") {
        println!("⚠️  Session expired - manual login required");
        println!("   Please log in to Twitter in the browser window...");
        println!("   Press Enter when done.");
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        // Navigate back to bookmarks after login
        page.goto_builder(BOOKMARKS_URL).goto().await?;
    }
    
    // Verify we're on bookmarks page
    let title = page.title().await?;
    println!("📄 Page title: {}", title);
    
    // Get HTML content
    let html = page.content().await?;
    println!("📝 Page content length: {} bytes", html.len());
    
    // Save session for next time
    println!("💾 Saving session...");
    let cookies = context.cookies(&["https://x.com".to_string()]).await?;
    std::fs::write(SESSION_FILE, serde_json::to_string_pretty(&cookies)?)?;
    
    // Parse bookmarks (basic check)
    let bookmark_count = html.matches("data-testid=\"tweet\"").count();
    println!("✅ Found {} tweets on bookmarks page", bookmark_count);
    
    browser.close().await?;
    
    println!("\n🎉 Playwright smoke test complete!");
    Ok(())
}
```

### 11.6 Smoke Test: Chromiumoxide

```rust
// examples/smoke_chromiumoxide.rs
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::network::CookieParam;
use futures::StreamExt;

const SESSION_FILE: &str = ".twitter-session-chromium.json";
const BOOKMARKS_URL: &str = "https://x.com/i/bookmarks";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔷 Chromiumoxide Smoke Test\n");
    
    let (mut browser, mut handler) = Browser::launch(
        BrowserConfig::builder()
            .with_head() // Show browser for manual login
            .build()?
    ).await?;
    
    // Spawn handler task
    let handle = tokio::spawn(async move {
        while let Some(h) = handler.next().await {
            if h.is_err() { break; }
        }
    });
    
    let page = browser.new_page(BOOKMARKS_URL).await?;
    
    // Wait for page load
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    
    // Check current URL
    let url = page.url().await?.unwrap_or_default();
    println!("📍 Current URL: {}", url);
    
    if url.contains("login") || url.contains("flow") {
        println!("⚠️  Session expired - manual login required");
        println!("   Please log in to Twitter in the browser window...");
        println!("   Press Enter when done.");
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        // Navigate to bookmarks
        page.goto(BOOKMARKS_URL).await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
    
    // Get page content
    let html = page.content().await?;
    println!("📝 Page content length: {} bytes", html.len());
    
    // Count tweets
    let bookmark_count = html.matches("data-testid=\"tweet\"").count();
    println!("✅ Found {} tweets on bookmarks page", bookmark_count);
    
    browser.close().await?;
    handle.await?;
    
    println!("\n🎉 Chromiumoxide smoke test complete!");
    Ok(())
}
```

### 11.7 Smoke Test: Cookie-Only (Baseline)

```rust
// examples/smoke_cookies.rs
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};

const SESSION_FILE: &str = ".twitter-cookies.json";
const BOOKMARKS_URL: &str = "https://x.com/i/bookmarks";

#[derive(serde::Deserialize, serde::Serialize)]
struct TwitterCookies {
    auth_token: String,
    ct0: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🍪 Cookie-Only Smoke Test (Baseline)\n");
    
    // Load cookies from file
    let cookies: TwitterCookies = match std::fs::read_to_string(SESSION_FILE) {
        Ok(s) => serde_json::from_str(&s)?,
        Err(_) => {
            println!("❌ No cookies file found at {}", SESSION_FILE);
            println!("   Create it with:");
            println!("   {{\"auth_token\": \"...\", \"ct0\": \"...\"}}");
            println!("\n   Get these from browser DevTools > Application > Cookies");
            return Ok(());
        }
    };
    
    println!("📂 Loaded cookies from {}", SESSION_FILE);
    
    let mut headers = HeaderMap::new();
    headers.insert(
        COOKIE, 
        HeaderValue::from_str(&format!(
            "auth_token={}; ct0={}", 
            cookies.auth_token, 
            cookies.ct0
        ))?
    );
    headers.insert(
        "x-csrf-token",
        HeaderValue::from_str(&cookies.ct0)?
    );
    headers.insert(
        "User-Agent",
        HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)")
    );
    
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;
    
    println!("🔖 Fetching bookmarks...");
    let response = client.get(BOOKMARKS_URL).send().await?;
    
    println!("📊 Status: {}", response.status());
    
    if response.status().is_success() {
        let html = response.text().await?;
        println!("📝 Response length: {} bytes", html.len());
        
        // Check if we got actual content vs redirect
        if html.contains("data-testid=\"tweet\"") {
            let count = html.matches("data-testid=\"tweet\"").count();
            println!("✅ Found {} tweets!", count);
        } else if html.contains("login") {
            println!("⚠️  Got redirected to login - cookies may be invalid");
        } else {
            println!("❓ Got response but no tweets found - may need JS rendering");
        }
    } else {
        println!("❌ Request failed");
    }
    
    println!("\n🎉 Cookie-only smoke test complete!");
    Ok(())
}
```

### 11.8 Expected Results & Decision Matrix

| Approach | Expected Result | Choose If... |
|----------|-----------------|--------------|
| **Playwright-Rust** | ✅ Full functionality | Need reliable long-term solution, Node.js OK |
| **Chromiumoxide** | ✅ Full functionality | Want pure Rust, can manage Chrome binary |
| **Cookie-Only** | ⚠️ Partial (needs JS) | Bookmarks API is available without JS |

### 11.9 Running the Smoke Tests

```bash
# Add dependencies for smoke tests
cd crates/research

# Test 1: Playwright (requires Node.js)
cargo run --example smoke_playwright

# Test 2: Chromiumoxide (requires Chrome/Chromium)  
cargo run --example smoke_chromiumoxide

# Test 3: Cookie baseline (export cookies manually first)
cargo run --example smoke_cookies
```

### 11.10 One-Time Auth Flow

Since `auth_token` is long-lived, the auth flow will be:

```
┌─────────────────────────────────────────────────────────────────┐
│                     First Run / Session Expired                  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
              ┌───────────────────────────────┐
              │  Launch browser (non-headless) │
              │  Navigate to x.com/login       │
              └───────────────────────────────┘
                              │
                              ▼
              ┌───────────────────────────────┐
              │  Display prompt to terminal:   │
              │  "Log in to Twitter, then     │
              │   press Enter to continue"     │
              └───────────────────────────────┘
                              │
                              ▼ (user logs in manually)
              ┌───────────────────────────────┐
              │  Extract & save cookies:       │
              │  - auth_token (long-lived)     │
              │  - ct0 (will regenerate)       │
              └───────────────────────────────┘
                              │
                              ▼
              ┌───────────────────────────────┐
              │  Switch to headless mode       │
              │  for polling                   │
              └───────────────────────────────┘
```

**Session validity check** (run before each poll):
- Make lightweight request to `x.com` with saved cookies
- If 401/403 or redirect to login → trigger re-auth flow
- If 200 → session still valid, proceed with polling

---

## 13. Alternatives Considered

### A. Twitter API (v2)
- **Pros**: Official, stable, well-documented
- **Cons**: Expensive ($100+/month), limited features on free tier
- **Decision**: Rejected due to cost

### B. Nitter Instances
- **Pros**: No auth needed, RSS-like access
- **Cons**: No bookmark access, instances frequently go down
- **Decision**: Rejected - can't access bookmarks

### C. Browser Extension
- **Pros**: Real-time, no polling needed
- **Cons**: Requires browser to be open, not server-friendly
- **Decision**: Could be future enhancement for real-time

---

## 14. Future Enhancements

1. **Real-time Mode**: Browser extension for immediate capture
2. **Multi-account**: Support multiple Twitter accounts
3. **Vector Search**: Semantic search over research corpus
4. **GitHub Integration**: Auto-create issues for actionable research
5. **Slack/Discord**: Notify on high-relevance finds
6. **Topic Tracking**: Follow specific hashtags/users
7. **Research Reports**: Weekly digest generation

---

## Appendix A: Sample Prompts

### A.1 Categorization Prompt

```handlebars
Categorize this tweet content into one or more categories.

Categories:
- agents: AI/LLM agent patterns, autonomous systems
- rust: Rust language, crates, tooling
- infrastructure: Kubernetes, cloud, DevOps
- tooling: Developer tools, MCP, IDEs
- architecture: Software design patterns
- devops: CI/CD, automation, deployment
- security: Security practices, vulnerabilities
- research: Academic papers, studies
- announcements: Product launches, releases
- other: Doesn't fit other categories

Tweet: {{text}}

Respond with JSON array of matching categories.
```

### A.2 Summary Prompt

```handlebars
Summarize this research item for a technical audience.

## Original Tweet
{{tweet_text}}

{{#if enriched_content}}
## Linked Content
{{#each enriched_content}}
### {{this.title}}
{{this.excerpt}}
{{/each}}
{{/if}}

Write a 2-3 sentence summary highlighting key technical insights.
```

---

*Document Version: 1.0*

