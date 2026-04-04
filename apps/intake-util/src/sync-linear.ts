/**
 * Linear sync utilities for the intake pipeline.
 *
 * Two operations:
 *   init   — Create a Linear project + PRD issue at pipeline start.
 *   issues — Create task/subtask issues after docs/prompts are generated.
 *
 * Uses direct fetch to Linear GraphQL API (LINEAR_API_KEY env var).
 */

import fs from 'node:fs';
import path from 'node:path';
import type { GeneratedTask } from './types';

const LINEAR_API_URL = 'https://api.linear.app/graphql';

// =============================================================================
// Types
// =============================================================================

interface GraphQLResponse<T> {
  data?: T;
  errors?: Array<{ message: string }>;
}

interface LinearProject {
  id: string;
  name: string;
  url: string;
}

interface LinearIssue {
  id: string;
  identifier: string;
  title: string;
  url: string;
}

interface LinearLabel {
  id: string;
  name: string;
}

interface TeamMember {
  id: string;
  name: string;
  displayName: string;
}

export interface InitResult {
  projectId: string;
  projectUrl: string;
  prdIssueId: string;
  prdIdentifier: string;
  teamId: string;
  agentMap: Record<string, string>;
}

export interface SyncIssueEntry {
  taskId: number;
  linearId: string;
  identifier: string;
  subtaskIssues: Array<{
    subtaskId: number;
    linearId: string;
    identifier: string;
  }>;
}

export interface SyncIssuesResult {
  issueCount: number;
  parentIssueId?: string;
  parentIssueIdentifier?: string;
  taskIssueCount?: number;
  subtaskIssueCount?: number;
  assignedIssueCount?: number;
  unassignedIssueCount?: number;
  unresolvedAgents?: string[];
  issues: SyncIssueEntry[];
}

// =============================================================================
// GraphQL Executor
// =============================================================================

async function execute<T>(apiKey: string, query: string, variables: Record<string, unknown> = {}): Promise<T> {
  const authHeader = apiKey.startsWith('lin_api_') ? apiKey : `Bearer ${apiKey}`;

  const response = await fetch(LINEAR_API_URL, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: authHeader,
    },
    body: JSON.stringify({ query, variables }),
  });

  if (!response.ok) {
    const body = await response.text();
    throw new Error(`Linear API returned ${response.status}: ${body}`);
  }

  const json = (await response.json()) as GraphQLResponse<T>;

  if (json.errors?.length) {
    const messages = json.errors.map((e) => e.message).join(', ');
    throw new Error(`GraphQL errors: ${messages}`);
  }

  if (!json.data) {
    throw new Error('No data in GraphQL response');
  }

  return json.data;
}

// =============================================================================
// Team Key → UUID Resolution
// =============================================================================

/**
 * Resolve a team identifier to a UUID.
 * If it looks like a UUID already, return it as-is.
 * Otherwise, query all teams and match by key (e.g., "CTOPA").
 */
export async function resolveTeamId(apiKey: string, teamIdOrKey: string): Promise<string> {
  // UUIDs contain hyphens and are 36 chars; team keys are short alphanumeric
  if (teamIdOrKey.includes('-') && teamIdOrKey.length > 20) {
    return teamIdOrKey; // already a UUID
  }

  interface TeamsResponse {
    teams: { nodes: Array<{ id: string; key: string; name: string }> };
  }

  const data = await execute<TeamsResponse>(
    apiKey,
    `query { teams { nodes { id key name } } }`,
  );

  const team = data.teams.nodes.find(
    (t) => t.key.toLowerCase() === teamIdOrKey.toLowerCase(),
  );

  if (!team) {
    const available = data.teams.nodes.map((t) => `${t.key} (${t.name})`).join(', ');
    throw new Error(`Team key "${teamIdOrKey}" not found. Available teams: ${available}`);
  }

  return team.id;
}

// =============================================================================
// Label Helpers
// =============================================================================

