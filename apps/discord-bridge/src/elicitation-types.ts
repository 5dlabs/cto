/**
 * Elicitation Protocol Types
 *
 * Platform-agnostic types for human-in-the-loop decisions.
 * Consumed by both the Linear bridge and the Discord bridge via HTTP POST.
 */

// =============================================================================
// ElicitationRequest — sent by workflow steps to bridges via HTTP POST
// =============================================================================

export interface ElicitationOption {
  /** Machine-readable value */
  value: string;
  /** Human-readable label */
  label: string;
  /** Which side advocated this option */
  advocated_by?: 'optimist' | 'pessimist';
  /** Committee votes for this option */
  vote_count?: number;
  /** Brief rationale */
  description?: string;
}

export interface VoteSummary {
  total_voters: number;
  tally: Record<string, number>;
  consensus_strength: number;
  escalated: boolean;
  voter_notes?: Array<{ voter_id: string; chose: string; reasoning: string }>;
}

export interface ElicitationRequest {
  /** Unique elicitation ID, e.g. "{sessionId}-{decisionPointId}" */
  elicitation_id: string;
  /** Deliberation session */
  session_id: string;
  /** Decision point ID */
  decision_id: string;
  /** The decision being asked */
  question: string;
  /** Decision category */
  category: string;
  /** Available options */
  options: ElicitationOption[];
  /** Committee winner (pre-selected default) */
  recommended_option?: string;
  /** Committee vote breakdown */
  vote_summary: VoteSummary;
  /** Whether re-deliberation button should be shown */
  allow_redeliberation: boolean;
  /** Seconds before auto-selecting recommended (0 = no timeout) */
  timeout_seconds: number;
  /** true = read-only display (full_auto), false = interactive */
  informational: boolean;
  timestamp: string;
  /** Target Linear issue for this elicitation */
  linear_issue_id?: string;
  /** Target Discord channel for this elicitation */
  discord_channel_id?: string;
  /** Lobster resume token for workflow continuation */
  resume_token?: string;
  /** Optional metadata (run_id, agent_id, etc.) */
  metadata?: Record<string, string>;
}

// =============================================================================
// ElicitationResponse — published by bridge when human responds
// =============================================================================

export interface ElicitationResponse {
  elicitation_id: string;
  source: 'linear' | 'discord';
  response_type: 'selection' | 'redeliberate';
  /** For selection — which option was chosen */
  selected_option?: string;
  /** For redeliberation — additional context/instructions from the human */
  user_context?: string;
  user_id: string;
  timestamp: string;
}

// =============================================================================
// ElicitationCancel — cross-platform "already answered"
// =============================================================================

export interface ElicitationCancel {
  elicitation_id: string;
  answered_by: 'linear' | 'discord';
  selected_option?: string;
  timestamp: string;
}

// =============================================================================
// Helpers
// =============================================================================

export function createElicitationRequest(
  sessionId: string,
  decisionId: string,
  question: string,
  category: string,
  options: ElicitationOption[],
  voteSummary: VoteSummary,
  opts: {
    recommendedOption?: string;
    allowRedeliberation?: boolean;
    timeoutSeconds?: number;
    informational?: boolean;
    linearIssueId?: string;
    discordChannelId?: string;
  } = {},
): ElicitationRequest {
  return {
    elicitation_id: `${sessionId}-${decisionId}`,
    session_id: sessionId,
    decision_id: decisionId,
    question,
    category,
    options,
    recommended_option: opts.recommendedOption,
    vote_summary: voteSummary,
    allow_redeliberation: opts.allowRedeliberation ?? true,
    timeout_seconds: opts.timeoutSeconds ?? 0,
    informational: opts.informational ?? false,
    timestamp: new Date().toISOString(),
    linear_issue_id: opts.linearIssueId,
    discord_channel_id: opts.discordChannelId,
  };
}

export function createElicitationResponse(
  elicitationId: string,
  source: 'linear' | 'discord',
  userId: string,
  opts: { selectedOption?: string; userContext?: string },
): ElicitationResponse {
  return {
    elicitation_id: elicitationId,
    source,
    response_type: opts.userContext ? 'redeliberate' : 'selection',
    selected_option: opts.selectedOption,
    user_context: opts.userContext,
    user_id: userId,
    timestamp: new Date().toISOString(),
  };
}

export function createElicitationCancel(
  elicitationId: string,
  answeredBy: 'linear' | 'discord',
  selectedOption?: string,
): ElicitationCancel {
  return {
    elicitation_id: elicitationId,
    answered_by: answeredBy,
    selected_option: selectedOption,
    timestamp: new Date().toISOString(),
  };
}
