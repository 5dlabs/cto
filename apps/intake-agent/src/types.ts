/**
 * Type definitions for the intake-agent JSON protocol.
 * These types mirror the Rust structs in crates/intake/src/ai/
 */

// =============================================================================
// Request Types (Rust → TypeScript)
// =============================================================================

/**
 * Operation types supported by the agent.
 */
export type Operation = 
  | 'parse_prd' 
  | 'expand_task' 
  | 'analyze_complexity' 
  | 'generate' 
  | 'research'
  | 'research_capabilities'
  | 'generate_with_critic'
  | 'validate_content'
  | 'provider_status'
  | 'deliberate'
  | 'ping';

/**
 * Generation options matching Rust GenerateOptions.
 */
export interface GenerateOptions {
  /** Temperature for sampling (0.0 to 1.0) */
  temperature?: number;
  /** Maximum tokens to generate */
  max_tokens?: number;
  /** Stop sequences */
  stop_sequences?: string[];
  /** Whether to request JSON output */
  json_mode?: boolean;
  /** Enable MCP tools for research */
  mcp_enabled?: boolean;
  /** Path to MCP config file */
  mcp_config?: string;
  /** Force disable extended thinking */
  force_disable_thinking?: boolean;
  /** Budget in tokens for extended thinking */
  thinking_budget?: number;
}

/**
 * Base request structure for all operations.
 */
export interface AgentRequest<T = unknown> {
  /** Operation to perform */
  operation: Operation;
  /** Model to use (e.g., "claude-sonnet-4-20250514") */
  model?: string;
  /** Generation options */
  options?: GenerateOptions;
  /** Operation-specific payload */
  payload: T;
}

// -----------------------------------------------------------------------------
// Operation-specific payloads
// -----------------------------------------------------------------------------

/**
 * Payload for parse_prd operation.
 */
export interface ParsePrdPayload {
  /** Content of the PRD file */
  prd_content: string;
  /** Path to the PRD file (for context) */
  prd_path: string;
  /** Target number of tasks to generate (0 = auto) */
  num_tasks?: number;
  /** Starting ID for tasks */
  next_id?: number;
  /** Enable research mode with MCP tools */
  research?: boolean;
  /** Default priority for tasks */
  default_task_priority?: string;
  /** Project root path */
  project_root?: string;
}

/**
 * Payload for expand_task operation.
 */
export interface ExpandTaskPayload {
  /** The task to expand */
  task: TaskSummary;
  /** Target number of subtasks */
  subtask_count?: number;
  /** Starting subtask ID */
  next_subtask_id?: number;
  /** Enable research mode */
  use_research?: boolean;
  /** Additional context for expansion */
  additional_context?: string;
  /** Complexity-based expansion prompt */
  expansion_prompt?: string;
  /** Reasoning from complexity analysis */
  complexity_reasoning_context?: string;
  /** Enable subagent-aware expansion */
  enable_subagents?: boolean;
  /** Project root path */
  project_root?: string;
}

/**
 * Summary of a task for expand_task context.
 */
export interface TaskSummary {
  id: string;
  title: string;
  description: string;
  details?: string;
  test_strategy?: string;
  status: string;
  dependencies: string[];
}

/**
 * Payload for analyze_complexity operation.
 */
export interface AnalyzeComplexityPayload {
  /** Tasks to analyze */
  tasks: TaskSummary[];
  /** Complexity threshold for subtask recommendation */
  threshold?: number;
  /** Enable research mode */
  use_research?: boolean;
  /** Project root path */
  project_root?: string;
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

// -----------------------------------------------------------------------------
// Operation-specific response data
// -----------------------------------------------------------------------------

/**
 * Task priority levels.
 */
export type TaskPriority = 'high' | 'medium' | 'low';

/**
 * Task status values.
 */
export type TaskStatus = 'pending' | 'in_progress' | 'done' | 'blocked' | 'deferred';

/**
 * Decision point for captured discovery.
 */
export interface DecisionPoint {
  id: string;
  category: 'architecture' | 'error-handling' | 'data-model' | 'api-design' | 'ux-behavior' | 'performance' | 'security';
  description: string;
  options: string[];
  requires_approval: boolean;
  constraint_type: 'hard' | 'soft' | 'open' | 'escalation';
}

/**
 * Generated task from parse_prd.
 */
export interface GeneratedTask {
  id: number;
  title: string;
  description: string;
  status?: TaskStatus;
  dependencies: number[];
  priority?: TaskPriority;
  details?: string;
  test_strategy?: string;
  subtasks?: GeneratedSubtask[];
  decision_points?: DecisionPoint[];
}

/**
 * Subagent type for subagent-aware expansion.
 */
export type SubagentType = 'implementer' | 'reviewer' | 'tester' | 'researcher' | 'documenter';

/**
 * Generated subtask from expand_task.
 */
export interface GeneratedSubtask {
  id: number;
  title: string;
  description: string;
  status?: TaskStatus;
  dependencies: number[];
  details?: string;
  test_strategy?: string;
  subagent_type?: SubagentType;
  parallelizable?: boolean;
}

/**
 * Response data for parse_prd operation.
 */
export interface ParsePrdData {
  tasks: GeneratedTask[];
}

/**
 * Response data for expand_task operation.
 */
export interface ExpandTaskData {
  subtasks: GeneratedSubtask[];
}

/**
 * Task complexity analysis result.
 */
export interface TaskComplexityAnalysis {
  task_id: number;
  task_title: string;
  complexity_score: number;
  recommended_subtasks: number;
  expansion_prompt: string;
  reasoning: string;
}

/**
 * Response data for analyze_complexity operation.
 */
export interface AnalyzeComplexityData {
  complexity_analysis: TaskComplexityAnalysis[];
}

/**
 * Ping response for health check.
 */
export interface PingData {
  status: 'ok';
  version: string;
  sdk_version: string;
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
    [
      'parse_prd',
      'parse_prd_iterative',
      'expand_task', 
      'analyze_complexity', 
      'generate',
      'generate_prompts',
      'research',
      'research_capabilities',
      'generate_with_critic',
      'generate_with_debate',
      'validate_content',
      'generate_docs',
      'provider_status',
      'deliberate',
      'ping',
    ].includes(req['operation'] as string)
  );
}

// =============================================================================
// Deliberation Types
// =============================================================================

/**
 * Payload for the deliberate operation.
 */
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
}

/**
 * A single turn in the debate log.
 */
export interface DebateTurn {
  turn: number;
  speaker: 'optimist' | 'pessimist';
  content: string;
  decision_point_raised?: string;
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
 */
export interface DecisionPoint {
  id: string;
  question: string;
  category: 'architecture' | 'error-handling' | 'data-model' | 'api-design' | 'ux-behavior' | 'performance' | 'security' | 'technology-choice' | 'infrastructure';
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
  decision_points: DecisionPoint[];
  design_brief: string;
  debate_log: DebateTurn[];
}
