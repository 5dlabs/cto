/**
 * Multi-Agent Debate Planning System v2
 * 
 * Flow: Research → Proposals (3 perspectives) → Debate → Consensus → Critique → Final
 * 
 * Features:
 * - Configurable depth (light/medium/deep)
 * - Fullstack/UI perspective
 * - Domain focus (security/performance/compliance)
 * - Multi-provider support (any provider → any role)
 */

import type { TokenUsage, GeneratedTask } from '../types';
import { getAvailableProvider, providerRegistry, type ProviderName, type ModelProvider } from '../providers';
import { createLogger } from '../utils/logger';

const logger = createLogger('debate-planning');

// =============================================================================
// Types
// =============================================================================

export type DebateDepth = 'light' | 'medium' | 'deep';
export type DomainFocus = 'general' | 'security' | 'performance' | 'compliance';
export type DebateRole = 'proposer' | 'critic' | 'synthesizer';

export interface RoleProviderConfig {
  provider: ProviderName;
  model?: string; // Uses provider default if not specified
}

export interface DebatePlanningConfig {
  /** Debate depth - light (1 round), medium (2), deep (3) */
  depth: DebateDepth;
  /** Domain focus for specialized perspectives */
  domainFocus: DomainFocus;
  /** Skip research phase for simple PRDs */
  skipResearch: boolean;
  /** Include fullstack/UI perspective */
  includeFullstack: boolean;
  /** Provider config per role (defaults to Claude for all) */
  providers: {
    proposer: RoleProviderConfig;
    critic: RoleProviderConfig;
    synthesizer: RoleProviderConfig;
  };
}

export interface ResearchFindings {
  summary: string;
  findings: Array<{ topic: string; content: string }>;
  recommendations: string[];
}

export interface AgentProposal {
  agent: 'pessimist' | 'optimist' | 'fullstack';
  solution: string;
  tasks: GeneratedTask[];
  keyPoints: string[];
}

export interface DebateRound {
  round: number;
  critiques: Array<{
    from: string;
    to: string;
    points: string[];
  }>;
}

export interface ConsensusResult {
  mergedSolution: string;
  agreedPoints: string[];
  tradeoffs: string[];
  tasks: GeneratedTask[];
}

export interface DebatePlanningResult {
  success: boolean;
  config: DebatePlanningConfig;
  research?: ResearchFindings;
  proposals?: Record<string, AgentProposal>;
  debate?: DebateRound[];
  consensus?: ConsensusResult;
  critique?: { issues: Array<{ severity: string; description: string }>; fixes: string[] };
  finalPlan?: { tasks: GeneratedTask[]; summary: string };
  error?: string;
  usage: TokenUsage;
}

// =============================================================================
// Prompts
// =============================================================================

const PROMPTS = {
  pessimist: `You are a PESSIMISTIC planning agent analyzing a PRD.

Your job: Identify risks, failure modes, and conservative estimates.
Think about: security vulnerabilities, scalability limits, dependency failures, edge cases.

You MUST respond with valid JSON (no markdown, no explanation):
{
  "solution": "Your proposed approach focusing on risk mitigation",
  "tasks": [
    {"id": 1, "title": "Task name", "description": "What to do", "priority": "high", "dependencies": []}
  ],
  "keyPoints": ["Key risk or mitigation point"]
}`,

  optimist: `You are an OPTIMISTIC planning agent analyzing a PRD.

Your job: Identify opportunities, efficiency gains, and best-case approaches.
Think about: parallelization, modern tools, quick wins, innovation.

You MUST respond with valid JSON (no markdown, no explanation):
{
  "solution": "Your proposed approach focusing on opportunities",
  "tasks": [
    {"id": 1, "title": "Task name", "description": "What to do", "priority": "high", "dependencies": []}
  ],
  "keyPoints": ["Key opportunity or efficiency"]
}`,

  fullstack: `You are a FULLSTACK planning agent analyzing a PRD.

Your job: Identify frontend/UI needs, API design, and user experience.
Think about: user flows, component architecture, state management, responsive design.

You MUST respond with valid JSON (no markdown, no explanation):
{
  "solution": "Your proposed approach focusing on UI/UX and integration",
  "tasks": [
    {"id": 1, "title": "Task name", "description": "What to do", "priority": "high", "dependencies": []}
  ],
  "keyPoints": ["Key UI/UX or integration point"]
}`,

  synthesis: `Merge multiple agent proposals into one balanced plan.

Include: risk mitigations + efficiency gains + UI/UX coverage.
Resolve conflicts by choosing the safer option with better UX.

You MUST respond with valid JSON (no markdown, no explanation):
{
  "mergedSolution": "Unified approach description",
  "agreedPoints": ["Point all agents agreed on"],
  "tradeoffs": ["Tradeoff made and why"],
  "tasks": [
    {"id": 1, "title": "Task name", "description": "What to do", "priority": "high", "dependencies": []}
  ]
}`,

  critique: `Review this plan and find issues.

Look for: structural problems, missing components, feasibility concerns, gaps.

You MUST respond with valid JSON (no markdown, no explanation):
{
  "issues": [{"severity": "critical", "description": "Issue description"}],
  "fixes": ["How to fix the issue"]
}`,

  remediate: `Apply the suggested fixes to the plan. ONE PASS ONLY.

You MUST respond with valid JSON (no markdown, no explanation):
{
  "summary": "What was changed",
  "tasks": [
    {"id": 1, "title": "Task", "description": "Desc", "status": "pending", "priority": "high", "dependencies": [], "details": "Implementation details", "testStrategy": "How to test"}
  ]
}`,

  research: `Research this PRD. Identify best practices, patterns, and pitfalls.

You MUST respond with valid JSON (no markdown, no explanation):
{
  "summary": "Research summary",
  "findings": [{"topic": "Topic", "content": "Finding"}],
  "recommendations": ["Recommendation"]
}`,
};

