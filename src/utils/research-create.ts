#!/usr/bin/env bun
/**
 * Research Create - Automated PRD Generator
 * 
 * Runs every 5 minutes:
 * 1. Search X via Grok with CTO keywords
 * 2. Evaluate if worthwhile project
 * 3. Categorize: agent-flow, cto, trader
 * 4. Create PRD and push to git
 */

import { execSync } from "child_process";
import fs from "fs/promises";
import path from "path";
import { CTO_KEYWORDS } from "./keywords";

const CONFIG = {
  // Paths
  RESULTS_DIR: "./docs/grok-results",
  SUMMARIES_DIR: "./docs/grok-summaries",
  PRDS_DIR: "./docs/prds",
  STATE_FILE: "./docs/grok-summaries/.research-state.json",
  
  // Categories for routing
  CATEGORIES: {
    "agent-flow": ["claude-sdk", "openai-sdk", "mcp", "coding-agents", "developer-tools"],
    "cto": ["llm-inference", "vector-databases", "ai-security", "kubernetes", "cloud-infra"],
    "trader": ["blockchain-solana", "ai-trading", "defi", "prediction-markets"],
    "investors": ["angel", "vc", "seed-fund", "venture-capital", "early-stage"],
    "startup-credits": ["startup-credits", "startup-perks", "cloud-credits", "free-tier"]
  },
  
  // Grok API
  GROK_API_URL: "https://api.x.ai/v1",
  GROK_MODEL: "grok-4-1-fast-reasoning",
  
  // Criteria
  MIN_ENGAGEMENT: 50,
  MIN_LIKES_FOR_PRD: 100,
};

// Get Grok API key - supports both 1Password CLI and environment variable
function getGrokApiKey(): string {
  // First check environment variable (for cron/headless)
  const envKey = process.env.GROK_API_KEY;
  if (envKey) {
    return envKey;
  }

  // Fall back to 1Password CLI
  try {
    return execSync('op item get "Grok X API Key" --vault Automation --fields xai_api_key --reveal', {
      encoding: "utf-8", timeout: 10000
    }).trim();
  } catch {
    throw new Error("Failed to get Grok API key. Set GROK_API_KEY env var or ensure 1Password CLI is authenticated.");
  }
}

// Route to category
function routeToCategory(content: string): string {
  const lower = content.toLowerCase();
  
  // Investor keywords
  const investorKw = /angel|vc|venture capital|seed fund|pre-seed|early-stage|funding round| Series A|accelerator/i;
  if (investorKw.test(content)) return "investors";
  
  // Startup credits keywords
  const creditsKw = /startup credit|startup program|cloud credit|free tier|founder perk|startup discount/i;
  if (creditsKw.test(content)) return "startup-credits";
  
  // Trader keywords
  const traderKw = /solana|trading|defi|crypto|wallet|token|swap|perp|margin|arbitrage/i;
  if (traderKw.test(content)) return "trader";
  
  // Agent flow keywords
  const agentKw = /claude|agent|mcp|skill|workflow|subagent|orchestration/i;
  if (agentKw.test(content)) return "agent-flow";
  
  // Default to CTO
  return "cto";
}

