/**
 * activity-templates — Rich markdown templates for Linear Agent activities.
 *
 * Each function returns a markdown string suitable for the `body` field
 * of a Linear Agent activity. Markdown headers, tables, code blocks,
 * and status indicators are used to surface pipeline progress clearly.
 */

// ---------------------------------------------------------------------------
// Pipeline Start
// ---------------------------------------------------------------------------

export interface PipelineStartConfig {
  projectName: string;
  taskCount: number;
  modelPrimary?: string;
  modelFrontier?: string;
  deliberationStatus?: string;
  designStatus?: string;
}

export function pipelineStartActivity(config: PipelineStartConfig): string {
  const lines: string[] = [
    `## Pipeline Started`,
    '',
    `**Project:** ${config.projectName}`,
    `**Tasks requested:** ${config.taskCount}`,
  ];
  if (config.modelPrimary) {
    lines.push(`**Primary model:** \`${config.modelPrimary}\``);
  }
  if (config.modelFrontier) {
    lines.push(`**Frontier model:** \`${config.modelFrontier}\``);
  }
  if (config.deliberationStatus) {
    lines.push(`**Deliberation:** ${config.deliberationStatus}`);
  }
  if (config.designStatus) {
    lines.push(`**Design:** ${config.designStatus}`);
  }
  lines.push('', '---', '', 'Parsing PRD and analyzing task complexity...');
  return lines.join('\n');
}

// ---------------------------------------------------------------------------
// PRD Parsed
// ---------------------------------------------------------------------------

export function prdParsedActivity(taskCount: number): string {
  return [
    `## PRD Parsed`,
    '',
    `Extracted **${taskCount}** tasks from the PRD.`,
    '',
    'Proceeding to complexity analysis and committee review...',
  ].join('\n');
}

// ---------------------------------------------------------------------------
// Complexity Analysis
// ---------------------------------------------------------------------------

export interface ComplexityTask {
  id: number;
  title: string;
  complexity?: number | string;
  agent?: string;
  priority?: string;
}

export function complexityAnalysisActivity(tasks: ComplexityTask[]): string {
  const lines: string[] = [
    `## Complexity Analysis`,
    '',
    `Analyzed **${tasks.length}** tasks for implementation complexity.`,
    '',
    '| # | Task | Complexity | Agent | Priority |',
    '|---|------|-----------|-------|----------|',
  ];
  for (const t of tasks) {
    const complexity = t.complexity ?? '---';
    const agent = t.agent ?? '_unassigned_';
    const priority = t.priority ?? '---';
    lines.push(`| ${t.id} | ${truncate(t.title, 40)} | ${complexity} | ${agent} | ${priority} |`);
  }
  return lines.join('\n');
}

// ---------------------------------------------------------------------------
// Vote / Tally Result
// ---------------------------------------------------------------------------

export interface VoteTally {
  verdict: 'approve' | 'revise' | 'reject';
  average_scores?: Record<string, number>;
  vote_breakdown?: { approve: number; revise: number; reject: number };
  consensus_score?: number;
  suggestions?: string[];
}

export function voteResultActivity(tally: VoteTally): string {
  const icon = tally.verdict === 'approve' ? 'Approved' :
    tally.verdict === 'revise' ? 'Revision Requested' : 'Rejected';
  const lines: string[] = [
    `## Committee Vote: ${icon}`,
    '',
  ];

  if (tally.vote_breakdown) {
    const b = tally.vote_breakdown;
    lines.push(`**Votes:** ${b.approve} approve / ${b.revise} revise / ${b.reject} reject`);
  }
  if (tally.consensus_score !== undefined) {
    lines.push(`**Consensus:** ${(tally.consensus_score * 100).toFixed(0)}%`);
  }
  lines.push('');

  if (tally.average_scores) {
    lines.push('| Dimension | Score |');
    lines.push('|-----------|-------|');
    for (const [dim, score] of Object.entries(tally.average_scores)) {
      lines.push(`| ${formatDimension(dim)} | ${typeof score === 'number' ? score.toFixed(1) : score} |`);
    }
    lines.push('');
  }

  if (tally.suggestions && tally.suggestions.length > 0) {
    lines.push('**Suggestions:**');
    for (const s of tally.suggestions) {
      lines.push(`- ${s}`);
    }
  }
  return lines.join('\n');
}

// ---------------------------------------------------------------------------
// Task Refinement Result
// ---------------------------------------------------------------------------

