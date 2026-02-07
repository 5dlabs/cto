/**
 * Grok X Search - Configuration
 *
 * Handles API key retrieval (env var or 1Password) and default settings.
 */

import { execSync } from 'child_process';
import type { GrokClientConfig } from './types.js';

export const GROK_API_URL = 'https://api.x.ai/v1';
export const GROK_MODEL = 'grok-4-1-fast-reasoning';

/**
 * Resolve the Grok API key from environment or 1Password.
 *
 * Priority:
 *  1. Explicit `apiKey` in config
 *  2. `GROK_API_KEY` env var
 *  3. 1Password vault lookup
 */
export function resolveApiKey(config?: GrokClientConfig): string {
  if (config?.apiKey) return config.apiKey;

  const envKey = process.env.GROK_API_KEY ?? process.env.XAI_API_KEY;
  if (envKey) return envKey;

  return getApiKeyFrom1Password();
}

/**
 * Retrieve API key from 1Password CLI.
 */
function getApiKeyFrom1Password(): string {
  try {
    const result = execSync(
      'op item get "Grok X API Key" --vault Automation --fields xai_api_key --reveal',
      { encoding: 'utf-8', timeout: 10_000 },
    );
    return result.trim();
  } catch {
    throw new Error(
      'GROK_API_KEY not set. Set GROK_API_KEY env var, pass apiKey in config, or install 1Password CLI.',
    );
  }
}

/**
 * Build a full client config with defaults filled in.
 */
export function resolveConfig(config?: GrokClientConfig): Required<GrokClientConfig> {
  return {
    apiKey: resolveApiKey(config),
    model: config?.model ?? process.env.GROK_MODEL ?? GROK_MODEL,
    apiUrl: config?.apiUrl ?? process.env.GROK_API_URL ?? GROK_API_URL,
  };
}
