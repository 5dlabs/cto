/**
 * tally-decision-votes — Tally committee votes on a single decision point.
 *
 * Input (stdin): Array of { voter_id, chosen_option, confidence, reasoning, concerns[] }
 * Output: { winning_option, consensus_strength, tally, escalated, votes }
 *
 * Distinct from the task-quality `tally` subcommand — this tallies
 * decision-point votes (chosen_option) not task verdicts.
 */

export interface DecisionVote {
  voter_id: string;
  chosen_option: string;
  confidence: number;
  reasoning: string;
  concerns: string[];
}

export interface DecisionTallyResult {
  winning_option: string | null;
  consensus_strength: number;
  tally: Record<string, number>;
  escalated: boolean;
  total_voters: number;
  votes: DecisionVote[];
  voter_notes: Array<{ voter_id: string; chose: string; reasoning: string }>;
}

export function tallyDecisionVotes(votes: DecisionVote[]): DecisionTallyResult {
  // Normalize invalid options to abstain
  const normalized = votes.map((v) => {
    if (!v.chosen_option || v.chosen_option === 'abstain') {
      return { ...v, chosen_option: 'abstain' };
    }
    return v;
  });

  // Build tally (exclude abstains)
  const tally: Record<string, number> = {};
  for (const vote of normalized) {
    if (vote.chosen_option !== 'abstain') {
      tally[vote.chosen_option] = (tally[vote.chosen_option] ?? 0) + 1;
    }
  }

  const nonAbstain = normalized.filter((v) => v.chosen_option !== 'abstain');
  const maxVotes = Math.max(...Object.values(tally), 0);
  const winners = Object.entries(tally).filter(([, count]) => count === maxVotes);
  const isTie = winners.length > 1 || nonAbstain.length === 0;
  const winningOption = isTie ? null : winners[0]?.[0] ?? null;
  const consensusStrength =
    nonAbstain.length > 0 ? Math.round((maxVotes / nonAbstain.length) * 100) / 100 : 0;

  const voterNotes = nonAbstain.map((v) => ({
    voter_id: v.voter_id,
    chose: v.chosen_option,
    reasoning: v.reasoning,
  }));

  return {
    winning_option: winningOption,
    consensus_strength: consensusStrength,
    tally,
    escalated: isTie,
    total_voters: votes.length,
    votes: normalized,
    voter_notes: voterNotes.length > 0 ? voterNotes : [],
  };
}
