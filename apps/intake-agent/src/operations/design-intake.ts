import { mkdir, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { StitchDirectClient } from '../stitch-client';

export interface DesignIntakePayload {
  prd_content: string;
  design_prompt?: string;
  design_artifacts_path?: string;
  design_urls?: string;
  design_mode?: 'ingest_only' | 'ingest_plus_stitch';
  output_dir?: string;
  project_name?: string;
}

type FrontendTarget = 'web' | 'mobile' | 'desktop';

interface FrontendDetectionResult {
  hasFrontend: boolean;
  frontendTargets: FrontendTarget[];
  confidence: number;
  evidence: string[];
}

interface CrawledUrl {
  url: string;
  status: 'ok' | 'error';
  title?: string;
  snippet?: string;
  error?: string;
}

interface StitchCandidate {
  target: FrontendTarget;
  deviceType: 'DESKTOP' | 'MOBILE' | 'TABLET' | 'AGNOSTIC';
  screenId?: string;
  htmlUrl?: string;
  imageUrl?: string;
  rationale: string;
  prompt: string;
  status: 'generated' | 'failed';
  error?: string;
}

interface StitchSummary {
  status: 'generated' | 'skipped' | 'failed';
  reason?: string;
  projectId?: string;
  warnings: string[];
  candidates: StitchCandidate[];
}

interface ScreenRef {
  name: string;
  screenId: string;
}

function collectScreenRefs(value: unknown, out: ScreenRef[]): void {
  if (!value) {
    return;
  }
  if (Array.isArray(value)) {
    for (const item of value) {
      collectScreenRefs(item, out);
    }
    return;
  }
  if (typeof value !== 'object') {
    return;
  }

  const obj = value as Record<string, unknown>;
  const nameValue = obj['name'];
  if (typeof nameValue === 'string' && nameValue.includes('/screens/')) {
    const parts = nameValue.split('/screens/');
    const screenId = parts[1];
    if (screenId) {
      out.push({ name: nameValue, screenId });
    }
  }
  for (const nestedValue of Object.values(obj)) {
    collectScreenRefs(nestedValue, out);
  }
}

function readDownloadUrl(result: Record<string, unknown>, key: string): string | undefined {
  const camel = result[key] as Record<string, unknown> | undefined;
  const snake = result[key.replace(/[A-Z]/g, (m) => `_${m.toLowerCase()}`)] as Record<string, unknown> | undefined;
  const camelUrl = camel?.['downloadUrl'];
  if (typeof camelUrl === 'string' && camelUrl.length > 0) {
    return camelUrl;
  }
  const snakeUrl = snake?.['download_url'];
  if (typeof snakeUrl === 'string' && snakeUrl.length > 0) {
    return snakeUrl;
  }
  return undefined;
}

export interface DesignContextResult {
  projectName: string;
  mode: 'ingest_only' | 'ingest_plus_stitch';
  prompt: string;
  artifactPath: string;
  urls: string[];
  crawledUrls: CrawledUrl[];
  hasFrontend: boolean;
  frontendTargets: FrontendTarget[];
  confidence: number;
  evidence: string[];
  stitch: StitchSummary;
}

function parseDesignUrls(raw?: string): string[] {
  if (!raw || raw.trim() === '') {
    return [];
  }

  const trimmed = raw.trim();
  if (trimmed.startsWith('[')) {
    try {
      const parsed = JSON.parse(trimmed);
      if (Array.isArray(parsed)) {
        return parsed
          .map((entry) => String(entry).trim())
          .filter((entry) => entry.startsWith('http://') || entry.startsWith('https://'));
      }
    } catch {
      // Fall through to CSV parsing.
    }
  }

  return trimmed
    .split(/[,\n]/g)
    .map((entry) => entry.trim())
    .filter((entry) => entry.startsWith('http://') || entry.startsWith('https://'));
}

function detectFrontendSignals(input: string): FrontendDetectionResult {
  const lines = input.split('\n');
  const evidence: string[] = [];

  const targetPatterns: Record<FrontendTarget, RegExp[]> = {
    web: [
      /\b(web|website|landing page|frontend|ui|ux|next\.js|react|svelte|vue)\b/i,
      /\b(browser|responsive|html|css)\b/i,
    ],
    mobile: [
      /\b(mobile|ios|android|react native|expo)\b/i,
      /\b(app store|play store)\b/i,
    ],
    desktop: [
      /\b(desktop|electron|tauri|mac app|windows app)\b/i,
      /\b(system tray|native window)\b/i,
    ],
  };

  const matchedTargets: FrontendTarget[] = [];
  for (const target of Object.keys(targetPatterns) as FrontendTarget[]) {
    const patterns = targetPatterns[target];
    const hasMatch = patterns.some((pattern) => pattern.test(input));
    if (hasMatch) {
      matchedTargets.push(target);
      const firstMatchLine = lines.find((line) => patterns.some((pattern) => pattern.test(line)));
      if (firstMatchLine) {
        evidence.push(`[${target}] ${firstMatchLine.slice(0, 180)}`);
      }
    }
  }

  const backendOnlySignals = /\b(api only|backend only|service only|no frontend|headless)\b/i.test(input);
  const hasFrontend = matchedTargets.length > 0 || !backendOnlySignals;

  const confidenceBase = matchedTargets.length > 0 ? 0.7 : 0.45;
  const confidence = Math.min(0.95, confidenceBase + matchedTargets.length * 0.1);

  const frontendTargets: FrontendTarget[] = matchedTargets.length > 0 ? matchedTargets : (hasFrontend ? ['web'] : []);
  return {
    hasFrontend,
    frontendTargets,
    confidence,
    evidence: evidence.slice(0, 6),
  };
}

async function crawlUrls(urls: string[]): Promise<CrawledUrl[]> {
  const results: CrawledUrl[] = [];

  for (const url of urls) {
    try {
      const response = await fetch(url);
      if (!response.ok) {
        results.push({
          url,
          status: 'error',
          error: `HTTP ${response.status}`,
        });
        continue;
      }

      const html = await response.text();
      const titleMatch = html.match(/<title>([^<]+)<\/title>/i);
      const plainText = html.replace(/<script[\s\S]*?<\/script>/gi, ' ').replace(/<style[\s\S]*?<\/style>/gi, ' ').replace(/<[^>]+>/g, ' ').replace(/\s+/g, ' ').trim();
      results.push({
        url,
        status: 'ok',
        title: titleMatch?.[1]?.trim(),
        snippet: plainText.slice(0, 320),
      });
    } catch (error) {
      results.push({
        url,
        status: 'error',
        error: error instanceof Error ? error.message : String(error),
      });
    }
  }

  return results;
}

function deviceForTarget(target: FrontendTarget): 'DESKTOP' | 'MOBILE' | 'TABLET' | 'AGNOSTIC' {
  switch (target) {
    case 'mobile':
      return 'MOBILE';
    case 'desktop':
      return 'DESKTOP';
    case 'web':
      return 'DESKTOP';
    default:
      return 'DESKTOP';
  }
}

async function generateStitchCandidates(
  context: DesignContextResult,
  outputDir: string,
): Promise<StitchSummary> {
  const warnings: string[] = [];

  if (!context.hasFrontend) {
    return {
      status: 'skipped',
      reason: 'frontend_not_detected',
      warnings,
      candidates: [],
    };
  }

  const stitchApiKey = process.env['STITCH_API_KEY']?.trim();
  if (!stitchApiKey) {
    warnings.push('STITCH_API_KEY is missing; skipping Stitch generation and continuing in ingest-only mode.');
    return {
      status: 'skipped',
      reason: 'missing_stitch_api_key',
      warnings,
      candidates: [],
    };
  }

  const client = new StitchDirectClient(stitchApiKey);

  let projectId = process.env['STITCH_PROJECT_ID']?.trim();
  try {
    if (!projectId) {
      const createResult = await client.callTool('create_project', {
        title: `${context.projectName} Design Intake`,
      });
      const createAny = createResult as Record<string, unknown>;
      const nested = (createAny['result'] ?? {}) as Record<string, unknown>;
      const namePath =
        (createAny['name'] as string | undefined) ||
        (nested['name'] as string | undefined) ||
        '';
      const projectIdFromName =
        namePath.startsWith('projects/') ? namePath.replace(/^projects\//, '') : undefined;
      projectId =
        (createAny['projectId'] as string | undefined) ||
        (createAny['id'] as string | undefined) ||
        (nested['projectId'] as string | undefined) ||
        (nested['id'] as string | undefined) ||
        projectIdFromName;
    }
  } catch (error) {
    warnings.push(`Failed to create Stitch project: ${error instanceof Error ? error.message : String(error)}`);
    await client.close();
    return {
      status: 'failed',
      reason: 'create_project_failed',
      warnings,
      candidates: [],
    };
  }

  if (!projectId) {
    warnings.push('Unable to resolve Stitch project ID from create_project result.');
    await client.close();
    return {
      status: 'failed',
      reason: 'missing_project_id',
      warnings,
      candidates: [],
    };
  }

  const candidates: StitchCandidate[] = [];
  for (const target of context.frontendTargets) {
    const deviceType = deviceForTarget(target);
    const promptParts = [
      `Improve and modernize this ${target} interface for project ${context.projectName}.`,
      context.prompt ? `Design intent: ${context.prompt}` : '',
      context.urls.length > 0 ? `Reference URLs: ${context.urls.join(', ')}` : '',
      'Return production-ready UI structure with clear hierarchy, accessibility, and conversion-focused layout.',
    ].filter(Boolean);
    const prompt = promptParts.join(' ');

    try {
      console.error(`[design-intake] generating Stitch candidate for ${target} (${deviceType})`);
      const generateResult = await client.callTool('generate_screen_from_text', {
        projectId,
        prompt,
        deviceType,
      });
      const generateAny = generateResult as Record<string, unknown>;
      const components =
        (generateAny['outputComponents'] as unknown[] | undefined) ||
        (generateAny['output_components'] as unknown[] | undefined) ||
        [];
      const screenRefs: ScreenRef[] = [];
      collectScreenRefs(components, screenRefs);
      const primaryScreen = screenRefs[0];

      if (!primaryScreen) {
        candidates.push({
          target,
          deviceType,
          rationale: `Stitch returned no screen refs for ${target}.`,
          prompt,
          status: 'failed',
          error: 'No screen returned from generate_screen_from_text',
        });
        continue;
      }

      const getScreenResult = await client.callTool('get_screen', {
        name: primaryScreen.name,
        projectId,
        screenId: primaryScreen.screenId,
      });
      const screenAny = getScreenResult as Record<string, unknown>;
      const htmlUrl = readDownloadUrl(screenAny, 'htmlCode');
      const imageUrl = readDownloadUrl(screenAny, 'screenshot');

      candidates.push({
        target,
        deviceType,
        screenId: primaryScreen.screenId,
        htmlUrl,
        imageUrl,
        rationale: `Generated ${target} modernization candidate from intake design context.`,
        prompt,
        status: 'generated',
      });
      console.error(`[design-intake] Stitch candidate ready for ${target} (${deviceType})`);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      console.error(`[design-intake] Stitch candidate failed for ${target} (${deviceType}): ${message}`);
      candidates.push({
        target,
        deviceType,
        rationale: `Failed to generate ${target} candidate.`,
        prompt,
        status: 'failed',
        error: message,
      });
    }
  }

  await client.close();

  const stitchDir = join(outputDir, 'stitch');
  await mkdir(stitchDir, { recursive: true });
  await writeFile(
    join(stitchDir, 'stitch-run.json'),
    JSON.stringify(
      {
        timestamp: new Date().toISOString(),
        projectId,
        mode: context.mode,
        targets: context.frontendTargets,
      },
      null,
      2,
    ),
  );
  await writeFile(join(stitchDir, 'candidates.json'), JSON.stringify(candidates, null, 2));

  const generatedCount = candidates.filter((candidate) => candidate.status === 'generated').length;
  return {
    status: generatedCount > 0 ? 'generated' : 'failed',
    projectId,
    warnings,
    candidates,
  };
}

// =============================================================================
// Phase B: Generate Design Variants
// =============================================================================

export interface DesignVariantPayload {
  candidates_path: string;
  output_dir?: string;
  variant_count?: number;
  creative_range?: 'REFINE' | 'EXPLORE' | 'REIMAGINE';
}

export interface DesignVariant {
  variant_id: string;
  source_screen_id: string;
  source_target: FrontendTarget;
  label: string;
  description: string;
  aspects_changed: string[];
  screen_id?: string;
  html_url?: string;
  image_url?: string;
  status: 'generated' | 'failed';
  error?: string;
}

export interface DesignVariantsResult {
  project_id: string;
  screens: Array<{
    source_screen_id: string;
    source_target: FrontendTarget;
    variants: DesignVariant[];
  }>;
  total_variants: number;
  timestamp: string;
}

const FIFE_HIRES_SUFFIX = '=w1920';

function hiresImageUrl(url: string | undefined): string | undefined {
  if (!url) return undefined;
  if (url.includes('=w') || url.includes('=s')) return url;
  return `${url}${FIFE_HIRES_SUFFIX}`;
}

const VARIANT_LABELS = ['Layout Focus', 'Color Scheme', 'Typography & Spacing'];
const VARIANT_ASPECTS: string[][] = [
  ['LAYOUT'],
  ['COLOR_SCHEME'],
  ['TEXT_FONT', 'TEXT_CONTENT'],
];

export async function generateDesignVariants(payload: DesignVariantPayload): Promise<DesignVariantsResult> {
  const { readFile } = await import('node:fs/promises');

  const candidatesRaw = await readFile(payload.candidates_path, 'utf-8');
  const candidates: StitchCandidate[] = JSON.parse(candidatesRaw);

  const stitchRunRaw = await readFile(
    payload.candidates_path.replace('candidates.json', 'stitch-run.json'),
    'utf-8',
  );
  const stitchRun: { projectId: string } = JSON.parse(stitchRunRaw);
  const projectId = stitchRun.projectId;

  const generatedCandidates = candidates.filter(c => c.status === 'generated' && c.screenId);
  if (generatedCandidates.length === 0) {
    return {
      project_id: projectId,
      screens: [],
      total_variants: 0,
      timestamp: new Date().toISOString(),
    };
  }

  const stitchApiKey = process.env['STITCH_API_KEY']?.trim();
  if (!stitchApiKey) {
    throw new Error('STITCH_API_KEY is required for design variant generation');
  }

  const client = new StitchDirectClient(stitchApiKey);
  const variantCount = payload.variant_count ?? 3;
  const creativeRange = payload.creative_range ?? 'EXPLORE';
  const outputDir = payload.output_dir ?? '.intake/design/stitch';

  const screens: DesignVariantsResult['screens'] = [];

  for (const candidate of generatedCandidates) {
    const screenVariants: DesignVariant[] = [];

    try {
      const generateResult = await client.callTool('generate_variants', {
        projectId,
        selectedScreenIds: [candidate.screenId],
        prompt: `Generate ${variantCount} design variants exploring different visual directions for this ${candidate.target} interface.`,
        variantOptions: {
          variantCount,
          creativeRange,
          aspects: [],
        },
        modelId: 'GEMINI_3_1_PRO',
        deviceType: candidate.deviceType,
      });

      const resultAny = generateResult as Record<string, unknown>;
      const components =
        (resultAny['outputComponents'] as unknown[] | undefined) ||
        (resultAny['output_components'] as unknown[] | undefined) ||
        [];
      const screenRefs: ScreenRef[] = [];
      collectScreenRefs(components, screenRefs);

      for (let i = 0; i < screenRefs.length && i < variantCount; i++) {
        const ref = screenRefs[i];
        const label = VARIANT_LABELS[i] ?? `Variant ${i + 1}`;
        const aspects = VARIANT_ASPECTS[i] ?? [];

        try {
          const getScreenResult = await client.callTool('get_screen', {
            name: ref.name,
            projectId,
            screenId: ref.screenId,
          });
          const screenAny = getScreenResult as Record<string, unknown>;
          const htmlUrl = readDownloadUrl(screenAny, 'htmlCode');
          const rawImageUrl = readDownloadUrl(screenAny, 'screenshot');

          screenVariants.push({
            variant_id: `${candidate.screenId}-v${i + 1}`,
            source_screen_id: candidate.screenId!,
            source_target: candidate.target,
            label,
            description: `${candidate.target} variant emphasizing ${label.toLowerCase()}.`,
            aspects_changed: aspects,
            screen_id: ref.screenId,
            html_url: htmlUrl,
            image_url: hiresImageUrl(rawImageUrl),
            status: 'generated',
          });
        } catch (err) {
          screenVariants.push({
            variant_id: `${candidate.screenId}-v${i + 1}`,
            source_screen_id: candidate.screenId!,
            source_target: candidate.target,
            label,
            description: `Failed to retrieve variant screen.`,
            aspects_changed: aspects,
            status: 'failed',
            error: err instanceof Error ? err.message : String(err),
          });
        }
      }

      if (screenRefs.length === 0) {
        screenVariants.push({
          variant_id: `${candidate.screenId}-v1`,
          source_screen_id: candidate.screenId!,
          source_target: candidate.target,
          label: 'Fallback',
          description: 'No variants returned from Stitch; using original.',
          aspects_changed: [],
          screen_id: candidate.screenId,
          html_url: candidate.htmlUrl,
          image_url: hiresImageUrl(candidate.imageUrl),
          status: 'generated',
        });
      }
    } catch (err) {
      screenVariants.push({
        variant_id: `${candidate.screenId}-v1`,
        source_screen_id: candidate.screenId!,
        source_target: candidate.target,
        label: 'Original (fallback)',
        description: `Variant generation failed; using original.`,
        aspects_changed: [],
        screen_id: candidate.screenId,
        html_url: candidate.htmlUrl,
        image_url: hiresImageUrl(candidate.imageUrl),
        status: 'failed',
        error: err instanceof Error ? err.message : String(err),
      });
    }

    screens.push({
      source_screen_id: candidate.screenId!,
      source_target: candidate.target,
      variants: screenVariants,
    });
  }

  await client.close();

  const result: DesignVariantsResult = {
    project_id: projectId,
    screens,
    total_variants: screens.reduce((sum, s) => sum + s.variants.filter(v => v.status === 'generated').length, 0),
    timestamp: new Date().toISOString(),
  };

  await mkdir(outputDir, { recursive: true });
  await writeFile(join(outputDir, 'design-variants.json'), JSON.stringify(result, null, 2));

  return result;
}

export async function designIntake(payload: DesignIntakePayload): Promise<DesignContextResult> {
  const outputDir = payload.output_dir?.trim() || '.intake/design';
  const projectName = payload.project_name?.trim() || 'project';
  const mode = payload.design_mode === 'ingest_only' ? 'ingest_only' : 'ingest_plus_stitch';
  const prompt = payload.design_prompt?.trim() ?? '';
  const artifactPath = payload.design_artifacts_path?.trim() ?? '';
  const urls = parseDesignUrls(payload.design_urls);

  await mkdir(outputDir, { recursive: true });
  await mkdir(join(outputDir, 'crawled'), { recursive: true });

  const combinedInput = [payload.prd_content || '', prompt, urls.join('\n')].join('\n');
  const detection = detectFrontendSignals(combinedInput);
  const crawledUrls = await crawlUrls(urls);
  await writeFile(join(outputDir, 'crawled', 'urls.json'), JSON.stringify(crawledUrls, null, 2));

  const context: DesignContextResult = {
    projectName,
    mode,
    prompt,
    artifactPath,
    urls,
    crawledUrls,
    hasFrontend: detection.hasFrontend,
    frontendTargets: detection.frontendTargets,
    confidence: detection.confidence,
    evidence: detection.evidence,
    stitch: {
      status: 'skipped',
      reason: 'not_requested',
      warnings: [],
      candidates: [],
    },
  };

  if (mode === 'ingest_plus_stitch') {
    context.stitch = await generateStitchCandidates(context, outputDir);
  }

  await writeFile(join(outputDir, 'design-context.json'), JSON.stringify(context, null, 2));
  return context;
}
