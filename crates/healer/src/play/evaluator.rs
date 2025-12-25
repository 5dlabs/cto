//! Probe-based evaluation using LLM for context engineering quality assessment.
//!
//! This module provides LLM-powered evaluation of agent sessions by asking
//! targeted probe questions and scoring the responses. This addresses the
//! finding that traditional metrics (ROUGE, embedding similarity) fail to
//! capture functional compression quality.
//!
//! Reference: Agent-Skills-for-Context-Engineering/skills/evaluation

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info, warn};

use super::types::{ArtifactTrail, EvaluationProbe, EvaluationResults, ProbeResult, ProbeType};

/// Configuration for the probe evaluator.
#[derive(Debug, Clone)]
pub struct EvaluatorConfig {
    /// URL of the tools-server or LLM endpoint
    pub llm_endpoint: String,
    /// Model to use for evaluation
    pub model: String,
    /// Request timeout
    pub timeout: Duration,
    /// Minimum score to pass (0.0-1.0)
    pub pass_threshold: f32,
}

impl Default for EvaluatorConfig {
    fn default() -> Self {
        Self {
            llm_endpoint: "http://tools-server:8080/v1/chat/completions".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            timeout: Duration::from_secs(30),
            pass_threshold: 0.7,
        }
    }
}

/// Request format for OpenAI-compatible chat API.
#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
}

/// Chat message format.
#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

/// Response format from OpenAI-compatible chat API.
#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
}

#[derive(Debug, Deserialize)]
struct ChatMessageResponse {
    content: String,
}

/// LLM-powered probe evaluator for context engineering quality assessment.
///
/// Sends probe questions to an LLM and scores the responses against expected
/// keywords to measure how well an agent retained critical information.
pub struct ProbeEvaluator {
    config: EvaluatorConfig,
    http_client: reqwest::Client,
}

impl ProbeEvaluator {
    /// Create a new probe evaluator with the given configuration.
    #[must_use]
    pub fn new(config: EvaluatorConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .unwrap_or_default();

        Self {
            config,
            http_client,
        }
    }

