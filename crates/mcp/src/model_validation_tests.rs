//! Test module for Task 2: Flexible CLI Model Configuration
//!
//! Tests the new permissive `validate_model_name` function and
//! the `autocorrect_anthropic_model` function for deprecated model handling.

use crate::autocorrect_anthropic_model;
use crate::validate_model_name;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that the new validation accepts any non-empty model string
    #[test]
    fn test_permissive_model_validation() {
        // All these model types should be accepted now
        assert!(validate_model_name("claude-3-5-sonnet-20241022").is_ok());
        assert!(validate_model_name("gpt-4o-2024-08-06").is_ok());
        assert!(validate_model_name("o3").is_ok());
        assert!(validate_model_name("gemini-2.0-flash-exp").is_ok());
        assert!(validate_model_name("llama3.1:70b").is_ok());
        assert!(validate_model_name("my-custom-model").is_ok());
        assert!(validate_model_name("opus").is_ok());
        assert!(validate_model_name("sonnet").is_ok());
        assert!(validate_model_name("haiku").is_ok());
    }

    /// Test that only empty strings are rejected
    #[test]
    fn test_empty_model_rejection() {
        assert!(validate_model_name("").is_err());
        assert!(validate_model_name("   ").is_err());
        assert!(validate_model_name("\t\n").is_err());
    }

    /// Test edge cases that should be accepted
    #[test]
    fn test_edge_cases() {
        assert!(validate_model_name("a").is_ok());
        assert!(validate_model_name("1").is_ok());
        assert!(validate_model_name("model-with-special-chars!@#").is_ok());
        assert!(validate_model_name("model/with/slashes").is_ok());
        assert!(validate_model_name("model:with:colons").is_ok());
        assert!(validate_model_name("http://example.com/model").is_ok());
        let long_name = "very-long-model-name".repeat(50);
        assert!(validate_model_name(&long_name).is_ok());
    }

    /// Test backward compatibility with existing Claude model names
    #[test]
    fn test_backward_compatibility() {
        // These were the only models accepted before Task 2
        assert!(validate_model_name("claude-3-5-sonnet-20241022").is_ok());
        assert!(validate_model_name("claude-3-opus-20240229").is_ok());
        assert!(validate_model_name("claude-3-haiku-20240307").is_ok());
        assert!(validate_model_name("opus").is_ok());
        assert!(validate_model_name("sonnet").is_ok());
        assert!(validate_model_name("haiku").is_ok());

        // Legacy formats should still work
        assert!(validate_model_name("claude-sonnet-4-20250514").is_ok());
    }

    /// Test that previously rejected models are now accepted
    #[test]
    fn test_new_model_support() {
        // These would have been rejected by the old validation
        assert!(validate_model_name("gpt-4").is_ok());
        assert!(validate_model_name("gpt-4o").is_ok());
        assert!(validate_model_name("o3").is_ok());
        assert!(validate_model_name("gemini-pro").is_ok());
        assert!(validate_model_name("gemini-2.0-flash-exp").is_ok());
        assert!(validate_model_name("llama3.1:70b").is_ok());
        assert!(validate_model_name("mistral-7b-instruct").is_ok());
        assert!(validate_model_name("my-custom-model").is_ok());
        assert!(validate_model_name("localhost:11434/model").is_ok());
        assert!(validate_model_name("experimental-model-v2").is_ok());
    }

    /// Test the error message for empty models
    #[test]
    fn test_error_message() {
        let result = validate_model_name("");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Model name cannot be empty"));
    }

    /// Performance test - validation should be very fast
    #[test]
    fn test_validation_performance() {
        use std::time::Instant;

        let models = vec![
            "claude-3-5-sonnet-20241022",
            "gpt-4o-2024-08-06",
            "gemini-2.0-flash-exp",
            "very-long-model-name-with-lots-of-characters-and-details-that-goes-on-and-on",
        ];

        let start = Instant::now();
        for _ in 0..10000 {
            for model in &models {
                let _ = validate_model_name(model);
            }
        }
        let duration = start.elapsed();

        // Should be very fast - less than 50ms for 40,000 validations
        assert!(
            duration.as_millis() < 50,
            "Validation took {:?} ms for 40k validations",
            duration.as_millis()
        );
    }

    /// Test autocorrection of deprecated Anthropic model IDs
    #[test]
    fn test_autocorrect_deprecated_models() {
        // Deprecated claude-3-5-sonnet-20241022 should be corrected
        let (corrected, was_corrected) =
            autocorrect_anthropic_model("claude-3-5-sonnet-20241022", "anthropic");
        assert!(was_corrected);
        assert_eq!(corrected, "claude-sonnet-4-5-20250929");

        // claude-3-5-sonnet (short form) should also be corrected
        let (corrected, was_corrected) =
            autocorrect_anthropic_model("claude-3-5-sonnet", "anthropic");
        assert!(was_corrected);
        assert_eq!(corrected, "claude-sonnet-4-5-20250929");
    }

    /// Test that autocorrection does not modify valid models
    #[test]
    fn test_autocorrect_preserves_valid_models() {
        // Valid models should pass through unchanged
        let (model, was_corrected) =
            autocorrect_anthropic_model("claude-sonnet-4-5-20250929", "anthropic");
        assert!(!was_corrected);
        assert_eq!(model, "claude-sonnet-4-5-20250929");

        let (model, was_corrected) =
            autocorrect_anthropic_model("claude-opus-4-5-20251101", "anthropic");
        assert!(!was_corrected);
        assert_eq!(model, "claude-opus-4-5-20251101");
    }

    /// Test that autocorrection only applies to anthropic provider
    #[test]
    fn test_autocorrect_skips_non_anthropic() {
        // Same deprecated model name but different provider should not be corrected
        let (model, was_corrected) =
            autocorrect_anthropic_model("claude-3-5-sonnet-20241022", "openai");
        assert!(!was_corrected);
        assert_eq!(model, "claude-3-5-sonnet-20241022");

        let (model, was_corrected) =
            autocorrect_anthropic_model("claude-3-5-sonnet-20241022", "claude-code");
        assert!(!was_corrected);
        assert_eq!(model, "claude-3-5-sonnet-20241022");
    }
}
