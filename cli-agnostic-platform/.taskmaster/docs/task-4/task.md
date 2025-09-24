# Task 4: Implement Configuration Resolution Engine

## Overview
Build the multi-source configuration merger that handles Helm defaults, cto-config.json overrides, and runtime parameters with field-level granularity. This system provides the intelligent configuration resolution that enables agents to have cluster-wide defaults while supporting repository-specific customizations.

## Context
The Multi-CLI Agent Platform must support complex configuration scenarios where operators set cluster-wide defaults through Helm charts, while individual repositories can override specific settings in cto-config.json. The system must resolve these multiple configuration sources with clear precedence rules and provide transparency into how final configurations are derived.

## Technical Specification

### 1. Configuration Sources and Hierarchy
```
Priority (High → Low):
1. Runtime Parameters (immediate execution overrides)
2. Environment Variables (deployment-specific)
3. CtoConfig (repository-level cto-config.json)
4. Helm Values (cluster-level agent.agentCliConfigs)
5. Built-in Defaults (fallback values)
```

### 2. AgentResolver Architecture
```rust
pub struct AgentResolver {
    helm_loader: HelmConfigLoader,
    cto_loader: CtoConfigLoader,
    env_loader: EnvironmentLoader,
    cache: Arc<ConfigurationCache>,
    validator: ConfigurationValidator,
    audit_logger: AuditLogger,
    merge_engine: MergeEngine,
}

impl AgentResolver {
    pub async fn resolve_agent_config(&self, agent_name: &str, runtime_params: Option<RuntimeParameters>) -> Result<ResolvedAgentConfig> {
        // Load from all sources
        let helm_config = self.helm_loader.load_for_agent(agent_name).await?;
        let cto_config = self.cto_loader.load_for_agent(agent_name).await?;
        let env_config = self.env_loader.load_for_agent(agent_name).await?;

        // Apply merge strategy
        let merged = self.merge_engine.merge_configurations(
            vec![helm_config, cto_config, env_config],
            runtime_params
        ).await?;

        // Validate final configuration
        self.validator.validate(&merged).await?;

        // Cache result
        self.cache.store(agent_name, &merged).await?;

        // Audit log
        self.audit_logger.log_resolution(agent_name, &merged).await?;

        Ok(merged)
    }
}
```

### 3. Multi-Source Configuration Loading

#### Helm Configuration Loader
```rust
pub struct HelmConfigLoader {
    k8s_client: Client,
    config_map_name: String,
    namespace: String,
}

impl HelmConfigLoader {
    pub async fn load_for_agent(&self, agent_name: &str) -> Result<HelmAgentConfig> {
        let config_map: ConfigMap = self.k8s_client
            .get(&self.config_map_name, &self.namespace)
            .await?;

        let helm_values = config_map.data
            .get("values.yaml")
            .ok_or_else(|| anyhow!("Helm values not found"))?;

        let parsed: HelmValues = serde_yaml::from_str(helm_values)?;

        // Extract agent-specific configuration
        parsed.agent
            .agent_cli_configs
            .get(agent_name)
            .cloned()
            .ok_or_else(|| anyhow!("Agent configuration not found: {}", agent_name))
    }
}
```

#### CtoConfig Loader
```rust
pub struct CtoConfigLoader {
    git_client: GitClient,
    repo_cache: RepoCache,
}

impl CtoConfigLoader {
    pub async fn load_for_agent(&self, agent_name: &str) -> Result<CtoAgentConfig> {
        // Get repository for agent (from GitHub app mapping)
        let repo_info = self.get_repo_for_agent(agent_name).await?;

        // Clone or pull repository
        let repo_path = self.git_client.ensure_repo(&repo_info).await?;

        // Load cto-config.json
        let config_path = repo_path.join("cto-config.json");
        if !config_path.exists() {
            return Ok(CtoAgentConfig::empty());
        }

        let config_content = tokio::fs::read_to_string(config_path).await?;
        let cto_config: CtoConfig = serde_json::from_str(&config_content)?;

        // Extract agent-specific configuration
        cto_config.agents
            .get(agent_name)
            .cloned()
            .unwrap_or_default()
    }
}
```

### 4. Advanced Merge Strategies
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum MergeStrategy {
    Replace,    // Completely replace the value
    Merge,      // Merge objects, replace primitives
    Append,     // Append arrays, merge objects
    DeepMerge,  // Recursive merge with conflict resolution
}

