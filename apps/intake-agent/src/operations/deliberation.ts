/**
 * Deliberation Operation
 *
 * Orchestrates a time-boxed design debate between the Optimist and Pessimist
 * agents via NATS, with a 5-member multi-model committee voting on decision
 * points raised during the debate.
 *
 * Flow:
 *   1. Broadcast PRD to both debate agents
 *   2. Conduct debate loop (relay turns, enforce timebox)
 *   3. On DECISION_POINT: fan-out committee votes, tally, broadcast result
 *   4. On timeout or consensus: compile design brief, return DeliberationResult
 */

import { createHash, randomUUID } from 'crypto';
import type {
  DeliberatePayload,
  DeliberationResult,
  DeliberationDecisionPoint,
  CommitteeVote,
  DebateTurn,
} from '../types';
import type { AgentResponse, TokenUsage } from '../types';

// =============================================================================
// Constants
// =============================================================================

const DEFAULT_TIMEBOX_MINUTES = 30;
const DEFAULT_VOTE_TIMEOUT_SECONDS = 120;
const DEFAULT_COMMITTEE_IDS = [
  'committee-1',
  'committee-2',
  'committee-3',
  'committee-4',
  'committee-5',
];
const SOFT_WARNING_RATIO = 0.8; // Send warning when 80% of timebox elapsed
const AGENT_SKIP_TIMEOUT_MS = 5 * 60 * 1000;

// =============================================================================
// NATS Abstraction
// =============================================================================

interface NatsMessage {
  type: string;
  session_id: string;
  [key: string]: unknown;
}

interface NatsClient {
  publish(subject: string, message: NatsMessage): Promise<void>;
  subscribe(subject: string, handler: (msg: NatsMessage) => void): () => void;
  request(subject: string, message: NatsMessage, timeoutMs: number): Promise<NatsMessage>;
}

/**
 * Resolve the NATS client from the environment.
 * In production this is provided by the OpenClaw runtime.
 * Falls back to a stub that uses the intake-agent's tool invocation.
 */
async function getNatsClient(): Promise<NatsClient> {
  // OpenClaw injects a global NATS client when the agent is running in-pod.
  // When running in standalone binary mode (testing), we fall back to a stub.
  const globalNats = (globalThis as Record<string, unknown>).__openclaw_nats__;
  if (globalNats) {
    return globalNats as NatsClient;
  }

  // Stub implementation — logs NATS messages to stderr for local testing.
  console.error('[DELIBERATION] NATS client not found — using stub (messages logged only)');
  const handlers: Map<string, ((msg: NatsMessage) => void)[]> = new Map();

  return {
    async publish(subject: string, message: NatsMessage) {
      console.error(`[NATS→${subject}]`, JSON.stringify(message, null, 2));
    },
    subscribe(subject: string, handler: (msg: NatsMessage) => void) {
      if (!handlers.has(subject)) handlers.set(subject, []);
      handlers.get(subject)!.push(handler);
      return () => {
        const list = handlers.get(subject) ?? [];
        const idx = list.indexOf(handler);
        if (idx >= 0) list.splice(idx, 1);
      };
    },
    async request(subject: string, message: NatsMessage, timeoutMs: number): Promise<NatsMessage> {
      console.error(`[NATS REQUEST→${subject}]`, JSON.stringify(message, null, 2));
      // Stub returns a synthetic abstain vote for testing
      await new Promise(res => setTimeout(res, Math.min(timeoutMs, 100)));
      return {
        type: 'vote_response',
        session_id: message.session_id,
        voter_id: subject,
        decision_id: (message as Record<string, unknown>).decision_id as string,
        chosen_option: ((message as Record<string, unknown>).options as string[])?.[0] ?? 'abstain',
        confidence: 0.5,
        reasoning: 'Stub vote (NATS not connected)',
        concerns: [],
      };
    },
  };
}

// =============================================================================
// Decision Point Parser
// =============================================================================

const DECISION_POINT_REGEX = /DECISION_POINT:\s*\n\s*id:\s*(\S+)\s*\n\s*category:\s*(\S+)\s*\n\s*question:\s*(.+?)\s*\n\s*my_option:\s*(.+?)\s*\n\s*reasoning:\s*(.+?)(?=\n\n|\n[A-Z]|$)/gs;

interface ParsedDecisionPoint {
  id: string;
  category: string;
  question: string;
  proposingOption: string;
  reasoning: string;
  raisedBy: 'optimist' | 'pessimist';
}

