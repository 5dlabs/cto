import { mkdir, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { StitchDirectClient } from '../stitch-client';

// Static import so Bun's bundler includes framer-api in the compiled binary.
// The try/catch at call sites handles the case where it's unavailable.
let framerApiModule: { connect?: (url: string, key: string) => Promise<any> } | null = null;
try {
  framerApiModule = await import('framer-api');
} catch {
  // framer-api not available — offline mode
}

export type DesignProvider = 'stitch' | 'framer';
export type DesignProviderMode = 'stitch' | 'framer' | 'both' | 'auto';
export type LegacyDesignMode = 'ingest_only' | 'ingest_plus_stitch';
type FrontendTarget = 'web' | 'mobile' | 'desktop';
type DeviceType = 'DESKTOP' | 'MOBILE' | 'TABLET' | 'AGNOSTIC';

export interface DesignIntakePayload {
  prd_content: string;
  design_prompt?: string;
  design_artifacts_path?: string;
  design_urls?: string;
  design_mode?: LegacyDesignMode | DesignProviderMode;
  design_provider?: DesignProviderMode;
  design_framer_project?: string;
  output_dir?: string;
  project_name?: string;
}

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

export interface ProviderCandidate {
  provider: DesignProvider;
  candidate_id?: string;
  target: FrontendTarget;
  deviceType: DeviceType;
  screenId?: string;
  htmlUrl?: string;
  imageUrl?: string;
  rationale: string;
  prompt: string;
  status: 'generated' | 'failed';
  error?: string;
  meta?: Record<string, unknown>;
}

export interface ProviderSummary {
  status: 'generated' | 'skipped' | 'failed';
  reason?: string;
  projectId?: string;
  warnings: string[];
  candidates: ProviderCandidate[];
}

interface ScreenRef {
  name: string;
  screenId: string;
}

interface ComponentLibraryArtifact {
  version: string;
  projectName: string;
  provider_mode: DesignProviderMode;
  providers: DesignProvider[];
  tokens: {
    color: Array<{ name: string; value: string; description?: string }>;
    typography: Array<{ name: string; value: string; description?: string }>;
    spacing: Array<{ name: string; value: string; description?: string }>;
    radius: Array<{ name: string; value: string; description?: string }>;
  };
  primitives: Array<{ name: string; description: string; states: string[]; provider_refs?: string[] }>;
  patterns: Array<{ name: string; description: string; states: string[]; provider_refs?: string[] }>;
  component_map: Array<{
    target: FrontendTarget;
    provider: DesignProvider;
    candidate_id: string;
    screen_id?: string;
    notes?: string;
  }>;
  framer_code_components: Array<{
    name: string;
    props: Array<{ name: string; type: string; default?: string | number | boolean }>;
    property_controls: string[];
    notes?: string;
  }>;
  generated_at: string;
}

export interface DesignContextResult {
  projectName: string;
  mode: LegacyDesignMode | DesignProviderMode;
  providerMode: DesignProviderMode;
  prompt: string;
  artifactPath: string;
  urls: string[];
  crawledUrls: CrawledUrl[];
  hasFrontend: boolean;
  frontendTargets: FrontendTarget[];
  confidence: number;
  evidence: string[];
  providers: Record<DesignProvider, ProviderSummary>;
  // Backward-compatible aliases
  stitch: ProviderSummary;
  framer: ProviderSummary;
  normalized_candidates: ProviderCandidate[];
  component_library?: {
    path: string;
    format: 'json';
    version: string;
  };
  design_system?: {
    path: string;
    format: 'markdown';
  };
}

interface ProviderExecutionContext {
  projectName: string;
  mode: LegacyDesignMode | DesignProviderMode;
  providerMode: DesignProviderMode;
  framerProjectRef: string;
  prompt: string;
  artifactPath: string;
  urls: string[];
  hasFrontend: boolean;
  frontendTargets: FrontendTarget[];
}

interface DesignProviderHandler {
  name: DesignProvider;
  generate(context: ProviderExecutionContext, outputDir: string): Promise<ProviderSummary>;
}

async function dynamicImport(moduleName: string): Promise<unknown> {
  const importer = new Function('moduleName', 'return import(moduleName);') as (moduleName: string) => Promise<unknown>;
  return importer(moduleName);
}

function collectScreenRefs(value: unknown, out: ScreenRef[]): void {
  if (!value) return;
  if (Array.isArray(value)) {
    for (const item of value) {
      collectScreenRefs(item, out);
    }
    return;
  }
  if (typeof value !== 'object') return;

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
  return { hasFrontend, frontendTargets, confidence, evidence: evidence.slice(0, 6) };
}

async function crawlUrls(urls: string[]): Promise<CrawledUrl[]> {
  const results: CrawledUrl[] = [];
  for (const url of urls) {
    try {
      const response = await fetch(url);
      if (!response.ok) {
        results.push({ url, status: 'error', error: `HTTP ${response.status}` });
        continue;
      }
      const html = await response.text();
      const titleMatch = html.match(/<title>([^<]+)<\/title>/i);
      const plainText = html
        .replace(/<script[\s\S]*?<\/script>/gi, ' ')
        .replace(/<style[\s\S]*?<\/style>/gi, ' ')
        .replace(/<[^>]+>/g, ' ')
        .replace(/\s+/g, ' ')
        .trim();
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

function deviceForTarget(target: FrontendTarget): DeviceType {
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

function deriveProviderMode(payload: DesignIntakePayload): DesignProviderMode {
  const explicit = payload.design_provider?.trim() as DesignProviderMode | undefined;
  if (explicit && ['stitch', 'framer', 'both', 'auto'].includes(explicit)) {
    return explicit;
  }
  const mode = payload.design_mode?.trim() as string | undefined;
  switch (mode) {
    case 'stitch':
    case 'framer':
    case 'both':
    case 'auto':
      return mode;
    case 'ingest_plus_stitch':
      return 'stitch';
    default:
      return 'auto';
  }
}

function deriveLegacyMode(payload: DesignIntakePayload): LegacyDesignMode {
  if (payload.design_mode === 'ingest_only') return 'ingest_only';
  return 'ingest_plus_stitch';
}

function resolveFramerProjectUrl(projectRefRaw: string): string | undefined {
  const projectRef = projectRefRaw.trim();
  if (!projectRef) {
    return undefined;
  }
  if (projectRef.startsWith('http://') || projectRef.startsWith('https://')) {
    return projectRef;
  }
  // Support passing raw Framer project IDs (e.g. fr_xxx) by expanding to a project URL.
  if (/^[a-zA-Z0-9_-]+$/.test(projectRef)) {
    return `https://framer.com/projects/${projectRef}`;
  }
  return undefined;
}

function requestedProviders(providerMode: DesignProviderMode, legacyMode: LegacyDesignMode): DesignProvider[] {
  if (legacyMode === 'ingest_only') return [];
  switch (providerMode) {
    case 'stitch':
      return ['stitch'];
    case 'framer':
      return ['framer'];
    case 'both':
      return ['stitch', 'framer'];
    case 'auto':
      // Safe default in rollout: keep Stitch as baseline, enable Framer explicitly.
      return ['stitch'];
    default:
      return ['stitch'];
  }
}

function withCandidateIds(candidates: ProviderCandidate[]): ProviderCandidate[] {
  return candidates.map((candidate, index) => ({
    ...candidate,
    candidate_id: candidate.candidate_id || `${candidate.provider}-${candidate.target}-${index + 1}`,
  }));
}

function createEmptySummary(reason: string): ProviderSummary {
  return {
    status: 'skipped',
    reason,
    warnings: [],
    candidates: [],
  };
}

async function generateStitchCandidates(
  context: ProviderExecutionContext,
  outputDir: string,
): Promise<ProviderSummary> {
  const warnings: string[] = [];
  if (!context.hasFrontend) {
    return createEmptySummary('frontend_not_detected');
  }
  const stitchApiKey = process.env['STITCH_API_KEY']?.trim();
  if (!stitchApiKey) {
    warnings.push('STITCH_API_KEY is missing; skipping Stitch generation.');
    return { status: 'skipped', reason: 'missing_stitch_api_key', warnings, candidates: [] };
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
      const namePath = (createAny['name'] as string | undefined) || (nested['name'] as string | undefined) || '';
      const projectIdFromName = namePath.startsWith('projects/') ? namePath.replace(/^projects\//, '') : undefined;
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
    return { status: 'failed', reason: 'create_project_failed', warnings, candidates: [] };
  }

  if (!projectId) {
    warnings.push('Unable to resolve Stitch project ID from create_project result.');
    await client.close();
    return { status: 'failed', reason: 'missing_project_id', warnings, candidates: [] };
  }

  const candidates: ProviderCandidate[] = [];
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
          provider: 'stitch',
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
        provider: 'stitch',
        target,
        deviceType,
        screenId: primaryScreen.screenId,
        htmlUrl,
        imageUrl,
        rationale: `Generated ${target} modernization candidate from intake design context.`,
        prompt,
        status: 'generated',
      });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      candidates.push({
        provider: 'stitch',
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
        provider: 'stitch',
        projectId,
        mode: context.mode,
        providerMode: context.providerMode,
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
    candidates: withCandidateIds(candidates),
  };
}

/**
 * Build a Framer DSL string that creates a landing page from the PRD context.
 * The DSL uses project-update syntax: +NodeType id parent="X" position="N"; SET id attrs;
 */
function buildLandingPageDsl(
  parentFrameId: string,
  projectName: string,
  prdSummary: string,
): string {
  // Extract a tagline from the PRD (first ~80 chars of meaningful text)
  const prdLines = prdSummary.split('\n').filter((l) => l.trim().length > 10);
  const tagline =
    prdLines.find(
      (l) =>
        !l.startsWith('#') &&
        !l.startsWith('-') &&
        !l.startsWith('*') &&
        l.length > 20 &&
        l.length < 120,
    ) || `Welcome to ${projectName}`;

  // Extract service/feature keywords from PRD bullet points
  const bullets = prdLines
    .filter((l) => /^[\s]*[-*•]/.test(l))
    .map((l) => l.replace(/^[\s]*[-*•]\s*/, '').trim())
    .filter((l) => l.length > 5 && l.length < 80)
    .slice(0, 6);

  // Build up to 3 service cards from bullets
  const cards = bullets.slice(0, 3).map((b, i) => {
    const title = b.split(/[.,:;]/).at(0)?.trim().substring(0, 40) || `Feature ${i + 1}`;
    const desc = b.length > title.length ? b : `${title} — professional solutions tailored to your needs.`;
    return { title, desc, idx: i };
  });
  // Fallback cards if PRD had no bullets
  if (cards.length === 0) {
    cards.push(
      { title: 'Services', desc: 'Explore our full range of professional services.', idx: 0 },
      { title: 'Portfolio', desc: 'See our work across events and productions.', idx: 1 },
      { title: 'Contact', desc: 'Get in touch for a custom quote.', idx: 2 },
    );
  }

  const cmds: string[] = [];
  const p = parentFrameId;

  // Configure parent as vertical stack
  cmds.push(
    `SET ${p} layout="stack" stackDirection="vertical" stackAlignment="center" gap="0" overflow="clip";`,
  );

  // ── Hero section ──
  cmds.push(`+FrameNode _hero parent="${p}" position="0";`);
  cmds.push(
    `SET _hero name="Hero" layout="stack" stackDirection="vertical" stackAlignment="center" stackDistribution="center" gap="24" padding="120px 40px" width="1fr" height="auto" background="rgb(10,10,10)" position="relative" overflow="clip";`,
  );
  // Heading
  cmds.push(`+RichTextNode _heroH parent="_hero" position="0";`);
  cmds.push(`SET _heroH name="Heading" width="auto" height="auto" position="relative";`);
  cmds.push(`+TextBlock _heroHB parent="_heroH" position="0";`);
  cmds.push(`SET _heroHB textAlignment="center";`);
  cmds.push(`+TextRun _heroHR parent="_heroHB" position="0";`);
  cmds.push(
    `SET _heroHR content="${escDsl(projectName)}" fontFamily="Geist" fontWeight="700" fontSize="72px" lineHeight="1.1" textColor="rgb(255,255,255)";`,
  );
  // Subtitle
  cmds.push(`+RichTextNode _heroS parent="_hero" position="1";`);
  cmds.push(`SET _heroS name="Subtitle" width="600px" height="auto" position="relative";`);
  cmds.push(`+TextBlock _heroSB parent="_heroS" position="0";`);
  cmds.push(`SET _heroSB textAlignment="center";`);
  cmds.push(`+TextRun _heroSR parent="_heroSB" position="0";`);
  cmds.push(
    `SET _heroSR content="${escDsl(tagline)}" fontFamily="Geist" fontWeight="400" fontSize="20px" lineHeight="1.6" textColor="rgb(153,153,153)";`,
  );
  // CTA row
  cmds.push(`+FrameNode _ctaR parent="_hero" position="2";`);
  cmds.push(
    `SET _ctaR name="CTA" layout="stack" stackDirection="horizontal" stackAlignment="center" gap="16" width="auto" height="auto" position="relative";`,
  );
  cmds.push(`+FrameNode _ctaP parent="_ctaR" position="0";`);
  cmds.push(
    `SET _ctaP name="Primary CTA" layout="stack" stackDirection="horizontal" stackAlignment="center" stackDistribution="center" padding="16px 32px" width="auto" height="auto" background="rgb(255,255,255)" borderRadius="8" position="relative" overflow="clip";`,
  );
  cmds.push(`+RichTextNode _ctaPT parent="_ctaP" position="0";`);
  cmds.push(`SET _ctaPT width="auto" height="auto" position="relative";`);
  cmds.push(`+TextBlock _ctaPTB parent="_ctaPT" position="0";`);
  cmds.push(`+TextRun _ctaPTR parent="_ctaPTB" position="0";`);
  cmds.push(
    `SET _ctaPTR content="Get Started" fontFamily="Geist" fontWeight="600" fontSize="16px" textColor="rgb(10,10,10)";`,
  );
  cmds.push(`+FrameNode _ctaS parent="_ctaR" position="1";`);
  cmds.push(
    `SET _ctaS name="Secondary CTA" layout="stack" stackDirection="horizontal" stackAlignment="center" stackDistribution="center" padding="16px 32px" width="auto" height="auto" background="transparent" borderRadius="8" border="1px solid rgb(51,51,51)" position="relative" overflow="clip";`,
  );
  cmds.push(`+RichTextNode _ctaST parent="_ctaS" position="0";`);
  cmds.push(`SET _ctaST width="auto" height="auto" position="relative";`);
  cmds.push(`+TextBlock _ctaSTB parent="_ctaST" position="0";`);
  cmds.push(`+TextRun _ctaSTR parent="_ctaSTB" position="0";`);
  cmds.push(
    `SET _ctaSTR content="Learn More" fontFamily="Geist" fontWeight="600" fontSize="16px" textColor="rgb(255,255,255)";`,
  );

  // ── Services section ──
  cmds.push(`+FrameNode _svc parent="${p}" position="1";`);
  cmds.push(
    `SET _svc name="Services" layout="stack" stackDirection="vertical" stackAlignment="center" gap="40" padding="80px 40px" width="1fr" height="auto" background="rgb(255,255,255)" position="relative" overflow="clip";`,
  );
  cmds.push(`+RichTextNode _svcH parent="_svc" position="0";`);
  cmds.push(`SET _svcH name="Services Heading" width="auto" height="auto" position="relative";`);
  cmds.push(`+TextBlock _svcHB parent="_svcH" position="0";`);
  cmds.push(`SET _svcHB textAlignment="center";`);
  cmds.push(`+TextRun _svcHR parent="_svcHB" position="0";`);
  cmds.push(
    `SET _svcHR content="What We Do" fontFamily="Geist" fontWeight="700" fontSize="48px" lineHeight="1.2" textColor="rgb(10,10,10)";`,
  );
  // Grid
  cmds.push(`+FrameNode _svcG parent="_svc" position="1";`);
  cmds.push(
    `SET _svcG name="Grid" layout="grid" gridColumnCount="${Math.min(cards.length, 3)}" gridRowHeightType="auto" gap="24" width="1fr" height="auto" position="relative";`,
  );
  for (const card of cards) {
    const ci = card.idx;
    cmds.push(`+FrameNode _cd${ci} parent="_svcG" position="${ci}";`);
    cmds.push(
      `SET _cd${ci} name="${escDsl(card.title)}" layout="stack" stackDirection="vertical" gap="12" padding="32px" width="1fr" height="auto" background="rgb(245,245,245)" borderRadius="12" position="relative" overflow="clip";`,
    );
    cmds.push(`+RichTextNode _cd${ci}T parent="_cd${ci}" position="0";`);
    cmds.push(`SET _cd${ci}T width="auto" height="auto" position="relative";`);
    cmds.push(`+TextBlock _cd${ci}TB parent="_cd${ci}T" position="0";`);
    cmds.push(`+TextRun _cd${ci}TR parent="_cd${ci}TB" position="0";`);
    cmds.push(
      `SET _cd${ci}TR content="${escDsl(card.title)}" fontFamily="Geist" fontWeight="600" fontSize="20px" textColor="rgb(10,10,10)";`,
    );
    cmds.push(`+RichTextNode _cd${ci}D parent="_cd${ci}" position="1";`);
    cmds.push(`SET _cd${ci}D width="auto" height="auto" position="relative";`);
    cmds.push(`+TextBlock _cd${ci}DB parent="_cd${ci}D" position="0";`);
    cmds.push(`+TextRun _cd${ci}DR parent="_cd${ci}DB" position="0";`);
    cmds.push(
      `SET _cd${ci}DR content="${escDsl(card.desc)}" fontFamily="Geist" fontWeight="400" fontSize="16px" lineHeight="1.6" textColor="rgb(100,100,100)";`,
    );
  }

  // ── Footer section ──
  cmds.push(`+FrameNode _ftr parent="${p}" position="2";`);
  cmds.push(
    `SET _ftr name="Footer" layout="stack" stackDirection="vertical" stackAlignment="center" stackDistribution="center" gap="8" padding="40px" width="1fr" height="auto" background="rgb(10,10,10)" position="relative" overflow="clip";`,
  );
  cmds.push(`+RichTextNode _ftrT parent="_ftr" position="0";`);
  cmds.push(`SET _ftrT width="auto" height="auto" position="relative";`);
  cmds.push(`+TextBlock _ftrTB parent="_ftrT" position="0";`);
  cmds.push(`SET _ftrTB textAlignment="center";`);
  cmds.push(`+TextRun _ftrTR parent="_ftrTB" position="0";`);
  cmds.push(
    `SET _ftrTR content="© ${new Date().getFullYear()} ${escDsl(projectName)}" fontFamily="Geist" fontWeight="400" fontSize="14px" textColor="rgb(100,100,100)";`,
  );

  return cmds.join(' ');
}

/** Escape double-quotes for the Framer DSL attribute values. */
function escDsl(s: string): string {
  return s.replace(/"/g, '\\"').replace(/\n/g, ' ');
}

async function generateFramerCandidates(
  context: ProviderExecutionContext,
  outputDir: string,
): Promise<ProviderSummary> {
  const warnings: string[] = [];
  if (!context.hasFrontend) {
    return createEmptySummary('frontend_not_detected');
  }

  const apiKey = process.env['FRAMER_API_KEY']?.trim();
  const projectRef = context.framerProjectRef || process.env['FRAMER_PROJECT_URL']?.trim() || process.env['FRAMER_PROJECT_ID']?.trim() || '';
  const projectUrl = resolveFramerProjectUrl(projectRef);
  if (!apiKey || !projectUrl) {
    warnings.push('FRAMER_API_KEY and a Framer project target (design_framer_project, FRAMER_PROJECT_URL, or FRAMER_PROJECT_ID) are required; skipping Framer generation.');
    return {
      status: 'skipped',
      reason: 'missing_framer_credentials',
      warnings,
      candidates: [],
    };
  }

  const framerDir = join(outputDir, 'framer');
  await mkdir(framerDir, { recursive: true });

  let publishUrl = process.env['FRAMER_PREVIEW_URL']?.trim();
  let publishId: string | undefined;
  let changedPaths: { added: string[]; removed: string[]; modified: string[] } | undefined;
  let contributorCount: number | undefined;
  let sdkAvailable = false;
  let dslApplied = false;

  try {
    const framerApiMod = framerApiModule;
    const connect = framerApiMod?.connect;
    if (typeof connect === 'function') {
      sdkAvailable = true;
      const framer = await connect(projectUrl, apiKey);
      try {
        // 1. Read page structure to find the Desktop breakpoint frame
        const webPages = await framer.getNodesWithType?.('WebPageNode');
        const pageId = webPages?.[0]?.id;
        let parentFrameId = pageId; // fallback to page root

        if (pageId) {
          const children = await framer.getChildren?.(pageId);
          // Find the Desktop breakpoint (main content container)
          const desktop = children?.find(
            (c: { name?: string; __class?: string }) =>
              c.__class === 'FrameNode' && /desktop/i.test(c.name || ''),
          );
          if (desktop) parentFrameId = desktop.id;
        }

        // 2. Build landing page DSL from PRD context
        if (parentFrameId) {
          const dsl = buildLandingPageDsl(parentFrameId, context.projectName, context.prompt);
          await writeFile(join(framerDir, 'applied-dsl.txt'), dsl);
          await framer.applyAgentChanges(dsl, { pagePath: '/' });
          dslApplied = true;
        }

        // 3. Publish (retry with backoff — tree may need time to settle after changes)
        for (let attempt = 0; attempt < 3; attempt++) {
          try {
            if (attempt > 0) await new Promise((r) => setTimeout(r, 2000 * attempt));
            const publishResult = await framer.publish?.();
            publishId = publishResult?.deployment?.id;
            if (publishId && framer.getPublishInfo) {
              const publishInfo = await framer.getPublishInfo();
              publishUrl =
                publishInfo?.staging?.url ||
                publishInfo?.production?.url ||
                publishInfo?.previewUrl ||
                publishInfo?.preview_url ||
                publishInfo?.stagingUrl ||
                publishInfo?.staging_url ||
                publishUrl;
            }
            break; // success
          } catch (pubErr) {
            if (attempt === 2) {
              warnings.push(`Framer publish failed after ${attempt + 1} attempts: ${pubErr instanceof Error ? pubErr.message : String(pubErr)}`);
            }
          }
        }
        if (framer.getChangedPaths) {
          changedPaths = await framer.getChangedPaths();
        }
        if (framer.getChangeContributors) {
          const contributors = await framer.getChangeContributors();
          contributorCount = Array.isArray(contributors) ? contributors.length : undefined;
        }
      } finally {
        await framer.disconnect?.();
      }
    } else {
      warnings.push('framer-api package is unavailable; generating Framer artifacts in offline mode.');
    }
  } catch (error) {
    warnings.push(`Framer API call failed: ${error instanceof Error ? error.message : String(error)}`);
  }

  const now = Date.now();
  const candidates = withCandidateIds(
    context.frontendTargets.map((target, idx) => {
      const deviceType = deviceForTarget(target);
      const promptParts = [
        `Create a ${target} Framer implementation direction for ${context.projectName}.`,
        context.prompt ? `Intent: ${context.prompt}` : '',
        context.urls.length > 0 ? `Reference URLs: ${context.urls.join(', ')}` : '',
        'Favor reusable components with clear property controls for Framer code components.',
      ].filter(Boolean);
      return {
        provider: 'framer' as const,
        target,
        deviceType,
        screenId: `framer-${target}-${now + idx}`,
        htmlUrl: publishUrl,
        imageUrl: undefined,
        rationale: `Prepared ${target} Framer candidate with component-library oriented guidance.`,
        prompt: promptParts.join(' '),
        status: 'generated' as const,
        meta: {
          publish_id: publishId,
          publish_url: publishUrl,
          dsl_applied: dslApplied,
          sdk_available: sdkAvailable,
          changed_paths: changedPaths,
          contributor_count: contributorCount,
          project_url: projectUrl,
          project_ref: projectRef,
        },
      };
    }),
  );

  await writeFile(
    join(framerDir, 'framer-run.json'),
    JSON.stringify(
      {
        timestamp: new Date().toISOString(),
        provider: 'framer',
        mode: context.mode,
        providerMode: context.providerMode,
        projectRef,
        projectUrl,
        publishUrl,
        publishId,
        changedPaths,
        contributorCount,
        sdkAvailable,
      },
      null,
      2,
    ),
  );
  await writeFile(join(framerDir, 'candidates.json'), JSON.stringify(candidates, null, 2));

  return {
    status: candidates.length > 0 ? 'generated' : 'failed',
    reason: candidates.length > 0 ? undefined : 'no_candidates_generated',
    warnings,
    candidates,
  };
}

function getProviderHandlers(): Record<DesignProvider, DesignProviderHandler> {
  return {
    stitch: {
      name: 'stitch',
      generate: generateStitchCandidates,
    },
    framer: {
      name: 'framer',
      generate: generateFramerCandidates,
    },
  };
}

async function writeComponentLibraryArtifacts(
  outputDir: string,
  context: Pick<DesignContextResult, 'projectName' | 'providerMode' | 'normalized_candidates' | 'frontendTargets' | 'providers'>,
): Promise<{ componentPath: string; designSystemPath: string }> {
  const providersInUse = Array.from(
    new Set(context.normalized_candidates.map((candidate) => candidate.provider)),
  ) as DesignProvider[];
  const componentLibrary: ComponentLibraryArtifact = {
    version: '1.0.0',
    projectName: context.projectName,
    provider_mode: context.providerMode,
    providers: providersInUse,
    tokens: {
      color: [
        { name: 'color.background.base', value: '#0B0C0F', description: 'Primary app background' },
        { name: 'color.text.primary', value: '#F4F6F8', description: 'Primary text color' },
        { name: 'color.accent.brand', value: '#7C5CFF', description: 'Primary call-to-action accent' },
      ],
      typography: [
        { name: 'font.family.base', value: 'Inter, system-ui, sans-serif' },
        { name: 'font.size.body', value: '16px' },
        { name: 'font.size.h1', value: '40px' },
      ],
      spacing: [
        { name: 'space.2', value: '8px' },
        { name: 'space.4', value: '16px' },
        { name: 'space.6', value: '24px' },
      ],
      radius: [
        { name: 'radius.sm', value: '8px' },
        { name: 'radius.md', value: '12px' },
        { name: 'radius.lg', value: '16px' },
      ],
    },
    primitives: [
      {
        name: 'Button',
        description: 'Primary action with variant and size controls.',
        states: ['default', 'hover', 'disabled', 'loading'],
        provider_refs: providersInUse,
      },
      {
        name: 'Input',
        description: 'Text input with hint, error, and success states.',
        states: ['default', 'focus', 'error', 'success'],
        provider_refs: providersInUse,
      },
      {
        name: 'Card',
        description: 'Surface container used for content grouping.',
        states: ['default', 'interactive', 'selected'],
        provider_refs: providersInUse,
      },
    ],
    patterns: [
      {
        name: 'HeroSection',
        description: 'Top-level value proposition area with CTA and social proof.',
        states: ['default', 'with-secondary-cta'],
        provider_refs: providersInUse,
      },
      {
        name: 'FeatureGrid',
        description: 'Responsive feature list with icon, title, and supporting copy.',
        states: ['2-column', '3-column', 'mobile-stack'],
        provider_refs: providersInUse,
      },
      {
        name: 'PricingStack',
        description: 'Tiered pricing cards with highlighted recommended plan.',
        states: ['monthly', 'annual', 'enterprise'],
        provider_refs: providersInUse,
      },
    ],
    component_map: context.normalized_candidates.map((candidate) => ({
      target: candidate.target,
      provider: candidate.provider,
      candidate_id: candidate.candidate_id || `${candidate.provider}-${candidate.target}`,
      screen_id: candidate.screenId,
      notes: candidate.rationale,
    })),
    framer_code_components:
      context.providers.framer.status === 'generated'
        ? [
            {
              name: 'HeroCta',
              props: [
                { name: 'headline', type: 'string', default: 'Design systems that ship faster' },
                { name: 'subcopy', type: 'string', default: 'From intake signal to implementation-ready components.' },
                { name: 'ctaLabel', type: 'string', default: 'Get started' },
              ],
              property_controls: ['headline', 'subcopy', 'ctaLabel'],
              notes: 'Use Framer property controls for copy and variant tuning.',
            },
            {
              name: 'FeatureCard',
              props: [
                { name: 'title', type: 'string', default: 'Reusable component primitives' },
                { name: 'description', type: 'string', default: 'Define once, compose everywhere.' },
                { name: 'icon', type: 'string', default: 'sparkles' },
              ],
              property_controls: ['title', 'description', 'icon'],
            },
          ]
        : [],
    generated_at: new Date().toISOString(),
  };

  const componentPath = join(outputDir, 'component-library.json');
  const designSystemPath = join(outputDir, 'design-system.md');
  await writeFile(componentPath, JSON.stringify(componentLibrary, null, 2));

  const designSystemDoc = [
    '# Design System',
    '',
    `Project: ${context.projectName}`,
    `Provider mode: ${context.providerMode}`,
    `Providers generated: ${providersInUse.join(', ') || 'none'}`,
    '',
    '## Scope',
    '- Output includes token foundations, reusable primitives, and interaction patterns.',
    '- Component map ties each candidate back to provider provenance for traceability.',
    '',
    '## Core Primitives',
    '- Button, Input, Card',
    '',
    '## Composition Patterns',
    '- HeroSection, FeatureGrid, PricingStack',
    '',
    '## Provider Notes',
    '- Stitch candidates prioritize visual exploration and variant generation.',
    '- Framer candidates prioritize component composition and property-control readiness.',
    '',
  ].join('\n');
  await writeFile(designSystemPath, designSystemDoc);
  return { componentPath, designSystemPath };
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
  source_provider?: DesignProvider;
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
    source_provider?: DesignProvider;
    variants: DesignVariant[];
  }>;
  total_variants: number;
  timestamp: string;
}

const FIFE_HIRES_SUFFIX = '=w1920';
const VARIANT_LABELS = ['Layout Focus', 'Color Scheme', 'Typography & Spacing'];
const VARIANT_ASPECTS: string[][] = [['LAYOUT'], ['COLOR_SCHEME'], ['TEXT_FONT', 'TEXT_CONTENT']];

function hiresImageUrl(url: string | undefined): string | undefined {
  if (!url) return undefined;
  if (url.includes('=w') || url.includes('=s')) return url;
  return `${url}${FIFE_HIRES_SUFFIX}`;
}

function isProviderCandidateArray(value: unknown): value is ProviderCandidate[] {
  return Array.isArray(value) && value.every((item) => typeof item === 'object' && item !== null && 'target' in item);
}

export async function generateDesignVariants(payload: DesignVariantPayload): Promise<DesignVariantsResult> {
  const { readFile } = await import('node:fs/promises');
  const candidatesRaw = await readFile(payload.candidates_path, 'utf-8');
  const parsedCandidates = JSON.parse(candidatesRaw) as unknown;
  const candidates: ProviderCandidate[] = isProviderCandidateArray(parsedCandidates)
    ? withCandidateIds(parsedCandidates as ProviderCandidate[])
    : [];

  const stitchCandidates = candidates.filter(
    (candidate) => (candidate.provider ?? 'stitch') === 'stitch' && candidate.status === 'generated' && candidate.screenId,
  );
  const nonStitchCandidates = candidates.filter(
    (candidate) => (candidate.provider ?? 'stitch') !== 'stitch' && candidate.status === 'generated',
  );

  let projectId = process.env['STITCH_PROJECT_ID']?.trim() || 'design-intake';
  try {
    const stitchRunRaw = await readFile(payload.candidates_path.replace('candidates.json', 'stitch-run.json'), 'utf-8');
    const stitchRun: { projectId?: string } = JSON.parse(stitchRunRaw);
    if (stitchRun.projectId) {
      projectId = stitchRun.projectId;
    }
  } catch {
    // No stitch run file (e.g. framer-only mode); keep fallback project id.
  }

  const variantCount = payload.variant_count ?? 3;
  const creativeRange = payload.creative_range ?? 'EXPLORE';
  const outputDir = payload.output_dir ?? '.intake/design/stitch';
  const screens: DesignVariantsResult['screens'] = [];

  // Framer and other non-Stitch providers currently provide a baseline generated variant.
  for (const candidate of nonStitchCandidates) {
    const sourceId = candidate.screenId || candidate.candidate_id || `${candidate.provider}-${candidate.target}`;
    screens.push({
      source_screen_id: sourceId,
      source_target: candidate.target,
      source_provider: candidate.provider,
      variants: [
        {
          variant_id: `${sourceId}-v1`,
          source_screen_id: sourceId,
          source_target: candidate.target,
          source_provider: candidate.provider,
          label: 'Baseline',
          description: `${candidate.provider} baseline candidate for ${candidate.target}.`,
          aspects_changed: ['COMPONENT_COMPOSITION'],
          screen_id: candidate.screenId,
          html_url: candidate.htmlUrl,
          image_url: hiresImageUrl(candidate.imageUrl),
          status: 'generated',
        },
      ],
    });
  }

  if (stitchCandidates.length > 0) {
    const stitchApiKey = process.env['STITCH_API_KEY']?.trim();
    if (!stitchApiKey) {
      throw new Error('STITCH_API_KEY is required for Stitch design variant generation');
    }
    const client = new StitchDirectClient(stitchApiKey);
    try {
      for (const candidate of stitchCandidates) {
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
            if (!ref) {
              continue;
            }
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
                source_provider: 'stitch',
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
                source_provider: 'stitch',
                label,
                description: 'Failed to retrieve variant screen.',
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
              source_provider: 'stitch',
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
            source_provider: 'stitch',
            label: 'Original (fallback)',
            description: 'Variant generation failed; using original.',
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
          source_provider: 'stitch',
          variants: screenVariants,
        });
      }
    } finally {
      await client.close();
    }
  }

  const result: DesignVariantsResult = {
    project_id: projectId,
    screens,
    total_variants: screens.reduce((sum, screen) => sum + screen.variants.filter((v) => v.status === 'generated').length, 0),
    timestamp: new Date().toISOString(),
  };

  await mkdir(outputDir, { recursive: true });
  await writeFile(join(outputDir, 'design-variants.json'), JSON.stringify(result, null, 2));
  return result;
}

export async function designIntake(payload: DesignIntakePayload): Promise<DesignContextResult> {
  const outputDir = payload.output_dir?.trim() || '.intake/design';
  const projectName = payload.project_name?.trim() || 'project';
  const mode = payload.design_mode ?? 'ingest_plus_stitch';
  const providerMode = deriveProviderMode(payload);
  const legacyMode = deriveLegacyMode(payload);
  const prompt = payload.design_prompt?.trim() ?? '';
  const artifactPath = payload.design_artifacts_path?.trim() ?? '';
  const urls = parseDesignUrls(payload.design_urls);

  await mkdir(outputDir, { recursive: true });
  await mkdir(join(outputDir, 'crawled'), { recursive: true });

  const combinedInput = [payload.prd_content || '', prompt, urls.join('\n')].join('\n');
  const detection = detectFrontendSignals(combinedInput);
  const crawledUrls = await crawlUrls(urls);
  await writeFile(join(outputDir, 'crawled', 'urls.json'), JSON.stringify(crawledUrls, null, 2));

  const providers: Record<DesignProvider, ProviderSummary> = {
    stitch: createEmptySummary('not_requested'),
    framer: createEmptySummary('not_requested'),
  };

  const providerContext: ProviderExecutionContext = {
    projectName,
    mode,
    providerMode,
    framerProjectRef: payload.design_framer_project?.trim() ?? '',
    prompt,
    artifactPath,
    urls,
    hasFrontend: detection.hasFrontend,
    frontendTargets: detection.frontendTargets,
  };
  const handlers = getProviderHandlers();
  const selectedProviders = requestedProviders(providerMode, legacyMode);

  for (const providerName of selectedProviders) {
    providers[providerName] = await handlers[providerName].generate(providerContext, outputDir);
  }

  const normalizedCandidates = withCandidateIds(
    (['stitch', 'framer'] as DesignProvider[]).flatMap((provider) =>
      providers[provider].candidates.map((candidate) => ({
        ...candidate,
        provider,
      })),
    ),
  );
  await writeFile(join(outputDir, 'candidates.normalized.json'), JSON.stringify(normalizedCandidates, null, 2));

  const context: DesignContextResult = {
    projectName,
    mode,
    providerMode,
    prompt,
    artifactPath,
    urls,
    crawledUrls,
    hasFrontend: detection.hasFrontend,
    frontendTargets: detection.frontendTargets,
    confidence: detection.confidence,
    evidence: detection.evidence,
    providers,
    stitch: providers.stitch,
    framer: providers.framer,
    normalized_candidates: normalizedCandidates,
  };

  const artifacts = await writeComponentLibraryArtifacts(outputDir, {
    projectName: context.projectName,
    providerMode: context.providerMode,
    normalized_candidates: context.normalized_candidates,
    frontendTargets: context.frontendTargets,
    providers: context.providers,
  });
  context.component_library = { path: artifacts.componentPath, format: 'json', version: '1.0.0' };
  context.design_system = { path: artifacts.designSystemPath, format: 'markdown' };

  await writeFile(join(outputDir, 'design-context.json'), JSON.stringify(context, null, 2));
  return context;
}