// Domain-specific additions
const DOMAIN_PROMPTS: Record<DomainFocus, string> = {
  general: '',
  security: `
SECURITY FOCUS: Include specific configs (bcrypt cost, token expiry, cipher suites), threat models, compliance requirements.`,
  performance: `
PERFORMANCE FOCUS: Include benchmarks, caching strategies, connection pooling, load testing requirements.`,
  compliance: `
COMPLIANCE FOCUS: Include GDPR/SOC2/HIPAA requirements, audit logging, data retention, PII handling.`,
};

// Depth configs
const DEPTH_CONFIG: Record<DebateDepth, { rounds: number; skipCritique: boolean }> = {
  light: { rounds: 1, skipCritique: true },
  medium: { rounds: 2, skipCritique: false },
  deep: { rounds: 3, skipCritique: false },
};

// =============================================================================
// Helpers
// =============================================================================

function parseJSON<T>(text: string, fallback: T): T {
  try {
    const match = text.match(/\{[\s\S]*\}/);
    if (match) return JSON.parse(match[0]) as T;
  } catch {}
  return fallback;
}

function addUsage(total: TokenUsage, add: TokenUsage): void {
  total.input_tokens += add.input_tokens;
  total.output_tokens += add.output_tokens;
  total.total_tokens += add.total_tokens;
}

/**
 * Query a provider with a prompt.
 */
async function queryProvider(
  roleConfig: RoleProviderConfig,
  systemPrompt: string,
  userPrompt: string
): Promise<{ text: string; usage: TokenUsage }> {
  const provider = getAvailableProvider(roleConfig.provider);
  const model = roleConfig.model || provider.defaultModel;
  
  logger.debug(`Querying ${roleConfig.provider}/${model}`);
  
  const response = await provider.generate(userPrompt, systemPrompt, {
    maxTokens: 8192,
    temperature: 0.7,
  }, model);
  
  if (!response.success) {
    throw new Error(`Provider ${roleConfig.provider} failed: ${response.error}`);
  }
  
  return {
    text: response.text || '',
    usage: response.usage,
  };
}

// =============================================================================
// Phase Implementations
// =============================================================================

async function runResearch(
  prd: string,
  config: DebatePlanningConfig
): Promise<{ findings: ResearchFindings; usage: TokenUsage }> {
  logger.info('Research phase', { provider: config.providers.proposer.provider });
  const { text, usage } = await queryProvider(
    config.providers.proposer,
    PROMPTS.research + DOMAIN_PROMPTS[config.domainFocus],
    `PRD:\n${prd}`
  );
  return {
    findings: parseJSON(text, { summary: text.slice(0, 500), findings: [], recommendations: [] }),
    usage,
  };
}

async function runProposals(
  prd: string,
  research: ResearchFindings | null,
  config: DebatePlanningConfig
): Promise<{ proposals: Record<string, AgentProposal>; usage: TokenUsage }> {
  logger.info('Proposal phase', { provider: config.providers.proposer.provider });
  
  const context = research 
    ? `Research:\n${research.summary}\n\nPRD:\n${prd}`
    : `PRD:\n${prd}`;

  const domainSuffix = DOMAIN_PROMPTS[config.domainFocus];
  
  // Always run pessimist + optimist
  const agents: Array<{ key: string; prompt: string }> = [
    { key: 'pessimist', prompt: PROMPTS.pessimist + domainSuffix },
    { key: 'optimist', prompt: PROMPTS.optimist + domainSuffix },
  ];
  
  // Optionally add fullstack
  if (config.includeFullstack) {
    agents.push({ key: 'fullstack', prompt: PROMPTS.fullstack + domainSuffix });
  }

  const results = await Promise.all(
    agents.map(async ({ key, prompt }) => {
      const { text, usage } = await queryProvider(config.providers.proposer, prompt, context);
      const parsed = parseJSON(text, { solution: '', tasks: [], keyPoints: [] });
      return { key, proposal: { agent: key as AgentProposal['agent'], ...parsed }, usage };
    })
  );

  const totalUsage: TokenUsage = { input_tokens: 0, output_tokens: 0, total_tokens: 0 };
  const proposals: Record<string, AgentProposal> = {};
  
  for (const r of results) {
    proposals[r.key] = r.proposal;
    addUsage(totalUsage, r.usage);
  }

  return { proposals, usage: totalUsage };
}

