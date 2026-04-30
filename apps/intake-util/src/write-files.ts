/**
 * Write Files — writes LLM-generated JSON output to disk as individual files.
 *
 * Supports three modes:
 *   --type docs:      reads {task_docs: [{task_id, task_md, decisions_md, acceptance_md}]}
 *                     writes task-{id}/task.md, task-{id}/decisions.md, task-{id}/acceptance.md
 *
 *   --type prompts:   reads {task_prompts: [{task_id, prompt_md, prompt_xml, subtasks}]}
 *                     writes task-{id}/prompt.md, task-{id}/prompt.xml,
 *                            task-{id}/subtasks/task-{id}.{subtask_id}/prompt.md
 *
 *   --type workflows: reads {task_workflows: [{task_id, workflow_yaml}]}
 *                     writes task-{id}/implementation.lobster.yaml
 */

import * as fs from 'fs/promises';
import * as path from 'path';

interface DocEntry {
  task_id: number;
  task_md: string;
  decisions_md: string;
  acceptance_md: string;
}

interface SubtaskEntry {
  subtask_id?: number | string | null;
  id?: number | string | null;
  title?: string;
  prompt_md: string;
}

interface PromptEntry {
  task_id: number;
  prompt_md: string;
  prompt_xml: string;
  subtasks?: SubtaskEntry[];
}

interface WorkflowEntry {
  task_id: number;
  workflow_yaml: string;
  quality_yaml?: string;
  security_yaml?: string;
  testing_yaml?: string;
}

function asText(value: unknown): string {
  if (typeof value === 'string') return value;
  if (value == null) return '';
  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return String(value);
  }
}

function asArray<T>(input: unknown, keys: string[]): T[] | null {
  if (Array.isArray(input)) return input as T[];
  if (input && typeof input === 'object') {
    const obj = input as Record<string, unknown>;
    for (const key of keys) {
      if (Array.isArray(obj[key])) return obj[key] as T[];
    }
  }
  return null;
}

export async function writeFiles(
  input: unknown,
  basePath: string,
  type: 'docs' | 'prompts' | 'workflows',
): Promise<{ files_written: number; paths: string[] }> {
  const paths: string[] = [];

  if (type === 'workflows') {
    const workflows = asArray<WorkflowEntry>(input, ['task_workflows', 'workflows']);
    if (!workflows) {
      throw new Error('Expected workflow array (task_workflows/workflows or raw array) for --type workflows');
    }

    for (const wf of workflows) {
      const taskDir = path.join(basePath, `task-${wf.task_id}`);
      await fs.mkdir(taskDir, { recursive: true });

      const implPath = path.join(taskDir, 'implementation.lobster.yaml');
      await fs.writeFile(implPath, asText(wf.workflow_yaml));
      paths.push(implPath);

      if (wf.quality_yaml) {
        const qPath = path.join(taskDir, 'quality.lobster.yaml');
        await fs.writeFile(qPath, asText(wf.quality_yaml));
        paths.push(qPath);
      }
      if (wf.security_yaml) {
        const sPath = path.join(taskDir, 'security.lobster.yaml');
        await fs.writeFile(sPath, asText(wf.security_yaml));
        paths.push(sPath);
      }
      if (wf.testing_yaml) {
        const tPath = path.join(taskDir, 'testing.lobster.yaml');
        await fs.writeFile(tPath, asText(wf.testing_yaml));
        paths.push(tPath);
      }
    }

    // Write master play.lobster.yaml at the tasks root
    const obj = input as Record<string, unknown>;
    if (typeof obj.play_yaml === 'string') {
      const playPath = path.join(basePath, 'play.lobster.yaml');
      await fs.writeFile(playPath, obj.play_yaml);
      paths.push(playPath);
    }
  } else if (type === 'docs') {
    const docs = asArray<DocEntry>(input, ['task_docs', 'docs']);
    if (!docs) {
      throw new Error('Expected docs array (task_docs/docs or raw array) for --type docs');
    }

    for (const doc of docs) {
      const taskDir = path.join(basePath, `task-${doc.task_id}`);
      await fs.mkdir(taskDir, { recursive: true });

      const files: [string, string][] = [
        ['task.md', asText(doc.task_md)],
        ['decisions.md', asText(doc.decisions_md)],
        ['acceptance.md', asText(doc.acceptance_md)],
      ];

      for (const [name, content] of files) {
        const filePath = path.join(taskDir, name);
        await fs.writeFile(filePath, content);
        paths.push(filePath);
      }
    }
  } else {
    const prompts = asArray<PromptEntry>(input, ['task_prompts', 'prompts']);
    if (!prompts) {
      throw new Error('Expected prompts array (task_prompts/prompts or raw array) for --type prompts');
    }

    for (const prompt of prompts) {
      const taskDir = path.join(basePath, `task-${prompt.task_id}`);
      await fs.mkdir(taskDir, { recursive: true });

      await fs.writeFile(path.join(taskDir, 'prompt.md'), asText(prompt.prompt_md));
      paths.push(path.join(taskDir, 'prompt.md'));

      await fs.writeFile(path.join(taskDir, 'prompt.xml'), asText(prompt.prompt_xml));
      paths.push(path.join(taskDir, 'prompt.xml'));

      if (prompt.subtasks && prompt.subtasks.length > 0) {
        let subtaskIndex = 0;
        for (const st of prompt.subtasks) {
          subtaskIndex += 1;
          const rawSubtaskId = st.subtask_id ?? st.id ?? subtaskIndex;
          const normalizedSubtaskId = String(rawSubtaskId).trim() || String(subtaskIndex);
          const safeSubtaskId = normalizedSubtaskId.replace(/[^A-Za-z0-9._-]/g, '-');
          const stDir = path.join(taskDir, 'subtasks', `task-${prompt.task_id}.${safeSubtaskId}`);
          await fs.mkdir(stDir, { recursive: true });
          await fs.writeFile(path.join(stDir, 'prompt.md'), asText(st.prompt_md));
          paths.push(path.join(stDir, 'prompt.md'));
        }
      }
    }
  }

  return { files_written: paths.length, paths };
}
