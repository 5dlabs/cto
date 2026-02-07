/**
 * Type definitions for multi-model provider abstraction.
 * Enables collaboration between Claude, Minimax, and OpenAI/Codex models.
 */

import type { TokenUsage } from '../types';

// =============================================================================
// Provider Types
// =============================================================================

/**
 * Supported model provider names.
 */
export type ProviderName = 'claude' | 'minimax' | 'codex';

/**
 * Model identifiers for each provider.
 */
export interface ModelIdentifiers {
  claude: 'claude-sonnet-4-20250514' | 'claude-opus-4-20250514' | 'claude-3-5-sonnet-latest';
  minimax: 'MiniMax-M2.1' | 'MiniMax-M2.1-lightning' | 'MiniMax-M2';
  codex: 'gpt-4o' | 'gpt-4-turbo' | 'o1-preview' | 'gpt-4o-mini';
}

/**
 * Options for provider generation requests.
 */
export interface ProviderOptions {
  /** Temperature for sampling (0.0 to 1.0) */
  temperature?: number;
  /** Maximum tokens to generate */
  maxTokens?: number;
  /** Stop sequences */
  stopSequences?: string[];
  /** Request JSON output format */
  jsonMode?: boolean;
  /** Custom timeout in milliseconds */
  timeout?: number;
}

/**
 * Response from a provider generation request.
 */
export interface ProviderResponse {
  /** Whether the request succeeded */
  success: boolean;
  /** Generated text (on success) */
  text?: string;
  /** Error message (on failure) */
  error?: string;
  /** Token usage information */
  usage: TokenUsage;
  /** Model that generated the response */
  model: string;
  /** Provider name */
  provider: ProviderName;
  /** Response latency in milliseconds */
  latencyMs?: number;
}

/**
 * Interface that all model providers must implement.
 */
export interface ModelProvider {
  /** Provider name identifier */
  readonly name: ProviderName;
  
  /** Default model for this provider */
  readonly defaultModel: string;
  
  /**
   * Check if the provider is available (API key configured).
   */
  isAvailable(): boolean;
  
  /**
   * Generate text using the provider's model.
   * 
   * @param prompt - The user prompt
   * @param systemPrompt - The system prompt
   * @param options - Generation options
   * @param model - Specific model to use (optional, uses default if not specified)
   */
  generate(
    prompt: string,
    systemPrompt: string,
    options?: ProviderOptions,
    model?: string
  ): Promise<ProviderResponse>;
}

// =============================================================================
// Multi-Model Configuration
// =============================================================================

/**
 * Configuration for multi-model collaboration.
 */
export interface MultiModelConfig {
  /** Provider to use for generation */
  generator: ProviderName;
  /** Provider to use for critique/validation */
  critic: ProviderName;
  /** Maximum refinement iterations */
  maxRefinements: number;
  /** Confidence threshold for approval (0.0-1.0) */
  criticThreshold: number;
  /** Specific model for generator (optional) */
  generatorModel?: string;
  /** Specific model for critic (optional) */
  criticModel?: string;
}

/**
 * Default multi-model configuration.
 */
export const DEFAULT_MULTI_MODEL_CONFIG: MultiModelConfig = {
  generator: 'claude',
  critic: 'minimax',
  maxRefinements: 2,
  criticThreshold: 0.8,
};

// =============================================================================
// Critic Types
// =============================================================================

/**
 * Severity levels for critic issues.
 */
export type IssueSeverity = 'critical' | 'major' | 'minor';

/**
 * A single issue identified by the critic.
 */
export interface CriticIssue {
  /** Severity of the issue */
  severity: IssueSeverity;
  /** Location/context where the issue was found */
  location: string;
  /** Description of the issue */
  description: string;
  /** Suggested fix */
  suggestion: string;
}

/**
 * Result of critic validation.
 */
export interface CriticResult {
  /** Whether the content is approved */
  approved: boolean;
  /** List of issues found */
  issues: CriticIssue[];
  /** General suggestions for improvement */
  suggestions: string[];
  /** Confidence score (0.0-1.0) */
  confidence: number;
  /** Raw reasoning from the critic */
  reasoning?: string;
}

/**
 * Response from multi-model generation with critic.
 */
export interface MultiModelResponse {
  /** Whether the operation succeeded */
  success: boolean;
  /** Final generated text */
  text?: string;
  /** Error message on failure */
  error?: string;
  /** Number of refinement iterations performed */
  refinements: number;
  /** Final critic result */
  criticResult?: CriticResult;
  /** Combined token usage across all models */
  usage: TokenUsage;
  /** Breakdown of usage per provider */
  usageByProvider: Record<ProviderName, TokenUsage>;
  /** Generator provider used */
  generator: ProviderName;
  /** Critic provider used */
  critic: ProviderName;
  /** Total latency in milliseconds */
  totalLatencyMs: number;
}

// =============================================================================
// Provider Registry Types
// =============================================================================

/**
 * Registry of available providers.
 */
export interface ProviderRegistry {
  /** Get a provider by name */
  get(name: ProviderName): ModelProvider | undefined;
  /** Check if a provider is available */
  isAvailable(name: ProviderName): boolean;
  /** List all available providers */
  listAvailable(): ProviderName[];
  /** Get all registered providers */
  all(): Map<ProviderName, ModelProvider>;
}

// =============================================================================
// Payload Types for Operations
// =============================================================================

/**
 * Payload for generate_with_critic operation.
 */
export interface GenerateWithCriticPayload {
  /** System prompt for the generator */
  system_prompt: string;
  /** User prompt (main input) */
  user_prompt: string;
  /** Optional prefill for the generator */
  prefill?: string;
  /** Multi-model configuration */
  config?: Partial<MultiModelConfig>;
  /** Context for the critic to understand what to evaluate */
  critic_context?: string;
}

/**
 * Response data for generate_with_critic operation.
 */
export interface GenerateWithCriticData {
  /** Generated text */
  text: string;
  /** Number of refinements performed */
  refinements: number;
  /** Final critic result */
  critic_result: CriticResult;
  /** Usage breakdown by provider */
  usage_by_provider: Record<string, TokenUsage>;
}
