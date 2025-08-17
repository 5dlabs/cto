# Autonomous Agent Prompt: Create MCP Documentation Server Integration

## Mission

You are tasked with integrating the MCP (Model Context Protocol) documentation server with Rex/Blaze implementation agents to enable a documentation-first workflow. This integration ensures implementation agents research existing patterns and best practices before coding, reducing iteration cycles and improving code quality.

## Context

**System Architecture**: Multi-agent Play Workflow with specialized agents
- **Implementation Agents**: Rex/Blaze need documentation access for better implementations
- **Quality Agents**: Cleo focuses on code quality, doesn't need documentation queries
- **Testing Agents**: Tess focuses on testing, doesn't need documentation queries

**Your Role**: DevOps/Integration engineer enabling documentation-first development workflow

**Problem Statement**: Implementation agents currently start coding without researching existing patterns, leading to:
- Rework cycles during code review (Cleo phase)
- Testing failures due to API misuse (Tess phase) 
- Inconsistent implementation patterns
- Missed opportunities to reuse existing components

## Primary Objectives

### 1. MCP Server Deployment Verification
Ensure the existing `rustdocs-mcp` server is operational and accessible from agent containers.

### 2. Container Script Integration
Modify Rex and Blaze container scripts to include MCP documentation server access and connectivity testing.

### 3. Agent-Specific Tool Configuration
Configure MCP documentation tools only for implementation agents (Rex/Blaze), not quality/testing agents (Cleo/Tess).

### 4. Documentation-First Workflow Implementation
Update agent system prompts to enforce documentation research before implementation.

## Technical Implementation

### Phase 1: MCP Server Validation

**Verify Existing Deployment**:
```bash
# Check server deployment status
kubectl get deployment rustdocs-mcp-server -n agent-platform
kubectl get service rustdocs-mcp-service -n agent-platform
kubectl get pods -l app=rustdocs-mcp-server -n agent-platform

# Test server accessibility
kubectl port-forward svc/rustdocs-mcp-service 8080:8080 -n agent-platform &
curl http://localhost:8080/health
curl "http://localhost:8080/api/v1/docs/search?q=serde"
```

**Expected Server Configuration**:
```yaml
# Service endpoint for container access
MCP_DOCS_SERVER: "http://rustdocs-mcp-service.agent-platform.svc.cluster.local:8080"
```

### Phase 2: Container Script Modifications

**Target Files**:
- `infra/charts/controller/claude-templates/code/container-rex.sh.hbs`
- `infra/charts/controller/claude-templates/code/container-blaze.sh.hbs`

**Required Integration Pattern**:
```handlebars
{{!-- Documentation-first setup for implementation agents --}}
{{#if (or (eq github_app "5DLabs-Rex") (eq github_app "5DLabs-Blaze"))}}
echo "📚 Configuring documentation-first workflow"

# MCP Documentation Server Configuration
export MCP_DOCS_SERVER="http://rustdocs-mcp-service.agent-platform.svc.cluster.local:8080"
export DOCS_FIRST_MODE="true"

# Test documentation server connectivity with retry logic
test_docs_connectivity() {
    local max_retries=3
    local retry_delay=2
    
    for i in $(seq 1 $max_retries); do
        echo "🔍 Attempt $i/$max_retries: Testing MCP docs server..."
        if curl -f -s --connect-timeout 5 "$MCP_DOCS_SERVER/health" >/dev/null 2>&1; then
            echo "✅ Documentation server accessible"
            return 0
        fi
        
        if [ $i -lt $max_retries ]; then
            echo "⏳ Retrying in ${retry_delay}s..."
            sleep $retry_delay
        fi
    done
    
    echo "⚠️  Documentation server not accessible after $max_retries attempts"
    echo "📝 Proceeding without documentation queries"
    export DOCS_FIRST_MODE="false"
    return 1
}

test_docs_connectivity
{{else}}
echo "🎯 Quality/Testing agent - documentation queries not required"
export DOCS_FIRST_MODE="false"
{{/if}}
```

### Phase 3: Client Configuration Updates

**Target File**: `infra/charts/controller/claude-templates/code/client-config.json.hbs`

