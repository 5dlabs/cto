/**
 * Grok X Search - Merged Keywords
 *
 * Combines:
 *  - keywords.json (investor + startup credit keywords managed via MCP tools)
 *  - CTO research keywords (static, code-defined)
 *
 * The JSON file is the source of truth for investor/startupCredit keywords
 * that can be modified at runtime via the MCP server's add_keyword /
 * remove_keyword tools. CTO_KEYWORDS are static research categories.
 */

import fs from 'fs/promises';
import path from 'path';
import { fileURLToPath } from 'url';
import type { KeywordConfig } from './types.js';

// ---------------------------------------------------------------------------
// JSON keyword config (investor + startup credit)
// ---------------------------------------------------------------------------

let configCache: KeywordConfig | null = null;

/**
 * Load the JSON keyword config from disk.
 * Caches after the first successful load; call `clearConfigCache()` to reload.
 */
export async function loadKeywordsConfig(
  filePath?: string,
): Promise<KeywordConfig> {
  if (configCache) return configCache;

  const resolvedPath =
    filePath ??
    path.join(
      path.dirname(fileURLToPath(import.meta.url)),
      '../../keywords.json',
    );

  try {
    const raw = await fs.readFile(resolvedPath, 'utf-8');
    configCache = JSON.parse(raw) as KeywordConfig;
    return configCache;
  } catch (error) {
    throw new Error(
      `Failed to load keywords: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
}

export function clearConfigCache(): void {
  configCache = null;
}

// ---------------------------------------------------------------------------
// Default fallback keywords (same as keywords.json defaults)
// ---------------------------------------------------------------------------

export function getDefaultInvestorKeywords(): string[] {
  return [
    'early-stage investor',
    'angel investor',
    'seed fund',
    'pre-seed funding',
    'VC firm',
    'venture capital',
    'tech investor',
    'startup investor',
    'SaaS investor',
    'AI fund',
    'Canadian VC',
    'US VC',
    'global venture capital',
    'accelerator investor',
    'angel network',
    'Series A',
    'Series B',
    'growth equity',
    'micro VC',
    'splash fund',
  ];
}

export function getDefaultStartupCreditKeywords(): string[] {
  return [
    'startup credits',
    'startup program',
    'startup perk',
    'free tier startup',
    'startup discount',
    'cloud credits startup',
    'software for startups',
    'founder credits',
    'startup sandbox',
    'AWS Activate',
    'Google for Startups',
    'Microsoft for Startups',
    'Stripe Atlas',
    'Cloudflare Workers',
    'Vercel Startup',
    'GitHub Student',
    'OpenAI Startup Fund',
  ];
}

// ---------------------------------------------------------------------------
// Accessors
// ---------------------------------------------------------------------------

/**
 * Get investor keywords from JSON config, falling back to defaults.
 */
export async function getInvestorKeywords(): Promise<string[]> {
  try {
    const config = await loadKeywordsConfig();
    return config.investors.keywords;
  } catch {
    return getDefaultInvestorKeywords();
  }
}

/**
 * Get startup credit keywords from JSON config, falling back to defaults.
 */
export async function getStartupCreditKeywords(): Promise<string[]> {
  try {
    const config = await loadKeywordsConfig();
    return config.startupCredits.keywords;
  } catch {
    return getDefaultStartupCreditKeywords();
  }
}

/**
 * Get all investor + startup credit keywords combined.
 */
export async function getAllKeywords(): Promise<string[]> {
  const [inv, credits] = await Promise.all([
    getInvestorKeywords(),
    getStartupCreditKeywords(),
  ]);
  return [...inv, ...credits];
}

// ---------------------------------------------------------------------------
// CTO Research Keywords (static categories)
// ---------------------------------------------------------------------------

/** Developer-focused CTO research keywords across 30+ categories */
export const CTO_KEYWORDS: Record<string, string[]> = {
  agentDevelopment: [
    'agentic AI development',
    'multi-agent architecture',
    'agent orchestration patterns',
    'agent communication protocol',
    'agent state management',
    'agent tool calling',
    'function calling LLM',
    'agent execution model',
    'autonomous agent loops',
    'agent handoff',
  ],
  claudeSDK: [
    'Claude API',
    'Claude SDK',
    'Claude Code CLI',
    'Anthropic API',
    'Claude function calling',
    'Claude tool use',
    'Claude messages API',
    'Claude computer use',
    'Claude context window',
    'Claude structured output',
  ],
  openAISDK: [
    'OpenAI API',
    'OpenAI SDK',
    'GPT-4 API',
    'Chat Completions API',
    'OpenAI function calling',
    'OpenAI tool use',
    'OpenAI structured outputs',
    'OpenAI o1 model',
    'OpenAI batch API',
    'OpenAI streaming',
  ],
  mcp: [
    'Model Context Protocol',
    'MCP server implementation',
    'MCP client',
    'Anthropic MCP',
    'MCP tools definition',
    'MCP protocol spec',
    'MCP resource handling',
    'MCP prompts',
    'MCP sampling',
    'MCP stdio transport',
  ],
  llmInfrastructure: [
    'LLM inference',
    'LLM serving',
    'vLLM',
    'TensorRT-LLM',
    'llama.cpp',
    'LLM batching',
    'KV cache optimization',
    'model quantization',
    'LLM deployment',
    'inference optimization',
  ],
  vectorDatabases: [
    'vector database',
    'Pinecone',
    'Weaviate',
    'Qdrant',
    'ChromaDB',
    'Milvus',
    'vector search',
    'embedding models',
    'semantic search',
    'RAG implementation',
  ],
  aiObservability: [
    'LLM observability',
    'LangSmith',
    'LangChain tracing',
    'OpenTelemetry LLM',
    'prompt monitoring',
    'token usage tracking',
    'LLM evaluation',
    'cost monitoring',
    'latency tracking',
    'model benchmarking',
  ],
  aiSecurity: [
    'prompt injection',
    'LLM security',
    'adversarial prompting',
    'jailbreak prevention',
    'output sanitization',
    'input validation LLM',
    'security testing AI',
    'red teaming AI',
    'AI safety',
    'model robustness',
  ],
  codingAgents: [
    'SWE-bench',
    'AI coding agent',
    'autonomous coding',
    'code generation LLM',
    'AI code review',
    'refactoring agent',
    'test generation AI',
    'documentation AI',
    'bug fix agent',
    'agentic programming',
  ],
  developerTools: [
    'Cursor IDE',
    'GitHub Copilot',
    'VS Code AI',
    'Claude Code',
    'ai-shell',
    'Tabby AI',
    'Codeium',
    'Cody AI',
    'OpenAI Codex',
    'ai-code-assistant',
  ],
  mcpServers: [
    'MCP GitHub server',
    'MCP filesystem',
    'MCP database',
    'MCP API client',
    'MCP PostgreSQL',
    'MCP Redis',
    'MCP Slack',
    'MCP Figma',
    'MCP custom server',
    'build MCP server',
  ],
  kubernetes: [
    'Kubernetes',
    'k8s AI',
    'container orchestration',
    'Kubernetes AI deployment',
    'Helm charts AI',
    'operator pattern AI',
    'Kustomize',
    'Argo CD',
    'Knative',
    'service mesh AI',
  ],
  cloudInfra: [
    'AWS Bedrock',
    'Google Vertex AI',
    'Azure AI',
    'Cloudflare Workers AI',
    'Vercel AI SDK',
    'serverless LLM',
    'edge inference',
    'Lambda AI',
    'Cloud Run AI',
    'infrastructure as code AI',
  ],
  dockerSandboxes: [
    'Docker',
    'Docker Compose',
    'containerization AI',
    'sandbox environment',
    'isolated execution',
    'gVisor',
    'Firecracker',
    'Kata Containers',
    'container security',
    'buildah',
  ],
  rustAI: [
    'Rust AI',
    'llama.cpp Rust',
    'Candle Rust',
    'Rust LLM inference',
    'Rust embeddings',
    'Rust async AI',
    'Bevy AI',
    'Rust agent framework',
    'Rust MCP server',
    'systems programming AI',
  ],
  goAI: [
    'Go AI',
    'Go LLM',
    'Golang AI tools',
    'Go MCP server',
    'Go agent framework',
    'Go concurrency AI',
    'Go Kubernetes AI',
    'ChiRouter AI',
    'Go middleware AI',
    'GopherAI',
  ],
  tsNodeAI: [
    'TypeScript AI',
    'Node.js AI SDK',
    'Deno AI',
    'Bun AI',
    'LangChain.js',
    'LlamaIndex.js',
    'Vercel AI SDK',
    'Next.js AI',
    'AI API TypeScript',
    'AI SDK Node',
  ],
  memoryContext: [
    'long-term memory AI',
    'vector memory',
    'semantic memory',
    'context management',
    'memory persistence',
    'agent memory bank',
    'knowledge retrieval',
    'context compression',
    'memory optimization',
    'persistent embeddings',
  ],
  evaluationTesting: [
    'LLM evaluation',
    'benchmarking AI',
    'testing AI models',
    'unit tests AI',
    'integration tests LLM',
    'eval harness',
    'OpenAI evals',
    'LLMUnit',
    'prompt testing',
    'AI regression testing',
  ],
  apiDesign: [
    'API design AI',
    'REST API AI',
    'GraphQL AI',
    'gRPC AI',
    'API gateway AI',
    'API versioning',
    'API documentation',
    'OpenAPI AI',
    'API rate limiting',
    'API authentication',
  ],
  promptEngineering: [
    'prompt engineering',
    'prompt templates',
    'few-shot prompting',
    'chain-of-thought',
    'ReAct prompting',
    'system prompts',
    'prompt optimization',
    'prompt versioning',
    'prompt testing',
    'prompt injection',
  ],
  multiModel: [
    'model routing',
    'LLM gateway',
    'multi-model LLM',
    'model fallback',
    'load balancing AI',
    'cost routing',
    'latency routing',
    'model selection',
    'LLM proxy',
    'unified API gateway',
  ],
  autonomousAgents: [
    'autonomous agent',
    'self-improving AI',
    'agent loops',
    'reflection agent',
    'agent planning',
    'goal decomposition',
    'task execution agent',
    'agent persistence',
    'agent monitoring',
    'agent recovery',
  ],
  deepSeekKimi: [
    'DeepSeek',
    'DeepSeek V3',
    'DeepSeek R1',
    'Kimi AI',
    'Kimi K2',
    'Moonshot AI',
    'DeepSeek API',
    'DeepSeek code',
    'Kimi context',
    'Chinese LLM',
  ],
  mistralAI: [
    'Mistral AI',
    'Mistral 7B',
    'Mistral Large',
    'Mixtral',
    'Mistral API',
    'Mistral fine-tuning',
    'CPTQ Mistral',
    'Mistral deployment',
    'Mistral edge',
    'Codestral',
  ],
  geminiGoogle: [
    'Gemini API',
    'Google AI',
    'Vertex AI',
    'Gemini Pro',
    'Gemini Ultra',
    'Gemini Flash',
    'PaLM 2',
    'Google AI Studio',
    'MakerSuite',
    'Gemini function calling',
  ],
  openSourceModels: [
    'Llama',
    'Llama 3',
    'Mistral',
    'Qwen',
    'Yi',
    'Gemma',
    'Phi-3',
    'OpenChat',
    'Starcoder',
    'CodeLlama',
  ],
  blockchainSolana: [
    'Solana',
    'Solana agent',
    'Solana program',
    'Solana Rust',
    'anchor framework',
    'Solana DeFi',
    'Jupiter aggregator',
    'Raydium',
    'Solana smart contracts',
    'Solana CPI',
  ],
  claudeCodePatterns: [
    'Claude Code prompts',
    'Claude Code settings',
    'Claude Code tools',
    'Claude Code workflow',
    'Claude Code MCP',
    'Claude mode',
    'Claude plan mode',
    'Claude execute mode',
    'Claude CLI',
    'Claude project context',
  ],
  openClaw: [
    'OpenClaw',
    'clawdbot',
    'OpenClaw agents',
    'agent orchestration',
    'multi-agent platform',
    'agent skills system',
    'OpenClaw SDK',
    'CLAWD CLI',
    'OpenClaw MCP',
    'agent workflow platform',
  ],
};

/** Flat array of all CTO research keywords. */
export const ALL_CTO_KEYWORDS: string[] = Object.values(CTO_KEYWORDS).flat();

/**
 * Get keywords for a specific CTO research category.
 */
export function getCtoKeywords(category: string): string[] {
  return CTO_KEYWORDS[category] ?? [];
}

/**
 * Get all CTO category names.
 */
export function getCtoCategoryNames(): string[] {
  return Object.keys(CTO_KEYWORDS);
}
