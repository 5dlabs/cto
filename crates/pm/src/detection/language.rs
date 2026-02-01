//! Language detection from file extensions.

use serde::{Deserialize, Serialize};

/// Programming languages we can detect and route to agents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    Rust,
    Go,
    TypeScript,
    JavaScript,
    CSharp,
    Cpp,
    Python,
    Swift,
    Kotlin,
    Java,
    Ruby,
    Php,
    Shell,
    Yaml,
    Json,
    Markdown,
    Other,
}

impl Language {
    /// Human-readable name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Rust => "Rust",
            Self::Go => "Go",
            Self::TypeScript => "TypeScript",
            Self::JavaScript => "JavaScript",
            Self::CSharp => "C#",
            Self::Cpp => "C++",
            Self::Python => "Python",
            Self::Swift => "Swift",
            Self::Kotlin => "Kotlin",
            Self::Java => "Java",
            Self::Ruby => "Ruby",
            Self::Php => "PHP",
            Self::Shell => "Shell",
            Self::Yaml => "YAML",
            Self::Json => "JSON",
            Self::Markdown => "Markdown",
            Self::Other => "Other",
        }
    }

    /// Is this a "real" programming language (not config/docs)?
    pub fn is_code(&self) -> bool {
        !matches!(self, Self::Yaml | Self::Json | Self::Markdown | Self::Other)
    }
}

/// Detect language from file extension
pub fn detect_from_extension(ext: Option<&str>) -> Option<Language> {
    let ext = ext?;

    Some(match ext.to_lowercase().as_str() {
        // Rust
        "rs" => Language::Rust,

        // Go
        "go" => Language::Go,

        // TypeScript/JavaScript
        "ts" | "tsx" | "mts" | "cts" => Language::TypeScript,
        "js" | "jsx" | "mjs" | "cjs" => Language::JavaScript,

        // C#
        "cs" => Language::CSharp,

        // C/C++
        "cpp" | "cc" | "cxx" | "c" | "h" | "hpp" | "hxx" => Language::Cpp,

        // Python
        "py" | "pyi" | "pyw" => Language::Python,

        // Swift
        "swift" => Language::Swift,

        // Kotlin
        "kt" | "kts" => Language::Kotlin,

        // Java
        "java" => Language::Java,

        // Ruby
        "rb" | "rake" | "gemspec" => Language::Ruby,

        // PHP
        "php" => Language::Php,

        // Shell
        "sh" | "bash" | "zsh" | "fish" => Language::Shell,

        // Config/Data
        "yaml" | "yml" => Language::Yaml,
        "json" | "jsonc" => Language::Json,

        // Docs
        "md" | "mdx" => Language::Markdown,

        // Skip these - not useful for detection
        "lock" | "sum" | "toml" | "mod" => return None,

        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_extension() {
        assert_eq!(detect_from_extension(Some("rs")), Some(Language::Rust));
    }

    #[test]
    fn test_typescript_extensions() {
        assert_eq!(
            detect_from_extension(Some("ts")),
            Some(Language::TypeScript)
        );
        assert_eq!(
            detect_from_extension(Some("tsx")),
            Some(Language::TypeScript)
        );
        assert_eq!(
            detect_from_extension(Some("mts")),
            Some(Language::TypeScript)
        );
    }

    #[test]
    fn test_cpp_extensions() {
        assert_eq!(detect_from_extension(Some("cpp")), Some(Language::Cpp));
        assert_eq!(detect_from_extension(Some("h")), Some(Language::Cpp));
        assert_eq!(detect_from_extension(Some("hpp")), Some(Language::Cpp));
    }

    #[test]
    fn test_skip_lock_files() {
        assert_eq!(detect_from_extension(Some("lock")), None);
        assert_eq!(detect_from_extension(Some("sum")), None);
    }
}
