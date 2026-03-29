const LINEAR_API_URL = "https://api.linear.app/graphql";

interface GraphQLResponse<T> {
  data?: T;
  errors?: Array<{ message: string }>;
}

export interface LinearIssue {
  id: string;
  identifier: string;
  title: string;
  description?: string;
  url: string;
  priority: number;
  state?: { id: string; name: string; type: string };
  team?: { id: string; name: string; key: string };
}

export interface LinearProject {
  id: string;
  name: string;
  description?: string;
  url: string;
}

export interface LinearComment {
  id: string;
  body: string;
  createdAt: string;
}

export interface LinearLabel {
  id: string;
  name: string;
  color: string;
}

export interface WorkflowState {
  id: string;
  name: string;
  type: string;
  position: number;
}

export interface LinearCustomView {
  id: string;
  name: string;
  url: string;
}

// =============================================================================
// Agent Activity Types (ported from crates/pm/src/activities.rs)
// =============================================================================

export type ActivityContent =
  | { type: 'thought'; body: string }
  | { type: 'action'; action: string; parameter: string; result?: string }
  | { type: 'elicitation'; body: string }
  | { type: 'response'; body: string }
  | { type: 'error'; body: string };

export type ActivitySignal = 'auth' | 'select';

export interface SelectOption {
  value: string;
  label?: string;
}

export interface SelectSignalMetadata {
  options: SelectOption[];
}

export interface AuthSignalMetadata {
  url: string;
  userId?: string;
  providerName?: string;
}

export type SignalMetadata = AuthSignalMetadata | SelectSignalMetadata;

export interface AgentActivityCreateInput {
  agentSessionId: string;
  content: ActivityContent;
  signal?: ActivitySignal;
  signalMetadata?: SignalMetadata;
  ephemeral?: boolean;
}

export interface AgentSessionUpdateInput {
  plan?: PlanStep[];
  externalLink?: string;
}

export type PlanStepStatus = 'pending' | 'inProgress' | 'completed' | 'canceled';

export interface PlanStep {
  content: string;
  status: PlanStepStatus;
}

export interface LinearClient {
  createProject(name: string, teamId: string, description?: string): Promise<LinearProject>;
  createIssue(opts: {
    title: string;
    description?: string;
    teamId: string;
    projectId?: string;
    labelIds?: string[];
    assigneeId?: string;
    parentId?: string;
  }): Promise<LinearIssue>;
  createComment(issueId: string, body: string): Promise<LinearComment>;
  updateIssueState(issueId: string, stateName: string): Promise<void>;
  getOrCreateLabel(teamId: string, name: string): Promise<LinearLabel>;
  getWorkflowStates(teamId: string): Promise<WorkflowState[]>;
  createCustomView(opts: {
    name: string;
    teamId: string;
    projectId?: string;
    shared?: boolean;
  }): Promise<LinearCustomView>;
  createAgentActivity(input: AgentActivityCreateInput): Promise<{ id: string }>;
  updateAgentSession(sessionId: string, input: AgentSessionUpdateInput): Promise<void>;
  resolveTeamId(teamIdentifier: string): Promise<string>;
  /** Poll session activities — returns activities created after `afterCursor` (ISO timestamp) */
  getSessionActivities(sessionId: string, afterCursor?: string): Promise<SessionActivity[]>;
  /** Get the authenticated app user ID (cached after first call) */
  getAppUserId(): Promise<string>;
  /** Set delegate on an issue if none is currently set */
  setDelegateIfNone(issueId: string): Promise<boolean>;
  /** Move issue to first "started" workflow state */
  moveToStartedStatus(issueId: string): Promise<boolean>;
}

export interface SessionActivity {
  id: string;
  createdAt: string;
  contentType: string;
  body?: string;
}

