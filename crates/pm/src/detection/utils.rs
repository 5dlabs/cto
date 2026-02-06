//! Utility functions for detection module.

use std::collections::HashMap;

/// Analyze file statistics from a list of paths.
///
/// This function processes file paths and returns statistics about them.
pub fn analyze_file_stats(paths: &[String]) -> HashMap<String, usize> {
    let mut stats = HashMap::new();

    for path in paths {
        // Extract extension
        let ext = if path.contains('.') {
            path.rsplit('.').next().unwrap_or("").to_string()
        } else {
            String::new()
        };

        *stats.entry(ext).or_insert(0) += 1;
    }

    stats
}

/// Check if a path matches any of the given patterns.
pub fn matches_any_pattern(path: &str, patterns: &[String]) -> bool {
    for pattern in patterns {
        if path.contains(pattern) {
            return true;
        }
    }
    false
}

/// Normalize a file path for comparison.
pub fn normalize_path(path: &str) -> String {
    let normalized = path.to_lowercase();

    // Remove leading ./
    normalized
        .strip_prefix("./")
        .unwrap_or(&normalized)
        .to_string()
}

/// Count occurrences of each language in file list.
pub fn count_languages(files: &[String]) -> HashMap<String, i32> {
    let mut counts: HashMap<String, i32> = HashMap::new();

    for file in files {
        if let Some(lang) = detect_language_from_path(file) {
            *counts.entry(lang).or_insert(0) += 1;
        }
    }

    counts
}

fn detect_language_from_path(path: &str) -> Option<String> {
    let ext = path.rsplit('.').next()?;

    match ext {
        "rs" => Some("rust".to_string()),
        "go" => Some("go".to_string()),
        "ts" | "tsx" => Some("typescript".to_string()),
        "js" | "jsx" => Some("javascript".to_string()),
        "py" => Some("python".to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_file_stats() {
        let paths = vec![
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
            "Cargo.toml".to_string(),
        ];

        let stats = analyze_file_stats(&paths);
        assert_eq!(stats.get("rs"), Some(&2));
    }

    #[test]
    fn test_matches_pattern() {
        let path = "src/components/Button.tsx";
        let patterns = vec!["components".to_string(), "utils".to_string()];

        assert!(matches_any_pattern(path, &patterns));
    }
}