export function taskRefinementActivity(taskCount: number, verdict: string): string {
  return [
    `## Tasks Refined`,
    '',
    `**Expanded tasks:** ${taskCount}`,
    `**Verdict:** ${verdict}`,
    '',
    'Proceeding to artifact generation...',
  ].join('\n');
}

// ---------------------------------------------------------------------------
// Docs Validation
// ---------------------------------------------------------------------------

export interface DocsValidationSummary {
  docCount: number;
  taskCount: number;
  missingAcceptance: number;
  missingDecisions: number;
  valid: boolean;
}

export function docsValidationActivity(summary: DocsValidationSummary): string {
  const status = summary.valid ? 'Passed' : 'Degraded';
  const lines: string[] = [
    `## Documentation Validation: ${status}`,
    '',
    '| Metric | Value |',
    '|--------|-------|',
    `| Documents generated | ${summary.docCount} |`,
    `| Tasks expected | ${summary.taskCount} |`,
    `| Missing acceptance criteria | ${summary.missingAcceptance} |`,
    `| Missing decision points | ${summary.missingDecisions} |`,
    `| Doc/task match | ${summary.docCount === summary.taskCount ? 'Yes' : 'No (' + summary.docCount + '/' + summary.taskCount + ')'} |`,
  ];
  return lines.join('\n');
}

// ---------------------------------------------------------------------------
// Artifact Gates
// ---------------------------------------------------------------------------

export interface ArtifactGates {
  task_count: number;
  doc_count: number;
  prompt_count: number;
  workflow_count: number;
  subtask_count?: number;
  decision_point_count?: number;
}

export function artifactGatesActivity(gates: ArtifactGates): string {
  const checkOrX = (val: number) => val > 0 ? `${val}` : `0 (MISSING)`;
  const lines: string[] = [
    `## Artifact Gates`,
    '',
    '| Artifact | Count | Status |',
    '|----------|-------|--------|',
    `| Tasks | ${gates.task_count} | ${gates.task_count > 0 ? 'Pass' : 'FAIL'} |`,
    `| Docs | ${checkOrX(gates.doc_count)} | ${gates.doc_count > 0 ? 'Pass' : 'FAIL'} |`,
    `| Prompts | ${checkOrX(gates.prompt_count)} | ${gates.prompt_count > 0 ? 'Pass' : 'FAIL'} |`,
    `| Workflows | ${checkOrX(gates.workflow_count)} | ${gates.workflow_count > 0 ? 'Pass' : 'FAIL'} |`,
  ];
  if (gates.subtask_count !== undefined) {
    lines.push(`| Subtasks | ${gates.subtask_count} | ${gates.subtask_count > 0 ? 'Pass' : 'Warn'} |`);
  }
  if (gates.decision_point_count !== undefined) {
    lines.push(`| Decision points | ${gates.decision_point_count} | ${gates.decision_point_count > 0 ? 'Pass' : 'Warn'} |`);
  }

  const allPass = gates.task_count > 0 && gates.doc_count > 0 &&
    gates.prompt_count > 0 && gates.workflow_count > 0;
  lines.push('');
  lines.push(allPass ? '**All gates passed.** Proceeding to commit and PR.' : '**Some gates failed.** Review required.');
  return lines.join('\n');
}

// ---------------------------------------------------------------------------
// Linear Sync
// ---------------------------------------------------------------------------

export interface LinearSyncResult {
  issueCount: number;
  unassignedIssueCount?: number;
  unresolvedAgents?: string[];
  issues?: Array<{
    taskId?: number;
    title?: string;
    issueId?: string;
    issueUrl?: string;
    agent?: string;
  }>;
}

export function linearSyncActivity(syncResult: LinearSyncResult, workspaceSlug?: string): string {
  const lines: string[] = [
    `## Linear Sync Complete`,
    '',
    `**Issues created:** ${syncResult.issueCount}`,
  ];
  if (syncResult.unassignedIssueCount && syncResult.unassignedIssueCount > 0) {
    lines.push(`**Unassigned:** ${syncResult.unassignedIssueCount}`);
  }
  if (syncResult.unresolvedAgents && syncResult.unresolvedAgents.length > 0) {
    lines.push(`**Unresolved agents:** ${syncResult.unresolvedAgents.join(', ')}`);
  }

  if (syncResult.issues && syncResult.issues.length > 0) {
    lines.push('');
    lines.push('| Task | Issue | Agent |');
    lines.push('|------|-------|-------|');
    for (const issue of syncResult.issues) {
      const taskCol = issue.taskId != null ? `#${issue.taskId}` : '---';
      const titleCol = issue.title ? truncate(issue.title, 35) : '---';
      // Use Linear URL format for auto-rendering @mentions when workspace slug is available
      let issueCol: string;
      if (issue.issueUrl) {
        issueCol = `[${issue.issueId ?? 'link'}](${issue.issueUrl})`;
      } else if (issue.issueId && workspaceSlug) {
        const mentionUrl = `https://linear.app/${workspaceSlug}/issue/${issue.issueId}`;
        issueCol = `[${issue.issueId}](${mentionUrl})`;
      } else {
        issueCol = issue.issueId ?? '---';
      }
      const agentCol = issue.agent ?? '_unassigned_';
      lines.push(`| ${taskCol} ${titleCol} | ${issueCol} | ${agentCol} |`);
    }
  }
  return lines.join('\n');
}

