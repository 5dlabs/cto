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
  subtask_id: number;
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
}

export async function writeFiles(
  input: unknown,
  basePath: string,
  type: 'docs' | 'prompts' | 'workflows',
): Promise<{ files_written: number; paths: string[] }> {
  const paths: string[] = [];

  if (type === 'workflows') {
    const data = input as { task_workflows: WorkflowEntry[] };
    if (!data.task_workflows || !Array.isArray(data.task_workflows)) {
      throw new Error('Expected { task_workflows: [...] } for --type workflows');
    }

    for (const wf of data.task_workflows) {
      const taskDir = path.join(basePath, `task-${wf.task_id}`);
      await fs.mkdir(taskDir, { recursive: true });

      const filePath = path.join(taskDir, 'implementation.lobster.yaml');
      await fs.writeFile(filePath, wf.workflow_yaml);
      paths.push(filePath);
    }
  } else if (type === 'docs') {
    const data = input as { task_docs: DocEntry[] };
    if (!data.task_docs || !Array.isArray(data.task_docs)) {
      throw new Error('Expected { task_docs: [...] } for --type docs');
    }

    for (const doc of data.task_docs) {
      const taskDir = path.join(basePath, `task-${doc.task_id}`);
      await fs.mkdir(taskDir, { recursive: true });

      const files: [string, string][] = [
        ['task.md', doc.task_md],
        ['decisions.md', doc.decisions_md],
        ['acceptance.md', doc.acceptance_md],
      ];

      for (const [name, content] of files) {
        const filePath = path.join(taskDir, name);
        await fs.writeFile(filePath, content);
        paths.push(filePath);
      }
    }
  } else {
    const data = input as { task_prompts: PromptEntry[] };
    if (!data.task_prompts || !Array.isArray(data.task_prompts)) {
      throw new Error('Expected { task_prompts: [...] } for --type prompts');
    }

    for (const prompt of data.task_prompts) {
      const taskDir = path.join(basePath, `task-${prompt.task_id}`);
      await fs.mkdir(taskDir, { recursive: true });

      await fs.writeFile(path.join(taskDir, 'prompt.md'), prompt.prompt_md);
      paths.push(path.join(taskDir, 'prompt.md'));

      await fs.writeFile(path.join(taskDir, 'prompt.xml'), prompt.prompt_xml);
      paths.push(path.join(taskDir, 'prompt.xml'));

      if (prompt.subtasks && prompt.subtasks.length > 0) {
        for (const st of prompt.subtasks) {
          const stDir = path.join(taskDir, 'subtasks', `task-${prompt.task_id}.${st.subtask_id}`);
          await fs.mkdir(stDir, { recursive: true });
          await fs.writeFile(path.join(stDir, 'prompt.md'), st.prompt_md);
          paths.push(path.join(stDir, 'prompt.md'));
        }
      }
    }
  }

  return { files_written: paths.length, paths };
}
