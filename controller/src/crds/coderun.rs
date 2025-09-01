#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::types::CLIType;

    #[test]
    fn test_cli_config_creation() {
        let cli_config = CLIConfig {
            cli_type: CLIType::Codex,
            model: "gpt-4".to_string(),
            settings: {
                let mut settings = HashMap::new();
                settings.insert("approval_policy".to_string(), serde_json::json!("on-failure"));
                settings
            },
            sandbox_mode: Some("workspace-write".to_string()),
            max_tokens: Some(4096),
            temperature: Some(0.7),
        };

        assert_eq!(cli_config.cli_type, CLIType::Codex);
        assert_eq!(cli_config.model, "gpt-4");
        assert_eq!(cli_config.sandbox_mode, Some("workspace-write".to_string()));
        assert_eq!(cli_config.max_tokens, Some(4096));
        assert_eq!(cli_config.temperature, Some(0.7));
    }

    #[test]
    fn test_coderun_with_cli_config() {
        let cli_config = CLIConfig {
            cli_type: CLIType::Claude,
            model: "sonnet".to_string(),
            settings: HashMap::new(),
            sandbox_mode: None,
            max_tokens: None,
            temperature: None,
        };

        let code_run_spec = CodeRunSpec {
            task_id: 42,
            service: "test-service".to_string(),
            repository_url: "https://github.com/test/repo".to_string(),
            docs_repository_url: "https://github.com/test/docs".to_string(),
            docs_project_directory: None,
            working_directory: None,
            model: "sonnet".to_string(), // Legacy field for backward compatibility
            github_user: None,
            github_app: Some("test-app".to_string()),
            context_version: 1,
            docs_branch: "main".to_string(),
            continue_session: false,
            overwrite_memory: false,
            env: HashMap::new(),
            env_from_secrets: Vec::new(),
            enable_docker: None,
            task_requirements: None,
            service_account_name: None,
            cli_config: Some(cli_config), // New CLI-agnostic field
        };

        assert_eq!(code_run_spec.task_id, 42);
        assert_eq!(code_run_spec.service, "test-service");
        assert!(code_run_spec.cli_config.is_some());
        assert_eq!(code_run_spec.cli_config.as_ref().unwrap().cli_type, CLIType::Claude);
    }
}