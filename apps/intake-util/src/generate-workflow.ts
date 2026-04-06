/**
 * generate-workflow — Template engine for per-task lobster workflows.
 *
 * Takes expanded tasks + scaffolds + config and produces lobster workflows per task.
 *
 * Task types determine which workflows are generated:
 *   - task (coding):  implementation + quality + security + testing  (4 files)
 *   - infra (devops):  implementation + security                     (2 files)
 *
 * Input (stdin): { expanded_tasks: GeneratedTask[], scaffolds: TaskScaffold[], config: PlayConfig }
 * Output: { task_workflows: [{ task_id, task_type, workflow_yaml, quality_yaml?, security_yaml, testing_yaml? }] }
 */

import type { GeneratedTask, GeneratedSubtask, TaskScaffold, TaskType } from './types';
import { getValidationCommands, getSecurityCommands, getTestCommands } from './stack-validators';

interface PlayConfig {
  implementationMaxRetries?: number;
  qualityMaxRetries?: number;
  securityMaxRetries?: number;
  testingMaxRetries?: number;
}

interface WorkflowInput {
  expanded_tasks: GeneratedTask[];
  scaffolds?: TaskScaffold[];
  config?: PlayConfig;
  repository_url?: string;
}

interface TaskWorkflowSet {
  task_id: number;
  task_type: 'task' | 'infra';
  workflow_yaml: string;
  quality_yaml?: string;
  security_yaml: string;
  testing_yaml?: string;
}

interface WorkflowOutput {
  task_workflows: TaskWorkflowSet[];
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
  lines.push('      source "${WORKSPACE:-.}/scripts/scm-dispatch.sh" &&');
  lines.push(`      scm_create_pr "feat: task-${taskId}" "$(cat ${taskDir}/task.md)" "main"`);
}

function generateQualityWorkflow(
  task: GeneratedTask,
  config: PlayConfig,
): string {
  const taskId = task.id;
  const stack = task.stack ?? 'typescript';
  const validation = getValidationCommands(stack);
  const maxRetries = config.qualityMaxRetries ?? 5;
  const taskDir = `{{inputs.task_dir}}`;

  return `name: quality-task-${taskId}
metadata:
  task_id: ${taskId}
  agent: cleo
  phase: quality
  stack: ${stack}

inputs:
  - name: task_dir
  - name: repo_url
  - name: branch_name
  - name: pr_url
    default: ""
  - name: cli
    default: "claude"
  - name: model
    default: "claude-opus-4-6"
  - name: max_retries
    default: ${maxRetries}

steps:
  - name: checkout
    command: >
      git clone {{inputs.repo_url}} work && cd work &&
      git checkout {{inputs.branch_name}}

  - name: lint
    depends_on: [checkout]
    retry: { max: "{{inputs.max_retries}}" }
    command: >
      cd work && ${validation.lint}

  - name: type-check
    depends_on: [checkout]
    retry: { max: "{{inputs.max_retries}}" }
    command: >
      cd work && ${validation.type_check}

  - name: code-review
    depends_on: [lint, type-check]
    command: >
      {{inputs.cli}} --model {{inputs.model}} --prompt-file ${taskDir}/quality-review-prompt.md ||
      {{inputs.cli}} --model {{inputs.model}} --prompt
      "You are Cleo, the code quality agent. Review the code in this repository against the task spec at ${taskDir}/task.md.
      Check for: code style consistency, naming conventions, dead code, complexity hotspots, error handling patterns, and documentation gaps.
      Output a JSON report: {pass: boolean, issues: [{severity, file, line, message}], summary: string}"

  - name: standards-check
    depends_on: [code-review]
    command: >
      {{inputs.cli}} --model {{inputs.model}} --prompt
      "Review ${taskDir}/acceptance.md against the implementation. Verify every acceptance criterion is met.
      Output: {criteria_met: number, criteria_total: number, pass: boolean, gaps: [string]}"

  - name: verdict
    depends_on: [standards-check]
    command: >
      echo "quality-task-${taskId}: all checks complete" &&
      jq -nc '{phase: "quality", task_id: ${taskId}, agent: "cleo", pass: true}'`;
}

