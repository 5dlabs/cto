# AI Agent Prompt: Configuration Resolution Engine

You are a senior configuration management engineer with expertise in multi-source configuration merging, distributed systems, and complex enterprise software configuration patterns. Your mission is to build the intelligent configuration resolution system that will unify Helm cluster defaults with repository-specific overrides.

## Your Critical Mission
Create a sophisticated configuration resolution engine that seamlessly merges configurations from multiple sources (Helm, cto-config.json, environment variables, runtime parameters) with clear precedence rules, field-level granularity, and comprehensive audit trails. This system is essential for the Multi-CLI Agent Platform's flexibility and maintainability.

## Configuration Challenge
You must handle complex scenarios like:
- **Cluster Operators**: Set secure defaults via Helm for all agents
- **Repository Owners**: Override specific settings in cto-config.json
- **Deployment Environments**: Apply environment-specific configurations
- **Runtime Operations**: Provide immediate parameter overrides
- **Audit Requirements**: Track all configuration decisions and changes

## Technical Requirements

### 1. Multi-Source Configuration Loading
Design loaders for each configuration source:
```rust
pub struct AgentResolver {
    helm_loader: HelmConfigLoader,      // Kubernetes ConfigMaps
    cto_loader: CtoConfigLoader,        // Repository cto-config.json
    env_loader: EnvironmentLoader,      // Environment variables
    runtime_loader: RuntimeLoader,      // Immediate overrides
    cache: Arc<ConfigurationCache>,     // Redis caching
    validator: ConfigurationValidator,  // JSON Schema validation
    audit_logger: AuditLogger,         // Complete audit trail
}
```

### 2. Intelligent Merge Strategies
Implement sophisticated merge algorithms:
```rust
pub enum MergeStrategy {
    Replace,    // Complete replacement for primitives
    Merge,      // Object merging with primitive replacement  
    Append,     // Array concatenation with deduplication
    DeepMerge,  // Recursive merge with conflict resolution
}

pub struct MergeEngine {
    strategies: HashMap<String, MergeStrategy>,
    conflict_resolver: ConflictResolver,
    precedence_rules: PrecedenceEngine,
}
```

### 3. Precedence and Conflict Resolution
Clear hierarchy with intelligent conflict handling:
```
Precedence (Highest → Lowest):
1. Runtime Parameters (immediate execution overrides)
2. Environment Variables (deployment-specific configuration)
3. CtoConfig Repository (repository-level cto-config.json)
4. Helm Values (cluster-level agent.agentCliConfigs)
5. Built-in Defaults (system fallback values)
```

## Implementation Strategy

### Phase 1: Core Resolution Framework
1. Design AgentResolver with comprehensive multi-source loading
2. Implement each configuration source loader with error handling
3. Create merge engine with configurable strategies per field
4. Add configuration precedence rules with conflict detection

### Phase 2: Advanced Merge Logic
1. Implement deep merge algorithm for nested JSON structures
2. Add array handling (append, merge, replace strategies)
3. Create conflict resolution with user-defined rules
4. Build merge tracing for audit and debugging purposes

### Phase 3: Validation and Schema
1. Implement JSON Schema validation using jsonschema-rs
2. Create detailed validation error messages with suggestions
3. Add schema versioning support for backward compatibility
4. Build validation pipeline with early error detection

### Phase 4: Caching and Performance
1. Implement Redis-based configuration cache with compression
2. Add TTL-based cache invalidation (5-minute default)
3. Create cache warming strategies for frequently accessed configs
4. Add performance monitoring and cache hit rate metrics

### Phase 5: Preview and Audit
1. Build configuration preview API showing changes before application
2. Implement comprehensive audit logging with correlation IDs
3. Create change diff calculation with impact assessment
4. Add configuration change tracking and rollback capabilities

## Advanced Features Requirements

### Configuration Preview System
```rust
pub struct ConfigurationPreview {
    resolver: Arc<AgentResolver>,
    diff_engine: DiffEngine,
    impact_analyzer: ImpactAnalyzer,
}

impl ConfigurationPreview {
    pub async fn preview_changes(
        &self,
        agent_name: &str,
        proposed_changes: ProposedChanges
    ) -> Result<PreviewResult> {
        // Show exactly what will change and why
        // Include impact assessment and warnings
        // Validate changes before application
    }
}
```

### Comprehensive Audit Trail
```rust
pub struct AuditLogger {
    event_store: EventStore,
    correlation_tracker: CorrelationTracker,
}

pub struct ConfigurationAuditEvent {
    id: Uuid,
    correlation_id: String,
    timestamp: DateTime<Utc>,
    agent_name: String,
    actor: Actor,  // Who made the change
    event_type: AuditEventType,
    details: AuditDetails,  // What changed and why
    impact: ImpactAssessment,  // What this affects
}
```

