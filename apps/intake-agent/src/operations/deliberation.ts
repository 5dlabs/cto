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
import { connect as natsConnect, StringCodec, type NatsConnection } from 'nats';
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
 * Falls back to a direct NATS connection via NATS_URL env var (for CodeRun pods).
 * Final fallback is a stub for local testing.
 */
async function getNatsClient(): Promise<NatsClient> {
  // OpenClaw injects a global NATS client when the agent is running in-pod.
  // When running in standalone binary mode (testing), we fall back to a stub.
  const globalNats = (globalThis as Record<string, unknown>).__openclaw_nats__;
  if (globalNats) {
    return globalNats as NatsClient;
  }

  // Direct NATS connection via NATS_URL env var.
  // Used when running as a CodeRun pod on the same cluster as the debate agents.
  const natsUrl = process.env['NATS_URL'];
  if (natsUrl) {
    console.error(`[DELIBERATION] Connecting to NATS at ${natsUrl}`);
    const nc: NatsConnection = await natsConnect({ servers: natsUrl });
    const sc = StringCodec();

    const subscriptions = new Map<string, ReturnType<NatsConnection['subscribe']>>();

    return {
      async publish(subject: string, message: NatsMessage): Promise<void> {
        nc.publish(subject, sc.encode(JSON.stringify(message)));
      },
      subscribe(subject: string, handler: (msg: NatsMessage) => void) {
        const sub = nc.subscribe(subject);
        subscriptions.set(subject, sub);
        // Process messages asynchronously
        (async () => {
          for await (const m of sub) {
            try {
              const parsed = JSON.parse(sc.decode(m.data)) as NatsMessage;
              handler(parsed);
            } catch (e) {
              console.error(`[NATS] Failed to parse message on ${subject}:`, e);
            }
          }
        })().catch(console.error);
        return () => {
          sub.unsubscribe();
          subscriptions.delete(subject);
        };
      },
      async request(subject: string, message: NatsMessage, timeoutMs: number): Promise<NatsMessage> {
        // Convert NatsMessage to AgentMessage format expected by nats-messenger plugin.
        // Subscribe to agent.deliberation.inbox for the response.
        const replySubject = 'agent.deliberation.inbox';
        const agentMsg = {
          from: 'deliberation',
          to: subject.replace('agent.', '').replace('.inbox', ''),
          subject,
          message: `[DEBATE TURN ${(message as Record<string, unknown>)['turn'] ?? ''}]\n\n${(message as Record<string, unknown>)['content'] ?? ''}\n\nSession: ${message.session_id}. Minutes remaining: ${(message as Record<string, unknown>)['minutes_remaining'] ?? '?'}. Please respond with your full debate position. End with: nats(action="publish", to="deliberation", message="<your full response>")`,
          priority: 'normal',
          timestamp: new Date().toISOString(),
          type: 'message',
          replyTo: replySubject,
        };
        return new Promise((resolve, reject) => {
          const timer = setTimeout(() => {
            sub.unsubscribe();
            reject(new Error(`Timeout waiting for reply from ${subject}`));
          }, timeoutMs);
          const sub = nc.subscribe(replySubject);
          (async () => {
            for await (const m of sub) {
              try {
                const data = JSON.parse(sc.decode(m.data)) as Record<string, unknown>;
                if (data['type'] === 'discovery_ping' || data['type'] === 'discovery_pong') continue;
                clearTimeout(timer);
                sub.unsubscribe();
                const reply: NatsMessage = {
                  type: 'debate_turn_reply',
                  session_id: message.session_id,
                  content: (data['message'] as string) ?? '',
                };
                resolve(reply);
                break;
              } catch (e) {
                console.error('[NATS] Failed to parse reply:', e);
              }
            }
          })().catch(reject);
          nc.publish(subject, sc.encode(JSON.stringify(agentMsg)));
        });
      },
    };
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

const VALID_CATEGORIES = [
  'architecture',
  'error-handling',
  'data-model',
  'api-design',
  'ux-behavior',
  'performance',
  'security',
  'technology-choice',
  'infrastructure',
] as const;

function isValidCategory(category: string): category is DeliberationDecisionPoint['category'] {
  return VALID_CATEGORIES.includes(category as DeliberationDecisionPoint['category']);
}

interface ParsedDecisionPoint {
  id: string;
  category: DeliberationDecisionPoint['category'];
  question: string;
  proposingOption: string;
  reasoning: string;
  raisedBy: 'optimist' | 'pessimist';
}

function parseDecisionPoints(content: string, speaker: 'optimist' | 'pessimist'): ParsedDecisionPoint[] {
  const points: ParsedDecisionPoint[] = [];
  // Fresh regex each call — no shared lastIndex state
  const blockRegex = /DECISION_POINT:\s*\n([\s\S]+?)(?=\n\nDECISION_POINT:|\n\n(?!\s)|$)/g;
  let blockMatch: RegExpExecArray | null;

  while ((blockMatch = blockRegex.exec(content)) !== null) {
    const block = blockMatch[1] ?? '';
    const get = (field: string) => {
      const m = new RegExp(`^${field}:\\s*(.+?)\\s*$`, 'm').exec(block);
      return m?.[1]?.trim() ?? '';
    };
    const id = get('id').replace(/[.,;:!?]+$/, '');
    const category = get('category').replace(/[.,;:!?]+$/, '');
    const question = get('question');
    const my_option = get('my_option');
    const reasoning = block.replace(/^(?:id|category|question|my_option):[^\n]*\n?/gm, '').trim()
      || get('reasoning');

    if (!id || !category || !question || !my_option) continue;

    if (!isValidCategory(category)) {
      console.error(`[DELIBERATION] Invalid category "${category}" — skipping decision point ${id}`);
      continue;
    }
    points.push({
      id,
      category,
      question,
      proposingOption: my_option,
      reasoning,
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

      const chosenOption = (response.chosen_option as string) ?? 'abstain';
      const validOptions = [...options, 'abstain'];
      const isValidOption = validOptions.includes(chosenOption);
      
      return {
        voter_id: voterId,
        chosen_option: isValidOption ? chosenOption : 'abstain',
        confidence: (response.confidence as number) ?? 0,
        reasoning: isValidOption 
          ? (response.reasoning as string) ?? 'No reasoning provided'
          : `Invalid vote option "${chosenOption}" provided - counted as abstain. ${(response.reasoning as string) ?? ''}`,
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
    category: dp.category,
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
  let timeoutCount = 0;
  let deliberationStatus: DeliberationResult['status'] | undefined = undefined;
  let softWarningEmitted = false;

  // Positions tracked for each decision point
  const optimistPositions = new Map<string, string>();
  const pessimistPositions = new Map<string, string>();

  console.error(`[DELIBERATION] Session ${sessionId} started (timebox: ${payload.timebox_minutes ?? DEFAULT_TIMEBOX_MINUTES}min)`);

  // ─── Step 1: PRD broadcast ───────────────────────────────────────────────
  // NOTE: The broadcast-prd step in deliberation.lobster.yaml already sends
  // deliberation_start messages to both agents. We do NOT duplicate that here.
  // When running standalone (no Lobster workflow), the stub NATS client logs
  // messages but doesn't require initialization broadcasts.

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
      try {
        await Promise.all([
          nats.publish('agent.optimist.inbox', warningMsg),
          nats.publish('agent.pessimist.inbox', warningMsg),
        ]);
      } catch (err) {
        console.error('[DELIBERATION] Failed to send timebox warning:', err);
      }
    }

    // Check if there's enough time remaining for an agent turn
    // Require at least AGENT_SKIP_TIMEOUT_MS remaining to start a new turn
    if (remaining < AGENT_SKIP_TIMEOUT_MS) {
      console.error('[DELIBERATION] Insufficient time remaining for next turn — stopping debate');
      break;
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
      console.error(`[DELIBERATION] ${nextSpeaker} timed out on turn ${turnCount + 1} — skipping turn and continuing`);
      timeoutCount++;
      // Skip this turn and continue with a synthetic "no response" message
      lastSpeaker = nextSpeaker;
      lastContent = `[Agent ${nextSpeaker} did not respond within the timeout period]`;
      continue;
    }

    const responseContent = (response.content as string) ?? '';
    turnCount++;

    // Parse decision points raised in THIS turn before logging the turn,
    // so we can annotate the log entry with what was actually raised here.
    // (Resolution happens later and is tracked separately.)
    const newDPs = parseDecisionPoints(responseContent, nextSpeaker);
    const raisedThisTurn = newDPs.map(dp => dp.id);

    // Log the turn — annotate with IDs raised in this turn's content
    debateLog.push({
      turn: turnCount,
      speaker: nextSpeaker,
      content: responseContent,
      timestamp: new Date().toISOString(),
      ...(raisedThisTurn.length > 0 && {
        decision_point_raised: raisedThisTurn.length === 1
          ? raisedThisTurn[0]
          : raisedThisTurn,
      }),
    });

    // Register newly raised decision points and track positions
    for (const dp of newDPs) {
      console.error(`[DELIBERATION] Decision point raised: ${dp.id} by ${nextSpeaker}`);
      pendingDecisionPoints.set(dp.id, dp);

      if (nextSpeaker === 'optimist') {
        optimistPositions.set(dp.id, dp.proposingOption);
      } else {
        pessimistPositions.set(dp.id, dp.proposingOption);
      }
    }

    // If we have a pending decision point with both positions, trigger committee vote.
    // Resolution is tracked in resolvedDecisionPoints — NOT mixed into decision_point_raised.
    for (const [dpId, dp] of pendingDecisionPoints.entries()) {
      const optPos = optimistPositions.get(dpId);
      const pesPos = pessimistPositions.get(dpId);

      if (optPos && pesPos) {
        // Check if there's enough time remaining for a committee vote
        const currentRemaining = timeboxMs - (Date.now() - startTime);
        const voteTimeoutMs = voteTimeoutSeconds * 1000;
        if (currentRemaining < voteTimeoutMs) {
          console.error(`[DELIBERATION] Insufficient time remaining for committee vote on ${dpId} — deferring`);
          continue;
        }

        pendingDecisionPoints.delete(dpId);
        optimistPositions.delete(dpId);
        pessimistPositions.delete(dpId);

        // Halt debate, conduct vote
        const resolved = await conductCommitteeVote(
          nats, sessionId, dp, optPos, pesPos,
          payload.prd_content, committeeIds, voteTimeoutSeconds
        );
        resolvedDecisionPoints.push(resolved);

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
        try {
          await Promise.all([
            nats.publish('agent.optimist.inbox', voteResultMsg),
            nats.publish('agent.pessimist.inbox', voteResultMsg),
          ]);
        } catch (err) {
          console.error(`[DELIBERATION] Failed to broadcast vote result for ${dpId}:`, err);
        }
      }
    }

    // Check for explicit consensus signal (require phrase at sentence boundaries)
    const contentLower = responseContent.toLowerCase();
    const consensusPattern = /(?:^|[.!?,]\s+)(?:i\s+(?:fully\s+)?(?:agree|concede)|you'?re\s+right|we\s+have\s+consensus|i\s+withdraw\s+my\s+objection)(?:[,.\s]|$)/;
    if (consensusPattern.test(contentLower)) {
      deliberationStatus = 'consensus';
      break;
    }

    lastSpeaker = nextSpeaker;
    lastContent = responseContent;
  }

  // Process any remaining pending decision points that never got both positions
  for (const [dpId, dp] of pendingDecisionPoints.entries()) {
    console.error(`[DELIBERATION] Decision point ${dpId} never received both positions — marking as unresolved/escalated`);
    const optPos = optimistPositions.get(dpId);
    const pesPos = pessimistPositions.get(dpId);
    
    // Create a decision point record with whatever positions we have
    const unresolvedDP: DeliberationDecisionPoint = {
      id: dp.id,
      question: dp.question,
      category: dp.category,
      options: [optPos ?? '[AGENT DID NOT PROVIDE POSITION]', pesPos ?? '[AGENT DID NOT PROVIDE POSITION]'],
      raised_by: dp.raisedBy,
      votes: [],
      vote_tally: {},
      winning_option: undefined,
      consensus_strength: 0,
      escalated: true,
    };
    resolvedDecisionPoints.push(unresolvedDP);
  }

  // Assign final status based on how the deliberation ended
  if (!deliberationStatus) {
    // Check if agents were largely unresponsive (majority of turns timed out)
    // If more than half of the turns timed out, mark as 'timeout' instead of 'completed'
    if (turnCount > 0 && timeoutCount > turnCount / 2) {
      deliberationStatus = 'timeout';
    } else {
      // Natural timebox expiry with responsive agents
      deliberationStatus = 'completed';
    }
  }
  
  // Check if any escalated decision points should flip status to 'escalated'
  // Only override if the session completed normally (not timeout or consensus)
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
