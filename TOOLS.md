# Tools & Skills Reference

## MCP Server (In-Cluster)

Connected via `.mcp.json` → `http://cto-tools.cto.svc.cluster.local:3000/mcp`

150+ tools across 23 categories. Use `ToolSearch` to discover and load tools by keyword.

| Category | Pattern | Count | Purpose |
|----------|---------|-------|---------|
| linear | `linear_*` | 187 | Issues, projects, cycles, teams |
| grafana | `grafana_*` | 56 | Dashboards, alerts, Loki/Prometheus queries |
| github | `github_*` | 26 | PRs, issues, code, branches |
| playwright | `playwright_*` | 22 | Browser automation |
| argocd | `argocd_*` | 14 | GitOps sync, rollback, resources |
| octocode | `octocode_*` | 6 | GitHub code search, repo structure, PR archaeology |
| openmemory | `openmemory_*` | 6 | Persistent cross-session memory |
| loki | `loki_*` | 7 | Log search, patterns, correlations |
| tavily | `tavily_*` | 5 | Web search, crawl, extract, research |
| perplexity | `modelcontextprotocol_*` | 4 | AI search, reasoning, deep research |
| exa | `exa_*` | 2 | Web search, code context |
| terraform | `terraform_*` | 9 | Provider/module lookup |
| solana | `solana_*` | 3 | Anchor framework, docs, expert |
| context7 | `context7_*` | 2 | Library documentation (needs API key on server) |
| graphql | `graphql_*` | 2 | Schema introspection, queries |
| better_auth | `better_auth_*` | 4 | Auth docs search |
| pg_aiguide | `pg_aiguide_*` | 2 | Postgres/TimescaleDB docs |
| ai_elements | `ai_elements_*` | 2 | AI UI components |

Full catalog: `docs/tools-catalog.md`

## Installed Skills (`~/.agents/skills/`)

Skills are prompt-based capabilities loaded automatically by Claude Code and other agents.

### Web Scraping & Research
| Skill | What it does |
|-------|-------------|
| `firecrawl` | Main Firecrawl skill (scrape, crawl, map, search, agent, browser) |
| `firecrawl-scrape` | Scrape single URLs to markdown |
| `firecrawl-crawl` | Crawl multi-page sites |
| `firecrawl-map` | Map site URL structure |
| `firecrawl-search` | Web search via Firecrawl |
| `firecrawl-download` | Download full sites to `.firecrawl/` |
| `firecrawl-browser` | Cloud browser sandbox (Playwright) |
| `firecrawl-agent` | AI agent for autonomous web research |

### Documentation & Context
| Skill | What it does |
|-------|-------------|
| `context7` | Look up library/framework docs via Context7 |
| `context7-cli` | Context7 CLI integration |
| `context7-mcp` | Context7 MCP server setup |
| `find-docs` | Documentation research agent |

### Development
| Skill | What it does |
|-------|-------------|
| `shadcn` | shadcn/ui component management |
| `skill-finder` | Find and evaluate ClawHub skills |
| `tavily` | Web search via Tavily |
| `ultrathink` | Extended reasoning |
| `mgrep-code-search` | Code search across repos |
| `copilot-sdk` | GitHub Copilot SDK reference |

### Utilities
| Skill | What it does |
|-------|-------------|
| `markdown-converter` | Convert between formats |
| `youtube-transcript` | Extract YouTube transcripts |
| `beautiful-mermaid` | Generate Mermaid diagrams |
| `nano-banana-2` / `nano-banana-pro` | Image generation |
| `gpt-image-1-5` | GPT image generation |
| `promptify` | Prompt engineering |
| `ray-so-code-snippet` | Code snippet screenshots |
| `lorem-ipsum` | Placeholder text |
| `here-be-git` | Git workflows |
| `notion-api` / `todoist-api` / `raindrop-api` | Productivity integrations |
| `anki-connect` | Anki flashcard integration |
| `upstash-redis-kv` | Redis KV operations |
| `gog-cli` | GOG CLI |

### Trading

Two skill layers: **local** (executable scripts in `skills/trader/`) and **global** (prompt-based in `~/.agents/skills/`).

#### Local Execution Skills (`skills/trader/`)

Scripts run via `bun`/`node`/`python`/`bash`. These contain actual executable code.

**Execution**
| Skill | Chain | Entry Point | What it does |
|-------|-------|-------------|-------------|
| `moltiumv2` | Solana | `tools/moltium/local/ctl.mjs` | pump.fun, PumpSwap, Raydium AMM v4, autostrategy runtime |
| `jup-skill` | Solana | `scripts/fetch-api.ts` | Jupiter Ultra/Metis swaps, limit orders, DCA |
| `solana-connect` | Solana | `scripts/solana.js` | Secure wallet toolkit (generate, balance, send, txns) |
| `solana-sniper-bot` | Solana | `scripts/sniper.py` | Autonomous token sniper + LLM rug detection |
| `solana-copy-trader` | Solana | `scripts/index.js` | Whale copy trading via Helius WebSocket |
| `solana-funding-arb` | Solana | `scripts/` | Funding rate arb (Drift + Flash Trade) |
| `base-trader` | Base | SKILL.md | Bankr API trading |
| `polymarket-copytrading` | Polygon | `scripts/status.py` | Mirror top Polymarket traders via Simmer |
| `solana-swaps` / `solana-easy-swap` / `solana-transfer` | Solana | scripts | Jupiter swaps, transfers |
| `clawswap` | Solana | SKILL.md | ClawSwap perp futures |

