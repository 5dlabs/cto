# Autonomous Agent Prompt: Implement Agent-Specific PVC Naming

## Mission

You are tasked with modifying the Rust controller to implement agent-specific PVC naming that enables proper workspace isolation for multi-agent workflows. Your mission is to extract agent names from GitHub App identifiers and create dedicated persistent workspaces for Rex, Cleo, and Tess agents.

## Context

The current controller creates PVCs with a generic `workspace-{service}` naming pattern. The multi-agent orchestration system requires isolated workspaces where each agent maintains separate persistent contexts, enabling independent cancellation, session continuity, and knowledge accumulation without interference.

## Objectives

1. **Implement Agent Name Extraction**
   - Create robust parsing logic for GitHub App field (`5DLabs-Rex` → `rex`)
   - Handle various naming patterns including bot suffixes
   - Implement proper error handling and validation
   - Support current and future agent naming conventions

2. **Modify PVC Creation Logic**
   - Update PVC naming to `workspace-{service}-{agent}` pattern
   - Ensure Kubernetes naming constraint compliance
   - Implement idempotent PVC creation using kube-rs
   - Add proper labeling for agent workspace identification

3. **Maintain Backward Compatibility**
   - Handle existing legacy PVC names during transition
   - Implement migration strategy for existing workspaces
   - Ensure existing workflows continue functioning
   - Provide graceful fallback mechanisms

4. **Update Controller Integration**
   - Modify reconciliation logic to use agent-specific naming
   - Update pod creation to mount correct agent workspaces
   - Implement comprehensive error handling
   - Add proper logging and monitoring

## Technical Requirements

### Agent Name Extraction Function
```rust
use regex::Regex;

fn extract_agent_name(github_app: &str) -> Result<String, String> {
    // Handle patterns like:
    // '5DLabs-Rex' -> 'rex'
    // '5DLabs-Cleo[bot]' -> 'cleo' 
    // '5DLabs-Tess' -> 'tess'
    
    let re = Regex::new(r"(?i)5dlabs[_-]?(\w+)(?:\[bot\])?").unwrap();
    
    if let Some(caps) = re.captures(github_app) {
        let agent_name = caps.get(1).unwrap().as_str().to_lowercase();
        
        // Validate Kubernetes naming constraints
        if validate_k8s_name(&agent_name)? {
            Ok(agent_name)
        } else {
            Err(format!("Invalid agent name for Kubernetes: {}", agent_name))
        }
    } else {
        Err(format!("Cannot extract agent name from: {}", github_app))
    }
}
```

### PVC Management Logic
```rust
use kube::api::{Api, PostParams};
use k8s_openapi::api::core::v1::PersistentVolumeClaim;

async fn ensure_agent_pvc(
    code_run: &CodeRun,
    client: &kube::Client,
) -> Result<String, kube::Error> {
    let agent_name = extract_agent_name(&code_run.spec.github_app)
        .map_err(|e| kube::Error::Api(ErrorResponse::default()))?;
        
    let pvc_name = format!(
        "workspace-{}-{}",
        code_run.spec.service,
        agent_name
    );
    
    let namespace = code_run.metadata.namespace.as_ref().unwrap();
    let pvc_api: Api<PersistentVolumeClaim> = Api::namespaced(client.clone(), namespace);
    
    // Check if PVC exists, create if missing
    match pvc_api.get(&pvc_name).await {
        Ok(_) => Ok(pvc_name),
        Err(kube::Error::Api(e)) if e.code == 404 => {
            let pvc_spec = create_agent_pvc_spec(&pvc_name, &code_run.spec.service, &agent_name);
            pvc_api.create(&PostParams::default(), &pvc_spec).await?;
            Ok(pvc_name)
        }
        Err(e) => Err(e),
    }
}
```

### Controller Integration
```rust
// In reconcile function
async fn reconcile(
    code_run: Arc<CodeRun>,
    ctx: Arc<Context>,
) -> Result<Action, Error> {
    let client = &ctx.client;
    
    // Extract agent name early for consistent usage
    let agent_name = match extract_agent_name(&code_run.spec.github_app) {
        Ok(name) => name,
        Err(e) => {
            error!("Failed to extract agent name: {}", e);
            update_status_with_error(&code_run, &e, client).await?;
            return Ok(Action::requeue(Duration::from_secs(60)));
        }
    };
    
    // Ensure agent-specific PVC exists
    let pvc_name = ensure_agent_pvc(&code_run, client).await
        .map_err(|e| Error::PvcCreation(e.to_string()))?;
    
    // Create pod with agent workspace
    create_agent_pod(&code_run, &pvc_name, &agent_name, client).await?;
    
    Ok(Action::requeue(Duration::from_secs(30)))
}
```

## Implementation Strategy

### Phase 1: Foundation Development
1. Create agent name extraction function with comprehensive testing
2. Implement Kubernetes naming validation
3. Build error handling and logging infrastructure
4. Create unit tests for extraction logic

### Phase 2: PVC Logic Implementation
1. Modify PVC creation functions to use agent-specific naming
2. Implement idempotent PVC creation with proper error handling
3. Add PVC labeling for agent workspace identification
4. Create integration tests for PVC management

