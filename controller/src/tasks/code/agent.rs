//! Agent classification and name extraction for multi-agent workflow support

use regex::Regex;
use std::collections::HashSet;
use std::sync::OnceLock;

/// Static regex for agent name extraction (compiled once)
static AGENT_NAME_REGEX: OnceLock<Regex> = OnceLock::new();

/// Agent classifier for determining agent types and workspace requirements
#[derive(Debug, Clone)]
pub struct AgentClassifier {
    /// Set of known implementation agents that share workspace
    implementation_agents: HashSet<String>,
}

impl Default for AgentClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentClassifier {
    /// Create a new agent classifier with predefined implementation agents
    #[must_use]
    pub fn new() -> Self {
        let mut implementation_agents = HashSet::new();
        // Rex and Blaze are implementation agents that share workspace
        implementation_agents.insert("rex".to_string());
        implementation_agents.insert("blaze".to_string());

        Self {
            implementation_agents,
        }
    }

    /// Extract agent name from GitHub App identifier
    ///
    /// # Examples
    /// ```
    /// use controller::tasks::code::agent::AgentClassifier;
    /// let classifier = AgentClassifier::new();
    /// assert_eq!(classifier.extract_agent_name("5DLabs-Rex").unwrap(), "rex");
    /// assert_eq!(classifier.extract_agent_name("5DLabs-Cleo[bot]").unwrap(), "cleo");
    /// assert_eq!(classifier.extract_agent_name("5DLabs-Tess").unwrap(), "tess");
    /// ```
    pub fn extract_agent_name(&self, github_app: &str) -> Result<String, String> {
        // Initialize regex once for performance
        let regex = AGENT_NAME_REGEX.get_or_init(|| {
            Regex::new(r"(?i)5dlabs[_-]?(\w+)(?:\[bot\])?").expect("Invalid regex pattern")
        });

        if let Some(caps) = regex.captures(github_app) {
            if let Some(agent_match) = caps.get(1) {
                let agent_name = agent_match.as_str().to_lowercase();

                // Validate Kubernetes naming constraints
                Self::validate_k8s_name(&agent_name)?;

                Ok(agent_name)
            } else {
                Err(format!("Cannot extract agent name from: {github_app}"))
            }
        } else {
            Err(format!("Invalid GitHub App format: {github_app}"))
        }
    }

    /// Check if an agent is an implementation agent (Rex, Blaze, etc.)
    #[must_use]
    pub fn is_implementation_agent(&self, agent_name: &str) -> bool {
        self.implementation_agents.contains(agent_name)
    }

    /// Check if an agent requires an isolated workspace
    /// Implementation agents share workspace, others get isolated workspaces
    #[must_use]
    pub fn requires_isolated_workspace(&self, agent_name: &str) -> bool {
        !self.is_implementation_agent(agent_name)
    }

    /// Get the PVC name for heal CodeRuns (Remediation agents).
    ///
    /// Heal agents share a dedicated PVC with the heal monitor deployment.
    /// This is a static PVC managed by ArgoCD, not dynamically named per-service.
    ///
    /// # Returns
    /// - `heal-workspace` (shared between Heal monitor deployment and Remediation CodeRuns)
    #[must_use]
    pub fn get_heal_pvc_name(_service: &str) -> String {
        // Static PVC name - matches the ArgoCD-managed heal deployment
        "heal-workspace".to_string()
    }

    /// Get the appropriate PVC name based on agent classification.
    ///
    /// # Returns
    /// - `workspace-{service}` for implementation agents (shared workspace)
    /// - `workspace-{service}-{agent}` for non-implementation agents (isolated workspace)
    ///
    /// Note: For heal CodeRuns, use `get_heal_pvc_name` instead.
    pub fn get_pvc_name(&self, service: &str, github_app: &str) -> Result<String, String> {
        let agent_name = self.extract_agent_name(github_app)?;

        let pvc_name = if self.is_implementation_agent(&agent_name) {
            // Implementation agents share workspace
            format!("workspace-{service}")
        } else {
            // Non-implementation agents get isolated workspaces
            format!("workspace-{service}-{agent_name}")
        };

        // Ensure PVC name doesn't exceed Kubernetes limits
        if pvc_name.len() > 63 {
            // Truncate while preserving agent suffix if present
            if self.requires_isolated_workspace(&agent_name) {
                let workspace_prefix = "workspace-";
                let max_total_len: usize = 63;
                let suffix_len = agent_name.len() + 1; // hyphen before agent
                let service_max_len =
                    max_total_len.saturating_sub(workspace_prefix.len() + suffix_len);
                let truncated_service = if service.len() > service_max_len {
                    &service[..service_max_len]
                } else {
                    service
                };
                Ok(format!(
                    "{workspace_prefix}{truncated_service}-{agent_name}"
                ))
            } else {
                Ok(pvc_name[..63].to_string())
            }
        } else {
            Ok(pvc_name)
        }
    }

