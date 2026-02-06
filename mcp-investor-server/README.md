# MCP Investor Research Server

MCP (Model Context Protocol) server for researching early-stage investors and startup credits.

## Features

- **Search Investors**: Find angel investors, VCs, seed funds, and accelerators on X
- **Search Startup Credits**: Discover startup programs, credits, and perk offerings
- **Keyword Management**: Add/remove keywords via MCP tools
- **Configuration**: Keywords stored in `keywords.json` for easy updates

## Installation

```bash
# Install dependencies
bun install

# Build TypeScript
bun run build
```

## Usage

### Development (watch mode)

```bash
bun run dev
```

### Standalone Server

```bash
./bin/server.sh
```

### As MCP Plugin

Add to your Claude Code or Cursor config:

```json
{
  "mcpServers": {
    "investor-research": {
      "command": "/path/to/mcp-investor-server/bin/server.sh",
      "env": {
        "GROK_API_KEY": "your-api-key"
      }
    }
  }
}
```

## Available Tools

| Tool | Description |
|------|-------------|
| `search_investors` | Search for early-stage investors on X |
| `search_startup_credits` | Search for startup credit programs |
| `search_all` | Search both investors and credits in one call |
| `list_investor_keywords` | List all investor search keywords |
| `list_startup_credit_keywords` | List all startup credit keywords |
| `get_keywords_config` | Get full keywords configuration |
| `add_keyword` | Add new keyword (investors or startupCredits) |
| `remove_keyword` | Remove a keyword |
| `load_custom_keywords` | Load custom keywords from JSON file |
| `health_check` | Check server health and configuration |

## Custom Keywords

Load your own keyword configuration:

```bash
# Create custom keywords file
cat > custom-keywords.json << 'EOF'
{
  "investors": {
    "description": "My custom investors",
    "keywords": ["my niche investor", "specific fund"]
  },
  "startupCredits": {
    "description": "My custom credits",
    "keywords": ["my program", "specific perk"]
  }
}
EOF

# Use in server
./bin/server.sh --keywords custom-keywords.json
```

Or via the MCP tool:
```
use load_custom_keywords with filePath: "/path/to/custom-keywords.json"
```

## Keywords Configuration

Keywords are stored in `keywords.json`:

```json
{
  "investors": {
    "description": "Early-stage investors, angels, VCs, seed funds",
    "keywords": ["angel investor", "VC firm", "seed fund", ...]
  },
  "startupCredits": {
    "description": "Startup credits, perks, and free tier programs",
    "keywords": ["startup credits", "AWS Activate", ...]
  }
}
```

## Publishing to GitHub Package Registry

```bash
# Login to GitHub Packages
npm login --registry=https://npm.pkg.github.com

# Build and publish
bun run build
npm publish
```

## Requirements

- Bun 1.0+
- TypeScript 5.0+
- Grok API key (for X search)
- 1Password CLI (optional, for API key retrieval)