// Generate comprehensive PRD
function generatePRD(post: any, category: string): string {
  const timestamp = new Date().toISOString();
  const prdId = `${post.id}-${category}`;
  
  let technicalSection = "";
  let implementationSection = "";
  
  if (category === "agent-flow") {
    technicalSection = `
## Technical: Agent Flow

### Relevant to
- Claude Code workflows
- MCP integration
- Agent orchestration
- Skill system

### Technical Requirements
- [ ] MCP server implementation
- [ ] Tool calling patterns
- [ ] State management
- [ ] Subagent coordination`;
    
    implementationSection = `
## Implementation: Agent Flow

\`\`\`typescript
// Example implementation structure
interface AgentSkill {
  name: string;
  mcpServer?: MCPServer;
  tools: Tool[];
  state: AgentState;
}

async function executeSkill(skill: AgentSkill): Promise<Result> {
  // Implementation
}
\`\`\``;
  } else if (category === "trader") {
    technicalSection = `
## Technical: Trader

### Relevant to
- Solana trading agents
- DeFi strategies
- Prediction markets
- Portfolio management

### Technical Requirements
- [ ] Jupiter API integration
- [ ] Wallet management
- [ ] Risk controls
- [ ] MEV protection`;
    
    implementationSection = `
## Implementation: Trader

\`\`\`typescript
// Example implementation structure  
interface TradingAgent {
  strategy: TradingStrategy;
  wallet: WalletConnection;
  jupiter: JupiterClient;
  
  async executeTrade(signal: TradeSignal): Promise<TradeResult> {
    // Implementation
  }
}
\`\`\``;
  } else if (category === "investors") {
    technicalSection = `
## Investors Database Entry

### Relevant to
- Early-stage funding
- Angel networks
- Venture capital
- Geographic focus

### Key Information
- [ ] Investment stage focus
- [ ] Check size
- [ ] Industry focus
- [ ] Geographic reach
- [ ] Portfolio companies`;
    
    implementationSection = `
## Investor Profile Template

\`\`\`typescript
interface InvestorProfile {
  name: string;
  type: "angel" | "vc" | "accelerator" | "fund";
  stage: "pre-seed" | "seed" | "Series A";
  checkSize: { min: number; max: number };
  industries: string[];
  regions: string[];
  portfolio: string[];
  contact?: string;
  notes: string;
}
\`\`\``;
  } else if (category === "startup-credits") {
    technicalSection = `
## Startup Credits Entry

### Relevant to
- Cost optimization
- Tool adoption
- Vendor relationships
- Founder perks

### Key Information
- [ ] Vendor name
- [ ] Credit amount/type
- [ ] Eligibility
- [ ] Duration
- [ ] Integration requirements`;
    
    implementationSection = `
## Vendor Credits Template

\`\`\`typescript
interface VendorCredits {
  vendor: string;
  product: string;
  credits: {
    amount: string;
    type: "credits" | "discount" | "free-tier";
  };
  eligibility: "all" | "invite" | "application";
  duration: string;
  stack: string[];
  signupUrl: string;
  notes: string;
}
\`\`\``;
  } else {
    technicalSection = `
## Technical: CTO

### Relevant to
- Infrastructure
- Security
- Platform architecture
- DevOps

### Technical Requirements
- [ ] Scalable architecture
- [ ] Security review
- [ ] Monitoring
- [ ] Cost optimization`;
    
    implementationSection = `
## Implementation: CTO

\`\`\`typescript
// Example implementation structure
interface PlatformComponent {
  config: PlatformConfig;
  scaling: ScalingStrategy;
  monitoring: MonitoringConfig;
  
  async deploy(): Promise<DeploymentResult> {
    // Implementation
  }
}
\`\`\``;
  }
  
  return `---
id: "${prdId}"
category: "${category}"
author: "research-agent"
posted_at: "${timestamp}"
source_url: "${post.url}"
likes: ${post.likes}
status: "pending-review"
---

# ${post.id.substring(0, 20)}... - ${category.toUpperCase()}

## Summary

${post.content.substring(0, 800)}...

## Source

- **Post ID:** ${post.id}
- **URL:** ${post.url}
- **Engagement:** ${post.likes} likes

${technicalSection}

## Why Worthwhile

- Technical implementation (not just hype)
- Aligns with ${category} roadmap
- Potential for differentiation

${implementationSection}

## Next Steps

- [ ] Review technical feasibility
- [ ] Estimate implementation effort
- [ ] Add to ${category} backlog
- [ ] Schedule implementation

---
Generated: ${timestamp}
Research Agent
`;
}

// Git operations
async function gitAdd(paths: string[]): Promise<void> {
  try {
    execSync(`git add ${paths.join(" ")}`, { cwd: "/Users/jonathonfritz/agents/research" });
  } catch (e) {
    console.log(`Git add failed: ${e}`);
  }
}

async function gitCommit(message: string): Promise<void> {
  try {
    execSync(`git commit -m "${message}"`, { cwd: "/Users/jonathonfritz/agents/research" });
    console.log(`✅ Committed: ${message}`);
  } catch (e) {
    // Nothing to commit or error
  }
}

