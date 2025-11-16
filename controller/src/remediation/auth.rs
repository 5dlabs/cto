use anyhow::Result;
use dashmap::DashMap;
use std::collections::HashSet;
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// Author validation system for QA feedback comments
#[derive(Debug)]
pub struct AuthorValidator {
    /// Set of explicitly allowed authors
    allowed_authors: HashSet<String>,
    /// Cache for authorization results with TTL
    auth_cache: DashMap<String, (bool, Instant)>,
    /// Cache TTL duration
    cache_ttl: Duration,
    /// Team prefixes that are automatically allowed
    allowed_team_prefixes: Vec<String>,
}

impl AuthorValidator {
    /// Create a new validator with default settings
    #[must_use]
    pub fn new() -> Self {
        let mut allowed_authors = HashSet::new();

        // Core QA bot
        allowed_authors.insert("5DLabs-Tess".to_string());
        allowed_authors.insert("5DLabs-Tess[bot]".to_string());

        // Additional approved reviewers (configurable)
        allowed_authors.insert("approved-reviewer-1".to_string());
        allowed_authors.insert("approved-reviewer-2".to_string());

        Self {
            allowed_authors,
            auth_cache: DashMap::new(),
            cache_ttl: Duration::from_secs(300), // 5 minutes
            allowed_team_prefixes: vec!["5DLabs-".to_string()],
        }
    }

    /// Create validator with custom settings
    #[must_use]
    pub fn with_config(
        allowed_authors: HashSet<String>,
        cache_ttl_seconds: u64,
        team_prefixes: Vec<String>,
    ) -> Self {
        Self {
            allowed_authors,
            auth_cache: DashMap::new(),
            cache_ttl: Duration::from_secs(cache_ttl_seconds),
            allowed_team_prefixes: team_prefixes,
        }
    }

    /// Validate if an author is authorized to provide feedback
    pub fn validate_author(&self, author: &str) -> Result<()> {
        // Check cache first
        if let Some(entry) = self.auth_cache.get(author) {
            let (is_valid, timestamp) = entry.value();
            if timestamp.elapsed() < self.cache_ttl {
                return if *is_valid {
                    info!("Author '{}' authorized (cached)", author);
                    Ok(())
                } else {
                    warn!("Author '{}' not authorized (cached)", author);
                    Err(anyhow::anyhow!(
                        "Author '{author}' is not authorized to provide feedback"
                    ))
                };
            }
        }

        // Check against allowed authors and team prefixes
        let is_authorized =
            self.is_author_explicitly_allowed(author) || self.is_team_member(author);

        // Cache the result
        self.auth_cache
            .insert(author.to_string(), (is_authorized, Instant::now()));

        if is_authorized {
            info!("Author '{}' authorized", author);
            Ok(())
        } else {
            warn!(
                "Author '{}' not authorized - not in allowlist and doesn't match team patterns",
                author
            );
            Err(anyhow::anyhow!("Author '{author}' is not authorized to provide feedback. Contact an administrator to be added to the approved reviewers list."))
        }
    }

    /// Check if author is explicitly in the allowed list
    fn is_author_explicitly_allowed(&self, author: &str) -> bool {
        self.allowed_authors.contains(author)
    }

    /// Check if author matches team patterns
    fn is_team_member(&self, author: &str) -> bool {
        self.allowed_team_prefixes
            .iter()
            .any(|prefix| author.starts_with(prefix))
    }