    /// Validate that a name meets Kubernetes naming constraints
    fn validate_k8s_name(name: &str) -> Result<(), String> {
        if name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }

        if name.len() > 63 {
            return Err(format!(
                "Name '{name}' exceeds Kubernetes limit of 63 characters"
            ));
        }

        // Check for valid characters (alphanumeric and hyphens)
        if !name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(format!("Name '{name}' contains invalid characters. Only lowercase alphanumeric and hyphens are allowed"));
        }

        // Must start and end with alphanumeric
        if name.starts_with('-') || name.ends_with('-') {
            return Err(format!("Name '{name}' cannot start or end with a hyphen"));
        }

        Ok(())
    }

    /// Add a new implementation agent to the classifier
    /// This allows runtime configuration of implementation agents
    pub fn add_implementation_agent(&mut self, agent_name: &str) {
        self.implementation_agents.insert(agent_name.to_lowercase());
    }

    /// Check if workspace is shared (for implementation agents)
    #[must_use]
    pub fn is_shared_workspace(&self, agent_name: &str) -> bool {
        self.is_implementation_agent(agent_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_agent_name_basic() {
        let classifier = AgentClassifier::new();

        // Basic patterns
        assert_eq!(classifier.extract_agent_name("5DLabs-Rex").unwrap(), "rex");
        assert_eq!(
            classifier.extract_agent_name("5DLabs-Cleo").unwrap(),
            "cleo"
        );
        assert_eq!(
            classifier.extract_agent_name("5DLabs-Tess").unwrap(),
            "tess"
        );
        assert_eq!(
            classifier.extract_agent_name("5DLabs-Blaze").unwrap(),
            "blaze"
        );
    }

    #[test]
    fn test_extract_agent_name_with_bot_suffix() {
        let classifier = AgentClassifier::new();

        // Bot suffix patterns
        assert_eq!(
            classifier.extract_agent_name("5DLabs-Rex[bot]").unwrap(),
            "rex"
        );
        assert_eq!(
            classifier.extract_agent_name("5DLabs-Cleo[bot]").unwrap(),
            "cleo"
        );
        assert_eq!(
            classifier.extract_agent_name("5DLabs-Tess[bot]").unwrap(),
            "tess"
        );
    }

    #[test]
    fn test_extract_agent_name_case_insensitive() {
        let classifier = AgentClassifier::new();

        // Case variations
        assert_eq!(classifier.extract_agent_name("5dlabs-Rex").unwrap(), "rex");
        assert_eq!(classifier.extract_agent_name("5DLABS-REX").unwrap(), "rex");
        assert_eq!(
            classifier.extract_agent_name("5DLabs-CLEO").unwrap(),
            "cleo"
        );
    }

    #[test]
    fn test_extract_agent_name_underscore_separator() {
        let classifier = AgentClassifier::new();

        // Underscore separator
        assert_eq!(classifier.extract_agent_name("5DLabs_Rex").unwrap(), "rex");
        assert_eq!(
            classifier.extract_agent_name("5DLabs_Cleo[bot]").unwrap(),
            "cleo"
        );
    }

    #[test]
    fn test_extract_agent_name_errors() {
        let classifier = AgentClassifier::new();

        // Invalid patterns
        assert!(classifier.extract_agent_name("InvalidFormat").is_err());
        assert!(classifier.extract_agent_name("RandomApp").is_err());
        assert!(classifier.extract_agent_name("").is_err());
        assert!(classifier.extract_agent_name("5DLabs").is_err());
    }

    #[test]
    fn test_agent_classification() {
        let classifier = AgentClassifier::new();

        // Implementation agents
        assert!(classifier.is_implementation_agent("rex"));
        assert!(classifier.is_implementation_agent("blaze"));

        // Non-implementation agents
        assert!(!classifier.is_implementation_agent("cleo"));
        assert!(!classifier.is_implementation_agent("tess"));
        assert!(!classifier.is_implementation_agent("nova"));
    }

    #[test]
    fn test_workspace_requirements() {
        let classifier = AgentClassifier::new();

        // Implementation agents share workspace
        assert!(!classifier.requires_isolated_workspace("rex"));
        assert!(!classifier.requires_isolated_workspace("blaze"));
        assert!(classifier.is_shared_workspace("rex"));
        assert!(classifier.is_shared_workspace("blaze"));

        // Non-implementation agents require isolation
        assert!(classifier.requires_isolated_workspace("cleo"));
        assert!(classifier.requires_isolated_workspace("tess"));
        assert!(!classifier.is_shared_workspace("cleo"));
        assert!(!classifier.is_shared_workspace("tess"));
    }

    #[test]
    fn test_pvc_naming() {
        let classifier = AgentClassifier::new();

        // Implementation agents use shared workspace
        assert_eq!(
            classifier.get_pvc_name("cto", "5DLabs-Rex").unwrap(),
            "workspace-cto"
        );
        assert_eq!(
            classifier.get_pvc_name("cto", "5DLabs-Blaze").unwrap(),
            "workspace-cto"
        );

        // Non-implementation agents get isolated workspaces
        assert_eq!(
            classifier.get_pvc_name("cto", "5DLabs-Cleo").unwrap(),
            "workspace-cto-cleo"
        );
        assert_eq!(
            classifier.get_pvc_name("cto", "5DLabs-Tess").unwrap(),
            "workspace-cto-tess"
        );
    }

    #[test]
    fn test_pvc_name_truncation() {
        let classifier = AgentClassifier::new();

        // Long service name that would exceed 63 chars
        let long_service = "very-long-service-name-that-exceeds-kubernetes-limits-for-names";

        // Should truncate but preserve agent suffix for isolated workspaces
        let result = classifier
            .get_pvc_name(long_service, "5DLabs-Cleo")
            .unwrap();
        assert!(result.len() <= 63);
        assert!(result.starts_with("workspace-"));
        assert!(result.ends_with("-cleo"));

        // Shared workspace should just truncate
        let result = classifier.get_pvc_name(long_service, "5DLabs-Rex").unwrap();
        assert!(result.len() <= 63);
        assert!(result.starts_with("workspace-"));
    }

    #[test]
    fn test_k8s_name_validation() {
        let _classifier = AgentClassifier::new();

        // Valid names
        assert!(AgentClassifier::validate_k8s_name("rex").is_ok());
        assert!(AgentClassifier::validate_k8s_name("cleo-bot").is_ok());
        assert!(AgentClassifier::validate_k8s_name("agent-123").is_ok());

        // Invalid names
        assert!(AgentClassifier::validate_k8s_name("").is_err());
        assert!(AgentClassifier::validate_k8s_name("-rex").is_err());
        assert!(AgentClassifier::validate_k8s_name("rex-").is_err());
        assert!(AgentClassifier::validate_k8s_name("Rex").is_err()); // Uppercase
        assert!(AgentClassifier::validate_k8s_name("rex_bot").is_err()); // Underscore
        assert!(AgentClassifier::validate_k8s_name("rex.bot").is_err()); // Dot

        // Too long
        let long_name = "a".repeat(64);
        assert!(AgentClassifier::validate_k8s_name(&long_name).is_err());
    }

    #[test]
    fn test_add_implementation_agent() {
        let mut classifier = AgentClassifier::new();

        // Initially not an implementation agent
        assert!(!classifier.is_implementation_agent("morgan"));

        // Add as implementation agent
        classifier.add_implementation_agent("morgan");

        // Now it should be classified as implementation agent
        assert!(classifier.is_implementation_agent("morgan"));
        assert!(!classifier.requires_isolated_workspace("morgan"));
        assert_eq!(
            classifier.get_pvc_name("cto", "5DLabs-Morgan").unwrap(),
            "workspace-cto"
        );
    }

    #[test]
    fn test_heal_pvc_naming() {
        // Heal PVC uses a static name shared with ArgoCD-managed heal deployment
        assert_eq!(
            AgentClassifier::get_heal_pvc_name("cto"),
            "heal-workspace"
        );

        // Service name is ignored - always returns static PVC name
        assert_eq!(
            AgentClassifier::get_heal_pvc_name("my-service"),
            "heal-workspace"
        );
    }
}