// ---------------------------------------------------------------------------
// Action Activity (generic in-progress step)
// ---------------------------------------------------------------------------

export function actionActivity(step: string, detail: string): string {
  return [
    `## ${step}`,
    '',
    detail,
  ].join('\n');
}

// ---------------------------------------------------------------------------
// PR Created
// ---------------------------------------------------------------------------

export function prCreatedActivity(prUrl: string): string {
  return [
    `## Pull Request Created`,
    '',
    prUrl !== 'none' ? `**PR:** [${prUrl}](${prUrl})` : '**PR:** _Not created (manual creation needed)_',
    '',
    'Artifacts committed and ready for agent implementation.',
  ].join('\n');
}

// ---------------------------------------------------------------------------
// Debate Position (Optimist / Pessimist turn)
// ---------------------------------------------------------------------------

export interface DecisionPointSummary {
  id: string;
  question: string;
  position: string;
  category?: string;
}

const CHARACTERS: Record<string, { emoji: string; tagline: string }> = {
  optimist: { emoji: '⚡', tagline: 'Pushing the frontier — modern, scalable, evidence-driven.' },
  pessimist: { emoji: '🛡️', tagline: 'Anchoring to proven ground — operational simplicity, named failure modes.' },
  architect: { emoji: '🏛️', tagline: 'Evaluating structural integrity and long-term maintainability.' },
  pragmatist: { emoji: '⚖️', tagline: 'Weighing practical trade-offs and real-world constraints.' },
  minimalist: { emoji: '✂️', tagline: 'Cutting to essential complexity — less is more.' },
  designer: { emoji: '🎨', tagline: 'Crafting the visual identity — form follows function.' },
};

function sentimentEmoji(confidence: number): string {
  if (confidence >= 0.8) return '💪';
  if (confidence >= 0.6) return '🤔';
  return '⚠️';
}

export function debatePositionActivity(
  speaker: 'optimist' | 'pessimist',
  body: string,
  decisionPoints?: DecisionPointSummary[],
): string {
  const label = speaker.charAt(0).toUpperCase() + speaker.slice(1);
  const char = CHARACTERS[speaker] ?? { emoji: '💬', tagline: '' };
  const lines: string[] = [
    `## ${char.emoji} ${label}`,
    '',
    `> *${char.tagline}*`,
    '',
    '---',
    '',
  ];

  if (decisionPoints && decisionPoints.length > 0) {
    lines.push('| Decision | Category | Position |');
    lines.push('|----------|----------|----------|');
    for (const dp of decisionPoints) {
      lines.push(`| \`${dp.id}\` ${truncate(dp.question, 40)} | ${dp.category ?? '---'} | ${truncate(dp.position, 60)} |`);
    }
    lines.push('');
  }

  lines.push(truncateBody(body, 3000));
  return lines.join('\n');
}

// ---------------------------------------------------------------------------
// Voter Ballot (individual committee voter result)
// ---------------------------------------------------------------------------

export interface VoterBallotConfig {
  voterId: string;
  modelName: string;
  chosenOption: string;
  reasoning: string;
  confidence?: number;
  concerns?: string[];
}