/** Cache of team labels to avoid re-fetching per label lookup. */
const teamLabelCache = new Map<string, LinearLabel[]>();

async function fetchTeamLabels(apiKey: string, teamId: string): Promise<LinearLabel[]> {
  if (teamLabelCache.has(teamId)) return teamLabelCache.get(teamId)!;

  interface FindResponse {
    team: { labels: { nodes: LinearLabel[] } };
  }

  const findData = await execute<FindResponse>(
    apiKey,
    `query GetTeamLabels($teamId: String!) {
      team(id: $teamId) {
        labels { nodes { id name } }
      }
    }`,
    { teamId },
  );

  const labels = findData.team.labels.nodes;
  teamLabelCache.set(teamId, labels);
  return labels;
}

/**
 * Find an existing label by name, or try to create it.
 * Returns the label ID, or null if the label doesn't exist and creation is forbidden.
 */
async function getOrCreateLabel(apiKey: string, teamId: string, name: string): Promise<string | null> {
  const labels = await fetchTeamLabels(apiKey, teamId);
  const existing = labels.find((l) => l.name.toLowerCase() === name.toLowerCase());
  if (existing) return existing.id;

  // Try to create — some OAuth tokens lack label-create permission
  try {
    interface CreateResponse {
      issueLabelCreate: { success: boolean; issueLabel?: LinearLabel };
    }

    const createData = await execute<CreateResponse>(
      apiKey,
      `mutation CreateLabel($input: IssueLabelCreateInput!) {
        issueLabelCreate(input: $input) {
          success
          issueLabel { id name }
        }
      }`,
      { input: { teamId, name } },
    );

    if (createData.issueLabelCreate.issueLabel) {
      // Add to cache so subsequent lookups find it
      teamLabelCache.get(teamId)?.push(createData.issueLabelCreate.issueLabel);
      return createData.issueLabelCreate.issueLabel.id;
    }
  } catch {
    // Label creation forbidden — fall through
  }

  return null;
}

// =============================================================================
// Agent Lookup
// =============================================================================

async function fetchWorkspaceUsers(apiKey: string): Promise<Record<string, string>> {
  interface Response {
    users: { nodes: TeamMember[] };
  }

  // Query all workspace users (not just team members) so agent bots are found
  const data = await execute<Response>(
    apiKey,
    `query { users { nodes { id name displayName } } }`,
  );

  const map: Record<string, string> = {};
  for (const member of data.users.nodes) {
    // Match by lowercase name: "bolt" matches "Bolt" or "5DLabs-Bolt"
    const name = member.name.toLowerCase();
    const displayName = member.displayName.toLowerCase();

    // Direct name match
    map[name] = member.id;
    map[displayName] = member.id;

    // Strip "5dlabs-" or "5d-labs-" prefix for matching
    for (const prefix of ['5dlabs-', '5d-labs-']) {
      if (name.startsWith(prefix)) map[name.slice(prefix.length)] = member.id;
      if (displayName.startsWith(prefix)) map[displayName.slice(prefix.length)] = member.id;
    }
  }

  return map;
}

// =============================================================================
// Init: Create project + PRD issue
// =============================================================================

export interface InitOptions {
  projectName: string;
  teamId: string;
  prdContent: string;
  apiKey: string;
}

