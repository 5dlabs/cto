//! `OpenMemory` client for historical context.
//!
//! Provides integration with the `OpenMemory` service for:
//! - Querying past CI failures and their solutions
//! - Tracking agent performance statistics
//! - Learning from routing decisions
//! - Storing remediation outcomes

use anyhow::{Context as _, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use super::types::{Agent, CiFailure, CiFailureType, HistoricalContext, MemoryEntry};

/// `OpenMemory` client configuration.
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    /// `OpenMemory` service URL
    pub url: String,
    /// Agent namespace for memories
    pub namespace: String,
    /// Maximum number of results to return
    pub query_limit: usize,
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            url: "http://openmemory.cto.svc.cluster.local:8080".to_string(),
            namespace: "agent/healer".to_string(),
            query_limit: 20,
            timeout_ms: 5000,
        }
    }
}

/// `OpenMemory` client for querying and storing memories.
pub struct MemoryClient {
    client: Client,
    config: MemoryConfig,
}

/// Memory query request.
#[derive(Debug, Serialize)]
struct MemoryQuery {
    query: String,
    namespace: String,
    limit: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    filters: Option<QueryFilters>,
}

/// Query filters for memory searches.
#[derive(Debug, Serialize)]
struct QueryFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure_type: Option<String>,
}

/// Memory search result.
#[derive(Debug, Deserialize)]
struct MemorySearchResult {
    memories: Vec<MemoryRecord>,
}

/// A single memory record.
#[derive(Debug, Deserialize)]
struct MemoryRecord {
    id: String,
    content: String,
    #[serde(default)]
    metadata: MemoryMetadata,
    #[serde(default)]
    salience: f32,
}

/// Memory metadata.
#[derive(Debug, Default, Deserialize)]
#[allow(dead_code)] // Fields read via deserialization but not always accessed
struct MemoryMetadata {
    #[serde(default)]
    category: String,
    #[serde(default)]
    agent: String,
    #[serde(default)]
    failure_type: String,
    #[serde(default)]
    outcome: String,
    #[serde(default)]
    workflow_name: String,
    #[serde(default)]
    repository: String,
}

/// Memory creation request.
#[derive(Debug, Serialize)]
struct MemoryCreate {
    content: String,
    namespace: String,
    metadata: MemoryCreateMetadata,
}

/// Metadata for creating a memory.
#[derive(Debug, Serialize)]
struct MemoryCreateMetadata {
    category: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    outcome: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    workflow_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repository: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    workflow_run_id: Option<u64>,
}

/// Agent performance statistics.
#[derive(Debug, Default)]
pub struct AgentStats {
    /// Agent name
    pub agent: String,
    /// Total attempts
    pub total_attempts: u32,
    /// Successful fixes
    pub successes: u32,
    /// Failed attempts
    pub failures: u32,
    /// Average attempts to fix
    pub avg_attempts_to_fix: f32,
}

