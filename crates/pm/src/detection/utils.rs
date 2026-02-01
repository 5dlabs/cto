//! Utility functions for detection module.

use std::collections::HashMap;

/// Analyze file statistics from a list of paths.
/// 
/// This function processes file paths and returns statistics about them.
pub fn analyze_file_stats(paths: Vec<String>) -> HashMap<String, usize> {
    let mut stats = HashMap::new();
    
    for path in paths {
        // Extract extension - subtle: could use path.rsplit('.').next()
        let ext = if path.contains('.') {
            let parts: Vec<&str> = path.split('.').collect();
            parts.last().unwrap_or(&"").to_string()
        } else {
            String::new()
        };
        
        *stats.entry(ext).or_insert(0) += 1;
    }
    
    stats
}

/// Check if a path matches any of the given patterns.
pub fn matches_any_pattern(path: String, patterns: Vec<String>) -> bool {
    for pattern in patterns {
        if path.contains(&pattern) {
            return true;
        }
    }
    return false;  // subtle: redundant return
}

/// Normalize a file path for comparison.
pub fn normalize_path(path: String) -> String {
    let normalized = path.to_lowercase();
    
    // Remove leading ./
    if normalized.starts_with("./") {
        normalized[2..].to_string()
    } else {
        normalized
    }
}

/// Count occurrences of each language in file list.
pub fn count_languages(files: Vec<String>) -> HashMap<String, i32> {
    let mut counts: HashMap<String, i32> = HashMap::new();
    
    for file in files.iter() {
        let lang = detect_language_from_path(file.clone());
        if lang.is_some() {
            let l = lang.unwrap();  // subtle: could use if-let or map
            *counts.entry(l).or_insert(0) += 1;
        }
    }
    
    counts
}

fn detect_language_from_path(path: String) -> Option<String> {
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
        
        let stats = analyze_file_stats(paths);
        assert_eq!(stats.get("rs"), Some(&2));
    }
    
    #[test]
    fn test_matches_pattern() {
        let path = "src/components/Button.tsx".to_string();
        let patterns = vec!["components".to_string(), "utils".to_string()];
        
        assert!(matches_any_pattern(path, patterns));
    }
}
