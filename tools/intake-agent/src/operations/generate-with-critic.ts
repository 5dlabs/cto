/**
 * Generate with Critic Operation - Multi-model generation with validation.
 * 
 * Uses the critic/validator pattern where one model generates content
 * and another model reviews/critiques it for quality assurance.
 */

import type {
  AgentResponse,
} from '../types';
import type {
  GenerateWithCriticPayload,
  GenerateWithCriticData,
  MultiModelConfig,
  ProviderName,
} from '../providers/types';
import { generateWithCritic as runGenerateWithCritic } from '../orchestration';
import { providerRegistry, DEFAULT_MULTI_MODEL_CONFIG } from '../providers';

/**
 * Response data for provider_status operation.
 */
export interface ProviderStatusData {
  providers: Record<string, { available: boolean; model: string }>;
}

/**
 * Check provider availability status.
 */
export function getProviderStatus(): AgentResponse<ProviderStatusData> {
  const providers: Record<string, { available: boolean; model: string }> = {};
  
  for (const [name, provider] of providerRegistry.all()) {
    providers[name] = {
      available: provider.isAvailable(),
      model: provider.defaultModel,
    };
  }
  
  return {
    success: true,
    data: { providers },
    usage: { input_tokens: 0, output_tokens: 0, total_tokens: 0 },
    model: 'none',
    provider: 'intake-agent',
  };
}

/**
 * Generate content with multi-model critic validation.
 */
export async function generateWithCriticOperation(
  payload: GenerateWithCriticPayload
): Promise<AgentResponse<GenerateWithCriticData>> {
  try {
    // Build config from payload
    const config: Partial<MultiModelConfig> = {
      ...payload.config,
    };
    
    // Validate generator and critic are different providers if specified
    if (config.generator && config.critic && config.generator === config.critic) {
      // Allow same provider but log a note - might want different models
    }
    
    const result = await runGenerateWithCritic({
      systemPrompt: payload.system_prompt,
      userPrompt: payload.user_prompt,
      prefill: payload.prefill,
      config,
      criticContext: payload.critic_context,
    });
    
    if (!result.success || !result.text) {
      return {
        success: false,
        error: result.error || 'Generation with critic failed',
        error_type: 'api_error',
        details: result.criticResult?.reasoning,
      };
    }
    
    // Build response data
    const data: GenerateWithCriticData = {
      text: result.text,
      refinements: result.refinements,
      critic_result: result.criticResult || {
        approved: true,
        issues: [],
        suggestions: [],
        confidence: 1.0,
      },
      usage_by_provider: {},
    };
    
    // Convert usage by provider
    for (const [provider, usage] of Object.entries(result.usageByProvider)) {
      if (usage.total_tokens > 0) {
        data.usage_by_provider[provider] = usage;
      }
    }
    
    return {
      success: true,
      data,
      usage: result.usage,
      model: `${result.generator}+${result.critic}`,
      provider: 'multi-model',
    };
  } catch (e) {
    const error = e instanceof Error ? e.message : 'Unknown error';
    return {
      success: false,
      error: `Multi-model generation error: ${error}`,
      error_type: 'api_error',
    };
  }
}

/**
 * Validate content using critic only (no generation).
 */
export async function validateContentOperation(
  payload: {
    content: string;
    critic?: ProviderName;
    critic_model?: string;
    context?: string;
    content_type?: string;
    criteria?: string;
  }
): Promise<AgentResponse<{ critic_result: GenerateWithCriticData['critic_result'] }>> {
  try {
    const { validateOnly } = await import('../orchestration');
    
    const { result, usage } = await validateOnly(payload.content, {
      criticProvider: payload.critic,
      criticModel: payload.critic_model,
      context: payload.context,
      contentType: payload.content_type,
      evaluationCriteria: payload.criteria,
    });
    
    return {
      success: true,
      data: { critic_result: result },
      usage,
      model: payload.critic_model || DEFAULT_MULTI_MODEL_CONFIG.critic,
      provider: payload.critic || 'minimax',
    };
  } catch (e) {
    const error = e instanceof Error ? e.message : 'Unknown error';
    return {
      success: false,
      error: `Validation error: ${error}`,
      error_type: 'api_error',
    };
  }
}