    /// Add an author to the approved list
    pub fn add_approved_author(&mut self, author: String) -> Result<()> {
        if author.trim().is_empty() {
            return Err(anyhow::anyhow!("Author name cannot be empty"));
        }

        if self.allowed_authors.insert(author.clone()) {
            info!("Added '{}' to approved authors list", author);
            // Clear cache to force re-validation
            self.clear_cache();
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Author '{author}' is already in the approved list"
            ))
        }
    }

    /// Remove an author from the approved list
    pub fn remove_approved_author(&mut self, author: &str) -> Result<()> {
        if self.allowed_authors.remove(author) {
            info!("Removed '{}' from approved authors list", author);
            // Clear cache to force re-validation
            self.clear_cache();
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Author '{author}' is not in the approved list"
            ))
        }
    }

    /// Add a team prefix pattern
    pub fn add_team_prefix(&mut self, prefix: String) -> Result<()> {
        if prefix.trim().is_empty() {
            return Err(anyhow::anyhow!("Team prefix cannot be empty"));
        }

        if self.allowed_team_prefixes.contains(&prefix) {
            return Err(anyhow::anyhow!(
                "Team prefix '{prefix}' is already configured"
            ));
        }

        self.allowed_team_prefixes.push(prefix.clone());
        info!("Added team prefix '{}' to allowed patterns", prefix);
        // Clear cache to force re-validation
        self.clear_cache();
        Ok(())
    }

    /// Remove a team prefix pattern
    pub fn remove_team_prefix(&mut self, prefix: &str) -> Result<()> {
        if let Some(pos) = self.allowed_team_prefixes.iter().position(|p| p == prefix) {
            self.allowed_team_prefixes.remove(pos);
            info!("Removed team prefix '{}' from allowed patterns", prefix);
            // Clear cache to force re-validation
            self.clear_cache();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Team prefix '{prefix}' is not configured"))
        }
    }

    /// Get list of approved authors
    #[must_use]
    pub fn get_approved_authors(&self) -> Vec<String> {
        self.allowed_authors.iter().cloned().collect()
    }

    /// Get list of team prefixes
    #[must_use]
    pub fn get_team_prefixes(&self) -> Vec<String> {
        self.allowed_team_prefixes.clone()
    }

    /// Clear the authorization cache
    pub fn clear_cache(&self) {
        let cleared_count = self.auth_cache.len();
        self.auth_cache.clear();
        info!("Cleared authorization cache ({} entries)", cleared_count);
    }

    /// Get cache statistics
    #[must_use]
    pub fn get_cache_stats(&self) -> (usize, usize, usize) {
        let total_entries = self.auth_cache.len();
        let valid_entries = self
            .auth_cache
            .iter()
            .filter(|entry| entry.value().1.elapsed() < self.cache_ttl)
            .count();
        let expired_entries = total_entries - valid_entries;

        (total_entries, valid_entries, expired_entries)
    }

    /// Check if an author would be authorized without caching
    #[must_use]
    pub fn check_author_without_cache(&self, author: &str) -> bool {
        self.is_author_explicitly_allowed(author) || self.is_team_member(author)
    }

    /// Get cache TTL in seconds
    #[must_use]
    pub fn get_cache_ttl_seconds(&self) -> u64 {
        self.cache_ttl.as_secs()
    }

    /// Set cache TTL
    pub fn set_cache_ttl(&mut self, seconds: u64) {
        self.cache_ttl = Duration::from_secs(seconds);
        info!("Set cache TTL to {} seconds", seconds);
        // Clear cache when TTL changes
        self.clear_cache();
    }

    /// Pre-populate cache with known authors (useful for testing)
    pub fn preload_cache(&self, authors: Vec<String>, authorized: bool) {
        let now = Instant::now();
        let count = authors.len();
        for author in authors {
            self.auth_cache.insert(author, (authorized, now));
        }
        info!("Pre-loaded {} authors into cache", count);
    }
}

impl Default for AuthorValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe wrapper for shared validator access
#[derive(Debug, Clone)]
pub struct SharedAuthorValidator {
    inner: std::sync::Arc<std::sync::RwLock<AuthorValidator>>,
}

impl SharedAuthorValidator {
    /// Create a new shared validator
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: std::sync::Arc::new(std::sync::RwLock::new(AuthorValidator::new())),
        }
    }

    /// Validate author with shared access
    pub fn validate_author(&self, author: &str) -> Result<()> {
        let validator = self
            .inner
            .read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire read lock: {e}"))?;
        validator.validate_author(author)
    }

    /// Add approved author with shared access
    pub fn add_approved_author(&self, author: String) -> Result<()> {
        let mut validator = self
            .inner
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire write lock: {e}"))?;
        validator.add_approved_author(author)
    }

    /// Get approved authors with shared access
    pub fn get_approved_authors(&self) -> Result<Vec<String>> {
        let validator = self
            .inner
            .read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire read lock: {e}"))?;
        Ok(validator.get_approved_authors())
    }

    /// Clear cache with shared access
    pub fn clear_cache(&self) -> Result<()> {
        let validator = self
            .inner
            .read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire read lock: {e}"))?;
        validator.clear_cache();
        Ok(())
    }
}

impl Default for SharedAuthorValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_default_validator_creation() {
        let validator = AuthorValidator::new();

