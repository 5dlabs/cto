/**
 * Vote tallying logic for the 5-model voting committee.
 * Takes VoteBallot[] and computes majority decision.
 */

import type { VoteBallot, VoteBallotScores, TallyResult } from './types';

const SCORE_DIMENSIONS: (keyof VoteBallotScores)[] = [
  'task_decomposition',
  'dependency_ordering',
  'decision_point_coverage',
  'test_strategy_quality',
  'agent_assignment',
];

/**
 * Tally votes from multiple model ballots.
 */
export function tallyVotes(ballots: VoteBallot[]): TallyResult {
  if (ballots.length === 0) {
    return {
      verdict: 'revise',
      average_scores: {
        task_decomposition: 0,
        dependency_ordering: 0,
        decision_point_coverage: 0,
        test_strategy_quality: 0,
        agent_assignment: 0,
        overall: 0,
      },
      vote_breakdown: { approve: 0, revise: 0, reject: 0 },
      suggestions: [],
      consensus_score: 0,
    };
  }

  // Count verdicts
  const breakdown = { approve: 0, revise: 0, reject: 0 };
  for (const ballot of ballots) {
    breakdown[ballot.verdict]++;
  }

  // Majority verdict (>50%), default to 'revise' if no majority
  let verdict: 'approve' | 'revise' | 'reject' = 'revise';
  const majority = ballots.length / 2;
  if (breakdown.approve > majority) verdict = 'approve';
  else if (breakdown.reject > majority) verdict = 'reject';
  else if (breakdown.revise > majority) verdict = 'revise';
  else {
    // No clear majority — pick the highest count, tie-break: revise > reject > approve
    const max = Math.max(breakdown.approve, breakdown.revise, breakdown.reject);
    if (breakdown.revise >= max) verdict = 'revise';
    else if (breakdown.reject >= max) verdict = 'reject';
    else verdict = 'approve';
  }

  // Average scores per dimension
  const avgScores: Record<string, number> = {};
  for (const dim of SCORE_DIMENSIONS) {
    const sum = ballots.reduce((acc, b) => acc + b.scores[dim], 0);
    avgScores[dim] = Math.round((sum / ballots.length) * 100) / 100;
  }

  // Average overall score
  const overallSum = ballots.reduce((acc, b) => acc + b.overall_score, 0);
  const overallAvg = Math.round((overallSum / ballots.length) * 100) / 100;

  // Deduplicated suggestions (normalize: trim + lowercase for comparison)
  const seen = new Set<string>();
  const suggestions: string[] = [];
  for (const ballot of ballots) {
    for (const s of ballot.suggestions) {
      const normalized = s.trim().toLowerCase();
      if (normalized && !seen.has(normalized)) {
        seen.add(normalized);
        suggestions.push(s.trim());
      }
    }
  }

  // Consensus score: fraction agreeing with majority verdict
  const consensusScore = Math.round((breakdown[verdict] / ballots.length) * 100) / 100;

  return {
    verdict,
    average_scores: {
      task_decomposition: avgScores['task_decomposition'],
      dependency_ordering: avgScores['dependency_ordering'],
      decision_point_coverage: avgScores['decision_point_coverage'],
      test_strategy_quality: avgScores['test_strategy_quality'],
      agent_assignment: avgScores['agent_assignment'],
      overall: overallAvg,
    },
    vote_breakdown: breakdown,
    suggestions,
    consensus_score: consensusScore,
  };
}
