/**
 * PRD Research Operation
 *
 * Fires targeted Tavily searches before the deliberation debate to give each
 * agent evidence for their position:
 *   - Optimist memo: best practices and proven patterns
 *   - Pessimist memo: known failure modes and operational risks
 *
 * TODO: TAVILY_API_KEY must be added to the CodeRun pod secret so it is
 *       available as process.env.TAVILY_API_KEY at runtime.
 */

import type { ResearchMemos } from '../types';

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
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      api_key: apiKey,
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

/**
 * Summarize Tavily results into a concise memo paragraph.
 */
function summarizeResults(response: TavilyResponse, query: string): string {
  if (response.answer) {
    return `**Query**: ${query}\n\n${response.answer}`;
  }

  if (response.results.length === 0) {
    return `**Query**: ${query}\n\nNo results found.`;
  }

  const snippets = response.results
    .slice(0, 3)
    .map(r => `- **${r.title}**: ${r.content.slice(0, 300)}`)
    .join('\n');

  return `**Query**: ${query}\n\n${snippets}`;
}

/**
 * Run pre-debate PRD research via Tavily.
 * Returns two focused research memos: one for the optimist agent (best practices)
 * and one for the pessimist agent (failure modes and risks).
 */
export async function prdResearch(payload: { prd_content: string }): Promise<ResearchMemos> {
  const { prd_content } = payload;
  const { techStack, domain } = extractContext(prd_content);

  console.error(`[PRD-RESEARCH] Tech stack detected: ${techStack}`);
  console.error(`[PRD-RESEARCH] Domain detected: ${domain}`);

  // Fire 4 targeted searches in parallel
  const [bestPractices, productionProblems, architecturePatterns, failureModes] = await Promise.all([
    tavilySearch(`${techStack} best practices 2025`),
    tavilySearch(`${techStack} production deployment problems`),
    tavilySearch(`${domain} architecture patterns`),
    tavilySearch(`${domain} ${techStack} failure modes operational risks`),
  ]);

  const optimist = [
    '# Research Memo: What\'s Proven and Working\n',
    summarizeResults(bestPractices, `${techStack} best practices 2025`),
    '',
    summarizeResults(architecturePatterns, `${domain} architecture patterns`),
  ].join('\n');

  const pessimist = [
    '# Research Memo: Known Failure Modes and Risks\n',
    summarizeResults(productionProblems, `${techStack} production deployment problems`),
    '',
    summarizeResults(failureModes, `${domain} ${techStack} failure modes operational risks`),
  ].join('\n');

  console.error('[PRD-RESEARCH] Research memos compiled');

  return { optimist, pessimist };
}
