import { afterEach, beforeEach, describe, expect, test } from 'bun:test';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

import { designIntake } from './design-intake';

const ORIGINAL_ENV = {
  STITCH_API_KEY: process.env.STITCH_API_KEY,
  STITCH_PROJECT_ID: process.env.STITCH_PROJECT_ID,
  FRAMER_API_KEY: process.env.FRAMER_API_KEY,
  FRAMER_PROJECT_URL: process.env.FRAMER_PROJECT_URL,
  FRAMER_PROJECT_ID: process.env.FRAMER_PROJECT_ID,
  FRAMER_PREVIEW_URL: process.env.FRAMER_PREVIEW_URL,
};

function restoreEnv() {
  for (const [key, value] of Object.entries(ORIGINAL_ENV)) {
    if (value === undefined) {
      delete process.env[key];
    } else {
      process.env[key] = value;
    }
  }
}

describe('design intake provider modes', () => {
  let workDir = '';

  beforeEach(async () => {
    restoreEnv();
    delete process.env.STITCH_API_KEY;
    delete process.env.STITCH_PROJECT_ID;
    delete process.env.FRAMER_API_KEY;
    delete process.env.FRAMER_PROJECT_URL;
    delete process.env.FRAMER_PROJECT_ID;
    delete process.env.FRAMER_PREVIEW_URL;
    workDir = await mkdtemp(join(tmpdir(), 'design-intake-test-'));
  });

  afterEach(async () => {
    restoreEnv();
    if (workDir) {
      await rm(workDir, { recursive: true, force: true });
    }
  });

  test('defaults to stitch-safe auto routing with structured artifacts', async () => {
    const outputDir = join(workDir, 'out');
    const result = await designIntake({
      prd_content: 'Build a web dashboard with responsive UI.',
      design_prompt: 'Modern, professional UI system.',
      design_mode: 'ingest_plus_stitch',
      output_dir: outputDir,
      project_name: 'alpha',
    });

    expect(result.providerMode).toBe('stitch');
    expect(result.providers.stitch.status).toBe('skipped');
    expect(result.providers.stitch.reason).toBe('missing_stitch_api_key');
    expect(result.providers.framer.status).toBe('skipped');

    const componentLibraryPath = join(outputDir, 'component-library.json');
    const componentLibraryRaw = await readFile(componentLibraryPath, 'utf-8');
    const componentLibrary = JSON.parse(componentLibraryRaw) as { provider_mode?: string; tokens?: Record<string, unknown> };
    expect(componentLibrary.provider_mode).toBe('stitch');
    expect(componentLibrary.tokens).toBeDefined();
  });

  test('framer mode emits framer candidates and normalized output', async () => {
    process.env.FRAMER_API_KEY = 'framer-test-key';
    process.env.FRAMER_PROJECT_URL = 'https://framer.com/projects/test';
    process.env.FRAMER_PREVIEW_URL = 'https://example.framer.website';

    const outputDir = join(workDir, 'framer');
    const result = await designIntake({
      prd_content: 'Ship a React frontend with web and mobile targets.',
      design_prompt: 'Create reusable components with clear controls.',
      design_provider: 'framer',
      output_dir: outputDir,
      project_name: 'beta',
    });

    expect(result.providerMode).toBe('framer');
    expect(result.providers.framer.status).toBe('generated');
    expect(result.providers.framer.candidates.length).toBeGreaterThan(0);
    expect(result.normalized_candidates.some((candidate) => candidate.provider === 'framer')).toBeTrue();
  });

  test('both mode continues when stitch credentials are missing', async () => {
    process.env.FRAMER_API_KEY = 'framer-test-key';
    process.env.FRAMER_PROJECT_URL = 'https://framer.com/projects/test';

    const outputDir = join(workDir, 'both');
    const result = await designIntake({
      prd_content: 'Build a frontend for web and desktop use.',
      design_prompt: 'Focus on component-library readiness.',
      design_provider: 'both',
      output_dir: outputDir,
      project_name: 'gamma',
    });

    expect(result.providerMode).toBe('both');
    expect(result.providers.stitch.status).toBe('skipped');
    expect(result.providers.framer.status).toBe('generated');
    expect(result.normalized_candidates.length).toBeGreaterThan(0);
  });

  test('framer project id payload resolves to project URL', async () => {
    process.env.FRAMER_API_KEY = 'framer-test-key';
    process.env.FRAMER_PREVIEW_URL = 'https://example.framer.website';

    const outputDir = join(workDir, 'framer-id');
    const result = await designIntake({
      prd_content: 'Build a frontend for the dashboard.',
      design_prompt: 'Prioritize component-level reuse.',
      design_provider: 'framer',
      design_framer_project: 'fr_4j04q95j7s8n48kakevderq43c',
      output_dir: outputDir,
      project_name: 'delta',
    });

    expect(result.providers.framer.status).toBe('generated');
    const first = result.providers.framer.candidates[0];
    expect(first?.meta?.project_url).toBe('https://framer.com/projects/fr_4j04q95j7s8n48kakevderq43c');
    expect(first?.meta?.project_ref).toBe('fr_4j04q95j7s8n48kakevderq43c');
  });
});
