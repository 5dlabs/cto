# Acceptance Criteria: Configuration Resolution Engine

## Functional Requirements

### FR-1: Multi-Source Configuration Loading
**Requirement**: Load configurations from all specified sources
- [ ] Helm ConfigMap loader retrieves agent.agentCliConfigs values
- [ ] CtoConfig loader parses repository cto-config.json files
- [ ] Environment loader reads CLI-specific environment variables
- [ ] Runtime parameter loader handles immediate execution overrides
- [ ] Git integration clones/pulls repositories for cto-config.json access
- [ ] Kubernetes client integration for ConfigMap access
- [ ] Error handling for missing or inaccessible configuration sources

**Verification**:
```rust
#[tokio::test]
async fn test_multi_source_loading() {
    let resolver = AgentResolver::new().await.unwrap();

    // Test Helm loading
    let helm_config = resolver.helm_loader.load_for_agent("test-agent").await;
    assert!(helm_config.is_ok());

    // Test CtoConfig loading
    let cto_config = resolver.cto_loader.load_for_agent("test-agent").await;
    assert!(cto_config.is_ok());

    // Test environment loading
    let env_config = resolver.env_loader.load_for_agent("test-agent").await;
    assert!(env_config.is_ok());
}
```

### FR-2: Configuration Precedence Rules
**Requirement**: Apply correct precedence when merging configurations
- [ ] Runtime Parameters have highest precedence
- [ ] Environment Variables override CtoConfig and Helm
- [ ] CtoConfig overrides Helm defaults
- [ ] Helm provides base configuration
- [ ] Built-in defaults serve as fallback
- [ ] Precedence documented and tested with conflicting values

**Verification**:
```rust
#[tokio::test]
async fn test_precedence_rules() {
    let resolver = create_resolver_with_test_configs().await;

    // Set up conflicting values across sources
    let runtime_params = RuntimeParameters {
        model: Some("runtime-model".to_string()),
    };

    let resolved = resolver.resolve_agent_config("test-agent", Some(runtime_params)).await.unwrap();

    // Runtime should win
    assert_eq!(resolved.model, "runtime-model");
}
```

### FR-3: Advanced Merge Strategies
**Requirement**: Support sophisticated merge strategies for different field types
- [ ] Replace strategy completely replaces values
- [ ] Merge strategy combines objects, replaces primitives
- [ ] Append strategy concatenates arrays with deduplication
- [ ] DeepMerge strategy recursively merges nested structures
- [ ] Configurable merge strategy per field path
- [ ] Conflict detection and resolution

**Verification**:
```rust
#[tokio::test]
async fn test_merge_strategies() {
    let merge_engine = MergeEngine::new();

    let base = json!({
        \"tools\": {
            \"remote\": [\"tool1\"],
            \"local\": {\"filesystem\": {\"enabled\": true}}
        }
    });

    let overlay = json!({
        \"tools\": {
            \"remote\": [\"tool2\"],
            \"local\": {\"git\": {\"enabled\": true}}
        }
    });

    let result = merge_engine.merge_values(base, overlay, &MergeStrategy::DeepMerge).await.unwrap();

    // Verify deep merge combined arrays and objects
    assert_eq!(result[\"tools\"][\"remote\"].as_array().unwrap().len(), 2);
    assert!(result[\"tools\"][\"local\"][\"filesystem\"][\"enabled\"].as_bool().unwrap());
    assert!(result[\"tools\"][\"local\"][\"git\"][\"enabled\"].as_bool().unwrap());
}
```

### FR-4: JSON Schema Validation
**Requirement**: Comprehensive validation with helpful error messages
- [ ] JSON Schema validation using jsonschema-rs v0.20.0
- [ ] Schema versioning support for backward compatibility
- [ ] Detailed validation error messages with field paths
- [ ] Suggestion generation for common configuration mistakes
- [ ] Validation of final merged configuration before application
- [ ] Support for custom validation rules per CLI type

