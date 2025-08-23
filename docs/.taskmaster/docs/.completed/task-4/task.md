# Task 4: Implement Agent-Specific PVC Naming (Updated)

## Overview

Modify the Rust controller to extract agent names from the `github_app` field and implement conditional agent-specific PVC naming. **Implementation agents (Rex, Blaze) should continue using the shared `workspace-{service}` pattern**, while other agent types may require separate workspaces for proper isolation.

## Technical Context

The current controller uses a generic `workspace-{service}` naming pattern for PVCs. This pattern is **correct for implementation agents** who work on the same workspace surface. However, the system needs to support future agent types that may require isolated workspaces for independent operation, session continuity, and knowledge accumulation.

## Implementation Guide

### Phase 1: Agent Name Extraction Logic

1. **Create Agent Name Parser**
   ```rust
   // In controller/src/tasks/code/resources.rs
   fn extract_agent_name(github_app: &str) -> String {
       // Handle various GitHub App naming patterns
       // '5DLabs-Rex' -> 'rex'
       // '5DLabs-Cleo[bot]' -> 'cleo'
       // '5DLabs-Tess' -> 'tess'

       let app_name = github_app
           .split('-')
           .last()
           .unwrap_or(github_app)
           .split('[')
           .next()
           .unwrap_or(github_app);

       app_name.to_lowercase()
   }
   ```

2. **Implement Agent Type Classification**
   ```rust
   fn is_implementation_agent(agent_name: &str) -> bool {
       // Implementation agents work on the same workspace
       matches!(agent_name, "rex" | "blaze")
   }

   fn requires_isolated_workspace(agent_name: &str) -> bool {
       // Non-implementation agents may need isolated workspaces
       !is_implementation_agent(agent_name)
   }
   ```

### Phase 2: Conditional PVC Naming Logic

1. **Update PVC Name Generation**
   ```rust
   // In ensure_pvc_exists function
   async fn ensure_pvc_exists(&self, code_run: &CodeRun) -> Result<()> {
       let service_name = &code_run.spec.service;
       let github_app = code_run.spec.github_app.as_ref()
           .ok_or_else(|| Error::ConfigError("GitHub App is required".to_string()))?;

       let agent_name = extract_agent_name(github_app);

       // Implementation agents use shared workspace
       let pvc_name = if is_implementation_agent(&agent_name) {
           format!("workspace-{service_name}")
       } else {
           // Non-implementation agents get isolated workspaces
           format!("workspace-{service_name}-{agent_name}")
       };

       info!("📦 Ensuring PVC exists: {} (agent: {})", pvc_name, agent_name);
       self.ensure_pvc_exists(&pvc_name, service_name).await?;
       Ok(())
   }
   ```

2. **Maintain Backward Compatibility**
   ```rust
   // The existing workspace-{service} pattern remains the default
   // for implementation agents, ensuring no breaking changes
   ```

### Phase 3: Controller Integration

1. **Update Reconciliation Logic**
   ```rust
   // In reconcile_create_or_update function
   pub async fn reconcile_create_or_update(&self, code_run: &Arc<CodeRun>) -> Result<Action> {
       let name = code_run.name_any();
       info!("🚀 Creating/updating code resources for: {}", name);

       // Ensure PVC exists with conditional naming
       self.ensure_pvc_exists(code_run).await?;
       info!("✅ PVC check completed");

       // Continue with existing logic...
   }
   ```

2. **Update Volume Mounting**
   ```rust
   // In build_job_spec function
   fn build_job_spec(&self, code_run: &CodeRun, job_name: &str, cm_name: &str) -> Result<Job> {
       // ... existing volume setup ...

       // PVC workspace volume with conditional naming
       let service_name = &code_run.spec.service;
       let github_app = code_run.spec.github_app.as_ref()
           .ok_or_else(|| Error::ConfigError("GitHub App is required".to_string()))?;

       let agent_name = extract_agent_name(github_app);
       let pvc_name = if is_implementation_agent(&agent_name) {
           format!("workspace-{service_name}")
       } else {
           format!("workspace-{service_name}-{agent_name}")
       };

       volumes.push(json!({
           "name": "workspace",
           "persistentVolumeClaim": {
               "claimName": pvc_name
           }
       }));

       // ... rest of job spec ...
   }
   ```

## Code Examples

### Complete Agent Classification System
```rust
use regex::Regex;
use std::collections::HashSet;

pub struct AgentClassifier {
    implementation_agents: HashSet<String>,
    fallback_regex: Regex,
}

impl AgentClassifier {
    pub fn new() -> Self {
        let mut implementation_agents = HashSet::new();
        implementation_agents.insert("rex".to_string());
        implementation_agents.insert("blaze".to_string());

        let fallback_regex = Regex::new(r"(?i)5dlabs[_-]?(\w+)(?:\[bot\])?").unwrap();

        Self {
            implementation_agents,
            fallback_regex,
        }
    }

    pub fn extract_agent_name(&self, github_app: &str) -> Result<String, String> {
        if let Some(caps) = self.fallback_regex.captures(github_app) {
            Ok(caps.get(1).unwrap().as_str().to_lowercase())
        } else {
            Err(format!("Cannot extract agent name from: {}", github_app))
        }
    }

    pub fn is_implementation_agent(&self, agent_name: &str) -> bool {
        self.implementation_agents.contains(agent_name)
    }

    pub fn requires_isolated_workspace(&self, agent_name: &str) -> bool {
        !self.is_implementation_agent(agent_name)
    }

    pub fn get_pvc_name(&self, service: &str, github_app: &str) -> Result<String, String> {
        let agent_name = self.extract_agent_name(github_app)?;

        if self.is_implementation_agent(&agent_name) {
            Ok(format!("workspace-{}", service))
        } else {
            Ok(format!("workspace-{}-{}", service, agent_name))
        }
    }
}
```

