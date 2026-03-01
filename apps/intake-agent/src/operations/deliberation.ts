/**
 * Deliberation Operation
 *
 * Orchestrates a time-boxed design debate between the Optimist and Pessimist
 * agents via NATS, with a 5-member multi-model committee voting on decision
 * points raised during the debate.
 *
 * Flow:
 *   1. Broadcast PRD to both debate agents via deliberation.room
 *   2. Conduct debate loop (relay turns via room, enforce timebox)
 *   3. On DECISION_POINT: publish vote_request to room, collect responses, tally, broadcast result
 *   4. On timeout or consensus: compile design brief, return DeliberationResult
 *
 * Architecture: one shared NATS subject (deliberation.room) — all 7 agents
 * subscribe and respond there. Messages carry a `to` field indicating the
 * intended recipient(s); agents ignore messages not addressed to them.
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

const DELIBERATION_ROOM = 'deliberation.room';
const DELIBERATION_INBOX = 'agent.deliberation.inbox';
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

    return {
      async publish(subject: string, message: NatsMessage): Promise<void> {
        nc.publish(subject, sc.encode(JSON.stringify(message)));
      },
      subscribe(subject: string, handler: (msg: NatsMessage) => void) {
        const sub = nc.subscribe(subject);
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
        };
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
  };
}

// =============================================================================
// AgentMessage Helpers
// =============================================================================

/**
 * Wire format matching the nats-messenger plugin's AgentMessage type.
 * All messages published to agent inboxes must use this format so the
 * receiving agent's nats-messenger plugin can parse and present them.
 */
interface AgentMessage {
  from: string;
  to?: string;
  subject: string;
  message: string;
  priority: 'normal' | 'urgent';
  timestamp: string;
  replyTo?: string;
  type?: 'message';
}

/**
 * Publish a structured NatsMessage to a specific agent's inbox using
 * the AgentMessage wire format expected by the nats-messenger plugin.
 * The structured content is JSON-serialized into the  field.
 * The  is set to DELIBERATION_INBOX so agents can reply back.
 */
async function publishToAgent(
  nats: NatsClient,
  agentId: string,
  natsMessage: NatsMessage
): Promise<void> {
  const subject = `agent.${agentId}.inbox`;
  const agentMsg: AgentMessage = {
    from: 'intake',
    to: agentId,
    subject,
    message: JSON.stringify(natsMessage),
    priority: 'normal',
    timestamp: new Date().toISOString(),
    replyTo: DELIBERATION_INBOX,
    type: 'message',
  };
  // Cast to NatsMessage for the underlying publish call since we control
  // the NATS connection in getNatsClient() and accept any JSON object.
  await nats.publish(subject, agentMsg as unknown as NatsMessage);
}

// =============================================================================
// Room Helpers
// =============================================================================

/**
 * Subscribe to the room and resolve when the expected agent posts a response.
 * Rejects with an Error on timeout.
 */
async function waitForResponse(
  nats: NatsClient,
  _roomSubject: string,
  sessionId: string,
  expectedFrom: string,
  timeoutMs: number,
  expectedType?: string
): Promise<NatsMessage> {
  return new Promise((resolve, reject) => {
    let cleanup: (() => void) | undefined;

    const timer = setTimeout(() => {
      cleanup?.();
      reject(new Error(`Timeout waiting for ${expectedFrom}`));
    }, timeoutMs);

    // Subscribe to DELIBERATION_INBOX — agents reply here via the replyTo field
    // set in publishToAgent(). This implements the publish+subscribe reply pattern.
    cleanup = nats.subscribe(DELIBERATION_INBOX, (msg) => {
      const from = (msg.from as string) ?? '';
      if (from !== expectedFrom) return;

      // Agents send AgentMessage format; structured deliberation data is
      // JSON-encoded in the `message` field. Try to parse it.
      let structured: NatsMessage = msg;
      const rawMessage = msg.message as string | undefined;
      if (rawMessage) {
        try {
          const parsed = JSON.parse(rawMessage) as NatsMessage;
          structured = { ...parsed, from, session_id: sessionId };
        } catch {
          // Agent sent plain text — wrap as debate_response
          structured = { type: 'debate_response', session_id: sessionId, from, content: rawMessage };
        }
      }

      // Filter by message type if specified to prevent late responses from
      // previous phases being captured (e.g., late research_findings during debate)
      if (expectedType && structured.type !== expectedType) return;

      clearTimeout(timer);
      cleanup?.();
      resolve(structured);
    });
  });
}

/**
 * Subscribe to the room and collect vote_response messages from all voterIds.
 * Returns when all votes are received or on timeout (abstains fill missing votes).
 */
