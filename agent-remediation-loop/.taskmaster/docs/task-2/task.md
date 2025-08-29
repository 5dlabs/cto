# Task 2: Implement Feedback Comment Parser

## Overview

Build a robust Rust-based parsing system to extract structured feedback from PR comments in the Agent Remediation Loop. This parser enables automatic remediation by converting human-readable QA feedback into actionable data structures that Rex can use to implement fixes.

## Technical Requirements

### Core Components

1. **Data Structures** (`controller/src/remediation/types.rs`)
   - `StructuredFeedback` - Main feedback container
   - `IssueType` enum - Bug, MissingFeature, Regression, Performance
   - `Severity` enum - Critical, High, Medium, Low
   - `CriteriaStatus` - Checkbox state with associated text
   - `FeedbackMetadata` - Author, timestamp, PR context

2. **Pattern Extraction** (`controller/src/remediation/patterns.rs`)
   - Regex library for metadata extraction
   - Issue Type pattern: `r"Issue Type.*\[(.*?)\]"`
   - Severity pattern: `r"Severity.*\[(.*?)\]"`
   - Performance-optimized lazy regex compilation

3. **Markdown Parser** (`controller/src/remediation/markdown.rs`)
   - Checkbox extraction from acceptance criteria
   - Support for `- [ ]` and `- [x]` syntax
   - Nested list and indentation handling
   - Robust markdown parsing with `pulldown-cmark`

4. **Author Validation** (`controller/src/remediation/auth.rs`)
   - Verify comment author is Tess QA bot or approved reviewer
   - Configurable allowlist management
   - GitHub API integration for user verification
   - Caching mechanism for authorization results

5. **Error Handling** (`controller/src/remediation/error.rs`)
   - Custom error types with `thiserror`
   - Graceful degradation for partial extraction
   - Structured logging with `tracing`
   - Manual review fallbacks for malformed comments

## Implementation Guide

### Step 1: Define Data Structures

```rust
// controller/src/remediation/types.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredFeedback {
    pub issue_type: IssueType,
    pub severity: Severity,
    pub description: String,
    pub criteria_not_met: Vec<CriteriaStatus>,
    pub reproduction_steps: Option<Vec<String>>,
    pub expected_behavior: Option<String>,
    pub actual_behavior: Option<String>,
    pub metadata: FeedbackMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueType {
    Bug,
    MissingFeature,
    Regression,
    Performance,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriteriaStatus {
    pub description: String,
    pub completed: bool,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackMetadata {
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub comment_id: u64,
    pub pr_number: u32,
    pub task_id: String,
}

impl std::fmt::Display for IssueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueType::Bug => write!(f, "Bug"),
            IssueType::MissingFeature => write!(f, "Missing Feature"),
            IssueType::Regression => write!(f, "Regression"),
            IssueType::Performance => write!(f, "Performance"),
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "Critical"),
            Severity::High => write!(f, "High"),
            Severity::Medium => write!(f, "Medium"),
            Severity::Low => write!(f, "Low"),
        }
    }
}
```

### Step 2: Implement Pattern Extraction