// Post to Discord
async function postToDiscord(post: any, category: string, isPrd: boolean): Promise<void> {
  const emoji = isPrd ? "💡" : "📰";
  const type = isPrd ? "**NEW PRD**" : "Research";
  const route = category.toUpperCase();
  
  const message = `${emoji} **${type}** [${route}]
  
**ID:** ${post.id}
**Likes:** ${post.likes}
${isPrd ? "**PRD Created**" : ""}

${post.content.substring(0, 200)}...

${post.url}`;

  try {
    execSync(`/Users/jonathonfritz/.nvm/versions/node/v25.5.0/bin/clawdbot sessions_send research:default "${message.replace(/"/g, '\\"')}"`, {
      encoding: "utf-8", timeout: 10000
    });
  } catch (e) {
    console.log(`Discord post: ${message.substring(0, 100)}...`);
  }
}

// Main loop
async function researchLoop() {
  console.log(`\n🔍 [${new Date().toISOString()}] Research Create Starting...`);
  
  const apiKey = getGrokApiKey();
  const state = await loadState();
  const newFiles: string[] = [];
  
  // Search all categories
  const categories = Object.keys(CTO_KEYWORDS);
  
  for (const category of categories) {
    const keywords = CTO_KEYWORDS[category as keyof typeof CTO_KEYWORDS];
    if (!keywords) continue;
    
    console.log(`  📊 Searching: ${category}`);
    
    try {
      const response = await fetch(`${CONFIG.GROK_API_URL}/responses`, {
        method: "POST",
        headers: {
          "Authorization": `Bearer ${apiKey}`,
          "Content-Type": "application/json"
        },
        body: JSON.stringify({
          model: CONFIG.GROK_MODEL,
          input: [{ role: "user", content: `TOP posts about: ${keywords.join(", ")}. List post ID, likes, and summary.` }],
          tools: [{ type: "x_search" }]
        })
      });
      
      if (response.ok) {
        const data = await response.json();
        
        for (const item of data.output || []) {
          if (item.content && typeof item.content === "object") {
            for (const c of item.content) {
              if (c.type === "output_text" && c.text) {
                const text = c.text;
                const idMatch = text.match(/ID:\s*(\d+)/);
                const likeMatch = text.match(/(\d+)\s*likes?/i);
                
                if (idMatch && !state.processed.has(idMatch[1])) {
                  const id = idMatch[1];
                  const likes = parseInt(likeMatch?.[1] || "0");
                  
                  state.processed.add(id);
                  
                  const category = routeToCategory(text);
                  const isPrd = likes >= CONFIG.MIN_LIKES_FOR_PRD;
                  
                  // Post to Discord
                  await postToDiscord({ id, likes, content: text }, category, isPrd);
                  
                  if (isPrd) {
                    // Create PRD
                    const prd = generatePRD({ id, likes, content: text, url: `https://x.com/i/status/${id}` }, category);
                    const filePath = `${CONFIG.PRDS_DIR}/${id}-${category}.md`;
                    await fs.writeFile(filePath, prd);
                    newFiles.push(filePath);
                    console.log(`    ✅ PRD (${category}): ${id} (${likes} likes)`);
                  } else {
                    console.log(`    📰 ${category}: ${id} (${likes} likes)`);
                  }
                }
              }
            }
          }
        }
      }
    } catch (e) {
      console.log(`    Error: ${e}`);
    }
  }
  
  // Save state
  await saveState(state);
  
  // Git commit
  if (newFiles.length > 0) {
    await gitAdd(newFiles);
    await gitCommit(`research: ${newFiles.length} new PRDs - ${new Date().toISOString()}`);
  }
  
  console.log(`\n✨ Research complete. ${newFiles.length} PRDs created.`);
}

// State management
async function loadState(): Promise<{ processed: Set<string> }> {
  try {
    const content = await fs.readFile(CONFIG.STATE_FILE, "utf-8");
    const data = JSON.parse(content);
    return { processed: new Set(data.processed || []) };
  } catch {
    return { processed: new Set<string>() };
  }
}

async function saveState(state: { processed: Set<string> }): Promise<void> {
  await fs.writeFile(CONFIG.STATE_FILE, JSON.stringify({
    lastRun: new Date().toISOString(),
    processed: Array.from(state.processed)
  }, null, 2));
}

// CLI
async function main() {
  const args = process.argv.slice(2);
  
  if (args.includes("--once")) {
    await researchLoop();
  } else if (args.includes("--cron")) {
    console.log("Starting cron (every 5 minutes)...");
    while (true) {
      await researchLoop();
      await new Promise(r => setTimeout(r, 5 * 60 * 1000));
    }
  } else if (args.includes("--test")) {
    console.log("Test mode...");
    await researchLoop();
  }
}

main().catch(console.error);
