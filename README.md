# Cognitive Task Orchestrator

An AI-powered development platform that helps you generate documentation and implement code using Claude agents through simple MCP (Model Context Protocol) tools. The platform uses GitHub Apps for secure authentication and configuration-driven workflows.

## What It Does

The platform provides two main capabilities:
- **📝 Documentation Generation**: Automatically creates comprehensive documentation for your Task Master projects
- **⚡ Code Implementation**: Deploys autonomous Claude agents to implement specific tasks from your project

Both operations run as Kubernetes jobs with enhanced reliability through TTL-safe reconciliation, preventing infinite loops and ensuring proper resource cleanup. All results are automatically submitted via GitHub PRs.

## Getting Started

### Prerequisites
- Access to a Cursor/Claude environment with MCP support
- A project with Task Master initialized (`.taskmaster/` directory)
- GitHub repository for your project

## Installation

This is an integrated platform with a clear data flow:

**Component Architecture:**
- **MCP Server (`cto-mcp`)**: Handles MCP protocol calls from Cursor/Claude with configuration-driven defaults
- **Controller Service**: Kubernetes REST API that manages CodeRun/DocsRun CRDs via Argo Workflows
- **Argo Workflows**: Orchestrates agent deployment through workflow templates
- **Kubernetes Controllers**: Separate controllers for CodeRun and DocsRun resources with TTL-safe reconciliation
- **Agent Workspaces**: Isolated persistent volumes for each service with session continuity
- **GitHub Apps**: Secure authentication system replacing personal tokens

**Data Flow:**
1. Cursor calls `docs()` or `task()` via MCP protocol
2. MCP server loads configuration from `cto-config.json` and applies defaults
3. MCP server submits workflow to Argo with all required parameters
4. Argo Workflows creates CodeRun/DocsRun custom resources
5. Dedicated Kubernetes controllers reconcile CRDs with idempotent job management
6. Controllers deploy Claude agents as Jobs with workspace isolation
7. Agents authenticate via GitHub Apps and complete work
8. Agents submit GitHub PRs with automatic cleanup

### Deploy the Complete Platform

```bash
# Add the 5dlabs Helm repository
helm repo add 5dlabs https://5dlabs.github.io/cto
helm repo update

# Install Custom Resource Definitions (CRDs) first
kubectl apply -f https://raw.githubusercontent.com/5dlabs/cto/main/infra/charts/agent-platform/crds/platform-crds.yaml

# Install the agent-platform
helm install agent-platform 5dlabs/agent-platform --namespace agent-platform --create-namespace

# Setup agent secrets (interactive)
wget https://raw.githubusercontent.com/5dlabs/cto/main/infra/scripts/setup-agent-secrets.sh
chmod +x setup-agent-secrets.sh
./setup-agent-secrets.sh --help
```

**Requirements:**
- Kubernetes 1.19+
- Helm 3.2.0+
- GitHub Personal Access Token
- Anthropic API Key

**What you get:**
- Complete agent-platform platform deployed to Kubernetes
- REST API for task management
- Separate Kubernetes controllers for CodeRun/DocsRun resources with TTL-safe reconciliation
- Agent workspace management and isolation with persistent volumes
- Automatic resource cleanup and job lifecycle management
- MCP tools that connect to your deployment

### Optional: Remote Cluster Access with TwinGate

To access your Kubernetes cluster from anywhere (not just local network), install TwinGate connector:

```bash
# Add TwinGate Helm repository
helm repo add twingate https://twingate.github.io/helm-charts
helm repo update

# Install TwinGate connector (replace tokens with your actual values)
helm upgrade --install twingate-weightless-hummingbird twingate/connector \
  -n default \
  --set connector.network="maroonsnake" \
  --set connector.accessToken="your-access-token" \
  --set connector.refreshToken="your-refresh-token"
```

**Important**: After installation, add your Kubernetes service CIDR as resources in TwinGate admin panel. This enables the MCP tools to reach the agent-platform service using internal Kubernetes service URLs (e.g., `http://agent-platform.agent-platform.svc.cluster.local`) from anywhere.

