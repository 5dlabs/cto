/**
 * Critic/Validator Orchestrator - Multi-model collaboration.
 * 
 * Implements the critic/validator pattern where one model generates content
 * and another model reviews/critiques it, with optional refinement loops.
 */

import type { TokenUsage } from '../types';
import type {
  CriticResult,
  MultiModelConfig,
  MultiModelResponse,
  ProviderName,
  ProviderResponse,
} from '../providers/types';
import { getAvailableProvider, DEFAULT_MULTI_MODEL_CONFIG } from '../providers';
import {
  getCriticSystemPrompt,
  buildCriticPrompt,
  buildRefinerPrompt,
  REFINER_SYSTEM_PROMPT,
} from '../prompts/critic-templates';

// =============================================================================
// Types
// =============================================================================

/**
 * Options for the generate-with-critic operation.
 */
export interface GenerateWithCriticOptions {
  /** System prompt for the generator */
  systemPrompt: string;
  /** User prompt (main input) */
  userPrompt: string;
  /** Optional prefill for the generator */
  prefill?: string;
  /** Multi-model configuration */
  config?: Partial<MultiModelConfig>;
  /** Context for the critic */
  criticContext?: string;
  /** Content type for specialized critics */
  contentType?: 'tasks' | 'code' | 'docs' | 'general';
  /** Custom criteria for evaluation */
  evaluationCriteria?: string;
}

/**
 * Internal state for tracking usage across models.
 */
interface UsageTracker {
  byProvider: Record<ProviderName, TokenUsage>;
  total: TokenUsage;
}

/**
 * Structured dialog entry for generator/critic communication.
 * Used for debugging and observability of the multi-model collaboration.
 */
export interface MultiModelDialog {
  round: number;
  generator: {
    provider: string;
    model: string;
    output: string;
    tokens: number;
    latencyMs: number;
  };
  critic: {
    provider: string;
    model: string;
    approved: boolean;
    confidence: number;
    issues: Array<{
      severity: string;
      location: string;
      description: string;
    }>;
    reasoning: string;
    tokens: number;
    latencyMs: number;
  };
}

/**
 * Log a structured dialog entry for debugging.
 */
function logDialogEntry(dialog: MultiModelDialog): void {
  const status = dialog.critic.approved ? '✓ APPROVED' : '✗ NEEDS REFINEMENT';
  const issueCount = dialog.critic.issues.length;
  
  console.error(`[MULTI-MODEL] Round ${dialog.round}: ${status}`);
  console.error(`  Generator (${dialog.generator.provider}/${dialog.generator.model}): ${dialog.generator.tokens} tokens, ${dialog.generator.latencyMs}ms`);
  console.error(`  Critic (${dialog.critic.provider}/${dialog.critic.model}): confidence=${dialog.critic.confidence.toFixed(2)}, issues=${issueCount}, ${dialog.critic.tokens} tokens, ${dialog.critic.latencyMs}ms`);
  
  if (issueCount > 0 && !dialog.critic.approved) {
    console.error(`  Issues:`);
    for (const issue of dialog.critic.issues.slice(0, 3)) {
      console.error(`    - [${issue.severity}] ${issue.description.slice(0, 100)}${issue.description.length > 100 ? '...' : ''}`);
    }
    if (issueCount > 3) {
      console.error(`    ... and ${issueCount - 3} more issues`);
    }
  }
  
  if (dialog.critic.reasoning) {
    console.error(`  Reasoning: ${dialog.critic.reasoning.slice(0, 150)}${dialog.critic.reasoning.length > 150 ? '...' : ''}`);
  }
}

// =============================================================================
// Helper Functions
// =============================================================================

/**
 * Create an empty usage tracker.
 */
function createUsageTracker(): UsageTracker {
  return {
    byProvider: {
      claude: { input_tokens: 0, output_tokens: 0, total_tokens: 0 },
      minimax: { input_tokens: 0, output_tokens: 0, total_tokens: 0 },
      codex: { input_tokens: 0, output_tokens: 0, total_tokens: 0 },
    },
    total: { input_tokens: 0, output_tokens: 0, total_tokens: 0 },
  };
}