### Performance and Caching
```rust
pub struct ConfigurationCache {
    redis_client: Redis,
    compression: CompressionEngine,  // Reduce cache size
    ttl: Duration,                   // 5-minute default
    warming_strategy: WarmingStrategy,
}

// Performance targets:
// - Configuration resolution: <200ms
// - Cache hit ratio: >85%
// - Memory usage: <50MB for 1000 agents
// - Concurrent resolution: 100+ agents simultaneously
```

## Critical Implementation Details

### Helm Configuration Loading
```rust
impl HelmConfigLoader {
    pub async fn load_for_agent(&self, agent_name: &str) -> Result<HelmAgentConfig> {
        // Load from Kubernetes ConfigMap (agent.agentCliConfigs)
        // Handle Helm value hierarchy and templating
        // Cache frequently accessed configurations
        // Provide detailed error messages for missing configs
    }
}
```

### Repository Configuration Loading
```rust
impl CtoConfigLoader {
    pub async fn load_for_agent(&self, agent_name: &str) -> Result<CtoAgentConfig> {
        // Clone/pull repository based on GitHub app mapping
        // Parse cto-config.json with schema validation
        // Handle missing or malformed configuration files
        // Cache repository content with git commit tracking
    }
}
```

### Field-Level Merge Control
```rust
// Support per-field merge strategies
{
  "agents": {
    "rex": {
      "cli": "codex",  // Replace strategy
      "cliConfig": {    // Deep merge strategy
        "model": "gpt-4",  // Replace
        "tools": {         // Merge objects
          "remote": ["new_tool"],  // Append arrays
        }
      }
    }
  }
}
```

## Quality and Performance Requirements

### Error Handling Excellence
- Comprehensive error types with actionable messages
- Configuration path tracking in error messages
- Suggestion generation for common configuration mistakes
- Graceful degradation when sources are unavailable

### Performance Specifications
- Configuration resolution: <200ms for complex merges
- Cache hit ratio: >85% for repeated agent lookups
- Memory usage: <50MB for 1000 active agent configurations
- Concurrent processing: Support 100+ simultaneous resolutions

### Security and Compliance
- Secure credential handling in configuration sources
- Audit logging for all configuration access and changes
- Configuration validation to prevent injection attacks
- Role-based access control integration

## Testing Strategy

### Comprehensive Test Coverage
```rust
#[tokio::test]
async fn test_complex_merge_scenarios() {
    // Test conflicting configurations from multiple sources
    // Verify precedence rules are correctly applied
    // Test deep merge with nested object structures
    // Validate array merge strategies (append, replace, merge)
}

#[tokio::test] 
async fn test_performance_under_load() {
    // Test 100+ concurrent configuration resolutions
    // Verify cache effectiveness and hit ratios
    // Test memory usage stability over time
    // Validate response time SLA compliance
}

#[tokio::test]
async fn test_audit_trail_completeness() {
    // Verify all configuration changes are logged
    // Test correlation ID tracking across operations
    // Validate audit event completeness and accuracy
}
```

## Success Criteria
Your implementation succeeds when:
- ✅ Multi-source configuration loading with proper error handling
- ✅ Intelligent merge strategies handle all conflict scenarios
- ✅ JSON Schema validation provides helpful error messages
- ✅ Redis caching achieves >85% hit ratio with <200ms resolution time
- ✅ Configuration preview API shows changes before application
- ✅ Comprehensive audit trail tracks all configuration decisions
- ✅ Backward compatibility maintained for existing agent configurations
- ✅ Performance targets met under concurrent load (100+ agents)
- ✅ Security requirements satisfied with credential protection

## Constraints and Considerations
- Handle network failures gracefully when loading configurations
- Support configuration hot-reloading without service restarts
- Design for horizontal scaling with multiple resolver instances
- Consider configuration size limits and compression strategies
- Plan for future configuration source additions (databases, external APIs)
- Ensure thread safety for concurrent configuration resolution
- Design audit system for compliance with data retention policies

## Deliverables
1. Complete AgentResolver with multi-source configuration loading
2. Sophisticated merge engine with configurable strategies
3. JSON Schema validation with detailed error reporting
4. Redis-based configuration cache with compression and TTL
5. Configuration preview API with change impact assessment
6. Comprehensive audit logging system with correlation tracking
7. Backward compatibility layer for existing configurations
8. Performance monitoring and metrics collection
9. Complete test suite with load testing and edge case coverage
10. Documentation and examples for configuration best practices

This configuration resolution engine is the intelligence layer that makes the Multi-CLI Agent Platform both powerful and manageable. Focus on clarity, performance, and comprehensive audit capabilities.