**Safety & Analysis**
| Skill | Entry Point | What it does |
|-------|-------------|-------------|
| `prism-scanner` | `scripts/scan.sh {token}` | Rug-pull detection (holder concentration, LP locks, honeypots) |
| `fairscale-solana` | SKILL.md | Wallet reputation scoring (FairScore 0-100) |
| `crypto-price` | `scripts/` (Python) | Real-time prices + candlestick charts |
| `opendex` | SKILL.md | OpenDex API (token data, OHLCV, sentiment) |
| `technical-analyst` | SKILL.md | Systematic weekly chart analysis |
| `trading-research` | SKILL.md | Binance market data, whale tracker |

**Backtesting & Paper Trading**
| Skill | Entry Point | What it does |
|-------|-------------|-------------|
| `paper-trader` | `scripts/paper_trading.ts` | SQLite-backed paper trading with PnL |
| `hft-paper-trader` | SKILL.md | HF paper trading with Kelly criterion |
| `crypto-backtest` | SKILL.md | Multi-exchange backtesting (ccxt) |
| `crypto-self-learning` | SKILL.md | Self-improving trade analysis |
| `strategy-workflow` | SKILL.md | Full quant pipeline (Optuna, NautilusTrader) |
| `pair-trade-screener` | SKILL.md | Statistical arb / cointegration |

**Also**: `market-snapshot`, `crypto-market-analyzer`, `portfolio-watcher`, `questdb-agent-skill`, `solana-sniper-architect`

#### Global Trading Skills (`~/.agents/skills/`)

Prompt-based knowledge skills loaded automatically by Claude Code.

**Polymarket / Prediction Markets**
| Skill | Source | What it does |
|-------|--------|-------------|
| `grimoire-polymarket` | franalgaba/grimoire | Polymarket protocol reference (CLOB, CTF, order types) |
| `polymarket-api` | agentmc15 | Polymarket REST/WS API integration |
| `polymarket-prediction-market` | axwelbrand/arbibot | Polymarket arb strategies & bot architecture |
| `polymarket-copytrading` | spartanlabsxyz/simmer | Simmer SDK copy trading |
| `polymarket-signal-sniper` | spartanlabsxyz/simmer | Simmer signal-based sniping |
| `polymarket-mert-sniper` | spartanlabsxyz/simmer | MERT model sniping |
| `polymarket-ai-divergence` | spartanlabsxyz/simmer | AI model divergence detection |

**Solana DEX & Infrastructure**
| Skill | What it does |
|-------|-------------|
| `helius` | Helius RPC, webhooks, DAS API, priority fees |
| `drift` | Drift Protocol perp trading |
| `pumpfun` | pump.fun token creation & trading |
| `pump-fun-mechanics` | Bonding curve math, migration mechanics |
| `jupiter-swap-integration` | Jupiter v6 swap implementation patterns |
| `jito-bundles-and-priority-fees` | Jito bundle submission, tip optimization |
| `raydium` / `orca` / `meteora` | AMM pool mechanics |
| `solana-agent-kit` / `solana-kit` | Solana dev toolkits |
| `pyth` | Pyth oracle price feeds |

**Trading Strategy & Risk**
| Skill | What it does |
|-------|-------------|
| `trading-bot-architecture` | Bot design patterns (latency, state, recovery) |
| `sniper-dynamics-and-mitigation` | Sniper detection, anti-rug, MEV protection |
| `wallet-monitoring-bot` | Real-time wallet tracking bots |
| `meme-rush` | Binance memecoin momentum strategies |
| `bankr` | Bankr API (Base chain trading) |
| `whale-wallet-analysis` | Whale wallet tracking & copy patterns |
| `universal-trading` / `trade` | General trading frameworks |
| `trading-signal` / `trading-strategies` | Signal generation, strategy patterns |
| `rug-detection-checklist` | Pre-trade safety checklist |
| `token-analysis-checklist` / `token-research` | Token due diligence |
| `coingecko` | CoinGecko market data API |
| `dflow` | DFlow order flow |
| `metaplex` | NFT/token metadata |

### X Research (`apps/grok/`)

```bash
bun run ask "query"              # Open-ended X search via Grok
bun run ask --days 3 "query"     # Last N days
bun run cron                     # Keyword-based scheduled search
```

### Installing More Skills

```bash
# Search available skills
npx skills search <query>

# Install a skill globally
npx -y skills add <owner/repo@skill> --global --all -y
```

Browse: [context7.com/skills](https://context7.com/skills) | [skills.sh](https://skills.sh)

## CLI Tools

| CLI | Purpose |
|-----|---------|
| `firecrawl` | Web scraping CLI (authenticated, 3K credits/cycle) |
| `claude` | Claude Code |
| `codex` | OpenAI Codex |

### Firecrawl CLI Quick Reference

```bash
firecrawl scrape <url>              # Scrape URL to markdown
firecrawl search "query"            # Web search
firecrawl crawl <url>               # Crawl site
firecrawl map <url>                 # Map site URLs
firecrawl agent "prompt"            # AI research agent
firecrawl browser                   # Cloud browser
firecrawl --status                  # Check auth & credits
```

## Browser Automation

Two options available:
1. **Chrome MCP tools** (`mcp__claude-in-chrome__*`) — interact with active Chrome tabs
2. **Playwright MCP tools** (`playwright_*`) — headless browser via tools server

## API Keys (1Password)

| Service | 1Password Item | Vault | Used By |
|---------|---------------|-------|---------|
| Firecrawl | `Firecrawl API Key` | Automation | firecrawl CLI/skills |
| Context7 | `Context7 API Key` | Automation | context7 skill/MCP |
| Grok / xAI | `Grok X API Key` | Automation | `apps/grok/` |
| Helius | TBD | — | solana-copy-trader |
| Simmer | TBD | — | polymarket-copytrading |
| Jupiter | TBD | — | jup-skill |

Firecrawl is authenticated via OAuth (CLI). Context7 key: set `CONTEXT7_API_KEY` env var or configure on tools server.
