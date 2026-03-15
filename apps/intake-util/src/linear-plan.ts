/**
 * linear-plan — Update Linear agent session plan (progress checklist).
 */

const LINEAR_API_URL = 'https://api.linear.app/graphql';

interface PlanStep {
  content: string;
  status: 'pending' | 'inProgress' | 'completed' | 'canceled';
}

interface LinearPlanArgs {
  sessionId: string;
  plan: PlanStep[];
}

export async function linearPlan(args: LinearPlanArgs): Promise<void> {
  const apiKey = process.env['LINEAR_API_KEY'];
  if (!apiKey) throw new Error('LINEAR_API_KEY not set');

  const authHeader = apiKey.startsWith('lin_api_') ? apiKey : `Bearer ${apiKey}`;
  const res = await fetch(LINEAR_API_URL, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', Authorization: authHeader },
    body: JSON.stringify({
      query: `mutation AgentSessionUpdate($id: String!, $input: AgentSessionUpdateInput!) {
        agentSessionUpdate(id: $id, input: $input) { success }
      }`,
      variables: { id: args.sessionId, input: { plan: args.plan } },
    }),
  });

  if (!res.ok) throw new Error(`Linear API ${res.status}: ${await res.text()}`);
  const json = await res.json() as { data?: { agentSessionUpdate: { success: boolean } }; errors?: Array<{ message: string }> };
  if (json.errors?.length) throw new Error(`GraphQL: ${json.errors.map(e => e.message).join(', ')}`);
  if (!json.data?.agentSessionUpdate.success) throw new Error('Failed to update session plan');
}
