/**
 * linear-plan — Update Linear agent session plan (progress checklist).
 *
 * Accepts plan steps via:
 *   --plan-json <json-string>   Inline JSON array of {content, status} objects
 *   --plan-json -               Read JSON array from stdin
 *   --step <step-id>            Use canonical PIPELINE_STEPS and auto-compute statuses
 *
 * Requires:
 *   --session-id <id>           Linear agent session ID
 *
 * Error handling is non-fatal by design — callers should append `|| true`
 * and the function itself catches and logs rather than throwing.
 */

import { buildPlan } from './plan-steps';

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

/**
 * Build plan steps from either --plan-json or --step flag.
 * Returns null if neither is provided.
 */
export function resolvePlanSteps(
  planJson: string | undefined,
  stepId: string | undefined,
  stdinContent?: string,
): PlanStep[] | null {
  if (stepId) {
    return buildPlan(stepId) as PlanStep[];
  }

  if (planJson) {
    const raw = planJson === '-' ? (stdinContent ?? '') : planJson;
    if (!raw.trim()) return null;
    return JSON.parse(raw) as PlanStep[];
  }

  return null;
}