### Install MCP Server

For Cursor/Claude integration, install the MCP server:

```bash
# One-liner installer (Linux/macOS)
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/5dlabs/cto/releases/download/v0.2.0/tools-installer.sh | sh

# Verify installation
cto-mcp --help   # MCP server for Cursor/Claude integration
```

**What you get:**
- `cto-mcp` - MCP server that integrates with Cursor/Claude
- Multi-platform support (Linux x64/ARM64, macOS Intel/Apple Silicon, Windows x64)
- Automatic installation to system PATH

### Configure Project Settings

Create a `cto-config.json` file in your project root to configure agents, models, and defaults:

```json
{
  "version": "1.0",
  "defaults": {
    "docs": {
      "model": "claude-opus-4-20250514",
      "githubApp": "5DLabs-Morgan",
      "includeCodebase": false,
      "sourceBranch": "main"
    },
    "code": {
      "model": "claude-opus-4-20250514", 
      "githubApp": "5DLabs-Rex",
      "continueSession": false,
      "workingDirectory": ".",
      "overwriteMemory": false,
      "docsRepository": "https://github.com/your-org/your-docs-repo",
      "docsProjectDirectory": "projects/your-project",
      "service": "your-service-name"
    }
  },
  "agents": {
    "morgan": "5DLabs-Morgan",
    "rex": "5DLabs-Rex", 
    "blaze": "5DLabs-Blaze",
    "cipher": "5DLabs-Cipher"
  }
}
```

### Configure Cursor MCP Integration

After creating your configuration file, configure Cursor to use the MCP server by creating a `.cursor/mcp.json` file in your project directory:

```json
{
  "mcpServers": {
    "cto-mcp": {
      "command": "cto-mcp",
      "args": [],
      "env": {}
    }
  }
}
```

**Usage:**
1. Create the `cto-config.json` file in your project root with your specific settings
2. Create the `.cursor/mcp.json` file to enable MCP integration
3. Restart Cursor to load the MCP server
4. The `docs()` and `task()` functions will be available with your configured defaults

**Benefits of Configuration-Driven Approach:**
- **Simplified MCP Calls**: Most parameters have sensible defaults from your config
- **Dynamic Agent Lists**: Tool descriptions show available agents from your config
- **Consistent Settings**: All team members use the same model/agent assignments
- **Easy Customization**: Change defaults without modifying MCP server setup

### Building from Source (Development)

```bash
# Build from source
git clone https://github.com/5dlabs/cto.git
cd cto/controller

# Build MCP server
cargo build --release --bin cto-mcp

# Verify the build
./target/release/cto-mcp --help   # MCP server

# Install to your system (optional)
cp target/release/cto-mcp /usr/local/bin/
```

### MCP Tools Available

The platform exposes two primary MCP tools:

#### 1. `docs` - Generate Documentation
Analyzes your Task Master project and creates comprehensive documentation.

```javascript
// Minimal call using config defaults
docs({
  working_directory: "projects/my-app"
});

// Override specific parameters
docs({
  working_directory: "projects/my-app",
  agent: "morgan",
  model: "claude-3-5-sonnet-20241022"
});
```

**What happens:**
✅ Creates a Claude agent with your project context
✅ Analyzes all tasks in your Task Master project
✅ Generates comprehensive documentation
✅ Submits a GitHub PR with the docs

**Generated Documents:**
```
.taskmaster/docs/
├── task-1/
│   ├── task.md           # Comprehensive task documentation
│   ├── acceptance-criteria.md  # Clear success criteria
│   └── prompt.md         # Implementation guidance for agents
├── task-2/
│   ├── task.md
│   ├── acceptance-criteria.md
│   └── prompt.md
└── ...
```

#### 2. `task` - Implement Code
Deploys an autonomous Claude agent to implement a specific task from your Task Master project.