    /// Create a probe evaluator with default configuration.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(EvaluatorConfig::default())
    }

    /// Format the context for inclusion in a probe question.
    fn format_context(artifact_trail: &ArtifactTrail) -> String {
        use std::fmt::Write;

        let mut context = String::new();

        context.push_str("## Session Context\n\n");

        if !artifact_trail.files_created.is_empty() {
            context.push_str("### Files Created\n");
            for file in &artifact_trail.files_created {
                let _ = writeln!(context, "- {file}");
            }
            context.push('\n');
        }

        if !artifact_trail.files_modified.is_empty() {
            context.push_str("### Files Modified\n");
            for (file, summary) in &artifact_trail.files_modified {
                let _ = writeln!(context, "- {file}: {summary}");
            }
            context.push('\n');
        }

        if !artifact_trail.files_read.is_empty() {
            context.push_str("### Files Read\n");
            for file in &artifact_trail.files_read {
                let _ = writeln!(context, "- {file}");
            }
            context.push('\n');
        }

        if !artifact_trail.decisions_made.is_empty() {
            context.push_str("### Decisions Made\n");
            for decision in &artifact_trail.decisions_made {
                let _ = writeln!(context, "- {decision}");
            }
            context.push('\n');
        }

        context
    }

    /// Build a prompt for a probe question.
    fn build_prompt(probe: &EvaluationProbe, context: &str) -> String {
        format!(
            "You are evaluating an AI agent's knowledge retention. Based on the session context provided, answer the following question concisely and accurately.

{context}

---

**Question:** {question}

Answer the question based ONLY on the context provided above. Be specific and include relevant details like file names, error messages, or decision rationale as applicable.

**Answer:**",
            context = context,
            question = probe.question
        )
    }

    /// Evaluate a single probe by asking the LLM.
    pub async fn evaluate_probe(
        &self,
        probe: &EvaluationProbe,
        context: &str,
    ) -> Result<ProbeResult> {
        let prompt = Self::build_prompt(probe, context);

        debug!(
            probe_type = ?probe.probe_type,
            question = %probe.question,
            "Evaluating probe"
        );

        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt,
            }],
            max_tokens: 500,
            temperature: 0.0, // Deterministic for evaluation
        };

        let response = self
            .http_client
            .post(&self.config.llm_endpoint)
            .json(&request)
            .send()
            .await
            .context("Failed to send request to LLM endpoint")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!(
                status = %status,
                body = %body,
                "LLM request failed"
            );
            return Ok(ProbeResult {
                probe: probe.clone(),
                response: String::new(),
                score: 0.0,
                passed: false,
                notes: Some(format!("LLM request failed: {status}")),
            });
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .context("Failed to parse LLM response")?;

        let answer = chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        // Score the response against expected keywords
        let score = probe.score_response(&answer);
        let passed = score >= self.config.pass_threshold;

        debug!(
            probe_type = ?probe.probe_type,
            score = %score,
            passed = %passed,
            "Probe evaluated"
        );

        Ok(ProbeResult {
            probe: probe.clone(),
            response: answer,
            score,
            passed,
            notes: None,
        })
    }

    /// Run all probes and aggregate results.
    pub async fn run_evaluation(
        &self,
        probes: Vec<EvaluationProbe>,
        artifact_trail: &ArtifactTrail,
    ) -> Result<EvaluationResults> {
        let context = Self::format_context(artifact_trail);

        info!(
            probe_count = %probes.len(),
            "Running probe-based evaluation"
        );

        let mut results = Vec::with_capacity(probes.len());

        for probe in probes {
            match self.evaluate_probe(&probe, &context).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    warn!(error = %e, "Failed to evaluate probe, using fallback score");
                    results.push(ProbeResult {
                        probe: probe.clone(),
                        response: String::new(),
                        score: 0.0,
                        passed: false,
                        notes: Some(format!("Evaluation error: {e}")),
                    });
                }
            }
        }

        let mut evaluation = EvaluationResults::from_probes_with_threshold(
            results,
            self.config.pass_threshold,
        );
        evaluation = evaluation.with_artifact_trail(artifact_trail.clone());

        info!(
            overall_score = %evaluation.overall_score,
            passed = %evaluation.passed,
            "Evaluation complete"
        );

        Ok(evaluation)
    }

    /// Quick evaluation without LLM - uses keyword matching only.
    ///
    /// Useful for testing or when LLM is unavailable.
    pub fn evaluate_offline(
        &self,
        probes: Vec<EvaluationProbe>,
        artifact_trail: &ArtifactTrail,
    ) -> EvaluationResults {
        let context = Self::format_context(artifact_trail);

        let results: Vec<ProbeResult> = probes
            .into_iter()
            .map(|probe| {
                // For offline evaluation, check if expected keywords appear in context
                let score = if probe.expected_keywords.is_empty() {
                    0.5 // No keywords = neutral
                } else {
                    let context_lower = context.to_lowercase();
                    let matches = probe
                        .expected_keywords
                        .iter()
                        .filter(|kw| context_lower.contains(&kw.to_lowercase()))
                        .count();

                    #[allow(clippy::cast_precision_loss)]
                    let s = matches as f32 / probe.expected_keywords.len() as f32;
                    s
                };

                ProbeResult {
                    probe: probe.clone(),
                    response: "[Offline evaluation - no LLM response]".to_string(),
                    score,
                    passed: score >= self.config.pass_threshold,
                    notes: Some("Evaluated offline using keyword matching".to_string()),
                }
            })
            .collect();

        let mut evaluation = EvaluationResults::from_probes_with_threshold(
            results,
            self.config.pass_threshold,
        );
        evaluation = evaluation.with_artifact_trail(artifact_trail.clone());
        evaluation
    }
}