async function collectVotes(
  nats: NatsClient,
  _roomSubject: string,
  sessionId: string,
  voterIds: string[],
  timeoutMs: number
): Promise<CommitteeVote[]> {
  const votes: CommitteeVote[] = [];
  const remaining = new Set(voterIds);

  return new Promise((resolve) => {
    let cleanup: (() => void) | undefined;

    const timer = setTimeout(() => {
      cleanup?.();
      // Fill abstains for non-responders
      for (const voterId of remaining) {
        votes.push({
          voter_id: voterId,
          chosen_option: 'abstain',
          confidence: 0,
          reasoning: 'Timeout',
          concerns: [],
          timestamp: new Date().toISOString(),
        });
      }
      resolve(votes);
    }, timeoutMs);

    // Subscribe to DELIBERATION_INBOX — committee agents reply here
    cleanup = nats.subscribe(DELIBERATION_INBOX, (msg) => {
      const from = (msg.from as string) ?? '';
      if (!remaining.has(from)) return;

      // Parse structured vote data from AgentMessage.message field
      let structured: NatsMessage = msg;
      const rawMessage = msg.message as string | undefined;
      if (rawMessage) {
        try {
          const parsed = JSON.parse(rawMessage) as NatsMessage;
          structured = { ...parsed, from, session_id: sessionId };
        } catch {
          structured = { type: 'vote_response', session_id: sessionId, from };
        }
      }

      if (structured.session_id !== sessionId && msg.session_id !== sessionId) return;

      remaining.delete(from);
      votes.push({
        voter_id: from,
        chosen_option: (structured.chosen_option as string) ?? 'abstain',
        confidence: (structured.confidence as number) ?? 0,
        reasoning: (structured.reasoning as string) ?? '',
        concerns: (structured.concerns as string[]) ?? [],
        timestamp: new Date().toISOString(),
      });
      if (remaining.size === 0) {
        clearTimeout(timer);
        cleanup?.();
        resolve(votes);
      }
    });
  });
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

  // Publish ONE vote request to the room addressed to all committee members
  // Fan out vote_request to each committee member's inbox using AgentMessage format
  const votePayload = {
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
  await Promise.all(committeeIds.map(id => publishToAgent(nats, id, { ...votePayload, to: id })));

  // Collect votes from room — wait for vote_response messages
  const votes = await collectVotes(
    nats,
    DELIBERATION_ROOM,
    sessionId,
    committeeIds,
    voteTimeoutSeconds * 1000
  );

  // Normalize invalid vote options to abstain
  const normalizedVotes: CommitteeVote[] = votes.map(vote => {
    const validOptions = [...options, 'abstain'];
    if (!validOptions.includes(vote.chosen_option)) {
      return {
        ...vote,
        chosen_option: 'abstain',
        reasoning: `Invalid vote option "${vote.chosen_option}" provided - counted as abstain. ${vote.reasoning}`,
      };
    }
    return vote;
  });

  // Tally votes (abstains are excluded from majority calculation)
  const tally: Record<string, number> = {};
  for (const vote of normalizedVotes) {
    if (vote.chosen_option !== 'abstain') {
      tally[vote.chosen_option] = (tally[vote.chosen_option] ?? 0) + 1;
    }
  }

  const nonAbstainVotes = normalizedVotes.filter(v => v.chosen_option !== 'abstain');
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
    votes: normalizedVotes,
    vote_tally: tally,
    winning_option: winningOption,
    consensus_strength: consensusStrength,
    escalated: isTie,
  };
}

// =============================================================================
// Research Phase
// =============================================================================

const DEFAULT_RESEARCH_TIMEOUT_MINUTES = 5;

interface ResearchFindings {
  optimist: string;
  pessimist: string;
}

/**
 * Pre-debate research phase.
 *
 * Sends each debate agent a `research_request` message containing the PRD.
 * Agents are expected to use Tavily (web search) and Firecrawl (deep crawl)
 * to gather relevant information before the debate begins.
 * Both agents are queried concurrently; findings are collected and returned
 * as context that will be prepended to the first debate turn.
 */
