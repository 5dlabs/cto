//! Tags domain facade.

use std::sync::Arc;

use crate::entities::TagStats;
use crate::errors::TasksResult;
use crate::storage::Storage;

/// Tags domain facade providing high-level tag operations
pub struct TagsDomain {
    storage: Arc<dyn Storage>,
}

impl TagsDomain {
    /// Create a new tags domain
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self { storage }
    }

    /// List all tags
    pub async fn list_tags(&self) -> TasksResult<Vec<String>> {
        self.storage.get_all_tags().await
    }

    /// List all tags with statistics
    pub async fn list_tags_with_stats(&self) -> TasksResult<Vec<TagStats>> {
        self.storage.get_tags_with_stats().await
    }

    /// Get the current active tag
    pub async fn current_tag(&self) -> TasksResult<String> {
        self.storage.get_current_tag().await
    }

    /// Switch to a different tag
    pub async fn use_tag(&self, name: &str) -> TasksResult<()> {
        self.storage.set_current_tag(name).await
    }

    /// Create a new tag
    pub async fn create_tag(
        &self,
        name: &str,
        copy_from: Option<&str>,
        description: Option<&str>,
    ) -> TasksResult<()> {
        self.storage.create_tag(name, copy_from, description).await
    }

    /// Delete a tag
    pub async fn delete_tag(&self, name: &str) -> TasksResult<()> {
        self.storage.delete_tag(name).await
    }

    /// Rename a tag
    pub async fn rename_tag(&self, old_name: &str, new_name: &str) -> TasksResult<()> {
        self.storage.rename_tag(old_name, new_name).await
    }

    /// Copy a tag
    pub async fn copy_tag(
        &self,
        source: &str,
        target: &str,
        description: Option<&str>,
    ) -> TasksResult<()> {
        self.storage.copy_tag(source, target, description).await
    }

    /// Check if a tag exists
    pub async fn tag_exists(&self, name: &str) -> TasksResult<bool> {
        self.storage.tag_exists(name).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::FileStorage;
    use tempfile::TempDir;

    async fn setup() -> (TempDir, TagsDomain) {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(FileStorage::new(temp_dir.path()));
        storage.initialize().await.unwrap();
        let domain = TagsDomain::new(storage);
        (temp_dir, domain)
    }

    #[tokio::test]
    async fn test_list_tags() {
        let (_temp, domain) = setup().await;
        let tags = domain.list_tags().await.unwrap();
        assert!(tags.contains(&"master".to_string()));
    }

    #[tokio::test]
    async fn test_create_and_use_tag() {
        let (_temp, domain) = setup().await;

        domain.create_tag("feature-1", None, None).await.unwrap();
        assert!(domain.tag_exists("feature-1").await.unwrap());

        domain.use_tag("feature-1").await.unwrap();
        assert_eq!(domain.current_tag().await.unwrap(), "feature-1");
    }
}
