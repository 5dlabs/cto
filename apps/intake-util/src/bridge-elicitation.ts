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

interface ElicitationStatus {
  active: boolean;
  known: boolean;
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

async function readStatus(url: string): Promise<ElicitationStatus | undefined> {
  try {
    const res = await fetch(url);
    if (!res.ok) return undefined;
    const data = await res.json() as Partial<ElicitationStatus>;
    if (typeof data.active !== 'boolean' || typeof data.known !== 'boolean') {
      return undefined;
    }
    return { active: data.active, known: data.known };
  } catch {
    return undefined;
  }
}

async function waitForResolution(elicitationId: string, discordUrl: string, linearUrl: string, posted: { discord: boolean; linear: boolean }): Promise<void> {
  const pollTargets: string[] = [];
  if (posted.discord) {
    pollTargets.push(`${discordUrl}/elicitation/status/${encodeURIComponent(elicitationId)}`);
  }
  if (posted.linear) {
    pollTargets.push(`${linearUrl}/elicitation/status/${encodeURIComponent(elicitationId)}`);
  }
  if (pollTargets.length === 0) return;

  const timeoutSec = parseInt(process.env['ELICITATION_TIMEOUT_SECONDS'] ?? '300', 10);
  const deadline = timeoutSec > 0 ? Date.now() + (timeoutSec + 30) * 1000 : 0;

  await new Promise((resolve) => setTimeout(resolve, 1000));

  for (;;) {
    if (deadline > 0 && Date.now() > deadline) {
      console.error(`[bridge-elicitation] Timeout waiting for ${elicitationId} — bridge should have auto-selected`);
      return;
    }
    const statuses = await Promise.all(pollTargets.map((target) => readStatus(target)));
    const knownStatuses = statuses.filter((s): s is ElicitationStatus => !!s && s.known);
    if (knownStatuses.length > 0 && knownStatuses.every((s) => !s.active)) {
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, 2000));
  }
}

export async function bridgeElicitation(args: BridgeElicitationArgs): Promise<{ discord: boolean; linear: boolean; waited: boolean }> {
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
    timeout_seconds: isFullAuto ? 0 : parseInt(process.env['ELICITATION_TIMEOUT_SECONDS'] ?? '300', 10),
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

  if (!isFullAuto && (discord || linear)) {
    console.error(`[bridge-elicitation] Waiting for user input on ${request.elicitation_id}...`);
    await waitForResolution(request.elicitation_id, discordUrl, linearUrl, { discord, linear });
    console.error(`[bridge-elicitation] User input received for ${request.elicitation_id}.`);
  }

  return { discord, linear, waited: !isFullAuto && (discord || linear) };
}
