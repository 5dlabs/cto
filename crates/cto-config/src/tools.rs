//! Tool mappings for task-based config generation.
//!
//! Maps technology keywords to relevant MCP tools.

use std::collections::HashSet;

/// Technology to tool mapping.
pub struct TechToolMapping {
    /// Keywords that indicate this technology.
    pub keywords: &'static [&'static str],
    /// Tools to add when this technology is detected.
    pub tools: &'static [&'static str],
}

/// Tool mappings based on technology keywords in task content.
pub const TECH_TOOL_MAPPINGS: &[TechToolMapping] = &[
    // Database tools
    TechToolMapping {
        keywords: &["postgresql", "postgres", "pg_", "psql", "sqlx"],
        tools: &["postgres_query", "postgres_execute"],
    },
    TechToolMapping {
        keywords: &["redis", "valkey", "cache", "session"],
        tools: &["redis_get", "redis_set", "redis_del"],
    },
    TechToolMapping {
        keywords: &["mongodb", "mongo", "document store"],
        tools: &["mongodb_query", "mongodb_aggregate"],
    },
    TechToolMapping {
        keywords: &["elasticsearch", "opensearch", "full-text search"],
        tools: &["elasticsearch_search", "elasticsearch_index"],
    },
    // Storage tools
    TechToolMapping {
        keywords: &["s3", "object storage", "file upload", "seaweedfs", "minio"],
        tools: &["s3_list", "s3_get", "s3_put"],
    },
    // Messaging tools
    TechToolMapping {
        keywords: &["kafka", "event stream", "message queue"],
        tools: &["kafka_produce", "kafka_consume"],
    },
    TechToolMapping {
        keywords: &["rabbitmq", "amqp", "message broker"],
        tools: &["rabbitmq_publish", "rabbitmq_consume"],
    },
    TechToolMapping {
        keywords: &["nats", "jetstream"],
        tools: &["nats_publish", "nats_subscribe"],
    },
    // API tools
    TechToolMapping {
        keywords: &["graphql", "apollo", "schema"],
        tools: &["graphql_query", "graphql_introspect"],
    },
    TechToolMapping {
        keywords: &["websocket", "real-time", "socket.io", "ws://"],
        tools: &["websocket_connect", "websocket_send"],
    },
    TechToolMapping {
        keywords: &["grpc", "protobuf", "proto"],
        tools: &["grpc_call", "grpc_stream"],
    },
    // Infrastructure tools (for Bolt)
    TechToolMapping {
        keywords: &["kubernetes", "k8s", "deployment", "service", "ingress"],
        tools: &[
            "kubernetes_applyResource",
            "kubernetes_listResources",
            "kubernetes_getResource",
            "kubernetes_deleteResource",
        ],
    },
    TechToolMapping {
        keywords: &["helm", "chart"],
        tools: &["helm_install", "helm_upgrade", "helm_list"],
    },
    TechToolMapping {
        keywords: &["cloudnative-pg", "cnpg", "postgresql operator"],
        tools: &["kubernetes_applyResource", "kubernetes_getPodsLogs"],
    },
    TechToolMapping {
        keywords: &["redis operator", "redis cluster"],
        tools: &["kubernetes_applyResource", "kubernetes_getPodsLogs"],
    },
    // Frontend tools (for Blaze)
    TechToolMapping {
        keywords: &["shadcn", "radix", "ui component"],
        tools: &[
            "shadcn_list_components",
            "shadcn_get_component",
            "shadcn_get_component_demo",
            "shadcn_get_component_metadata",
        ],
    },
    TechToolMapping {
        keywords: &["tanstack", "react-query", "react-table"],
        tools: &["context7_get_library_docs"],
    },
    TechToolMapping {
        keywords: &["tailwind", "css", "styling"],
        tools: &["context7_get_library_docs"],
    },
    // Mobile tools (for Tap)
    TechToolMapping {
        keywords: &["expo", "react-native", "mobile"],
        tools: &["xcodebuild_simulator_build", "xcodebuild_run_tests"],
    },
    TechToolMapping {
        keywords: &["ios", "swift", "xcode"],
        tools: &[
            "xcodebuild_simulator_build",
            "xcodebuild_device_build",
            "xcodebuild_run_tests",
        ],
    },
    // Desktop tools (for Spark)
    TechToolMapping {
        keywords: &["electron", "desktop app"],
        tools: &["xcodebuild_macos_build"],
    },
    // Auth tools
    TechToolMapping {
        keywords: &["authentication", "auth", "oauth", "jwt", "better-auth"],
        tools: &["better_auth_generate_schema", "better_auth_add_plugin"],
    },
    // Testing tools
    TechToolMapping {
        keywords: &["playwright", "e2e test", "browser test"],
        tools: &[
            "browser_navigate",
            "browser_click",
            "browser_type",
            "browser_snapshot",
        ],
    },
    TechToolMapping {
        keywords: &["vitest", "jest", "unit test"],
        tools: &["shell_execute"],
    },
    // Search tools
    TechToolMapping {
        keywords: &["web search", "research", "documentation"],
        tools: &["firecrawl_scrape", "firecrawl_search", "brave_search"],
    },
];

