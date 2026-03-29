/**
 * validate.ts — Deterministic validation for merged fan-out output.
 *
 * Checks completeness (all expected task_ids present) and structural
 * integrity (required fields non-empty).
 */

export interface ValidationResult {
  valid: boolean;
  errors: string[];
}

interface DocItem {
  task_id?: number;
  task_md?: string;
  decisions_md?: string;
  acceptance_md?: string;
}

interface PromptSubtask {
  subtask_id?: number;
  prompt_md?: string;
}

interface PromptItem {
  task_id?: number;
  prompt_md?: string;
  prompt_xml?: string;
  subtasks?: PromptSubtask[];
}

export function validateDocs(merged: unknown[], taskIds: number[]): ValidationResult {
  const errors: string[] = [];
  const docs = merged as DocItem[];

  // Completeness: all expected task_ids present
  const foundIds = new Set(docs.map((d) => d.task_id));
  for (const id of taskIds) {
    if (!foundIds.has(id)) {
      errors.push(`Missing docs for task_id ${id}`);
    }
  }

  // Structural integrity
  for (const doc of docs) {
    if (doc.task_id == null) {
      errors.push('Doc entry missing task_id');
      continue;
    }
    if (!doc.task_md?.trim()) {
      errors.push(`Task ${doc.task_id}: task_md is empty`);
    }
    if (!doc.decisions_md?.trim()) {
      errors.push(`Task ${doc.task_id}: decisions_md is empty`);
    }
    if (!doc.acceptance_md?.trim()) {
      errors.push(`Task ${doc.task_id}: acceptance_md is empty`);
    }
  }

  return { valid: errors.length === 0, errors };
}

interface WorkflowItem {
  task_id?: number;
  workflow_yaml?: string;
}

export function validateWorkflows(merged: unknown[], taskIds: number[]): ValidationResult {
  const errors: string[] = [];
  const workflows = merged as WorkflowItem[];

  // Completeness
  const foundIds = new Set(workflows.map((w) => w.task_id));
  for (const id of taskIds) {
    if (!foundIds.has(id)) {
      errors.push(`Missing workflow for task_id ${id}`);
    }
  }

  // Structural integrity
  for (const wf of workflows) {
    if (wf.task_id == null) {
      errors.push('Workflow entry missing task_id');
      continue;
    }
    if (!wf.workflow_yaml?.trim()) {
      errors.push(`Task ${wf.task_id}: workflow_yaml is empty`);
      continue;
    }

    // Check for expected structure markers
    const yaml = wf.workflow_yaml;
    if (!yaml.includes('name:')) {
      errors.push(`Task ${wf.task_id}: workflow_yaml missing 'name:' field`);
    }
    if (!yaml.includes('steps:')) {
      errors.push(`Task ${wf.task_id}: workflow_yaml missing 'steps:' section`);
    }
    if (!yaml.includes('setup')) {
      errors.push(`Task ${wf.task_id}: workflow_yaml missing 'setup' step`);
    }
    if (!yaml.includes('create-pr')) {
      errors.push(`Task ${wf.task_id}: workflow_yaml missing 'create-pr' step`);
    }
  }

  return { valid: errors.length === 0, errors };
}

// =============================================================================
// Generic Validation (WS-4 toggleable verify steps)
// =============================================================================

/**
 * Generic validator for pipeline step outputs.
 * Supports all types needed by verify-* steps in workflows.
 */
