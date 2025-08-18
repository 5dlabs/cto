# Toolman Guide: Implement Agent-Specific PVC Naming

## Overview

This task requires Rust development capabilities combined with filesystem operations for code modification and testing. The selected tools focus on Rust code analysis, development, and validation for controller modification.

## Core Tools

### Filesystem Operations
The filesystem server provides essential tools for Rust development:

#### `read_file`
- **Purpose**: Read existing controller source files and understand current implementation
- **When to Use**: Examining `controller/src/tasks/code/resources.rs` and related files
- **Example Usage**: Read PVC creation logic and controller reconciliation patterns
- **Best Practice**: Start by understanding existing code structure before modifications

#### `write_file`
- **Purpose**: Implement agent name extraction logic and modify PVC creation functions
- **When to Use**: Adding new functions and updating existing controller code
- **Example Usage**: Create `extract_agent_name()` function and update PVC naming logic
- **Best Practice**: Make incremental changes and preserve existing functionality

#### `search_files`
- **Purpose**: Find relevant code patterns and PVC-related functions across codebase
- **When to Use**: Locating PVC creation logic and controller integration points
- **Example Usage**: Search for `PersistentVolumeClaim` usage and workspace patterns
- **Best Practice**: Understand all code locations that need modification

#### `directory_tree`
- **Purpose**: Map controller source structure and understand code organization
- **When to Use**: Understanding controller architecture and file dependencies
- **Example Usage**: Explore `controller/src/` structure and module relationships
- **Best Practice**: Understand module structure before making changes

#### `list_directory`
- **Purpose**: Inventory controller source files and identify modification targets
- **When to Use**: Cataloging files that need updates for agent-specific naming
- **Example Usage**: List files in `controller/src/tasks/code/` directory
- **Best Practice**: Ensure all relevant files are identified for modification

## Supporting Tools

### Rust Documentation Access

#### `rustdocs_query_rust_docs`
- **Purpose**: Access Rust documentation for kube-rs and related crates
- **When to Use**: Understanding PersistentVolumeClaim API and Kubernetes resource management
- **Example Usage**: Query documentation for `kube::Api<PersistentVolumeClaim>` usage
- **Best Practice**: Understand API patterns before implementing new functionality

### Knowledge Management

#### `memory_create_entities`
- **Purpose**: Create knowledge graph nodes for controller components and modifications
- **When to Use**: Recording discovered code patterns and implementation decisions
- **Example Usage**: Create entities for "PVC Creation", "Agent Name Extraction", "Controller Integration"
- **Best Practice**: Document complex controller logic and modification relationships

#### `memory_add_observations`
- **Purpose**: Add detailed findings about controller implementation and changes
- **When to Use**: Recording specific code patterns and modification approaches
- **Example Usage**: Add observations about reconciliation patterns and error handling
- **Best Practice**: Include code snippets and implementation details

### Research Tools

#### `brave_web_search`
- **Purpose**: Research Rust patterns, kube-rs usage, and Kubernetes controller best practices
- **When to Use**: Finding examples of PVC management and controller modification patterns
- **Example Usage**: Search for "kube-rs PersistentVolumeClaim creation patterns"
- **Best Practice**: Validate findings against official documentation and existing codebase

## Implementation Flow

### Phase 1: Code Discovery and Analysis
1. Use `directory_tree` to map controller source structure
2. Use `search_files` to locate PVC-related code and patterns
3. Use `read_file` to examine current PVC creation logic
4. Create knowledge entities for controller architecture components
5. Document current implementation patterns and constraints

### Phase 2: Agent Name Extraction Development
1. Use `rustdocs_query_rust_docs` to understand regex and string manipulation patterns
2. Use `write_file` to implement `extract_agent_name()` function
3. Create comprehensive unit tests for extraction logic
4. Add validation for Kubernetes naming constraints
5. Test with various GitHub App naming patterns

### Phase 3: PVC Logic Modification
1. Use `read_file` to understand existing PVC creation functions
2. Modify PVC naming logic to use agent-specific patterns
3. Implement idempotent PVC creation with kube-rs
4. Add proper error handling and recovery mechanisms
5. Create integration tests for PVC management

### Phase 4: Controller Integration
1. Update reconciliation logic to use new PVC naming
2. Modify pod creation to mount agent-specific workspaces
3. Implement backward compatibility for existing workflows
4. Add comprehensive logging and error handling
5. Test controller integration end-to-end

### Phase 5: Testing and Validation
1. Create unit tests for all new functions
2. Implement integration tests for controller behavior
3. Test backward compatibility scenarios
4. Validate performance impact measurements
5. Document changes and update technical documentation

## Rust Development Patterns

