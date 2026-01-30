/**
 * Adversarial Debate System - Three Agent Model
 * 
 * Pattern: Advocate-Adversary-Arbiter (AAA)
 * 
 * Flow:
 *   1. Research (optional) - Context gathering
 *   2. Opening Arguments - Both sides present cases
 *   3. Debate Rounds - Direct argumentation with rebuttals
 *   4. Arbiter Decision - Neutral judge weighs evidence and decides
 * 
 * Agents:
 *   - Optimist (Advocate): Argues FOR - opportunities, efficiency, best case
 *   - Pessimist (Adversary): Argues AGAINST - risks, failure modes, worst case  
 *   - Arbiter (Judge): Neutral, weighs both sides, makes reasoned decisions
 * 
 * Key Insight: Arbiter DECIDES, not merges. Picks winners with justification.
 */

import type { TokenUsage, GeneratedTask } from '../types';
import { getAvailableProvider, providerRegistry, type ProviderName } from '../providers';
import { createLogger } from '../utils/logger';

const logger = createLogger('adversarial-debate');

// =============================================================================
// Types
// =============================================================================

export type DebateDepth = 'light' | 'medium' | 'deep';
export type DomainFocus = 'general' | 'security' | 'performance' | 'compliance';

export interface ProviderConfig {
  provider: ProviderName;
  model?: string;
}

export interface AdversarialDebateConfig {
  /** Debate depth - affects number of rebuttal rounds */
  depth: DebateDepth;
  /** Domain focus for specialized perspectives */
  domainFocus: DomainFocus;
  /** Skip research phase */
  skipResearch: boolean;
  /** Provider config per agent role */
  providers: {
    optimist: ProviderConfig;   // Advocate
    pessimist: ProviderConfig;  // Adversary
    arbiter: ProviderConfig;    // Judge (should be most capable/neutral)
  };
}

export interface OpeningArgument {
  position: string;
  keyPoints: string[];
  evidence: string[];
  proposedTasks: GeneratedTask[];
}

export interface Rebuttal {
  targetPoints: string[];  // Which opponent points are being challenged
  counterArguments: string[];
  concessions: string[];   // Points they agree on (shows good faith)
}

export interface DebateExchange {
  round: number;
  optimistRebuttal: Rebuttal;
  pessimistRebuttal: Rebuttal;
}

export interface ArbiterDecision {
  verdict: string;  // Overall approach chosen
  reasoning: string;  // Why this decision
  optimistWins: string[];  // Points where optimist was right
  pessimistWins: string[];  // Points where pessimist was right
  synthesis: string[];  // Points that needed combining
  finalTasks: GeneratedTask[];
  confidenceScore: number;  // 0-1, how confident in decision
}

export interface AdversarialDebateResult {
  success: boolean;
  config: AdversarialDebateConfig;
  research?: { summary: string; findings: string[] };
  openingArguments?: {
    optimist: OpeningArgument;
    pessimist: OpeningArgument;
  };
  debateRounds?: DebateExchange[];
  arbiterDecision?: ArbiterDecision;
  error?: string;
  usage: TokenUsage;
}

// =============================================================================
// Agent Manifests (ACP-aligned)
// =============================================================================

export const AGENT_MANIFESTS = {
  optimist: {
    name: 'optimist',
    description: 'Advocate agent that argues FOR proposals - identifies opportunities, efficiency gains, and best-case approaches',
    input_content_types: ['text/plain', 'application/json'],
    output_content_types: ['application/json'],
    metadata: {
      role: 'advocate',
      capabilities: [
        { name: 'Opportunity Analysis', description: 'Identifies potential gains and efficiencies' },
        { name: 'Best-Case Planning', description: 'Plans for optimal outcomes' },
      ],
      bias: 'positive',
    },
  },
  pessimist: {
    name: 'pessimist', 
    description: 'Adversary agent that argues AGAINST proposals - identifies risks, failure modes, and worst-case scenarios',
    input_content_types: ['text/plain', 'application/json'],
    output_content_types: ['application/json'],
    metadata: {
      role: 'adversary',
      capabilities: [
        { name: 'Risk Analysis', description: 'Identifies vulnerabilities and failure modes' },
        { name: 'Worst-Case Planning', description: 'Plans for challenging scenarios' },
      ],
      bias: 'negative',
    },
  },
  arbiter: {
    name: 'arbiter',
    description: 'Neutral judge agent that weighs arguments and makes reasoned decisions - does NOT merge, DECIDES',
    input_content_types: ['text/plain', 'application/json'],
    output_content_types: ['application/json'],
    metadata: {
      role: 'judge',
      capabilities: [
        { name: 'Evidence Weighing', description: 'Evaluates strength of arguments' },
        { name: 'Decision Making', description: 'Makes final calls with justification' },
      ],
      bias: 'neutral',
    },
  },
};

