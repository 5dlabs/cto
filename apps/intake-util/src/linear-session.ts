/**
 * linear-session — Update Linear agent session external URLs.
 *
 * Uses the agentSessionUpdate mutation to add external URLs (clickable links)
 * to a Linear Agent session. Supports both full replacement and additive modes.
 *
 * ## Session State Management (for reference)
 *
 * Linear automatically manages session state based on activity types:
 *   - Emitting a 'thought' or 'action' activity → session becomes `active`
 *   - Emitting an 'elicitation' activity       → session becomes `awaitingInput`
 *   - Emitting a 'response' activity           → session becomes `complete`
 *   - Emitting an 'error' activity             → session becomes `error`
 *
 * No manual state management is needed — just emit the right activity types
 * via `linear-activity`. The `stale` and `pending` states are managed by
 * Linear internally (e.g., pending = session created but no activity yet,
 * stale = no activity for an extended period).
 */

const LINEAR_API_URL = 'https://api.linear.app/graphql';

interface ExternalUrl {
  label: string;
  url: string;
}

interface LinearSessionUpdateUrlsArgs {
  sessionId: string;
  /** Full replacement: set all external URLs to this array */
  externalUrls?: ExternalUrl[];
  /** Additive: URLs to append to existing external URLs */
  addUrls?: ExternalUrl[];
}

async function graphql(apiKey: string, query: string, variables: Record<string, unknown>): Promise<unknown> {
  const authHeader = apiKey.startsWith('lin_api_') ? apiKey : `Bearer ${apiKey}`;
  const res = await fetch(LINEAR_API_URL, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', Authorization: authHeader },
    body: JSON.stringify({ query, variables }),
  });
  if (!res.ok) throw new Error(`Linear API ${res.status}: ${await res.text()}`);
  const json = await res.json() as { data?: unknown; errors?: Array<{ message: string }> };
  if (json.errors?.length) throw new Error(`GraphQL: ${json.errors.map(e => e.message).join(', ')}`);
  return json.data;
}

async function fetchCurrentExternalUrls(apiKey: string, sessionId: string): Promise<ExternalUrl[]> {
  const data = await graphql(apiKey,
    `query AgentSession($id: String!) {
      agentSession(id: $id) { externalUrls }
    }`,
    { id: sessionId },
  ) as { agentSession?: { externalUrls?: ExternalUrl[] | null } };
  // externalUrls is a JSON scalar — Linear returns it as a plain array
  const raw = data?.agentSession?.externalUrls;
  if (!raw || !Array.isArray(raw)) return [];
  return raw as ExternalUrl[];
}

export async function linearSessionUpdateUrls(args: LinearSessionUpdateUrlsArgs): Promise<{ updated: boolean; externalUrls: ExternalUrl[] }> {
  const apiKey = process.env['LINEAR_API_KEY'];
  if (!apiKey) throw new Error('LINEAR_API_KEY not set');

  let urls: ExternalUrl[];

  if (args.externalUrls) {
    // Full replacement mode
    urls = args.externalUrls;
  } else if (args.addUrls && args.addUrls.length > 0) {
    // Additive mode: fetch current URLs then append new ones (dedup by url)
    const current = await fetchCurrentExternalUrls(apiKey, args.sessionId);
    const existingUrlSet = new Set(current.map(u => u.url));
    const newUrls = args.addUrls.filter(u => !existingUrlSet.has(u.url));
    urls = [...current, ...newUrls];
  } else {
    return { updated: false, externalUrls: [] };
  }

  const data = await graphql(apiKey,
    `mutation AgentSessionUpdate($id: String!, $input: AgentSessionUpdateInput!) {
      agentSessionUpdate(id: $id, input: $input) { success }
    }`,
    { id: args.sessionId, input: { externalUrls: urls } },
  ) as { agentSessionUpdate: { success: boolean } };

  if (!data.agentSessionUpdate.success) throw new Error('Failed to update session external URLs');
  return { updated: true, externalUrls: urls };
}