/**
 * Add provider response usage to tracker.
 */
function addUsage(tracker: UsageTracker, response: ProviderResponse): void {
  const usage = response.usage;
  const provider = response.provider;
  
  tracker.byProvider[provider].input_tokens += usage.input_tokens;
  tracker.byProvider[provider].output_tokens += usage.output_tokens;
  tracker.byProvider[provider].total_tokens += usage.total_tokens;
  
  tracker.total.input_tokens += usage.input_tokens;
  tracker.total.output_tokens += usage.output_tokens;
  tracker.total.total_tokens += usage.total_tokens;
}

/**
 * Parse critic JSON response.
 */
function parseCriticResponse(text: string): CriticResult | null {
  try {
    // Try to extract JSON from the response (it might have markdown code blocks)
    let jsonStr = text;
    
    // Check for JSON in code blocks
    const jsonMatch = text.match(/```(?:json)?\s*([\s\S]*?)```/);
    if (jsonMatch && jsonMatch[1]) {
      jsonStr = jsonMatch[1].trim();
    }
    
    const parsed = JSON.parse(jsonStr);
    
    // Validate the structure
    if (typeof parsed.approved !== 'boolean') {
      return null;
    }
    
    return {
      approved: parsed.approved,
      issues: Array.isArray(parsed.issues) ? parsed.issues.map((issue: Record<string, unknown>) => ({
        severity: (issue.severity as string) || 'minor',
        location: (issue.location as string) || 'unknown',
        description: (issue.description as string) || '',
        suggestion: (issue.suggestion as string) || '',
      })) : [],
      suggestions: Array.isArray(parsed.suggestions) ? parsed.suggestions : [],
      confidence: typeof parsed.confidence === 'number' ? parsed.confidence : 0.5,
      reasoning: parsed.reasoning || '',
    };
  } catch {
    return null;
  }
}

/**
 * Create a fallback critic result when parsing fails.
 */
function createFallbackCriticResult(rawResponse: string): CriticResult {
  return {
    approved: true, // Assume approved if we can't parse
    issues: [],
    suggestions: ['Unable to parse critic response - review manually'],
    confidence: 0.5,
    reasoning: `Raw response: ${rawResponse.slice(0, 200)}...`,
  };
}

// =============================================================================
// Main Orchestration Functions
// =============================================================================

/**
 * Run the critic to evaluate generated content.
 */
export async function runCritic(
  content: string,
  context: string,
  config: MultiModelConfig,
  contentType?: string,
  originalPrompt?: string,
  evaluationCriteria?: string
): Promise<{ result: CriticResult; response: ProviderResponse }> {
  const criticProvider = getAvailableProvider(config.critic);
  
  const systemPrompt = getCriticSystemPrompt(contentType);
  const userPrompt = buildCriticPrompt({
    content,
    context,
    originalPrompt,
    criteria: evaluationCriteria,
  });
  
  const response = await criticProvider.generate(
    userPrompt,
    systemPrompt,
    { jsonMode: true, temperature: 0.3 },
    config.criticModel
  );
  
  if (!response.success || !response.text) {
    return {
      result: {
        approved: false,
        issues: [{
          severity: 'critical',
          location: 'critic',
          description: `Critic failed: ${response.error || 'unknown error'}`,
          suggestion: 'Retry or use a different critic model',
        }],
        suggestions: [],
        confidence: 0,
        reasoning: response.error,
      },
      response,
    };
  }
  
  const parsed = parseCriticResponse(response.text);
  const result = parsed || createFallbackCriticResult(response.text);
  
  return { result, response };
}

/**
 * Run the refiner to improve content based on critic feedback.
 */
