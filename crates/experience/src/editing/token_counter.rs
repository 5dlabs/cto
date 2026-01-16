//! Token counting utilities.

use tiktoken_rs::cl100k_base;

/// Token counter using tiktoken.
pub struct TokenCounter {
    bpe: tiktoken_rs::CoreBPE,
}

impl TokenCounter {
    /// Create a new token counter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            bpe: cl100k_base().expect("Failed to load tiktoken"),
        }
    }

    /// Count tokens in a string.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn count(&self, text: &str) -> u32 {
        self.bpe.encode_ordinary(text).len() as u32
    }

    /// Count tokens in multiple strings.
    #[must_use]
    pub fn count_many(&self, texts: &[&str]) -> u32 {
        texts.iter().map(|t| self.count(t)).sum()
    }

    /// Estimate if text exceeds a token limit.
    #[must_use]
    pub fn exceeds_limit(&self, text: &str, limit: u32) -> bool {
        self.count(text) > limit
    }
}

impl Default for TokenCounter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_counting() {
        let counter = TokenCounter::new();

        // Simple text should have some tokens
        let count = counter.count("Hello, world!");
        assert!(count > 0);
        assert!(count < 10);
    }

    #[test]
    fn test_count_many() {
        let counter = TokenCounter::new();

        let texts = vec!["Hello", "World"];
        let total = counter.count_many(&texts);
        let individual = counter.count("Hello") + counter.count("World");

        assert_eq!(total, individual);
    }

    #[test]
    fn test_exceeds_limit() {
        let counter = TokenCounter::new();

        assert!(!counter.exceeds_limit("Hi", 100));
        assert!(counter.exceeds_limit(
            "This is a longer text that should exceed a very small limit",
            1
        ));
    }
}