function parseDecisionPoints(content: string, speaker: 'optimist' | 'pessimist'): ParsedDecisionPoint[] {
  const points: ParsedDecisionPoint[] = [];
  let match: RegExpExecArray | null;

  // Reset regex
  DECISION_POINT_REGEX.lastIndex = 0;

  while ((match = DECISION_POINT_REGEX.exec(content)) !== null) {
    points.push({
      id: (match[1] ?? '').trim(),
      category: (match[2] ?? '').trim(),
      question: (match[3] ?? '').trim(),
      proposingOption: (match[4] ?? '').trim(),
      reasoning: (match[5] ?? '').trim(),
      raisedBy: speaker,
    });
  }
  return points;
}

// =============================================================================
// Committee Voting
// =============================================================================

async function conductCommitteeVote(
  nats: NatsClient,
  sessionId: string,
  dp: ParsedDecisionPoint,
  optimistPosition: string,
  pessimistPosition: string,
  prdContext: string,
  committeeIds: string[],
  voteTimeoutSeconds: number
): Promise<DeliberationDecisionPoint> {
  console.error(`[DELIBERATION] Committee vote on ${dp.id}: "${dp.question}"`);

  const options = [optimistPosition, pessimistPosition];

  // Fan out vote requests to all committee members in parallel
  const votePromises = committeeIds.map(async (voterId): Promise<CommitteeVote> => {
    const request: NatsMessage = {
      type: 'vote_request',
      session_id: sessionId,
      decision_id: dp.id,
      question: dp.question,
      category: dp.category,
      options,
      optimist_position: optimistPosition,
      pessimist_position: pessimistPosition,
      context: prdContext.slice(0, 2000), // Limit context size
      deadline_seconds: voteTimeoutSeconds,
    };

    try {
      const response = await nats.request(
        `agent.${voterId}.inbox`,
        request,
        voteTimeoutSeconds * 1000
      );

      return {
        voter_id: voterId,
        chosen_option: (response.chosen_option as string) ?? 'abstain',
        confidence: (response.confidence as number) ?? 0,
        reasoning: (response.reasoning as string) ?? 'No reasoning provided',
        concerns: (response.concerns as string[]) ?? [],
        timestamp: new Date().toISOString(),
      };
    } catch {
      console.error(`[DELIBERATION] ${voterId} timed out — marking abstain`);
      return {
        voter_id: voterId,
        chosen_option: 'abstain',
        confidence: 0,
        reasoning: 'Vote timed out',
        concerns: [],
        timestamp: new Date().toISOString(),
      };
    }
  });

  const votes = await Promise.all(votePromises);

  // Tally votes (abstains are excluded from majority calculation)
  const tally: Record<string, number> = {};
  for (const vote of votes) {
    if (vote.chosen_option !== 'abstain') {
      tally[vote.chosen_option] = (tally[vote.chosen_option] ?? 0) + 1;
    }
  }

  const nonAbstainVotes = votes.filter(v => v.chosen_option !== 'abstain');
  const maxVotes = Math.max(...Object.values(tally), 0);
  const winners = Object.entries(tally).filter(([, count]) => count === maxVotes);
  const isTie = winners.length > 1 || nonAbstainVotes.length === 0;
  const winningOption = isTie ? undefined : winners[0]?.[0];
  const consensusStrength = nonAbstainVotes.length > 0
    ? maxVotes / nonAbstainVotes.length
    : 0;

  console.error(
    `[DELIBERATION] ${dp.id} result: ${winningOption ?? 'TIE'} (${JSON.stringify(tally)})`
  );

  return {
    id: dp.id,
    question: dp.question,
    category: dp.category as DeliberationDecisionPoint['category'],
    options,
    raised_by: dp.raisedBy,
    votes,
    vote_tally: tally,
    winning_option: winningOption,
    consensus_strength: consensusStrength,
    escalated: isTie,
  };
}

// =============================================================================
// Main Deliberation Loop
// =============================================================================

