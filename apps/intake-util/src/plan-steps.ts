/**
 * plan-steps — Canonical pipeline plan step definitions for Linear Agent Plan API.
 *
 * Each step has a stable `id` used to track position in the pipeline, and a
 * human-readable `content` string shown in the Linear session checklist.
 */

export interface PipelineStep {
  id: string;
  content: string;
}

export const PIPELINE_STEPS: PipelineStep[] = [
  { id: 'parse-prd', content: 'Parse PRD and extract requirements' },
  { id: 'analyze-complexity', content: 'Analyze task complexity and recommend subtask counts' },
  { id: 'review-tasks', content: 'Committee vote on task decomposition' },
  { id: 'refine-tasks', content: 'Refine tasks based on committee feedback' },
  { id: 'generate-artifacts', content: 'Generate docs, prompts, scaffolds, and workflows' },
  { id: 'quality-gate', content: 'Run quality gate on generated artifacts' },
  { id: 'commit-and-pr', content: 'Commit outputs and create pull request' },
  { id: 'sync-linear', content: 'Sync tasks to Linear as issues' },
];

export const DELIBERATION_STEPS: PipelineStep[] = [
  { id: 'delib-research', content: 'Research PRD and gather evidence for debate' },
  { id: 'delib-optimist', content: 'Optimist presents position on decision points' },
  { id: 'delib-pessimist', content: 'Pessimist responds with counter-arguments' },
  { id: 'delib-vote', content: 'Committee votes on each decision point' },
  { id: 'delib-human-review', content: 'Human reviews and confirms decisions' },
  { id: 'delib-compile', content: 'Compile design brief from resolved decisions' },
];

export type PlanStatus = 'pending' | 'inProgress' | 'completed' | 'canceled';

/**
 * Build a full plan array with appropriate statuses based on which step is
 * currently active.  Steps before `currentStepId` are marked completed,
 * the current step is inProgress, and everything after is pending.
 *
 * If `currentStepId` is not found, all steps are returned as pending.
 * If `currentStepId` is the special value `'__all_completed__'`, every step
 * is marked completed.
 *
 * Automatically selects the right step list based on the step id prefix:
 * `delib-*` uses DELIBERATION_STEPS, everything else uses PIPELINE_STEPS.
 */
export function buildPlan(
  currentStepId: string,
): Array<{ content: string; status: PlanStatus }> {
  const steps = currentStepId.startsWith('delib-')
    ? DELIBERATION_STEPS
    : PIPELINE_STEPS;

  if (currentStepId === '__all_completed__') {
    return steps.map((s) => ({ content: s.content, status: 'completed' as PlanStatus }));
  }

  const idx = steps.findIndex((s) => s.id === currentStepId);

  return steps.map((step, i) => {
    let status: PlanStatus;
    if (idx < 0) {
      status = 'pending';
    } else if (i < idx) {
      status = 'completed';
    } else if (i === idx) {
      status = 'inProgress';
    } else {
      status = 'pending';
    }
    return { content: step.content, status };
  });
}