pub struct MergeEngine {
    strategies: HashMap<String, MergeStrategy>,
    conflict_resolver: ConflictResolver,
}

impl MergeEngine {
    pub async fn merge_configurations(&self, configs: Vec<ConfigurationSource>) -> Result<MergedConfiguration> {
        let mut result = serde_json::Value::Object(serde_json::Map::new());

        // Sort by precedence (lowest to highest)
        let sorted_configs = self.sort_by_precedence(configs);

        for config_source in sorted_configs {
            result = self.merge_values(result, config_source.value, &config_source.strategy).await?;
        }

        Ok(MergedConfiguration::from_value(result)?)
    }

    async fn merge_values(&self, base: serde_json::Value, overlay: serde_json::Value, strategy: &MergeStrategy) -> Result<serde_json::Value> {
        match strategy {
            MergeStrategy::Replace => Ok(overlay),
            MergeStrategy::Merge => self.merge_objects(base, overlay).await,
            MergeStrategy::Append => self.append_arrays(base, overlay).await,
            MergeStrategy::DeepMerge => self.deep_merge(base, overlay).await,
        }
    }

    async fn deep_merge(&self, base: serde_json::Value, overlay: serde_json::Value) -> Result<serde_json::Value> {
        match (base, overlay) {
            (serde_json::Value::Object(mut base_obj), serde_json::Value::Object(overlay_obj)) => {
                for (key, overlay_value) in overlay_obj {
                    let merged_value = if let Some(base_value) = base_obj.remove(&key) {
                        self.deep_merge(base_value, overlay_value).await?
                    } else {
                        overlay_value
                    };
                    base_obj.insert(key, merged_value);
                }
                Ok(serde_json::Value::Object(base_obj))
            },
            (serde_json::Value::Array(mut base_arr), serde_json::Value::Array(overlay_arr)) => {
                base_arr.extend(overlay_arr);
                Ok(serde_json::Value::Array(base_arr))
            },
            (_, overlay) => Ok(overlay), // Replace for non-container types
        }
    }
}
```

### 5. JSON Schema Validation
```rust
pub struct ConfigurationValidator {
    schemas: HashMap<String, JSONSchema>,
    validator_cache: Arc<Mutex<LruCache<String, ValidationResult>>>,
}

impl ConfigurationValidator {
    pub async fn validate(&self, config: &MergedConfiguration) -> Result<ValidationReport> {
        let schema_key = format!("agent_config_v{}", config.schema_version);
        let schema = self.schemas
            .get(&schema_key)
            .ok_or_else(|| anyhow!("Schema not found: {}", schema_key))?;

        let validation_result = schema.validate(&config.as_json_value());

        match validation_result {
            Ok(_) => Ok(ValidationReport::valid()),
            Err(errors) => {
                let detailed_errors: Vec<ValidationError> = errors
                    .map(|error| ValidationError {
                        path: error.instance_path.to_string(),
                        message: error.to_string(),
                        suggestion: self.generate_suggestion(&error),
                    })
                    .collect();

                Ok(ValidationReport::invalid(detailed_errors))
            }
        }
    }

    fn generate_suggestion(&self, error: &jsonschema::ValidationError) -> Option<String> {
        // Generate helpful suggestions based on common configuration mistakes
        match error.keyword() {
            "required" => Some("Add the missing required field to your configuration".to_string()),
            "type" => Some(format!("Expected {}, but got {}", error.schema_value(), error.instance_value())),
            "enum" => Some(format!("Valid values are: {:?}", error.schema_value())),
            _ => None,
        }
    }
}
```

### 6. Configuration Caching with Redis
```rust
pub struct ConfigurationCache {
    redis_client: Redis,
    ttl: Duration,
    compression: CompressionEngine,
}

