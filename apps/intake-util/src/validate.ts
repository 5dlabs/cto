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