async function runResearchPhase(
  nats: NatsClient,
  sessionId: string,
  prdContent: string,
  researchTimeoutMinutes: number
): Promise<ResearchFindings> {
  const timeoutMs = researchTimeoutMinutes * 60 * 1000;
  console.error(`[DELIBERATION] Research phase started (timeout: ${researchTimeoutMinutes}min)`);

  const researchPayload = {
    type: 'research_request',
    session_id: sessionId,
    prd_content: prdContent,
    instructions: [
      'Use Tavily to search for relevant best practices, benchmarks, and recent developments related to the technology choices in this PRD.',
      'Use Firecrawl to deep-crawl any URLs referenced in the PRD or highly relevant documentation pages you find.',
      'Compile your findings into a structured summary covering: technology landscape, known trade-offs, and relevant real-world examples.',
      'Reply with a research_findings message containing your compiled findings as plain text in the `content` field.',
    ].join('\n'),
    timeout_minutes: researchTimeoutMinutes,
  };

  // Send research requests to both agents concurrently
  await Promise.all(['optimist', 'pessimist'].map(id =>
    publishToAgent(nats, id, { ...researchPayload, to: id })
  ));

  // Collect responses concurrently
  const [optimistResult, pessimistResult] = await Promise.allSettled([
    waitForResponse(nats, DELIBERATION_ROOM, sessionId, 'optimist', timeoutMs, 'research_findings'),
    waitForResponse(nats, DELIBERATION_ROOM, sessionId, 'pessimist', timeoutMs, 'research_findings'),
  ]);

  const optimistFindings = optimistResult.status === 'fulfilled'
    ? ((optimistResult.value.content as string) ?? '[No research findings]')
    : `[Research timeout after ${researchTimeoutMinutes} minutes]`;

  const pessimistFindings = pessimistResult.status === 'fulfilled'
    ? ((pessimistResult.value.content as string) ?? '[No research findings]')
    : `[Research timeout after ${researchTimeoutMinutes} minutes]`;

  console.error(`[DELIBERATION] Research phase complete — optimist: ${optimistFindings.length} chars, pessimist: ${pessimistFindings.length} chars`);

  return { optimist: optimistFindings, pessimist: pessimistFindings };
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

  // ─── Step 0: Research phase (optional) ────────────────────────────────────
  let researchContext = '';
  if (payload.research_enabled) {
    const researchTimeoutMinutes = payload.research_timeout_minutes ?? DEFAULT_RESEARCH_TIMEOUT_MINUTES;
    const findings = await runResearchPhase(
      nats, sessionId, payload.prd_content, researchTimeoutMinutes
    );
    researchContext = [
      '## Pre-Debate Research Findings',
      '',
      '### Optimist Research',
      findings.optimist,
      '',
      '### Pessimist Research',
      findings.pessimist,
    ].join('\n');
    console.error('[DELIBERATION] Research context compiled — feeding into debate');
  }

  // Reset startTime after research to ensure debate gets full timebox
  const debateStartTime = Date.now();

  // ─── Step 1: PRD broadcast ───────────────────────────────────────────────
  // Publish one deliberation_start message to the room — both optimist and
  // pessimist receive it since they both subscribe to deliberation.room.
  // Send deliberation_start to each debate agent's inbox using AgentMessage format
  const startPayload = {
    type: 'deliberation_start',
    session_id: sessionId,
    prd_content: payload.prd_content,
    infrastructure_context: payload.infrastructure_context ?? '',
    timebox_minutes: payload.timebox_minutes ?? DEFAULT_TIMEBOX_MINUTES,
    research_context: researchContext,
  };
  await Promise.all(['optimist', 'pessimist'].map(id =>
    publishToAgent(nats, id, { ...startPayload, to: id })
  ));

  // ─── Step 2: Debate loop ─────────────────────────────────────────────────
  let lastSpeaker: 'optimist' | 'pessimist' = 'pessimist'; // optimist goes first
  let lastContent = `Please begin by proposing your architectural approach for the PRD.`;

  while (Date.now() - debateStartTime < timeboxMs) {
    const elapsed = Date.now() - debateStartTime;
    const remaining = timeboxMs - elapsed;
    const minutesRemaining = Math.ceil(remaining / 60000);

    // Soft warning at 80% of timebox
    if (!softWarningEmitted && elapsed >= timeboxMs * SOFT_WARNING_RATIO) {
      softWarningEmitted = true;
      try {
        const warningPayload = {
          type: 'timebox_warning',
          session_id: sessionId,
          minutes_remaining: minutesRemaining,
          message: `⏱ ${minutesRemaining} minutes remaining. Begin moving toward final positions.`,
        };
        await Promise.all(['optimist', 'pessimist'].map(id =>
          publishToAgent(nats, id, { ...warningPayload, to: id })
        ));
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

    // Send turn to room — only the named agent should respond
    // Send debate turn directly to the agent's inbox using AgentMessage format
    await publishToAgent(nats, nextSpeaker, {
      type: 'debate_turn',
      session_id: sessionId,
      to: nextSpeaker,
      turn: turnCount + 1,
      from: lastSpeaker,
      content: lastContent,
      minutes_remaining: minutesRemaining,
      decision_points_resolved: resolvedDecisionPoints.map(d => d.id),
    });

    // Wait for response from room (agent publishes back to deliberation.room)
    let response: NatsMessage;
    try {
      response = await waitForResponse(
        nats,
        DELIBERATION_ROOM,
        sessionId,
        nextSpeaker,
        AGENT_SKIP_TIMEOUT_MS,
        'debate_response'
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
        const currentRemaining = timeboxMs - (Date.now() - debateStartTime);
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

        // Announce vote result to the room
        try {
          const resultPayload = {
            type: 'vote_result',
            session_id: sessionId,
            decision_id: dpId,
            question: dp.question,
            winning_option: resolved.winning_option ?? 'escalated',
            vote_tally: resolved.vote_tally,
            consensus_strength: resolved.consensus_strength ?? 0,
            escalated: resolved.escalated,
          };
          await Promise.all(['optimist', 'pessimist'].map(id =>
            publishToAgent(nats, id, { ...resultPayload, to: id })
          ));
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