```javascript
// Minimal call using config defaults
task({
  task_id: 5,
  repository: "https://github.com/myorg/my-project"
});

// Override specific parameters
task({
  task_id: 5,
  repository: "https://github.com/myorg/my-project",
  agent: "rex",
  service: "custom-service",
  working_directory: "services/api-server"
});

// Continue working on a partially completed or failed task
task({
  task_id: 5,
  repository: "https://github.com/myorg/my-project",
  continue_session: true
});
```

**What happens:**
✅ Creates a Claude agent with the generated docs as context
✅ Loads the specific task details from Task Master
✅ Implements the code autonomously
✅ Runs tests and validation
✅ Submits a GitHub PR with the implementation

## MCP Tool Reference

Complete parameter reference for both MCP tools.

### `docs` Tool Parameters

**Required:**
- `working_directory` - Working directory containing .taskmaster folder (e.g., `"projects/simple-api"`)

**Optional (with config defaults):**
- `agent` - Agent name to use (defaults to `defaults.docs.githubApp` mapping)
- `model` - Claude model to use (defaults to `defaults.docs.model`)
- `source_branch` - Source branch to work from (defaults to `defaults.docs.sourceBranch`)
- `include_codebase` - Include existing codebase as context (defaults to `defaults.docs.includeCodebase`)

### `task` Tool Parameters

**Required:**
- `task_id` - Task ID to implement from task files (integer, minimum 1)
- `repository` - Target repository URL (e.g., `"https://github.com/5dlabs/cto"`)

**Optional (with config defaults):**
- `service` - Target service name, creates workspace-{service} PVC (defaults to `defaults.code.service`)
- `docs_repository` - Documentation repository URL (defaults to `defaults.code.docsRepository`)
- `docs_project_directory` - Project directory within docs repository (defaults to `defaults.code.docsProjectDirectory`)
- `working_directory` - Working directory within target repository (defaults to `defaults.code.workingDirectory`)
- `agent` - Agent name for task assignment (defaults to `defaults.code.githubApp` mapping)
- `model` - Claude model to use (defaults to `defaults.code.model`)
- `continue_session` - Whether to continue a previous session (defaults to `defaults.code.continueSession`)
- `overwrite_memory` - Whether to overwrite CLAUDE.md memory file (defaults to `defaults.code.overwriteMemory`)
- `env` - Environment variables to set in the container (object with key-value pairs)
- `env_from_secrets` - Environment variables from secrets (array of objects with `name`, `secretName`, `secretKey`)

## Template Customization

The platform uses a template system to customize Claude agent behavior, settings, and prompts. Templates are Handlebars (`.hbs`) files that get rendered with task-specific data.

**Model Defaults**: Models are configured through `cto-config.json` defaults and can be overridden via MCP parameters. The platform supports all Claude models including `claude-opus-4-20250514` and `claude-3-5-sonnet-20241022`.

### Template Architecture

**Docs Tasks**: Generate documentation for Task Master projects

- **Prompts**: Rendered from `docs/prompt.md.hbs` template into ConfigMap
- **Settings**: `docs/settings.json.hbs` controls model, permissions, tools
- **Container Script**: `docs/container.sh.hbs` handles Git workflow and Claude execution

**Code Tasks**: Implement specific Task Master task IDs

- **Prompts**: Read from docs repository at `{docs_project_directory}/.taskmaster/docs/task-{id}/prompt.md` (or `_projects/{service}/.taskmaster/docs/task-{id}/prompt.md`)
- **Settings**: `code/settings.json.hbs` controls model, permissions, MCP tools
- **Container Script**: `code/container.sh.hbs` handles dual-repo workflow and Claude execution

### How to Customize

#### 1. Changing Agent Settings

Edit the settings template files directly:

```bash
# For docs generation agents
vim infra/charts/agent-platform/claude-templates/docs/settings.json.hbs

# For code implementation agents
vim infra/charts/agent-platform/claude-templates/code/settings.json.hbs
```

Settings control:

- Model selection (`claude-opus-4`, `claude-sonnet-4`, etc.)
- Tool permissions and access
- MCP tool configuration
- Enterprise managed settings

