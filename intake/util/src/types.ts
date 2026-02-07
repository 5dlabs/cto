/**
 * Type definitions for intake-util CLI.
 * Snake_case-only subset of intake-agent types.
 */

// =============================================================================
// Enums / Union Types
// =============================================================================

export type TaskPriority = 'high' | 'medium' | 'low';

export type TaskStatus = 'pending' | 'in_progress' | 'done' | 'blocked' | 'deferred';

export type SubagentType = 'implementer' | 'reviewer' | 'tester' | 'researcher' | 'documenter';

// =============================================================================
// Core Types
// =============================================================================

export interface DecisionPoint {
  id: string;
  category: 'architecture' | 'error-handling' | 'data-model' | 'api-design' | 'ux-behavior' | 'performance' | 'security';
  description: string;
  options: string[];
  requires_approval: boolean;
  constraint_type: 'hard' | 'soft' | 'open' | 'escalation';
  // compat: source code uses camelCase fallback
  requiresApproval?: boolean;
  constraintType?: string;
}

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
  // compat: source code uses camelCase fallback
  testStrategy?: string;
  subagentType?: string;
}

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
  // compat: source code uses camelCase fallback
  testStrategy?: string;
  decisionPoints?: DecisionPoint[];
}

// =============================================================================
// Response Types (needed by generate-prompts.ts verbatim signature)
// =============================================================================

export type ErrorType = 'api_error' | 'parse_error' | 'mcp_error' | 'validation_error' | 'file_error' | 'unknown';

export interface AgentSuccessResponse<T> {
  success: true;
  data: T;
  usage: { input_tokens: number; output_tokens: number; total_tokens: number };
  model: string;
  provider: string;
}

export interface AgentErrorResponse {
  success: false;
  error: string;
  error_type: ErrorType;
  details?: string;
}

export type AgentResponse<T> = AgentSuccessResponse<T> | AgentErrorResponse;

// =============================================================================
// Vote Tallying Types (matches vote-ballot.schema.json)
// =============================================================================

export interface VoteBallotScores {
  task_decomposition: number;
  dependency_ordering: number;
  decision_point_coverage: number;
  test_strategy_quality: number;
  agent_assignment: number;
}

export interface VoteBallot {
  voter_id: string;
  scores: VoteBallotScores;
  overall_score: number;
  verdict: 'approve' | 'revise' | 'reject';
  reasoning: string;
  suggestions: string[];
}

export interface TallyResult {
  verdict: 'approve' | 'revise' | 'reject';
  average_scores: VoteBallotScores & { overall: number };
  vote_breakdown: { approve: number; revise: number; reject: number };
  suggestions: string[];
  consensus_score: number;
}
