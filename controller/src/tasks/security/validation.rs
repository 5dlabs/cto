//! # Input Validation and Sanitization
//!
//! This module provides comprehensive input validation and sanitization
//! for the Agent Remediation Loop to prevent injection attacks and ensure data integrity.

use regex::Regex;
use std::collections::HashMap;
use thiserror::Error;

/// Input validation errors
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Input validation failed: {0}")]
    ValidationFailed(String),

    #[error("Input sanitization error: {0}")]
    SanitizationError(String),

    #[error("Input too long: {size} > {max}")]
    InputTooLong { size: usize, max: usize },

    #[error("Malicious content detected: {0}")]
    MaliciousContent(String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

impl From<regex::Error> for ValidationError {
    fn from(err: regex::Error) -> Self {
        ValidationError::ValidationFailed(format!("Regex error: {}", err))
    }
}

/// Result type for validation operations
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Input validation result
#[derive(Debug, Clone)]
pub struct InputValidationResult {
    pub is_valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub sanitized_input: Option<String>,
}

/// Input validator for comprehensive validation
pub struct InputValidator {
    max_comment_length: usize,
    malicious_patterns: Vec<Regex>,
    shell_metacharacters: Vec<char>,
}

impl InputValidator {
    /// Create a new input validator
    pub fn new() -> ValidationResult<Self> {
        let malicious_patterns = vec![
            // Script tags
            Regex::new(r"<script[^>]*>.*?</script>")?,
            // JavaScript URLs
            Regex::new(r"javascript:")?,
            // Event handlers
            Regex::new(r"on\w+\s*=")?,
            // Template injection
            Regex::new(r"\$\{.*?\}")?,
            Regex::new(r"\{\{.*?\}\}")?,
            // Function calls
            Regex::new(r"eval\s*\(")?,
            Regex::new(r"exec\s*\(")?,
            // Ruby template injection
            Regex::new(r"#\{.*?\}")?,
            // SQL injection patterns
            Regex::new(r";\s*DROP")?,
            Regex::new(r"'\s*OR\s*'1'\s*=\s*'1")?,
            // Command injection
            Regex::new(r";\s*(?:cat|ls|rm|cp|mv)")?,
            Regex::new(r"\|\s*(?:cat|ls|rm|cp|mv)")?,
            Regex::new(r"`.*?`")?, // Backticks
            Regex::new(r"\$\(.*?\)")?, // Command substitution
        ];

        Ok(Self {
            max_comment_length: 50 * 1024, // 50KB
            malicious_patterns,
            shell_metacharacters: vec!['|', '&', ';', '(', ')', '`', '$'],
        })
    }

    /// Validate input comprehensively
    pub async fn validate_input(&self, input: &str) -> ValidationResult<InputValidationResult> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // Check length
        if input.len() > self.max_comment_length {
            errors.push(format!("Input length {} exceeds maximum {}", input.len(), self.max_comment_length));
        }

        // Check for malicious patterns
        for pattern in &self.malicious_patterns {
            if pattern.is_match(input) {
                errors.push(format!("Malicious pattern detected: {}", pattern.as_str()));
            }
        }

        // Check for shell metacharacters
        let metachar_count = input.chars()
            .filter(|c| self.shell_metacharacters.contains(c))
            .count();

        if metachar_count > 0 {
            warnings.push(format!("Found {} shell metacharacters that may need escaping", metachar_count));
        }

        // Validate UTF-8
        if !std::str::from_utf8(input.as_bytes()).is_ok() {
            errors.push("Input contains invalid UTF-8 characters".to_string());
        }

        // Sanitize input
        let sanitized_input = match self.sanitize_input(input).await {
            Ok(sanitized) => Some(sanitized),
            Err(e) => {
                errors.push(format!("Sanitization failed: {}", e));
                None
            }
        };

        let is_valid = errors.is_empty();

        Ok(InputValidationResult {
            is_valid,
            warnings,
            errors,
            sanitized_input,
        })
    }

    /// Sanitize input for safe processing
    pub async fn sanitize_input(&self, input: &str) -> ValidationResult<String> {
        let mut sanitized = input.to_string();

        // HTML escape dangerous characters
        sanitized = sanitized
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
            .replace('/', "&#x2F;");

        // Remove or escape shell metacharacters
        for &metachar in &self.shell_metacharacters {
            let replacement = format!("\\{}", metachar);
            sanitized = sanitized.replace(metachar, &replacement);
        }

        // Remove malicious patterns
        for pattern in &self.malicious_patterns {
            sanitized = pattern.replace_all(&sanitized, "[MALICIOUS CONTENT REMOVED]").to_string();
        }

        // Trim whitespace
        sanitized = sanitized.trim().to_string();

        Ok(sanitized)
    }

    /// Validate structured feedback data
    pub async fn validate_structured_feedback(
        &self,
        feedback: &crate::remediation::StructuredFeedback,
    ) -> ValidationResult<InputValidationResult> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // Validate issue type
        if !matches!(feedback.issue_type,
            crate::remediation::IssueType::Bug |
            crate::remediation::IssueType::MissingFeature |
            crate::remediation::IssueType::Regression |
            crate::remediation::IssueType::Performance
        ) {
            errors.push("Invalid issue type".to_string());
        }

        // Validate severity
        if !matches!(feedback.severity,
            crate::remediation::Severity::Critical |
            crate::remediation::Severity::High |
            crate::remediation::Severity::Medium |
            crate::remediation::Severity::Low
        ) {
            errors.push("Invalid severity level".to_string());
        }

        // Validate description length
        if feedback.description.len() > 1000 {
            warnings.push("Description is very long (>1000 chars)".to_string());
        }

        // Validate criteria
        for (i, criterion) in feedback.criteria_not_met.iter().enumerate() {
            if criterion.description.is_empty() {
                errors.push(format!("Criterion {} has empty description", i));
            }
            if criterion.description.len() > 500 {
                warnings.push(format!("Criterion {} description is very long", i));
            }
        }

        // Validate reproduction steps
        if let Some(steps) = &feedback.reproduction_steps {
            for (i, step) in steps.iter().enumerate() {
                if step.is_empty() {
                    warnings.push(format!("Reproduction step {} is empty", i));
                }
                if step.len() > 300 {
                    warnings.push(format!("Reproduction step {} is very long", i));
                }
            }
        }

        let is_valid = errors.is_empty();

        Ok(InputValidationResult {
            is_valid,
            warnings,
            errors,
            sanitized_input: None, // Structured feedback doesn't need sanitization
        })
    }

    /// Check for XSS vulnerabilities
    pub fn detect_xss(&self, input: &str) -> bool {
        let xss_patterns = vec![
            r"<script[^>]*>.*?</script>",
            r"<img[^>]*src[^>]*=.*onerror.*>",
            r"javascript:",
            r"vbscript:",
            r"data:text/html",
            r"<iframe[^>]*>.*?</iframe>",
            r"<object[^>]*>.*?</object>",
            r"<embed[^>]*>.*?</embed>",
        ];

        for pattern in xss_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if regex.is_match(input) {
                    return true;
                }
            }
        }

        false
    }

    /// Check for SQL injection patterns
    pub fn detect_sql_injection(&self, input: &str) -> bool {
        let sql_patterns = vec![
            r";\s*(DROP|DELETE|UPDATE|INSERT|ALTER)",
            r"'\s*OR\s*'1'\s*=\s*'1",
            r"'\s*AND\s*'1'\s*=\s*'1",
            r"--",
            r"/\*.*?\*/",
            r"UNION\s+SELECT",
            r"INFORMATION_SCHEMA",
            r"LOAD_FILE",
            r"INTO OUTFILE",
        ];

        for pattern in sql_patterns {
            if let Ok(regex) = Regex::new(&format!("(?i){}", pattern)) { // Case insensitive
                if regex.is_match(input) {
                    return true;
                }
            }
        }

        false
    }

    /// Check for command injection patterns
    pub fn detect_command_injection(&self, input: &str) -> bool {
        let cmd_patterns = vec![
            r";\s*(cat|ls|rm|cp|mv|chmod|chown)",
            r"\|\s*(cat|ls|rm|cp|mv|chmod|chown)",
            r"`.*?`",
            r"\$\(.*?\)",
            r">\s*/.*", // Redirection to system files
            r"<\s*/.*", // Reading from system files
            r"2>&1",
            r">&2",
        ];

        for pattern in cmd_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if regex.is_match(input) {
                    return true;
                }
            }
        }

        false
    }

    /// Validate task ID format
    pub fn validate_task_id(&self, task_id: &str) -> ValidationResult<()> {
        if task_id.is_empty() {
            return Err(ValidationError::ValidationFailed("Task ID cannot be empty".to_string()));
        }

        if task_id.len() > 100 {
            return Err(ValidationError::InputTooLong { size: task_id.len(), max: 100 });
        }

        // Allow alphanumeric, hyphens, underscores, and dots
        let valid_chars = task_id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.');

        if !valid_chars {
            return Err(ValidationError::InvalidFormat(
                "Task ID contains invalid characters".to_string()
            ));
        }

        Ok(())
    }

    /// Validate PR number
    pub fn validate_pr_number(&self, pr_number: i32) -> ValidationResult<()> {
        if pr_number <= 0 {
            return Err(ValidationError::ValidationFailed(
                "PR number must be positive".to_string()
            ));
        }

        if pr_number > 1000000 { // Reasonable upper bound
            return Err(ValidationError::ValidationFailed(
                "PR number seems unreasonably high".to_string()
            ));
        }

        Ok(())
    }

    /// Get validation statistics
    pub async fn get_statistics(&self) -> HashMap<String, u64> {
        let mut stats = HashMap::new();

        // In a real implementation, these would be collected metrics
        stats.insert("malicious_patterns".to_string(), self.malicious_patterns.len() as u64);
        stats.insert("max_comment_length".to_string(), self.max_comment_length as u64);
        stats.insert("shell_metacharacters".to_string(), self.shell_metacharacters.len() as u64);

        stats
    }

    /// Update configuration
    pub fn update_config(&mut self, max_length: usize) -> ValidationResult<()> {
        if max_length == 0 {
            return Err(ValidationError::ValidationFailed(
                "Maximum length cannot be zero".to_string()
            ));
        }

        if max_length > 1024 * 1024 { // 1MB limit
            return Err(ValidationError::ValidationFailed(
                "Maximum length cannot exceed 1MB".to_string()
            ));
        }

        self.max_comment_length = max_length;
        Ok(())
    }

    /// Add malicious pattern
    pub fn add_malicious_pattern(&mut self, pattern: &str) -> ValidationResult<()> {
        let regex = Regex::new(pattern)
            .map_err(|e| ValidationError::ValidationFailed(
                format!("Invalid regex pattern: {}", e)
            ))?;

        self.malicious_patterns.push(regex);
        Ok(())
    }

    /// Remove malicious pattern
    pub fn remove_malicious_pattern(&mut self, index: usize) -> ValidationResult<()> {
        if index >= self.malicious_patterns.len() {
            return Err(ValidationError::ValidationFailed(
                format!("Pattern index {} out of bounds", index)
            ));
        }

        self.malicious_patterns.remove(index);
        Ok(())
    }
}
