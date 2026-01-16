//! Space model - scope for skills (per project or user).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A space groups skills by project or user scope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Space {
    /// Unique identifier.
    pub id: Uuid,

    /// Human-readable name for the space.
    pub name: String,

    /// Optional project identifier (e.g., Linear project ID, GitHub repo).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,

    /// Optional user identifier for per-user spaces.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,

    /// Additional metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,

    /// When this space was created.
    pub created_at: DateTime<Utc>,
}

impl Space {
    /// Create a new space with just a name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            project_id: None,
            user_id: None,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            created_at: Utc::now(),
        }
    }

    /// Create a project-scoped space.
    #[must_use]
    pub fn for_project(name: impl Into<String>, project_id: impl Into<String>) -> Self {
        Self {
            project_id: Some(project_id.into()),
            ..Self::new(name)
        }
    }

    /// Create a user-scoped space.
    #[must_use]
    pub fn for_user(name: impl Into<String>, user_id: impl Into<String>) -> Self {
        Self {
            user_id: Some(user_id.into()),
            ..Self::new(name)
        }
    }

    /// Add metadata to the space.
    pub fn set_metadata(&mut self, key: &str, value: serde_json::Value) {
        if let serde_json::Value::Object(ref mut map) = self.metadata {
            map.insert(key.to_string(), value);
        }
    }

    /// Get metadata value.
    #[must_use]
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_space_creation() {
        let space = Space::new("Test Space");
        assert_eq!(space.name, "Test Space");
        assert!(space.project_id.is_none());
        assert!(space.user_id.is_none());
    }

    #[test]
    fn test_project_space() {
        let space = Space::for_project("My Project", "proj-123");
        assert_eq!(space.project_id, Some("proj-123".to_string()));
    }

    #[test]
    fn test_user_space() {
        let space = Space::for_user("User Skills", "user@example.com");
        assert_eq!(space.user_id, Some("user@example.com".to_string()));
    }

    #[test]
    fn test_metadata() {
        let mut space = Space::new("Test");
        space.set_metadata("repo", serde_json::json!("5dlabs/cto"));

        assert_eq!(
            space.get_metadata("repo"),
            Some(&serde_json::json!("5dlabs/cto"))
        );
    }
}