export async function createProjectAndPrdIssue(opts: InitOptions): Promise<InitResult> {
  const { projectName, prdContent, apiKey } = opts;

  // Resolve team key (e.g., "CTOPA") → UUID
  const teamId = await resolveTeamId(apiKey, opts.teamId);

  // 1. Create project
  interface ProjectResponse {
    projectCreate: { success: boolean; project?: LinearProject };
  }

  const projectData = await execute<ProjectResponse>(
    apiKey,
    `mutation CreateProject($input: ProjectCreateInput!) {
      projectCreate(input: $input) {
        success
        project { id name url }
      }
    }`,
    { input: { name: projectName, teamIds: [teamId] } },
  );

  if (!projectData.projectCreate.success || !projectData.projectCreate.project) {
    throw new Error('Failed to create Linear project');
  }

  const project = projectData.projectCreate.project;

  // 1b. Create a board (Kanban) view scoped to this project
  try {
    interface CustomViewResponse {
      customViewCreate: { success: boolean; customView?: { id: string; name: string } };
    }
    const viewData = await execute<CustomViewResponse>(
      apiKey,
      `mutation CreateView($input: CustomViewCreateInput!) {
        customViewCreate(input: $input) {
          success
          customView { id name }
        }
      }`,
      {
        input: {
          name: `${projectName} — Board`,
          teamId,
          projectId: project.id,
          filterData: { project: { id: { eq: project.id } } },
          shared: true,
          modelName: 'customView',
        },
      },
    );
    if (viewData.customViewCreate.success && viewData.customViewCreate.customView) {
      const viewId = viewData.customViewCreate.customView.id;
      console.error(`Created board view: ${viewData.customViewCreate.customView.name} (${viewId})`);

      // Set layout to "board" (Kanban) with useful field visibility
      await execute(
        apiKey,
        `mutation SetViewPrefs($input: ViewPreferencesCreateInput!) {
          viewPreferencesCreate(input: $input) { success }
        }`,
        {
          input: {
            customViewId: viewId,
            type: 'organization',
            preferences: {
              layout: 'board',
              issueGrouping: 'status',
              showSubIssues: true,
              fieldAssignee: true,
              fieldPriority: true,
              fieldLabels: true,
              fieldEstimate: true,
            },
          },
        },
      );
    }
  } catch (err) {
    console.error(`Warning: failed to create project board view: ${err}`);
  }

  // 2. Get/create labels
  const [intakeLabelId, prdLabelId] = await Promise.all([
    getOrCreateLabel(apiKey, teamId, 'intake'),
    getOrCreateLabel(apiKey, teamId, 'prd'),
  ]);

  // 3. Fetch agent map
  const agentMap = await fetchWorkspaceUsers(apiKey);

  // 4. Create PRD issue (assigned to Morgan)
  const morganId = agentMap['morgan'] || undefined;

  interface IssueResponse {
    issueCreate: { success: boolean; issue?: LinearIssue };
  }

  const prdLabelIds = [intakeLabelId, prdLabelId].filter((id): id is string => id !== null);

  const issueInput: Record<string, unknown> = {
    title: `PRD: ${projectName}`,
    description: prdContent,
    teamId,
    projectId: project.id,
  };
  if (prdLabelIds.length > 0) issueInput.labelIds = prdLabelIds;
  if (morganId) issueInput.assigneeId = morganId;

  const issueData = await execute<IssueResponse>(
    apiKey,
    `mutation CreateIssue($input: IssueCreateInput!) {
      issueCreate(input: $input) {
        success
        issue { id identifier title url }
      }
    }`,
    { input: issueInput },
  );

  if (!issueData.issueCreate.success || !issueData.issueCreate.issue) {
    throw new Error('Failed to create PRD issue');
  }

  const prdIssue = issueData.issueCreate.issue;

  return {
    projectId: project.id,
    projectUrl: project.url,
    prdIssueId: prdIssue.id,
    prdIdentifier: prdIssue.identifier,
    teamId,
    agentMap,
  };
}

// =============================================================================
// Issues: Create task + subtask issues
// =============================================================================

export interface SyncIssuesOptions {
  tasks: GeneratedTask[];
  projectId: string;
  prdIssueId: string;
  teamId: string;
  baseUrl: string;
  prUrl?: string;
  agentMap: Record<string, string>;
  apiKey: string;
  /** Personal API key that can assign to app users (lin_api_* prefix). */
  personalApiKey?: string;
  /** PM server URL for per-agent OAuth tokens (enables self-assignment). */
  pmUrl?: string;
}

/**
 * Fetch an agent's Linear OAuth token from the PM server.
 * Returns the token string or null if unavailable.
 */