impl ConfigurationCache {
    pub async fn get(&self, cache_key: &str) -> Result<Option<CachedConfiguration>> {
        let compressed_data: Option<Vec<u8>> = self.redis_client
            .get(cache_key)
            .await?;

        if let Some(data) = compressed_data {
            let decompressed = self.compression.decompress(&data)?;
            let config: CachedConfiguration = serde_json::from_slice(&decompressed)?;

            // Check if still valid
            if config.expires_at > Utc::now() {
                Ok(Some(config))
            } else {
                // Expired, remove from cache
                self.redis_client.del(cache_key).await?;
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub async fn store(&self, cache_key: &str, config: &MergedConfiguration) -> Result<()> {
        let cached_config = CachedConfiguration {
            config: config.clone(),
            cached_at: Utc::now(),
            expires_at: Utc::now() + self.ttl,
            checksum: config.calculate_checksum(),
        };

        let serialized = serde_json::to_vec(&cached_config)?;
        let compressed = self.compression.compress(&serialized)?;

        self.redis_client
            .setex(cache_key, self.ttl.as_secs(), compressed)
            .await?;

        Ok(())
    }
}
```

### 7. Configuration Preview API
```rust
pub struct ConfigurationPreview {
    resolver: Arc<AgentResolver>,
}

impl ConfigurationPreview {
    pub async fn preview_configuration(&self, agent_name: &str, proposed_changes: ProposedChanges) -> Result<PreviewResult> {
        // Load current configuration
        let current_config = self.resolver.resolve_agent_config(agent_name, None).await?;

        // Apply proposed changes to create preview
        let preview_config = self.apply_proposed_changes(&current_config, proposed_changes).await?;

        // Show diff
        let diff = self.calculate_diff(&current_config, &preview_config).await?;

        // Validate preview configuration
        let validation = self.resolver.validator.validate(&preview_config).await?;

        Ok(PreviewResult {
            current: current_config,
            preview: preview_config,
            diff,
            validation,
            warnings: self.generate_warnings(&diff).await?,
        })
    }

    async fn calculate_diff(&self, current: &MergedConfiguration, preview: &MergedConfiguration) -> Result<ConfigurationDiff> {
        // Generate detailed diff showing what changed and why
        let current_json = current.as_json_value();
        let preview_json = preview.as_json_value();

        let changes = self.json_diff(&current_json, &preview_json);

        Ok(ConfigurationDiff {
            changes,
            impact_assessment: self.assess_impact(&changes).await?,
        })
    }
}
```

### 8. Comprehensive Audit Logging
```rust
pub struct AuditLogger {
    event_store: EventStore,
    correlation_id_generator: CorrelationIdGenerator,
}

impl AuditLogger {
    pub async fn log_resolution(&self, agent_name: &str, resolved_config: &MergedConfiguration) -> Result<()> {
        let audit_event = ConfigurationAuditEvent {
            id: Uuid::new_v4(),
            correlation_id: self.correlation_id_generator.generate(),
            timestamp: Utc::now(),
            event_type: AuditEventType::ConfigurationResolved,
            agent_name: agent_name.to_string(),
            actor: self.get_current_actor().await?,
            details: AuditDetails {
                resolved_config: resolved_config.clone(),
                sources_consulted: resolved_config.source_trace.clone(),
                merge_decisions: resolved_config.merge_trace.clone(),
                validation_results: resolved_config.validation_trace.clone(),
            },
            metadata: self.capture_metadata().await?,
        };

        self.event_store.store(audit_event).await?;
        Ok(())
    }

    pub async fn log_configuration_change(&self, agent_name: &str, before: &MergedConfiguration, after: &MergedConfiguration, reason: ChangeReason) -> Result<()> {
        let change_event = ConfigurationChangeEvent {
            id: Uuid::new_v4(),
            correlation_id: self.correlation_id_generator.generate(),
            timestamp: Utc::now(),
            event_type: AuditEventType::ConfigurationChanged,
            agent_name: agent_name.to_string(),
            actor: self.get_current_actor().await?,
            change_details: ConfigurationChange {
                before: before.clone(),
                after: after.clone(),
                diff: self.calculate_change_diff(before, after).await?,
                reason,
                impact_assessment: self.assess_change_impact(before, after).await?,
            },
        };

        self.event_store.store(change_event).await?;
        Ok(())
    }
}
```

## Implementation Steps

### Phase 1: Core Resolution Framework
1. Create AgentResolver structure with multi-source loading
2. Implement configuration source loaders (Helm, CtoConfig, Environment)
3. Design and implement merge engine with strategies
4. Add comprehensive error handling and logging

### Phase 2: Validation and Schema
1. Implement JSON Schema validation with detailed error messages
2. Create configuration validation pipeline
3. Add suggestion generation for common configuration errors
4. Build validation result reporting

### Phase 3: Caching and Performance
1. Implement Redis-based configuration cache
2. Add compression for cache storage efficiency
3. Build cache invalidation strategies
4. Add performance monitoring and metrics

### Phase 4: Preview and Audit
1. Create configuration preview API
2. Implement change diff calculation
3. Add comprehensive audit logging
4. Build configuration change tracking

### Phase 5: Integration and Testing
1. Integrate with CLI adapter system (Task 3)
2. Add backward compatibility handling
3. Implement comprehensive test suite
4. Add performance optimization

## Dependencies
- Task 1: Project structure for module organization
- Task 3: CLI adapter trait system for configuration application
- Kubernetes client for ConfigMap access
- Redis for configuration caching
- Git client for repository access
- JSON Schema validation library
- Audit logging infrastructure

## Success Criteria
- Multi-source configuration loading and merging
- Field-level merge strategies with conflict resolution
- JSON Schema validation with helpful error messages
- Redis caching with appropriate TTL
- Configuration preview API for change impact
- Comprehensive audit trail for all configuration changes
- Backward compatibility for existing agents
- Performance: <200ms for configuration resolution

## Files Created
```
controller/src/agents/
├── resolver.rs (main resolution engine)
├── config_loader/
│   ├── helm.rs (Helm ConfigMap loader)
│   ├── cto_config.rs (repository config loader)
│   ├── environment.rs (environment variable loader)
│   └── mod.rs
├── merge_engine.rs (configuration merging logic)
├── validation/
│   ├── validator.rs (JSON Schema validation)
│   ├── schemas/ (configuration schemas)
│   └── mod.rs
├── cache/
│   ├── redis_cache.rs (Redis caching implementation)
│   ├── compression.rs (cache compression)
│   └── mod.rs
├── preview.rs (configuration preview API)
└── audit/
    ├── logger.rs (audit event logging)
    ├── events.rs (audit event types)
    └── mod.rs

tests/
├── resolver_tests.rs
├── merge_engine_tests.rs
├── validation_tests.rs
└── integration/
    └── configuration_resolution_tests.rs
```

## Risk Mitigation

### Configuration Conflicts
- Clear precedence rules with documentation
- Conflict detection and resolution strategies
- Warning system for potentially problematic merges
- Preview system to validate changes before application

### Performance Impact
- Redis caching with compression
- Lazy loading of configuration sources
- Connection pooling for external services
- Metrics and monitoring for performance tracking

### Security and Compliance
- Audit logging for all configuration changes
- Secure credential handling in configurations
- Configuration validation to prevent injection attacks
- Role-based access control for configuration changes

## Testing Strategy
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_configuration_merge_precedence() {
        let resolver = AgentResolver::new().await.unwrap();

        // Test that runtime params override everything
        let runtime_params = RuntimeParameters {
            model: Some("runtime-override".to_string()),
        };

        let config = resolver.resolve_agent_config("test-agent", Some(runtime_params)).await.unwrap();
        assert_eq!(config.model, "runtime-override");
    }

    #[tokio::test]
    async fn test_deep_merge_strategy() {
        let merge_engine = MergeEngine::new();

        let base = json!({
            "tools": {
                "remote": ["tool1"],
                "local": {"filesystem": {"enabled": true}}
            }
        });

        let overlay = json!({
            "tools": {
                "remote": ["tool2"],
                "local": {"git": {"enabled": true}}
            }
        });

        let result = merge_engine.merge_values(base, overlay, &MergeStrategy::DeepMerge).await.unwrap();

        // Verify arrays were appended and objects were merged
        assert_eq!(result["tools"]["remote"].as_array().unwrap().len(), 2);
        assert!(result["tools"]["local"]["filesystem"]["enabled"].as_bool().unwrap());
        assert!(result["tools"]["local"]["git"]["enabled"].as_bool().unwrap());
    }
}
```

## Next Steps
After completion, this task enables:
- Task 5+: CLI-specific implementations can use resolved configurations
- Advanced configuration management features
- GitOps integration for configuration changes
- Multi-tenancy with tenant-specific configurations
- Configuration drift detection and remediation

This task provides the intelligent configuration resolution that makes the Multi-CLI Agent Platform flexible and maintainable.