### Agent Name Extraction Function
```rust
use regex::Regex;

fn extract_agent_name(github_app: &str) -> Result<String, String> {
    let re = Regex::new(r"(?i)5dlabs[_-]?(\w+)(?:\[bot\])?").unwrap();
    
    if let Some(caps) = re.captures(github_app) {
        let agent_name = caps.get(1).unwrap().as_str().to_lowercase();
        
        // Validate Kubernetes naming constraints
        validate_k8s_name(&agent_name)?;
        Ok(agent_name)
    } else {
        Err(format!("Cannot extract agent name from: {}", github_app))
    }
}

fn validate_k8s_name(name: &str) -> Result<(), String> {
    if name.len() > 63 {
        return Err("Name exceeds Kubernetes limit".to_string());
    }
    
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Err("Name contains invalid characters".to_string());
    }
    
    Ok(())
}
```

### PVC Creation with kube-rs
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
    
    match pvc_api.get(&pvc_name).await {
        Ok(_) => Ok(pvc_name),
        Err(kube::Error::Api(e)) if e.code == 404 => {
            let pvc_spec = create_pvc_spec(&pvc_name, &code_run.spec.service, &agent_name);
            pvc_api.create(&PostParams::default(), &pvc_spec).await?;
            Ok(pvc_name)
        }
        Err(e) => Err(e),
    }
}
```

### Controller Integration Pattern
```rust
// In reconcile function
async fn reconcile(
    code_run: Arc<CodeRun>,
    ctx: Arc<Context>,
) -> Result<Action, Error> {
    let client = &ctx.client;
    
    // Extract agent name early
    let agent_name = match extract_agent_name(&code_run.spec.github_app) {
        Ok(name) => name,
        Err(e) => {
            error!("Failed to extract agent name: {}", e);
            update_status_with_error(&code_run, &e, client).await?;
            return Ok(Action::requeue(Duration::from_secs(60)));
        }
    };
    
    // Ensure agent-specific PVC exists
    let pvc_name = ensure_agent_pvc(&code_run, client).await?;
    
    // Create pod with agent workspace
    create_agent_pod(&code_run, &pvc_name, &agent_name, client).await?;
    
    Ok(Action::requeue(Duration::from_secs(30)))
}
```

## Best Practices

### Code Development Principles
- Read and understand existing code before making modifications
- Implement incremental changes with comprehensive testing
- Maintain backward compatibility during transition periods
- Add proper error handling and logging throughout

### Rust-Specific Patterns
- Use Result types for proper error handling
- Implement idempotent operations for Kubernetes resources
- Follow Rust naming conventions and ownership patterns
- Add comprehensive unit tests for all new functions

### Controller Development Standards
- Maintain reconciliation idempotency
- Implement proper status updates and error reporting
- Add structured logging for operational visibility
- Consider performance impact of modifications

### Testing Strategy
- Create unit tests for agent name extraction logic
- Implement integration tests for PVC management
- Test backward compatibility scenarios thoroughly
- Validate performance impact with benchmarks

## Testing Strategy

### Unit Testing Approach
1. **Agent Name Extraction**: Test all supported GitHub App patterns
2. **Validation Logic**: Test Kubernetes naming constraint compliance
3. **Error Handling**: Verify appropriate error messages and recovery
4. **Edge Cases**: Test malformed inputs and boundary conditions

### Integration Testing Pattern
1. **PVC Creation**: Test with real Kubernetes cluster
2. **Controller Flow**: Validate complete reconciliation process
3. **Multi-Agent Scenarios**: Test concurrent agent operations
4. **Migration Testing**: Validate backward compatibility

### Performance Testing Requirements
1. **Extraction Speed**: Benchmark agent name extraction performance
2. **Memory Usage**: Monitor controller memory consumption
3. **Reconciliation Impact**: Measure reconciliation timing changes
4. **Concurrent Operations**: Test with multiple simultaneous requests

## Common Patterns

### Controller Modification Pattern
1. Analyze existing code structure and patterns
2. Implement new functionality with proper error handling
3. Integrate with existing reconciliation logic
4. Add comprehensive testing and validation
5. Document changes and update technical specifications

### Error Handling Pattern
1. Use Result types for all potentially failing operations
2. Provide specific error messages for troubleshooting
3. Implement graceful fallback mechanisms where appropriate
4. Add structured logging for operational visibility

## Troubleshooting

### Development Issues
- Use `rustdocs_query_rust_docs` for API documentation questions
- Search existing codebase for similar patterns and implementations
- Test changes incrementally to isolate issues
- Use unit tests to validate individual function behavior

### Integration Problems
- Verify Kubernetes API permissions for PVC operations
- Check namespace and resource naming constraints
- Validate controller RBAC permissions
- Test with actual Kubernetes cluster for integration validation

### Performance Concerns
- Benchmark critical paths before and after modifications
- Monitor memory usage during extended operations
- Test concurrent operations to identify bottlenecks
- Profile code execution for optimization opportunities

## Notes

This task focuses on Rust controller development with emphasis on:
- Agent-specific workspace isolation through PVC naming
- Robust parsing and validation of GitHub App identifiers
- Idempotent Kubernetes resource management
- Backward compatibility and migration support
- Comprehensive testing and performance validation

The tool selection enables comprehensive Rust development while maintaining access to documentation and research capabilities essential for controller modification.