export async function runRefiner(
  content: string,
  originalPrompt: string,
  criticResult: CriticResult,
  config: MultiModelConfig
): Promise<ProviderResponse> {
  const generatorProvider = getAvailableProvider(config.generator);
  
  const userPrompt = buildRefinerPrompt({
    content,
    originalPrompt,
    generator: config.generator,
    critic: config.critic,
    approved: criticResult.approved,
    confidence: criticResult.confidence,
    issues: criticResult.issues,
    suggestions: criticResult.suggestions,
    reasoning: criticResult.reasoning || '',
  });
  
  return generatorProvider.generate(
    userPrompt,
    REFINER_SYSTEM_PROMPT,
    { temperature: 0.5 },
    config.generatorModel
  );
}

/**
 * Generate content with critic validation and optional refinement.
 */
export async function generateWithCritic(
  options: GenerateWithCriticOptions
): Promise<MultiModelResponse> {
  const startTime = Date.now();
  const config: MultiModelConfig = {
    ...DEFAULT_MULTI_MODEL_CONFIG,
    ...options.config,
  };
  
  const usageTracker = createUsageTracker();
  const dialogHistory: MultiModelDialog[] = [];
  let refinements = 0;
  let currentContent = '';
  let finalCriticResult: CriticResult | undefined;
  
  console.error(`[MULTI-MODEL] Starting generation with critic (generator=${config.generator}, critic=${config.critic}, maxRefinements=${config.maxRefinements})`);
  
  try {
    // Step 1: Initial generation
    const generatorProvider = getAvailableProvider(config.generator);
    
    let prompt = options.userPrompt;
    if (options.prefill) {
      prompt = `${options.userPrompt}\n\n${options.prefill}`;
    }
    
    const genStartTime = Date.now();
    const generateResponse = await generatorProvider.generate(
      prompt,
      options.systemPrompt,
      { temperature: 0.7 },
      config.generatorModel
    );
    const genLatencyMs = Date.now() - genStartTime;
    
    addUsage(usageTracker, generateResponse);
    
    if (!generateResponse.success || !generateResponse.text) {
      console.error(`[MULTI-MODEL] Initial generation failed: ${generateResponse.error || 'unknown error'}`);
      return {
        success: false,
        error: `Generation failed: ${generateResponse.error || 'unknown error'}`,
        refinements: 0,
        usage: usageTracker.total,
        usageByProvider: usageTracker.byProvider,
        generator: config.generator,
        critic: config.critic,
        totalLatencyMs: Date.now() - startTime,
      };
    }
    
    currentContent = generateResponse.text;
    console.error(`[MULTI-MODEL] Initial generation complete: ${generateResponse.usage.total_tokens} tokens, ${genLatencyMs}ms`);
    
    // Step 2: Critic loop
    let round = 1;
    while (refinements < config.maxRefinements) {
      const criticStartTime = Date.now();
      const { result: criticResult, response: criticResponse } = await runCritic(
        currentContent,
        options.criticContext || '',
        config,
        options.contentType,
        options.userPrompt,
        options.evaluationCriteria
      );
      const criticLatencyMs = Date.now() - criticStartTime;
      
      addUsage(usageTracker, criticResponse);
      finalCriticResult = criticResult;
      
      // Log structured dialog entry
      const dialogEntry: MultiModelDialog = {
        round,
        generator: {
          provider: config.generator,
          model: config.generatorModel || 'default',
          output: currentContent.slice(0, 500) + (currentContent.length > 500 ? '...' : ''),
          tokens: round === 1 ? generateResponse.usage.total_tokens : 0, // Only initial gen for round 1
          latencyMs: round === 1 ? genLatencyMs : 0,
        },
        critic: {
          provider: config.critic,
          model: config.criticModel || 'default',
          approved: criticResult.approved,
          confidence: criticResult.confidence,
          issues: criticResult.issues.map(i => ({
            severity: i.severity,
            location: i.location,
            description: i.description,
          })),
          reasoning: criticResult.reasoning || '',
          tokens: criticResponse.usage.total_tokens,
          latencyMs: criticLatencyMs,
        },
      };
      dialogHistory.push(dialogEntry);
      logDialogEntry(dialogEntry);
      
      // Check if approved or meets confidence threshold
      if (criticResult.approved || criticResult.confidence >= config.criticThreshold) {
        console.error(`[MULTI-MODEL] Content approved (approved=${criticResult.approved}, confidence=${criticResult.confidence.toFixed(2)} >= threshold=${config.criticThreshold})`);
        break;
      }
      
      // Check for critical issues that can't be refined
      const criticalIssues = criticResult.issues.filter(i => i.severity === 'critical');
      if (criticalIssues.length > 0 && refinements === config.maxRefinements - 1) {
        console.error(`[MULTI-MODEL] Critical issues found on last refinement, attempting final fix`);
      }
      
      // Step 3: Refine based on feedback
      refinements++;
      round++;
      console.error(`[MULTI-MODEL] Starting refinement ${refinements}/${config.maxRefinements}`);
      
      const refineStartTime = Date.now();
      const refineResponse = await runRefiner(
        currentContent,
        options.userPrompt,
        criticResult,
        config
      );
      const refineLatencyMs = Date.now() - refineStartTime;
      
      addUsage(usageTracker, refineResponse);
      
      if (!refineResponse.success || !refineResponse.text) {
        console.error(`[MULTI-MODEL] Refinement ${refinements} failed: ${refineResponse.error || 'unknown error'}`);
        break;
      }
      
      console.error(`[MULTI-MODEL] Refinement ${refinements} complete: ${refineResponse.usage.total_tokens} tokens, ${refineLatencyMs}ms`);
      currentContent = refineResponse.text;
      
      // Update the dialog entry with refiner info
      const lastEntry = dialogHistory[dialogHistory.length - 1];
      if (lastEntry) {
        lastEntry.generator.tokens = refineResponse.usage.total_tokens;
        lastEntry.generator.latencyMs = refineLatencyMs;
      }
    }
    
    // Run final critic if we haven't yet (or if we hit max refinements)
    if (!finalCriticResult || refinements === config.maxRefinements) {
      console.error(`[MULTI-MODEL] Running final critic evaluation`);
      const { result, response } = await runCritic(
        currentContent,
        options.criticContext || '',
        config,
        options.contentType,
        options.userPrompt,
        options.evaluationCriteria
      );
      addUsage(usageTracker, response);
      finalCriticResult = result;
    }
    
    const totalLatencyMs = Date.now() - startTime;
    console.error(`[MULTI-MODEL] Complete: ${refinements} refinements, ${usageTracker.total.total_tokens} total tokens, ${totalLatencyMs}ms`);
    
    return {
      success: true,
      text: currentContent,
      refinements,
      criticResult: finalCriticResult,
      usage: usageTracker.total,
      usageByProvider: usageTracker.byProvider,
      generator: config.generator,
      critic: config.critic,
      totalLatencyMs,
    };
  } catch (e) {
    const error = e instanceof Error ? e.message : 'Unknown error';
    console.error(`[MULTI-MODEL] Orchestration error: ${error}`);
    return {
      success: false,
      error: `Orchestration error: ${error}`,
      refinements,
      criticResult: finalCriticResult,
      usage: usageTracker.total,
      usageByProvider: usageTracker.byProvider,
      generator: config.generator,
      critic: config.critic,
      totalLatencyMs: Date.now() - startTime,
    };
  }
}

/**
 * Validate content without generation (critic-only mode).
 */
export async function validateOnly(
  content: string,
  options: {
    criticProvider?: ProviderName;
    criticModel?: string;
    context?: string;
    contentType?: string;
    evaluationCriteria?: string;
  } = {}
): Promise<{ result: CriticResult; usage: TokenUsage }> {
  const config: MultiModelConfig = {
    ...DEFAULT_MULTI_MODEL_CONFIG,
    critic: options.criticProvider || 'minimax',
    criticModel: options.criticModel,
  };
  
  const { result, response } = await runCritic(
    content,
    options.context || '',
    config,
    options.contentType,
    undefined,
    options.evaluationCriteria
  );
  
  return { result, usage: response.usage };
}
