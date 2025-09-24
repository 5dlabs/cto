# Acceptance Criteria: Template Management System

## Functional Requirements

### FR-1: Dynamic Template Discovery and Loading
**Requirement**: Automatic discovery and loading of templates from filesystem
- [ ] Template discovery walks directory structure: templates/{cli_type}/{template_type}.hbs
- [ ] Support for template types: SystemPrompt, ConfigFile, EntrypointScript, MemoryFile
- [ ] Template compilation and caching for performance
- [ ] Template validation during loading with syntax checking
- [ ] Checksum calculation for change detection

### FR-2: Intelligent Fallback Chain
**Requirement**: Multi-level fallback for missing templates
- [ ] CLI-specific template selection (templates/claude/system_prompt.hbs)
- [ ] Generic fallback (templates/generic/system_prompt.hbs)
- [ ] Default fallback (templates/defaults/fallback_template.hbs)
- [ ] Template inheritance (Qwen inherits from Gemini)
- [ ] Fallback strategy logging for debugging

### FR-3: High-Performance Template Rendering
**Requirement**: Sub-10ms rendering with high throughput
- [ ] Template rendering <10ms for typical templates
- [ ] Sustained throughput of 1000+ templates per second
- [ ] LRU cache for rendered templates with configurable TTL
- [ ] Memory usage <2MB for 100 compiled templates
- [ ] Cache hit ratio >90% for repeated contexts

**Verification**:
```rust
#[tokio::test]
async fn test_performance_requirements() {
    let manager = TemplateManager::new().await.unwrap();

    // Test single render performance
    let start = Instant::now();
    manager.render_template(CLIType::Claude, TemplateType::SystemPrompt, &context).await.unwrap();
    assert!(start.elapsed() < Duration::from_millis(10));

    // Test sustained throughput
    let start = Instant::now();
    for _ in 0..1000 {
        manager.render_template(CLIType::Claude, TemplateType::SystemPrompt, &context).await.unwrap();
    }
    assert!(start.elapsed() < Duration::from_secs(1));
}
```

### FR-4: Advanced Handlebars Helpers
**Requirement**: CLI-specific template helpers and conditional rendering
- [ ] cli_config_format helper returns correct format (JSON/TOML/YAML)
- [ ] if_streaming helper for conditional streaming support
- [ ] if_multimodal helper for multimodal capability detection
- [ ] model_context_window helper returns correct token limits
- [ ] escape_shell helper for safe shell script generation
- [ ] Custom helpers work with all CLI types

**Verification**:
```rust
#[tokio::test]
async fn test_custom_helpers() {
    let manager = TemplateManager::new().await.unwrap();

    let template = "{{cli_config_format cli_type}}";
    let context = create_context_for_cli(CLIType::Codex);
    let result = manager.render_inline_template(template, &context).await.unwrap();
    assert_eq!(result, "toml");

    let context = create_context_for_cli(CLIType::Claude);
    let result = manager.render_inline_template(template, &context).await.unwrap();
    assert_eq!(result, "json");
}
```

### FR-5: Hot Reloading for Development
**Requirement**: Real-time template updates without service restart
- [ ] File system watching with notify-rs v7.0.0
- [ ] Debounced reloading with 500ms delay
- [ ] Selective template updates (only changed files)
- [ ] Error recovery on invalid template syntax
- [ ] Development-mode only activation
- [ ] <100ms reload time for template changes

**Verification**:
```rust
#[tokio::test]
async fn test_hot_reloading() {
    let manager = create_manager_with_hot_reload().await;

    // Initial render
    let initial = manager.render_template(CLIType::Claude, TemplateType::SystemPrompt, &context).await.unwrap();

    // Modify template file
    tokio::fs::write("templates/claude/system_prompt.hbs", "Updated: {{agent_name}}").await.unwrap();

    // Wait for reload
    tokio::time::sleep(Duration::from_millis(600)).await;

    // Verify updated template is used
    let updated = manager.render_template(CLIType::Claude, TemplateType::SystemPrompt, &context).await.unwrap();
    assert!(updated.starts_with("Updated:"));
    assert_ne!(initial, updated);
}
```

## Non-Functional Requirements

### NFR-1: Performance and Scalability
**Requirement**: High-performance template operations
- [ ] Template compilation time <50ms per template
- [ ] Memory usage linear with template count
- [ ] Concurrent rendering support (100+ simultaneous)
- [ ] Cache effectiveness >90% hit ratio
- [ ] No memory leaks in long-running operations

### NFR-2: Security
**Requirement**: Secure template execution
- [ ] No arbitrary code execution in templates
- [ ] Input sanitization and escaping
- [ ] Resource limits for template rendering
- [ ] Safe helper functions only
- [ ] Template injection prevention

### NFR-3: Observability
**Requirement**: Comprehensive monitoring
- [ ] Render time histograms by CLI and template type
- [ ] Cache hit/miss ratios
- [ ] Template error rates and types
- [ ] Hot reload event tracking
- [ ] Performance dashboard integration

## Test Cases