**Verification**:
```rust
#[tokio::test]
async fn test_schema_validation() {
    let validator = ConfigurationValidator::new().await.unwrap();

    // Test valid configuration
    let valid_config = create_valid_agent_config();
    let result = validator.validate(&valid_config).await.unwrap();
    assert!(result.is_valid());

    // Test invalid configuration with helpful errors
    let invalid_config = create_invalid_agent_config();
    let result = validator.validate(&invalid_config).await.unwrap();
    assert!(!result.is_valid());
    assert!(!result.errors.is_empty());
    assert!(result.errors[0].suggestion.is_some());
}
```

### FR-5: Redis-Based Configuration Caching
**Requirement**: High-performance caching with compression and TTL
- [ ] Redis integration using redis-rs v0.27.0
- [ ] 5-minute TTL for cached configurations
- [ ] Compression for cache storage efficiency
- [ ] Cache invalidation on configuration source changes
- [ ] Cache warming strategies for frequently accessed configs
- [ ] Performance monitoring with hit/miss ratios

**Verification**:
```rust
#[tokio::test]
async fn test_configuration_caching() {
    let cache = ConfigurationCache::new().await.unwrap();
    let config = create_test_configuration();

    // Test cache store
    cache.store(\"test-agent\", &config).await.unwrap();

    // Test cache retrieve
    let cached = cache.get(\"test-agent\").await.unwrap();
    assert!(cached.is_some());
    assert_eq!(cached.unwrap().config, config);

    // Test TTL expiration
    tokio::time::sleep(Duration::from_secs(301)).await; // 5 minutes + 1 second
    let expired = cache.get(\"test-agent\").await.unwrap();
    assert!(expired.is_none());
}
```

## Non-Functional Requirements

### NFR-1: Performance
**Requirement**: High-performance configuration resolution
- [ ] Configuration resolution <200ms for complex merges
- [ ] Cache hit ratio >85% for repeated agent lookups
- [ ] Memory usage <50MB for 1000 active configurations
- [ ] Support 100+ concurrent configuration resolutions
- [ ] Response time SLA maintained under load

**Verification**:
```rust
#[tokio::test]
async fn test_performance_requirements() {
    let resolver = AgentResolver::new().await.unwrap();

    // Test resolution time
    let start = Instant::now();
    resolver.resolve_agent_config(\"test-agent\", None).await.unwrap();
    assert!(start.elapsed() < Duration::from_millis(200));

    // Test concurrent resolution
    let mut handles = vec![];
    for i in 0..100 {
        let resolver = resolver.clone();
        handles.push(tokio::spawn(async move {
            resolver.resolve_agent_config(&format!(\"agent-{}\", i), None).await
        }));
    }

    let start = Instant::now();
    for handle in handles {
        handle.await.unwrap().unwrap();
    }
    assert!(start.elapsed() < Duration::from_secs(10)); // All 100 in <10s
}
```

### NFR-2: Reliability
**Requirement**: Robust error handling and recovery
- [ ] Graceful degradation when configuration sources unavailable
- [ ] Automatic retry with exponential backoff for transient failures
- [ ] Circuit breaker pattern for failing external services
- [ ] Comprehensive error types with actionable messages
- [ ] No data loss during configuration merging

### NFR-3: Observability
**Requirement**: Comprehensive monitoring and audit trail
- [ ] Configuration resolution metrics (success/failure rates, latency)
- [ ] Cache performance metrics (hit/miss ratios, size)
- [ ] Audit logging for all configuration changes
- [ ] Correlation ID tracking across operations
- [ ] Performance dashboards and alerting

### NFR-4: Security
**Requirement**: Secure configuration handling
- [ ] Secure credential handling in configuration sources
- [ ] No sensitive data logged or cached insecurely
- [ ] Configuration validation prevents injection attacks
- [ ] Role-based access control integration
- [ ] Audit compliance for configuration changes

## Test Cases