export async function runDeliberation(
  payload: DeliberatePayload
): Promise<AgentResponse<DeliberationResult>> {
  const startTime = Date.now();
  const sessionId = payload.session_id ?? randomUUID();
  const timeboxMs = (payload.timebox_minutes ?? DEFAULT_TIMEBOX_MINUTES) * 60 * 1000;
  const voteTimeoutSeconds = payload.vote_timeout_seconds ?? DEFAULT_VOTE_TIMEOUT_SECONDS;
  const committeeIds = payload.committee_ids ?? DEFAULT_COMMITTEE_IDS;
  const prdHash = createHash('sha256').update(payload.prd_content).digest('hex').slice(0, 16);

  const nats = await getNatsClient();

  const debateLog: DebateTurn[] = [];
  const resolvedDecisionPoints: DeliberationDecisionPoint[] = [];
  const pendingDecisionPoints = new Map<string, ParsedDecisionPoint>();

  let turnCount = 0;
  let deliberationStatus: DeliberationResult['status'] | undefined = undefined;
  let softWarningEmitted = false;

  // Positions tracked for each decision point
  const optimistPositions = new Map<string, string>();
  const pessimistPositions = new Map<string, string>();

  console.error(`[DELIBERATION] Session ${sessionId} started (timebox: ${payload.timebox_minutes ?? DEFAULT_TIMEBOX_MINUTES}min)`);

  // ─── Step 1: Broadcast PRD to both agents ───────────────────────────────
  const startMsg = {
    session_id: sessionId,
    prd_content: payload.prd_content,
    infrastructure_context: payload.infrastructure_context ?? '',
    timebox_minutes: payload.timebox_minutes ?? DEFAULT_TIMEBOX_MINUTES,
  };

  await Promise.all([
    nats.publish('agent.optimist.inbox', {
      type: 'deliberation_start',
      ...startMsg,
      your_role: 'optimist',
      opponent_id: 'pessimist',
    }),
    nats.publish('agent.pessimist.inbox', {
      type: 'deliberation_start',
      ...startMsg,
      your_role: 'pessimist',
      opponent_id: 'optimist',
    }),
  ]);

  // ─── Step 2: Debate loop ─────────────────────────────────────────────────
  let lastSpeaker: 'optimist' | 'pessimist' = 'pessimist'; // optimist goes first
  let lastContent = `Please begin by proposing your architectural approach for the PRD.`;

  while (Date.now() - startTime < timeboxMs) {
    const elapsed = Date.now() - startTime;
    const remaining = timeboxMs - elapsed;
    const minutesRemaining = Math.ceil(remaining / 60000);

    // Soft warning at 80% of timebox
    if (!softWarningEmitted && elapsed >= timeboxMs * SOFT_WARNING_RATIO) {
      softWarningEmitted = true;
      const warningMsg: NatsMessage = {
        type: 'timebox_warning',
        session_id: sessionId,
        minutes_remaining: minutesRemaining,
        message: `⏱ You have ${minutesRemaining} minutes remaining. Begin moving toward final positions.`,
      };
      await Promise.all([
        nats.publish('agent.optimist.inbox', warningMsg),
        nats.publish('agent.pessimist.inbox', warningMsg),
      ]);
    }

    // Alternate speakers: optimist goes first
    const nextSpeaker: 'optimist' | 'pessimist' = lastSpeaker === 'optimist' ? 'pessimist' : 'optimist';

    // Send turn to next speaker
    const turnMsg: NatsMessage = {
      type: 'debate_turn',
      session_id: sessionId,
      turn: turnCount + 1,
      from: lastSpeaker,
      content: lastContent,
      minutes_remaining: minutesRemaining,
      decision_points_resolved: resolvedDecisionPoints.map(d => d.id),
    };

    let response: NatsMessage;
    try {
      response = await nats.request(
        `agent.${nextSpeaker}.inbox`,
        turnMsg,
        AGENT_SKIP_TIMEOUT_MS
      );
    } catch {
      console.error(`[DELIBERATION] ${nextSpeaker} timed out on turn ${turnCount + 1} — ending deliberation`);
      deliberationStatus = 'timeout';
      break;
    }

    const responseContent = (response.content as string) ?? '';
    turnCount++;

    // Log the turn
    debateLog.push({
      turn: turnCount,
      speaker: nextSpeaker,
      content: responseContent,
      timestamp: new Date().toISOString(),
    });

    // Parse any decision points raised in this turn
    const newDPs = parseDecisionPoints(responseContent, nextSpeaker);
    for (const dp of newDPs) {
      console.error(`[DELIBERATION] Decision point raised: ${dp.id} by ${nextSpeaker}`);
      pendingDecisionPoints.set(dp.id, dp);

      // Track positions
      if (nextSpeaker === 'optimist') {
        optimistPositions.set(dp.id, dp.proposingOption);
      } else {
        pessimistPositions.set(dp.id, dp.proposingOption);
      }
    }

    // If we have a pending decision point with both positions, trigger committee vote
    for (const [dpId, dp] of pendingDecisionPoints.entries()) {
      const optPos = optimistPositions.get(dpId);
      const pesPos = pessimistPositions.get(dpId);

      if (optPos && pesPos) {
        pendingDecisionPoints.delete(dpId);

        // Halt debate, conduct vote
        const resolved = await conductCommitteeVote(
          nats, sessionId, dp, optPos, pesPos,
          payload.prd_content, committeeIds, voteTimeoutSeconds
        );
        resolvedDecisionPoints.push(resolved);

        // Update debate log entry with decision point reference
        const lastEntry = debateLog[debateLog.length - 1];
        if (lastEntry) {
          lastEntry.decision_point_raised = dpId;
        }

        // Broadcast vote result back to both agents
        const voteResultMsg: NatsMessage = {
          type: 'vote_result',
          session_id: sessionId,
          decision_id: dpId,
          question: dp.question,
          winning_option: resolved.winning_option ?? 'escalated',
          vote_tally: resolved.vote_tally,
          consensus_strength: resolved.consensus_strength ?? 0,
          escalated: resolved.escalated,
        };
        await Promise.all([
          nats.publish('agent.optimist.inbox', voteResultMsg),
          nats.publish('agent.pessimist.inbox', voteResultMsg),
        ]);
      }
    }

    // Check for explicit consensus signal
    if (responseContent.toLowerCase().includes('i agree with the optimist') ||
        responseContent.toLowerCase().includes('i agree with the pessimist') ||
        responseContent.toLowerCase().includes('we have consensus')) {
      deliberationStatus = 'consensus';
      break;
    }

    lastSpeaker = nextSpeaker;
    lastContent = responseContent;
  }

  // Assign final status based on how the deliberation ended
  if (!deliberationStatus) {
    // Natural timebox expiry (no explicit timeout or consensus)
    deliberationStatus = 'completed';
  }
  
  // Check if any escalated decision points should flip status to 'escalated'
  const hasEscalated = resolvedDecisionPoints.some(d => d.escalated);
  if (hasEscalated && deliberationStatus === 'completed') {
    deliberationStatus = 'escalated';
  }

  // ─── Step 3: Compile design brief ────────────────────────────────────────
  // The design brief compilation happens via the compile-brief step in the
  // lobster workflow. Here we return the raw deliberation result.
  // In standalone mode (direct binary call), we generate a simple brief.
  const designBrief = compileBasicBrief(
    payload.prd_content,
    resolvedDecisionPoints,
    debateLog,
    deliberationStatus
  );

  const result: DeliberationResult = {
    session_id: sessionId,
    prd_hash: prdHash,
    started_at: new Date(startTime).toISOString(),
    completed_at: new Date().toISOString(),
    timebox_minutes: payload.timebox_minutes ?? DEFAULT_TIMEBOX_MINUTES,
    debate_turns: turnCount,
    status: deliberationStatus,
    decision_points: resolvedDecisionPoints,
    design_brief: designBrief,
    debate_log: debateLog,
  };

  const usage: TokenUsage = { input_tokens: 0, output_tokens: 0, total_tokens: 0 };

  console.error(
    `[DELIBERATION] Session ${sessionId} complete: ${deliberationStatus}, ` +
    `${turnCount} turns, ${resolvedDecisionPoints.length} decisions, ` +
    `${Math.round((Date.now() - startTime) / 1000)}s elapsed`
  );

  return {
    success: true,
    data: result,
    usage,
    model: 'multi-agent',
    provider: 'deliberation',
  };
}