// =============================================================================
// Prompts
// =============================================================================

const PROMPTS = {
  optimist_opening: `You are the OPTIMIST in a formal debate about implementing a software project.

Your role: ADVOCATE - Argue FOR the most efficient, innovative approach.

Focus on:
- Quick wins and parallel work opportunities
- Modern tools and patterns that accelerate delivery
- Best-case timelines with proper conditions
- Innovation opportunities others might miss

You will face a PESSIMIST who will challenge your points. Be prepared to defend them.

Respond with valid JSON only:
{
  "position": "Your overall stance (2-3 sentences)",
  "keyPoints": ["Point 1", "Point 2", ...],
  "evidence": ["Supporting evidence/reasoning for each point"],
  "proposedTasks": [
    {"id": 1, "title": "Task", "description": "What and why", "priority": "high|medium|low", "dependencies": [], "estimateHours": 8}
  ]
}`,

  pessimist_opening: `You are the PESSIMIST in a formal debate about implementing a software project.

Your role: ADVERSARY - Argue for the most CAUTIOUS, risk-aware approach.

Focus on:
- Security vulnerabilities and how to prevent them
- Scalability limits and failure modes
- Dependency risks and mitigation strategies
- Edge cases that could cause production issues

You will face an OPTIMIST who will challenge your points. Be prepared to defend them.

Respond with valid JSON only:
{
  "position": "Your overall stance (2-3 sentences)",
  "keyPoints": ["Point 1", "Point 2", ...],
  "evidence": ["Supporting evidence/reasoning for each point"],
  "proposedTasks": [
    {"id": 1, "title": "Task", "description": "What and why", "priority": "high|medium|low", "dependencies": [], "estimateHours": 8}
  ]
}`,

  optimist_rebuttal: `You are the OPTIMIST responding to the PESSIMIST's arguments.

Review their points and provide rebuttals. Be intellectually honest:
- Challenge weak arguments with evidence
- CONCEDE valid points (this strengthens your credibility)
- Propose alternatives where you disagree

Respond with valid JSON only:
{
  "targetPoints": ["Quote the specific pessimist points you're addressing"],
  "counterArguments": ["Your rebuttal to each point"],
  "concessions": ["Points where the pessimist is actually right - be honest"]
}`,

  pessimist_rebuttal: `You are the PESSIMIST responding to the OPTIMIST's arguments.

Review their points and provide rebuttals. Be intellectually honest:
- Challenge unrealistic assumptions with evidence
- CONCEDE valid points (this strengthens your credibility)  
- Point out risks they've overlooked

Respond with valid JSON only:
{
  "targetPoints": ["Quote the specific optimist points you're addressing"],
  "counterArguments": ["Your rebuttal to each point"],
  "concessions": ["Points where the optimist is actually right - be honest"]
}`,

  arbiter_decision: `You are the ARBITER - a neutral judge making the final decision.

You have heard both sides:
- OPTIMIST: Argued for efficiency and opportunity
- PESSIMIST: Argued for caution and risk mitigation

Your job is NOT to merge or compromise. Your job is to DECIDE:
- Which points are correct?
- Where should we be optimistic vs cautious?
- What is the RIGHT approach given the evidence?

Be decisive. Wishy-washy "let's do both" is a cop-out. Make calls.

Respond with valid JSON only:
{
  "verdict": "Clear statement of the chosen approach",
  "reasoning": "Why this approach wins (2-3 sentences)",
  "optimistWins": ["Points where optimist was RIGHT and we'll follow their advice"],
  "pessimistWins": ["Points where pessimist was RIGHT and we'll follow their advice"],
  "synthesis": ["Points that genuinely needed elements from both (use sparingly)"],
  "finalTasks": [
    {"id": 1, "title": "Task", "description": "What to do", "priority": "high|medium|low", "dependencies": [], "estimateHours": 8, "rationale": "Why this task (who won this point)"}
  ],
  "confidenceScore": 0.85
}`,

  research: `Analyze this PRD and provide context for a debate between optimist and pessimist planners.

Identify:
- Key technical decisions to be made
- Areas where risk/reward tradeoffs exist
- Industry best practices relevant to this project
- Potential pitfalls from similar projects

Respond with valid JSON only:
{
  "summary": "Brief context summary",
  "findings": ["Key finding 1", "Key finding 2", ...]
}`,
};