**Agent-Specific Tool Configuration**:
```handlebars
{
  "remoteTools": [
    {{#if (or (eq github_app "5DLabs-Rex") (eq github_app "5DLabs-Blaze"))}}
    "rustdocs_query_rust_docs",
    "rustdocs_search_documentation",
    "rustdocs_get_crate_info",
    {{/if}}
    "brave-search_brave_web_search",
    "memory_create_entities",
    "memory_query_entities"
  ],
  "localServers": {
    {{#if (or (eq github_app "5DLabs-Rex") (eq github_app "5DLabs-Blaze"))}}
    "mcp-rustdocs": {
      "command": "mcp-rustdocs-client",
      "args": ["--server-url", "{{MCP_DOCS_SERVER}}"],
      "env": {
        "MCP_SERVER_URL": "{{MCP_DOCS_SERVER}}",
        "RUST_LOG": "info"
      }
    },
    {{/if}}
    "mcp-memory": {
      "command": "mcp-memory-server",
      "args": ["--storage-path", "/workspace/memory"]
    }
  }
}
```

### Phase 4: System Prompt Enhancement

**Target Files**: 
- `infra/charts/controller/claude-templates/agents/rex-system-prompt.md.hbs`
- `infra/charts/controller/claude-templates/agents/blaze-system-prompt.md.hbs`

**Documentation-First Workflow Instructions**:
```markdown
## CRITICAL: Documentation-First Implementation Approach

**MANDATORY WORKFLOW**: Before writing any code, you MUST research using available documentation tools:

### 1. Initial Research Phase (Required)
- Use `rustdocs_search_documentation` to find similar implementations
- Use `rustdocs_query_rust_docs` to understand relevant APIs
- Use `rustdocs_get_crate_info` for dependency information
- Use `brave-search_brave_web_search` for additional context if needed

### 2. Pattern Analysis (Required)
- Identify existing implementation patterns in the codebase
- Understand established conventions and best practices
- Find reusable components and utilities
- Document your research findings before implementation

### 3. Implementation Planning (Required)
- Plan integration with current architecture based on documentation
- Choose appropriate APIs and data structures from research
- Design for testing and maintainability following documented patterns
- Include documentation references in implementation comments

### Documentation Query Examples
```
# Search for existing patterns
rustdocs_search_documentation("authentication token management")
rustdocs_search_documentation("error handling patterns")

# Query specific APIs
rustdocs_query_rust_docs("tokio::net::TcpStream")
rustdocs_query_rust_docs("serde::Serialize")

# Get crate information
rustdocs_get_crate_info("sqlx")
rustdocs_get_crate_info("clap")
```

**Quality Gate**: Your implementation will be reviewed by Cleo for adherence to established patterns. Following documentation-first approach reduces review cycles and improves approval likelihood.
```

## Critical Success Criteria

### 1. Infrastructure Validation
- [ ] MCP documentation server deployed and accessible
- [ ] Service discovery configured correctly
- [ ] Health check endpoint responding
- [ ] Documentation content up-to-date

### 2. Agent Integration
- [ ] Rex containers have MCP documentation tools available
- [ ] Blaze containers have MCP documentation tools available
- [ ] Cleo containers do NOT have documentation tools (focus on quality)
- [ ] Tess containers do NOT have documentation tools (focus on testing)

### 3. Workflow Implementation
- [ ] Container scripts test MCP server connectivity
- [ ] Retry logic handles temporary server unavailability
- [ ] Graceful fallback when documentation server unavailable
- [ ] Agent system prompts enforce documentation-first workflow

### 4. Quality Assurance
- [ ] No impact on Cleo or Tess agent functionality
- [ ] Backward compatibility with existing Rex/Blaze workflows
- [ ] Documentation queries improve implementation quality
- [ ] Reduced iteration cycles in multi-agent pipeline

## Implementation Strategy

### Step 1: Infrastructure Assessment
```bash
# Validate MCP server deployment
kubectl get all -l app=rustdocs-mcp-server -n agent-platform

# Test service accessibility
kubectl run test-pod --rm -i --tty --image=alpine/curl --restart=Never -- \
  curl -f "http://rustdocs-mcp-service.agent-platform.svc.cluster.local:8080/health"

# Verify documentation endpoints
kubectl run test-pod --rm -i --tty --image=alpine/curl --restart=Never -- \
  curl "http://rustdocs-mcp-service.agent-platform.svc.cluster.local:8080/api/v1/docs/search?q=serde"
```

### Step 2: Template System Updates
1. **Modify container script templates** to include MCP server configuration
2. **Update client configuration templates** with agent-specific MCP tools
3. **Enhance system prompt templates** with documentation-first instructions
4. **Test template rendering** with different github_app values

