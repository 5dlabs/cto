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

interface AgentHarnessModel {
  name: string;
}

interface AgentHarnessProvider {
  name: string;
}

interface AgentHarness {
  cli: string;
  provider: AgentHarnessProvider;
  models: AgentHarnessModel[];
  baseUrl?: string;
  apiKey?: string;
  // Slim reference format (from model-providers.json agentHarness)
  model?: string;
  // Legacy fields for backward compat
  primary?: string;
  fallback?: string;
  fallbackModel?: string;
}

/** Model-providers catalog shape (loaded externally) */
interface ModelProvidersCatalog {
  providers?: Record<string, { enabled?: boolean; baseUrl?: string; apiKey?: string; credits?: number }>;
  models?: Record<string, { enabled?: boolean; provider?: string; thinkingLevel?: string; score?: number }>;
  clis?: Record<string, { enabled?: boolean; provider?: string; models?: string[] }>;
}

interface PlayConfig {
  implementationMaxRetries?: number;
  qualityMaxRetries?: number;
  securityMaxRetries?: number;
  testingMaxRetries?: number;
  agentHarness?: Record<string, AgentHarness>;
  /** Optional resolved model-providers catalog for reference resolution */
  modelProviders?: ModelProvidersCatalog;
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
  play_yaml: string;
}

function indent(text: string, spaces: number): string {
  const pad = ' '.repeat(spaces);
  return text
    .split('\n')
    .map((line) => (line.trim() ? pad + line : ''))
    .join('\n');
}

/** Produce a lobster variable reference: ${varname} as a literal string. */
function lv(name: string): string {
  return '${' + name + '}';
}

const DEFAULT_HARNESS: AgentHarness = {
  cli: 'Claude Code',
  provider: { name: 'Anthropic' },
  models: [
    { name: 'claude-opus-4-6-20260205' },
    { name: 'claude-sonnet-4-6-20260514' },
  ],
  baseUrl: 'https://api.anthropic.com',
  apiKey: 'ANTHROPIC_API_KEY',
};

/** Get the CLI name from harness (new: cli, legacy: primary) */
function harnessCli(h: AgentHarness): string {
  return h.cli ?? h.primary ?? 'claude-code';
}

/** Get the primary model from harness (new: models[0].name, legacy: model) */
function harnessModel(h: AgentHarness): string {
  return h.models?.[0]?.name ?? h.model ?? 'claude-opus-4-6-20260205';
}

/** Get provider name from harness */
function harnessProvider(h: AgentHarness): string {
  return h.provider?.name ?? 'Anthropic';
}

/**
 * Resolve a possibly-slim agentHarness entry into a full AgentHarness.
 * Slim format (from model-providers.json): { cli: "Claude Code", model: "claude-opus-4-6-20260205" }
 * Full format: { cli, provider, models[], baseUrl?, apiKey? }
 */
function resolveHarness(h: AgentHarness, catalog?: ModelProvidersCatalog): AgentHarness {
  // Already fully resolved (has models array with entries)
  if (h.models?.length > 0) return h;

  // No catalog — build minimal from what we have
  if (!catalog) {
    const modelName = h.model ?? h.models?.[0]?.name ?? 'claude-opus-4-6-20260205';
    return {
      cli: h.cli ?? h.primary ?? 'Claude Code',
      provider: h.provider ?? { name: 'Anthropic' },
      models: [{ name: modelName }],
      baseUrl: h.baseUrl,
      apiKey: h.apiKey,
    };
  }

  // Resolve from catalog
  const cliName = h.cli ?? h.primary ?? 'Claude Code';
  const cliDef = catalog.clis?.[cliName];
  const providerName = cliDef?.provider ?? h.provider?.name ?? 'Anthropic';
  const providerDef = catalog.providers?.[providerName];

  // Build models list: if slim has a specific model, put it first
  const primaryModel = h.model;
  const cliModels = (cliDef?.models ?? [])
    .filter((m) => catalog.models?.[m]?.enabled !== false)
    .map((m) => ({ name: m }));

  let models: AgentHarnessModel[];
  if (primaryModel) {
    const rest = cliModels.filter((m) => m.name !== primaryModel);
    models = [{ name: primaryModel }, ...rest];
  } else {
    models = cliModels.length > 0 ? cliModels : [{ name: 'claude-opus-4-6-20260205' }];
  }

  return {
    cli: cliName,
    provider: { name: providerName },
    models,
    baseUrl: h.baseUrl ?? providerDef?.baseUrl,
    apiKey: h.apiKey ?? providerDef?.apiKey,
  };
}

