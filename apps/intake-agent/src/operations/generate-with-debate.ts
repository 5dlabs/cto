/**
 * Generate with Debate - Multi-perspective debate with arbiter decision.
 * 
 * Implements the Advocate-Adversary-Arbiter pattern:
 * 1. Research - Gather information
 * 2. Advocate - Create proposal
 * 3. Adversary - Critique and debate
 * 4. Arbiter - Make final decision (not mushy merging!)
 */

import type { TokenUsage } from '../types';
import type {
  DebateResult,
  MultiModelConfig,
  ProviderName,
  ProviderResponse,
} from '../providers/types';
import { getAvailableProvider, DEFAULT_MULTI_MODEL_CONFIG } from '../providers';
import {
  getAdvocateSystemPrompt,
  buildAdvocatePrompt,
  getAdversarySystemPrompt,
  buildAdversaryPrompt,
  getArbiterSystemPrompt,
  buildArbiterPrompt,
} from '../prompts/debate-templates';
import { parseCriticResponse, createFallbackCriticResult } from '../orchestration/critic-validator';

// =============================================================================
// Types
// =============================================================================

export interface GenerateWithDebateOptions {
  /** System prompt for the advocate */
  systemPrompt: string;
  /** User prompt (main input) */
  userPrompt: string;
  /** Optional prefill for the advocate */
  prefill?: string;
  /** Multi-model configuration */
  config?: Partial<MultiModelConfig>;
  /** Context for the debate */
  context?: string;
  /** Content type for specialized debates */
  contentType?: 'tasks' | 'code' | 'docs' | 'general';
}

interface DebateDialog {
  round: number;
  advocate: {
    provider: string;
    model: string;
    proposal: string;
    tokens: number;
    latencyMs: number;
  };
  adversary: {
    provider: string;
    model: string;
    critique: string;
    concerns: string[];
    confidence: number;
    tokens: number;
    latencyMs: number;
  };
  arbiter: {
    provider: string;
    model: string;
    decision: 'advocate' | 'adversary' | 'revise';
    rationale: string;
    tokens: number;
    latencyMs: number;
  };
}

function logDebateEntry(dialog: DebateDialog): void {
  const decision = dialog.arbiter.decision === 'advocate' ? '✅ ADVOCATE WINS' 
    : dialog.arbiter.decision === 'adversary' ? '❌ ADVERSARY WINS'
    : '🔄 REVISE NEEDED';
  
  console.error(`[DEBATE] Round ${dialog.round}: ${decision}`);
  console.error(`  Advocate (${dialog.advocate.provider}/${dialog.advocate.model}): ${dialog.advocate.tokens} tokens, ${dialog.advocate.latencyMs}ms`);
  console.error(`  Adversary (${dialog.adversary.provider}/${dialog.adversary.model}): concerns=${dialog.adversary.concerns.length}, ${dialog.adversary.tokens} tokens, ${dialog.adversary.latencyMs}ms`);
  console.error(`  Arbiter (${dialog.arbiter.provider}/${dialog.arbiter.model}): ${dialog.arbiter.tokens} tokens, ${dialog.arbiter.latencyMs}ms`);
  console.error(`  Rationale: ${dialog.arbiter.rationale.slice(0, 150)}...`);
}

// =============================================================================
// Main Debate Functions
// =============================================================================

/**
 * Run the advocate to create a proposal
 */
async function runAdvocate(
  options: GenerateWithDebateOptions,
  config: MultiModelConfig
): Promise<{ proposal: string; response: ProviderResponse }> {
  const advocateProvider = getAvailableProvider(config.generator);
  
  const systemPrompt = getAdvocateSystemPrompt(options.contentType);
  const userPrompt = buildAdvocatePrompt({
    task: options.userPrompt,
    context: options.context || '',
    prefill: options.prefill,
  });
  
  console.error(`[DEBATE] Calling advocate generate with model: ${config.generatorModel || 'default'}`);
  
  try {
    const response = await advocateProvider.generate(
      userPrompt,
      systemPrompt,
      { temperature: 0.7 },
      config.generatorModel
    );
    
    console.error(`[DEBATE] Advocate response: success=${response.success}, text_len=${response.text?.length || 0}`);
    
    if (!response.text) {
      throw new Error(`Advocate returned no text: ${response.error || 'unknown error'}`);
    }
    
    return { proposal: response.text, response };
  } catch (e) {
    console.error(`[DEBATE] Advocate error: ${e instanceof Error ? e.message : 'unknown'}`);
    throw e;
  }
}

/**
 * Run the adversary to critique the proposal
 */
