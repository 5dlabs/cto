/**
 * generate-workflow — Template engine for per-task implementation workflows.
 *
 * Takes expanded tasks + scaffolds + config and produces a deterministic
 * implementation.lobster.yaml per task. The workflow controls the execution
 * lifecycle (setup, subtask sequencing, validation gates, PR creation) while
 * prompts remain free-form for agent creativity.
 *
 * Input (stdin): { expanded_tasks: GeneratedTask[], scaffolds: TaskScaffold[], config: PlayConfig }
 * Output: { task_workflows: [{ task_id: number, workflow_yaml: string }] }
 */

import type { GeneratedTask, GeneratedSubtask, TaskScaffold } from './types';
import { getValidationCommands } from './stack-validators';

interface PlayConfig {
  implementationMaxRetries?: number;
  qualityMaxRetries?: number;
}

interface WorkflowInput {
  expanded_tasks: GeneratedTask[];
  scaffolds?: TaskScaffold[];
  config?: PlayConfig;
  repository_url?: string;
}

interface WorkflowOutput {
  task_workflows: Array<{ task_id: number; workflow_yaml: string }>;
}

function indent(text: string, spaces: number): string {
  const pad = ' '.repeat(spaces);
  return text
    .split('\n')
    .map((line) => (line.trim() ? pad + line : ''))
    .join('\n');
}

function generateTaskWorkflow(
  task: GeneratedTask,
  scaffold: TaskScaffold | undefined,
  config: PlayConfig,
  repositoryUrl: string,
): string {
  const taskId = task.id;
  const agent = task.agent ?? 'nova';
  const stack = task.stack ?? 'typescript';
  const validation = getValidationCommands(stack);
  const maxImplRetries = config.implementationMaxRetries ?? 10;
  const maxQualRetries = config.qualityMaxRetries ?? 5;
  const subtasks = task.subtasks ?? [];
  const branchDefault = `task-${taskId}/feature-branch`;
  const taskDir = `{{inputs.task_dir}}`;

  const lines: string[] = [];

  // Header
  lines.push(`name: implement-task-${taskId}`);
  lines.push(`metadata:`);
  lines.push(`  task_id: ${taskId}`);
  lines.push(`  agent: ${agent}`);
  lines.push(`  stack: ${stack}`);
  if (task.dependencies.length > 0) {
    lines.push(`  depends_on_tasks: [${task.dependencies.join(', ')}]`);
  }
  lines.push('');

  // Inputs
  lines.push('inputs:');
  lines.push('  - name: task_dir');
  lines.push('  - name: repo_url');
  lines.push('  - name: branch_name');
  lines.push(`    default: "${branchDefault}"`);
  lines.push('  - name: agent');
  lines.push(`    default: "${agent}"`);
  lines.push('  - name: cli');
  lines.push('    default: "claude"');
  lines.push('  - name: model');
  lines.push('    default: "claude-opus-4-6"');
  lines.push('  - name: max_implementation_retries');
  lines.push(`    default: ${maxImplRetries}`);
  lines.push('  - name: max_quality_retries');
  lines.push(`    default: ${maxQualRetries}`);
  lines.push('');

  // Steps
  lines.push('steps:');

  // Setup
  lines.push('  - name: setup');
  lines.push('    command: >');
  lines.push('      git clone {{inputs.repo_url}} work && cd work &&');
  lines.push('      git checkout -b {{inputs.branch_name}}');
  lines.push('');

  // Scaffold files
  if (scaffold?.file_structure && scaffold.file_structure.length > 0) {
    const dirs = new Set<string>();
    for (const f of scaffold.file_structure) {
      const dir = f.path.replace(/\/[^/]+$/, '');
      if (dir && dir !== f.path) dirs.add(dir);
    }
    if (dirs.size > 0) {
      lines.push('  - name: scaffold-files');
      lines.push('    depends_on: [setup]');
      lines.push('    command: >');
      lines.push(`      cd work && mkdir -p ${Array.from(dirs).join(' ')}`);
      lines.push('');
    }
  }

  const scaffoldStep = scaffold?.file_structure?.length ? 'scaffold-files' : 'setup';

  // Subtask implementation + validation steps
  if (subtasks.length === 0) {
    // No subtasks — single implementation step
    lines.push('  - name: implement');
    lines.push(`    depends_on: [${scaffoldStep}]`);
    lines.push(`    retry: { max: "{{inputs.max_implementation_retries}}" }`);
    lines.push('    command: >');
    lines.push(`      {{inputs.cli}} --model {{inputs.model}} --prompt-file ${taskDir}/prompt.md`);
    lines.push('');

    lines.push('  - name: validate');
    lines.push('    depends_on: [implement]');
    lines.push(`    retry: { max: "{{inputs.max_quality_retries}}" }`);
    lines.push('    command: >');
    lines.push(`      cd work && ${validation.type_check} && ${validation.test} && ${validation.lint}`);
    lines.push('');

    appendFinalSteps(lines, taskId, taskDir, ['validate']);
  } else {
    // Build dependency graph for subtasks
    const subtaskSteps = generateSubtaskSteps(
      subtasks,
      taskId,
      taskDir,
      scaffoldStep,
      validation,
    );
    lines.push(...subtaskSteps.lines);

    appendFinalSteps(lines, taskId, taskDir, subtaskSteps.finalValidateSteps);
  }

  return lines.join('\n');
}

