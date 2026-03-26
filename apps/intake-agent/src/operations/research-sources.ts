/**
 * Multi-Source Research Providers
 *
 * Direct HTTP integrations with research APIs for the intake pipeline.
 * Each provider gracefully degrades when its API key is missing.
 *
 * Providers:
 *   - Exa: Neural semantic search (requires EXA_API_KEY)
 *   - Perplexity: Synthesized analysis via sonar-pro (requires PERPLEXITY_API_KEY)
 *   - Firecrawl: Deep page extraction to markdown (requires FIRECRAWL_API_KEY)
 *   - Tavily: Already implemented in prd-research.ts
 *   - Grok/X: Real-time social signal intelligence (requires GROK_API_KEY or XAI_API_KEY)
 *   - Dataverse/SN13: Real-time social signal intelligence (requires MC_API or MACROCOSMOS_API_KEY)
 */

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface ResearchResult {
  source: 'exa' | 'perplexity' | 'tavily' | 'firecrawl' | 'grok-x';
  title: string;
  url?: string;
  content: string;
  score?: number;
}

// ---------------------------------------------------------------------------
// Exa — neural semantic search
// https://docs.exa.ai/reference/search
// ---------------------------------------------------------------------------

interface ExaSearchResult {
  title: string;
  url: string;
  text?: string;
  score?: number;
}

interface ExaSearchResponse {
  results: ExaSearchResult[];
}

/**
 * Search Exa for semantically relevant documents.
 * Returns empty array if EXA_API_KEY is not set.
 */
export async function exaSearch(query: string): Promise<ResearchResult[]> {
  const apiKey = process.env['EXA_API_KEY'];
  if (!apiKey) {
    console.error('[RESEARCH-SOURCES] EXA_API_KEY not set — skipping Exa search');
    return [];
  }

  try {
    const resp = await fetch('https://api.exa.ai/search', {
      method: 'POST',
      headers: {
        'x-api-key': apiKey,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        query,
        num_results: 5,
        use_autoprompt: true,
        type: 'neural',
        text: true,
      }),
    });

    if (!resp.ok) {
      const text = await resp.text();
      console.error(`[RESEARCH-SOURCES] Exa search failed (${resp.status}): ${text}`);
      return [];
    }

    const data = (await resp.json()) as ExaSearchResponse;

    return (data.results ?? []).map((r) => ({
      source: 'exa' as const,
      title: r.title ?? 'Untitled',
      url: r.url,
      content: r.text ?? '',
      score: r.score,
    }));
  } catch (err) {
    console.error('[RESEARCH-SOURCES] Exa search error:', err);
    return [];
  }
}

// ---------------------------------------------------------------------------
// Perplexity — synthesized analysis via sonar-pro
// Prefers OpenRouter (OPENROUTER_API_KEY), falls back to direct Perplexity API.
// ---------------------------------------------------------------------------

interface PerplexityResponse {
  choices?: Array<{
    message?: {
      content?: string;
    };
  }>;
}

/**
 * Ask Perplexity a research question and get a synthesized analysis.
 * Tries OpenRouter first (prepaid credits), falls back to direct Perplexity API.
 * Returns empty string if neither key is set.
 */
export async function perplexityAsk(question: string): Promise<string> {
  const openRouterKey = process.env['OPENROUTER_API_KEY'];
  const perplexityKey = process.env['PERPLEXITY_API_KEY'];

  if (!openRouterKey && !perplexityKey) {
    console.error('[RESEARCH-SOURCES] Neither OPENROUTER_API_KEY nor PERPLEXITY_API_KEY set — skipping Perplexity query');
    return '';
  }

  const useOpenRouter = !!openRouterKey;
  const apiUrl = useOpenRouter
    ? 'https://openrouter.ai/api/v1/chat/completions'
    : 'https://api.perplexity.ai/chat/completions';
  const apiKey = useOpenRouter ? openRouterKey : perplexityKey;
  const model = useOpenRouter ? 'perplexity/sonar-pro' : 'sonar-pro';

  console.error(`[RESEARCH-SOURCES] Perplexity query via ${useOpenRouter ? 'OpenRouter' : 'direct API'}`);

  try {
    const resp = await fetch(apiUrl, {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${apiKey}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        model,
        messages: [{ role: 'user', content: question }],
      }),
    });

    if (!resp.ok) {
      const text = await resp.text();
      console.error(`[RESEARCH-SOURCES] Perplexity query failed (${resp.status}): ${text}`);
      return '';
    }

    const data = (await resp.json()) as PerplexityResponse;
    return data.choices?.[0]?.message?.content ?? '';
  } catch (err) {
    console.error('[RESEARCH-SOURCES] Perplexity query error:', err);
    return '';
  }
}