// Domain-specific additions
const DOMAIN_PROMPTS: Record<DomainFocus, string> = {
  general: '',
  security: '\n\nSECURITY FOCUS: Emphasize auth, encryption, input validation, OWASP top 10.',
  performance: '\n\nPERFORMANCE FOCUS: Emphasize caching, connection pooling, async patterns, load testing.',
  compliance: '\n\nCOMPLIANCE FOCUS: Emphasize GDPR, audit logging, data retention, PII handling.',
};

const DEPTH_CONFIG: Record<DebateDepth, { rounds: number }> = {
  light: { rounds: 1 },
  medium: { rounds: 2 },
  deep: { rounds: 3 },
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

async function queryProvider(
  config: ProviderConfig,
  systemPrompt: string,
  userPrompt: string
): Promise<{ text: string; usage: TokenUsage }> {
  const provider = getAvailableProvider(config.provider);
  const model = config.model || provider.defaultModel;
  
  logger.debug(`Querying ${config.provider}/${model}`);
  
  const response = await provider.generate(userPrompt, systemPrompt, {
    maxTokens: 8192,
    temperature: 0.7,
  }, model);
  
  if (!response.success) {
    throw new Error(`Provider ${config.provider} failed: ${response.error}`);
  }
  
  return { text: response.text || '', usage: response.usage };
}

// =============================================================================
// Phase Implementations
// =============================================================================

async function runResearch(
  prd: string,
  config: AdversarialDebateConfig
): Promise<{ research: { summary: string; findings: string[] }; usage: TokenUsage }> {
  logger.info('=== RESEARCH PHASE ===');
  
  const { text, usage } = await queryProvider(
    config.providers.arbiter, // Use arbiter for neutral research
    PROMPTS.research + DOMAIN_PROMPTS[config.domainFocus],
    `PRD:\n${prd}`
  );
  
  const parsed = parseJSON(text, { summary: '', findings: [] });
  return { research: parsed, usage };
}

async function runOpeningArguments(
  prd: string,
  research: { summary: string; findings: string[] } | null,
  config: AdversarialDebateConfig
): Promise<{ optimist: OpeningArgument; pessimist: OpeningArgument; usage: TokenUsage }> {
  logger.info('=== OPENING ARGUMENTS ===');
  
  const context = research 
    ? `Research Context:\n${research.summary}\n\nPRD:\n${prd}`
    : `PRD:\n${prd}`;
  
  const domainSuffix = DOMAIN_PROMPTS[config.domainFocus];
  
  // Run both opening arguments in parallel
  const [optimistResult, pessimistResult] = await Promise.all([
    queryProvider(config.providers.optimist, PROMPTS.optimist_opening + domainSuffix, context),
    queryProvider(config.providers.pessimist, PROMPTS.pessimist_opening + domainSuffix, context),
  ]);
  
  const totalUsage: TokenUsage = { input_tokens: 0, output_tokens: 0, total_tokens: 0 };
  addUsage(totalUsage, optimistResult.usage);
  addUsage(totalUsage, pessimistResult.usage);
  
  const defaultOpening: OpeningArgument = { position: '', keyPoints: [], evidence: [], proposedTasks: [] };
  
  return {
    optimist: parseJSON(optimistResult.text, defaultOpening),
    pessimist: parseJSON(pessimistResult.text, defaultOpening),
    usage: totalUsage,
  };
}

async function runDebateRounds(
  openings: { optimist: OpeningArgument; pessimist: OpeningArgument },
  config: AdversarialDebateConfig
): Promise<{ rounds: DebateExchange[]; usage: TokenUsage }> {
  const { rounds: maxRounds } = DEPTH_CONFIG[config.depth];
  logger.info(`=== DEBATE PHASE (${maxRounds} rounds) ===`);
  
  const totalUsage: TokenUsage = { input_tokens: 0, output_tokens: 0, total_tokens: 0 };
  const rounds: DebateExchange[] = [];
  
  let currentOptimistPosition = openings.optimist;
  let currentPessimistPosition = openings.pessimist;
  
  for (let round = 1; round <= maxRounds; round++) {
    logger.info(`Round ${round}/${maxRounds}`);
    
    // Format opponent's position for rebuttal
    const optimistContext = `PESSIMIST's position:\n${currentPessimistPosition.position}\n\nTheir key points:\n${currentPessimistPosition.keyPoints.map((p, i) => `${i+1}. ${p}`).join('\n')}`;
    const pessimistContext = `OPTIMIST's position:\n${currentOptimistPosition.position}\n\nTheir key points:\n${currentOptimistPosition.keyPoints.map((p, i) => `${i+1}. ${p}`).join('\n')}`;
    
    // Run rebuttals in parallel
    const [optimistRebuttal, pessimistRebuttal] = await Promise.all([
      queryProvider(config.providers.optimist, PROMPTS.optimist_rebuttal, optimistContext),
      queryProvider(config.providers.pessimist, PROMPTS.pessimist_rebuttal, pessimistContext),
    ]);
    
    addUsage(totalUsage, optimistRebuttal.usage);
    addUsage(totalUsage, pessimistRebuttal.usage);
    
    const defaultRebuttal: Rebuttal = { targetPoints: [], counterArguments: [], concessions: [] };
    
    const exchange: DebateExchange = {
      round,
      optimistRebuttal: parseJSON(optimistRebuttal.text, defaultRebuttal),
      pessimistRebuttal: parseJSON(pessimistRebuttal.text, defaultRebuttal),
    };
    
    rounds.push(exchange);
    
    // Check for convergence (both making same concessions)
    if (round > 1) {
      const prevExchange = rounds[round - 2];
      const sameOptimistConcessions = JSON.stringify(exchange.optimistRebuttal.concessions) === 
                                       JSON.stringify(prevExchange.optimistRebuttal.concessions);
      const samePessimistConcessions = JSON.stringify(exchange.pessimistRebuttal.concessions) === 
                                        JSON.stringify(prevExchange.pessimistRebuttal.concessions);
      if (sameOptimistConcessions && samePessimistConcessions) {
        logger.info('Debate converged - both sides stable');
        break;
      }
    }
  }
  
  return { rounds, usage: totalUsage };
}

async function runArbiterDecision(
  openings: { optimist: OpeningArgument; pessimist: OpeningArgument },
  rounds: DebateExchange[],
  config: AdversarialDebateConfig
): Promise<{ decision: ArbiterDecision; usage: TokenUsage }> {
  logger.info('=== ARBITER DECISION ===');
  
  // Compile the full debate transcript for the arbiter
  const transcript = `
## OPENING ARGUMENTS

### OPTIMIST
Position: ${openings.optimist.position}
Key Points:
${openings.optimist.keyPoints.map((p, i) => `${i+1}. ${p}`).join('\n')}

### PESSIMIST  
Position: ${openings.pessimist.position}
Key Points:
${openings.pessimist.keyPoints.map((p, i) => `${i+1}. ${p}`).join('\n')}

## DEBATE ROUNDS

${rounds.map(r => `
### Round ${r.round}

OPTIMIST's Rebuttals:
${r.optimistRebuttal.counterArguments.map((a, i) => `- ${a}`).join('\n')}
OPTIMIST Concedes: ${r.optimistRebuttal.concessions.join(', ') || 'Nothing'}

PESSIMIST's Rebuttals:
${r.pessimistRebuttal.counterArguments.map((a, i) => `- ${a}`).join('\n')}
PESSIMIST Concedes: ${r.pessimistRebuttal.concessions.join(', ') || 'Nothing'}
`).join('\n')}

## PROPOSED TASKS

### From OPTIMIST:
${openings.optimist.proposedTasks.map(t => `- ${t.title}: ${t.description}`).join('\n')}

### From PESSIMIST:
${openings.pessimist.proposedTasks.map(t => `- ${t.title}: ${t.description}`).join('\n')}
`;

  const { text, usage } = await queryProvider(
    config.providers.arbiter,
    PROMPTS.arbiter_decision + DOMAIN_PROMPTS[config.domainFocus],
    transcript
  );
  
  const defaultDecision: ArbiterDecision = {
    verdict: '',
    reasoning: '',
    optimistWins: [],
    pessimistWins: [],
    synthesis: [],
    finalTasks: [],
    confidenceScore: 0,
  };
  
  return { decision: parseJSON(text, defaultDecision), usage };
}

// =============================================================================
// Main Entry Point
// =============================================================================

const DEFAULT_PROVIDERS: AdversarialDebateConfig['providers'] = {
  optimist: { provider: 'claude' },
  pessimist: { provider: 'minimax' },  // Different model = different "personality"
  arbiter: { provider: 'claude' },     // Most capable for judgment
};

export async function runAdversarialDebate(
  prdContent: string,
  partialConfig: Partial<AdversarialDebateConfig> = {}
): Promise<AdversarialDebateResult> {
  const config: AdversarialDebateConfig = {
    depth: partialConfig.depth ?? 'medium',
    domainFocus: partialConfig.domainFocus ?? 'general',
    skipResearch: partialConfig.skipResearch ?? false,
    providers: {
      ...DEFAULT_PROVIDERS,
      ...partialConfig.providers,
    },
  };
  
  const totalUsage: TokenUsage = { input_tokens: 0, output_tokens: 0, total_tokens: 0 };
  
  logger.info('Starting adversarial debate', {
    depth: config.depth,
    domainFocus: config.domainFocus,
    providers: {
      optimist: config.providers.optimist.provider,
      pessimist: config.providers.pessimist.provider,
      arbiter: config.providers.arbiter.provider,
    },
  });
  
  try {
    // Phase 1: Research (optional)
    let research: { summary: string; findings: string[] } | null = null;
    if (!config.skipResearch) {
      const r = await runResearch(prdContent, config);
      research = r.research;
      addUsage(totalUsage, r.usage);
    }
    
    // Phase 2: Opening Arguments (parallel)
    const { optimist, pessimist, usage: openingUsage } = await runOpeningArguments(
      prdContent, research, config
    );
    addUsage(totalUsage, openingUsage);
    
    // Phase 3: Debate Rounds
    const { rounds, usage: debateUsage } = await runDebateRounds(
      { optimist, pessimist }, config
    );
    addUsage(totalUsage, debateUsage);
    
    // Phase 4: Arbiter Decision
    const { decision, usage: decisionUsage } = await runArbiterDecision(
      { optimist, pessimist }, rounds, config
    );
    addUsage(totalUsage, decisionUsage);
    
    logger.info('=== DEBATE COMPLETE ===', {
      totalTokens: totalUsage.total_tokens,
      taskCount: decision.finalTasks.length,
      confidence: decision.confidenceScore,
      optimistWins: decision.optimistWins.length,
      pessimistWins: decision.pessimistWins.length,
    });
    
    return {
      success: true,
      config,
      research: research ?? undefined,
      openingArguments: { optimist, pessimist },
      debateRounds: rounds,
      arbiterDecision: decision,
      usage: totalUsage,
    };
  } catch (e) {
    const error = e instanceof Error ? e.message : 'Unknown error';
    logger.error('Adversarial debate failed', { error });
    return { success: false, config, error, usage: totalUsage };
  }
}

export const DEFAULT_ADVERSARIAL_CONFIG: AdversarialDebateConfig = {
  depth: 'medium',
  domainFocus: 'general',
  skipResearch: false,
  providers: DEFAULT_PROVIDERS,
};