async function fetchAgentToken(pmUrl: string, agent: string): Promise<string | null> {
  try {
    const resp = await fetch(`${pmUrl}/oauth/token/${agent}`, { signal: AbortSignal.timeout(10_000) });
    if (!resp.ok) {
      console.error(`fetchAgentToken: ${agent} → HTTP ${resp.status}`);
      return null;
    }
    const data = await resp.json() as { status: string; access_token?: string };
    if (data.status === 'ok' && data.access_token) {
      return data.access_token;
    }
    console.error(`fetchAgentToken: ${agent} → status=${data.status}, no token`);
    return null;
  } catch (err) {
    console.error(`fetchAgentToken: ${agent} → ${err}`);
    return null;
  }
}

function extractAgent(task: GeneratedTask): string {
  const normalize = (value: string): string => {
    return value
      .trim()
      .toLowerCase()
      .replace(/^5d-labs-/, '')
      .replace(/^5dlabs-/, '')
      .replace(/[^a-z0-9_-]/g, '');
  };

  if (task.agent && task.agent.trim().length > 0) {
    const normalized = normalize(task.agent);
    if (normalized.length > 0) return normalized;
  }

  // Common title formats:
  //   [Bolt] Do thing
  //   Bolt: Do thing
  //   (Bolt - ...) Do thing
  const bracketMatch = task.title.match(/^\[([^\]]+)\]/);
  if (bracketMatch?.[1]) {
    const normalized = normalize(bracketMatch[1]);
    if (normalized.length > 0) return normalized;
  }

  const colonMatch = task.title.match(/^([A-Za-z0-9_-]+)\s*:/);
  if (colonMatch?.[1]) {
    const normalized = normalize(colonMatch[1]);
    if (normalized.length > 0) return normalized;
  }

  const parenMatch = task.title.match(/\(([A-Za-z0-9_-]+)\s*-/);
  if (parenMatch?.[1]) {
    const normalized = normalize(parenMatch[1]);
    if (normalized.length > 0) return normalized;
  }

  return 'morgan';
}

/**
 * Build the task issue body as acceptance criteria:
 * description, details, test strategy, subtask checklist, dependencies.
 */
function readDocSnippet(filePath: string, maxChars: number): string {
  try {
    const content = fs.readFileSync(filePath, 'utf-8').trim();
    if (!content) return '';
    if (content.length <= maxChars) return content;
    return `${content.slice(0, maxChars)}\n\n...[truncated]`;
  } catch {
    return '';
  }
}

function buildTaskDescription(task: GeneratedTask, baseUrl: string, prUrl: string): string {
  const lines: string[] = [];

  // Acceptance criteria header
  lines.push(`## Description\n${task.description}`);

  if (task.details) {
    lines.push('', `## Details\n${task.details}`);
  }

  if (task.test_strategy || task.testStrategy) {
    lines.push('', `## Testing Strategy\n${task.test_strategy || task.testStrategy}`);
  }

  // Subtask checklist (acceptance view)
  if (task.subtasks && task.subtasks.length > 0) {
    lines.push('', '## Subtasks');
    for (const st of task.subtasks) {
      lines.push(`- [ ] ${st.title}`);
    }
  }

  if (task.dependencies.length > 0) {
    lines.push('', `**Blocked by:** tasks ${task.dependencies.join(', ')}`);
  }

  // Decision points
  if (task.decision_points && task.decision_points.length > 0) {
    lines.push('', '## Decision Points');
    for (const dp of task.decision_points) {
      const approval = dp.requires_approval || dp.requiresApproval ? ' ⚠️ requires approval' : '';
      lines.push(`- **${dp.description}** (${dp.category})${approval}`);
      const opts = Array.isArray(dp.options) ? dp.options.join(', ') : (dp.options ?? 'none specified');
      lines.push(`  Options: ${opts}`);
    }
  }

  // Links to generated docs
  if (baseUrl) {
    lines.push('', '---', '**Generated docs:**');
    lines.push(`- [prompt.md](${baseUrl}/.tasks/docs/task-${task.id}/prompt.md)`);
    lines.push(`- [acceptance.md](${baseUrl}/.tasks/docs/task-${task.id}/acceptance.md)`);
    lines.push(`- [task.md](${baseUrl}/.tasks/docs/task-${task.id}/task.md)`);
  }

  if (prUrl) {
    lines.push('', `**PR:** ${prUrl}`);
  }

  const taskDocPath = path.join('.tasks', 'docs', `task-${task.id}`, 'task.md');
  const acceptanceDocPath = path.join('.tasks', 'docs', `task-${task.id}`, 'acceptance.md');
  const promptDocPath = path.join('.tasks', 'docs', `task-${task.id}`, 'prompt.md');

  const taskDoc = readDocSnippet(taskDocPath, 4000);
  if (taskDoc) {
    lines.push('', '## Task Document (embedded)', '```markdown', taskDoc, '```');
  }
  const acceptanceDoc = readDocSnippet(acceptanceDocPath, 3000);
  if (acceptanceDoc) {
    lines.push('', '## Acceptance Criteria (embedded)', '```markdown', acceptanceDoc, '```');
  }
  const promptDoc = readDocSnippet(promptDocPath, 2500);
  if (promptDoc) {
    lines.push('', '## Prompt (embedded excerpt)', '```markdown', promptDoc, '```');
  }

  return lines.join('\n');
}

