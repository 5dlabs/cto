/**
 * PRD Research Operation — Multi-Source
 *
 * Fires targeted searches across multiple research providers before the
 * deliberation debate to give each agent evidence for their position:
 *   - Optimist memo: best practices, proven patterns, architecture guidance
 *   - Pessimist memo: failure modes, operational risks, postmortems
 *
 * Research providers (graceful degradation — missing keys are skipped):
 *   Phase 1 (parallel): Exa, Perplexity, Tavily
 *   Phase 2 (conditional, sequential): Firecrawl deep-extraction
 */

import type { ResearchMemos } from '../types';
import {
  exaSearch,
  perplexityAsk,
  grokXQuery,
  firecrawlExtract,
  selectUrlsForDeepExtract,
  type ResearchResult,
} from './research-sources';

// ---------------------------------------------------------------------------
// Tavily (kept from original implementation)
// ---------------------------------------------------------------------------

interface TavilyResult {
  title: string;
  url: string;
  content: string;
  score?: number;
}

interface TavilyResponse {
  answer?: string;
  results: TavilyResult[];
}

async function tavilySearch(query: string): Promise<TavilyResponse> {
  const apiKey = process.env['TAVILY_API_KEY'];
  if (!apiKey) {
    console.error('[PRD-RESEARCH] TAVILY_API_KEY not set — returning empty results for query:', query);
    return { results: [] };
  }

  const resp = await fetch('https://api.tavily.com/search', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${apiKey}`,
    },
    body: JSON.stringify({
      query,
      search_depth: 'basic',
      max_results: 5,
      include_answer: true,
    }),
  });

  if (!resp.ok) {
    const text = await resp.text();
    console.error(`[PRD-RESEARCH] Tavily search failed (${resp.status}): ${text}`);
    return { results: [] };
  }

  return resp.json() as Promise<TavilyResponse>;
}

// ---------------------------------------------------------------------------
// Context extraction (kept from original implementation)
// ---------------------------------------------------------------------------

/**
 * Extract technology stack and domain keywords from PRD markdown.
 * Scans headings, bold text, and code blocks for named technologies.
 */
function extractContext(prdContent: string): { techStack: string; domain: string } {
  const techKeywords: string[] = [];
  const domainKeywords: string[] = [];

  // Extract from code blocks (e.g. ```typescript, ```python)
  const codeBlockLangs = [...prdContent.matchAll(/```(\w+)/g)].map(m => m[1]).filter(Boolean) as string[];
  techKeywords.push(...codeBlockLangs);

  // Extract bold text (**something**) — often tech/product names
  const boldText = [...prdContent.matchAll(/\*\*([^*]+)\*\*/g)].map(m => m[1]).filter(Boolean) as string[];
  techKeywords.push(...boldText.slice(0, 8));

  // Extract H2/H3 headings for domain context
  const headings = [...prdContent.matchAll(/^#{2,3}\s+(.+)$/gm)].map(m => m[1]).filter(Boolean) as string[];
  domainKeywords.push(...headings.slice(0, 5));

  // Common tech stack markers in plain text
  const techPatterns = [
    /\b(React|Vue|Angular|Svelte|Next\.js|Nuxt|Remix)\b/gi,
    /\b(Node\.js|Bun|Deno|Python|Go|Rust|Java|\.NET|Ruby)\b/gi,
    /\b(PostgreSQL|MySQL|MongoDB|Redis|SQLite|Cassandra|DynamoDB)\b/gi,
    /\b(Kubernetes|Docker|AWS|GCP|Azure|Terraform|Pulumi)\b/gi,
    /\b(GraphQL|REST|gRPC|WebSocket|NATS|Kafka|RabbitMQ)\b/gi,
    /\b(TypeScript|JavaScript|Python|Golang|Rust)\b/gi,
  ];
  for (const pattern of techPatterns) {
    const matches = [...prdContent.matchAll(pattern)].map(m => m[0]);
    techKeywords.push(...matches);
  }

  const uniqueTech = [...new Set(techKeywords.filter(t => t.length > 1))].slice(0, 6);
  const uniqueDomain = [...new Set(domainKeywords.filter(d => d.length > 2))].slice(0, 3);

  const techStack = uniqueTech.length > 0 ? uniqueTech.join(', ') : 'web application';
  const domain = uniqueDomain.length > 0 ? uniqueDomain.join(', ') : 'software system';

  return { techStack, domain };
}

// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

/**
 * Convert Tavily response into ResearchResult[] for uniform handling.
 */
function tavilyToResearchResults(response: TavilyResponse): ResearchResult[] {
  return response.results.map((r) => ({
    source: 'tavily' as const,
    title: r.title,
    url: r.url,
    content: r.content,
    score: r.score,
  }));
}

/**
 * Format a section of research results into a readable memo block.
 */
function formatSection(
  heading: string,
  source: string,
  results: ResearchResult[],
): string {
  if (results.length === 0) {
    return '';
  }

  const snippets = results
    .slice(0, 5)
    .map((r) => {
      const urlRef = r.url ? ` (${r.url})` : '';
      return `- **${r.title}**${urlRef}: ${r.content.slice(0, 400)}`;
    })
    .join('\n');

  return `## ${heading}\n_[Source: ${source}]_\n\n${snippets}`;
}

/**
 * Format a Perplexity synthesized analysis into a memo block.
 */
function formatPerplexitySection(heading: string, analysis: string): string {
  if (!analysis) {
    return '';
  }
  return `## ${heading}\n_[Source: Perplexity]_\n\n${analysis}`;
}

/**
 * Format a Tavily response with its answer into a memo block.
 */
function formatTavilySection(heading: string, response: TavilyResponse, query: string): string {
  if (response.answer) {
    const snippets = response.results
      .slice(0, 3)
      .map((r) => `- **${r.title}** (${r.url}): ${r.content.slice(0, 300)}`)
      .join('\n');

    return `## ${heading}\n_[Source: Tavily]_\n\n**Query**: ${query}\n\n${response.answer}\n\n${snippets}`;
  }

  if (response.results.length === 0) {
    return '';
  }

  const snippets = response.results
    .slice(0, 3)
    .map((r) => `- **${r.title}** (${r.url}): ${r.content.slice(0, 300)}`)
    .join('\n');

  return `## ${heading}\n_[Source: Tavily]_\n\n**Query**: ${query}\n\n${snippets}`;
}

// ---------------------------------------------------------------------------
// Main research orchestration
// ---------------------------------------------------------------------------

/**
 * Run multi-source PRD research.
 *
 * Phase 1 (parallel): Exa, Perplexity, Tavily
 * Phase 2 (conditional): Firecrawl deep-extraction of top URLs
 *
 * Returns two focused research memos:
 *   - optimist: best practices, proven patterns, architecture guidance
 *   - pessimist: failure modes, operational risks, postmortems
 */
export async function prdResearch(payload: { prd_content: string }): Promise<ResearchMemos> {
  const { prd_content } = payload;
  const { techStack, domain } = extractContext(prd_content);

  console.error(`[PRD-RESEARCH] Tech stack detected: ${techStack}`);
  console.error(`[PRD-RESEARCH] Domain detected: ${domain}`);

  // =========================================================================
  // Phase 1 — parallel searches across Exa, Perplexity, and Tavily
  // =========================================================================
  console.error('[PRD-RESEARCH] Phase 1: launching parallel searches (Exa, Perplexity, Tavily, Grok/X)');

  const [
    exaArchitecture,
    exaSimilarSystems,
    perplexityAnalysis,
    tavilyBestPractices,
    tavilyFailureModes,
    xSentiment,
  ] = await Promise.all([
    // Exa: architecture patterns for tech stack
    exaSearch(`architecture patterns and best practices for ${techStack}`),

    // Exa: similar systems in the domain
    exaSearch(`${domain} system design case studies and implementation examples`),

    // Perplexity: synthesized tradeoff analysis
    perplexityAsk(
      `Analyze tradeoffs for building ${domain} with ${techStack}. ` +
      `Cover best practices, common pitfalls, and recommended architecture patterns. ` +
      `Include both optimistic success patterns and pessimistic failure scenarios.`,
    ),

    // Tavily: best practices (existing pattern)
    tavilySearch(`${techStack} best practices ${new Date().getFullYear()}`),

    // Tavily: failure modes (existing pattern)
    tavilySearch(`${domain} ${techStack} failure modes operational risks`),

    // Grok/X: real-time social signal intelligence
    grokXQuery(
      `What are developers and engineers saying on X about ${techStack} for ${domain}? ` +
      `Summarize the key opinions, experiences, gotchas, and recommendations. ` +
      `Include both positive experiences and complaints/warnings.`,
    ),
  ]);

  console.error(
    `[PRD-RESEARCH] Phase 1 results — ` +
    `Exa architecture: ${exaArchitecture.length}, ` +
    `Exa similar: ${exaSimilarSystems.length}, ` +
    `Perplexity: ${perplexityAnalysis.length > 0 ? 'yes' : 'no'}, ` +
    `Tavily best practices: ${tavilyBestPractices.results.length}, ` +
    `Tavily failure modes: ${tavilyFailureModes.results.length}, ` +
    `Grok/X: ${xSentiment.length > 0 ? 'yes' : 'no'}`,
  );

  // =========================================================================
  // Phase 2 — conditional Firecrawl deep-extraction
  // =========================================================================
  // Collect all Phase 1 results into a flat list for URL selection
  const allPhase1Results: ResearchResult[] = [
    ...exaArchitecture,
    ...exaSimilarSystems,
    ...tavilyToResearchResults(tavilyBestPractices),
    ...tavilyToResearchResults(tavilyFailureModes),
  ];

  const urlsToExtract = selectUrlsForDeepExtract(allPhase1Results, 3);
  let firecrawlResults: ResearchResult[] = [];

  if (urlsToExtract.length > 0) {
    console.error(`[PRD-RESEARCH] Phase 2: deep-extracting ${urlsToExtract.length} URLs via Firecrawl`);
    firecrawlResults = await firecrawlExtract(urlsToExtract);
    console.error(`[PRD-RESEARCH] Phase 2 results — Firecrawl: ${firecrawlResults.length} pages extracted`);
  } else {
    console.error('[PRD-RESEARCH] Phase 2: no candidate URLs for deep extraction — skipping Firecrawl');
  }

  // =========================================================================
  // Compile memos
  // =========================================================================
  console.error('[PRD-RESEARCH] Compiling research memos');

  // Split Perplexity analysis roughly in half for optimist/pessimist.
  // If the analysis contains a clear break around risks/pitfalls, split there.
  const { optimistAnalysis, pessimistAnalysis } = splitPerplexityAnalysis(perplexityAnalysis);

  // Split X sentiment into optimist/pessimist halves
  const { optimistAnalysis: xOptimist, pessimistAnalysis: xPessimist } = splitPerplexityAnalysis(xSentiment);

  // Split Firecrawl results: those from architecture-ish URLs go to optimist,
  // postmortem-ish URLs go to pessimist, rest split evenly.
  const { optimistExtracts, pessimistExtracts } = splitFirecrawlResults(firecrawlResults);

  // Filter Exa results by relevance to each perspective
  const exaOptimistResults = [
    ...exaArchitecture,          // Architecture patterns are optimist material
    ...exaSimilarSystems.filter((r) =>
      !r.content.toLowerCase().includes('fail') &&
      !r.content.toLowerCase().includes('postmortem') &&
      !r.content.toLowerCase().includes('incident'),
    ),
  ];

  const exaPessimistResults = [
    ...exaSimilarSystems.filter((r) =>
      r.content.toLowerCase().includes('fail') ||
      r.content.toLowerCase().includes('postmortem') ||
      r.content.toLowerCase().includes('incident') ||
      r.content.toLowerCase().includes('risk') ||
      r.content.toLowerCase().includes('pitfall'),
    ),
  ];

  // Assemble optimist memo
  const optimistSections = [
    '# Research Memo: What\'s Proven and Working\n',
    formatSection('Architecture Patterns', 'Exa', exaOptimistResults),
    formatPerplexitySection('Synthesized Analysis — Best Practices', optimistAnalysis),
    formatTavilySection(
      'Industry Best Practices',
      tavilyBestPractices,
      `${techStack} best practices ${new Date().getFullYear()}`,
    ),
    formatPerplexitySection('Developer Sentiment on X — Positive', xOptimist),
    formatSection('Deep Extracts — Architecture Docs', 'Firecrawl', optimistExtracts),
  ].filter(Boolean);

  const optimist = optimistSections.join('\n\n');

  // Assemble pessimist memo
  const pessimistSections = [
    '# Research Memo: Known Failure Modes and Risks\n',
    formatSection('Failure Case Studies', 'Exa', exaPessimistResults),
    formatPerplexitySection('Synthesized Analysis — Risks & Pitfalls', pessimistAnalysis),
    formatTavilySection(
      'Failure Modes & Operational Risks',
      tavilyFailureModes,
      `${domain} ${techStack} failure modes operational risks`,
    ),
    formatPerplexitySection('Developer Sentiment on X — Warnings', xPessimist),
    formatSection('Deep Extracts — Postmortem Details', 'Firecrawl', pessimistExtracts),
  ].filter(Boolean);

  const pessimist = pessimistSections.join('\n\n');

  console.error('[PRD-RESEARCH] Research memos compiled (multi-source)');

  return { optimist, pessimist };
}

// ---------------------------------------------------------------------------
// Analysis splitting helpers
// ---------------------------------------------------------------------------

/**
 * Split Perplexity analysis into optimist and pessimist portions.
 * Looks for natural break points around risk/pitfall sections.
 */
function splitPerplexityAnalysis(analysis: string): {
  optimistAnalysis: string;
  pessimistAnalysis: string;
} {
  if (!analysis) {
    return { optimistAnalysis: '', pessimistAnalysis: '' };
  }

  // Try to find a natural break at a heading containing risk/pitfall/failure keywords
  const riskHeadingPattern = /\n(#{1,3}\s*.*(pitfall|risk|failure|drawback|limitation|concern|challenge|caveat|downside|warning).*)\n/i;
  const match = analysis.match(riskHeadingPattern);

  if (match && match.index !== undefined) {
    const optimistAnalysis = analysis.slice(0, match.index).trim();
    const pessimistAnalysis = analysis.slice(match.index).trim();
    return { optimistAnalysis, pessimistAnalysis };
  }

  // Fallback: try to split at paragraph containing risk keywords
  const paragraphs = analysis.split(/\n\n+/);
  const riskIdx = paragraphs.findIndex((p) =>
    /\b(pitfall|risk|failure|drawback|limitation|concern|however|warning|caveat|downside)\b/i.test(p),
  );

  if (riskIdx > 0) {
    const optimistAnalysis = paragraphs.slice(0, riskIdx).join('\n\n').trim();
    const pessimistAnalysis = paragraphs.slice(riskIdx).join('\n\n').trim();
    return { optimistAnalysis, pessimistAnalysis };
  }

  // Last fallback: split roughly in half
  const midpoint = Math.floor(paragraphs.length / 2);
  return {
    optimistAnalysis: paragraphs.slice(0, midpoint).join('\n\n').trim(),
    pessimistAnalysis: paragraphs.slice(midpoint).join('\n\n').trim(),
  };
}

/**
 * Split Firecrawl deep-extracted results into optimist and pessimist buckets
 * based on URL patterns and content keywords.
 */
function splitFirecrawlResults(results: ResearchResult[]): {
  optimistExtracts: ResearchResult[];
  pessimistExtracts: ResearchResult[];
} {
  const optimistExtracts: ResearchResult[] = [];
  const pessimistExtracts: ResearchResult[] = [];

  for (const r of results) {
    const lowerUrl = (r.url ?? '').toLowerCase();
    const lowerContent = r.content.toLowerCase();

    const isPostmortem =
      lowerUrl.includes('postmortem') ||
      lowerUrl.includes('post-mortem') ||
      lowerUrl.includes('incident') ||
      lowerContent.includes('postmortem') ||
      lowerContent.includes('root cause') ||
      lowerContent.includes('outage');

    if (isPostmortem) {
      pessimistExtracts.push(r);
    } else {
      optimistExtracts.push(r);
    }
  }

  return { optimistExtracts, pessimistExtracts };
}