        // Should contain core QA bot
        assert!(validator.allowed_authors.contains("5DLabs-Tess"));
        assert!(validator
            .allowed_team_prefixes
            .contains(&"5DLabs-".to_string()));
    }

    #[test]
    fn test_validate_author_core_qa_bot() {
        let validator = AuthorValidator::new();
        assert!(validator.validate_author("5DLabs-Tess").is_ok());
        assert!(validator.validate_author("5DLabs-Tess[bot]").is_ok());
    }

    #[test]
    fn test_validate_author_team_member() {
        let validator = AuthorValidator::new();
        assert!(validator.validate_author("5DLabs-Developer").is_ok());
        assert!(validator.validate_author("5DLabs-QA").is_ok());
    }

    #[test]
    fn test_validate_author_unauthorized() {
        let validator = AuthorValidator::new();
        let result = validator.validate_author("random-user");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not authorized"));
    }

    #[test]
    fn test_add_remove_approved_author() {
        let mut validator = AuthorValidator::new();

        // Add author
        assert!(validator
            .add_approved_author("test-author".to_string())
            .is_ok());
        assert!(validator.allowed_authors.contains("test-author"));
        assert!(validator.validate_author("test-author").is_ok());

        // Try to add again (should fail)
        assert!(validator
            .add_approved_author("test-author".to_string())
            .is_err());

        // Remove author
        assert!(validator.remove_approved_author("test-author").is_ok());
        assert!(!validator.allowed_authors.contains("test-author"));
        assert!(validator.validate_author("test-author").is_err());
    }

    #[test]
    fn test_team_prefix_management() {
        let mut validator = AuthorValidator::new();

        // Add team prefix
        assert!(validator.add_team_prefix("MyTeam-".to_string()).is_ok());
        assert!(validator
            .allowed_team_prefixes
            .contains(&"MyTeam-".to_string()));
        assert!(validator.validate_author("MyTeam-Developer").is_ok());

        // Remove team prefix
        assert!(validator.remove_team_prefix("MyTeam-").is_ok());
        assert!(!validator
            .allowed_team_prefixes
            .contains(&"MyTeam-".to_string()));
        assert!(validator.validate_author("MyTeam-Developer").is_err());
    }

    #[test]
    fn test_caching_behavior() {
        let validator = AuthorValidator::new();

        // First validation should work and cache
        assert!(validator.validate_author("5DLabs-Tess").is_ok());
        assert_eq!(validator.auth_cache.len(), 1);

        // Second validation should use cache
        assert!(validator.validate_author("5DLabs-Tess").is_ok());

        // Check cache stats
        let (total, valid, expired) = validator.get_cache_stats();
        assert_eq!(total, 1);
        assert_eq!(valid, 1);
        assert_eq!(expired, 0);
    }

    #[test]
    fn test_cache_expiration() {
        let mut validator = AuthorValidator::new();
        validator.set_cache_ttl(1); // 1 second TTL

        // Add to cache
        assert!(validator.validate_author("5DLabs-Tess").is_ok());

        // Wait for expiration
        thread::sleep(Duration::from_secs(2));

        // Cache should still have entry but it should be considered expired
        let (total, valid, expired) = validator.get_cache_stats();
        assert_eq!(total, 1);
        assert_eq!(valid, 0);
        assert_eq!(expired, 1);
    }

    #[test]
    fn test_clear_cache() {
        let validator = AuthorValidator::new();

        // Add some entries to cache
        let _ = validator.validate_author("5DLabs-Tess");
        let _ = validator.validate_author("random-user");

        assert_eq!(validator.auth_cache.len(), 2);

        // Clear cache
        validator.clear_cache();
        assert_eq!(validator.auth_cache.len(), 0);
    }

    #[test]
    fn test_shared_validator() {
        let shared_validator = SharedAuthorValidator::new();

        // Test validation
        assert!(shared_validator.validate_author("5DLabs-Tess").is_ok());

        // Test adding author
        assert!(shared_validator
            .add_approved_author("test-author".to_string())
            .is_ok());

        // Test getting authors
        let authors = shared_validator.get_approved_authors().unwrap();
        assert!(authors.contains(&"test-author".to_string()));

        // Test cache clearing
        assert!(shared_validator.clear_cache().is_ok());
    }

    #[test]
    fn test_empty_author_validation() {
        let mut validator = AuthorValidator::new();

        // Empty author should fail
        assert!(validator.add_approved_author(String::new()).is_err());
        assert!(validator.add_approved_author("   ".to_string()).is_err());

        // Empty prefix should fail
        assert!(validator.add_team_prefix(String::new()).is_err());
        assert!(validator.add_team_prefix("   ".to_string()).is_err());
    }

    #[test]
    fn test_preload_cache() {
        let validator = AuthorValidator::new();

        let test_authors = vec!["preload-1".to_string(), "preload-2".to_string()];

        validator.preload_cache(test_authors.clone(), true);

        assert_eq!(validator.auth_cache.len(), 2);

        // Should be authorized due to preload
        for author in &test_authors {
            assert!(validator.validate_author(author).is_ok());
        }
    }

    #[test]
    fn test_cache_ttl_management() {
        let mut validator = AuthorValidator::new();

        assert_eq!(validator.get_cache_ttl_seconds(), 300); // Default 5 minutes

        validator.set_cache_ttl(600); // 10 minutes
        assert_eq!(validator.get_cache_ttl_seconds(), 600);
    }
}