// =============================================================================
// Fallback Brief Compiler (used when not running in Lobster workflow)
// =============================================================================

function compileBasicBrief(
  _prdContent: string,
  decisions: DeliberationDecisionPoint[],
  _log: DebateTurn[],
  status: string
): string {
  const lines: string[] = ['# Design Brief', '', `_Status: ${status}_`, ''];

  if (decisions.length === 0) {
    lines.push('## No Decision Points Resolved', '', 'No explicit decision points were raised during deliberation.');
  } else {
    lines.push('## Resolved Decisions', '');
    for (const d of decisions) {
      if (d.escalated) {
        lines.push(`### [${d.id}] ${d.question} ⚠️ ESCALATED`);
        lines.push(`**Status**: Tie vote — human decision required`);
        lines.push(`**Options**: ${d.options.join(' vs ')}`);
      } else {
        const votes = Object.values(d.vote_tally).reduce((a, b) => a + b, 0);
        const count = d.vote_tally[d.winning_option ?? ''] ?? 0;
        lines.push(`### [${d.id}] ${d.question}`);
        lines.push(`**Verdict**: ${d.winning_option}`);
        lines.push(`**Consensus**: ${count}/${votes} committee members (${Math.round((d.consensus_strength ?? 0) * 100)}%)`);
      }
      lines.push('');
    }
  }

  return lines.join('\n');
}
