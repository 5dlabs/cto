use serde_json::{json, Value};
use std::collections::HashMap;

/// Get tool schemas for MCP protocol with rich descriptions
pub fn get_tool_schemas() -> Value {
    json!({
        "tools": [
            get_intake_schema(),
            get_play_schema(&HashMap::new()),
            get_play_status_schema(),
            get_jobs_schema(),
            get_stop_job_schema(),
            get_input_schema(),
            get_add_mcp_server_schema(),
            get_remove_mcp_server_schema(),
            get_update_mcp_server_schema(),
            // Cluster management tools
            get_cluster_create_schema(),
            get_cluster_status_schema(),
            get_cluster_list_schema(),
            get_cluster_delete_schema()
        ]
    })
}

/// Get tool schemas with config-based agent descriptions
pub fn get_tool_schemas_with_config(agents: &HashMap<String, crate::AgentConfig>) -> Value {
    json!({
        "tools": [
            get_intake_schema(),
            get_play_schema(agents),
            get_play_status_schema(),
            get_jobs_schema(),
            get_stop_job_schema(),
            get_input_schema(),
            get_add_mcp_server_schema(),
            get_remove_mcp_server_schema(),
            get_update_mcp_server_schema(),
            // Cluster management tools
            get_cluster_create_schema(),
            get_cluster_status_schema(),
            get_cluster_list_schema(),
            get_cluster_delete_schema()
        ]
    })
}

/// Unified intake tool schema - combines PRD parsing and documentation generation
fn get_intake_schema() -> Value {
    json!({
        "name": "intake",
        "description": "Process a PRD to generate tasks and documentation. Parses PRD, generates task breakdowns with complexity analysis, creates agent prompts (XML + Markdown), and optionally submits a PR. Use local=true for immediate local execution without Argo.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "project_name": {
                    "type": "string",
                    "description": "Name of the project subdirectory (required). Will contain .tasks folder with tasks and documentation."
                },
                "repository": {
                    "type": "string",
                    "description": "Target repository in org/repo format (e.g., '5dlabs/agent-sandbox'). Optional - if not provided, auto-detects from git remote in workspace."
                },
                "prd_content": {
                    "type": "string",
                    "description": "PRD content as a string (optional). If not provided, reads from {project_name}/prd.md or {project_name}/prd.txt"
                },
                "architecture_content": {
                    "type": "string",
                    "description": "Architecture document content (optional). If not provided, reads from {project_name}/architecture.md if it exists"
                },
                "local": {
                    "type": "boolean",
                    "description": "Run intake locally using tasks CLI instead of submitting Argo workflow (optional, defaults to false)",
                    "default": false
                },
                "num_tasks": {
                    "type": "integer",
                    "description": "Target number of tasks to generate (optional, defaults to 15)",
                    "default": 15
                },
                "expand": {
                    "type": "boolean",
                    "description": "Expand tasks into subtasks (optional, defaults to true)",
                    "default": true
                },
                "analyze": {
                    "type": "boolean",
                    "description": "Analyze task complexity (optional, defaults to true)",
                    "default": true
                },
                "model": {
                    "type": "string",
                    "description": "AI model to use (optional, defaults to claude-sonnet-4-20250514)"
                },
                "enrich_context": {
                    "type": "boolean",
                    "description": "Auto-scrape URLs found in PRD via Firecrawl to enrich task context (optional, defaults to true)",
                    "default": true
                },
                "include_codebase": {
                    "type": "boolean",
                    "description": "Include existing codebase as markdown context for documentation generation (optional, defaults to false)"
                },
                "cli": {
                    "type": "string",
                    "description": "CLI to use for documentation generation (optional, defaults to claude). Supports claude, cursor, codex.",
                    "enum": ["claude", "cursor", "codex"],
                    "default": "claude"
                }
            },
            "required": ["project_name"]
        }
    })
}

