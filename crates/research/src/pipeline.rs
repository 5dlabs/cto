//! Research pipeline - orchestrates the full poll-analyze-enrich-store flow.

use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;

use tasks::ai::AIProvider;

use crate::analysis::RelevanceAnalyzer;
use crate::auth::Session;
use crate::enrichment::{EnrichedLink, LinkEnricher};
use crate::storage::{IndexEntry, MarkdownWriter, ResearchEntry, ResearchIndex};
use crate::twitter::{BookmarkPoller, PollConfig, PollState};

/// Configuration for the research pipeline.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Output directory for research entries.
    pub output_dir: PathBuf,
    /// State file path.
    pub state_path: PathBuf,
    /// Index file path.
    pub index_path: PathBuf,
    /// Minimum relevance score to save.
    pub min_relevance: f32,
    /// Max bookmarks to process per run.
    pub batch_size: usize,
    /// AI model to use for analysis.
    pub model: String,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("/data/research"),
            state_path: PathBuf::from("/data/state.json"),
            index_path: PathBuf::from("/data/research/index.json"),
            min_relevance: 0.5,
            batch_size: 10,
            model: "claude-sonnet-4-20250514".to_string(),
        }
    }
}

/// Result of a single poll cycle.
#[derive(Debug, Default)]
pub struct PollCycleResult {
    /// Number of bookmarks fetched.
    pub fetched: usize,
    /// Number analyzed.
    pub analyzed: usize,
    /// Number saved (met relevance threshold).
    pub saved: usize,
    /// Number skipped (below threshold).
    pub skipped: usize,
    /// Errors encountered.
    pub errors: Vec<String>,
}

/// Research pipeline orchestrator.
pub struct Pipeline {
    config: PipelineConfig,
    session: Session,
    provider: Arc<dyn AIProvider>,
}

impl Pipeline {
    /// Create a new pipeline.
    #[must_use]
    pub fn new(config: PipelineConfig, session: Session, provider: Arc<dyn AIProvider>) -> Self {
        Self {
            config,
            session,
            provider,
        }
    }

    /// Run a single poll cycle.
    pub async fn poll_cycle(&self) -> Result<PollCycleResult> {
        let mut result = PollCycleResult::default();

        tracing::info!("Starting poll cycle");

        // Load state
        let mut state = PollState::load(&self.config.state_path)?;
        tracing::debug!(processed = state.processed.len(), "Loaded state");

        // Load index
        let mut index = ResearchIndex::load(&self.config.index_path)?;

        // Set up components
        let poll_config = PollConfig {
            batch_size: self.config.batch_size,
            ..Default::default()
        };
        let poller = BookmarkPoller::new(self.session.clone(), poll_config);
        let analyzer = RelevanceAnalyzer::new(self.provider.clone(), self.config.model.clone())?;
        let enricher = match LinkEnricher::from_env() {
            Ok(e) => Some(e),
            Err(e) => {
                tracing::warn!(error = %e, "Firecrawl not available - skipping enrichment");
                None
            }
        };
        let writer = MarkdownWriter::new(self.config.output_dir.clone());

        // Fetch new bookmarks
        let bookmarks = match poller.poll(&mut state).await {
            Ok(b) => b,
            Err(e) => {
                state.record_failure();
                state.save(&self.config.state_path)?;
                return Err(e);
            }
        };
        result.fetched = bookmarks.len();

        if bookmarks.is_empty() {
            tracing::info!("No new bookmarks to process");
            state.save(&self.config.state_path)?;
            return Ok(result);
        }

        // Process each bookmark
        for bookmark in bookmarks {
            tracing::info!(id = %bookmark.id, author = %bookmark.author.handle, "Processing bookmark");

            // Analyze relevance
            let relevance = match analyzer.analyze(&bookmark).await {
                Ok(r) => r,
                Err(e) => {
                    tracing::warn!(id = %bookmark.id, error = %e, "Analysis failed");
                    result.errors.push(format!("{}: {e}", bookmark.id));
                    // Mark as processed to avoid duplicate retries on next poll
                    state.mark_processed(&bookmark.id);
                    continue;
                }
            };
            result.analyzed += 1;

            // Check threshold
            if !relevance.is_relevant(self.config.min_relevance) {
                tracing::debug!(
                    id = %bookmark.id,
                    score = relevance.score,
                    threshold = self.config.min_relevance,
                    "Below relevance threshold"
                );
                state.mark_processed(&bookmark.id);
                result.skipped += 1;
                continue;
            }

            // Enrich links if needed
            let enriched: Vec<EnrichedLink> = if relevance.should_enrich {
                if let Some(ref enricher) = enricher {
                    match enricher.enrich(&bookmark).await {
                        Ok(links) => links,
                        Err(e) => {
                            tracing::warn!(id = %bookmark.id, error = %e, "Enrichment failed");
                            Vec::new()
                        }
                    }
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };

            // Create research entry
            let mut entry = ResearchEntry::new(bookmark.clone(), relevance);
            entry = entry.with_enriched(enriched);
            entry.generate_tags();

            // Write to markdown
            let path = match writer.write(&entry) {
                Ok(p) => p,
                Err(e) => {
                    tracing::error!(id = %bookmark.id, error = %e, "Failed to write entry");
                    result
                        .errors
                        .push(format!("{}: write failed: {e}", bookmark.id));
                    continue;
                }
            };

            // Add to index
            index.add(IndexEntry {
                id: entry.bookmark.id.clone(),
                author: entry.bookmark.author.handle.clone(),
                preview: truncate_preview(&entry.bookmark.text),
                score: entry.relevance.score,
                categories: entry.relevance.categories.clone(),
                processed_at: entry.processed_at,
                path: path.to_string_lossy().to_string(),
            });

            state.mark_processed(&bookmark.id);
            result.saved += 1;

            tracing::info!(
                id = %bookmark.id,
                score = entry.relevance.score,
                categories = ?entry.relevance.categories,
                "Saved research entry"
            );
        }

        // Save state and index
        state.save(&self.config.state_path)?;
        index.save(&self.config.index_path)?;

        tracing::info!(
            fetched = result.fetched,
            analyzed = result.analyzed,
            saved = result.saved,
            skipped = result.skipped,
            errors = result.errors.len(),
            "Poll cycle complete"
        );

        Ok(result)
    }
}

/// Truncate text for preview, respecting UTF-8 character boundaries.
fn truncate_preview(text: &str) -> String {
    const MAX_CHARS: usize = 100;

    let char_count = text.chars().count();
    if char_count <= MAX_CHARS {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(MAX_CHARS).collect();
        format!("{truncated}...")
    }
}