impl MemoryClient {
    /// Create a new memory client.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn new(config: MemoryConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_millis(config.timeout_ms))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client, config })
    }

    /// Query for similar past failures.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails.
    pub async fn query_similar_failures(
        &self,
        failure: &CiFailure,
        failure_type: Option<&CiFailureType>,
    ) -> Result<HistoricalContext> {
        let query = format!(
            "CI failure in {} workflow: {} {}",
            failure.workflow_name,
            failure_type.map_or("general", CiFailureType::short_name),
            failure
                .job_name
                .as_ref()
                .map_or(String::new(), |j| format!("job: {j}"))
        );

        let filters = Some(QueryFilters {
            category: Some("ci-failures".to_string()),
            agent: None,
            failure_type: failure_type.map(|ft| ft.short_name().to_string()),
        });

        let results = self.search(&query, filters).await?;

        // Extract relevant information
        let similar_failures: Vec<MemoryEntry> = results
            .iter()
            .filter(|r| !r.metadata.outcome.is_empty())
            .take(5)
            .map(|r| MemoryEntry {
                id: r.id.clone(),
                content: format!(
                    "[{}] {}: {} (by {})",
                    r.metadata.outcome, r.metadata.workflow_name, r.content, r.metadata.agent
                ),
                score: f64::from(r.salience),
                metadata: std::collections::HashMap::new(),
            })
            .collect();

        let known_solutions: Vec<MemoryEntry> = results
            .iter()
            .filter(|r| r.metadata.outcome == "success")
            .take(3)
            .map(|r| MemoryEntry {
                id: r.id.clone(),
                content: r.content.clone(),
                score: f64::from(r.salience),
                metadata: std::collections::HashMap::new(),
            })
            .collect();

        Ok(HistoricalContext {
            similar_failures,
            known_solutions,
            agent_success_patterns: Vec::new(), // Populated by separate query
        })
    }

    /// Query agent success rates for a given failure type.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails.
    pub async fn query_agent_success_rates(
        &self,
        failure_type: Option<&CiFailureType>,
    ) -> Result<Vec<(String, f32)>> {
        let query = format!(
            "agent performance {} success rate",
            failure_type.map_or("general", CiFailureType::short_name)
        );

        let filters = Some(QueryFilters {
            category: Some("agent-performance".to_string()),
            agent: None,
            failure_type: failure_type.map(|ft| ft.short_name().to_string()),
        });

        let results = self.search(&query, filters).await?;

        // Aggregate by agent
        let mut agent_stats: std::collections::HashMap<String, (u32, u32)> =
            std::collections::HashMap::new();

        for record in results {
            if record.metadata.agent.is_empty() {
                continue;
            }

            let entry = agent_stats
                .entry(record.metadata.agent.clone())
                .or_insert((0, 0));
            entry.1 += 1; // Total

            if record.metadata.outcome == "success" {
                entry.0 += 1; // Successes
            }
        }

        // Calculate success rates
        #[allow(clippy::cast_precision_loss)]
        let mut rates: Vec<(String, f32)> = agent_stats
            .into_iter()
            .map(|(agent, (successes, total))| {
                let rate = if total > 0 {
                    successes as f32 / total as f32
                } else {
                    0.0
                };
                (agent, rate)
            })
            .collect();

        // Sort by success rate descending
        rates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(rates)
    }

    /// Store a remediation outcome.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails.
    pub async fn store_remediation_outcome(
        &self,
        failure: &CiFailure,
        failure_type: Option<&CiFailureType>,
        agent: Agent,
        outcome: &str,
        description: &str,
    ) -> Result<()> {
        let content = format!(
            "{} {} in {}: {}",
            outcome.to_uppercase(),
            failure_type.map_or("General", CiFailureType::short_name),
            failure.workflow_name,
            description
        );

        let metadata = MemoryCreateMetadata {
            category: "ci-failures".to_string(),
            agent: Some(agent.name().to_string()),
            failure_type: failure_type.map(|ft| ft.short_name().to_string()),
            outcome: Some(outcome.to_string()),
            workflow_name: Some(failure.workflow_name.clone()),
            repository: Some(failure.repository.clone()),
            workflow_run_id: Some(failure.workflow_run_id),
        };

        self.create_memory(&content, metadata).await?;

        // Also update agent performance stats
        let perf_content = format!(
            "Agent {} {} fixing {} failure",
            agent.name(),
            outcome,
            failure_type.map_or("general", CiFailureType::short_name)
        );

        let perf_metadata = MemoryCreateMetadata {
            category: "agent-performance".to_string(),
            agent: Some(agent.name().to_string()),
            failure_type: failure_type.map(|ft| ft.short_name().to_string()),
            outcome: Some(outcome.to_string()),
            workflow_name: None,
            repository: None,
            workflow_run_id: None,
        };

        self.create_memory(&perf_content, perf_metadata).await?;

        info!(
            "Stored remediation outcome: {} by {} for workflow {}",
            outcome,
            agent.name(),
            failure.workflow_name
        );

        Ok(())
    }

    /// Store a routing decision for learning.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails.
    pub async fn store_routing_decision(
        &self,
        failure_type: Option<&CiFailureType>,
        selected_agent: Agent,
        actual_agent: Option<Agent>,
        success: bool,
    ) -> Result<()> {
        let mismatch = actual_agent.is_some_and(|a| a != selected_agent);

        let content = if mismatch {
            format!(
                "Routing mismatch: selected {} but {} was needed for {} ({})",
                selected_agent.name(),
                actual_agent.map_or("unknown", Agent::name),
                failure_type.map_or("general", CiFailureType::short_name),
                if success { "eventually succeeded" } else { "failed" }
            )
        } else {
            format!(
                "Routing {} for {}: selected {}",
                if success { "SUCCESS" } else { "FAILURE" },
                failure_type.map_or("general", CiFailureType::short_name),
                selected_agent.name()
            )
        };

        let metadata = MemoryCreateMetadata {
            category: "routing-decisions".to_string(),
            agent: Some(selected_agent.name().to_string()),
            failure_type: failure_type.map(|ft| ft.short_name().to_string()),
            outcome: Some(if success {
                "success".to_string()
            } else {
                "failure".to_string()
            }),
            workflow_name: None,
            repository: None,
            workflow_run_id: None,
        };

        self.create_memory(&content, metadata).await?;

        if mismatch {
            warn!(
                "Routing mismatch detected: {} -> {} for {}",
                selected_agent.name(),
                actual_agent.map_or("unknown", Agent::name),
                failure_type.map_or("general", CiFailureType::short_name)
            );
        }

        Ok(())
    }

    /// Store an escalation event.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails.
    pub async fn store_escalation(
        &self,
        failure: &CiFailure,
        failure_type: Option<&CiFailureType>,
        attempts: u32,
        reason: &str,
    ) -> Result<()> {
        let content = format!(
            "ESCALATION after {} attempts: {} in {} - {}",
            attempts,
            failure_type.map_or("General", CiFailureType::short_name),
            failure.workflow_name,
            reason
        );

        let metadata = MemoryCreateMetadata {
            category: "escalations".to_string(),
            agent: None,
            failure_type: failure_type.map(|ft| ft.short_name().to_string()),
            outcome: Some("escalated".to_string()),
            workflow_name: Some(failure.workflow_name.clone()),
            repository: Some(failure.repository.clone()),
            workflow_run_id: Some(failure.workflow_run_id),
        };

        self.create_memory(&content, metadata).await?;

        info!(
            "Stored escalation for workflow {} after {} attempts",
            failure.workflow_name, attempts
        );

        Ok(())
    }

    /// Search memories.
    async fn search(
        &self,
        query: &str,
        filters: Option<QueryFilters>,
    ) -> Result<Vec<MemoryRecord>> {
        let request = MemoryQuery {
            query: query.to_string(),
            namespace: self.config.namespace.clone(),
            limit: self.config.query_limit,
            filters,
        };

        let url = format!("{}/api/v1/search", self.config.url);

        debug!("Querying OpenMemory: {query}");

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to query OpenMemory")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("OpenMemory query failed: {} - {}", status, body);
            return Ok(Vec::new()); // Return empty on error to not block
        }

        let result: MemorySearchResult = response
            .json()
            .await
            .context("Failed to parse OpenMemory response")?;

        debug!("Found {} memories", result.memories.len());

        Ok(result.memories)
    }

    /// Create a new memory.
    async fn create_memory(&self, content: &str, metadata: MemoryCreateMetadata) -> Result<()> {
        let request = MemoryCreate {
            content: content.to_string(),
            namespace: self.config.namespace.clone(),
            metadata,
        };

        let url = format!("{}/api/v1/memories", self.config.url);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to create memory")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("Failed to create memory: {} - {}", status, body);
            // Don't fail the operation, just log
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_config_default() {
        let config = MemoryConfig::default();
        assert!(config.url.contains("openmemory"));
        assert_eq!(config.namespace, "agent/healer");
        assert_eq!(config.query_limit, 20);
    }
}
