//! Language, framework, and agent detection for remediation routing.
//!
//! This module analyzes PR changed files and package manifests to determine:
//! 1. Primary programming language
//! 2. Framework/platform (Next.js, Expo, Electron, etc.)
//! 3. Which agent should handle remediation
//!
//! The tricky part is distinguishing TypeScript variants:
//! - Next.js/React web → Blaze
//! - Expo/React Native mobile → Tap
//! - Electron desktop → Spark
//! - Elysia/Effect backend → Nova

mod agent;
mod framework;
mod language;

pub use agent::Agent;
pub use framework::Framework;
pub use language::Language;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of analyzing a PR's changed files
#[derive(Debug, Clone, Default)]
pub struct DetectionResult {
    /// Primary language detected (by file count)
    pub primary_language: Option<Language>,
    /// All languages detected with file counts
    pub language_counts: HashMap<Language, usize>,
    /// Framework/platform detected (if any)
    pub framework: Option<Framework>,
    /// Confidence level (0.0 - 1.0)
    pub confidence: f32,
    /// Detection signals that led to this result
    pub signals: Vec<DetectionSignal>,
}

/// A signal that contributed to detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionSignal {
    pub source: SignalSource,
    pub signal_type: SignalType,
    pub value: String,
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalSource {
    FileExtension,
    FilePath,
    PackageJson,
    CargoToml,
    GoMod,
    DirectoryStructure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalType {
    Language(Language),
    Framework(Framework),
}

/// Changed file from a PR
#[derive(Debug, Clone)]
pub struct ChangedFile {
    pub path: String,
    pub additions: usize,
    pub deletions: usize,
}

impl ChangedFile {
    /// Get the file extension (lowercase)
    pub fn extension(&self) -> Option<&str> {
        self.path.rsplit('.').next().map(|s| {
            // Handle special cases like .d.ts
            if s == "ts" && self.path.ends_with(".d.ts") {
                return "d.ts";
            }
            s
        })
    }

    /// Get the filename without path
    pub fn filename(&self) -> &str {
        self.path.rsplit('/').next().unwrap_or(&self.path)
    }
}

/// Detect language, framework, and recommended agent from changed files
pub fn detect_from_files(files: &[ChangedFile]) -> DetectionResult {
    let mut result = DetectionResult::default();

    // Step 1: Count languages by file extension
    for file in files {
        if let Some(lang) = language::detect_from_extension(file.extension()) {
            *result.language_counts.entry(lang).or_default() += 1;
            result.signals.push(DetectionSignal {
                source: SignalSource::FileExtension,
                signal_type: SignalType::Language(lang),
                value: file.path.clone(),
                weight: 1.0,
            });
        }

        // Step 2: Check path patterns for framework hints
        if let Some(framework) = framework::detect_from_path(&file.path) {
            result.signals.push(DetectionSignal {
                source: SignalSource::FilePath,
                signal_type: SignalType::Framework(framework),
                value: file.path.clone(),
                weight: 2.0, // Path patterns are strong signals
            });
            // Set framework if not already set (first match wins)
            if result.framework.is_none() {
                result.framework = Some(framework);
            }
        }
    }

    // Step 3: Determine primary language
    result.primary_language = result
        .language_counts
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(lang, _)| *lang);

    // Calculate confidence based on signal consistency
    result.confidence = calculate_confidence(&result);

    result
}

/// Detect framework from package.json content
pub fn detect_from_package_json(content: &str) -> Option<Framework> {
    framework::detect_from_package_json(content)
}

/// Select the best agent for the detection result
pub fn select_agent(result: &DetectionResult) -> Agent {
    agent::select_agent(result)
}

/// Full detection pipeline: files + optional package.json
pub fn detect_full(files: &[ChangedFile], package_json: Option<&str>) -> (DetectionResult, Agent) {
    let mut result = detect_from_files(files);

    // Enhance with package.json if available
    if let Some(pkg) = package_json {
        if let Some(framework) = detect_from_package_json(pkg) {
            // Package.json is authoritative for TypeScript projects
            if result.framework.is_none()
                || matches!(
                    result.primary_language,
                    Some(Language::TypeScript) | Some(Language::JavaScript)
                )
            {
                result.framework = Some(framework);
                result.signals.push(DetectionSignal {
                    source: SignalSource::PackageJson,
                    signal_type: SignalType::Framework(framework),
                    value: "package.json".to_string(),
                    weight: 3.0, // Package.json is very authoritative
                });
            }
        }
    }

    let agent = select_agent(&result);
    (result, agent)
}

fn calculate_confidence(result: &DetectionResult) -> f32 {
    if result.language_counts.is_empty() {
        return 0.0;
    }

    let total_files: usize = result.language_counts.values().sum();
    let primary_count = result
        .primary_language
        .and_then(|lang| result.language_counts.get(&lang))
        .copied()
        .unwrap_or(0);

    // Base confidence on how dominant the primary language is
    let language_confidence = primary_count as f32 / total_files as f32;

    // Boost if we have framework detection
    let framework_boost = if result.framework.is_some() { 0.2 } else { 0.0 };

    (language_confidence + framework_boost).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_detection() {
        let files = vec![
            ChangedFile {
                path: "src/main.rs".to_string(),
                additions: 10,
                deletions: 5,
            },
            ChangedFile {
                path: "src/lib.rs".to_string(),
                additions: 20,
                deletions: 0,
            },
        ];

        let (result, agent) = detect_full(&files, None);
        assert_eq!(result.primary_language, Some(Language::Rust));
        assert_eq!(agent, Agent::Rex);
    }

    #[test]
    fn test_nextjs_detection() {
        let files = vec![
            ChangedFile {
                path: "components/UserList.tsx".to_string(),
                additions: 50,
                deletions: 10,
            },
            ChangedFile {
                path: "app/page.tsx".to_string(),
                additions: 20,
                deletions: 5,
            },
        ];

        let package_json = r#"{"dependencies": {"next": "15.0.0", "react": "19.0.0"}}"#;
        let (result, agent) = detect_full(&files, Some(package_json));

        assert_eq!(result.primary_language, Some(Language::TypeScript));
        assert_eq!(result.framework, Some(Framework::NextJs));
        assert_eq!(agent, Agent::Blaze);
    }

    #[test]
    fn test_expo_detection() {
        let files = vec![ChangedFile {
            path: "app/(tabs)/home.tsx".to_string(),
            additions: 50,
            deletions: 10,
        }];

        let package_json = r#"{"dependencies": {"expo": "52.0.0", "react-native": "0.76.0"}}"#;
        let (result, agent) = detect_full(&files, Some(package_json));

        assert_eq!(result.framework, Some(Framework::Expo));
        assert_eq!(agent, Agent::Tap);
    }

    #[test]
    fn test_electron_detection() {
        let files = vec![ChangedFile {
            path: "src/main/index.ts".to_string(),
            additions: 30,
            deletions: 5,
        }];

        let package_json = r#"{"dependencies": {"electron": "29.0.0", "react": "19.0.0"}}"#;
        let (result, agent) = detect_full(&files, Some(package_json));

        assert_eq!(result.framework, Some(Framework::Electron));
        assert_eq!(agent, Agent::Spark);
    }
}
