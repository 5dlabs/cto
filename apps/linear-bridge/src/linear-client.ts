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
}

export function createLinearClient(
  apiKey: string,
  logger: { info: Function; warn: Function; error: Function },
): LinearClient {
  // Linear API keys (lin_api_*) are sent directly; OAuth tokens use Bearer prefix
  const authHeader = apiKey.startsWith("lin_api_") ? apiKey : `Bearer ${apiKey}`;

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
    async createProject(name, teamId, description) {
      interface Response {
        projectCreate: { success: boolean; project?: LinearProject };
      }

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
            teamIds: [teamId],
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

      const input: Record<string, unknown> = { title, teamId };
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
        { teamId },
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
        { teamId, name },
      );

      if (!createData.issueLabelCreate.issueLabel) {
        throw new Error(`Failed to create label '${name}'`);
      }
      return createData.issueLabelCreate.issueLabel;
    },

    async getWorkflowStates(teamId) {
      interface Response {
        team: { states: { nodes: WorkflowState[] } };
      }

      const data = await execute<Response>(
        `query GetTeamWorkflowStates($teamId: String!) {
          team(id: $teamId) {
            states { nodes { id name type position } }
          }
        }`,
        { teamId },
      );

      return data.team.states.nodes;
    },

    async createCustomView({ name, teamId, projectId, shared }) {
      interface Response {
        customViewCreate: { success: boolean; customView?: LinearCustomView };
      }

      const input: Record<string, unknown> = {
        name,
        teamId,
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
  };
}
