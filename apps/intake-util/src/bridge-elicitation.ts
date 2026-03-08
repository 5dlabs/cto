/**
 * bridge-elicitation — POST elicitation request to both bridges.
 * Includes resume token for Lobster workflow continuation.
 */

import type { ElicitationRequest, ElicitationOption, VoteSummary } from './types';

interface BridgeElicitationArgs {
  sessionId: string;
  decisionId: string;
  voteResult: {
    question?: string;
    category?: string;
    options?: ElicitationOption[];
    vote_summary?: VoteSummary;
    recommended_option?: string;
    winning_option?: string;
    [key: string]: unknown;
  };
  linearSessionId?: string;
  resumeToken?: string;
  humanReviewMode?: string;
  linearIssueId?: string;
  discordChannelId?: string;
  runId?: string;
}

async function postToBridge(url: string, payload: unknown): Promise<boolean> {
  try {
    const res = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    return res.ok;
  } catch {
    return false;
  }
}

export async function bridgeElicitation(args: BridgeElicitationArgs): Promise<{ discord: boolean; linear: boolean }> {
  const discordUrl = process.env['DISCORD_BRIDGE_URL'] ?? 'http://discord-bridge.bots.svc:3200';
  const linearUrl = process.env['LINEAR_BRIDGE_URL'] ?? 'http://linear-bridge.bots.svc:3100';

  const vr = args.voteResult;
  const isFullAuto = args.humanReviewMode === 'full_auto';

  const request: ElicitationRequest = {
    elicitation_id: `${args.sessionId}-${args.decisionId}`,
    session_id: args.sessionId,
    decision_id: args.decisionId,
    question: vr.question ?? `Decision ${args.decisionId}`,
    category: vr.category ?? 'general',
    options: vr.options ?? [],
    recommended_option: vr.recommended_option ?? vr.winning_option,
    vote_summary: vr.vote_summary ?? { total_voters: 0, tally: {}, consensus_strength: 0, escalated: false },
    allow_redeliberation: !isFullAuto,
    timeout_seconds: isFullAuto ? 0 : 300,
    informational: isFullAuto,
    timestamp: new Date().toISOString(),
    linear_issue_id: args.linearIssueId,
    discord_channel_id: args.discordChannelId,
    resume_token: args.resumeToken,
    metadata: {
      ...(args.runId ? { run_id: args.runId } : {}),
    },
  };

  const [discord, linear] = await Promise.all([
    postToBridge(`${discordUrl}/elicitation`, request),
    postToBridge(`${linearUrl}/elicitation`, request),
  ]);

  return { discord, linear };
}