/// Analyze text content and return tools needed based on technology keywords.
#[must_use]
pub fn analyze_content_for_tools(content: &str) -> HashSet<String> {
    let mut tools = HashSet::new();
    let content_lower = content.to_lowercase();

    for mapping in TECH_TOOL_MAPPINGS {
        let has_keyword = mapping.keywords.iter().any(|kw| content_lower.contains(kw));
        if has_keyword {
            for tool in mapping.tools {
                tools.insert((*tool).to_string());
            }
        }
    }

    tools
}

/// Task-like trait for tool analysis.
pub trait ToolAnalyzable {
    /// Get the title of the item.
    fn title(&self) -> &str;
    /// Get the description of the item.
    fn description(&self) -> &str;
    /// Get additional details of the item.
    fn details(&self) -> &str;
    /// Get the agent hint for routing.
    fn agent_hint(&self) -> Option<&str>;
}

/// Analyze a task-like item for tools.
#[must_use]
pub fn analyze_task_for_tools<T: ToolAnalyzable>(task: &T) -> HashSet<String> {
    let content = format!("{} {} {}", task.title(), task.description(), task.details());
    analyze_content_for_tools(&content)
}

/// Analyze multiple tasks for a specific agent and return tools needed.
#[must_use]
pub fn analyze_agent_tasks_for_tools<T: ToolAnalyzable>(
    tasks: &[T],
    agent_name: &str,
) -> HashSet<String> {
    let mut tools = HashSet::new();
    let agent_lower = agent_name.to_lowercase();

    for task in tasks {
        if let Some(hint) = task.agent_hint() {
            if hint.to_lowercase() == agent_lower {
                tools.extend(analyze_task_for_tools(task));
            }
        }
    }

    tools
}

/// Analyze all tasks and return global technology tools.
#[must_use]
pub fn analyze_all_tasks_for_tools<T: ToolAnalyzable>(tasks: &[T]) -> HashSet<String> {
    let mut tools = HashSet::new();
    for task in tasks {
        tools.extend(analyze_task_for_tools(task));
    }
    tools
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestTask {
        title: String,
        description: String,
        details: String,
        agent: Option<String>,
    }

    impl ToolAnalyzable for TestTask {
        fn title(&self) -> &str {
            &self.title
        }
        fn description(&self) -> &str {
            &self.description
        }
        fn details(&self) -> &str {
            &self.details
        }
        fn agent_hint(&self) -> Option<&str> {
            self.agent.as_deref()
        }
    }

    #[test]
    fn test_analyze_content_postgres() {
        let tools = analyze_content_for_tools("Setup PostgreSQL database with sqlx");
        assert!(tools.contains("postgres_query"));
        assert!(tools.contains("postgres_execute"));
    }

    #[test]
    fn test_analyze_content_redis() {
        let tools = analyze_content_for_tools("Add Redis caching layer");
        assert!(tools.contains("redis_get"));
        assert!(tools.contains("redis_set"));
    }

    #[test]
    fn test_analyze_content_kubernetes() {
        let tools = analyze_content_for_tools("Deploy to Kubernetes cluster");
        assert!(tools.contains("kubernetes_applyResource"));
        assert!(tools.contains("kubernetes_listResources"));
    }

    #[test]
    fn test_analyze_content_shadcn() {
        let tools = analyze_content_for_tools("Build UI with shadcn components");
        assert!(tools.contains("shadcn_list_components"));
        assert!(tools.contains("shadcn_get_component"));
    }

    #[test]
    fn test_analyze_task() {
        let task = TestTask {
            title: "Setup Database".to_string(),
            description: "Configure PostgreSQL".to_string(),
            details: "Use sqlx for queries".to_string(),
            agent: Some("rex".to_string()),
        };

        let tools = analyze_task_for_tools(&task);
        assert!(tools.contains("postgres_query"));
    }

    #[test]
    fn test_analyze_agent_tasks() {
        let tasks = vec![
            TestTask {
                title: "Setup Database".to_string(),
                description: "Configure PostgreSQL".to_string(),
                details: String::new(),
                agent: Some("rex".to_string()),
            },
            TestTask {
                title: "Build UI".to_string(),
                description: "Use shadcn".to_string(),
                details: String::new(),
                agent: Some("blaze".to_string()),
            },
        ];

        let rex_tools = analyze_agent_tasks_for_tools(&tasks, "rex");
        assert!(rex_tools.contains("postgres_query"));
        assert!(!rex_tools.contains("shadcn_list_components"));

        let blaze_tools = analyze_agent_tasks_for_tools(&tasks, "blaze");
        assert!(blaze_tools.contains("shadcn_list_components"));
        assert!(!blaze_tools.contains("postgres_query"));
    }
}
