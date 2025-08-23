# Autonomous Agent Prompt: Implement Conditional Agent-Specific PVC Naming

## Mission

You are tasked with modifying the Rust controller to implement conditional agent-specific PVC naming that enables proper workspace management for multi-agent workflows. Your mission is to extract agent names from GitHub App identifiers and implement a classification system where **implementation agents (Rex, Blaze) continue using the shared workspace pattern**, while other agent types may require isolated workspaces.

## Context

The current controller creates PVCs with a generic `workspace-{service}` naming pattern. This pattern is **correct for implementation agents** who work on the same workspace surface. The multi-agent orchestration system needs to support future agent types that may require isolated workspaces for independent operation, session continuity, and knowledge accumulation without interference.

## Objectives

1. **Implement Agent Name Extraction**
   - Create robust parsing logic for GitHub App field (`5DLabs-Rex` → `rex`)
   - Handle various naming patterns including bot suffixes
   - Implement proper error handling and validation
   - Support current and future agent naming conventions

2. **Implement Agent Classification System**
   - Identify implementation agents (Rex, Blaze) that use shared workspace
   - Classify non-implementation agents that may need isolated workspaces
   - Create extensible system for future agent types
   - Maintain backward compatibility for existing implementation agents

3. **Modify PVC Creation Logic**
   - Update PVC naming to use conditional pattern based on agent type
   - Implementation agents: `workspace-{service}` (shared)
   - Non-implementation agents: `workspace-{service}-{agent}` (isolated)
   - Ensure Kubernetes naming constraint compliance
   - Implement idempotent PVC creation using kube-rs

4. **Maintain Backward Compatibility**
   - **CRITICAL**: Implementation agents continue using existing `workspace-{service}` pattern
   - No disruption to currently running CodeRun instances
   - Existing PVCs remain accessible to implementation agents
   - Provide graceful fallback mechanisms

5. **Update Controller Integration**
   - Modify reconciliation logic to use conditional PVC naming
   - Update pod creation to mount correct workspaces based on agent type
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

### Agent Classification System
```rust
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

### Conditional PVC Management Logic
```rust
use kube::api::{Api, PostParams};
use k8s_openapi::api::core::v1::PersistentVolumeClaim;

