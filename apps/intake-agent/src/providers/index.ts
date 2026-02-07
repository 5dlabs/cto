/**
 * Provider Registry - Manages multi-model providers.
 * 
 * Provides a unified interface for accessing Claude, Minimax, and OpenAI/Codex providers.
 * Supports both Claude Code SDK (OAuth) and Anthropic API (API key).
 */

import type { ModelProvider, ProviderName, ProviderRegistry } from './types';
import { createClaudeProvider } from './claude';
import { minimaxProvider } from './minimax';
import { codexProvider } from './codex';

// Re-export types and providers
export * from './types';
export { minimaxProvider } from './minimax';
export { codexProvider } from './codex';

/**
 * Lazy-initialized Claude provider to avoid side-effect logging at module load time.
 * This is important for CLI flags like --help and --version to work without noise.
 */
let _claudeProvider: ModelProvider | null = null;
function getClaudeProvider(): ModelProvider {
  if (_claudeProvider === null) {
    _claudeProvider = createClaudeProvider();
  }
  return _claudeProvider;
}

/**
 * Map of all registered providers (initialized lazily for claude).
 */
const providers = new Map<ProviderName, () => ModelProvider>([
  ['claude', getClaudeProvider],
  ['minimax', () => minimaxProvider],
  ['codex', () => codexProvider],
]);

/**
 * Provider registry implementation.
 */
class ProviderRegistryImpl implements ProviderRegistry {
  /**
   * Get a provider by name.
   */
  get(name: ProviderName): ModelProvider | undefined {
    const factory = providers.get(name);
    return factory ? factory() : undefined;
  }
  
  /**
   * Check if a provider is available (configured and ready).
   */
  isAvailable(name: ProviderName): boolean {
    const factory = providers.get(name);
    return factory ? factory().isAvailable() : false;
  }
  
  /**
   * List all available providers.
   */
  listAvailable(): ProviderName[] {
    const available: ProviderName[] = [];
    for (const [name, factory] of providers) {
      if (factory().isAvailable()) {
        available.push(name);
      }
    }
    return available;
  }
  
  /**
   * Get all registered providers.
   */
  all(): Map<ProviderName, ModelProvider> {
    const result = new Map<ProviderName, ModelProvider>();
    for (const [name, factory] of providers) {
      result.set(name, factory());
    }
    return result;
  }
}

/**
 * Singleton instance of the provider registry.
 */
export const providerRegistry: ProviderRegistry = new ProviderRegistryImpl();

/**
 * Get a provider by name, throwing if not found.
 */
export function getProvider(name: ProviderName): ModelProvider {
  const provider = providerRegistry.get(name);
  if (!provider) {
    throw new Error(`Unknown provider: ${name}`);
  }
  return provider;
}

/**
 * Get a provider by name, throwing if not available.
 */
export function getAvailableProvider(name: ProviderName): ModelProvider {
  const provider = getProvider(name);
  if (!provider.isAvailable()) {
    throw new Error(`Provider ${name} is not available. Check API key configuration.`);
  }
  return provider;
}

/**
 * Check provider availability and return status.
 */
export function checkProviderStatus(): Record<ProviderName, { available: boolean; model: string }> {
  const status: Record<ProviderName, { available: boolean; model: string }> = {} as Record<ProviderName, { available: boolean; model: string }>;
  
  for (const [name, factory] of providers) {
    const provider = factory();
    status[name] = {
      available: provider.isAvailable(),
      model: provider.defaultModel,
    };
  }
  
  return status;
}