interface SubtaskResult {
  lines: string[];
  finalValidateSteps: string[];
}

function generateSubtaskSteps(
  subtasks: GeneratedSubtask[],
  taskId: number,
  taskDir: string,
  scaffoldStep: string,
  validation: ReturnType<typeof getValidationCommands>,
): SubtaskResult {
  const lines: string[] = [];
  const validateStepNames: string[] = [];

  for (const st of subtasks) {
    const stIdx = st.id;
    const implName = `implement-subtask-${stIdx}`;
    const valName = `validate-subtask-${stIdx}`;

    // Determine dependencies
    const deps: string[] = [];
    if (st.dependencies && st.dependencies.length > 0) {
      // Depend on validation of prerequisite subtasks
      for (const depId of st.dependencies) {
        deps.push(`validate-subtask-${depId}`);
      }
    } else {
      deps.push(scaffoldStep);
    }

    // Implementation step
    lines.push(`  - name: ${implName}`);
    lines.push(`    depends_on: [${deps.join(', ')}]`);
    lines.push(`    retry: { max: "{{inputs.max_implementation_retries}}" }`);
    lines.push('    command: >');
    lines.push(`      {{inputs.cli}} --model {{inputs.model}} --prompt-file ${taskDir}/subtasks/task-${taskId}.${stIdx}/prompt.md`);
    lines.push('');

    // Validation step
    lines.push(`  - name: ${valName}`);
    lines.push(`    depends_on: [${implName}]`);
    lines.push(`    retry: { max: "{{inputs.max_quality_retries}}" }`);
    lines.push('    command: >');
    lines.push(`      cd work && ${validation.type_check} && ${validation.test} && ${validation.lint}`);
    lines.push('');

    validateStepNames.push(valName);
  }

  return { lines, finalValidateSteps: validateStepNames };
}

function appendFinalSteps(
  lines: string[],
  taskId: number,
  taskDir: string,
  dependsOn: string[],
): void {
  // Integration validation
  lines.push('  - name: integration-validation');
  lines.push(`    depends_on: [${dependsOn.join(', ')}]`);
  lines.push('    command: >');
  lines.push('      cd work && bun tsc --noEmit && bun test --run && bun lint');
  lines.push('');

  // Acceptance check
  lines.push('  - name: acceptance-check');
  lines.push('    depends_on: [integration-validation]');
  lines.push('    command: >');
  lines.push(`      {{inputs.cli}} --model {{inputs.model}} --prompt "Review changes against ${taskDir}/acceptance.md. Output PASS or FAIL."`);
  lines.push('');

  // Create PR
  lines.push('  - name: create-pr');
  lines.push('    depends_on: [acceptance-check]');
  lines.push('    command: >');
  lines.push('      cd work && git add -A &&');
  lines.push(`      git commit -m "feat: task-${taskId}" &&`);
  lines.push('      git push origin {{inputs.branch_name}} &&');
  lines.push(`      gh pr create --title "feat: task-${taskId}" --body-file ${taskDir}/task.md`);
}

export function generateWorkflows(input: WorkflowInput): WorkflowOutput {
  const { expanded_tasks, scaffolds, config, repository_url } = input;
  const playConfig: PlayConfig = config ?? {};
  const repoUrl = repository_url ?? '';

  const scaffoldMap = new Map<number, TaskScaffold>();
  if (scaffolds) {
    for (const s of scaffolds) {
      if (Array.isArray(s)) {
        // Handle { scaffolds: [...] } wrapper
        continue;
      }
      scaffoldMap.set(s.task_id, s);
    }
  }

  const taskWorkflows: Array<{ task_id: number; workflow_yaml: string }> = [];

  for (const task of expanded_tasks) {
    const scaffold = scaffoldMap.get(task.id);
    const yaml = generateTaskWorkflow(task, scaffold, playConfig, repoUrl);
    taskWorkflows.push({ task_id: task.id, workflow_yaml: yaml });
  }

  return { task_workflows: taskWorkflows };
}
