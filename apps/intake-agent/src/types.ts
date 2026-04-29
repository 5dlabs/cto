/**
 * Type definitions for the intake-agent JSON protocol.
 * Remaining operations: ping, prd_research, design_intake, design_variants.
 * All LLM-based operations are handled by Lobster llm-task steps.
 * Deliberation-related interfaces below are retained as shared workflow types.
 */

// =============================================================================
// Request Types
// =============================================================================

/**
 * Operation types supported by the agent.
 * All LLM-based operations are now handled by Lobster llm-task steps.
 * Deliberation is now a full Lobster workflow (deliberation.lobster.yaml).
 */
export type Operation =
  | 'ping'
  | 'prd_research'
  | 'design_intake'
  | 'design_variants'
  | 'generate_deliberation_video';

/**
 * Base request structure for all operations.
 */
export interface AgentRequest<T = unknown> {
  /** Operation to perform */
  operation: Operation;
  /** Operation-specific payload */
  payload: T;
}

// =============================================================================
// Response Types (TypeScript → Rust)
// =============================================================================

/**
 * Token usage information.
 */
export interface TokenUsage {
  /** Number of input tokens */
  input_tokens: number;
  /** Number of output tokens */
  output_tokens: number;
  /** Total tokens (input + output) */
  total_tokens: number;
}

/**
 * Success response structure.
 */
export interface AgentSuccessResponse<T> {
  success: true;
  /** Operation result data */
  data: T;
  /** Token usage information */
  usage: TokenUsage;
  /** Model that generated the response */
  model: string;
  /** Provider name */
  provider: string;
}

/**
 * Error types for categorization.
 */
export type ErrorType = 'api_error' | 'parse_error' | 'mcp_error' | 'validation_error' | 'unknown';

/**
 * Error response structure.
 */
export interface AgentErrorResponse {
  success: false;
  /** Error message */
  error: string;
  /** Error category */
  error_type: ErrorType;
  /** Additional error details (optional) */
  details?: string;
}

/**
 * Union type for all responses.
 */
export type AgentResponse<T> = AgentSuccessResponse<T> | AgentErrorResponse;

/**
 * Ping response for health check.
 */
export interface PingData {
  status: 'ok';
  version: string;
}

// =============================================================================
// Type Guards
// =============================================================================

/**
 * Check if a response is successful.
 */
export function isSuccessResponse<T>(
  response: AgentResponse<T>
): response is AgentSuccessResponse<T> {
  return response.success === true;
}

/**
 * Check if a response is an error.
 */
export function isErrorResponse<T>(
  response: AgentResponse<T>
): response is AgentErrorResponse {
  return response.success === false;
}

/**
 * Validate request has required fields.
 */
export function validateRequest(request: unknown): request is AgentRequest {
  if (typeof request !== 'object' || request === null) {
    return false;
  }
  const req = request as Record<string, unknown>;
  return (
    typeof req['operation'] === 'string' &&
    ['ping', 'prd_research', 'design_intake', 'design_variants', 'generate_deliberation_video'].includes(req['operation'] as string)
  );
}

// =============================================================================
// Deliberation Types
// =============================================================================

/**
 * Research memos produced by the prd_research operation.
 */
export interface ResearchMemos {
  /** Evidence for what's proven and working in the ecosystem */
  optimist: string;
  /** Known failure modes and operational risks */
  pessimist: string;
}

/**
 * Payload shape used by the deliberation workflow.
 */
export type HumanReviewMode = 'full_auto' | 'semi_auto' | 'manual';

export interface DeliberatePayload {
  /** Unique session identifier (UUID) */
  session_id: string;
  /** Full PRD markdown content */
  prd_content: string;
  /** Maximum deliberation time in minutes */
  timebox_minutes?: number;
  /** Available cluster operators/services (injected context) */
  infrastructure_context?: string;
  /** Committee member agent IDs */
  committee_ids?: string[];
  /** Seconds to wait for each committee vote before marking abstain */
  vote_timeout_seconds?: number;
  /** Pre-debate research memos from Tavily (optimist = best practices, pessimist = failure modes) */
  research_memos?: ResearchMemos | AgentSuccessResponse<ResearchMemos>;
  /**
   * Human review mode for committee vote decisions:
   * - full_auto: use committee winner immediately, publish informational-only to bridges
   * - semi_auto: publish interactive elicitation, auto-select after 10s if no response (default)
   * - manual: publish interactive elicitation, block until human responds
   */
  human_review_mode?: HumanReviewMode;
  /** Target Linear issue ID for elicitation rendering */
  linear_issue_id?: string;
  /** Target Discord channel ID for elicitation rendering */
  discord_channel_id?: string;
}

/**
 * A single turn in the debate log.
 */
export interface DebateTurn {
  turn: number;
  speaker: 'optimist' | 'pessimist';
  content: string;
  decision_point_raised?: string | string[];
  timestamp: string;
}

/**
 * A committee member's vote on a decision point.
 */
export interface CommitteeVote {
  voter_id: string;
  chosen_option: string;
  confidence: number;
  reasoning: string;
  concerns: string[];
  timestamp?: string;
}

/**
 * A single decision point raised during deliberation.
 * Renamed to avoid collision with the task-level DecisionPoint type.
 */
export interface DeliberationDecisionPoint {
  id: string;
  question: string;
  category:
    | 'architecture'
    | 'error-handling'
    | 'data-model'
    | 'api-design'
    | 'ux-behavior'
    | 'visual-identity'
    | 'design-system'
    | 'component-library'
    | 'layout-pattern'
    | 'performance'
    | 'security'
    | 'technology-choice'
    | 'infrastructure';
  options: string[];
  raised_by: 'optimist' | 'pessimist' | 'intake';
  votes: CommitteeVote[];
  vote_tally: Record<string, number>;
  winning_option?: string;
  consensus_strength?: number;
  escalated: boolean;
}

/**
 * Full output of a deliberation session.
 */
export interface DeliberationResult {
  session_id: string;
  prd_hash?: string;
  started_at: string;
  completed_at?: string;
  timebox_minutes: number;
  debate_turns: number;
  status: 'completed' | 'timeout' | 'consensus' | 'escalated';
  decision_points: DeliberationDecisionPoint[];
  design_brief: string;
  debate_log: DebateTurn[];
}
