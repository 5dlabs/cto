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
  type DebateRole,
  type RoleProviderConfig,
} from './debate-planning';

// New: Adversarial Three-Agent Model (Advocate-Adversary-Arbiter)
export {
  runAdversarialDebate,
  DEFAULT_ADVERSARIAL_CONFIG,
  AGENT_MANIFESTS,
  type AdversarialDebateConfig,
  type AdversarialDebateResult,
  type OpeningArgument,
  type Rebuttal,
  type DebateExchange,
  type ArbiterDecision,
} from './adversarial-debate';