/// Generate standard probes for a play based on artifact trail.
///
/// This is a helper function that creates a default set of probes
/// based on what's in the artifact trail.
#[must_use]
pub fn generate_standard_probes(artifact_trail: &ArtifactTrail) -> Vec<EvaluationProbe> {
    let mut probes = Vec::new();

    // Artifact probes - test file tracking
    if !artifact_trail.files_modified.is_empty() {
        let files: Vec<_> = artifact_trail.files_modified.keys().cloned().collect();
        probes.push(
            EvaluationProbe::new(
                ProbeType::Artifact,
                "Which files have been modified in this session?",
            )
            .with_keywords(files),
        );
    }

    if !artifact_trail.files_created.is_empty() {
        probes.push(
            EvaluationProbe::new(ProbeType::Artifact, "What new files were created?")
                .with_keywords(artifact_trail.files_created.clone()),
        );
    }

    // Decision probes
    if !artifact_trail.decisions_made.is_empty() {
        let decision_keywords: Vec<_> = artifact_trail
            .decisions_made
            .iter()
            .flat_map(|d| d.split_whitespace().take(3).map(String::from))
            .collect();

        probes.push(
            EvaluationProbe::new(
                ProbeType::Decision,
                "What key decisions were made during this task?",
            )
            .with_keywords(decision_keywords),
        );
    }

    // Standard probes that always apply
    probes.push(EvaluationProbe::new(
        ProbeType::Continuation,
        "What is the next step to complete this task?",
    ));

    probes.push(
        EvaluationProbe::new(
            ProbeType::Acceptance,
            "Have all acceptance criteria been met?",
        )
        .with_keywords(vec![
            "complete".to_string(),
            "pass".to_string(),
            "success".to_string(),
            "done".to_string(),
        ]),
    );

    probes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_artifact_trail() -> ArtifactTrail {
        let mut trail = ArtifactTrail::default();
        trail.files_created.push("src/new_module.rs".to_string());
        trail
            .files_modified
            .insert("src/main.rs".to_string(), "added error handling".to_string());
        trail.files_read.push("Cargo.toml".to_string());
        trail
            .decisions_made
            .push("Using anyhow for error handling".to_string());
        trail
    }

    #[test]
    fn test_format_context() {
        let evaluator = ProbeEvaluator::with_defaults();
        let trail = sample_artifact_trail();
        let context = ProbeEvaluator::format_context(&trail);

        assert!(context.contains("src/new_module.rs"));
        assert!(context.contains("src/main.rs"));
        assert!(context.contains("added error handling"));
        assert!(context.contains("Cargo.toml"));
        assert!(context.contains("anyhow"));
    }

    #[test]
    fn test_generate_standard_probes() {
        let trail = sample_artifact_trail();
        let probes = generate_standard_probes(&trail);

        assert!(!probes.is_empty());

        // Should have artifact probe for modified files
        let artifact_probe = probes
            .iter()
            .find(|p| p.probe_type == ProbeType::Artifact);
        assert!(artifact_probe.is_some());

        // Should have continuation probe
        let continuation_probe = probes
            .iter()
            .find(|p| p.probe_type == ProbeType::Continuation);
        assert!(continuation_probe.is_some());
    }

    #[test]
    fn test_offline_evaluation() {
        let evaluator = ProbeEvaluator::with_defaults();
        let trail = sample_artifact_trail();
        let probes = generate_standard_probes(&trail);

        let results = evaluator.evaluate_offline(probes, &trail);

        assert!(!results.probes.is_empty());
        assert!(results.artifact_trail.is_some());
    }

    #[test]
    fn test_probe_scoring() {
        let probe = EvaluationProbe::new(ProbeType::Artifact, "What files were modified?")
            .with_keywords(vec!["main.rs".to_string(), "lib.rs".to_string()]);

        // Response with one keyword
        let score1 = probe.score_response("I modified main.rs to add error handling");
        assert!((score1 - 0.5).abs() < 0.01); // 1/2 keywords = 0.5

        // Response with both keywords
        let score2 = probe.score_response("I modified main.rs and lib.rs");
        assert!((score2 - 1.0).abs() < 0.01); // 2/2 keywords = 1.0

        // Response with no keywords
        let score3 = probe.score_response("I made some changes to the code");
        assert!((score3 - 0.0).abs() < 0.01); // 0/2 keywords = 0.0
    }
}