### PVC Management with Conditional Logic
```rust
use kube::api::{Api, PostParams};
use k8s_openapi::api::core::v1::PersistentVolumeClaim;

pub async fn ensure_conditional_pvc(
    code_run: &CodeRun,
    client: &kube::Client,
) -> Result<String, kube::Error> {
    let classifier = AgentClassifier::new();
    let pvc_name = classifier.get_pvc_name(&code_run.spec.service, &code_run.spec.github_app)
        .map_err(|e| kube::Error::Api(ErrorResponse::default()))?;

    let namespace = code_run.metadata.namespace.as_ref()
        .ok_or_else(|| kube::Error::Api(ErrorResponse::default()))?;

    let pvc_api: Api<PersistentVolumeClaim> = Api::namespaced(client.clone(), namespace);

    // Check if PVC exists, create if missing
    match pvc_api.get(&pvc_name).await {
        Ok(_) => {
            info!("Agent PVC exists: {}", pvc_name);
            Ok(pvc_name)
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            // Create new PVC
            let pvc_spec = create_pvc_spec(&pvc_name, &code_run.spec.service);
            pvc_api.create(&PostParams::default(), &pvc_spec).await?;
            info!("Created agent PVC: {}", pvc_name);
            Ok(pvc_name)
        }
        Err(e) => Err(e),
    }
}
```

## Architecture Patterns

### Conditional Workspace Strategy
The new PVC naming strategy ensures:
1. **Implementation Agents**: Continue using shared `workspace-{service}` (Rex, Blaze)
2. **Future Agents**: Get isolated `workspace-{service}-{agent}` workspaces
3. **Backward Compatibility**: No changes to existing implementation agent behavior
4. **Extensibility**: Easy to add new agent types with different workspace requirements

### Migration Strategy
```rust
// No migration needed - implementation agents continue using existing pattern
// New agent types automatically get isolated workspaces
Workspace Naming Evolution:
Phase 1: Legacy "workspace-{service}" (existing - continues for implementation agents)
Phase 2: Conditional "workspace-{service}" or "workspace-{service}-{agent}" (new)
```

### Error Handling Patterns
- **Graceful Degradation**: Fall back to shared workspace if agent classification fails
- **Clear Error Messages**: Specific errors for debugging agent name issues
- **Validation**: Ensure agent names meet Kubernetes naming constraints
- **Logging**: Comprehensive logging for troubleshooting PVC issues

## Key Implementation Details

### Agent Name Validation
```rust
fn validate_agent_name(agent_name: &str) -> Result<(), String> {
    // Kubernetes naming constraints
    if agent_name.len() > 63 {
        return Err("Agent name too long for Kubernetes naming".to_string());
    }

    if !agent_name.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Err("Agent name contains invalid characters".to_string());
    }

    if agent_name.starts_with('-') || agent_name.ends_with('-') {
        return Err("Agent name cannot start or end with hyphen".to_string());
    }

    Ok(())
}
```

### PVC Creation with Proper Labels
```rust
fn create_pvc_spec(pvc_name: &str, service: &str, agent: Option<&str>) -> PersistentVolumeClaim {
    let mut labels = std::collections::BTreeMap::new();
    labels.insert("service".to_string(), service.to_string());
    labels.insert("component".to_string(), "agent-workspace".to_string());

    if let Some(agent_name) = agent {
        labels.insert("agent".to_string(), agent_name.to_string());
        labels.insert("workspace-type".to_string(), "isolated".to_string());
    } else {
        labels.insert("workspace-type".to_string(), "shared".to_string());
    }

    PersistentVolumeClaim {
        metadata: ObjectMeta {
            name: Some(pvc_name.to_string()),
            labels: Some(labels),
            ..Default::default()
        },
        spec: Some(PersistentVolumeClaimSpec {
            access_modes: Some(vec!["ReadWriteOnce".to_string()]),
            resources: Some(ResourceRequirements {
                requests: Some({
                    let mut requests = std::collections::BTreeMap::new();
                    requests.insert("storage".to_string(), Quantity("10Gi".to_string()));
                    requests
                }),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}
```

## Testing Strategy

### Unit Testing
1. **Agent Classification**: Test implementation vs non-implementation agent detection
2. **PVC Name Generation**: Verify correct naming for different agent types
3. **Error Handling**: Test with invalid inputs and edge cases
4. **Backward Compatibility**: Ensure implementation agents continue using shared workspace

### Integration Testing
1. **Controller Reconciliation**: Test with different agent types
2. **PVC Creation**: Verify PVCs created with correct naming
3. **Workspace Sharing**: Confirm implementation agents share workspace
4. **Workspace Isolation**: Confirm non-implementation agents get isolated workspaces

### Performance Testing
1. **Classification Speed**: Benchmark agent classification performance
2. **PVC Operations**: Test PVC creation and lookup efficiency
3. **Memory Usage**: Monitor controller memory with new logic
4. **Concurrent Operations**: Test with multiple simultaneous requests

## References

- [Kubernetes PVC API Documentation](https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.28/#persistentvolumeclaim-v1-core)
- [kube-rs PVC Examples](https://docs.rs/kube/latest/kube/)
- [Controller Architecture](.taskmaster/docs/architecture.md)
- [Multi-Agent Workspace Design](.taskmaster/docs/prd.txt)