export function createLinearClient(
  apiKey: string,
  logger: { info: Function; warn: Function; error: Function },
): LinearClient {
  // Linear API keys (lin_api_*) are sent directly; OAuth tokens use Bearer prefix
  const authHeader = apiKey.startsWith("lin_api_") ? apiKey : `Bearer ${apiKey}`;
  const teamIdCache = new Map<string, string>();
  let cachedAppUserId: string | undefined;

  function looksLikeUuid(value: string): boolean {
    return /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(value);
  }

  async function execute<T>(query: string, variables: Record<string, unknown> = {}): Promise<T> {
    const response = await fetch(LINEAR_API_URL, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
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
      const messages = json.errors.map((e) => e.message).join(", ");
      throw new Error(`GraphQL errors: ${messages}`);
    }

    if (!json.data) {
      throw new Error("No data in GraphQL response");
    }

    return json.data;
  }

  return {
    async resolveTeamId(teamIdentifier) {
      if (!teamIdentifier) {
        throw new Error("Team identifier is required");
      }
      if (looksLikeUuid(teamIdentifier)) {
        return teamIdentifier;
      }
      const cached = teamIdCache.get(teamIdentifier);
      if (cached) {
        return cached;
      }

      interface TeamsResponse {
        teams: { nodes: Array<{ id: string; key: string; name: string }> };
      }

      const data = await execute<TeamsResponse>(
        `query ResolveTeamIdentifier {
          teams {
            nodes { id key name }
          }
        }`,
      );

      const match = data.teams.nodes.find(
        (team) => team.key === teamIdentifier || team.id === teamIdentifier,
      );
      if (!match) {
        throw new Error(`Unable to resolve team identifier '${teamIdentifier}' to a UUID`);
      }

      teamIdCache.set(teamIdentifier, match.id);
      teamIdCache.set(match.id, match.id);
      logger.info(`Resolved Linear team ${teamIdentifier} -> ${match.id}`);
      return match.id;
    },

    async createProject(name, teamId, description) {
      interface Response {
        projectCreate: { success: boolean; project?: LinearProject };
      }

      const resolvedTeamId = await this.resolveTeamId(teamId);

      const data = await execute<Response>(
        `mutation CreateProject($input: ProjectCreateInput!) {
          projectCreate(input: $input) {
            success
            project { id name description url }
          }
        }`,
        {
          input: {
            name,
            teamIds: [resolvedTeamId],
            ...(description ? { description } : {}),
          },
        },
      );

      if (!data.projectCreate.success || !data.projectCreate.project) {
        throw new Error("Failed to create project");
      }
      return data.projectCreate.project;
    },

    async createIssue({ title, description, teamId, projectId, labelIds, assigneeId, parentId }) {
      interface Response {
        issueCreate: { success: boolean; issue?: LinearIssue };
      }

      const resolvedTeamId = await this.resolveTeamId(teamId);
      const input: Record<string, unknown> = { title, teamId: resolvedTeamId };
      if (description) input.description = description;
      if (projectId) input.projectId = projectId;
      if (labelIds?.length) input.labelIds = labelIds;
      if (assigneeId) input.assigneeId = assigneeId;
      if (parentId) input.parentId = parentId;

      const data = await execute<Response>(
        `mutation CreateIssue($input: IssueCreateInput!) {
          issueCreate(input: $input) {
            success
            issue {
              id identifier title description url priority
              state { id name type }
              team { id name key }
            }
          }
        }`,
        { input },
      );

      if (!data.issueCreate.success || !data.issueCreate.issue) {
        throw new Error("Failed to create issue");
      }
      return data.issueCreate.issue;
    },

    async createComment(issueId, body) {
      interface Response {
        commentCreate: { success: boolean; comment?: LinearComment };
      }

      const data = await execute<Response>(
        `mutation CreateComment($input: CommentCreateInput!) {
          commentCreate(input: $input) {
            success
            comment { id body createdAt }
          }
        }`,
        { input: { issueId, body } },
      );

      if (!data.commentCreate.comment) {
        throw new Error("Failed to create comment");
      }
      return data.commentCreate.comment;
    },

    async updateIssueState(issueId, stateName) {
      // First get the issue to find its team
      interface IssueResponse {
        issue: { team: { id: string } };
      }

      const issueData = await execute<IssueResponse>(
        `query GetIssue($id: String!) {
          issue(id: $id) { team { id } }
        }`,
        { id: issueId },
      );

      const teamId = issueData.issue.team.id;

      // Get workflow states for the team
      const states = await this.getWorkflowStates(teamId);
      const state = states.find((s) => s.name.toLowerCase() === stateName.toLowerCase());
      if (!state) {
        throw new Error(`Workflow state '${stateName}' not found`);
      }

      // Update the issue
      interface UpdateResponse {
        issueUpdate: { success: boolean };
      }

      const data = await execute<UpdateResponse>(
        `mutation UpdateIssue($id: String!, $input: IssueUpdateInput!) {
          issueUpdate(id: $id, input: $input) { success }
        }`,
        { id: issueId, input: { stateId: state.id } },
      );

      if (!data.issueUpdate.success) {
        throw new Error("Failed to update issue state");
      }
    },

    async getOrCreateLabel(teamId, name) {
      const resolvedTeamId = await this.resolveTeamId(teamId);
      // Query existing labels
      interface FindResponse {
        team: { labels: { nodes: LinearLabel[] } };
      }

      const findData = await execute<FindResponse>(
        `query GetTeamLabels($teamId: String!) {
          team(id: $teamId) {
            labels { nodes { id name color } }
          }
        }`,
        { teamId: resolvedTeamId },
      );

      const existing = findData.team.labels.nodes.find((l) => l.name === name);
      if (existing) return existing;

      // Create new label
      interface CreateResponse {
        issueLabelCreate: { success: boolean; issueLabel?: LinearLabel };
      }

      const createData = await execute<CreateResponse>(
        `mutation CreateLabel($teamId: String!, $name: String!) {
          issueLabelCreate(input: { teamId: $teamId, name: $name }) {
            success
            issueLabel { id name color }
          }
        }`,
        { teamId: resolvedTeamId, name },
      );

      if (!createData.issueLabelCreate.issueLabel) {
        throw new Error(`Failed to create label '${name}'`);
      }
      return createData.issueLabelCreate.issueLabel;
    },

    async getWorkflowStates(teamId) {
      const resolvedTeamId = await this.resolveTeamId(teamId);
      interface Response {
        team: { states: { nodes: WorkflowState[] } };
      }

      const data = await execute<Response>(
        `query GetTeamWorkflowStates($teamId: String!) {
          team(id: $teamId) {
            states { nodes { id name type position } }
          }
        }`,
        { teamId: resolvedTeamId },
      );

      return data.team.states.nodes;
    },

    async createCustomView({ name, teamId, projectId, shared }) {
      interface Response {
        customViewCreate: { success: boolean; customView?: LinearCustomView };
      }

      const resolvedTeamId = await this.resolveTeamId(teamId);

      const input: Record<string, unknown> = {
        name,
        teamId: resolvedTeamId,
        shared: shared ?? true,
      };
      if (projectId) {
        input.filterData = { project: { id: { eq: projectId } } };
      }

      const data = await execute<Response>(
        `mutation CreateCustomView($input: CustomViewCreateInput!) {
          customViewCreate(input: $input) {
            success
            customView { id name url }
          }
        }`,
        { input },
      );

      if (!data.customViewCreate.customView) {
        throw new Error("Failed to create custom view");
      }
      return data.customViewCreate.customView;
    },

    async createAgentActivity(input) {
      interface Response {
        agentActivityCreate: {
          success: boolean;
          agentActivity?: { id: string };
        };
      }

      const data = await execute<Response>(
        `mutation AgentActivityCreate($input: AgentActivityCreateInput!) {
          agentActivityCreate(input: $input) {
            success
            agentActivity { id }
          }
        }`,
        { input },
      );

      if (!data.agentActivityCreate.success || !data.agentActivityCreate.agentActivity) {
        throw new Error("Failed to create agent activity");
      }
      return data.agentActivityCreate.agentActivity;
    },

    async updateAgentSession(sessionId, input) {
      interface Response {
        agentSessionUpdate: { success: boolean };
      }

      const data = await execute<Response>(
        `mutation AgentSessionUpdate($id: String!, $input: AgentSessionUpdateInput!) {
          agentSessionUpdate(id: $id, input: $input) {
            success
          }
        }`,
        { id: sessionId, input },
      );

      if (!data.agentSessionUpdate.success) {
        throw new Error("Failed to update agent session");
      }
    },

    async getSessionActivities(sessionId, afterCursor) {
      interface Response {
        agentSession: {
          activities: {
            nodes: Array<{
              id: string;
              createdAt: string;
              content: { type?: string; body?: string } | { type?: string; action?: string };
            }>;
          };
        };
      }

      const data = await execute<Response>(
        `query SessionActivities($id: String!) {
          agentSession(id: $id) {
            activities {
              nodes {
                id
                createdAt
                content {
                  ... on AgentActivityPromptContent { type body }
                  ... on AgentActivityResponseContent { type body }
                  ... on AgentActivityElicitationContent { type body }
                  ... on AgentActivityThoughtContent { type body }
                  ... on AgentActivityErrorContent { type body }
                  ... on AgentActivityActionContent { type action }
                }
              }
            }
          }
        }`,
        { id: sessionId },
      );

      const nodes = data.agentSession?.activities?.nodes ?? [];
      const activities: SessionActivity[] = nodes.map(n => ({
        id: n.id,
        createdAt: n.createdAt,
        contentType: (n.content as any)?.type ?? 'unknown',
        body: (n.content as any)?.body ?? (n.content as any)?.action,
      }));

      if (afterCursor) {
        const cutoff = new Date(afterCursor).getTime();
        return activities.filter(a => new Date(a.createdAt).getTime() > cutoff);
      }
      return activities;
    },

    async getAppUserId() {
      if (cachedAppUserId) return cachedAppUserId;

      interface Response {
        viewer: { id: string };
      }

      const data = await execute<Response>(`query { viewer { id } }`);
      cachedAppUserId = data.viewer.id;
      logger.info(`Resolved app user ID: ${cachedAppUserId}`);
      return cachedAppUserId;
    },

    async setDelegateIfNone(issueId) {
      try {
        interface IssueResponse {
          issue: { delegate?: { id: string } | null };
        }

        const issueData = await execute<IssueResponse>(
          `query GetIssueDelegate($id: String!) {
            issue(id: $id) { delegate { id } }
          }`,
          { id: issueId },
        );

        if (issueData.issue.delegate) {
          logger.info(`Issue ${issueId} already has delegate ${issueData.issue.delegate.id}`);
          return false;
        }

        const appUserId = await this.getAppUserId();

        interface UpdateResponse {
          issueUpdate: { success: boolean };
        }

        const data = await execute<UpdateResponse>(
          `mutation SetDelegate($id: String!, $input: IssueUpdateInput!) {
            issueUpdate(id: $id, input: $input) { success }
          }`,
          { id: issueId, input: { delegateId: appUserId } },
        );

        if (data.issueUpdate.success) {
          logger.info(`Set delegate on issue ${issueId} to app user ${appUserId}`);
        }
        return data.issueUpdate.success;
      } catch (err) {
        logger.warn(`Failed to set delegate on issue ${issueId}: ${err}`);
        return false;
      }
    },

    async moveToStartedStatus(issueId) {
      try {
        interface IssueResponse {
          issue: {
            state: { type: string };
            team: { id: string };
          };
        }

        const issueData = await execute<IssueResponse>(
          `query GetIssueState($id: String!) {
            issue(id: $id) {
              state { type }
              team { id }
            }
          }`,
          { id: issueId },
        );

        // Only move if currently in a non-started state
        if (issueData.issue.state.type === 'started') {
          return false;
        }

        const states = await this.getWorkflowStates(issueData.issue.team.id);
        const startedStates = states
          .filter(s => s.type === 'started')
          .sort((a, b) => a.position - b.position);

        if (startedStates.length === 0) {
          logger.warn(`No "started" workflow states found for issue ${issueId}`);
          return false;
        }

        interface UpdateResponse {
          issueUpdate: { success: boolean };
        }

        const data = await execute<UpdateResponse>(
          `mutation MoveToStarted($id: String!, $input: IssueUpdateInput!) {
            issueUpdate(id: $id, input: $input) { success }
          }`,
          { id: issueId, input: { stateId: startedStates[0].id } },
        );

        if (data.issueUpdate.success) {
          logger.info(`Moved issue ${issueId} to started state "${startedStates[0].name}"`);
        }
        return data.issueUpdate.success;
      } catch (err) {
        logger.warn(`Failed to move issue ${issueId} to started: ${err}`);
        return false;
      }
    },
  };
}