### Step 3: Integration Testing
```bash
# Create test CodeRuns for each agent type
kubectl apply -f - <<EOF
apiVersion: agents.platform/v1
kind: CodeRun
metadata:
  name: test-rex-docs
  namespace: agent-platform
spec:
  github_app: "5DLabs-Rex"
  service: "test-service"
  continue_session: false
EOF

# Verify MCP tools available in Rex container
kubectl exec -it test-rex-docs -- ls -la /workspace/.config/
kubectl exec -it test-rex-docs -- env | grep MCP

# Verify Cleo doesn't have documentation tools
kubectl apply -f - <<EOF
apiVersion: agents.platform/v1
kind: CodeRun
metadata:
  name: test-cleo-no-docs
  namespace: agent-platform
spec:
  github_app: "5DLabs-Cleo"
  service: "test-service"
  continue_session: false
EOF
```

### Step 4: End-to-End Validation
1. **Test documentation-first workflow** with real implementation task
2. **Verify improved code quality** through documentation research
3. **Validate no impact** on Cleo and Tess operations
4. **Monitor performance** of documentation queries

## Key Files to Modify

### Container Templates
- `infra/charts/controller/claude-templates/code/container-rex.sh.hbs`
- `infra/charts/controller/claude-templates/code/container-blaze.sh.hbs`

### Client Configuration
- `infra/charts/controller/claude-templates/code/client-config.json.hbs`

### System Prompts
- `infra/charts/controller/claude-templates/agents/rex-system-prompt.md.hbs`
- `infra/charts/controller/claude-templates/agents/blaze-system-prompt.md.hbs`

### Testing Resources
- Create test CodeRun manifests for validation
- Document testing procedures

## Error Handling and Resilience

### MCP Server Unavailability
- Implement retry logic with exponential backoff
- Provide clear logging when documentation unavailable
- Allow agents to proceed with implementation (with warning)
- Monitor documentation server uptime

### Network Issues
- Use connection timeouts to prevent hanging
- Implement circuit breaker pattern for repeated failures
- Log connectivity issues for debugging
- Graceful degradation to implementation without docs

### Authentication Failures
- Handle authentication token expiration
- Provide clear error messages for auth issues
- Document authentication setup requirements
- Monitor authentication success rates

## Testing Commands

### MCP Server Testing
```bash
# Test server health
curl "http://rustdocs-mcp-service.agent-platform.svc.cluster.local:8080/health"

# Test documentation search
curl "http://rustdocs-mcp-service.agent-platform.svc.cluster.local:8080/api/v1/docs/search?q=tokio"

# Test from within cluster
kubectl run test-connectivity --rm -i --tty --image=alpine/curl --restart=Never -- \
  sh -c 'for i in $(seq 1 5); do curl -f http://rustdocs-mcp-service.agent-platform.svc.cluster.local:8080/health && break; sleep 1; done'
```

### Agent Container Testing
```bash
# Test Rex has documentation tools
kubectl exec -it <rex-pod> -- ls -la ~/.config/claude/client-config.json
kubectl exec -it <rex-pod> -- env | grep -E '(MCP|DOCS)'

# Test Cleo doesn't have documentation tools
kubectl exec -it <cleo-pod> -- ls -la ~/.config/claude/client-config.json
kubectl exec -it <cleo-pod> -- env | grep -E '(MCP|DOCS)' || echo "No docs config (expected)"
```

## Expected Deliverables

1. **Verified MCP server deployment** with health checks passing
2. **Updated container scripts** for Rex/Blaze with MCP integration
3. **Agent-specific client configurations** with appropriate tool access
4. **Enhanced system prompts** enforcing documentation-first workflow
5. **Test validation** showing agents have correct tool access
6. **Documentation** of integration setup and troubleshooting

## Dependencies & Prerequisites

- **Task 6**: Multi-agent orchestration system operational
- **Existing MCP server**: `rustdocs-mcp` deployment in cluster
- **Kubernetes DNS**: Service discovery working correctly
- **Template system**: Handlebars templating system functional

## Constraints

- **Agent Isolation**: Documentation tools only for implementation agents
- **Performance**: Documentation queries shouldn't significantly slow implementations
- **Resilience**: System must work even when documentation server unavailable
- **Backward Compatibility**: Don't break existing agent functionality

## Quality Gates

Before marking complete:
- [ ] MCP documentation server accessible from all agent containers
- [ ] Rex and Blaze have documentation tools configured
- [ ] Cleo and Tess do NOT have documentation tools
- [ ] Container scripts handle server unavailability gracefully
- [ ] System prompts enforce documentation-first approach
- [ ] No regression in existing multi-agent workflows
- [ ] End-to-end testing shows improved implementation quality

This integration establishes a documentation-first development culture that reduces iteration cycles and improves code quality across the multi-agent orchestration system.