fn get_play_schema(agents: &HashMap<String, crate::AgentConfig>) -> Value {
    json!({
        "name": "play",
        "description": "Submit a Play workflow for multi-agent orchestration (Rex/Blaze → Cleo → Tess → Atlas → Bolt) with event-driven coordination",
        "inputSchema": {
            "type": "object",
            "properties": {
                "task_id": {
                    "type": "integer",
                    "description": "Task ID to implement from task files. Optional - if not provided, will auto-detect next available task based on dependencies and priority.",
                    "minimum": 1
                },
                "repository": {
                    "type": "string",
                    "description": "Target repository URL (e.g., 5dlabs/cto). Optional if defaults.play.repository is set in config."
                },
                "repository_path": {
                    "type": "string",
                    "description": "Absolute path to the repository on disk (e.g., /Users/name/code/cto-parallel-test). Use this when the target repository is not in the current workspace. Optional - if not provided, will use workspace detection."
                },
                "service": {
                    "type": "string",
                    "description": "Service identifier for persistent workspace. Optional if defaults.play.service is set in config.",
                    "pattern": "^[a-z0-9-]+$"
                },
                "docs_repository": {
                    "type": "string",
                    "description": "Documentation repository URL. Optional if defaults.play.docsRepository is set in config."
                },
                "docs_project_directory": {
                    "type": "string",
                    "description": "Project directory within docs repository (e.g., docs). Optional if defaults.play.docsProjectDirectory is set in config."
                },
                "implementation_agent": {
                    "type": "string",
                    "description": if agents.is_empty() {
                        "Agent for backend/general implementation work (e.g., 5DLabs-Rex)".to_string()
                    } else {
                        let agent_list = agents.keys().map(std::string::String::as_str).collect::<Vec<_>>().join(", ");
                        format!("Agent for backend/general implementation work. Available agents: {agent_list}")
                    }
                },
                "frontend_agent": {
                    "type": "string",
                    "description": if agents.is_empty() {
                        "Agent for frontend tasks (React, UI components) (e.g., 5DLabs-Blaze). Optional if defaults.play.frontendAgent is set in config.".to_string()
                    } else {
                        let agent_list = agents.keys().map(std::string::String::as_str).collect::<Vec<_>>().join(", ");
                        format!("Agent for frontend tasks (React, UI components). Available agents: {agent_list}")
                    }
                },
                "quality_agent": {
                    "type": "string",
                    "description": if agents.is_empty() {
                        "Agent for quality assurance (e.g., 5DLabs-Cleo)".to_string()
                    } else {
                        let agent_list = agents.keys().map(std::string::String::as_str).collect::<Vec<_>>().join(", ");
                        format!("Agent for quality assurance. Available agents: {agent_list}")
                    }
                },
                "testing_agent": {
                    "type": "string",
                    "description": if agents.is_empty() {
                        "Agent for testing and validation (e.g., 5DLabs-Tess)".to_string()
                    } else {
                        let agent_list = agents.keys().map(std::string::String::as_str).collect::<Vec<_>>().join(", ");
                        format!("Agent for testing and validation. Available agents: {agent_list}")
                    }
                },
                "parallel_execution": {
                    "type": "boolean",
                    "description": "Enable parallel execution of independent tasks. When true, analyzes task dependencies and runs tasks in parallel execution levels. When false (default), runs tasks sequentially one at a time. Requires tasks.json with proper dependencies.",
                    "default": false
                },
                "model": {
                    "type": "string",
                    "description": "Claude model to use for all agents (optional, defaults to configuration)"
                },
                "opencode_max_retries": {
                    "type": "integer",
                    "description": "Override maximum retry attempts for OpenCode runs (defaults to configuration)"
                },
                "cli": {
                    "type": "string",
                    "description": "CLI tool to use for all agents (optional, defaults to configuration)"
                }
            },
            "required": []
        }
    })
}

fn get_play_status_schema() -> Value {
    json!({
        "name": "play_status",
        "description": "Query current play workflow status and progress. Shows active workflows, next available tasks, and blocked tasks.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "repository": {
                    "type": "string",
                    "description": "Target repository URL (e.g., 5dlabs/cto). Optional if defaults.play.repository is set in config."
                }
            },
            "required": []
        }
    })
}

fn get_jobs_schema() -> Value {
    json!({
        "name": "jobs",
        "description": "List running Argo workflows (play, intake, and other workflows). Returns simplified status info.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "namespace": {"type": "string", "description": "Kubernetes namespace (default: cto)"},
                "include": {"type": "array", "items": {"type": "string", "enum": ["play", "intake", "workflow"]}, "description": "Filter which workflow types to include (default: all)"}
            }
        }
    })
}

fn get_stop_job_schema() -> Value {
    json!({
        "name": "stop_job",
        "description": "Stop a running Argo workflow (intake, play, or generic workflow).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "job_type": {"type": "string", "enum": ["intake", "play", "workflow"], "description": "Type of workflow to stop"},
                "name": {"type": "string", "description": "Workflow name"},
                "namespace": {"type": "string", "description": "Kubernetes namespace (default: cto)"}
            },
            "required": ["job_type", "name"]
        }
    })
}

fn get_input_schema() -> Value {
    json!({
        "name": "input",
        "description": "Send a live user message to a running Claude job via stream-json. Route by explicit job name or by user label.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "text": {"type": "string", "description": "Plain text to send as a user message"},
                "namespace": {"type": "string", "description": "Kubernetes namespace (default: cto)"},
                "fifo_path": {"type": "string", "description": "FIFO path inside container (default: /workspace/agent-input.jsonl)"},
                "job_type": {"type": "string", "enum": ["code"], "description": "Optional job type filter when routing by user"},
                "name": {"type": "string", "description": "Optional CodeRun resource name when routing by explicit job"},
                "user": {"type": "string", "description": "Optional user label (agents.platform/user) to route to active job"}
            },
            "required": ["text"]
        }
    })
}