### Phase 3: Controller Integration
1. Update reconciliation logic to use new PVC naming
2. Modify pod creation to mount agent-specific workspaces
3. Implement comprehensive error handling and recovery
4. Add monitoring and observability features

### Phase 4: Migration and Compatibility
1. Implement migration strategy for existing workspaces
2. Add backward compatibility checks and fallbacks
3. Create validation scripts for existing workflows
4. Document upgrade procedures for operators

## Code Implementation Details

### Agent Name Parsing Patterns
```rust
use std::collections::HashMap;

pub struct AgentNameExtractor {
    known_patterns: HashMap<String, String>,
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
            known_patterns: patterns,
            fallback_regex,
        }
    }
    
    pub fn extract(&self, github_app: &str) -> Result<String, String> {
        // Try exact match first for performance
        if let Some(agent) = self.known_patterns.get(github_app) {
            return Ok(agent.clone());
        }
        
        // Use regex for unknown patterns
        if let Some(caps) = self.fallback_regex.captures(github_app) {
            let agent_name = caps.get(1).unwrap().as_str().to_lowercase();
            self.validate_agent_name(&agent_name)?;
            return Ok(agent_name);
        }
        
        Err(format!("Cannot extract agent name from: {}", github_app))
    }
    
    fn validate_agent_name(&self, name: &str) -> Result<(), String> {
        if name.len() > 63 {
            return Err("Agent name exceeds Kubernetes limit (63 chars)".to_string());
        }
        
        if !name.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return Err("Agent name contains invalid characters".to_string());
        }
        
        if name.starts_with('-') || name.ends_with('-') {
            return Err("Agent name cannot start/end with hyphen".to_string());
        }
        
        Ok(())
    }
}
```

### PVC Specification Builder
```rust
use k8s_openapi::api::core::v1::{
    PersistentVolumeClaim, PersistentVolumeClaimSpec, ResourceRequirements,
};
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use std::collections::BTreeMap;

fn create_agent_pvc_spec(
    pvc_name: &str,
    service: &str,
    agent: &str,
) -> PersistentVolumeClaim {
    let mut labels = BTreeMap::new();
    labels.insert("app.kubernetes.io/name".to_string(), "agent-workspace".to_string());
    labels.insert("app.kubernetes.io/component".to_string(), "storage".to_string());
    labels.insert("service".to_string(), service.to_string());
    labels.insert("agent".to_string(), agent.to_string());
    
    let mut annotations = BTreeMap::new();
    annotations.insert(
        "description".to_string(),
        format!("Persistent workspace for {} agent on {} service", agent, service),
    );
    
    let mut resource_requests = BTreeMap::new();
    resource_requests.insert("storage".to_string(), Quantity("10Gi".to_string()));
    
    PersistentVolumeClaim {
        metadata: ObjectMeta {
            name: Some(pvc_name.to_string()),
            labels: Some(labels),
            annotations: Some(annotations),
            ..Default::default()
        },
        spec: Some(PersistentVolumeClaimSpec {
            access_modes: Some(vec!["ReadWriteOnce".to_string()]),
            resources: Some(ResourceRequirements {
                requests: Some(resource_requests),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}
```

## Testing and Validation Requirements

### Unit Testing
1. **Agent Name Extraction**: Test all supported GitHub App patterns
2. **Validation Logic**: Test Kubernetes naming constraint compliance
3. **Error Handling**: Verify appropriate error messages for invalid inputs
4. **Performance**: Benchmark extraction speed with large datasets

### Integration Testing
1. **PVC Creation**: Test with real Kubernetes cluster
2. **Controller Integration**: Verify reconciliation with new logic
3. **Multi-Agent Scenarios**: Test concurrent agent PVC creation
4. **Migration Testing**: Validate backward compatibility scenarios

### End-to-End Testing
1. **Workflow Validation**: Test complete multi-agent workflows
2. **Workspace Isolation**: Verify agents cannot access other workspaces
3. **Session Continuity**: Confirm agents resume previous sessions correctly
4. **Performance Impact**: Measure controller performance changes

## Success Criteria

- Agent name extraction works for all current and expected GitHub App patterns
- PVCs created with correct `workspace-{service}-{agent}` naming
- Existing workflows continue functioning during transition period
- Agent workspaces are properly isolated and independent
- Controller performance remains within acceptable bounds
- Comprehensive error handling provides clear troubleshooting information
- Migration path documented for existing deployments

## Key Implementation Notes

### Error Handling Strategy
- **Graceful Degradation**: Fall back to legacy behavior when extraction fails
- **Clear Error Messages**: Provide specific troubleshooting information
- **Retry Logic**: Implement appropriate backoff for transient failures
- **Observability**: Add metrics and logging for monitoring

### Performance Considerations
- **Efficient Extraction**: Use exact matches before regex parsing
- **Caching**: Consider caching extraction results for repeated operations
- **Minimal Allocations**: Optimize string handling for performance
- **Concurrent Operations**: Ensure thread safety for parallel requests

### Operational Requirements
- **Monitoring**: Add metrics for PVC creation success/failure rates
- **Logging**: Comprehensive structured logging for troubleshooting
- **Documentation**: Clear upgrade procedures for existing deployments
- **Validation**: Tools to verify migration success and workspace isolation

Begin implementation with the agent name extraction logic, ensuring robust testing before proceeding to PVC modification and controller integration.