export function validateGeneric(type: string, input: string, strict: boolean = false): ValidationResult {
  const errors: string[] = [];

  if (!input.trim()) {
    return { valid: false, errors: ['Input is empty'] };
  }

  switch (type) {
    case 'generated-task':
    case 'tasks': {
      let parsed: unknown;
      try { parsed = JSON.parse(input); } catch { return { valid: false, errors: ['Invalid JSON'] }; }
      if (!Array.isArray(parsed)) return { valid: false, errors: ['Expected JSON array of tasks'] };
      for (const t of parsed as Array<Record<string, unknown>>) {
        if (!t.id) errors.push(`Task missing 'id'`);
        if (!t.title) errors.push(`Task ${t.id ?? '?'} missing 'title'`);
        if (!t.description) errors.push(`Task ${t.id ?? '?'} missing 'description'`);
        if (!Array.isArray(t.dependencies)) errors.push(`Task ${t.id ?? '?'} missing 'dependencies' array`);
      }
      break;
    }

    case 'complexity-analysis':
    case 'complexity': {
      let parsed: unknown;
      try { parsed = JSON.parse(input); } catch { return { valid: false, errors: ['Invalid JSON'] }; }
      const obj = parsed as Record<string, unknown>;
      if (!obj.overall_complexity && !obj.complexity) errors.push('Missing complexity field');
      break;
    }

    case 'expanded-tasks': {
      let parsed: unknown;
      try { parsed = JSON.parse(input); } catch { return { valid: false, errors: ['Invalid JSON'] }; }
      if (!Array.isArray(parsed)) return { valid: false, errors: ['Expected JSON array'] };
      for (const t of parsed as Array<Record<string, unknown>>) {
        if (!t.id) errors.push(`Task missing 'id'`);
        const subtasks = t.subtasks;
        if (!Array.isArray(subtasks) || subtasks.length === 0) {
          if (strict) errors.push(`Task ${t.id ?? '?'}: no subtasks`);
          continue;
        }
        for (const [index, raw] of subtasks.entries()) {
          if (!raw || typeof raw !== 'object' || Array.isArray(raw)) {
            errors.push(`Task ${t.id ?? '?'}: subtask ${index + 1} is not an object`);
            continue;
          }
          const st = raw as Record<string, unknown>;
          if (!st.id && st.id !== 0) errors.push(`Task ${t.id ?? '?'}: subtask ${index + 1} missing 'id'`);
          if (!st.title) errors.push(`Task ${t.id ?? '?'}: subtask ${st.id ?? index + 1} missing 'title'`);
          if (!st.description) errors.push(`Task ${t.id ?? '?'}: subtask ${st.id ?? index + 1} missing 'description'`);
          if (!Array.isArray(st.dependencies)) {
            errors.push(`Task ${t.id ?? '?'}: subtask ${st.id ?? index + 1} missing 'dependencies' array`);
          }
        }
      }
      break;
    }

    case 'scaffold':
    case 'scaffolds': {
      let parsed: unknown;
      try { parsed = JSON.parse(input); } catch { return { valid: false, errors: ['Invalid JSON'] }; }
      const arr = Array.isArray(parsed) ? parsed : (parsed as Record<string, unknown>).scaffolds;
      if (!Array.isArray(arr)) return { valid: false, errors: ['Expected scaffolds array'] };
      for (const s of arr as Array<Record<string, unknown>>) {
        if (!s.task_id) errors.push(`Scaffold missing 'task_id'`);
      }
      break;
    }

    case 'skill-recommendations': {
      let parsed: unknown;
      try { parsed = JSON.parse(input); } catch { return { valid: false, errors: ['Invalid JSON'] }; }
      const obj = parsed as Record<string, unknown>;
      const recs = Array.isArray(parsed) ? parsed : (obj.recommendations ?? obj.task_skills);
      if (!Array.isArray(recs)) return { valid: false, errors: ['Expected recommendations array or object with recommendations/task_skills'] };
      for (const r of recs as Array<Record<string, unknown>>) {
        if (!r.task_id && r.task_id !== 0) errors.push('Skill recommendation missing task_id');
      }
      break;
    }

    case 'tally': {
      let parsed: unknown;
      try { parsed = JSON.parse(input); } catch { return { valid: false, errors: ['Invalid JSON'] }; }
      const obj = parsed as Record<string, unknown>;
      if (!obj.verdict) errors.push('Tally missing verdict');
      if (!obj.vote_breakdown) errors.push('Tally missing vote_breakdown');
      break;
    }

    case 'debate-turn': {
      if (!input.trim()) {
        errors.push('Debate turn is empty');
      }
      // Token limit check (approximate)
      if (input.length > 50000) {
        errors.push(`Debate turn too long: ${input.length} chars (max ~50000)`);
      }
      break;
    }

    case 'decision-points': {
      let parsed: unknown;
      try { parsed = JSON.parse(input); } catch { return { valid: false, errors: ['Invalid JSON'] }; }
      if (!Array.isArray(parsed)) return { valid: false, errors: ['Expected JSON array of decision points'] };
      for (const dp of parsed as Array<Record<string, unknown>>) {
        if (!dp.id) errors.push(`Decision point missing 'id'`);
        if (!dp.question) errors.push(`DP ${dp.id ?? '?'} missing 'question'`);
      }
      break;
    }

    case 'project-decision-points': {
      let parsed: unknown;
      try { parsed = JSON.parse(input); } catch { return { valid: false, errors: ['Invalid JSON'] }; }
      if (!Array.isArray(parsed)) return { valid: false, errors: ['Expected JSON array of project decision points'] };
      const validCategories = new Set(['architecture', 'language-runtime', 'service-topology', 'platform-choice', 'build-vs-buy', 'data-model', 'api-design', 'ux-behavior', 'security', 'visual-identity', 'design-system', 'component-library', 'layout-pattern']);
      const validConstraints = new Set(['hard', 'soft', 'open', 'escalation']);
      for (const dp of parsed as Array<Record<string, unknown>>) {
        const dpId = (dp.id as string) ?? '?';
        if (!dp.id) errors.push(`Project DP missing 'id'`);
        if (!dp.category) errors.push(`DP ${dpId} missing 'category'`);
        else if (!validCategories.has(dp.category as string)) errors.push(`DP ${dpId} invalid category '${dp.category}'`);
        if (!dp.description) errors.push(`DP ${dpId} missing 'description'`);
        if (!Array.isArray(dp.options) || (dp.options as unknown[]).length < 2) errors.push(`DP ${dpId} needs at least 2 options`);
        if (typeof dp.requires_approval !== 'boolean') errors.push(`DP ${dpId} missing 'requires_approval' boolean`);
        if (!dp.constraint_type) errors.push(`DP ${dpId} missing 'constraint_type'`);
        else if (!validConstraints.has(dp.constraint_type as string)) errors.push(`DP ${dpId} invalid constraint_type '${dp.constraint_type}'`);
        if (!Array.isArray(dp.affected_tasks)) errors.push(`DP ${dpId} missing 'affected_tasks' array`);
        if (!dp.rationale) { if (strict) errors.push(`DP ${dpId} missing 'rationale'`); }
      }
      break;
    }

    case 'decision-tally': {
      let parsed: unknown;
      try { parsed = JSON.parse(input); } catch { return { valid: false, errors: ['Invalid JSON'] }; }
      const obj = parsed as Record<string, unknown>;
      if (!obj.winning_option && !obj.winner) errors.push('Decision tally missing winner');
      break;
    }

    case 'deliberation-result': {
      let parsed: unknown;
      try { parsed = JSON.parse(input); } catch { return { valid: false, errors: ['Invalid JSON'] }; }
      const obj = parsed as Record<string, unknown>;
      if (!obj.design_brief && !obj.result) errors.push('Deliberation result missing design_brief');
      if (!obj.decision_points && !obj.decisions) errors.push('Deliberation result missing decision data');
      break;
    }

    default:
      // Unknown type — just check non-empty
      if (strict) {
        errors.push(`Unknown validation type: ${type}`);
      }
  }

  return { valid: errors.length === 0, errors };
}