async function runAdversary(
  proposal: string,
  originalTask: string,
  context: string,
  config: MultiModelConfig,
  contentType?: string
): Promise<{ critique: DebateResult; response: ProviderResponse }> {
  const adversaryProvider = getAvailableProvider(config.critic);
  
  const systemPrompt = getAdversarySystemPrompt(contentType);
  const userPrompt = buildAdversaryPrompt({
    proposal,
    originalTask,
    context,
  });
  
  const response = await adversaryProvider.generate(
    userPrompt,
    systemPrompt,
    { jsonMode: true, temperature: 0.3 },
    config.criticModel
  );
  
  if (!response.success || !response.text) {
    return {
      critique: {
        approved: false,
        issues: [{
          severity: 'critical',
          location: 'adversary',
          description: `Adversary failed: ${response.error || 'unknown error'}`,
          suggestion: 'Retry debate',
        }],
        suggestions: [],
        confidence: 0,
        reasoning: response.error,
      },
      response,
    };
  }
  
  const parsed = parseCriticResponse(response.text);
  const critique = parsed || createFallbackCriticResult(response.text);
  
  return { critique, response };
}

/**
 * Run the arbiter to make a final decision
 */
async function runArbiter(
  proposal: string,
  critique: DebateResult,
  originalTask: string,
  context: string,
  config: MultiModelConfig,
  contentType?: string
): Promise<{ decision: 'advocate' | 'adversary' | 'revise'; rationale: string; response: ProviderResponse }> {
  const arbiterProvider = getAvailableProvider('claude'); // Arbiter should use Claude for best judgment
  
  const systemPrompt = getArbiterSystemPrompt(contentType);
  const userPrompt = buildArbiterPrompt({
    proposal,
    critique,
    originalTask,
    context,
  });
  
  const response = await arbiterProvider.generate(
    userPrompt,
    systemPrompt,
    { temperature: 0.2 }, // Low temp for consistent decisions
    config.criticModel
  );
  
  if (!response.success || !response.text) {
    return {
      decision: 'revise',
      rationale: `Arbiter failed: ${response.error || 'unknown error'}`,
      response,
    };
  }
  
  // Parse arbiter decision
  const decisionMatch = response.text.match(/(?:decision|verdict)[:\s]+(\w+)/i);
  const rationaleMatch = response.text.match(/(?:rationale|reasoning|explanation)[:\s]+([\s\S]+?)(?:\n\n|$)/i);
  
  const decision = decisionMatch?.[1]?.toLowerCase();
  const finalDecision: 'advocate' | 'adversary' | 'revise' = 
    decision === 'advocate' ? 'advocate' :
    decision === 'adversary' ? 'adversary' : 'revise';
  
  return {
    decision: finalDecision,
    rationale: rationaleMatch?.[1]?.trim() || response.text.slice(0, 300),
    response,
  };
}

/**
 * Generate content with multi-perspective debate and arbiter decision.
 */