### TC-1: Template Fallback Chain
**Scenario**: Missing CLI-specific template uses fallbacks
```rust
#[tokio::test]
async fn test_fallback_chain() {
    let manager = TemplateManager::new().await.unwrap();

    // Request template for CLI without specific template
    let result = manager.render_template(
        CLIType::NewUnsupportedCLI,
        TemplateType::SystemPrompt,
        &create_test_context()
    ).await;

    assert!(result.is_ok());
    // Should have used generic fallback
    let audit_trail = manager.get_last_render_audit();
    assert_eq!(audit_trail.strategy_used, FallbackStrategy::Generic);
}
```

### TC-2: Template Versioning
**Scenario**: Gradual rollout of new template versions
```rust
#[tokio::test]
async fn test_template_versioning() {
    let mut manager = TemplateManager::with_versioning().await.unwrap();

    // Deploy new template version at 10% rollout
    manager.deploy_template_version("v2.0", 10.0).await.unwrap();

    // Test consistent assignment
    let agent_hash = manager.hash_agent_name("consistent-agent");
    let version1 = manager.select_version_for_agent("consistent-agent").await;
    let version2 = manager.select_version_for_agent("consistent-agent").await;
    assert_eq!(version1, version2);

    // Test rollout percentage
    let mut v2_count = 0;
    for i in 0..100 {
        let version = manager.select_version_for_agent(&format!("agent-{}", i)).await;
        if version == "v2.0" {
            v2_count += 1;
        }
    }
    assert!((v2_count as f64 - 10.0).abs() < 5.0); // Within 5% of target
}
```

### TC-3: Template Context Handling
**Scenario**: Rich context data for template rendering
```rust
#[tokio::test]
async fn test_template_context() {
    let manager = TemplateManager::new().await.unwrap();
    let context = TemplateContext {
        agent_name: "test-agent".to_string(),
        cli_type: CLIType::Claude,
        model: "claude-3-opus".to_string(),
        capabilities: CliCapabilities {
            supports_streaming: true,
            max_context_tokens: 200_000,
            ..Default::default()
        },
        ..Default::default()
    };

    let template = "Agent {{agent_name}} uses {{model}} with {{#if_streaming}}streaming{{else}}no streaming{{/if_streaming}}";
    let result = manager.render_inline_template(template, &context).await.unwrap();

    assert_eq!(result, "Agent test-agent uses claude-3-opus with streaming");
}
```

### TC-4: Error Handling and Recovery
**Scenario**: Graceful handling of template errors
```rust
#[tokio::test]
async fn test_error_handling() {
    let manager = TemplateManager::new().await.unwrap();

    // Test invalid template syntax
    let invalid_template = "{{#invalid_helper}}content{{/invalid_helper}}";
    let result = manager.render_inline_template(invalid_template, &context).await;

    match result {
        Err(TemplateError::HelperNotFound { helper_name, suggestions }) => {
            assert_eq!(helper_name, "invalid_helper");
            assert!(!suggestions.is_empty());
        }
        _ => panic!("Expected HelperNotFound error"),
    }

    // Test fallback on template error
    let context_without_required_field = TemplateContext::minimal();
    let result = manager.render_template(
        CLIType::Claude,
        TemplateType::SystemPrompt,
        &context_without_required_field
    ).await;

    // Should fall back to default template
    assert!(result.is_ok());
}
```

### TC-5: Cache Effectiveness
**Scenario**: Template caching improves performance
```rust
#[tokio::test]
async fn test_cache_effectiveness() {
    let manager = TemplateManager::new().await.unwrap();
    let context = create_test_context();

    // First render (cache miss)
    let start = Instant::now();
    let result1 = manager.render_template(CLIType::Claude, TemplateType::SystemPrompt, &context).await.unwrap();
    let first_duration = start.elapsed();

    // Second render (cache hit)
    let start = Instant::now();
    let result2 = manager.render_template(CLIType::Claude, TemplateType::SystemPrompt, &context).await.unwrap();
    let cached_duration = start.elapsed();

    assert_eq!(result1, result2);
    assert!(cached_duration < first_duration / 2); // Significant speedup

    let metrics = manager.get_cache_metrics();
    assert!(metrics.hit_ratio > 0.5);
}
```

## Quality Gates

### Performance Benchmarks
- [ ] Template rendering <10ms (p95)
- [ ] Throughput >1000 templates/second sustained
- [ ] Memory usage <2MB for 100 templates
- [ ] Cache hit ratio >90% after warmup
- [ ] Hot reload time <100ms

### Security Validation
- [ ] No template injection vulnerabilities
- [ ] Input sanitization prevents XSS
- [ ] Resource limits prevent DoS
- [ ] Safe helper functions only

### Code Quality
- [ ] All code passes cargo clippy
- [ ] Code coverage >95% for template logic
- [ ] Comprehensive error handling
- [ ] Clear documentation and examples

## Definition of Done
Task 5 is considered complete when:
- [ ] All functional requirements implemented and tested
- [ ] Performance requirements met under load
- [ ] Security requirements satisfied
- [ ] Hot reloading works in development mode
- [ ] Template versioning system operational
- [ ] Comprehensive test coverage with benchmarks
- [ ] Integration with CLI adapter system
- [ ] Monitoring and metrics collection
- [ ] Documentation and examples complete

## Rollback Criteria
Immediate rollback if:
- Template rendering performance regression >50%
- Template compilation errors break existing functionality
- Hot reloading causes service instability
- Cache system failures affect response times
- Security vulnerabilities in template execution
- Memory leaks in template operations