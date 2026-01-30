/**
 * Orchestration module - Multi-model collaboration patterns.
 */

export {
  generateWithCritic,
  runCritic,
  runRefiner,
  validateOnly,
  type GenerateWithCriticOptions,
  type MultiModelDialog,
} from './critic-validator';

export {
  generatePlanWithDebate,
  DEFAULT_DEBATE_CONFIG,
  type DebatePlanningConfig,
  type DebatePlanningResult,
  type AgentProposal,
  type DebateRound,
  type ConsensusResult,
  type ResearchFindings,
  type DebateDepth,
  type DomainFocus,
} from './debate-planning';