fn get_add_mcp_server_schema() -> Value {
    json!({
        "name": "add_mcp_server",
        "description": "Add a new MCP server to the platform from a GitHub repository. Fetches the README, creates a CodeRun for Rex to analyze and update values.yaml, creates PR, and auto-merges after CI passes. A verification CodeRun will automatically run after merge to confirm the server is available.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "github_url": {
                    "type": "string",
                    "description": "GitHub repository URL for the MCP server (e.g., https://github.com/anthropics/github-mcp, https://github.com/modelcontextprotocol/server-slack)"
                },
                "skip_merge": {
                    "type": "boolean",
                    "description": "If true, create PR but don't auto-merge. Useful for review before deployment. Default: false"
                }
            },
            "required": ["github_url"]
        }
    })
}

fn get_remove_mcp_server_schema() -> Value {
    json!({
        "name": "remove_mcp_server",
        "description": "Remove an MCP server from the platform. Creates a CodeRun for Rex to remove the server from values.yaml, creates PR, and auto-merges after CI passes.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "server_key": {
                    "type": "string",
                    "description": "The server key to remove (e.g., 'github', 'slack', 'brave-search'). Use list_mcp_servers or check the tools config to see available servers."
                },
                "skip_merge": {
                    "type": "boolean",
                    "description": "If true, create PR but don't auto-merge. Default: false"
                }
            },
            "required": ["server_key"]
        }
    })
}

fn get_update_mcp_server_schema() -> Value {
    json!({
        "name": "update_mcp_server",
        "description": "Update an existing MCP server configuration. Re-fetches README from GitHub and creates a CodeRun for Rex to update values.yaml if changes are needed.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "server_key": {
                    "type": "string",
                    "description": "The server key to update (e.g., 'github', 'slack')"
                },
                "github_url": {
                    "type": "string",
                    "description": "Optional: Override GitHub URL if different from original or if original URL is not stored"
                },
                "skip_merge": {
                    "type": "boolean",
                    "description": "If true, create PR but don't auto-merge. Default: false"
                }
            },
            "required": ["server_key"]
        }
    })
}

// =============================================================================
// Cluster Management Tools
// =============================================================================

