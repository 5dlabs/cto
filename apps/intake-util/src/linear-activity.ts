/**
 * linear-activity — Direct Linear Agent API activity creation.
 * Supports CLI aliases: thought, plan, action, elicitation, response, error
 * and normalizes to Linear-supported content types.
 */

const LINEAR_API_URL = 'https://api.linear.app/graphql';

type ActivityType = 'thought' | 'plan' | 'action' | 'elicitation' | 'response' | 'error';
type LinearContentType = 'action' | 'elicitation' | 'response' | 'error';

interface LinearActivityArgs {
  sessionId: string;
  type: ActivityType;
  body: string;
  ephemeral?: boolean;
  // For 'action' type
  action?: string;
  parameter?: string;
  result?: string;
  // For 'elicitation' with select signal
  signal?: 'select';
  options?: Array<{ label: string; value: string }>;
  /** Raw signalMetadata JSON — takes precedence over `options` when both are set */
  signalMetadata?: Record<string, unknown>;
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

export async function linearActivity(args: LinearActivityArgs): Promise<{ id: string }> {
  const apiKey = process.env['LINEAR_API_KEY'];
  if (!apiKey) throw new Error('LINEAR_API_KEY not set');

  // Map CLI aliases to Linear-supported content types.
  // 'thought' and 'plan' both map to 'response' (Linear has no native thought type).
  // 'plan' prepends a plan header so the body is visually distinct from thoughts.
  const normalizedType: LinearContentType = (() => {
    if (args.type === 'thought' || args.type === 'plan') return 'response';
    return args.type;
  })();

  let content: Record<string, string>;
  if (normalizedType === 'action') {
    content = {
      type: 'action',
      action: args.action ?? 'Processing',
      parameter: args.parameter ?? '',
      ...(args.result ? { result: args.result } : {}),
    };
  } else {
    // For 'plan' type, wrap the body with a plan header for visual distinction.
    const body = args.type === 'plan'
      ? `**Plan:** ${args.body}`
      : args.body;
    content = { type: normalizedType, body };
  }

  const input: Record<string, unknown> = {
    agentSessionId: args.sessionId,
    content,
  };
  if (args.ephemeral) input.ephemeral = true;
  if (args.signal) {
    input.signal = args.signal;
    // --signal-metadata takes precedence over --options
    if (args.signalMetadata) {
      input.signalMetadata = args.signalMetadata;
    } else if (args.options) {
      input.signalMetadata = { options: args.options };
    }
  }

  const data = await graphql(apiKey,
    `mutation AgentActivityCreate($input: AgentActivityCreateInput!) {
      agentActivityCreate(input: $input) {
        success
        agentActivity { id }
      }
    }`,
    { input },
  ) as { agentActivityCreate: { success: boolean; agentActivity?: { id: string } } };

  if (!data.agentActivityCreate.success || !data.agentActivityCreate.agentActivity) {
    throw new Error('Failed to create agent activity');
  }
  return data.agentActivityCreate.agentActivity;
}