```rust
// controller/src/remediation/patterns.rs
use anyhow::{Result, Context};
use lazy_static::lazy_static;
use regex::Regex;
use crate::remediation::types::{IssueType, Severity};

lazy_static! {
    static ref ISSUE_TYPE_PATTERN: Regex = 
        Regex::new(r"(?m)^\s*\*\*Issue Type\*\*:\s*\[(.*?)\]").unwrap();
    static ref SEVERITY_PATTERN: Regex = 
        Regex::new(r"(?m)^\s*\*\*Severity\*\*:\s*\[(.*?)\]").unwrap();
    static ref DESCRIPTION_PATTERN: Regex = 
        Regex::new(r"(?ms)### Description\s*\n(.*?)(?:\n### |$)").unwrap();
    static ref STEPS_PATTERN: Regex = 
        Regex::new(r"(?ms)### Steps to Reproduce.*?\n((?:\d+\..*?\n?)+)").unwrap();
    static ref EXPECTED_PATTERN: Regex = 
        Regex::new(r"(?m)^\s*-?\s*\*\*Expected\*\*:\s*(.+)$").unwrap();
    static ref ACTUAL_PATTERN: Regex = 
        Regex::new(r"(?m)^\s*-?\s*\*\*Actual\*\*:\s*(.+)$").unwrap();
}

pub struct PatternExtractor;

impl PatternExtractor {
    pub fn extract_issue_type(body: &str) -> Result<IssueType> {
        let captures = ISSUE_TYPE_PATTERN.captures(body)
            .context("Issue Type not found in comment")?;
        
        let issue_type_str = captures.get(1)
            .context("Issue Type value not captured")?
            .as_str()
            .trim();
        
        match issue_type_str {
            "Bug" => Ok(IssueType::Bug),
            "Missing Feature" => Ok(IssueType::MissingFeature),
            "Regression" => Ok(IssueType::Regression),
            "Performance" => Ok(IssueType::Performance),
            _ => Err(anyhow::anyhow!("Unknown issue type: {}", issue_type_str)),
        }
    }
    
    pub fn extract_severity(body: &str) -> Result<Severity> {
        let captures = SEVERITY_PATTERN.captures(body)
            .context("Severity not found in comment")?;
        
        let severity_str = captures.get(1)
            .context("Severity value not captured")?
            .as_str()
            .trim();
        
        match severity_str {
            "Critical" => Ok(Severity::Critical),
            "High" => Ok(Severity::High),
            "Medium" => Ok(Severity::Medium),
            "Low" => Ok(Severity::Low),
            _ => Err(anyhow::anyhow!("Unknown severity: {}", severity_str)),
        }
    }
    
    pub fn extract_description(body: &str) -> Result<String> {
        let captures = DESCRIPTION_PATTERN.captures(body)
            .context("Description section not found")?;
        
        Ok(captures.get(1)
            .context("Description content not captured")?
            .as_str()
            .trim()
            .to_string())
    }
    
    pub fn extract_reproduction_steps(body: &str) -> Result<Vec<String>> {
        let captures = STEPS_PATTERN.captures(body)
            .context("Steps to Reproduce section not found")?;
        
        let steps_text = captures.get(1)
            .context("Steps content not captured")?
            .as_str();
        
        let steps: Vec<String> = steps_text
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    // Remove leading number and dot
                    let step = trimmed.split_once('.').map_or(trimmed, |(_, rest)| rest.trim());
                    Some(step.to_string())
                }
            })
            .collect();
        
        if steps.is_empty() {
            Err(anyhow::anyhow!("No reproduction steps found"))
        } else {
            Ok(steps)
        }
    }
    
    pub fn extract_expected_actual(body: &str) -> (Option<String>, Option<String>) {
        let expected = EXPECTED_PATTERN.captures(body)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().trim().to_string());
        
        let actual = ACTUAL_PATTERN.captures(body)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().trim().to_string());
        
        (expected, actual)
    }
}
```

### Step 3: Build Markdown Parser

```rust
// controller/src/remediation/markdown.rs
use anyhow::{Result, Context};
use regex::Regex;
use crate::remediation::types::CriteriaStatus;

pub struct MarkdownParser;

impl MarkdownParser {
    pub fn extract_criteria_checkboxes(body: &str) -> Result<Vec<CriteriaStatus>> {
        // Find the "Acceptance Criteria Not Met" section
        let criteria_section = Self::extract_criteria_section(body)?;
        
        // Parse checkboxes in the section
        Self::parse_checkboxes(&criteria_section)
    }
    
    fn extract_criteria_section(body: &str) -> Result<String> {
        let section_regex = Regex::new(
            r"(?ms)### Acceptance Criteria Not Met\s*\n(.*?)(?:\n### |\n\*\*|$)"
        ).unwrap();
        
        let captures = section_regex.captures(body)
            .context("Acceptance Criteria Not Met section not found")?;
        
        Ok(captures.get(1)
            .context("Criteria section content not captured")?
            .as_str()
            .to_string())
    }
    
    fn parse_checkboxes(section: &str) -> Result<Vec<CriteriaStatus>> {
        let checkbox_regex = Regex::new(r"(?m)^\s*-\s*\[([ x])\]\s*(.+)$").unwrap();
        
        let mut criteria = Vec::new();
        let mut line_number = 0;
        
        for line in section.lines() {
            line_number += 1;
            
            if let Some(captures) = checkbox_regex.captures(line) {
                let checkbox_state = captures.get(1).unwrap().as_str();
                let description = captures.get(2).unwrap().as_str().trim().to_string();
                
                let completed = match checkbox_state {
                    "x" | "X" => true,
                    " " | "" => false,
                    _ => false,
                };
                
                criteria.push(CriteriaStatus {
                    description,
                    completed,
                    line_number: Some(line_number),
                });
            }
        }
        
        if criteria.is_empty() {
            Err(anyhow::anyhow!("No checkboxes found in criteria section"))
        } else {
            Ok(criteria)
        }
    }
    
    pub fn extract_unmet_criteria(body: &str) -> Result<Vec<String>> {
        let criteria = Self::extract_criteria_checkboxes(body)?;
        
        let unmet: Vec<String> = criteria
            .into_iter()
            .filter(|c| !c.completed)
            .map(|c| c.description)
            .collect();
        
        if unmet.is_empty() {
            Err(anyhow::anyhow!("All criteria appear to be met"))
        } else {
            Ok(unmet)
        }
    }
}
```