export function voterBallotActivity(config: VoterBallotConfig): string {
  const char = CHARACTERS[config.voterId.toLowerCase()] ?? { emoji: '🗳️', tagline: '' };
  const label = config.voterId.charAt(0).toUpperCase() + config.voterId.slice(1);
  const confidence = config.confidence ?? 0.6;
  const sEmoji = sentimentEmoji(confidence);
  const lines: string[] = [
    `## ${char.emoji} ${label}`,
    '',
  ];
  if (char.tagline) {
    lines.push(`> *${char.tagline}*`);
    lines.push('');
  }
  lines.push(
    `**Vote:** ${config.chosenOption} ${sEmoji}`,
    `**Confidence:** ${(confidence * 100).toFixed(0)}% ${sEmoji}`,
    `**Model:** \`${config.modelName}\``,
    '',
    '---',
    '',
    truncateBody(config.reasoning, 800),
  );
  if (config.concerns && config.concerns.length > 0) {
    lines.push('', '**Concerns:**');
    for (const c of config.concerns) {
      lines.push(`- ⚠️ ${c}`);
    }
  }
  return lines.join('\n');
}

// ---------------------------------------------------------------------------
// Tally Result (committee vote summary before elicitation)
// ---------------------------------------------------------------------------

export interface DecisionTallyConfig {
  decisionId: string;
  question: string;
  tally: Record<string, number>;
  totalVoters: number;
  winningOption: string;
  consensusStrength?: number;
  escalated?: boolean;
  voterNotes?: string[];
}

export function tallyResultActivity(config: DecisionTallyConfig): string {
  const lines: string[] = [
    `## 🗳️ Decision Vote: \`${config.decisionId}\``,
    '',
    `**Question:** ${config.question}`,
    '',
    '| Option | Votes | |',
    '|--------|-------|-|',
  ];
  for (const [option, count] of Object.entries(config.tally)) {
    const winner = option === config.winningOption;
    const indicator = winner ? '✅' : '';
    lines.push(`| ${option} | ${count}/${config.totalVoters} | ${indicator} |`);
  }
  lines.push('');
  if (config.consensusStrength !== undefined) {
    lines.push(`**Consensus:** ${(config.consensusStrength * 100).toFixed(0)}%`);
  }
  if (config.escalated) {
    lines.push('**Status:** Escalated to human review');
  } else {
    lines.push(`**Recommendation:** ${config.winningOption}`);
  }
  if (config.voterNotes && config.voterNotes.length > 0) {
    lines.push('', '**Voter Notes:**');
    for (const n of config.voterNotes) {
      lines.push(`- ${n}`);
    }
  }
  return lines.join('\n');
}

// ---------------------------------------------------------------------------
// Deliberation Started
// ---------------------------------------------------------------------------

export function deliberationStartActivity(dpCount: number, sessionId: string): string {
  return [
    `## 🧠 Deliberation Started`,
    '',
    `**Session:** \`${sessionId}\``,
    `**Decision points:** ${dpCount}`,
    '',
    '**The committee is assembling:**',
    '⚡ **Optimist** — advocates modern, scalable approaches',
    '🛡️ **Pessimist** — anchors to proven, operational simplicity',
    '🏛️ **Architect** — evaluates structural integrity',
    '⚖️ **Pragmatist** — weighs practical trade-offs',
    '✂️ **Minimalist** — cuts to essential complexity',
    '',
    '---',
    '',
    '*Debate is beginning...*',
  ].join('\n');
}

// ---------------------------------------------------------------------------
// Deliberation Complete
// ---------------------------------------------------------------------------

export interface DeliberationCompleteConfig {
  resolvedCount: number;
  totalDps: number;
  elapsedMinutes: number;
  debateTurns: number;
  decisions?: Array<{ id: string; winner: string }>;
}

export function deliberationCompleteActivity(config: DeliberationCompleteConfig): string {
  const lines: string[] = [
    `## 🏁 Deliberation Complete`,
    '',
    '| Metric | Value |',
    '|--------|-------|',
    `| Status | Complete |`,
    `| Resolved | ${config.resolvedCount} of ${config.totalDps} |`,
    `| Turns | ${config.debateTurns} |`,
    `| Duration | ${config.elapsedMinutes} min |`,
  ];
  if (config.decisions && config.decisions.length > 0) {
    lines.push('', '### Decisions', '');
    for (const d of config.decisions) {
      lines.push(`✅ **\`${d.id}\`**: ${truncate(d.winner, 80)}`);
    }
  }
  lines.push('', '*Compiling design brief from resolved decisions...*');
  return lines.join('\n');
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function truncate(str: string, maxLen: number): string {
  if (!str) return '---';
  return str.length > maxLen ? str.slice(0, maxLen - 1) + '...' : str;
}

function truncateBody(str: string, maxLen: number): string {
  if (!str) return '';
  return str.length > maxLen ? str.slice(0, maxLen - 3) + '...' : str;
}

function formatDimension(dim: string): string {
  return dim.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase());
}
