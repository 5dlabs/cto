# Task 4: Implement Agent-Specific PVC Naming

## Overview

Modify the Rust controller to extract agent names from the `github_app` field and implement a `workspace-{service}-{agent}` PVC naming pattern for proper agent workspace isolation in the multi-agent orchestration system.

## Technical Context

The current controller uses a generic `workspace-{service}` naming pattern for PVCs. Multi-agent workflows require isolated workspaces where Rex, Cleo, and Tess agents maintain separate persistent contexts and can be cancelled/restarted independently without affecting other agents' accumulated knowledge.

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

2. **Implement Robust Pattern Matching**
   ```rust
   use regex::Regex;
   
   fn extract_agent_name(github_app: &str) -> Result<String, String> {
       let re = Regex::new(r"(?i)5dlabs[_-]?(\w+)(?:\[bot\])?").unwrap();
       
       if let Some(caps) = re.captures(github_app) {
           Ok(caps.get(1).unwrap().as_str().to_lowercase())
       } else {
           Err(format!("Cannot extract agent name from: {}", github_app))
       }
   }
   ```

### Phase 2: PVC Creation Logic Modification

1. **Update PVC Name Generation**
   ```rust
   // In create_pvc function
   async fn create_pvc(
       code_run: &CodeRun,
       client: &kube::Client,
   ) -> Result<PersistentVolumeClaim, kube::Error> {
       let agent_name = extract_agent_name(&code_run.spec.github_app)
           .map_err(|e| kube::Error::Api(kube::error::ErrorResponse {
               // Handle extraction error
           }))?;
           
       let pvc_name = format!(
           "workspace-{}-{}",
           code_run.spec.service,
           agent_name
       );
       
       // Create PVC with new naming pattern
       create_pvc_resource(&pvc_name, &code_run.metadata.namespace)
   }
   ```

2. **Implement Legacy PVC Compatibility**
   ```rust
   async fn ensure_pvc_exists(
       code_run: &CodeRun,
       client: &kube::Client,
   ) -> Result<String, kube::Error> {
       let agent_name = extract_agent_name(&code_run.spec.github_app)?;
       let new_pvc_name = format!(
           "workspace-{}-{}", 
           code_run.spec.service, 
           agent_name
       );
       let legacy_pvc_name = format!(
           "workspace-{}", 
           code_run.spec.service
       );
       
       // Try new naming first, fall back to legacy if needed
       match get_pvc(&new_pvc_name, client).await {
           Ok(pvc) => Ok(new_pvc_name),
           Err(_) => {
               // Check for legacy PVC
               match get_pvc(&legacy_pvc_name, client).await {
                   Ok(_) => {
                       // Migrate legacy PVC or create new one
                       create_pvc_with_name(&new_pvc_name, client).await?;
                       Ok(new_pvc_name)
                   }
                   Err(_) => {
                       // Create new PVC
                       create_pvc_with_name(&new_pvc_name, client).await?;
                       Ok(new_pvc_name)
                   }
               }
           }
       }
   }
   ```

### Phase 3: Controller Integration

1. **Update Reconciliation Logic**
   ```rust
   // In controller reconcile function
   async fn reconcile(
       code_run: Arc<CodeRun>,
       ctx: Arc<Context>,
   ) -> Result<Action, Error> {
       let client = &ctx.client;
       
       // Extract agent name early for consistent usage
       let agent_name = match extract_agent_name(&code_run.spec.github_app) {
           Ok(name) => name,
           Err(e) => {
               update_status_with_error(&code_run, &e, client).await?;
               return Ok(Action::requeue(Duration::from_secs(60)));
           }
       };
       
       // Create PVC with agent-specific naming
       let pvc_name = ensure_pvc_exists(&code_run, client).await?;
       
       // Continue with pod creation using new PVC name
       create_agent_pod(&code_run, &pvc_name, &agent_name, client).await?;
       
       Ok(Action::requeue(Duration::from_secs(30)))
   }
   ```

2. **Pod Volume Mount Configuration**
   ```rust
   fn create_pod_spec(
       code_run: &CodeRun,
       pvc_name: &str,
       agent_name: &str,
   ) -> Result<PodSpec, Error> {
       let volume_mount = VolumeMount {
           name: "workspace".to_string(),
           mount_path: "/workspace".to_string(),
           ..Default::default()
       };
       
       let volume = Volume {
           name: "workspace".to_string(),
           persistent_volume_claim: Some(PersistentVolumeClaimVolumeSource {
               claim_name: pvc_name.to_string(),
               ..Default::default()
           }),
           ..Default::default()
       };
       
       // Build pod spec with proper workspace mounting
       build_agent_pod_spec(code_run, agent_name, volume_mount, volume)
   }
   ```

## Code Examples