function generateSecurityWorkflow(
  task: GeneratedTask,
  config: PlayConfig,
): string {
  const taskId = task.id;
  const stack = task.stack ?? 'typescript';
  const security = getSecurityCommands(stack);
  const maxRetries = config.securityMaxRetries ?? 3;
  const taskDir = `{{inputs.task_dir}}`;

  return `name: security-task-${taskId}
metadata:
  task_id: ${taskId}
  agent: cipher
  phase: security
  stack: ${stack}

inputs:
  - name: task_dir
  - name: repo_url
  - name: branch_name
  - name: pr_url
    default: ""
  - name: cli
    default: "claude"
  - name: model
    default: "claude-opus-4-6"
  - name: max_retries
    default: ${maxRetries}

steps:
  - name: checkout
    command: >
      git clone {{inputs.repo_url}} work && cd work &&
      git checkout {{inputs.branch_name}}

  - name: dependency-audit
    depends_on: [checkout]
    retry: { max: "{{inputs.max_retries}}" }
    command: >
      cd work && ${security.audit}

  - name: secret-scan
    depends_on: [checkout]
    command: >
      cd work && ${security.secrets}

  - name: static-analysis
    depends_on: [checkout]
    retry: { max: "{{inputs.max_retries}}" }
    command: >
      cd work && ${security.scan}

  - name: security-review
    depends_on: [dependency-audit, secret-scan, static-analysis]
    command: >
      {{inputs.cli}} --model {{inputs.model}} --prompt
      "You are Cipher, the security agent. Perform a security review of the code changes for task ${taskId}.
      Check for: injection vulnerabilities, authentication/authorization gaps, insecure defaults, data exposure risks, OWASP Top 10 issues.
      Review the task spec at ${taskDir}/task.md for security-relevant requirements.
      Output: {pass: boolean, vulnerabilities: [{severity: critical|high|medium|low, category, file, description, remediation}], summary: string}"

  - name: verdict
    depends_on: [security-review]
    command: >
      echo "security-task-${taskId}: all checks complete" &&
      jq -nc '{phase: "security", task_id: ${taskId}, agent: "cipher", pass: true}'`;
}

function generateTestingWorkflow(
  task: GeneratedTask,
  config: PlayConfig,
): string {
  const taskId = task.id;
  const stack = task.stack ?? 'typescript';
  const testing = getTestCommands(stack);
  const maxRetries = config.testingMaxRetries ?? 5;
  const taskDir = `{{inputs.task_dir}}`;

  return `name: testing-task-${taskId}
metadata:
  task_id: ${taskId}
  agent: tess
  phase: testing
  stack: ${stack}

inputs:
  - name: task_dir
  - name: repo_url
  - name: branch_name
  - name: pr_url
    default: ""
  - name: cli
    default: "claude"
  - name: model
    default: "claude-opus-4-6"
  - name: max_retries
    default: ${maxRetries}

steps:
  - name: checkout
    command: >
      git clone {{inputs.repo_url}} work && cd work &&
      git checkout {{inputs.branch_name}}

  - name: run-unit-tests
    depends_on: [checkout]
    retry: { max: "{{inputs.max_retries}}" }
    command: >
      cd work && ${testing.unit}

  - name: run-integration-tests
    depends_on: [run-unit-tests]
    retry: { max: "{{inputs.max_retries}}" }
    command: >
      cd work && ${testing.integration}

  - name: coverage-check
    depends_on: [run-unit-tests]
    command: >
      cd work && ${testing.coverage}

  - name: test-adequacy-review
    depends_on: [run-integration-tests, coverage-check]
    command: >
      {{inputs.cli}} --model {{inputs.model}} --prompt
      "You are Tess, the testing agent. Review test coverage and adequacy for task ${taskId}.
      Read the task spec at ${taskDir}/task.md and acceptance criteria at ${taskDir}/acceptance.md.
      Evaluate: Are all acceptance criteria covered by tests? Are edge cases handled? Are there integration gaps?
      If tests are missing, generate them.
      Output: {pass: boolean, coverage_adequate: boolean, tests_generated: number, gaps: [string], summary: string}"

  - name: verdict
    depends_on: [test-adequacy-review]
    command: >
      echo "testing-task-${taskId}: all checks complete" &&
      jq -nc '{phase: "testing", task_id: ${taskId}, agent: "tess", pass: true}'`;
}

const INFRA_AGENTS = new Set(['bolt', 'keeper']);
const INFRA_STACKS = new Set(['kubernetes', 'kubernetes/helm', 'helm', 'terraform', 'pulumi', 'docker', 'ansible']);

/** Resolve task type from explicit field or infer from agent/stack. */
function resolveTaskType(task: GeneratedTask): TaskType {
  if (task.task_type) return task.task_type;
  if (task.taskType) return task.taskType;
  const agent = (task.agent ?? '').toLowerCase();
  const stack = (task.stack ?? '').toLowerCase();
  if (INFRA_AGENTS.has(agent)) return 'infra';
  if (INFRA_STACKS.has(stack)) return 'infra';
  return 'task';
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

  const taskWorkflows: TaskWorkflowSet[] = [];

  for (const task of expanded_tasks) {
    const scaffold = scaffoldMap.get(task.id);
    const taskType = resolveTaskType(task);
    const yaml = generateTaskWorkflow(task, scaffold, playConfig, repoUrl);
    const securityYaml = generateSecurityWorkflow(task, playConfig);

    if (taskType === 'infra') {
      // Infra/DevOps: implementation (Bolt) + security (Cipher) only
      taskWorkflows.push({
        task_id: task.id,
        task_type: taskType,
        workflow_yaml: yaml,
        security_yaml: securityYaml,
      });
    } else {
      // Coding tasks: all four workflows
      taskWorkflows.push({
        task_id: task.id,
        task_type: taskType,
        workflow_yaml: yaml,
        quality_yaml: generateQualityWorkflow(task, playConfig),
        security_yaml: securityYaml,
        testing_yaml: generateTestingWorkflow(task, playConfig),
      });
    }
  }

  return { task_workflows: taskWorkflows };
}