// ---------------------------------------------------------------------------
// Firecrawl — deep page extraction to markdown
// https://docs.firecrawl.dev/api-reference/endpoint/scrape
// ---------------------------------------------------------------------------

interface FirecrawlResponse {
  data?: {
    markdown?: string;
  };
}

/**
 * Extract full-page markdown content from a list of URLs via Firecrawl.
 * Returns empty array if FIRECRAWL_API_KEY is not set.
 * Processes URLs sequentially to respect rate limits.
 */
export async function firecrawlExtract(urls: string[]): Promise<ResearchResult[]> {
  const apiKey = process.env['FIRECRAWL_API_KEY'];
  if (!apiKey) {
    console.error('[RESEARCH-SOURCES] FIRECRAWL_API_KEY not set — skipping Firecrawl extraction');
    return [];
  }

  if (urls.length === 0) {
    return [];
  }

  const results: ResearchResult[] = [];

  for (const url of urls) {
    try {
      const resp = await fetch('https://api.firecrawl.dev/v1/scrape', {
        method: 'POST',
        headers: {
          Authorization: `Bearer ${apiKey}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          url,
          formats: ['markdown'],
        }),
      });

      if (!resp.ok) {
        const text = await resp.text();
        console.error(`[RESEARCH-SOURCES] Firecrawl scrape failed for ${url} (${resp.status}): ${text}`);
        continue;
      }

      const data = (await resp.json()) as FirecrawlResponse;
      const markdown = data.data?.markdown ?? '';

      if (markdown.length > 0) {
        results.push({
          source: 'firecrawl',
          title: extractTitleFromMarkdown(markdown) || url,
          url,
          content: markdown.slice(0, 3000), // Cap per-page content
        });
      }
    } catch (err) {
      console.error(`[RESEARCH-SOURCES] Firecrawl scrape error for ${url}:`, err);
    }
  }

  return results;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/**
 * Extract the first H1/H2 heading from markdown, or return empty string.
 */
function extractTitleFromMarkdown(md: string): string {
  const match = md.match(/^#{1,2}\s+(.+)$/m);
  return match?.[1]?.trim() ?? '';
}

/**
 * Check if a URL looks like an architecture doc or postmortem
 * based on URL path patterns.
 */
export function isDeepExtractCandidate(url: string): boolean {
  const lowerUrl = url.toLowerCase();
  const patterns = ['blog', 'architecture', 'postmortem', 'post-mortem', 'case-study', 'case_study'];
  return patterns.some((p) => lowerUrl.includes(p));
}

/**
 * Select top URLs from research results for deep extraction.
 * Prioritises URLs matching architecture/postmortem patterns,
 * then falls back to highest-scoring results.
 */
export function selectUrlsForDeepExtract(
  results: ResearchResult[],
  maxUrls: number = 3,
): string[] {
  const candidateUrls: Array<{ url: string; priority: number }> = [];

  for (const r of results) {
    if (!r.url) continue;

    const priority = isDeepExtractCandidate(r.url)
      ? 2 + (r.score ?? 0) // Pattern match gets a boost
      : r.score ?? 0;

    candidateUrls.push({ url: r.url, priority });
  }

  // Deduplicate by URL
  const seen = new Set<string>();
  const unique = candidateUrls.filter((c) => {
    if (seen.has(c.url)) return false;
    seen.add(c.url);
    return true;
  });

  // Sort descending by priority, take top N
  unique.sort((a, b) => b.priority - a.priority);
  return unique.slice(0, maxUrls).map((c) => c.url);
}

// ---------------------------------------------------------------------------
// Grok X Search — real-time social signal intelligence
// Uses the Grok Responses API with x_search tool to query X/Twitter.
// Requires XAI_API_KEY or GROK_API_KEY.
// ---------------------------------------------------------------------------

const GROK_API_URL = 'https://api.x.ai/v1';
const GROK_MODEL = 'grok-4-1-fast-reasoning';

interface GrokResponseOutput {
  content?: Array<{
    type: string;
    text?: string;
  }>;
}

/**
 * Query X via Grok's Responses API with x_search enabled.
 * Returns a synthesized natural language analysis of what people are
 * saying on X about the given topic.
 * Returns empty string if no API key is set.
 */
export async function grokXQuery(question: string): Promise<string> {
  const apiKey = process.env['GROK_API_KEY'] ?? process.env['XAI_API_KEY'];
  if (!apiKey) {
    console.error('[RESEARCH-SOURCES] GROK_API_KEY/XAI_API_KEY not set — skipping X search');
    return '';
  }

  try {
    const resp = await fetch(`${GROK_API_URL}/responses`, {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${apiKey}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        model: GROK_MODEL,
        input: [{ role: 'user', content: question }],
        tools: [{
          type: 'x_search',
          enable_image_understanding: false,
          enable_video_understanding: false,
        }],
      }),
    });

    if (!resp.ok) {
      const text = await resp.text();
      console.error(`[RESEARCH-SOURCES] Grok X search failed (${resp.status}): ${text}`);
      return '';
    }

    const data = (await resp.json()) as { output?: GrokResponseOutput[] };
    const parts: string[] = [];
    for (const item of data.output ?? []) {
      for (const entry of item.content ?? []) {
        if (entry.type === 'output_text' && typeof entry.text === 'string') {
          parts.push(entry.text);
        }
      }
    }

    return parts.join('\n\n') || '';
  } catch (err) {
    console.error('[RESEARCH-SOURCES] Grok X search error:', err);
    return '';
  }
}

// ---------------------------------------------------------------------------
// Dataverse CLI (macrocosmos) — real-time social signal intelligence
// Uses the upstream dataverse-cli HTTP endpoint that powers `dv search`.
// Requires MC_API or MACROCOSMOS_API_KEY.
// ---------------------------------------------------------------------------

const DATAVERSE_BASE_URL = 'https://constellation.api.cloud.macrocosmos.ai';
const DATAVERSE_SN13_SERVICE = 'sn13.v1.Sn13Service';
const DATAVERSE_ONDEMAND_METHOD = 'OnDemandData';
const DATAVERSE_CLIENT_ID = 'dataverse-rust-cli';

interface DataverseOnDemandResponse {
  status?: string;
  data?: Array<Record<string, unknown>>;
  meta?: unknown;
}

interface DataversePost {
  author: string;
  text: string;
  datetime: string;
  likes: number;
}

function getDataverseApiKey(): string | undefined {
  return process.env['MC_API'] ?? process.env['MACROCOSMOS_API_KEY'];
}

function getLast24hIsoWindow(): { start: string; end: string } {
  const now = new Date();
  const start = new Date(now.getTime() - 24 * 60 * 60 * 1000);
  return { start: start.toISOString(), end: now.toISOString() };
}

function normalizeDataverseKeywords(techStack: string, domain: string): string[] {
  // Dataverse search accepts up to 5 keywords. We attempt to keep them
  // human-relevant (split on commas) and de-duplicate.
  const fromTechStack = techStack
    .split(',')
    .map((s) => s.trim())
    .filter(Boolean);

  const candidates = [...fromTechStack, domain]
    .map((k) => k.replace(/\\s+/g, ' ').trim())
    .filter((k) => k.length > 1);

  const unique = Array.from(new Set(candidates));
  return unique.slice(0, 5);
}

function extractDataversePost(post: Record<string, unknown>): DataversePost | null {
  const rawText = typeof post['text'] === 'string' ? post['text'] : '';
  const text = rawText.replace(/\\s+/g, ' ').trim();
  if (!text) return null;

  const author = typeof post?.['user'] === 'object' && post?.['user'] !== null
    ? typeof (post['user'] as Record<string, unknown>)['username'] === 'string'
      ? String((post['user'] as Record<string, unknown>)['username'])
      : ''
    : '';

  const datetime = typeof post['datetime'] === 'string' ? post['datetime'] : '';

  const tweet = typeof post['tweet'] === 'object' && post['tweet'] !== null
    ? (post['tweet'] as Record<string, unknown>)
    : undefined;

  const likesRaw = tweet ? tweet['like_count'] : undefined;
  const likes =
    typeof likesRaw === 'number'
      ? likesRaw
      : typeof likesRaw === 'string'
        ? Number.parseInt(likesRaw, 10) || 0
        : 0;

  return { author: author || 'unknown', text, datetime, likes };
}

async function dataverseXSearchPosts(keywords: string[], limit: number): Promise<DataversePost[]> {
  const apiKey = getDataverseApiKey();
  if (!apiKey) {
    console.error('[RESEARCH-SOURCES] MC_API/MACROCOSMOS_API_KEY not set — skipping Dataverse X search');
    return [];
  }

  if (keywords.length === 0) return [];

  const { start, end } = getLast24hIsoWindow();
  const url = `${DATAVERSE_BASE_URL}/${DATAVERSE_SN13_SERVICE}/${DATAVERSE_ONDEMAND_METHOD}`;

  try {
    const resp = await fetch(url, {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${apiKey}`,
        'Content-Type': 'application/json',
        'x-client-id': DATAVERSE_CLIENT_ID,
      },
      body: JSON.stringify({
        // These match dv CLI defaults (`dv search x ...`):
        source: 'X',
        usernames: [],
        keywords,
        start_date: start,
        end_date: end,
        limit,
        keyword_mode: 'any',
      }),
    });

    if (!resp.ok) {
      const text = await resp.text().catch(() => '');
      console.error(`[RESEARCH-SOURCES] Dataverse search failed (${resp.status}): ${text}`);
      return [];
    }

    const data = (await resp.json()) as DataverseOnDemandResponse;
    if (data.status && data.status !== 'success') {
      console.error(`[RESEARCH-SOURCES] Dataverse search returned status=${data.status}`);
      return [];
    }

    const posts = (data.data ?? [])
      .map((p) => extractDataversePost(p))
      .filter((p): p is DataversePost => !!p)
      .sort((a, b) => b.likes - a.likes);

    return posts;
  } catch (err) {
    console.error('[RESEARCH-SOURCES] Dataverse search error:', err);
    return [];
  }
}

/**
 * Summarize developer sentiment from Dataverse (dv search) into an
 * LLM-friendly memo string that can be split into optimist/pessimist.
 */
export async function dataverseDeveloperSentiment(techStack: string, domain: string): Promise<string> {
  // We want this to degrade gracefully: if either Dataverse or Perplexity
  // is missing, we return an empty string.
  const keywords = normalizeDataverseKeywords(techStack, domain);
  if (keywords.length === 0) return '';

  const posts = await dataverseXSearchPosts(keywords, 30);
  if (posts.length === 0) return '';

  const postsSample = posts
    .slice(0, 15)
    .map((p, idx) => {
      const dt = p.datetime ? p.datetime.slice(0, 16) : '';
      return `${idx + 1}. @${p.author} (likes: ${p.likes})${dt ? ` @ ${dt}` : ''}: ${p.text}`;
    })
    .join('\n');

  const perplexityQuestion = [
    'You are analyzing real developer sentiment from X posts collected via Dataverse.',
    'Create a Markdown response with TWO sections using these exact headings:',
    '',
    '## Optimistic experiences and best practices',
    '## Risks, pitfall, failure modes, and warnings',
    '',
    `Tech stack: ${techStack}`,
    `Domain: ${domain}`,
    '',
    'In each bullet, include an author handle and like count from the evidence posts.',
    'Make sure the second section includes clear recommendations for avoiding common failure modes.',
    '',
    'Evidence posts:',
    postsSample,
  ].join('\n');

  const perplexitySummary = await perplexityAsk(perplexityQuestion);
  if (perplexitySummary) return perplexitySummary;

  // Fallback: heuristic sentiment split without relying on Perplexity.
  // This is intentionally conservative and only aims to preserve a
  // structure compatible with `splitPerplexityAnalysis`.
  const negativeWords = [
    'risk',
    'pitfall',
    'failure',
    'warn',
    'warning',
    'drawback',
    'limitation',
    'concern',
    'challenge',
    'issue',
    'bug',
    'broken',
    'outage',
    'slow',
    'cost',
    'expensive',
    'down',
    'fail',
  ];

  const isNegative = (t: string) => {
    const lower = t.toLowerCase();
    return negativeWords.some((w) => lower.includes(w));
  };

  const optimisticPosts = posts.filter((p) => !isNegative(p.text)).slice(0, 8);
  const riskyPosts = posts.filter((p) => isNegative(p.text)).slice(0, 8);

  const formatBullets = (list: DataversePost[]) =>
    list.length === 0
      ? '- (none surfaced in the sampled posts)'
      : list
          .map((p) => {
            const trimmed = p.text.length > 220 ? `${p.text.slice(0, 220)}…` : p.text;
            return `- @${p.author} (likes: ${p.likes}): ${trimmed}`;
          })
          .join('\n');

  return [
    '## Optimistic experiences and best practices',
    formatBullets(optimisticPosts),
    '',
    '## Risks, pitfall, failure modes, and warnings',
    formatBullets(riskyPosts),
  ].join('\n');
}