export function validatePrompts(merged: unknown[], taskIds: number[]): ValidationResult {
  const errors: string[] = [];
  const prompts = merged as PromptItem[];

  // Completeness
  const foundIds = new Set(prompts.map((p) => p.task_id));
  for (const id of taskIds) {
    if (!foundIds.has(id)) {
      errors.push(`Missing prompts for task_id ${id}`);
    }
  }

  // Structural integrity
  for (const prompt of prompts) {
    if (prompt.task_id == null) {
      errors.push('Prompt entry missing task_id');
      continue;
    }
    if (!prompt.prompt_md?.trim()) {
      errors.push(`Task ${prompt.task_id}: prompt_md is empty`);
    }
    if (!prompt.prompt_xml?.trim()) {
      errors.push(`Task ${prompt.task_id}: prompt_xml is empty`);
    }

    // Subtask structural check (if present)
    if (prompt.subtasks) {
      for (const st of prompt.subtasks) {
        if (st.subtask_id == null) {
          errors.push(`Task ${prompt.task_id}: subtask missing subtask_id`);
        } else if (!st.prompt_md?.trim()) {
          errors.push(`Task ${prompt.task_id}, subtask ${st.subtask_id}: prompt_md is empty`);
        }
      }
    }
  }

  return { valid: errors.length === 0, errors };
}
