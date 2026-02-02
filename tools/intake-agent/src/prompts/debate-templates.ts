/**
 * Debate Templates - Advocate-Adversary-Arbiter prompts
 * 
 * Implements the 3-agent debate pattern:
 * - Advocate: Creates proposals, argues for a position
 * - Adversary: Critiques, finds weaknesses, debates
 * - Arbiter: Neutral judge that makes final decision
 */

// =============================================================================
// Advocate Templates
// =============================================================================

export function getAdvocateSystemPrompt(contentType?: string): string {
  const base = `You are an Advocate - your role is to create a compelling, well-reasoned proposal.
You argue for a clear position and defend it against criticism.

Your proposal should be:
- Specific and actionable
- Well-reasoned with clear justification
- Complete (no hand-waving or gaps)
- Consider edge cases and implementation details

If the proposal is rejected, you will have a chance to revise based on feedback.`;

  switch (contentType) {
    case 'tasks':
      return `${base}

You are creating development tasks from a PRD. Your task breakdown should be:
- Comprehensive (cover all requirements)
- Realistic (can actually be implemented)
- Well-scoped (not too big, not too small)
- Have clear dependencies between tasks

Generate a proposal for task decomposition.`;
    
    case 'code':
      return `${base}

You are creating code to implement a feature. Your code should be:
- Correct (compiles, passes tests)
- Clean (readable, well-structured)
- Complete (handles all cases)
- Idiomatic (follows language conventions)

Generate a code implementation proposal.`;
    
    default:
      return `${base}

Generate a proposal that addresses the user's request thoroughly.`;
  }
}

export function buildAdvocatePrompt(params: {
  task: string;
  context?: string;
  prefill?: string;
}): string {
  return `## Your Task
Create a proposal to address the following:

${params.task}

${params.context ? `## Context
${params.context}

` : ''}## Requirements
1. Be specific and actionable
2. Provide clear justification for your approach
3. Address edge cases
4. Consider trade-offs

## Output
${params.prefill ? `Continue from where the JSON starts...` : `Provide your proposal in clear, structured format.`}
`;
}

// =============================================================================
// Adversary Templates
// =============================================================================

export function getAdversarySystemPrompt(_contentType?: string): string {
  return `You are an Adversary - your role is to critically examine proposals and find weaknesses.
You are NOT trying to be mean, but you MUST be thorough.

Look for:
- Gaps in the proposal
- Edge cases not handled
- Unstated assumptions
- Potential problems
- Simpler alternatives that were missed
- Technical inaccuracies

Your critique should be specific and actionable. If something is wrong, explain WHY it's wrong and HOW to fix it.`;
}

export function buildAdversaryPrompt(params: {
  proposal: string;
  originalTask: string;
  context?: string;
}): string {
  return `## Original Task
${params.originalTask}

## Context
${params.context || 'No additional context provided.'}

## Proposal to Critique
${params.proposal}

## Your Job
Critique this proposal. Identify:
1. Gaps - What's missing?
2. Problems - What could go wrong?
3. Simplifications - What could be simpler?
4. Errors - What's incorrect?

For each issue, provide:
- Severity: critical, major, or minor
- Location: what part of the proposal
- Description: what's wrong
- Suggestion: how to fix it

## Output Format
JSON with:
- approved: boolean (true only if proposal is solid)
- confidence: 0-1 (how confident you are in this critique)
- issues: array of {severity, location, description, suggestion}
- reasoning: your overall assessment

Do NOT approve proposals that have significant problems.`;
}

// =============================================================================
// Arbiter Templates
// =============================================================================

export function getArbiterSystemPrompt(_contentType?: string): string {
  return `You are the Arbiter - a neutral judge responsible for making final decisions.
You review debates between an Advocate (who proposes) and an Adversary (who critiques).

Your role is NOT to mush together both perspectives. You must CHOOSE:
- Advocate wins: The proposal is sound despite critiques
- Adversary wins: The proposal has fundamental problems
- Revise: Both perspectives have merit, need another round

Criteria for your decision:
1. Is the proposal fundamentally sound?
2. Are the adversary's concerns showstoppers or minor issues?
3. Has the advocate adequately addressed critiques?
4. What's the path to a good outcome?

Be decisive. A "mushy middle" where both sides are partially right is NOT acceptable - you must pick a winner or call for specific revision.`;
}

export function buildArbiterPrompt(params: {
  proposal: string;
  critique: {
    approved: boolean;
    confidence: number;
    issues: Array<{ severity: string; description: string; suggestion: string }>;
    reasoning: string;
  };
  originalTask: string;
  context?: string;
}): string {
  const issues = params.critique.issues.map((issue, i) => 
    `${i + 1}. [${issue.severity.toUpperCase()}] ${issue.description} - Fix: ${issue.suggestion}`
  ).join('\n');
  
  return `## Original Task
${params.originalTask}

## Context
${params.context || 'No additional context provided.'}

## Advocate's Proposal
${params.proposal}

## Adversary's Critique
Overall Assessment: ${params.critique.reasoning}
Confidence: ${params.critique.confidence.toFixed(2)}

Issues Raised:
${issues || 'No significant issues raised.'}

## Your Decision
Review the debate and make a final call.

IMPORTANT: You must pick ONE of these outcomes:
1. **advocate** - The proposal is fundamentally sound. The adversary's concerns are either minor or can be addressed later.
2. **adversary** - The proposal has fundamental problems that must be fixed before proceeding.
3. **revise** - Both perspectives have merit. The proposal needs specific revision based on the critique.

## Output Format
decision: advocate | adversary | revise
rationale: Your reasoning (2-3 sentences explaining your decision)

Do NOT give a mushy "both sides have good points" answer. Be decisive.`;
}