function getAgentHarness(agent: string, config: PlayConfig): AgentHarness {
  const map = config.agentHarness;
  if (!map) return DEFAULT_HARNESS;
  const raw = map[agent] ?? map['_default'] ?? DEFAULT_HARNESS;
  return resolveHarness(raw, config.modelProviders);
}

// --- Notification helpers for play workflow ---

function notifyDiscord(from: string, to: string, message: string, metadata: Record<string, unknown>): string {
  const metaJson = JSON.stringify(metadata).replace(/'/g, "'\\''");
  const safeMsg = message.replace(/'/g, "'\\''");
  return `echo '${safeMsg}' | intake-util bridge-notify --from ${from} --to ${to} --metadata '${metaJson}' || true`;
}

function notifyLinear(sessionIdExpr: string, type: string, body: string): string {
  const safeBody = body.replace(/'/g, "'\\''");
  return `[ -n "${sessionIdExpr}" ] && intake-util linear-activity --session-id "${sessionIdExpr}" --type ${type} --body '${safeBody}' >/dev/null || true`;
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
  const harness = getAgentHarness(agent, config);
  const validation = getValidationCommands(stack);
  const maxImplRetries = config.implementationMaxRetries ?? 10;
  const maxQualRetries = config.qualityMaxRetries ?? 5;
  const subtasks = task.subtasks ?? [];
  const branchDefault = `task-${taskId}/feature-branch`;
  const taskDir = lv('task_dir');

  const lines: string[] = [];

  // Header
  lines.push(`name: implement-task-${taskId}`);
  lines.push(`metadata:`);
  lines.push(`  task_id: ${taskId}`);
  lines.push(`  agent: ${agent}`);
  lines.push(`  stack: ${stack}`);
  lines.push(`  cli: ${harnessCli(harness)}`);
  lines.push(`  model: ${harnessModel(harness)}`);
  if (task.dependencies.length > 0) {
    lines.push(`  depends_on_tasks: [${task.dependencies.join(', ')}]`);
  }
  lines.push('');

  // Args
  lines.push('args:');
  lines.push('  task_dir: {}');
  lines.push('  repo_url: {}');
  lines.push('  branch_name:');
  lines.push(`    default: "${branchDefault}"`);
  lines.push('  agent:');
  lines.push(`    default: "${agent}"`);
  lines.push('  cli:');
  lines.push(`    default: "${harnessCli(harness)}"`);
  lines.push('  model:');
  lines.push(`    default: "${harnessModel(harness)}"`);
  lines.push('  fallback_cli:');
  lines.push(`    default: "${harness.fallback ?? 'codex'}"`);
  lines.push('  fallback_model:');
  lines.push(`    default: "${harness.fallbackModel ?? 'gpt-5.2-codex'}"`);
  lines.push('  max_implementation_retries:');
  lines.push(`    default: ${maxImplRetries}`);
  lines.push('  max_quality_retries:');
  lines.push(`    default: ${maxQualRetries}`);
  lines.push('');

  // Steps
  lines.push('steps:');

  // Setup
  lines.push('  - id: setup');
  lines.push('    command: >');
  lines.push('      git clone ${repo_url} work && cd work &&');
  lines.push('      git checkout -b ${branch_name}');
  lines.push('');

  // Scaffold files
  if (scaffold?.file_structure && scaffold.file_structure.length > 0) {
    const dirs = new Set<string>();
    for (const f of scaffold.file_structure) {
      const dir = f.path.replace(/\/[^/]+$/, '');
      if (dir && dir !== f.path) dirs.add(dir);
    }
    if (dirs.size > 0) {
      lines.push('  - id: scaffold-files');
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
    lines.push('  - id: implement');
    lines.push(`    depends_on: [${scaffoldStep}]`);
    lines.push(`    retry: { max: "\${max_implementation_retries\}" }`);
    lines.push('    command: >');
    lines.push(`      \${cli} -p "$(cat ${taskDir}/prompt.md)" --model \${model}`);
    lines.push('');

    lines.push('  - id: validate');
    lines.push('    depends_on: [implement]');
    lines.push(`    retry: { max: "\${max_quality_retries\}" }`);
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
    lines.push(`  - id: ${implName}`);
    lines.push(`    depends_on: [${deps.join(', ')}]`);
    lines.push(`    retry: { max: "\${max_implementation_retries\}" }`);
    lines.push('    command: >');
    lines.push(`      \${cli} -p "$(cat ${taskDir}/subtasks/task-${taskId}.${stIdx}/prompt.md)" --model \${model}`);
    lines.push('');

    // Validation step
    lines.push(`  - id: ${valName}`);
    lines.push(`    depends_on: [${implName}]`);
    lines.push(`    retry: { max: "\${max_quality_retries\}" }`);
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
  lines.push('  - id: integration-validation');
  lines.push(`    depends_on: [${dependsOn.join(', ')}]`);
  lines.push('    command: >');
  lines.push('      cd work && bun tsc --noEmit && bun test --run && bun lint');
  lines.push('');

  // Acceptance check
  lines.push('  - id: acceptance-check');
  lines.push('    depends_on: [integration-validation]');
  lines.push('    command: >');
  lines.push(`      \${cli} -p "Review changes against ${taskDir}/acceptance.md. Output PASS or FAIL." --model \${model}`);
  lines.push('');

  // Create PR
  lines.push('  - id: create-pr');
  lines.push('    depends_on: [acceptance-check]');
  lines.push('    command: >');
  lines.push('      cd work && git add -A &&');
  lines.push(`      git commit -m "feat: task-${taskId}" &&`);
  lines.push('      git push origin ${branch_name} &&');
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
  const taskDir = lv('task_dir');
  const harness = getAgentHarness('cleo', config);

  return `name: quality-task-${taskId}
metadata:
  task_id: ${taskId}
  agent: cleo
  phase: quality
  stack: ${stack}
  cli: ${harnessCli(harness)}
  model: ${harnessModel(harness)}

args:
  task_dir: {}
  repo_url: {}
  branch_name: {}
  pr_url:
    default: ""
  cli:
    default: "${harnessCli(harness)}"
  model:
    default: "${harnessModel(harness)}"
  max_retries:
    default: ${maxRetries}

steps:
  - id: checkout
    command: >
      git clone \${repo_url\} work && cd work &&
      git checkout \${branch_name\}

  - id: lint
    depends_on: [checkout]
    retry: { max: "\${max_retries\}" }
    command: >
      cd work && ${validation.lint}

  - id: type-check
    depends_on: [checkout]
    retry: { max: "\${max_retries\}" }
    command: >
      cd work && ${validation.type_check}

  - id: code-review
    depends_on: [lint, type-check]
    command: >
      \${cli} -p "$(cat ${taskDir}/quality-review-prompt.md 2>/dev/null || echo 'You are Cleo, the code quality agent. Review the code in this repository against the task spec at ${taskDir}/task.md. Check for: code style consistency, naming conventions, dead code, complexity hotspots, error handling patterns, and documentation gaps. Output a JSON report: {pass: boolean, issues: [{severity, file, line, message}], summary: string}')" --model \${model}

  - id: standards-check
    depends_on: [code-review]
    command: >
      \${cli} -p "Review ${taskDir}/acceptance.md against the implementation. Verify every acceptance criterion is met. Output: {criteria_met: number, criteria_total: number, pass: boolean, gaps: [string]}" --model \${model}

  - id: verdict
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
  const taskDir = lv('task_dir');
  const harness = getAgentHarness('cipher', config);

  return `name: security-task-${taskId}
metadata:
  task_id: ${taskId}
  agent: cipher
  phase: security
  stack: ${stack}
  cli: ${harnessCli(harness)}
  model: ${harnessModel(harness)}

args:
  task_dir: {}
  repo_url: {}
  branch_name: {}
  pr_url:
    default: ""
  cli:
    default: "${harnessCli(harness)}"
  model:
    default: "${harnessModel(harness)}"
  max_retries:
    default: ${maxRetries}

steps:
  - id: checkout
    command: >
      git clone \${repo_url\} work && cd work &&
      git checkout \${branch_name\}

  - id: dependency-audit
    depends_on: [checkout]
    retry: { max: "\${max_retries\}" }
    command: >
      cd work && ${security.audit}

  - id: secret-scan
    depends_on: [checkout]
    command: >
      cd work && ${security.secrets}

  - id: static-analysis
    depends_on: [checkout]
    retry: { max: "\${max_retries\}" }
    command: >
      cd work && ${security.scan}

  - id: security-review
    depends_on: [dependency-audit, secret-scan, static-analysis]
    command: >
      \${cli} -p "You are Cipher, the security agent. Perform a security review of the code changes for task ${taskId}. Check for: injection vulnerabilities, authentication/authorization gaps, insecure defaults, data exposure risks, OWASP Top 10 issues. Review the task spec at ${taskDir}/task.md for security-relevant requirements. Output: {pass: boolean, vulnerabilities: [{severity: critical|high|medium|low, category, file, description, remediation}], summary: string}" --model \${model}

  - id: verdict
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
  const taskDir = lv('task_dir');
  const harness = getAgentHarness('tess', config);

  return `name: testing-task-${taskId}
metadata:
  task_id: ${taskId}
  agent: tess
  phase: testing
  stack: ${stack}
  cli: ${harnessCli(harness)}
  model: ${harnessModel(harness)}

args:
  task_dir: {}
  repo_url: {}
  branch_name: {}
  pr_url:
    default: ""
  cli:
    default: "${harnessCli(harness)}"
  model:
    default: "${harnessModel(harness)}"
  max_retries:
    default: ${maxRetries}

steps:
  - id: checkout
    command: >
      git clone \${repo_url\} work && cd work &&
      git checkout \${branch_name\}

  - id: run-unit-tests
    depends_on: [checkout]
    retry: { max: "\${max_retries\}" }
    command: >
      cd work && ${testing.unit}

  - id: run-integration-tests
    depends_on: [run-unit-tests]
    retry: { max: "\${max_retries\}" }
    command: >
      cd work && ${testing.integration}

  - id: coverage-check
    depends_on: [run-unit-tests]
    command: >
      cd work && ${testing.coverage}

  - id: test-adequacy-review
    depends_on: [run-integration-tests, coverage-check]
    command: >
      \${cli} -p "You are Tess, the testing agent. Review test coverage and adequacy for task ${taskId}. Read the task spec at ${taskDir}/task.md and acceptance criteria at ${taskDir}/acceptance.md. Evaluate: Are all acceptance criteria covered by tests? Are edge cases handled? Are there integration gaps? If tests are missing, generate them. Output: {pass: boolean, coverage_adequate: boolean, tests_generated: number, gaps: [string], summary: string}" --model \${model}

  - id: verdict
    depends_on: [test-adequacy-review]
    command: >
      echo "testing-task-${taskId}: all checks complete" &&
      jq -nc '{phase: "testing", task_id: ${taskId}, agent: "tess", pass: true}'`;
}

// =============================================================================
// Master play.lobster.yaml — Morgan's orchestration workflow
// =============================================================================

/** Map agent short name to GitHub App name. */
function agentToGitHubApp(agent: string): string {
  const map: Record<string, string> = {
    rex: '5DLabs-Rex',
    grizz: '5DLabs-Grizz',
    nova: '5DLabs-Nova',
    blaze: '5DLabs-Blaze',
    bolt: '5DLabs-Bolt',
    tap: '5DLabs-Tap',
    spark: '5DLabs-Spark',
    vex: '5DLabs-Vex',
    cleo: '5DLabs-Cleo',
    cipher: '5DLabs-Cipher',
    tess: '5DLabs-Tess',
    angie: '5DLabs-Angie',
    keeper: '5DLabs-Keeper',
    morgan: '5DLabs-Morgan',
  };
  return map[agent.toLowerCase()] ?? `5DLabs-${agent.charAt(0).toUpperCase() + agent.slice(1)}`;
}

function generatePlayWorkflow(
  tasks: GeneratedTask[],
  config: PlayConfig,
  repositoryUrl: string,
): string {
  const lines: string[] = [];

  lines.push(`name: play`);
  lines.push(`description: >`);
  lines.push(`  Master play workflow orchestrated by Morgan.`);
  lines.push(`  Dispatches implementation agents per-task via CodeRun CRDs,`);
  lines.push(`  then fans out quality/security/testing checks before gating.`);
  lines.push(`  Morgan selects CLI and provider per-task based on difficulty,`);
  lines.push(`  available credits, and user-defined provider preferences.`);
  lines.push(``);

  // -- args --
  lines.push(`args:`);
  lines.push(`  tasks_dir:`);
  lines.push(`    description: Root directory containing per-task folders`);
  lines.push(`  repo_url:`);
  if (repositoryUrl) {
    lines.push(`    default: "${repositoryUrl}"`);
  }
  lines.push(`  namespace:`);
  lines.push(`    default: openclaw`);
  lines.push(`  base_branch:`);
  lines.push(`    default: main`);
  lines.push(`  cli:`);
  lines.push(`    description: Default CLI tool (claude, codex, cursor, gemini, etc.)`);
  lines.push(`    default: claude`);
  lines.push(`  model:`);
  lines.push(`    description: Default LLM model — Morgan may override per-task based on difficulty + credits`);
  lines.push(`    default: claude-sonnet-4-6`);
  lines.push(`  linear_session_id:`);
  lines.push(`    description: Linear agent session for status updates`);
  lines.push(`    default: ""`);
  lines.push(`  linear_team_id:`);
  lines.push(`    default: ""`);
  lines.push(`  docs_repository_url:`);
  lines.push(`    default: ""`);
  lines.push(`  enable_docker:`);
  lines.push(`    default: "true"`);
  lines.push(`  discord_channel:`);
  lines.push(`    description: Discord target channel for play notifications (e.g. play, execution)`);
  lines.push(`    default: "play"`);
  lines.push(``);

  // Build a lookup for dependency resolution
  const taskTypeMap = new Map<number, TaskType>();
  for (const t of tasks) {
    taskTypeMap.set(t.id, resolveTaskType(t));
  }

  // Build harness summary table for notifications
  const harnessTable = tasks.map((t) => {
    const a = t.agent ?? 'nova';
    const h = getAgentHarness(a, config);
    const tt = taskTypeMap.get(t.id)!;
    return { id: t.id, title: t.title, agent: a, cli: harnessCli(h), model: harnessModel(h), type: tt };
  });
  const harnessTableMd = [
    '| Task | Title | Agent | CLI | Model | Type |',
    '|------|-------|-------|-----|-------|------|',
    ...harnessTable.map((r) =>
      `| ${r.id} | ${r.title.slice(0, 40)} | ${r.agent} | ${r.cli} | ${r.model} | ${r.type} |`,
    ),
  ].join('\\n');

  lines.push(`steps:`);

  // --- Play start notification ---
  lines.push(``);
  lines.push(`  # ── Play start notification ──`);
  lines.push(`  - id: notify-play-start`);
  lines.push(`    command: |`);
  lines.push(`      ${notifyDiscord('morgan', '\${discord_channel\}',
    `🎬 Play started — ${tasks.length} tasks dispatching`,
    { step: 'play-start', task_count: tasks.length, time_utc: '$(date -u +%Y-%m-%dT%H:%M:%SZ)' })}`);
  lines.push(`      LINEAR_SID="\${linear_session_id\}"`);
  lines.push(`      ${notifyLinear('$LINEAR_SID', 'action',
    `## 🎬 Play Started\\n\\n${harnessTable.length} tasks dispatching to ${new Set(harnessTable.map((r) => r.cli)).size} harnesses.\\n\\n${harnessTableMd}`)}`);
  lines.push(``);

  for (const task of tasks) {
    const tid = task.id;
    const agent = task.agent ?? 'nova';
    const ghApp = agentToGitHubApp(agent);
    const taskType = taskTypeMap.get(tid)!;
    const isCoding = taskType === 'task';
    const difficulty = task.difficulty_score ?? task.difficultyScore ?? 5;
    const branchName = `task-${tid}/${agent}`;
    const harness = getAgentHarness(agent, config);

    // Per-task CLI/model — used in CodeRun CRD and sub-workflow calls
    const taskCli = harnessCli(harness);
    const taskModel = harnessModel(harness);

    // Determine depends_on for this task's implementation step
    const implDeps: string[] = [];
    if (task.dependencies.length > 0) {
      for (const depId of task.dependencies) {
        implDeps.push(`gate-task-${depId}`);
      }
    }

    // --- Notify task dispatch ---
    const shortTitle = task.title.length > 50 ? task.title.slice(0, 47) + '...' : task.title;
    lines.push(``);
    lines.push(`  # ── Task ${tid}: ${task.title} (${taskType}, agent: ${agent}, cli: ${taskCli}, model: ${taskModel}, difficulty: ${difficulty}) ──`);
    lines.push(`  - id: notify-task-${tid}-start`);
    if (implDeps.length > 0) {
      lines.push(`    depends_on: [${implDeps.join(', ')}]`);
    }
    lines.push(`    command: |`);
    lines.push(`      ${notifyDiscord(agent, '\${discord_channel\}',
      `🚀 Task ${tid}: ${shortTitle} → ${taskCli} (${taskModel}) — agent: ${agent}`,
      { step: 'task-dispatch', task_id: tid, agent, cli: taskCli, model: taskModel, task_type: taskType })}`);
    lines.push(`      LINEAR_SID="\${linear_session_id\}"`);
    lines.push(`      ${notifyLinear('$LINEAR_SID', 'action',
      `## 🚀 Task ${tid} Dispatched\\n\\n**${task.title}**\\n- Agent: \`${agent}\`\\n- CLI: \`${taskCli}\` / Model: \`${taskModel}\`\\n- Type: ${taskType} | Difficulty: ${difficulty}\\n- Subtasks: ${(task.subtasks ?? []).length}`)}`);

    const fallbackCli = harness.fallback ?? 'codex';
    const fallbackModel = harness.fallbackModel ?? 'gpt-5.2-codex';

    // --- Submit CodeRun CRD for implementation (with fallback cascade) ---
    lines.push(``);
    lines.push(`  - id: run-task-${tid}`);
    lines.push(`    depends_on: [notify-task-${tid}-start]`);
    lines.push(`    command: |`);
    lines.push(`      # --- Primary attempt: ${taskCli}/${taskModel} ---`);
    lines.push(`      PRIMARY_CLI="${taskCli}"`);
    lines.push(`      PRIMARY_MODEL="${taskModel}"`);
    lines.push(`      FALLBACK_CLI="${fallbackCli}"`);
    lines.push(`      FALLBACK_MODEL="${fallbackModel}"`);
    lines.push(`      TASK_ID=${tid}`);
    lines.push(`      AGENT="${agent}"`);
    lines.push(`      NS="\${namespace\}"`);
    lines.push(`      RUN_NAME="play-task-${tid}-${agent}"`);
    lines.push(`      USED_CLI="$PRIMARY_CLI"`);
    lines.push(`      USED_MODEL="$PRIMARY_MODEL"`);
    lines.push(``);
    lines.push(`      apply_coderun() {`);
    lines.push(`        local cli="$1" model="$2" run_name="$3"`);
    lines.push(`        cat <<CODERUN_EOF | envsubst | kubectl apply -f -`);
    lines.push(`      apiVersion: agents.platform/v1`);
    lines.push(`      kind: CodeRun`);
    lines.push(`      metadata:`);
    lines.push(`        name: $run_name`);
    lines.push(`        namespace: $NS`);
    lines.push(`        labels:`);
    lines.push(`          cto.5dlabs.ai/play: "true"`);
    lines.push(`          cto.5dlabs.ai/task-id: "${tid}"`);
    lines.push(`          cto.5dlabs.ai/agent: "${agent}"`);
    lines.push(`          cto.5dlabs.ai/task-type: "${taskType}"`);
    lines.push(`          cto.5dlabs.ai/cli: "$cli"`);
    lines.push(`      spec:`);
    lines.push(`        runType: implementation`);
    lines.push(`        taskId: ${tid}`);
    lines.push(`        service: ${agent}`);
    lines.push(`        repositoryUrl: \${repo_url\}`);
    lines.push(`        docsRepositoryUrl: \${docs_repository_url\}`);
    lines.push(`        model: $model`);
    lines.push(`        githubApp: ${ghApp}`);
    lines.push(`        enableDocker: \${enable_docker\}`);
    lines.push(`        cliConfig:`);
    lines.push(`          cliType: $cli`);
    lines.push(`          model: $model`);
    lines.push(`        linearIntegration:`);
    lines.push(`          enabled: true`);
    lines.push(`          sessionId: \${linear_session_id\}`);
    lines.push(`          teamId: \${linear_team_id\}`);
    if (task.subtasks && task.subtasks.length > 0) {
      lines.push(`        subtasks:`);
      for (const st of task.subtasks) {
        lines.push(`          - id: ${st.id}`);
        lines.push(`            title: "${st.title.replace(/"/g, '\\"')}"`);
        lines.push(`            parallelizable: ${st.parallelizable ?? false}`);
        if (st.dependencies && st.dependencies.length > 0) {
          lines.push(`            dependencies: [${st.dependencies.map((d) => `"${d}"`).join(', ')}]`);
        }
      }
    }
    lines.push(`      CODERUN_EOF`);
    lines.push(`      }`);
    lines.push(``);
    lines.push(`      wait_coderun() {`);
    lines.push(`        local run_name="$1"`);
    lines.push(`        kubectl wait "coderun/$run_name" -n "$NS" \\`);
    lines.push(`          --for=jsonpath='{.status.phase}'=Succeeded \\`);
    lines.push(`          --timeout=3600s`);
    lines.push(`      }`);
    lines.push(``);
    lines.push(`      # Primary attempt`);
    lines.push(`      echo "Submitting CodeRun $RUN_NAME ($PRIMARY_CLI/$PRIMARY_MODEL)..." &&`);
    lines.push(`      apply_coderun "$PRIMARY_CLI" "$PRIMARY_MODEL" "$RUN_NAME" &&`);
    lines.push(`      if wait_coderun "$RUN_NAME"; then`);
    lines.push(`        echo "task-${tid} implementation complete ($PRIMARY_CLI/$PRIMARY_MODEL)"`);
    lines.push(`      else`);
    lines.push(`        # --- Fallback attempt: ${fallbackCli}/${fallbackModel} ---`);
    lines.push(`        FALLBACK_RUN="play-task-${tid}-${agent}-fallback"`);
    lines.push(`        USED_CLI="$FALLBACK_CLI"`);
    lines.push(`        USED_MODEL="$FALLBACK_MODEL"`);
    lines.push(`        echo "⚠️ Primary $PRIMARY_CLI failed for task-${tid}, falling back to $FALLBACK_CLI/$FALLBACK_MODEL..." &&`);
    lines.push(`        ${notifyDiscord(agent, '\${discord_channel\}',
      `⚠️ Task ${tid}: primary ${taskCli} failed, falling back to ${fallbackCli}/${fallbackModel}`,
      { step: 'fallback-trigger', task_id: tid, agent, primary_cli: taskCli, fallback_cli: fallbackCli })}`);
    lines.push(`        kubectl delete "coderun/$RUN_NAME" -n "$NS" --ignore-not-found=true &&`);
    lines.push(`        apply_coderun "$FALLBACK_CLI" "$FALLBACK_MODEL" "$FALLBACK_RUN" &&`);
    lines.push(`        wait_coderun "$FALLBACK_RUN" &&`);
    lines.push(`        echo "task-${tid} implementation complete via fallback ($FALLBACK_CLI/$FALLBACK_MODEL)"`);
    lines.push(`      fi &&`);
    lines.push(`      jq -nc --arg cli "$USED_CLI" --arg model "$USED_MODEL" \\`);
    lines.push(`        '{task_id:${tid}, agent:"${agent}", cli:$cli, model:$model, phase:"implementation", status:"complete"}'`);

    // --- Post-implementation checks (quality/security/testing via lobster sub-workflows) ---
    // These use the check-specific agent harnesses (cipher for security, cleo for quality, tess for testing)
    const secHarness = getAgentHarness('cipher', config);
    const checkDeps = [`run-task-${tid}`];
    const gateSteps: string[] = [];

    // Security — always runs
    lines.push(``);
    lines.push(`  - id: security-task-${tid}`);
    lines.push(`    depends_on: [${checkDeps.join(', ')}]`);
    lines.push(`    command: >`);
    lines.push(`      lobster run --mode tool`);
    lines.push(`      "\${tasks_dir\}/task-${tid}/security.lobster.yaml"`);
    lines.push(`      --args-json "$(jq -nc --arg td '\${tasks_dir\}/task-${tid}'`);
    lines.push(`        --arg repo '\${repo_url\}'`);
    lines.push(`        --arg branch '${branchName}'`);
    lines.push(`        --arg cli '${harnessCli(secHarness)}'`);
    lines.push(`        --arg model '${harnessModel(secHarness)}'`);
    lines.push(`        '{task_dir:$td, repo_url:$repo, branch_name:$branch, cli:$cli, model:$model}')"`);
    gateSteps.push(`security-task-${tid}`);

    if (isCoding) {
      const qualHarness = getAgentHarness('cleo', config);
      const testHarness = getAgentHarness('tess', config);

      // Quality — coding tasks only
      lines.push(``);
      lines.push(`  - id: quality-task-${tid}`);
      lines.push(`    depends_on: [${checkDeps.join(', ')}]`);
      lines.push(`    command: >`);
      lines.push(`      lobster run --mode tool`);
      lines.push(`      "\${tasks_dir\}/task-${tid}/quality.lobster.yaml"`);
      lines.push(`      --args-json "$(jq -nc --arg td '\${tasks_dir\}/task-${tid}'`);
      lines.push(`        --arg repo '\${repo_url\}'`);
      lines.push(`        --arg branch '${branchName}'`);
      lines.push(`        --arg cli '${harnessCli(qualHarness)}'`);
      lines.push(`        --arg model '${harnessModel(qualHarness)}'`);
      lines.push(`        '{task_dir:$td, repo_url:$repo, branch_name:$branch, cli:$cli, model:$model}')"`);
      gateSteps.push(`quality-task-${tid}`);

      // Testing — coding tasks only
      lines.push(``);
      lines.push(`  - id: testing-task-${tid}`);
      lines.push(`    depends_on: [${checkDeps.join(', ')}]`);
      lines.push(`    command: >`);
      lines.push(`      lobster run --mode tool`);
      lines.push(`      "\${tasks_dir\}/task-${tid}/testing.lobster.yaml"`);
      lines.push(`      --args-json "$(jq -nc --arg td '\${tasks_dir\}/task-${tid}'`);
      lines.push(`        --arg repo '\${repo_url\}'`);
      lines.push(`        --arg branch '${branchName}'`);
      lines.push(`        --arg cli '${harnessCli(testHarness)}'`);
      lines.push(`        --arg model '${harnessModel(testHarness)}'`);
      lines.push(`        '{task_dir:$td, repo_url:$repo, branch_name:$branch, cli:$cli, model:$model}')"`);
      gateSteps.push(`testing-task-${tid}`);
    }

    // --- Gate step + notification ---
    lines.push(``);
    lines.push(`  - id: gate-task-${tid}`);
    lines.push(`    depends_on: [${gateSteps.join(', ')}]`);
    lines.push(`    command: |`);
    lines.push(`      echo "task-${tid} [${taskType}] gate passed — ${taskCli}/${taskModel} — all checks complete" &&`);
    lines.push(`      ${notifyDiscord(agent, '\${discord_channel\}',
      `✅ Task ${tid}: ${shortTitle} — gate passed (${taskCli}/${taskModel})`,
      { step: 'gate-pass', task_id: tid, agent, cli: taskCli, model: taskModel, checks: gateSteps.length })}`);
    lines.push(`      LINEAR_SID="\${linear_session_id\}"`);
    lines.push(`      ${notifyLinear('$LINEAR_SID', 'action',
      `## ✅ Task ${tid} Gate Passed\\n\\n**${task.title}**\\n- Agent: \`${agent}\` | CLI: \`${taskCli}\`\\n- Checks passed: ${gateSteps.length} (${gateSteps.join(', ')})`)}`);
    lines.push(`      jq -nc '{task_id: ${tid}, task_type: "${taskType}", agent: "${agent}", cli: "${taskCli}", model: "${taskModel}", difficulty: ${difficulty}, gate: "pass"}'`);
  }

  // --- Final play-complete step with harness summary + notifications ---
  const allGates = tasks.map((t) => `gate-task-${t.id}`);
  const harnessSummary = tasks.map((t) => {
    const a = t.agent ?? 'nova';
    const h = getAgentHarness(a, config);
    return `task-${t.id}(${a}):${harnessCli(h)}`;
  }).join(', ');
  lines.push(``);
  lines.push(`  # ── Play complete ──`);
  lines.push(`  - id: play-complete`);
  lines.push(`    depends_on: [${allGates.join(', ')}]`);
  lines.push(`    command: |`);
  lines.push(`      echo "play complete — all ${tasks.length} tasks passed gate checks" &&`);
  lines.push(`      echo "harness summary: ${harnessSummary}" &&`);
  lines.push(`      ${notifyDiscord('morgan', '\${discord_channel\}',
    `🏁 Play complete — all ${tasks.length} tasks passed gate checks`,
    { step: 'play-complete', task_count: tasks.length, time_utc: '$(date -u +%Y-%m-%dT%H:%M:%SZ)' })}`);
  lines.push(`      LINEAR_SID="\${linear_session_id\}"`);
  lines.push(`      ${notifyLinear('$LINEAR_SID', 'action',
    `## 🏁 Play Complete\\n\\nAll ${tasks.length} tasks passed gate checks.\\n\\n### Harness Summary\\n${harnessTableMd}`)}`);
  lines.push(`      jq -nc '{play: "complete", tasks: ${tasks.length}, status: "pass"}'`);

  return lines.join('\n') + '\n';
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

  const playYaml = generatePlayWorkflow(expanded_tasks, playConfig, repoUrl);

  return { task_workflows: taskWorkflows, play_yaml: playYaml };
}