See [Claude Code Settings](https://docs.anthropic.com/en/docs/claude-code/settings) for complete configuration options.

#### 2. Updating Prompts

**For docs tasks** (affects all documentation generation):

```bash
# Edit the docs prompt template
vim infra/charts/agent-platform/claude-templates/docs/prompt.md.hbs
```

**For code tasks** (affects specific task implementation):

```bash
# Edit task-specific files in your docs repository
vim {docs_project_directory}/.taskmaster/docs/task-{id}/prompt.md
vim {docs_project_directory}/.taskmaster/docs/task-{id}/task.md
vim {docs_project_directory}/.taskmaster/docs/task-{id}/acceptance-criteria.md
```

#### 3. Adding Custom Hooks

Hooks are shell scripts that run during agent execution. Add new hook files to the `claude-templates` directory:

```bash
# Create new hook script (docs example)
vim infra/charts/agent-platform/claude-templates/docs/hooks/my-custom-hook.sh.hbs

# Create new hook script (code example)
vim infra/charts/agent-platform/claude-templates/code/hooks/my-custom-hook.sh.hbs
```

Hook files are automatically discovered and rendered. Ensure the hook name matches any references in your settings templates.

See [Claude Code Hooks Guide](https://docs.anthropic.com/en/docs/claude-code/hooks-guide) for detailed hook configuration and examples.

#### 4. Deploying Template Changes

After editing any template files, redeploy the agent-platform:

```bash
# Deploy template changes
helm upgrade agent-platform . -n agent-platform

# Verify ConfigMap was updated
kubectl get configmap claude-templates-configmap -n agent-platform -o yaml
```

**Important**: Template changes only affect new agent jobs. Running jobs continue with their original templates.

### Template Variables

Common variables available in templates:

- `{{task_id}}` - Task ID for code tasks
- `{{service_name}}` - Target service name
- `{{github_user}}` - GitHub username
- `{{repository_url}}` - Target repository URL
- `{{working_directory}}` - Working directory path
- `{{model}}` - Claude model name
- `{{docs_repository_url}}` - Documentation repository URL

## Best Practices

1. **Configure `cto-config.json` first** to set up your agents, models, and repository defaults
2. **Always generate docs first** to establish baseline documentation  
3. **Implement tasks sequentially** based on dependencies
4. **Use minimal MCP calls** - let configuration defaults handle most parameters
5. **Use `continue_session: true`** for retries on the same task
6. **Review GitHub PRs promptly** - agents provide detailed logs and explanations
7. **Update config file** when adding new agents or changing project structure

## Support

- Check GitHub PRs for detailed agent logs and explanations
- Review Task Master project structure in `.taskmaster/` directory
- Verify `cto-config.json` configuration and GitHub Apps authentication setup
- Ensure Argo Workflows are properly deployed and accessible

## License

This project is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0). This means:

- ✅ You can use, modify, and distribute this software freely
- ✅ You can use it for commercial purposes
- ⚠️ If you deploy a modified version on a network server, you must provide source code access to users
- ⚠️ Any derivative works must also be licensed under AGPL-3.0

The AGPL license is specifically designed for server-side software to ensure that improvements to the codebase remain open source, even when deployed as a service. This protects the open source nature of the project while allowing commercial use.

**Source Code Access**: Since this platform operates as a network service, users interacting with it have the right to access the source code under AGPL-3.0. The complete source code is available at this repository, ensuring full compliance with AGPL-3.0's network clause.

For more details, see the [LICENSE](LICENSE) file.

## Related Projects

- **[Task Master AI](https://github.com/eyaltoledano/claude-task-master)** - The AI-powered task management system that works perfectly with this agent-platform platform. Task Master AI helps you break down complex projects into manageable tasks, which can then be implemented using this platform's `task()` MCP tool.

## Roadmap

See our [ROADMAP.md](ROADMAP.md) for upcoming features and planned enhancements to the platform.

---

*The platform runs on Kubernetes and automatically manages Claude agent deployments, workspace isolation, and GitHub integration. All you need to do is call the MCP tools and review the resulting PRs.*