### Complete Agent Name Extraction
```rust
use regex::Regex;
use std::collections::HashMap;

pub struct AgentNameExtractor {
    patterns: HashMap<String, String>,
    fallback_regex: Regex,
}

impl AgentNameExtractor {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();
        patterns.insert("5DLabs-Rex".to_string(), "rex".to_string());
        patterns.insert("5DLabs-Blaze".to_string(), "blaze".to_string());
        patterns.insert("5DLabs-Cleo".to_string(), "cleo".to_string());
        patterns.insert("5DLabs-Tess".to_string(), "tess".to_string());
        
        let fallback_regex = Regex::new(r"(?i)5dlabs[_-]?(\w+)(?:\[bot\])?").unwrap();
        
        Self {
            patterns,
            fallback_regex,
        }
    }
    
    pub fn extract(&self, github_app: &str) -> Result<String, String> {
        // Try exact match first
        if let Some(agent) = self.patterns.get(github_app) {
            return Ok(agent.clone());
        }
        
        // Try regex extraction
        if let Some(caps) = self.fallback_regex.captures(github_app) {
            return Ok(caps.get(1).unwrap().as_str().to_lowercase());
        }
        
        Err(format!("Cannot extract agent name from: {}", github_app))
    }
}
```

### PVC Management with Migration
```rust
use kube::api::{Api, ListParams, PostParams};
use k8s_openapi::api::core::v1::PersistentVolumeClaim;

pub async fn manage_agent_pvc(
    code_run: &CodeRun,
    client: &kube::Client,
) -> Result<String, kube::Error> {
    let agent_name = extract_agent_name(&code_run.spec.github_app)
        .map_err(|e| kube::Error::Api(ErrorResponse::default()))?;
        
    let namespace = code_run.metadata.namespace.as_ref()
        .ok_or_else(|| kube::Error::Api(ErrorResponse::default()))?;
        
    let pvc_api: Api<PersistentVolumeClaim> = Api::namespaced(client.clone(), namespace);
    
    let new_pvc_name = format!("workspace-{}-{}", code_run.spec.service, agent_name);
    let legacy_pvc_name = format!("workspace-{}", code_run.spec.service);
    
    // Check if new PVC exists
    match pvc_api.get(&new_pvc_name).await {
        Ok(_) => {
            info!("Agent PVC exists: {}", new_pvc_name);
            Ok(new_pvc_name)
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            // Create new PVC
            let pvc_spec = create_pvc_spec(&new_pvc_name, &code_run.spec.service);
            pvc_api.create(&PostParams::default(), &pvc_spec).await?;
            info!("Created agent PVC: {}", new_pvc_name);
            Ok(new_pvc_name)
        }
        Err(e) => Err(e),
    }
}
```

## Architecture Patterns

### Agent Workspace Isolation
The new PVC naming pattern ensures:
1. **Independent Workspaces**: Each agent maintains separate persistent storage
2. **Clean Cancellation**: Running agents can be cancelled without affecting others
3. **Session Continuity**: Agents continue their own previous sessions
4. **Knowledge Accumulation**: Agents build expertise in their dedicated workspaces

### Migration Strategy
```rust
// Transition period logic
Workspace Naming Evolution:
Phase 1: Legacy "workspace-{service}" (existing)
Phase 2: Dual support - check new naming, fall back to legacy
Phase 3: Pure "workspace-{service}-{agent}" (target)
```

### Error Handling Patterns
- **Graceful Degradation**: Fall back to legacy naming if extraction fails
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
fn create_pvc_spec(pvc_name: &str, service: &str, agent: &str) -> PersistentVolumeClaim {
    let mut labels = std::collections::BTreeMap::new();
    labels.insert("service".to_string(), service.to_string());
    labels.insert("agent".to_string(), agent.to_string());
    labels.insert("component".to_string(), "agent-workspace".to_string());
    
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
1. **Agent Name Extraction**: Test with various GitHub App naming patterns
2. **PVC Name Generation**: Verify correct formatting and validation
3. **Error Handling**: Test with invalid inputs and edge cases
4. **Migration Logic**: Test legacy PVC handling and transitions

### Integration Testing
1. **Controller Reconciliation**: Test with different agent types
2. **PVC Creation**: Verify PVCs created with correct naming
3. **Workspace Isolation**: Confirm agents get separate workspaces
4. **Backward Compatibility**: Ensure existing workflows continue working

### Performance Testing
1. **Extraction Speed**: Benchmark agent name extraction performance
2. **PVC Operations**: Test PVC creation and lookup efficiency
3. **Memory Usage**: Monitor controller memory with new logic
4. **Concurrent Operations**: Test with multiple simultaneous requests

## References

- [Kubernetes PVC API Documentation](https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.28/#persistentvolumeclaim-v1-core)
- [kube-rs PVC Examples](https://docs.rs/kube/latest/kube/)
- [Controller Architecture](.taskmaster/docs/architecture.md)
- [Multi-Agent Workspace Design](.taskmaster/docs/prd.txt)