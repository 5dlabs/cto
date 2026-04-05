import { afterEach, beforeEach, describe, expect, test } from 'bun:test';

import { prdResearch } from './prd-research';
import { hermesAnalyze } from './research-sources';

const ORIGINAL_ENV = {
  NOUS_API_KEY: process.env.NOUS_API_KEY,
  NOUS_BASE_URL: process.env.NOUS_BASE_URL,
  NOUS_MODEL: process.env.NOUS_MODEL,
  OPENROUTER_API_KEY: process.env.OPENROUTER_API_KEY,
  PERPLEXITY_API_KEY: process.env.PERPLEXITY_API_KEY,
  TAVILY_API_KEY: process.env.TAVILY_API_KEY,
  EXA_API_KEY: process.env.EXA_API_KEY,
  GROK_API_KEY: process.env.GROK_API_KEY,
  XAI_API_KEY: process.env.XAI_API_KEY,
  MC_API: process.env.MC_API,
  MACROCOSMOS_API_KEY: process.env.MACROCOSMOS_API_KEY,
};

const ORIGINAL_FETCH = globalThis.fetch;

function restoreEnv() {
  for (const [key, value] of Object.entries(ORIGINAL_ENV)) {
    if (value === undefined) {
      delete process.env[key];
    } else {
      process.env[key] = value;
    }
  }
}

describe('Hermes research integration', () => {
  beforeEach(() => {
    restoreEnv();
    delete process.env.OPENROUTER_API_KEY;
    delete process.env.PERPLEXITY_API_KEY;
    delete process.env.TAVILY_API_KEY;
    delete process.env.EXA_API_KEY;
    delete process.env.GROK_API_KEY;
    delete process.env.XAI_API_KEY;
    delete process.env.MC_API;
    delete process.env.MACROCOSMOS_API_KEY;
  });

  afterEach(() => {
    restoreEnv();
    globalThis.fetch = ORIGINAL_FETCH;
  });

  test('hermesAnalyze skips cleanly when no key is set', async () => {
    delete process.env.NOUS_API_KEY;
    delete process.env.NOUS_BASE_URL;
    delete process.env.NOUS_MODEL;

    const result = await hermesAnalyze('Explain deployment risks.');

    expect(result).toBe('');
  });

  test('hermesAnalyze uses Nous chat completions and returns content', async () => {
    process.env.NOUS_API_KEY = 'sk-test-key';
    process.env.NOUS_BASE_URL = 'https://example.nous.test/v1';
    process.env.NOUS_MODEL = 'nousresearch/hermes-4-70b';

    let seenRequest: { url?: string; auth?: string; model?: string } = {};

    globalThis.fetch = async (input, init) => {
      const url = typeof input === 'string' ? input : input.toString();
      const body = init?.body ? JSON.parse(String(init.body)) : {};
      seenRequest = {
        url,
        auth: init?.headers && 'Authorization' in init.headers
          ? String((init.headers as Record<string, string>).Authorization)
          : undefined,
        model: body.model,
      };

      return new Response(
        JSON.stringify({
          choices: [{ message: { content: '## Proven patterns and best practices\nUse staged rollouts.\n\n## Risks, failure modes, and cautions\nWatch migration ordering.' } }],
        }),
        { status: 200, headers: { 'Content-Type': 'application/json' } },
      );
    };

    const result = await hermesAnalyze('Explain deployment risks.');

    expect(seenRequest.url).toBe('https://example.nous.test/v1/chat/completions');
    expect(seenRequest.auth).toBe('Bearer sk-test-key');
    expect(seenRequest.model).toBe('nousresearch/hermes-4-70b');
    expect(result).toContain('Proven patterns');
  });

  test('prdResearch includes Hermes sections in both memos when Hermes responds', async () => {
    process.env.NOUS_API_KEY = 'sk-test-key';
    process.env.NOUS_BASE_URL = 'https://example.nous.test/v1';
    process.env.NOUS_MODEL = 'nousresearch/hermes-4-70b';

    globalThis.fetch = async () =>
      new Response(
        JSON.stringify({
          choices: [
            {
              message: {
                content: [
                  '## Proven patterns and best practices',
                  '- Keep infra and rollout sequencing explicit.',
                  '',
                  '## Risks, failure modes, and cautions',
                  '- Watch migration ordering and staging drift.',
                ].join('\n'),
              },
            },
          ],
        }),
        { status: 200, headers: { 'Content-Type': 'application/json' } },
      );

    const result = await prdResearch({
      prd_content: [
        '# Project',
        'Build a Rust API with PostgreSQL, Kubernetes, and a React admin dashboard.',
        'Support RBAC, audit logs, and staged rollouts.',
      ].join('\n'),
    });

    expect(result.optimist).toContain('_[Source: Hermes]_');
    expect(result.optimist).toContain('Keep infra and rollout sequencing explicit');
    expect(result.pessimist).toContain('_[Source: Hermes]_');
    expect(result.pessimist).toContain('Watch migration ordering and staging drift');
  });
});