### Step 4: Add Author Validation

```rust
// controller/src/remediation/auth.rs
use anyhow::{Result, Context};
use std::collections::HashSet;
use dashmap::DashMap;
use std::time::{Duration, Instant};

pub struct AuthorValidator {
    allowed_authors: HashSet<String>,
    auth_cache: DashMap<String, (bool, Instant)>,
    cache_ttl: Duration,
}

impl AuthorValidator {
    pub fn new() -> Self {
        let mut allowed_authors = HashSet::new();
        
        // Core QA bot
        allowed_authors.insert("5DLabs-Tess".to_string());
        
        // Additional approved reviewers (configurable)
        allowed_authors.insert("approved-reviewer-1".to_string());
        allowed_authors.insert("approved-reviewer-2".to_string());
        
        Self {
            allowed_authors,
            auth_cache: DashMap::new(),
            cache_ttl: Duration::from_secs(300), // 5 minutes
        }
    }
    
    pub fn validate_author(&self, author: &str) -> Result<()> {
        // Check cache first
        if let Some((is_valid, timestamp)) = self.auth_cache.get(author) {
            if timestamp.elapsed() < self.cache_ttl {
                return if *is_valid {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Author '{}' is not authorized", author))
                };
            }
        }
        
        // Check against allowed authors
        let is_authorized = self.allowed_authors.contains(author) 
            || self.is_team_member(author).unwrap_or(false);
        
        // Cache the result
        self.auth_cache.insert(
            author.to_string(), 
            (is_authorized, Instant::now())
        );
        
        if is_authorized {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Author '{}' is not authorized to provide feedback", author))
        }
    }
    
    fn is_team_member(&self, author: &str) -> Result<bool> {
        // This could integrate with GitHub API to check team membership
        // For now, implement basic pattern matching
        Ok(author.starts_with("5DLabs-") || author.ends_with("-qa"))
    }
    
    pub fn add_approved_author(&mut self, author: String) {
        self.allowed_authors.insert(author);
    }
    
    pub fn remove_approved_author(&mut self, author: &str) {
        self.allowed_authors.remove(author);
    }
    
    pub fn clear_cache(&self) {
        self.auth_cache.clear();
    }
}

impl Default for AuthorValidator {
    fn default() -> Self {
        Self::new()
    }
}
```

### Step 5: Main Parser Implementation

```rust
// controller/src/remediation/parser.rs
use anyhow::{Result, Context};
use chrono::Utc;
use crate::remediation::{
    types::{StructuredFeedback, FeedbackMetadata},
    patterns::PatternExtractor,
    markdown::MarkdownParser,
    auth::AuthorValidator,
};

pub struct FeedbackParser {
    validator: AuthorValidator,
}

impl FeedbackParser {
    pub fn new() -> Self {
        Self {
            validator: AuthorValidator::new(),
        }
    }
    
    pub fn parse_comment(
        &self,
        comment_body: &str,
        author: &str,
        comment_id: u64,
        pr_number: u32,
        task_id: &str,
    ) -> Result<StructuredFeedback> {
        // Validate this is actionable feedback
        if !Self::is_actionable_feedback(comment_body) {
            return Err(anyhow::anyhow!("Comment is not actionable feedback"));
        }
        
        // Validate author
        self.validator.validate_author(author)
            .context("Author validation failed")?;
        
        // Extract components
        let issue_type = PatternExtractor::extract_issue_type(comment_body)
            .context("Failed to extract issue type")?;
        
        let severity = PatternExtractor::extract_severity(comment_body)
            .context("Failed to extract severity")?;
        
        let description = PatternExtractor::extract_description(comment_body)
            .context("Failed to extract description")?;
        
        let criteria_not_met = MarkdownParser::extract_criteria_checkboxes(comment_body)
            .context("Failed to extract criteria checkboxes")?;
        
        // Extract optional sections
        let reproduction_steps = PatternExtractor::extract_reproduction_steps(comment_body).ok();
        let (expected_behavior, actual_behavior) = PatternExtractor::extract_expected_actual(comment_body);
        
        // Build metadata
        let metadata = FeedbackMetadata {
            author: author.to_string(),
            timestamp: Utc::now(),
            comment_id,
            pr_number,
            task_id: task_id.to_string(),
        };
        
        Ok(StructuredFeedback {
            issue_type,
            severity,
            description,
            criteria_not_met,
            reproduction_steps,
            expected_behavior,
            actual_behavior,
            metadata,
        })
    }
    
    fn is_actionable_feedback(comment_body: &str) -> bool {
        comment_body.contains("🔴 Required Changes")
    }
    
    pub fn get_validator_mut(&mut self) -> &mut AuthorValidator {
        &mut self.validator
    }
}

impl Default for FeedbackParser {
    fn default() -> Self {
        Self::new()
    }
}
```

