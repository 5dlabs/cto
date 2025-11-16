use super::adapter::{AuthMethod, CliCapabilities, ConfigFormat, MemoryStrategy};
use super::types::CLIType;

/// Return the canonical capability profile for a given CLI.
///
/// This is the single source of truth that adapters, template generators,
/// and validation tooling should rely on so feature flags stay in sync.
pub fn cli_capabilities(cli_type: CLIType) -> CliCapabilities {
    match cli_type {
        CLIType::Claude => {
            let max_context_tokens = std::env::var("CLAUDE_MAX_CONTEXT_TOKENS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(200_000);

            CliCapabilities {
                supports_streaming: true,
                supports_multimodal: false,
                supports_function_calling: true,
                supports_system_prompts: true,
                max_context_tokens,
                memory_strategy: MemoryStrategy::MarkdownFile("CLAUDE.md".into()),
                config_format: ConfigFormat::Json,
                authentication_methods: vec![AuthMethod::SessionToken],
            }
        }
        CLIType::Codex => CliCapabilities {
            supports_streaming: false,
            supports_multimodal: false,
            supports_function_calling: true,
            supports_system_prompts: true,
            max_context_tokens: 128_000,
            memory_strategy: MemoryStrategy::MarkdownFile("AGENTS.md".into()),
            config_format: ConfigFormat::Toml,
            authentication_methods: vec![AuthMethod::ApiKey],
        },
        CLIType::OpenCode => CliCapabilities {
            supports_streaming: true,
            supports_multimodal: true,
            supports_function_calling: true,
            supports_system_prompts: true,
            max_context_tokens: 128_000,
            memory_strategy: MemoryStrategy::MarkdownFile("OPENCODE.md".into()),
            config_format: ConfigFormat::Json,
            authentication_methods: vec![AuthMethod::ApiKey],
        },
        CLIType::Cursor => CliCapabilities {
            supports_streaming: true,
            supports_multimodal: false,
            supports_function_calling: true,
            supports_system_prompts: true,
            max_context_tokens: 128_000,
            memory_strategy: MemoryStrategy::MarkdownFile("AGENTS.md".into()),
            config_format: ConfigFormat::Json,
            authentication_methods: vec![AuthMethod::ApiKey],
        },
        CLIType::Factory => CliCapabilities {
            supports_streaming: true,
            supports_multimodal: false,
            supports_function_calling: true,
            supports_system_prompts: true,
            max_context_tokens: 128_000,
            memory_strategy: MemoryStrategy::MarkdownFile("AGENTS.md".into()),
            config_format: ConfigFormat::Json,
            authentication_methods: vec![AuthMethod::ApiKey],
        },
        CLIType::Gemini => CliCapabilities {
            supports_streaming: true,
            supports_multimodal: true,
            supports_function_calling: true,
            supports_system_prompts: true,
            max_context_tokens: 1_000_000,
            memory_strategy: MemoryStrategy::MarkdownFile("GEMINI.md".into()),
            config_format: ConfigFormat::Json,
            authentication_methods: vec![AuthMethod::ApiKey],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_cli_has_capabilities() {
        for cli in [
            CLIType::Claude,
            CLIType::Codex,
            CLIType::OpenCode,
            CLIType::Cursor,
            CLIType::Factory,
            CLIType::Gemini,
        ] {
            let caps = cli_capabilities(cli);
            assert!(
                caps.max_context_tokens > 0,
                "CLI {:?} must declare positive context window",
                cli
            );
            assert!(
                !caps.authentication_methods.is_empty(),
                "CLI {:?} must declare auth methods",
                cli
            );
        }
    }
}