/**
 * Build the subtask issue body as the implementation prompt.
 */
function buildSubtaskDescription(
  subtask: { id: number; title: string; description: string; details?: string; test_strategy?: string; testStrategy?: string },
  taskId: number,
  baseUrl: string,
  parentTask: GeneratedTask,
  prUrl: string,
): string {
  const lines: string[] = [];

  lines.push(`## What to Build\n${subtask.description}`);

  if (subtask.details) {
    lines.push('', `## Implementation Details\n${subtask.details}`);
  }

  const testStrategy = subtask.test_strategy || subtask.testStrategy;
  if (testStrategy) {
    lines.push('', `## Testing\n${testStrategy}`);
  }

  lines.push('', `**Parent task:** ${parentTask.title}`);

  if (baseUrl) {
    lines.push('', '---');
    lines.push(`**Prompt:** [prompt.md](${baseUrl}/.tasks/docs/task-${taskId}/subtasks/task-${taskId}.${subtask.id}/prompt.md)`);
  }
  if (prUrl) {
    lines.push(`**PR:** ${prUrl}`);
  }

  const subtaskPromptPath = path.join('.tasks', 'docs', `task-${taskId}`, 'subtasks', `task-${taskId}.${subtask.id}`, 'prompt.md');
  const subtaskPrompt = readDocSnippet(subtaskPromptPath, 2500);
  if (subtaskPrompt) {
    lines.push('', '## Prompt (embedded excerpt)', '```markdown', subtaskPrompt, '```');
  }

  return lines.join('\n');
}

const PRIORITY_MAP: Record<string, number> = {
  high: 1,    // Urgent in Linear
  medium: 2,  // High in Linear
  low: 3,     // Medium in Linear
};