async function runDebate(
  proposals: Record<string, AgentProposal>,
  config: DebatePlanningConfig
): Promise<{ rounds: DebateRound[]; usage: TokenUsage }> {
  const { rounds: maxRounds } = DEPTH_CONFIG[config.depth];
  logger.info('Debate phase', { maxRounds, provider: config.providers.critic.provider });

  const rounds: DebateRound[] = [];
  const totalUsage: TokenUsage = { input_tokens: 0, output_tokens: 0, total_tokens: 0 };
  const agentKeys = Object.keys(proposals);

  for (let round = 1; round <= maxRounds; round++) {
    logger.info(`Round ${round}/${maxRounds}`);
    
    const critiques: DebateRound['critiques'] = [];

    // Each agent critiques each other agent (using critic provider)
    const critiquePromises = [];
    for (const from of agentKeys) {
      for (const to of agentKeys) {
        if (from === to) continue;
        critiquePromises.push(
          (async () => {
            const prompt = `Critique this ${to.toUpperCase()} proposal from ${from.toUpperCase()} perspective:\n${proposals[to].solution}\nOutput JSON: {"points":["critique1","suggestion1"]}`;
            const systemPrompt = PROMPTS[from as keyof typeof PROMPTS] || PROMPTS.pessimist;
            const { text, usage } = await queryProvider(config.providers.critic, systemPrompt, prompt);
            const parsed = parseJSON(text, { points: [] });
            return { from, to, points: parsed.points || [], usage };
          })()
        );
      }
    }

    const critiqueResults = await Promise.all(critiquePromises);
    for (const r of critiqueResults) {
      critiques.push({ from: r.from, to: r.to, points: r.points });
      addUsage(totalUsage, r.usage);
    }

    rounds.push({ round, critiques });

    // Early termination if critiques are repetitive
    if (round > 1) {
      const prev = JSON.stringify(rounds[round - 2].critiques.map(c => c.points));
      const curr = JSON.stringify(critiques.map(c => c.points));
      if (prev === curr) {
        logger.info('Debate converged early');
        break;
      }
    }
  }

  return { rounds, usage: totalUsage };
}

async function runConsensus(
  proposals: Record<string, AgentProposal>,
  debate: DebateRound[],
  config: DebatePlanningConfig
): Promise<{ consensus: ConsensusResult; usage: TokenUsage }> {
  logger.info('Consensus phase', { provider: config.providers.synthesizer.provider });

  const proposalSummary = Object.entries(proposals)
    .map(([k, v]) => `## ${k.toUpperCase()}\n${v.solution}\nKey: ${v.keyPoints.join(', ')}`)
    .join('\n\n');

  const debateSummary = debate
    .map(r => `Round ${r.round}:\n${r.critiques.map(c => `- ${c.from}→${c.to}: ${c.points.slice(0, 2).join('; ')}`).join('\n')}`)
    .join('\n');

  const prompt = `${proposalSummary}\n\nDebate:\n${debateSummary}`;
  const { text, usage } = await queryProvider(config.providers.synthesizer, PROMPTS.synthesis, prompt);

  return {
    consensus: parseJSON(text, { mergedSolution: '', agreedPoints: [], tradeoffs: [], tasks: [] }),
    usage,
  };
}

async function runCritique(
  consensus: ConsensusResult,
  config: DebatePlanningConfig
): Promise<{ issues: Array<{ severity: string; description: string }>; fixes: string[]; usage: TokenUsage }> {
  logger.info('Critique phase', { provider: config.providers.critic.provider });

  const prompt = `Plan:\n${consensus.mergedSolution}\n\nTasks:\n${consensus.tasks.map(t => `${t.id}. ${t.title}`).join('\n')}`;
  const { text, usage } = await queryProvider(
    config.providers.critic,
    PROMPTS.critique + DOMAIN_PROMPTS[config.domainFocus],
    prompt
  );

  const parsed = parseJSON(text, { issues: [], fixes: [] });
  return { ...parsed, usage };
}

