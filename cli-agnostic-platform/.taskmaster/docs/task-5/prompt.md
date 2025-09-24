# AI Agent Prompt: Template Management System

You are a senior template engine architect with expertise in dynamic content generation, Handlebars templating, hot-reloading systems, and high-performance template rendering. Your mission is to create the intelligent template management system that will adapt content for 8 different CLI tools while maintaining performance and developer experience.

## Your Critical Mission
Build a sophisticated template management system that provides CLI-specific content generation with intelligent fallback mechanisms, hot reloading for development, versioning for gradual rollouts, and sub-10ms rendering performance. This system is the content adaptation layer that makes the Multi-CLI Agent Platform both powerful and maintainable.

## Template Challenge
You must handle diverse CLI requirements:
- **Claude**: JSON configs, Human/Assistant prompts, CLAUDE.md memory
- **Codex**: TOML configs, system prompts, AGENTS.md memory, sandbox presets
- **Opencode**: JSON/JSONC configs, TypeScript-specific prompts, package.json templates
- **Gemini**: JSON configs, multimodal prompts, GEMINI.md memory, Google Cloud setup
- **Grok**: JSON configs, .grok/GROK.md subdirectory patterns
- **Python CLIs**: YAML configs, virtualenv setup, session-based memory

## Technical Architecture Requirements

### 1. Template Management Core
```rust
pub struct TemplateManager {
    registry: HashMap<TemplateKey, CompiledTemplate>,
    handlebars: Handlebars<'static>,
    fallback_chain: FallbackChain,
    hot_reloader: Option<HotReloader>,
    performance_cache: Arc<RwLock<LruCache<String, RenderedOutput>>>,
    metrics: TemplateMetrics,
    version_manager: TemplateVersionManager,
}
```

### 2. Intelligent Template Organization
```
templates/
├── claude/
│   ├── system_prompt.hbs     # Claude-specific prompts
│   ├── config_file.hbs       # JSON configuration
│   ├── entrypoint_script.hbs # Container initialization
│   └── memory_file.hbs       # CLAUDE.md template
├── codex/
│   ├── system_prompt.hbs     # Codex-optimized prompts
│   ├── config_file.toml.hbs  # TOML configuration
│   ├── entrypoint_script.hbs # Rust environment setup
│   └── memory_file.hbs       # AGENTS.md template
├── generic/                  # Fallback templates
│   ├── system_prompt.hbs
│   ├── config_file.hbs
│   └── entrypoint_script.hbs
└── defaults/                 # Last resort fallbacks
    └── fallback_template.hbs
```

## Implementation Strategy

### Phase 1: Core Template Engine
1. Design TemplateManager with registry and discovery system
2. Implement Handlebars setup with CLI-specific custom helpers
3. Create template key system for organization and lookup
4. Build template discovery from filesystem with caching

### Phase 2: Intelligent Selection System
1. Implement sophisticated fallback chain logic
2. Create template selection with CLI capability awareness
3. Add template inheritance patterns (e.g., Qwen inherits from Gemini)
4. Build template conflict resolution strategies

### Phase 3: Advanced Features
1. Add hot reloading with file system watching
2. Implement template versioning with gradual rollout
3. Create performance optimization with compiled template caching
4. Build comprehensive metrics and monitoring

### Phase 4: Development Experience
1. Add template validation and syntax checking
2. Implement template debugging and preview tools
3. Create template testing framework
4. Build documentation generation from templates

## Critical Performance Requirements

### Template Rendering Speed
- **Target**: Sub-10ms rendering for typical templates
- **Throughput**: 1000+ templates per second sustained
- **Memory**: <2MB for 100 compiled templates
- **Cache Hit Ratio**: >90% for repeated renders with same context

### Hot Reloading Performance
- **Reload Time**: <100ms for template changes
- **Zero Downtime**: No service interruption during reloads
- **Debouncing**: Handle rapid file changes efficiently
- **Selective Updates**: Only reload changed templates

## Advanced Template Features

### Custom Handlebars Helpers
```rust
// CLI-specific helpers
hb.register_helper("cli_config_format", Box::new(|h, _hb, ctx, _rc, out| {
    let cli_type = h.param(0).unwrap().value().as_str().unwrap();
    let format = match cli_type {
        "codex" => "toml",
        "claude" | "opencode" | "gemini" => "json",
        "cursor" | "openhands" => "yaml",
        _ => "json"
    };
    out.write(format)?;
    Ok(())
}));

// Conditional rendering
hb.register_helper("if_streaming", Box::new(streaming_helper));
hb.register_helper("if_multimodal", Box::new(multimodal_helper));
hb.register_helper("model_context_window", Box::new(context_window_helper));
```

### Intelligent Fallback Chain
```rust
pub enum FallbackStrategy {
    CliSpecific(CLIType),    // templates/claude/system_prompt.hbs
    Inherited(CLIType),      // Qwen -> Gemini inheritance
    Generic,                 // templates/generic/system_prompt.hbs
    Default,                 // templates/defaults/fallback.hbs
}

impl FallbackChain {
    pub fn build_for(cli_type: CLIType) -> Vec<FallbackStrategy> {
        match cli_type {
            CLIType::Qwen => vec![
                FallbackStrategy::CliSpecific(CLIType::Qwen),
                FallbackStrategy::Inherited(CLIType::Gemini), // Inherit from Gemini
                FallbackStrategy::Generic,
                FallbackStrategy::Default,
            ],
            _ => vec![
                FallbackStrategy::CliSpecific(cli_type),
                FallbackStrategy::Generic,
                FallbackStrategy::Default,
            ]
        }
    }
}
```