async fn ensure_conditional_pvc(
    code_run: &CodeRun,
    client: &kube::Client,
) -> Result<String, kube::Error> {
    let classifier = AgentClassifier::new();
    let pvc_name = classifier.get_pvc_name(&code_run.spec.service, &code_run.spec.github_app)
        .map_err(|e| kube::Error::Api(ErrorResponse::default()))?;

    let namespace = code_run.metadata.namespace.as_ref().unwrap();
    let pvc_api: Api<PersistentVolumeClaim> = Api::namespaced(client.clone(), namespace);

    // Check if PVC exists, create if missing
    match pvc_api.get(&pvc_name).await {
        Ok(_) => {
            info!("Agent PVC exists: {}", pvc_name);
            Ok(pvc_name)
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            let pvc_spec = create_agent_pvc_spec(&pvc_name, &code_run.spec.service, &agent_name);
            pvc_api.create(&PostParams::default(), &pvc_spec).await?;
            info!("Created agent PVC: {}", pvc_name);
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

    // Ensure conditional PVC exists
    let pvc_name = ensure_conditional_pvc(&code_run, client).await
        .map_err(|e| Error::PvcCreation(e.to_string()))?;

    // Create pod with appropriate workspace
    create_agent_pod(&code_run, &pvc_name, &agent_name, client).await?;

    Ok(Action::requeue(Duration::from_secs(30)))
}
```

## Implementation Strategy

### Phase 1: Foundation Development
1. Create agent name extraction function with comprehensive testing
2. Implement agent classification system with implementation agent identification
3. Build error handling and logging infrastructure
4. Create unit tests for extraction and classification logic

### Phase 2: Conditional PVC Logic Implementation
1. Modify PVC creation functions to use conditional naming based on agent type
2. Implement idempotent PVC creation with proper error handling
3. Add PVC labeling for workspace type identification (shared vs isolated)
4. Create integration tests for conditional PVC management

### Phase 3: Controller Integration
1. Update reconciliation logic to use conditional PVC naming
2. Modify pod creation to mount appropriate workspaces based on agent type
3. Implement comprehensive error handling and recovery
4. Add monitoring and observability features

### Phase 4: Backward Compatibility and Testing
1. Ensure implementation agents continue using existing workspace pattern
2. Add validation scripts for existing workflows
3. Create comprehensive test suite for all agent types
4. Document upgrade procedures for operators

## Code Implementation Details

### Agent Classification Patterns
```rust
use std::collections::HashSet;

pub struct AgentClassifier {
    known_implementation_agents: HashSet<String>,
    fallback_regex: Regex,
}

impl AgentClassifier {
    pub fn new() -> Self {
        let mut implementation_agents = HashSet::new();
        implementation_agents.insert("rex".to_string());
        implementation_agents.insert("blaze".to_string());

        let fallback_regex = Regex::new(r"(?i)5dlabs[_-]?(\w+)(?:\[bot\])?").unwrap();

        Self {
            known_implementation_agents: implementation_agents,
            fallback_regex,
        }
    }

    pub fn extract_agent_name(&self, github_app: &str) -> Result<String, String> {
        if let Some(caps) = self.fallback_regex.captures(github_app) {
            let agent_name = caps.get(1).unwrap().as_str().to_lowercase();
            self.validate_agent_name(&agent_name)?;
            Ok(agent_name)
        } else {
            Err(format!("Cannot extract agent name from: {}", github_app))
        }
    }

    pub fn is_implementation_agent(&self, agent_name: &str) -> bool {
        self.known_implementation_agents.contains(agent_name)
    }

    pub fn requires_isolated_workspace(&self, agent_name: &str) -> bool {
        !self.is_implementation_agent(agent_name)
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

fn create_conditional_pvc_spec(
    pvc_name: &str,
    service: &str,
    agent_name: Option<&str>,
) -> PersistentVolumeClaim {
    let mut labels = BTreeMap::new();
    labels.insert("app.kubernetes.io/name".to_string(), "agent-workspace".to_string());
    labels.insert("app.kubernetes.io/component".to_string(), "storage".to_string());
    labels.insert("service".to_string(), service.to_string());

    if let Some(agent) = agent_name {
        labels.insert("agent".to_string(), agent.to_string());
        labels.insert("workspace-type".to_string(), "isolated".to_string());
    } else {
        labels.insert("workspace-type".to_string(), "shared".to_string());
    }

    let mut annotations = BTreeMap::new();
    if let Some(agent) = agent_name {
        annotations.insert(
            "description".to_string(),
            format!("Isolated workspace for {} agent on {} service", agent, service),
        );
    } else {
        annotations.insert(
            "description".to_string(),
            format!("Shared workspace for implementation agents on {} service", service),
        );
    }

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
2. **Agent Classification**: Test implementation vs non-implementation agent detection
3. **Validation Logic**: Test Kubernetes naming constraint compliance
4. **Error Handling**: Verify appropriate error messages for invalid inputs
5. **Performance**: Benchmark extraction and classification speed

### Integration Testing
1. **PVC Creation**: Test with real Kubernetes cluster
2. **Controller Integration**: Verify reconciliation with conditional logic
3. **Multi-Agent Scenarios**: Test concurrent agent PVC creation
4. **Backward Compatibility**: Validate implementation agents continue using shared workspace

### End-to-End Testing
1. **Workflow Validation**: Test complete multi-agent workflows
2. **Workspace Strategy**: Verify correct workspace assignment based on agent type
3. **Session Continuity**: Confirm agents resume previous sessions correctly
4. **Performance Impact**: Measure controller performance changes

## Success Criteria

- Agent name extraction works for all current and expected GitHub App patterns
- Implementation agents (Rex, Blaze) continue using shared `workspace-{service}` pattern
- Non-implementation agents get isolated `workspace-{service}-{agent}` workspaces
- Existing workflows continue functioning without disruption
- Agent workspaces are properly managed based on classification
- Controller performance remains within acceptable bounds
- Comprehensive error handling provides clear troubleshooting information
- No migration required for existing deployments

## Key Implementation Notes

### Error Handling Strategy
- **Graceful Degradation**: Fall back to shared workspace if classification fails
- **Clear Error Messages**: Provide specific troubleshooting information
- **Retry Logic**: Implement appropriate backoff for transient failures
- **Observability**: Add metrics and logging for monitoring

### Performance Considerations
- **Efficient Classification**: Use HashSet for fast implementation agent lookup
- **Caching**: Consider caching classification results for repeated operations
- **Minimal Allocations**: Optimize string handling for performance
- **Concurrent Operations**: Ensure thread safety for parallel requests

### Operational Requirements
- **Monitoring**: Add metrics for PVC creation success/failure rates
- **Logging**: Comprehensive structured logging for troubleshooting
- **Documentation**: Clear upgrade procedures for existing deployments
- **Validation**: Tools to verify classification success and workspace assignment

### Backward Compatibility
- **CRITICAL**: Implementation agents must continue using existing workspace pattern
- **No Migration**: Existing PVCs remain accessible to implementation agents
- **Zero Disruption**: Current workflows continue functioning without changes
- **Future Extensibility**: System designed to accommodate new agent types

Begin implementation with the agent name extraction and classification logic, ensuring robust testing before proceeding to PVC modification and controller integration.