### TC-1: Complex Configuration Merge
**Scenario**: Multiple sources with overlapping and conflicting values
```rust
#[tokio::test]
async fn test_complex_merge_scenario() {
    let resolver = create_resolver_with_complex_configs().await;

    // Helm provides base config
    let helm_config = json!({
        \"model\": \"claude-3-opus\",
        \"temperature\": 0.7,
        \"tools\": {
            \"remote\": [\"base_tool\"],
            \"local\": {\"filesystem\": {\"enabled\": true}}
        }
    });

    // CtoConfig overrides some values
    let cto_config = json!({
        \"model\": \"claude-3-sonnet\",  // Override model
        \"tools\": {
            \"remote\": [\"repo_tool\"],  // Add to remote tools
            \"local\": {\"git\": {\"enabled\": true}}  // Add git tools
        }
    });

    let resolved = resolver.resolve_agent_config(\"test-agent\", None).await.unwrap();

    // Verify merge results
    assert_eq!(resolved.model, \"claude-3-sonnet\"); // CtoConfig won
    assert_eq!(resolved.temperature, 0.7); // Helm value preserved
    assert_eq!(resolved.tools.remote.len(), 2); // Arrays merged
    assert!(resolved.tools.local.filesystem.enabled); // Objects merged
    assert!(resolved.tools.local.git.enabled); // New tool added
}
```

### TC-2: Configuration Preview
**Scenario**: Preview changes before applying them
```rust
#[tokio::test]
async fn test_configuration_preview() {
    let preview_service = ConfigurationPreview::new().await.unwrap();

    let proposed_changes = ProposedChanges {
        model: Some(\"new-model\".to_string()),
        temperature: Some(0.9),
    };

    let preview = preview_service.preview_configuration(\"test-agent\", proposed_changes).await.unwrap();

    // Verify preview shows what will change
    assert!(preview.diff.changes.contains_key(\"model\"));
    assert!(preview.diff.changes.contains_key(\"temperature\"));
    assert!(preview.validation.is_valid());
    assert!(!preview.warnings.is_empty()); // Should warn about temperature change
}
```

### TC-3: Cache Effectiveness
**Scenario**: Verify caching improves performance and reduces load
```rust
#[tokio::test]
async fn test_cache_effectiveness() {
    let resolver = AgentResolver::new().await.unwrap();

    // First resolution (cache miss)
    let start = Instant::now();
    resolver.resolve_agent_config(\"test-agent\", None).await.unwrap();
    let first_duration = start.elapsed();

    // Second resolution (cache hit)
    let start = Instant::now();
    resolver.resolve_agent_config(\"test-agent\", None).await.unwrap();
    let cached_duration = start.elapsed();

    // Cache should be significantly faster
    assert!(cached_duration < first_duration / 2);

    // Check cache metrics
    let metrics = resolver.get_cache_metrics().await.unwrap();
    assert!(metrics.hit_ratio > 0.5); // At least 50% hit ratio
}
```

### TC-4: Error Handling and Recovery
**Scenario**: Handle various error conditions gracefully
```rust
#[tokio::test]
async fn test_error_handling() {
    let resolver = create_resolver_with_failing_sources().await;

    // Test missing configuration source
    let result = resolver.resolve_agent_config(\"nonexistent-agent\", None).await;
    match result {
        Err(ConfigurationError::AgentNotFound { agent_name, suggestion }) => {
            assert_eq!(agent_name, \"nonexistent-agent\");
            assert!(suggestion.is_some());
        }
        _ => panic!(\"Expected AgentNotFound error\"),
    }

    // Test invalid configuration
    let invalid_runtime_params = RuntimeParameters {
        model: Some(\"\".to_string()), // Invalid empty model
    };

    let result = resolver.resolve_agent_config(\"test-agent\", Some(invalid_runtime_params)).await;
    assert!(result.is_err());
}
```