### Template Context System
```rust
#[derive(Debug, Clone, Serialize)]
pub struct TemplateContext {
    // Agent identification
    pub agent_name: String,
    pub github_app: String,
    pub repository: RepositoryInfo,
    
    // CLI configuration
    pub cli_type: CLIType,
    pub cli_config: serde_json::Value,
    pub model: String,
    pub capabilities: CliCapabilities,
    
    // Runtime environment
    pub workspace_path: PathBuf,
    pub environment: HashMap<String, String>,
    pub container_info: ContainerInfo,
    
    // Template metadata
    pub template_version: Option<String>,
    pub render_timestamp: DateTime<Utc>,
    pub custom: HashMap<String, serde_json::Value>,
}
```

## Development Workflow Features

### Hot Reloading System
```rust
pub struct HotReloader {
    watcher: RecommendedWatcher,
    template_manager: Arc<RwLock<TemplateManager>>,
    reload_debouncer: Debouncer<RecommendedWatcher>,
}

// Features:
// - File system watching with notify-rs
// - Debounced reloading (500ms delay)
// - Selective template updates
// - Error recovery on invalid templates
// - Development-only activation
```

### Template Versioning
```rust
pub struct TemplateVersionManager {
    versions: HashMap<String, VersionedTemplateSet>,
    rollout_config: RolloutConfig,
}

// Gradual rollout capabilities:
// - Percentage-based rollouts
// - Consistent agent assignment
// - A/B testing support
// - Automatic rollback on errors
```

## Quality and Security Requirements

### Template Security
- **Sandboxed Execution**: No arbitrary code execution in templates
- **Input Sanitization**: All context variables properly escaped
- **Safe Helpers**: Only approved helper functions available
- **Resource Limits**: Memory and CPU limits for template rendering

### Error Handling Excellence
- **Graceful Degradation**: Fallback to default templates on errors
- **Detailed Error Messages**: Clear context about template failures
- **Recovery Mechanisms**: Automatic retry and fallback strategies
- **Development Feedback**: Rich error information for template debugging

### Performance Monitoring
```rust
pub struct TemplateMetrics {
    render_histogram: Histogram,      // Render time distribution
    cache_hits: Counter,              // Cache effectiveness
    cache_misses: Counter,
    template_errors: Counter,         // Error rates per template
    active_templates: Gauge,          // Currently loaded templates
    hot_reload_events: Counter,       // Development reload frequency
}
```

## Testing Strategy

### Comprehensive Test Coverage
```rust
#[tokio::test]
async fn test_template_fallback_chain() {
    // Test that missing CLI-specific templates fall back correctly
    let manager = TemplateManager::new().await.unwrap();
    let result = manager.render_template(
        CLIType::NewUnsupportedCLI,
        TemplateType::SystemPrompt,
        &create_test_context()
    ).await;
    
    assert!(result.is_ok());
    // Verify fallback path was used
}

#[tokio::test]
async fn test_performance_requirements() {
    // Test 1000 renders per second capability
    let manager = TemplateManager::new().await.unwrap();
    let start = Instant::now();
    
    for _ in 0..1000 {
        manager.render_template(
            CLIType::Claude,
            TemplateType::SystemPrompt,
            &create_test_context()
        ).await.unwrap();
    }
    
    assert!(start.elapsed() < Duration::from_secs(1));
}

#[tokio::test]
async fn test_hot_reloading() {
    // Test template updates without service disruption
    let manager = create_manager_with_hot_reload().await;
    
    // Modify template file
    modify_template_file("templates/claude/system_prompt.hbs").await;
    
    // Wait for reload
    tokio::time::sleep(Duration::from_millis(600)).await;
    
    // Verify new template is used
    let result = manager.render_template(
        CLIType::Claude,
        TemplateType::SystemPrompt,
        &create_test_context()
    ).await.unwrap();
    
    assert!(result.contains("updated content"));
}
```

## Success Criteria
Your implementation succeeds when:
- ✅ Template selection with intelligent fallback chains
- ✅ Sub-10ms rendering performance for typical templates
- ✅ 1000+ templates per second sustained throughput
- ✅ Hot reloading without service interruption (<100ms updates)
- ✅ Template versioning with gradual rollout capabilities
- ✅ CLI-specific custom helpers provide rich functionality
- ✅ Memory usage <2MB for 100 compiled templates
- ✅ Cache hit ratio >90% for repeated renders
- ✅ Comprehensive error handling with graceful degradation
- ✅ Security controls prevent template injection attacks

## Constraints and Considerations
- Template files must be version-controlled and auditable
- Hot reloading should be development-only for security
- Template inheritance should be clearly documented
- Performance monitoring must be comprehensive
- Security scanning for template injection vulnerabilities
- Multi-tenancy support for different template sets
- Backward compatibility during template schema changes
- Resource cleanup for unused templates

## Deliverables
1. Complete TemplateManager with discovery and registry
2. Sophisticated fallback chain with inheritance support
3. High-performance template rendering with caching
4. Hot reloading system for development workflow
5. Template versioning with gradual rollout capabilities
6. Custom Handlebars helpers for CLI-specific logic
7. Comprehensive metrics and performance monitoring
8. Security controls and input validation
9. Complete test suite with performance benchmarks
10. Documentation and examples for template creation

This template management system is the content intelligence layer that makes each CLI feel native while maintaining platform consistency. Focus on performance, developer experience, and extensibility.