async function runRemediation(
  consensus: ConsensusResult,
  critique: { issues: Array<{ severity: string; description: string }>; fixes: string[] },
  config: DebatePlanningConfig
): Promise<{ tasks: GeneratedTask[]; summary: string; usage: TokenUsage }> {
  logger.info('Remediation phase', { provider: config.providers.synthesizer.provider });

  const prompt = `Plan:\n${consensus.mergedSolution}\n\nIssues:\n${critique.issues.map(i => `[${i.severity}] ${i.description}`).join('\n')}\n\nFixes:\n${critique.fixes.join('\n')}`;
  const { text, usage } = await queryProvider(config.providers.synthesizer, PROMPTS.remediate, prompt);

  const parsed = parseJSON(text, { tasks: consensus.tasks, summary: 'No changes' });
  return { ...parsed, usage };
}

// =============================================================================
// Main Entry Point
// =============================================================================

/**
 * Default provider config - uses Claude for all roles.
 */
const DEFAULT_PROVIDERS: DebatePlanningConfig['providers'] = {
  proposer: { provider: 'claude' },
  critic: { provider: 'claude' },
  synthesizer: { provider: 'claude' },
};

export async function generatePlanWithDebate(
  prdContent: string,
  partialConfig: Partial<DebatePlanningConfig> = {}
): Promise<DebatePlanningResult> {
  // Merge provider configs
  const providersConfig = {
    ...DEFAULT_PROVIDERS,
    ...partialConfig.providers,
  };

  const config: DebatePlanningConfig = {
    depth: partialConfig.depth ?? 'medium',
    domainFocus: partialConfig.domainFocus ?? 'general',
    skipResearch: partialConfig.skipResearch ?? false,
    includeFullstack: partialConfig.includeFullstack ?? true,
    providers: providersConfig,
  };

  const totalUsage: TokenUsage = { input_tokens: 0, output_tokens: 0, total_tokens: 0 };
  const depthConfig = DEPTH_CONFIG[config.depth];

  // Log provider status
  const availableProviders = providerRegistry.listAvailable();
  logger.info('Starting debate planning', { 
    depth: config.depth, 
    domainFocus: config.domainFocus, 
    includeFullstack: config.includeFullstack,
    providers: {
      proposer: config.providers.proposer.provider,
      critic: config.providers.critic.provider,
      synthesizer: config.providers.synthesizer.provider,
    },
    availableProviders,
  });

  try {
    // Phase 1: Research (optional)
    let research: ResearchFindings | null = null;
    if (!config.skipResearch) {
      logger.info('=== RESEARCH ===');
      const r = await runResearch(prdContent, config);
      research = r.findings;
      addUsage(totalUsage, r.usage);
    }

    // Phase 2: Proposals
    logger.info('=== PROPOSALS ===');
    const { proposals, usage: proposalUsage } = await runProposals(prdContent, research, config);
    addUsage(totalUsage, proposalUsage);

    // Phase 3: Debate
    logger.info('=== DEBATE ===');
    const { rounds, usage: debateUsage } = await runDebate(proposals, config);
    addUsage(totalUsage, debateUsage);

    // Phase 4: Consensus
    logger.info('=== CONSENSUS ===');
    const { consensus, usage: consensusUsage } = await runConsensus(proposals, rounds, config);
    addUsage(totalUsage, consensusUsage);

    // Phase 5: Critique (skip for light depth)
    let critique: { issues: Array<{ severity: string; description: string }>; fixes: string[] } | undefined;
    let finalTasks = consensus.tasks;
    let summary = 'Direct from consensus';

    if (!depthConfig.skipCritique) {
      logger.info('=== CRITIQUE ===');
      const c = await runCritique(consensus, config);
      critique = { issues: c.issues, fixes: c.fixes };
      addUsage(totalUsage, c.usage);

      // Phase 6: Remediation
      logger.info('=== REMEDIATION ===');
      const r = await runRemediation(consensus, critique, config);
      finalTasks = r.tasks;
      summary = r.summary;
      addUsage(totalUsage, r.usage);
    }

    logger.info('=== COMPLETE ===', { totalTokens: totalUsage.total_tokens, taskCount: finalTasks.length });

    return {
      success: true,
      config,
      research: research ?? undefined,
      proposals,
      debate: rounds,
      consensus,
      critique,
      finalPlan: { tasks: finalTasks, summary },
      usage: totalUsage,
    };
  } catch (e) {
    const error = e instanceof Error ? e.message : 'Unknown error';
    logger.error('Debate planning failed', { error });
    return { success: false, config, error, usage: totalUsage };
  }
}

export const DEFAULT_DEBATE_CONFIG: DebatePlanningConfig = {
  depth: 'medium',
  domainFocus: 'general',
  skipResearch: false,
  includeFullstack: true,
  providers: DEFAULT_PROVIDERS,
};

// Re-export types for external use
export type { GeneratedTask };