### Step 6: Error Handling

```rust
// controller/src/remediation/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Comment is not actionable feedback")]
    NotActionableFeedback,
    
    #[error("Author '{author}' is not authorized")]
    UnauthorizedAuthor { author: String },
    
    #[error("Required field '{field}' is missing from comment")]
    MissingRequiredField { field: String },
    
    #[error("Invalid value '{value}' for field '{field}'")]
    InvalidFieldValue { field: String, value: String },
    
    #[error("Malformed comment structure: {reason}")]
    MalformedComment { reason: String },
    
    #[error("No acceptance criteria checkboxes found")]
    NoCriteriaFound,
    
    #[error("Regex compilation error: {0}")]
    RegexError(#[from] regex::Error),
    
    #[error("Generic parsing error: {0}")]
    Generic(#[from] anyhow::Error),
}

pub type ParseResult<T> = std::result::Result<T, ParseError>;
```

### Step 7: Integration Module

```rust
// controller/src/remediation/mod.rs
pub mod types;
pub mod patterns;
pub mod markdown;
pub mod auth;
pub mod parser;
pub mod error;

pub use parser::FeedbackParser;
pub use types::{StructuredFeedback, IssueType, Severity, CriteriaStatus, FeedbackMetadata};
pub use error::{ParseError, ParseResult};

// Re-export for external use
pub fn parse_feedback_comment(
    comment_body: &str,
    author: &str,
    comment_id: u64,
    pr_number: u32,
    task_id: &str,
) -> ParseResult<StructuredFeedback> {
    let parser = FeedbackParser::new();
    parser.parse_comment(comment_body, author, comment_id, pr_number, task_id)
        .map_err(ParseError::from)
}
```

## Testing Strategy

### Unit Tests Structure

```rust
// controller/src/remediation/tests/mod.rs
mod test_patterns;
mod test_markdown;
mod test_auth;
mod test_parser;
mod test_integration;

// Test data and utilities
pub mod fixtures;
pub mod utils;
```

### Key Test Cases

1. **Valid Comment Parsing**
   - Complete structured feedback
   - All field combinations
   - Various checkbox states

2. **Edge Cases**
   - Missing optional fields
   - Special characters and Unicode
   - Nested markdown structures
   - Large comments (>10MB)

3. **Error Conditions**
   - Malformed comments
   - Unauthorized authors
   - Invalid field values
   - Missing required sections

4. **Security Tests**
   - XSS prevention
   - Command injection attempts
   - Regex DoS prevention

5. **Performance Tests**
   - Large comment parsing
   - Regex optimization
   - Cache effectiveness

## Integration Points

### Controller API

Add endpoint to controller for Rex container usage:

```rust
// In controller main API
#[post("/api/v1/feedback/parse")]
async fn parse_feedback_endpoint(
    payload: Json<FeedbackRequest>,
) -> Result<Json<StructuredFeedback>, ApiError> {
    let feedback = parse_feedback_comment(
        &payload.comment_body,
        &payload.author,
        payload.comment_id,
        payload.pr_number,
        &payload.task_id,
    )?;
    
    Ok(Json(feedback))
}
```

### Rex Container Integration

Rex can call the parser through the controller API or direct library usage when compiled into the same binary.

## Dependencies

Add to `Cargo.toml`:

```toml
[dependencies]
anyhow = "1.0"
regex = "1.0"
serde = { version = "1.0", features = ["derive"] }
chrono = { version = "0.4", features = ["serde"] }
lazy_static = "1.4"
thiserror = "1.0"
dashmap = "5.4"
pulldown-cmark = "0.9"
tracing = "0.1"

[dev-dependencies]
proptest = "1.0"
```

This implementation provides a robust, well-tested foundation for parsing QA feedback comments into structured data that Rex can act upon in the remediation loop.