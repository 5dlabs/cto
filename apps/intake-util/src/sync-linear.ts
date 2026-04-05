/**
 * Linear sync utilities for the intake pipeline.
 *
 * Four operations:
 *   init         — Create a Linear project + PRD issue at pipeline start.
 *   issues       — Create task/subtask issues after docs/prompts are generated.
 *   rewrite-urls — Bulk-replace base URL in all project issue descriptions.
 *   github-sync  — Create GitHub issues mirroring each Linear issue (1:1 mapping).
 *
 * Uses direct fetch to Linear GraphQL API (LINEAR_API_KEY env var).
 */

import fs from 'node:fs';
import path from 'node:path';
import { execSync, execFileSync } from 'node:child_process';
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

const THROTTLE_MS = 500; // minimum gap between Linear API calls
let lastCallAt = 0;

async function execute<T>(apiKey: string, query: string, variables: Record<string, unknown> = {}): Promise<T> {
  const authHeader = apiKey.startsWith('lin_api_') ? apiKey : `Bearer ${apiKey}`;
  const MAX_RETRIES = 8;

  for (let attempt = 0; attempt <= MAX_RETRIES; attempt++) {
    // Throttle: enforce minimum gap between API calls
    const now = Date.now();
    const elapsed = now - lastCallAt;
    if (elapsed < THROTTLE_MS) {
      await new Promise((r) => setTimeout(r, THROTTLE_MS - elapsed));
    }
    lastCallAt = Date.now();

    const response = await fetch(LINEAR_API_URL, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: authHeader,
      },
      body: JSON.stringify({ query, variables }),
    });

    // Rate-limit / usage-limit: back off and retry
    if (response.status === 429 || response.status === 503) {
      const retryAfter = parseInt(response.headers.get('retry-after') ?? '', 10);
      const waitMs = (retryAfter > 0 ? retryAfter : Math.min(2 ** attempt * 5, 60)) * 1000;
      console.error(`Linear API ${response.status} — retrying in ${waitMs / 1000}s (attempt ${attempt + 1}/${MAX_RETRIES})`);
      await new Promise((r) => setTimeout(r, waitMs));
      continue;
    }

    if (!response.ok) {
      const body = await response.text();
      throw new Error(`Linear API returned ${response.status}: ${body}`);
    }

    const json = (await response.json()) as GraphQLResponse<T>;

    if (json.errors?.length) {
      const messages = json.errors.map((e) => e.message).join(', ');
      // Retry on usage/rate limit errors surfaced as GraphQL errors
      if (messages.includes('usage limit') || messages.includes('rate limit')) {
        const waitMs = Math.min(2 ** attempt * 5, 60) * 1000;
        console.error(`Linear GraphQL limit hit — retrying in ${waitMs / 1000}s (attempt ${attempt + 1}/${MAX_RETRIES})`);
        await new Promise((r) => setTimeout(r, waitMs));
        continue;
      }
      throw new Error(`GraphQL errors: ${messages}`);
    }

    if (!json.data) {
      throw new Error('No data in GraphQL response');
    }

    return json.data;
  }

  throw new Error('Linear API: max retries exceeded due to rate/usage limits');
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

  // 1. Find existing project by name, or create a new one
  interface ProjectResponse {
    projectCreate: { success: boolean; project?: LinearProject };
  }
  interface ProjectSearchResponse {
    projects: { nodes: LinearProject[] };
  }
  interface ProjectIssueCountResponse {
    project: { issues: { nodes: { id: string }[] } };
  }

  let project: LinearProject;

  const existingData = await execute<ProjectSearchResponse>(
    apiKey,
    `query FindProject($name: String!) {
      projects(filter: { name: { eq: $name } }) {
        nodes { id name url }
      }
    }`,
    { name: projectName },
  );

  if (existingData.projects.nodes.length > 0) {
    // If multiple projects share the same name, pick the one with the most issues
    if (existingData.projects.nodes.length === 1) {
      project = existingData.projects.nodes[0];
    } else {
      let best = existingData.projects.nodes[0];
      let bestCount = 0;
      for (const candidate of existingData.projects.nodes) {
        try {
          const countData = await execute<ProjectIssueCountResponse>(apiKey,
            `query($id: String!) { project(id: $id) { issues { nodes { id } } } }`,
            { id: candidate.id });
          const count = countData.project.issues.nodes.length;
          if (count > bestCount) { best = candidate; bestCount = count; }
        } catch { /* use first as fallback */ }
      }
      project = best;
    }
    console.error(`Reusing existing Linear project: ${project.name} (${project.id})`);
  } else {
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
    project = projectData.projectCreate.project;
    console.error(`Created new Linear project: ${project.name} (${project.id})`);
  }

  // 1b. Create custom views scoped to this project
  interface CustomViewResponse {
    customViewCreate: { success: boolean; customView?: { id: string; name: string } };
  }
  const VIEW_MUTATION = `mutation CreateView($input: CustomViewCreateInput!) {
    customViewCreate(input: $input) {
      success
      customView { id name }
    }
  }`;
  const PREFS_MUTATION = `mutation SetViewPrefs($input: ViewPreferencesCreateInput!) {
    viewPreferencesCreate(input: $input) { success }
  }`;

  // Query existing custom views for this project to avoid duplicates
  interface CustomViewListResponse {
    customViews: { nodes: { id: string; name: string }[] };
  }
  const existingViews = new Set<string>();
  try {
    const viewList = await execute<CustomViewListResponse>(apiKey,
      `query { customViews { nodes { id name } } }`, {});
    for (const v of viewList.customViews.nodes) {
      existingViews.add(v.name);
    }
  } catch {
    // ignore — will attempt creation regardless
  }

  const viewFilter = { project: { id: { eq: project.id } } };

  // Helper: create a custom view and set its board preferences
  async function createProjectView(
    viewName: string,
    grouping: string,
  ): Promise<void> {
    if (existingViews.has(viewName)) {
      console.error(`View exists: ${viewName} — skipping`);
      return;
    }
    const viewData = await execute<CustomViewResponse>(apiKey, VIEW_MUTATION, {
      input: {
        name: viewName,
        teamId,
        projectId: project.id,
        filterData: viewFilter,
        shared: true,
      },
    });
    if (viewData.customViewCreate.success && viewData.customViewCreate.customView) {
      const viewId = viewData.customViewCreate.customView.id;
      console.error(`Created view: ${viewData.customViewCreate.customView.name} (${viewId})`);
      await execute(apiKey, PREFS_MUTATION, {
        input: {
          customViewId: viewId,
          type: 'organization',
          viewType: 'customView',
          preferences: {
            layout: 'board',
            issueGrouping: grouping,
            showSubIssues: true,
          },
        },
      });
    }
  }

  // Board view (Kanban grouped by status)
  try {
    await createProjectView(`${projectName} — Board`, 'status');
  } catch (err) {
    console.error(`Warning: failed to create project board view: ${err}`);
  }

  // Agent view (board grouped by assignee)
  try {
    await createProjectView(`${projectName} — By Agent`, 'assignee');
  } catch (err) {
    console.error(`Warning: failed to create project agent view: ${err}`);
  }

  // Play Pipeline milestones (ordered by workflow stage)
  const PLAY_STAGES = [
    { name: 'Infrastructure', sortOrder: 0 },
    { name: 'Backend', sortOrder: 1 },
    { name: 'Frontend', sortOrder: 2 },
    { name: 'Testing', sortOrder: 3 },
    { name: 'Quality', sortOrder: 4 },
    { name: 'Security', sortOrder: 5 },
    { name: 'Deploy', sortOrder: 6 },
  ] as const;

  interface MilestoneResponse {
    projectMilestoneCreate: { success: boolean; projectMilestone?: { id: string; name: string } };
  }
  interface ExistingMilestonesResponse {
    project: { projectMilestones: { nodes: { id: string; name: string }[] } };
  }
  const milestoneMap: Record<string, string> = {};

  // Check for existing milestones first (idempotent re-runs)
  try {
    const existing = await execute<ExistingMilestonesResponse>(apiKey,
      `query GetMilestones($id: String!) { project(id: $id) { projectMilestones { nodes { id name } } } }`,
      { id: project.id },
    );
    for (const ms of existing.project.projectMilestones.nodes) {
      milestoneMap[ms.name.toLowerCase()] = ms.id;
    }
  } catch {
    // ignore — will create fresh
  }

  for (const stage of PLAY_STAGES) {
    if (milestoneMap[stage.name.toLowerCase()]) {
      console.error(`Milestone exists: ${stage.name} (${milestoneMap[stage.name.toLowerCase()]})`);
      continue;
    }
    try {
      const msData = await execute<MilestoneResponse>(apiKey,
        `mutation CreateMilestone($input: ProjectMilestoneCreateInput!) {
          projectMilestoneCreate(input: $input) { success projectMilestone { id name } }
        }`,
        { input: { name: stage.name, projectId: project.id, sortOrder: stage.sortOrder } },
      );
      if (msData.projectMilestoneCreate.success && msData.projectMilestoneCreate.projectMilestone) {
        milestoneMap[stage.name.toLowerCase()] = msData.projectMilestoneCreate.projectMilestone.id;
        console.error(`Created milestone: ${stage.name}`);
      }
    } catch (err) {
      console.error(`Warning: failed to create milestone ${stage.name}: ${err}`);
    }
  }

  // Play Pipeline view (grouped by milestone)
  try {
    await createProjectView(`${projectName} — Play Pipeline`, 'projectMilestone');
  } catch (err) {
    console.error(`Warning: failed to create play pipeline view: ${err}`);
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

interface DeliberationAudioArtifact {
  label: string;
  mp3Path: string;
  transcriptPath: string;
  statusPath: string;
  status: string;
  mp3Exists: boolean;
  transcriptExists: boolean;
}

function readJsonFile<T>(filePath: string): T | null {
  try {
    return JSON.parse(fs.readFileSync(filePath, 'utf-8')) as T;
  } catch {
    return null;
  }
}

function collectDeliberationAudioArtifacts(): DeliberationAudioArtifact[] {
  const cwd = process.cwd();
  const definitions = [
    {
      label: 'Architecture deliberation',
      mp3Path: path.join(cwd, '.tasks', 'audio', 'architecture-deliberation.mp3'),
      transcriptPath: path.join(cwd, '.tasks', 'audio', 'architecture-deliberation.transcript.json'),
      statusPath: path.join(cwd, '.intake', 'audio', 'architecture-deliberation.status.json'),
    },
    {
      label: 'Design deliberation',
      mp3Path: path.join(cwd, '.tasks', 'audio', 'design-deliberation.mp3'),
      transcriptPath: path.join(cwd, '.tasks', 'audio', 'design-deliberation.transcript.json'),
      statusPath: path.join(cwd, '.intake', 'audio', 'design-deliberation.status.json'),
    },
  ];

  return definitions.map((artifact) => {
    const statusJson = readJsonFile<{ status?: string }>(artifact.statusPath);
    const mp3Exists = fs.existsSync(artifact.mp3Path);
    const transcriptExists = fs.existsSync(artifact.transcriptPath);
    const status = mp3Exists ? 'ready' : (statusJson?.status ?? (transcriptExists ? 'pending' : 'not started'));
    return {
      ...artifact,
      status,
      mp3Exists,
      transcriptExists,
    };
  });
}

function toRepoRelative(filePath: string): string {
  return path.relative(process.cwd(), filePath).split(path.sep).join('/');
}

function buildDeliberationAudioSection(baseUrl: string): string[] {
  const artifacts = collectDeliberationAudioArtifacts().filter((artifact) => artifact.transcriptExists || artifact.mp3Exists || artifact.status !== 'not started');
  if (artifacts.length === 0) return [];

  const lines: string[] = ['', '## Deliberation Audio'];
  for (const artifact of artifacts) {
    lines.push(`- **${artifact.label}:** ${artifact.status}`);
    if (artifact.transcriptExists && baseUrl) {
      const transcriptPath = toRepoRelative(artifact.transcriptPath);
      lines.push(`  Transcript: [${transcriptPath}](${baseUrl}/${transcriptPath})`);
    }
    if (artifact.mp3Exists && baseUrl) {
      const mp3Path = toRepoRelative(artifact.mp3Path);
      lines.push(`  MP3: [${mp3Path}](${baseUrl}/${mp3Path})`);
    } else if (!artifact.mp3Exists) {
      lines.push('  MP3: pending background render');
    }
  }

  return lines;
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

  lines.push(...buildDeliberationAudioSection(baseUrl));

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

  lines.push(...buildDeliberationAudioSection(baseUrl));

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

// =============================================================================
// rewriteProjectUrls — Bulk-replace base URL in all project issue descriptions
// =============================================================================

export interface RewriteUrlsResult {
  updatedCount: number;
  skippedCount: number;
  errorCount: number;
  totalIssues: number;
}

export async function rewriteProjectUrls({
  projectId,
  oldBaseUrl,
  newBaseUrl,
  apiKey,
}: {
  projectId: string;
  oldBaseUrl: string;
  newBaseUrl: string;
  apiKey: string;
}): Promise<RewriteUrlsResult> {
  let updatedCount = 0;
  let skippedCount = 0;
  let errorCount = 0;
  let totalIssues = 0;
  let hasNextPage = true;
  let cursor: string | undefined;

  console.error(`rewrite-urls: replacing "${oldBaseUrl}" → "${newBaseUrl}" in project ${projectId}`);

  while (hasNextPage) {
    const afterClause = cursor ? `, after: "${cursor}"` : '';
    interface IssuesPage {
      project: {
        issues: {
          nodes: Array<{ id: string; identifier: string; description: string | null }>;
          pageInfo: { hasNextPage: boolean; endCursor: string | null };
        };
      };
    }

    const page = await execute<IssuesPage>(
      apiKey,
      `query($pid: String!) {
        project(id: $pid) {
          issues(first: 50${afterClause}) {
            nodes { id identifier description }
            pageInfo { hasNextPage endCursor }
          }
        }
      }`,
      { pid: projectId },
    );

    const issues = page.project.issues.nodes;
    const pageInfo = page.project.issues.pageInfo;
    totalIssues += issues.length;

    for (const issue of issues) {
      if (!issue.description || !issue.description.includes(oldBaseUrl)) {
        skippedCount++;
        continue;
      }

      const newDescription = issue.description.replaceAll(oldBaseUrl, newBaseUrl);
      try {
        interface UpdateResult {
          issueUpdate: { success: boolean };
        }
        await execute<UpdateResult>(
          apiKey,
          `mutation($id: String!, $input: IssueUpdateInput!) {
            issueUpdate(id: $id, input: $input) { success }
          }`,
          { id: issue.id, input: { description: newDescription } },
        );
        updatedCount++;
      } catch (err) {
        console.error(`rewrite-urls: failed to update ${issue.identifier}: ${err}`);
        errorCount++;
      }
    }

    hasNextPage = pageInfo.hasNextPage;
    cursor = pageInfo.endCursor ?? undefined;

    console.error(`rewrite-urls: processed ${totalIssues} issues (${updatedCount} updated, ${skippedCount} skipped, ${errorCount} errors)`);
  }

  return { updatedCount, skippedCount, errorCount, totalIssues };
}

// =============================================================================
// GitHub Sync — Create GitHub issues mirroring Linear project issues (1:1)
// =============================================================================

export interface GitHubSyncMapping {
  linearId: string;
  linearIdentifier: string;
  linearTitle: string;
  githubIssueNumber: number;
  githubIssueUrl: string;
}

export interface GitHubSyncResult {
  createdCount: number;
  skippedCount: number;
  errorCount: number;
  totalLinearIssues: number;
  mappings: GitHubSyncMapping[];
}

export interface GitHubSyncOptions {
  projectId: string;
  projectName?: string;
  repo: string;
  branch: string;
  apiKey: string;
  githubProject?: number;
}

/** Small delay to respect GitHub's secondary rate limits (~30 creates/min). */
function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Execute a `gh` CLI command and return the trimmed stdout.
 * Throws on non-zero exit with the stderr message.
 */
function gh(args: string[]): string {
  try {
    return execFileSync('gh', args, {
      encoding: 'utf-8',
      timeout: 30_000,
      env: { ...process.env },
    }).trim();
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : String(err);
    throw new Error(`gh CLI failed: ${message}`);
  }
}

/**
 * Check if a GitHub issue with the exact title already exists in the repo.
 * Returns the issue number if found, undefined otherwise.
 */
function findExistingGitHubIssue(repo: string, title: string): { number: number; url: string } | undefined {
  try {
    // Search for issues with the exact title (open or closed)
    const result = gh(
      ['issue', 'list', '--repo', repo, '--search', `"${title}" in:title`, '--json', 'number,title,url', '--limit', '50'],
    );
    if (!result) return undefined;

    const issues = JSON.parse(result) as Array<{ number: number; title: string; url: string }>;
    const match = issues.find((i) => i.title === title);
    return match ? { number: match.number, url: match.url } : undefined;
  } catch {
    return undefined;
  }
}

/** Agent display names for GitHub Project board columns. */
const AGENT_DISPLAY_NAMES: Record<string, string> = {
  bolt: 'Bolt (Infrastructure)',
  rex: 'Rex (Implementation)',
  blaze: 'Blaze (Frontend)',
  grizz: 'Grizz (Go)',
  nova: 'Nova (Node)',
  tess: 'Tess (QA)',
  cleo: 'Cleo (Quality)',
  cipher: 'Cipher (Security)',
  morgan: 'Morgan (PM)',
  atlas: 'Atlas (Integration)',
  stitch: 'Stitch (Review)',
  angie: 'Angie (Agent Systems)',
  spark: 'Spark (Prototype)',
  tap: 'Tap (Integration)',
  vex: 'Vex (Debug)',
  keeper: 'Keeper (Ops)',
  healer: 'Healer (Self-Heal)',
  pixel: 'Pixel (Desktop)',
};

/** Extract agent name from issue title patterns like [bolt], [rex], etc. */
function extractAgentFromTitle(title: string): string | undefined {
  const match = title.match(/^\[(\w+)\]/);
  if (match) {
    const name = match[1].toLowerCase();
    if (AGENT_DISPLAY_NAMES[name]) return name;
  }
  return undefined;
}

/** Get the GitHub Projects V2 internal node ID for a project by its number. */
function getProjectNodeId(owner: string, projectNumber: number): string | undefined {
  try {
    return gh(['api', 'graphql', '-f',
      `query=query { organization(login: "${owner}") { projectV2(number: ${projectNumber}) { id } } }`,
      '--jq', '.data.organization.projectV2.id']).trim() || undefined;
  } catch {
    try {
      return gh(['api', 'graphql', '-f',
        `query=query { user(login: "${owner}") { projectV2(number: ${projectNumber}) { id } } }`,
        '--jq', '.data.user.projectV2.id']).trim() || undefined;
    } catch { return undefined; }
  }
}

/** Create an "Agent" single-select field on a project with all agent options. */
function setupAgentField(projectNodeId: string): { fieldId: string; optionMap: Record<string, string> } | undefined {
  // Check if field already exists
  try {
    const fieldsJson = gh(['api', 'graphql', '-f',
      `query=query { node(id: "${projectNodeId}") { ... on ProjectV2 { fields(first: 30) { nodes { ... on ProjectV2SingleSelectField { id name options { id name } } } } } } }`,
      '--jq', '.data.node.fields.nodes']);
    const fields = JSON.parse(fieldsJson);
    const existing = fields.find((f: { name?: string }) => f?.name === 'Agent');
    if (existing) {
      const optionMap: Record<string, string> = {};
      for (const opt of existing.options || []) {
        optionMap[opt.name] = opt.id;
      }
      return { fieldId: existing.id, optionMap };
    }
  } catch { /* proceed to create */ }

  // Create the field with agent options
  const options = Object.values(AGENT_DISPLAY_NAMES)
    .map((name) => `{ name: "${name}", color: GRAY, description: "" }`)
    .join(', ');
  // Also add Pending and Complete
  const allOptions = `{ name: "Pending", color: YELLOW, description: "" }, ${options}, { name: "Complete ✅", color: GREEN, description: "" }`;
  try {
    const result = gh(['api', 'graphql', '-f',
      `query=mutation { createProjectV2Field(input: { projectId: "${projectNodeId}", dataType: SINGLE_SELECT, name: "Agent", singleSelectOptions: [${allOptions}] }) { projectV2Field { ... on ProjectV2SingleSelectField { id options { id name } } } } }`,
      '--jq', '.data.createProjectV2Field.projectV2Field']);
    const field = JSON.parse(result);
    const optionMap: Record<string, string> = {};
    for (const opt of field.options || []) {
      optionMap[opt.name] = opt.id;
    }
    console.error(`github-sync: created "Agent" field with ${Object.keys(optionMap).length} options`);
    return { fieldId: field.id, optionMap };
  } catch (err) {
    console.error(`github-sync: warning: could not create Agent field: ${err}`);
    return undefined;
  }
}

/** Add issue to project and optionally set its Agent field. Returns the project item ID. */
function addIssueToProject(
  repo: string,
  issueNumber: number,
  projectNodeId: string,
  agentField?: { fieldId: string; optionMap: Record<string, string> },
  agentName?: string,
): string | undefined {
  try {
    // Get issue node ID
    const nodeId = gh(['api', `repos/${repo}/issues/${issueNumber}`, '--jq', '.node_id']).trim();
    if (!nodeId) return undefined;

    // Add to project
    const itemResult = gh(['api', 'graphql', '-f',
      `query=mutation { addProjectV2ItemById(input: { projectId: "${projectNodeId}", contentId: "${nodeId}" }) { item { id } } }`,
      '--jq', '.data.addProjectV2ItemById.item.id']);
    const itemId = itemResult.trim();
    if (!itemId) return undefined;

    // Set Agent field value if we have one
    if (agentField && agentName) {
      const displayName = AGENT_DISPLAY_NAMES[agentName] || 'Pending';
      const optionId = agentField.optionMap[displayName] || agentField.optionMap['Pending'];
      if (optionId) {
        try {
          gh(['api', 'graphql', '-f',
            `query=mutation { updateProjectV2ItemFieldValue(input: { projectId: "${projectNodeId}", itemId: "${itemId}", fieldId: "${agentField.fieldId}", value: { singleSelectOptionId: "${optionId}" } }) { projectV2Item { id } } }`]);
        } catch { /* non-fatal */ }
      }
    }

    return itemId;
  } catch (err) {
    console.error(`github-sync: warning: could not add issue #${issueNumber} to project: ${err}`);
    return undefined;
  }
}

/**
 * Extract the agent name from a Linear issue's labels.
 * Looks for labels like "agent:bolt", "bolt", "rex", etc.
 */
function extractAgentLabel(labels: Array<{ name: string }>): string | undefined {
  for (const label of labels) {
    const lower = label.name.toLowerCase();
    // Check for "agent:X" pattern
    if (lower.startsWith('agent:')) return lower;
    // Check for known agent names used as labels
    const knownAgents = [
      'bolt', 'rex', 'blaze', 'grizz', 'tess', 'cleo', 'cipher',
      'healer', 'angie', 'keeper', 'nova', 'spark', 'tap', 'vex',
      'pixel', 'morgan', 'atlas', 'stitch',
    ];
    if (knownAgents.includes(lower)) return `agent:${lower}`;
  }
  return undefined;
}

/**
 * Sync all issues from a Linear project into GitHub issues with 1:1 mapping.
 *
 * Creates a GitHub Project, mirrors the full Linear hierarchy (parent → sub-issues),
 * uses full descriptions (no truncation), and links back to Linear.
 */
export async function syncGitHubIssues(opts: GitHubSyncOptions): Promise<GitHubSyncResult> {
  const { projectId, repo, branch, apiKey, githubProject } = opts;

  const mappings: GitHubSyncMapping[] = [];
  let createdCount = 0;
  let skippedCount = 0;
  let errorCount = 0;

  console.error(`github-sync: syncing Linear project ${projectId} → GitHub repo ${repo} (branch: ${branch})`);

  // Verify gh CLI is available and authenticated
  try {
    gh(['auth', 'status']);
  } catch (err) {
    throw new Error(`gh CLI is not authenticated. Run "gh auth login" first. Error: ${err}`);
  }

  // Ensure required labels exist
  for (const labelName of ['intake']) {
    try {
      gh(['label', 'create', labelName, '--repo', repo, '--force']);
    } catch { /* label may already exist */ }
  }

  // 1. Fetch ALL issues from the Linear project with parent/child info
  interface LinearIssueNode {
    id: string;
    identifier: string;
    title: string;
    description: string | null;
    url: string;
    labels: { nodes: Array<{ name: string }> };
    parent: { id: string; identifier: string } | null;
    children: { nodes: Array<{ id: string; identifier: string }> };
  }

  const allIssues: LinearIssueNode[] = [];
  let hasNextPage = true;
  let cursor: string | undefined;

  while (hasNextPage) {
    const afterClause = cursor ? `, after: "${cursor}"` : '';
    interface ProjectIssuesPage {
      project: {
        issues: {
          nodes: LinearIssueNode[];
          pageInfo: { hasNextPage: boolean; endCursor: string | null };
        };
      };
    }

    const page = await execute<ProjectIssuesPage>(
      apiKey,
      `query($pid: String!) {
        project(id: $pid) {
          issues(first: 50${afterClause}) {
            nodes {
              id identifier title description url
              labels { nodes { name } }
              parent { id identifier }
              children { nodes { id identifier } }
            }
            pageInfo { hasNextPage endCursor }
          }
        }
      }`,
      { pid: projectId },
    );

    allIssues.push(...page.project.issues.nodes);
    hasNextPage = page.project.issues.pageInfo.hasNextPage;
    cursor = page.project.issues.pageInfo.endCursor ?? undefined;
  }

  console.error(`github-sync: fetched ${allIssues.length} Linear issues`);

  // Build lookup maps
  const identifierSet = new Set(allIssues.map((i) => i.identifier));
  const issueByIdentifier = new Map(allIssues.map((i) => [i.identifier, i]));

  // Classify hierarchy: roots have no parent in this project
  const roots = allIssues.filter((i) => !i.parent || !identifierSet.has(i.parent.identifier));
  const childrenOf = (parentId: string) =>
    allIssues.filter((i) => i.parent && i.parent.identifier === parentId);

  // Flatten to 2 levels: skip "wrapper" roots that only serve as containers
  // (e.g., "Main Implementation Task"). Their children become top-level tasks,
  // and leaves stay as sub-issues of those tasks.
  const topLevelTasks: LinearIssueNode[] = [];
  const skipIssues = new Set<string>();

  for (const root of roots) {
    const children = childrenOf(root.identifier);
    const hasGrandchildren = children.some((c) => childrenOf(c.identifier).length > 0);

    if (hasGrandchildren && children.length > 0) {
      // This root is a 3-level wrapper — skip it, promote its children
      skipIssues.add(root.identifier);
      console.error(`github-sync: skipping wrapper "${root.title}" — promoting ${children.length} children to top-level`);
      topLevelTasks.push(...children);
    } else {
      // Root is either a standalone issue or a direct parent of leaves — keep it
      topLevelTasks.push(root);
    }
  }

  // Build processing order: top-level tasks first, then their leaf children
  const processingOrder: LinearIssueNode[] = [];
  for (const task of topLevelTasks) {
    processingOrder.push(task);
    const leaves = childrenOf(task.identifier);
    processingOrder.push(...leaves);
  }
  // Add standalone issues (PRDs, etc.) not yet seen
  const seen = new Set(processingOrder.map((i) => i.identifier));
  for (const issue of allIssues) {
    if (!seen.has(issue.identifier) && !skipIssues.has(issue.identifier)) {
      processingOrder.push(issue);
    }
  }

  console.error(`github-sync: processing ${processingOrder.length} issues (${topLevelTasks.length} top-level tasks, ${skipIssues.size} wrappers skipped)`);

  // 2. Create or find a GitHub Project + set up Agent field
  const owner = repo.split('/')[0];
  let ghProjectNumber: number | undefined = githubProject;
  let ghProjectNodeId: string | undefined;

  if (!ghProjectNumber) {
    const projectTitle = opts.projectName || 'intake';
    try {
      let ownerId = '';
      try {
        ownerId = gh(['api', 'graphql', '-f', `query={ organization(login: "${owner}") { id } }`, '--jq', '.data.organization.id']);
      } catch {
        ownerId = gh(['api', 'graphql', '-f', `query={ user(login: "${owner}") { id } }`, '--jq', '.data.user.id']);
      }
      if (ownerId) {
        const result = gh(['api', 'graphql', '-f',
          `query=mutation { createProjectV2(input: { ownerId: "${ownerId}", title: "${projectTitle}" }) { projectV2 { id number url } } }`,
          '--jq', '.data.createProjectV2.projectV2']);
        const proj = JSON.parse(result);
        ghProjectNumber = proj.number;
        ghProjectNodeId = proj.id;
        console.error(`github-sync: created GitHub Project #${ghProjectNumber}: ${projectTitle}`);
        // Link project to the repository and set description
        try {
          const repoNodeId = gh(['api', `repos/${repo}`, '--jq', '.node_id']).trim();
          if (repoNodeId && ghProjectNodeId) {
            gh(['api', 'graphql', '-f',
              `query=mutation { linkProjectV2ToRepository(input: { projectId: "${ghProjectNodeId}", repositoryId: "${repoNodeId}" }) { repository { name } } }`]);
            console.error(`github-sync: linked project to ${repo}`);
          }
        } catch { /* non-fatal */ }
        try {
          const desc = `Intake-generated project board. Show Agent column, hide Assignees (agents use GitHub Apps).`;
          gh(['api', 'graphql', '-f',
            `query=mutation { updateProjectV2(input: { projectId: "${ghProjectNodeId}", shortDescription: "${desc}" }) { projectV2 { id } } }`]);
        } catch { /* non-fatal */ }
      }
    } catch (err) {
      console.error(`github-sync: warning: could not create GitHub Project: ${err}`);
    }
  } else {
    ghProjectNodeId = getProjectNodeId(owner, ghProjectNumber);
  }

  // Set up Agent single-select field on the project board
  let agentField: { fieldId: string; optionMap: Record<string, string> } | undefined;
  if (ghProjectNodeId) {
    agentField = setupAgentField(ghProjectNodeId);
    if (agentField) {
      console.error(`github-sync: Agent field ready with ${Object.keys(agentField.optionMap).length} options`);
    }
  }

  // 3. Create issues in hierarchy order, tracking identifier → GitHub issue number + node ID
  const ghIssueMap = new Map<string, { number: number; url: string; nodeId: string }>();
  const repoUrl = `https://github.com/${repo}`;
  const branchUrl = `${repoUrl}/tree/${branch}`;

  for (const issue of processingOrder) {
    // Check for existing GitHub issue with same title
    const existing = findExistingGitHubIssue(repo, issue.title);
    if (existing) {
      console.error(`github-sync: skipping "${issue.identifier}" — GitHub issue #${existing.number} already exists`);
      // Get node ID for sub-issue linking
      try {
        const nodeId = gh(['issue', 'view', String(existing.number), '--repo', repo, '--json', 'id', '--jq', '.id']);
        ghIssueMap.set(issue.identifier, { number: existing.number, url: existing.url, nodeId });
      } catch { /* best effort */ }
      mappings.push({
        linearId: issue.id,
        linearIdentifier: issue.identifier,
        linearTitle: issue.title,
        githubIssueNumber: existing.number,
        githubIssueUrl: existing.url,
      });
      skippedCount++;
      continue;
    }

    // Build full body (no truncation)
    const description = issue.description || '_No description._';
    const body = [
      `> **Linear issue:** [${issue.identifier}](${issue.url})`,
      `> **Branch:** [\`${branch}\`](${branchUrl})`,
      '',
      '---',
      '',
      description,
    ].join('\n');

    // Determine labels
    const agentLabel = extractAgentLabel(issue.labels.nodes);
    const ghLabels: string[] = ['intake'];
    if (agentLabel) ghLabels.push(agentLabel);

    try {
      const createArgs = ['issue', 'create', '--repo', repo, '--title', issue.title, '--body', body];
      for (const l of ghLabels) {
        createArgs.push('--label', l);
      }

      const createResult = gh(createArgs);
      const ghIssueUrl = createResult.trim();
      const ghIssueNumberMatch = ghIssueUrl.match(/\/issues\/(\d+)$/);
      const ghIssueNumber = ghIssueNumberMatch ? parseInt(ghIssueNumberMatch[1], 10) : 0;

      if (!ghIssueNumber) {
        console.error(`github-sync: warning: could not parse issue number from: ${createResult}`);
        errorCount++;
        continue;
      }

      // Get the node ID for sub-issue linking
      let nodeId = '';
      try {
        nodeId = gh(['issue', 'view', String(ghIssueNumber), '--repo', repo, '--json', 'id', '--jq', '.id']);
      } catch { /* best effort */ }

      ghIssueMap.set(issue.identifier, { number: ghIssueNumber, url: ghIssueUrl, nodeId });

      console.error(`github-sync: created GitHub issue #${ghIssueNumber} for ${issue.identifier}`);

      mappings.push({
        linearId: issue.id,
        linearIdentifier: issue.identifier,
        linearTitle: issue.title,
        githubIssueNumber: ghIssueNumber,
        githubIssueUrl: ghIssueUrl,
      });
      createdCount++;

      // Back-link in Linear
      const ghLink = `\n\n---\n🔗 **GitHub issue:** [#${ghIssueNumber}](${ghIssueUrl})`;
      const updatedDescription = (issue.description || '') + ghLink;
      try {
        interface UpdateResult { issueUpdate: { success: boolean } }
        await execute<UpdateResult>(apiKey,
          `mutation($id: String!, $input: IssueUpdateInput!) { issueUpdate(id: $id, input: $input) { success } }`,
          { id: issue.id, input: { description: updatedDescription } },
        );
      } catch (err) {
        console.error(`github-sync: warning: could not update Linear back-link for ${issue.identifier}: ${err}`);
      }

      // Rate limit: ~2s between creates
      await sleep(2000);
    } catch (err) {
      console.error(`github-sync: failed to create GitHub issue for ${issue.identifier}: ${err}`);
      errorCount++;
    }
  }

  // 4. Link sub-issues to parents via the GitHub sub-issues API
  // Skip links where the parent was a skipped wrapper issue
  console.error(`github-sync: linking sub-issues...`);
  let linkedCount = 0;
  let alreadyLinkedCount = 0;

  for (const issue of allIssues) {
    if (!issue.parent || !identifierSet.has(issue.parent.identifier)) continue;
    if (skipIssues.has(issue.parent.identifier)) continue;  // parent was flattened out

    const parentGh = ghIssueMap.get(issue.parent.identifier);
    const childGh = ghIssueMap.get(issue.identifier);
    if (!parentGh || !childGh || !childGh.nodeId) continue;

    try {
      // Check if already linked by fetching parent's sub-issues
      const existingSubs = gh(['api', `repos/${repo}/issues/${parentGh.number}/sub_issues`, '--jq', '.[].number']);
      const existingNums = new Set(existingSubs.trim().split('\n').filter(Boolean).map(Number));
      if (existingNums.has(childGh.number)) {
        alreadyLinkedCount++;
        continue;
      }

      const issueJson = gh(['api', `repos/${repo}/issues/${childGh.number}`, '--jq', '.id']);
      const databaseId = parseInt(issueJson.trim(), 10);
      if (!databaseId) continue;

      gh(['api', `repos/${repo}/issues/${parentGh.number}/sub_issues`, '--method', 'POST',
        '-F', `sub_issue_id=${databaseId}`]);
      linkedCount++;
      console.error(`github-sync: linked #${childGh.number} as sub-issue of #${parentGh.number}`);
      await sleep(1000);
    } catch (err) {
      console.error(`github-sync: warning: could not link ${issue.identifier} → ${issue.parent.identifier}: ${err}`);
    }
  }
  if (alreadyLinkedCount) {
    console.error(`github-sync: ${alreadyLinkedCount} sub-issue links already existed (skipped)`);
  }

  // 5. Add all issues to GitHub Project with agent assignment
  let projectAddedCount = 0;
  if (ghProjectNodeId) {
    console.error(`github-sync: populating GitHub Project board...`);
    for (const issue of processingOrder) {
      const ghEntry = ghIssueMap.get(issue.identifier);
      if (!ghEntry) continue;

      // Determine agent: check own title, own labels, then inherit from parent
      let agentName = extractAgentFromTitle(issue.title)
        || extractAgentLabel(issue.labels?.nodes || [])?.replace('agent:', '');

      if (!agentName && issue.parent) {
        const parent = issueByIdentifier.get(issue.parent.identifier);
        if (parent) {
          agentName = extractAgentFromTitle(parent.title)
            || extractAgentLabel(parent.labels?.nodes || [])?.replace('agent:', '');
        }
      }

      const itemId = addIssueToProject(repo, ghEntry.number, ghProjectNodeId, agentField, agentName);
      if (itemId) {
        projectAddedCount++;
        if (projectAddedCount % 10 === 0) {
          console.error(`github-sync: added ${projectAddedCount} issues to project...`);
        }
      }
      await sleep(500);
    }
    console.error(`github-sync: added ${projectAddedCount} issues to project #${ghProjectNumber}`);
  }

  console.error(
    `github-sync: done — ${createdCount} created, ${skippedCount} skipped, ${errorCount} errors, ${linkedCount} sub-issue links, ${projectAddedCount} in project`,
  );

  return { createdCount, skippedCount, errorCount, totalLinearIssues: allIssues.length, mappings };
}