### TC-5: Audit Trail Completeness
**Scenario**: Verify all configuration operations are properly audited
```rust
#[tokio::test]
async fn test_audit_trail() {
    let resolver = AgentResolver::new().await.unwrap();
    let audit_store = resolver.audit_logger.event_store.clone();

    let config = resolver.resolve_agent_config(\"test-agent\", None).await.unwrap();

    // Check audit event was created
    let events = audit_store.get_events_for_agent(\"test-agent\").await.unwrap();
    assert!(!events.is_empty());

    let resolution_event = &events[0];
    assert_eq!(resolution_event.event_type, AuditEventType::ConfigurationResolved);
    assert_eq!(resolution_event.agent_name, \"test-agent\");
    assert!(resolution_event.correlation_id.len() > 0);
    assert!(!resolution_event.details.sources_consulted.is_empty());
}
```

## Quality Gates

### Code Quality
- [ ] All code passes `cargo clippy` with zero warnings
- [ ] Code coverage >90% for configuration logic
- [ ] All public APIs have comprehensive documentation
- [ ] Error messages are actionable and user-friendly

### Performance Benchmarks
- [ ] Configuration resolution <200ms (p95)
- [ ] Cache hit ratio >85% after warmup period
- [ ] Memory usage stable under concurrent load
- [ ] No memory leaks in long-running tests

### Security Validation
- [ ] No credentials logged or exposed in error messages
- [ ] Configuration validation prevents common injection attacks
- [ ] Audit trail captures all security-relevant events
- [ ] Rate limiting prevents configuration DoS attacks

## Integration Requirements

### IR-1: CLI Adapter Integration
**Requirement**: Seamless integration with Task 3 CLI adapters
- [ ] Resolved configurations compatible with all CLI adapter types
- [ ] Configuration format validation per CLI requirements
- [ ] Adapter-specific configuration sections properly handled

### IR-2: Kubernetes Integration
**Requirement**: Integration with Kubernetes configuration management
- [ ] ConfigMap watching for automatic configuration updates
- [ ] Helm value overrides properly applied
- [ ] Kubernetes RBAC integration for configuration access

### IR-3: Monitoring Integration
**Requirement**: Integration with observability stack
- [ ] Prometheus metrics export
- [ ] Jaeger tracing integration
- [ ] Structured logging with correlation IDs
- [ ] Custom dashboards for configuration monitoring

## Edge Cases and Error Scenarios

### EC-1: Configuration Source Conflicts
**Scenario**: Multiple sources provide conflicting values
**Expected**: Clear precedence resolution with audit trail

### EC-2: Malformed Configuration Data
**Scenario**: Invalid JSON/YAML in configuration sources
**Expected**: Detailed error messages with suggestions for fixes

### EC-3: Network Failures
**Scenario**: Temporary unavailability of configuration sources
**Expected**: Graceful fallback to cached configurations with warnings

### EC-4: Large Configuration Objects
**Scenario**: Configurations exceeding normal size limits
**Expected**: Efficient handling with compression and streaming

### EC-5: Rapid Configuration Changes
**Scenario**: Frequent updates to configuration sources
**Expected**: Cache invalidation and update without service disruption

## Definition of Done
Task 4 is considered complete when:
- [ ] All functional requirements implemented and tested
- [ ] Performance requirements met under concurrent load
- [ ] Security requirements satisfied with audit compliance
- [ ] Integration requirements completed with all dependent systems
- [ ] Quality gates passed including code coverage and performance
- [ ] Edge cases handled gracefully with proper error recovery
- [ ] Documentation complete with examples and troubleshooting guides
- [ ] Production deployment successful with monitoring enabled
- [ ] Backward compatibility verified with existing configurations

## Rollback Criteria
Immediate rollback if:
- Configuration resolution fails for existing agents
- Performance regression >30% vs baseline
- Security vulnerabilities discovered in validation
- Data corruption in configuration merging
- Cache system failures affecting availability
- Audit system failures affecting compliance