export async function syncTaskIssues(opts: SyncIssuesOptions): Promise<SyncIssuesResult> {
  const { tasks, projectId, prdIssueId, baseUrl, prUrl, agentMap, apiKey, personalApiKey, pmUrl } = opts;

  // ── Agent Delegation Model ──
  // Linear's agent model uses "delegate" (not "assignee") for app users:
  //   - issueCreate(input: { assigneeId: <app-user-id> }) sets the DELEGATE field
  //   - The assignee field is set to the token owner (human)
  //   - This is the intended behavior: humans maintain ownership, agents act on their behalf
  // Requirements:
  //   - Each agent app must have app:assignable scope (enabled via client_credentials token grant)
  //   - A personal API key (lin_api_*) is the simplest way to create issues with delegation
  //   - Agent self-assignment via OAuth tokens also works but is more complex
  //
  // Priority: personalApiKey > PM per-agent tokens > default apiKey
  const assignApiKey = personalApiKey || apiKey;
  if (personalApiKey) {
    console.error(`syncTaskIssues: using personal API key for agent delegation`);
  }

  const mapKeys = Object.keys(agentMap);
  console.error(`syncTaskIssues: agentMap has ${mapKeys.length} entries, tasks: ${tasks.length}`);
  if (mapKeys.length > 0) {
    console.error(`syncTaskIssues: agentMap sample keys: ${mapKeys.slice(0, 8).join(', ')}`);
  }
  const taskAgents = tasks.map(t => extractAgent(t));
  const uniqueAgents = [...new Set(taskAgents)];
  console.error(`syncTaskIssues: unique agents in tasks: ${uniqueAgents.join(', ')}`);
  for (const a of uniqueAgents) {
    console.error(`syncTaskIssues:   ${a} → ${agentMap[a] ? 'delegate:' + agentMap[a].slice(0, 8) + '...' : 'UNRESOLVED (no delegation)'}`);
  }

  // Pre-fetch per-agent tokens from PM server for self-assignment (fallback if no personal key).
  const agentTokens = new Map<string, string>();
  if (pmUrl && !personalApiKey) {
    console.error(`syncTaskIssues: fetching per-agent tokens from ${pmUrl}`);
    const tokenPromises = uniqueAgents.map(async (agent) => {
      const token = await fetchAgentToken(pmUrl, agent);
      if (token) {
        agentTokens.set(agent, token);
      }
    });
    await Promise.all(tokenPromises);
    console.error(`syncTaskIssues: got tokens for ${agentTokens.size}/${uniqueAgents.length} agents`);
  }

  // Resolve team key (e.g., "CTOPA") → UUID
  const teamId = await resolveTeamId(apiKey, opts.teamId);

  // Pre-fetch/create labels we'll need
  const intakeLabelId = await getOrCreateLabel(apiKey, teamId, 'intake');

  // Cache agent-name labels
  const agentLabelCache = new Map<string, string | null>();

  async function getAgentLabelId(agentName: string): Promise<string | null> {
    if (agentLabelCache.has(agentName)) return agentLabelCache.get(agentName)!;
    const id = await getOrCreateLabel(apiKey, teamId, agentName);
    agentLabelCache.set(agentName, id);
    return id;
  }

  // Create one parent issue so Linear always has a visible root task
  // with child tasks beneath it, even when subtasks are sparse.
  interface IssueResponse {
    issueCreate: { success: boolean; issue?: LinearIssue };
  }

  const parentInput: Record<string, unknown> = {
    title: 'Main Implementation Task',
    description: [
      'Parent issue generated by intake.',
      '',
      'Child issues under this task represent the generated implementation task breakdown.',
      `PRD issue id: ${prdIssueId}`,
    ].join('\n'),
    teamId,
    projectId,
  };
  if (intakeLabelId) parentInput.labelIds = [intakeLabelId];

  const parentIssueResp = await execute<IssueResponse>(
    apiKey,
    `mutation CreateIssue($input: IssueCreateInput!) {
      issueCreate(input: $input) {
        success
        issue { id identifier title url }
      }
    }`,
    { input: parentInput },
  );
  if (!parentIssueResp.issueCreate.success || !parentIssueResp.issueCreate.issue) {
    throw new Error('Failed to create parent implementation issue');
  }
  const parentIssue = parentIssueResp.issueCreate.issue;

  const issues: SyncIssueEntry[] = [];
  const unresolvedAgents = new Set<string>();
  let subtaskIssueCount = 0;
  let delegatedIssueCount = 0;
  let undelegatedIssueCount = 0;

  for (const task of tasks) {
    const agent = extractAgent(task);
    const agentLabelId = await getAgentLabelId(agent);
    const assigneeId = agentMap[agent] || undefined;
    // Use personal API key for assignment (can assign to app users),
    // or fall back to agent's own token, or default key.
    const agentToken = agentTokens.get(agent);
    const issueApiKey = personalApiKey || agentToken || apiKey;

    if (!assigneeId) {
      unresolvedAgents.add(agent);
      undelegatedIssueCount += 1;
    } else {
      delegatedIssueCount += 1;
    }

    const labelIds = [intakeLabelId, agentLabelId].filter((id): id is string => id !== null);

    const issueInput: Record<string, unknown> = {
      title: `[${agent}] ${task.title}`,
      description: buildTaskDescription(task, baseUrl, prUrl ?? ''),
      teamId,
      projectId,
      parentId: parentIssue.id,
    };
    if (labelIds.length > 0) issueInput.labelIds = labelIds;
    if (assigneeId) issueInput.assigneeId = assigneeId;
    if (task.priority && PRIORITY_MAP[task.priority]) {
      issueInput.priority = PRIORITY_MAP[task.priority];
    }

    // Helper: create issue using the agent's token for self-assignment,
    // falling back to default apiKey without assignee if assignment fails.
    async function createIssueWithFallback(input: Record<string, unknown>): Promise<LinearIssue> {
      try {
        const data = await execute<IssueResponse>(
          issueApiKey,
          `mutation CreateIssue($input: IssueCreateInput!) {
            issueCreate(input: $input) {
              success
              issue { id identifier title url }
            }
          }`,
          { input },
        );
        if (data.issueCreate.success && data.issueCreate.issue) {
          return data.issueCreate.issue;
        }
      } catch (err) {
        // If assignment failed ("App user not valid"), retry without assignee using default key
        if (input.assigneeId && String(err).includes('not valid')) {
          console.error(`syncTaskIssues: assignment failed for ${agent}, retrying without assignee`);
          const { assigneeId: _, ...withoutAssignee } = input;
          const retry = await execute<IssueResponse>(
            apiKey,
            `mutation CreateIssue($input: IssueCreateInput!) {
              issueCreate(input: $input) {
                success
                issue { id identifier title url }
              }
            }`,
            { input: withoutAssignee },
          );
          if (retry.issueCreate.success && retry.issueCreate.issue) {
            return retry.issueCreate.issue;
          }
        }
        throw err;
      }
      throw new Error('Failed to create issue');
    }

    const taskIssue = await createIssueWithFallback(issueInput);
    const subtaskIssues: SyncIssueEntry['subtaskIssues'] = [];

    // Create subtask child issues
    if (task.subtasks && task.subtasks.length > 0) {
      for (const subtask of task.subtasks) {
        const subtaskInput: Record<string, unknown> = {
          title: subtask.title,
          description: buildSubtaskDescription(subtask, task.id, baseUrl, task, prUrl ?? ''),
          teamId,
          projectId,
          parentId: taskIssue.id,
        };
        if (intakeLabelId) subtaskInput.labelIds = [intakeLabelId];
        if (assigneeId) {
          subtaskInput.assigneeId = assigneeId;
          delegatedIssueCount += 1;
        } else {
          undelegatedIssueCount += 1;
        }

        const subtaskIssue = await createIssueWithFallback(subtaskInput);
        subtaskIssueCount += 1;

        subtaskIssues.push({
          subtaskId: subtask.id,
          linearId: subtaskIssue.id,
          identifier: subtaskIssue.identifier,
        });
      }
    }

    issues.push({
      taskId: task.id,
      linearId: taskIssue.id,
      identifier: taskIssue.identifier,
      subtaskIssues,
    });
  }

  return {
    issueCount: issues.length + 1,
    parentIssueId: parentIssue.id,
    parentIssueIdentifier: parentIssue.identifier,
    taskIssueCount: issues.length,
    subtaskIssueCount,
    assignedIssueCount: delegatedIssueCount,
    unassignedIssueCount: undelegatedIssueCount,
    unresolvedAgents: [...unresolvedAgents].sort(),
    issues,
  };
}