#[allow(clippy::too_many_lines)]
fn get_cluster_create_schema() -> Value {
    json!({
        "name": "cluster_create",
        "description": "Deploy a new Kubernetes cluster on bare metal or cloud infrastructure. For bare metal: provisions servers, installs Talos Linux, bootstraps Kubernetes. For cloud: creates managed Kubernetes (EKS/GKE/AKS) or VMs with Talos. This is a long-running operation (typically 15-45 minutes depending on provider).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "cluster_name": {
                    "type": "string",
                    "description": "DNS-compatible cluster name (required). Must contain only alphanumeric characters and hyphens."
                },
                "provider": {
                    "type": "string",
                    "enum": [
                        "latitude", "cherry", "hetzner", "vultr", "scaleway", "ovh", "digitalocean", "onprem",
                        "aws", "gcp", "azure"
                    ],
                    "description": "Infrastructure provider. Bare metal: latitude (default), cherry, hetzner, vultr, scaleway, ovh, digitalocean, onprem. Cloud: aws (EKS), gcp (GKE), azure (AKS).",
                    "default": "latitude"
                },
                "deployment_mode": {
                    "type": "string",
                    "enum": ["managed", "talos"],
                    "description": "For cloud providers: 'managed' uses EKS/GKE/AKS, 'talos' provisions VMs with Talos Linux. Bare metal always uses Talos.",
                    "default": "talos"
                },
                "region": {
                    "type": "string",
                    "description": "Region for deployment. Bare metal examples: MIA2, DAL, ASH, LAX (Latitude); fsn1, nbg1 (Hetzner). Cloud examples: us-east-1 (AWS), us-central1 (GCP), eastus (Azure).",
                    "default": "MIA2"
                },
                "auto_region": {
                    "type": "boolean",
                    "description": "Automatically select the best region based on stock/availability (bare metal only).",
                    "default": false
                },
                "node_count": {
                    "type": "integer",
                    "description": "Total node count. For bare metal: 1 control plane + N-1 workers. For managed K8s: worker nodes only.",
                    "minimum": 1,
                    "default": 2
                },
                "node_type": {
                    "type": "string",
                    "description": "Instance/server type. Examples: c2-small-x86 (Latitude), cx21 (Hetzner), t3.medium (AWS), n1-standard-2 (GCP), Standard_D2s_v3 (Azure).",
                    "default": "c2-small-x86"
                },
                "cp_plan": {
                    "type": "string",
                    "description": "Server plan for control plane node (bare metal only, defaults to node_type if not specified)."
                },
                "worker_plan": {
                    "type": "string",
                    "description": "Server plan for worker nodes (bare metal only, defaults to node_type if not specified)."
                },
                "kubernetes_version": {
                    "type": "string",
                    "description": "Kubernetes version for managed clusters (e.g., '1.29', '1.30'). For Talos, this is determined by talos_version.",
                    "default": "1.29"
                },
                "talos_version": {
                    "type": "string",
                    "description": "Talos Linux version to install (bare metal and cloud Talos mode).",
                    "default": "v1.9.0"
                },
                "install_disk": {
                    "type": "string",
                    "description": "Disk path for OS installation (bare metal only).",
                    "default": "/dev/sda"
                },
                "storage_disk": {
                    "type": "string",
                    "description": "NVMe disk for Mayastor storage (bare metal only). Defaults to install_disk if not specified."
                },
                "storage_replicas": {
                    "type": "integer",
                    "description": "Number of Mayastor replicas (1-3, bare metal only).",
                    "minimum": 1,
                    "maximum": 3,
                    "default": 2
                },
                "enable_vlan": {
                    "type": "boolean",
                    "description": "Enable VLAN private networking (bare metal only).",
                    "default": true
                },
                "vlan_subnet": {
                    "type": "string",
                    "description": "Private network subnet for VLAN (bare metal only).",
                    "default": "10.8.0.0/24"
                },
                "enable_firewall": {
                    "type": "boolean",
                    "description": "Enable host-level firewall (Talos mode only).",
                    "default": true
                },
                "vpc_id": {
                    "type": "string",
                    "description": "Existing VPC/VNet ID to deploy into (cloud providers only). If not specified, creates a new one."
                },
                "subnet_ids": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Existing subnet IDs to deploy into (cloud providers only)."
                },
                "gitops_repo": {
                    "type": "string",
                    "description": "GitOps repository URL for platform deployment.",
                    "default": "https://github.com/5dlabs/cto"
                },
                "gitops_branch": {
                    "type": "string",
                    "description": "GitOps branch to deploy from.",
                    "default": "develop"
                },
                "profile": {
                    "type": "string",
                    "enum": ["standard", "production"],
                    "description": "Installation profile for resource sizing.",
                    "default": "standard"
                }
            },
            "required": ["cluster_name"]
        }
    })
}

fn get_cluster_status_schema() -> Value {
    json!({
        "name": "cluster_status",
        "description": "Get detailed status of a cluster. For bare metal: shows installation progress, server IPs, and errors. For cloud: shows cluster state, endpoint, and node status.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "cluster_name": {
                    "type": "string",
                    "description": "Name of the cluster to check status for."
                },
                "provider": {
                    "type": "string",
                    "enum": [
                        "latitude", "cherry", "hetzner", "vultr", "scaleway", "ovh", "digitalocean", "onprem",
                        "aws", "gcp", "azure"
                    ],
                    "description": "Provider to query (required for cloud clusters, optional for bare metal which checks local state)."
                }
            },
            "required": ["cluster_name"]
        }
    })
}

fn get_cluster_list_schema() -> Value {
    json!({
        "name": "cluster_list",
        "description": "List all managed clusters and their status. For bare metal: scans local state files. For cloud: queries the provider API.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "provider": {
                    "type": "string",
                    "enum": [
                        "latitude", "cherry", "hetzner", "vultr", "scaleway", "ovh", "digitalocean", "onprem",
                        "aws", "gcp", "azure", "all"
                    ],
                    "description": "Filter by provider. Use 'all' to list from all providers, or omit for bare metal local state only.",
                    "default": "all"
                }
            },
            "required": []
        }
    })
}

fn get_cluster_delete_schema() -> Value {
    json!({
        "name": "cluster_delete",
        "description": "Destroy a cluster and all associated resources. For bare metal: deletes servers, VLANs, and local state. For cloud: deletes the managed cluster or VMs. This action is irreversible.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "cluster_name": {
                    "type": "string",
                    "description": "Name of the cluster to delete."
                },
                "provider": {
                    "type": "string",
                    "enum": [
                        "latitude", "cherry", "hetzner", "vultr", "scaleway", "ovh", "digitalocean", "onprem",
                        "aws", "gcp", "azure"
                    ],
                    "description": "Provider where the cluster is deployed (required for cloud, optional for bare metal)."
                },
                "confirm": {
                    "type": "boolean",
                    "description": "Must be set to true to confirm deletion. This is a safety check."
                }
            },
            "required": ["cluster_name", "confirm"]
        }
    })
}