export async function generateWithDebate(
  options: GenerateWithDebateOptions
): Promise<{
  success: boolean;
  text?: string;
  decision?: 'advocate' | 'adversary' | 'revise';
  rationale?: string;
  debateHistory?: DebateDialog[];
  usage: TokenUsage;
  provider: string;
  totalLatencyMs: number;
  error?: string;
}> {
  const startTime = Date.now();
  const config: MultiModelConfig = {
    ...DEFAULT_MULTI_MODEL_CONFIG,
    ...options.config,
  };
  
  // Use Claude for arbiter (best judgment)
  const finalConfig = {
    ...config,
    critic: 'claude',
    criticModel: config.criticModel || 'claude-sonnet-4-20250514',
  };
  
  const debateHistory: DebateDialog[] = [];
  let currentProposal = '';
  let totalUsage: TokenUsage = { input_tokens: 0, output_tokens: 0, total_tokens: 0 };
  
  console.error(`[DEBATE] Starting debate (advocate=${config.generator}, adversary=${config.critic}, arbiter=claude)`);
  
  try {
    // Phase 1: Advocate creates proposal
    const advocateStart = Date.now();
    const { proposal, response: advocateResponse } = await runAdvocate(options, config);
    const advocateLatency = Date.now() - advocateStart;
    
    totalUsage.input_tokens += advocateResponse.usage.input_tokens;
    totalUsage.output_tokens += advocateResponse.usage.output_tokens;
    totalUsage.total_tokens += advocateResponse.usage.total_tokens;
    
    currentProposal = proposal;
    console.error(`[DEBATE] Advocate complete: ${advocateResponse.usage.total_tokens} tokens, ${advocateLatency}ms`);
    
    // Phase 2+3: Debate loop (adversary critiques, arbiter decides)
    for (let round = 1; round <= config.maxRefinements; round++) {
      console.error(`[DEBATE] Round ${round}: Adversary critique`);
      
      // Adversary critiques
      const adversaryStart = Date.now();
      const { critique, response: adversaryResponse } = await runAdversary(
        currentProposal,
        options.userPrompt,
        options.context || '',
        finalConfig,
        options.contentType
      );
      const adversaryLatency = Date.now() - adversaryStart;
      
      totalUsage.input_tokens += adversaryResponse.usage.input_tokens;
      totalUsage.output_tokens += adversaryResponse.usage.output_tokens;
      totalUsage.total_tokens += adversaryResponse.usage.total_tokens;
      
      // Arbiter decides
      console.error(`[DEBATE] Round ${round}: Arbiter decision`);
      const arbiterStart = Date.now();
      const { decision, rationale, response: arbiterResponse } = await runArbiter(
        currentProposal,
        critique,
        options.userPrompt,
        options.context || '',
        finalConfig,
        options.contentType
      );
      const arbiterLatency = Date.now() - arbiterStart;
      
      totalUsage.input_tokens += arbiterResponse.usage.input_tokens;
      totalUsage.output_tokens += arbiterResponse.usage.output_tokens;
      totalUsage.total_tokens += arbiterResponse.usage.total_tokens;
      
      // Log the round
      const dialog: DebateDialog = {
        round,
        advocate: {
          provider: config.generator,
          model: config.generatorModel || 'default',
          proposal: currentProposal.slice(0, 200) + '...',
          tokens: advocateResponse.usage.total_tokens,
          latencyMs: advocateLatency,
        },
        adversary: {
          provider: config.critic,
          model: config.criticModel || 'default',
          critique: '',
          concerns: critique.issues.map(i => i.description),
          confidence: critique.confidence,
          tokens: adversaryResponse.usage.total_tokens,
          latencyMs: adversaryLatency,
        },
        arbiter: {
          provider: 'claude',
          model: finalConfig.criticModel || 'default',
          decision,
          rationale,
          tokens: arbiterResponse.usage.total_tokens,
          latencyMs: arbiterLatency,
        },
      };
      debateHistory.push(dialog);
      logDebateEntry(dialog);
      
      // If arbiter approves, we're done
      if (decision === 'advocate') {
        console.error(`[DEBATE] ✅ Arbiter approved advocate's proposal`);
        return {
          success: true,
          text: currentProposal,
          decision: 'advocate',
          rationale,
          debateHistory,
          usage: totalUsage,
          provider: config.generator,
          totalLatencyMs: Date.now() - startTime,
        };
      }
      
      // If arbiter says adversary wins, use critique as final output (with note)
      if (decision === 'adversary') {
        console.error(`[DEBATE] ❌ Arbiter sided with adversary - proposal needs revision`);
        return {
          success: true,
          text: `// ARBITER REJECTED - Adversary's concerns must be addressed:\n\n` +
                critique.issues.map(i => `// ${i.severity.toUpperCase()}: ${i.description}`).join('\n') +
                `\n\n// Original proposal:\n${currentProposal}`,
          decision: 'adversary',
          rationale,
          debateHistory,
          usage: totalUsage,
          provider: config.generator,
          totalLatencyMs: Date.now() - startTime,
        };
      }
      
      // If arbiter says revise, loop continues
      console.error(`[DEBATE] 🔄 Arbiter requests revision (round ${round}/${config.maxRefinements})`);
      
      // Simple revision: append critique to proposal and let advocate regenerate
      currentProposal = `[Previous proposal rejected. Arbiter rationale: ${rationale}]

Original task: ${options.userPrompt}

Please revise your proposal to address these concerns and submit again.`;
    }
    
    // Max rounds reached
    console.error(`[DEBATE] Max rounds reached (${config.maxRefinements})`);
    return {
      success: false,
      error: `Debate exceeded maximum rounds (${config.maxRefinements}) without consensus`,
      decision: 'revise',
      debateHistory,
      usage: totalUsage,
      provider: config.generator,
      totalLatencyMs: Date.now() - startTime,
    };
    
  } catch (e) {
    const error = e instanceof Error ? e.message : 'Unknown error';
    console.error(`[DEBATE] Error: ${error}`);
    return {
      success: false,
      error: `Debate orchestration error: ${error}`,
      usage: totalUsage,
      provider: config.generator,
      totalLatencyMs: Date.now() - startTime,
    };
  }
}
