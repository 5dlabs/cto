# Task 12: Create MCP Documentation Server Integration

## Overview

Setup MCP (Model Context Protocol) documentation server integration and configure Rex/Blaze container scripts to query documentation before implementation. This task enables a documentation-first workflow specifically for implementation agents, ensuring they research best practices and existing patterns before coding.

## Context

The multi-agent orchestration system requires implementation agents (Rex/Blaze) to follow a documentation-first approach to reduce iteration cycles and improve code quality. By querying comprehensive documentation before implementation, agents can:

- Understand existing Rust crate patterns and APIs
- Follow established best practices
- Reduce back-and-forth with quality agents (Cleo)
- Minimize rework cycles in the testing phase (Tess)

## Technical Architecture

### MCP Documentation Server

The `rustdocs-mcp` server provides access to Rust documentation through MCP protocol:

```yaml
# Existing deployment in infra/charts/rustdocs-mcp/
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rustdocs-mcp-server
  namespace: agent-platform
spec:
  replicas: 2
  selector:
    matchLabels:
      app: rustdocs-mcp-server
  template:
    metadata:
      labels:
        app: rustdocs-mcp-server
    spec:
      containers:
      - name: server
        image: rustdocs-mcp:latest
        ports:
        - containerPort: 8080
          name: http
        env:
        - name: RUST_LOG
          value: "info"
        - name: DOCS_PATH
          value: "/docs"
        volumeMounts:
        - name: docs-volume
          mountPath: /docs
          readOnly: true
      volumes:
      - name: docs-volume
        configMap:
          name: rust-documentation
```

### Service Discovery Configuration

```yaml
apiVersion: v1
kind: Service
metadata:
  name: rustdocs-mcp-service
  namespace: agent-platform
spec:
  selector:
    app: rustdocs-mcp-server
  ports:
  - name: http
    port: 8080
    targetPort: 8080
  type: ClusterIP
```

### Container Script Integration

Update Rex and Blaze container scripts to include MCP documentation access:

```handlebars
{{!-- container-rex.sh.hbs --}}
#!/bin/bash
set -e

echo "🔍 Rex Implementation Agent - Documentation-First Approach"

# Documentation server configuration
export MCP_DOCS_SERVER="http://rustdocs-mcp-service.agent-platform.svc.cluster.local:8080"
export DOCS_FIRST_MODE="true"

{{#if (eq github_app "5DLabs-Rex")}}
echo "📚 Enabling documentation-first workflow for Rex"
echo "🔗 MCP Docs Server: $MCP_DOCS_SERVER"

# Verify documentation server accessibility
echo "🔍 Testing documentation server connectivity..."
if curl -f -s "$MCP_DOCS_SERVER/health" > /dev/null; then
    echo "✅ Documentation server accessible"
else
    echo "⚠️  Documentation server not accessible, continuing without docs"
    export DOCS_FIRST_MODE="false"
fi
{{/if}}

# Continue with existing container setup
{{> code/container-base.sh.hbs}}
```

## Implementation Requirements

### 1. MCP Server Deployment

**Status**: Already exists in `infra/charts/rustdocs-mcp/`

**Required Actions**:
- Verify deployment is operational
- Ensure service discovery configured
- Validate documentation content is up-to-date
- Test MCP protocol connectivity

### 2. Container Script Modifications

**Target Files**:
- `infra/charts/controller/claude-templates/code/container-rex.sh.hbs`
- `infra/charts/controller/claude-templates/code/container-blaze.sh.hbs`

**Required Changes**:
```handlebars
{{#if (or (eq github_app "5DLabs-Rex") (eq github_app "5DLabs-Blaze"))}}
# Documentation-first workflow setup
export MCP_DOCS_SERVER="http://rustdocs-mcp-service.agent-platform.svc.cluster.local:8080"
export ENABLE_DOCS_QUERIES="true"

echo "📚 Implementation agent configured for documentation-first approach"
echo "🔍 MCP Documentation Server: $MCP_DOCS_SERVER"

# Test documentation server connectivity
if ! curl -f -s "$MCP_DOCS_SERVER/health" >/dev/null 2>&1; then
    echo "⚠️  Warning: Documentation server not accessible"
    echo "📝 Continuing without documentation queries"
    export ENABLE_DOCS_QUERIES="false"
fi
{{else}}
# Cleo and Tess don't require documentation queries
echo "🎯 Quality/Testing agent - no documentation queries needed"
export ENABLE_DOCS_QUERIES="false"
{{/if}}
```

### 3. Client Configuration Updates

**Target File**: `infra/charts/controller/claude-templates/code/client-config.json.hbs`

**Required Changes**:
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
      "command": "mcp-client",
      "args": ["--server", "{{MCP_DOCS_SERVER}}"],
      "env": {
        "MCP_SERVER_URL": "{{MCP_DOCS_SERVER}}"
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

### 4. Agent System Prompt Updates

**Target Files**:
- `infra/charts/controller/claude-templates/agents/rex-system-prompt.md.hbs`
- `infra/charts/controller/claude-templates/agents/blaze-system-prompt.md.hbs`

**Required Additions**:
```markdown
## Documentation-First Implementation Approach

**CRITICAL**: Before implementing any code, you MUST query the MCP documentation server to understand:

1. **Existing Patterns**: Search for similar implementations in the codebase
2. **API Documentation**: Query relevant Rust crate documentation
3. **Best Practices**: Research established patterns for the specific functionality
4. **Integration Points**: Understand how new code should integrate with existing systems

### Documentation Query Workflow

1. **Initial Research Phase**:
   ```
   Use rustdocs_search_documentation to find relevant crates and modules
   Use rustdocs_query_rust_docs to get detailed API information
   Use rustdocs_get_crate_info for dependency and version information
   ```

2. **Implementation Planning**:
   - Review existing implementation patterns
   - Identify reusable components or utilities
   - Plan integration with current architecture
   - Document implementation approach

3. **Code Implementation**:
   - Follow documented patterns and conventions
   - Use established APIs and data structures
   - Implement with testing and maintainability in mind
   - Reference documentation sources in code comments

### Tools Available for Documentation Research

- `rustdocs_query_rust_docs`: Query specific Rust documentation
- `rustdocs_search_documentation`: Search across documentation for patterns
- `rustdocs_get_crate_info`: Get crate metadata and version information
- `brave-search_brave_web_search`: Supplement with external research if needed

**Remember**: This documentation-first approach reduces rework cycles and improves code quality by ensuring implementation follows established patterns.
```

## Service Discovery Implementation

### DNS-Based Service Discovery

```bash
# MCP server accessible via Kubernetes DNS
MCP_DOCS_SERVER="http://rustdocs-mcp-service.agent-platform.svc.cluster.local:8080"

# Health check endpoint
curl "$MCP_DOCS_SERVER/health"

# Documentation query endpoint
curl "$MCP_DOCS_SERVER/api/v1/docs/search?q=tokio"
```

### Authentication Configuration

```yaml
# If authentication required, add service account token
apiVersion: v1
kind: Secret
metadata:
  name: mcp-docs-auth
  namespace: agent-platform
type: Opaque
data:
  token: <base64-encoded-token>
```

### Retry Logic Implementation

```bash
# Container script retry logic
test_docs_server() {
    local max_retries=3
    local retry_count=0

    while [ $retry_count -lt $max_retries ]; do
        if curl -f -s "$MCP_DOCS_SERVER/health" >/dev/null; then
            echo "✅ Documentation server accessible"
            return 0
        fi

        retry_count=$((retry_count + 1))
        echo "⏳ Retry $retry_count/$max_retries: Testing docs server..."
        sleep 2
    done

    echo "❌ Documentation server not accessible after $max_retries retries"
    return 1
}
```

## Implementation Steps

### Phase 1: Verify MCP Server Deployment

1. **Check existing deployment**:
   ```bash
   kubectl get deployment rustdocs-mcp-server -n agent-platform
   kubectl get service rustdocs-mcp-service -n agent-platform
   kubectl get pods -l app=rustdocs-mcp-server -n agent-platform
   ```

2. **Test server accessibility**:
   ```bash
   kubectl port-forward svc/rustdocs-mcp-service 8080:8080 -n agent-platform
   curl http://localhost:8080/health
   curl "http://localhost:8080/api/v1/docs/search?q=serde"
   ```

3. **Validate documentation content**:
   - Ensure Rust documentation is current
   - Verify MCP protocol endpoints work
   - Test documentation search functionality

### Phase 2: Container Script Updates

1. **Modify Rex container template**:
   - Add MCP server configuration
   - Include connectivity testing
   - Add documentation-first instructions

2. **Modify Blaze container template**:
   - Mirror Rex configuration
   - Ensure consistency in documentation approach

3. **Test container script changes**:
   - Create test CodeRun with Rex/Blaze
   - Verify MCP server connectivity from containers
   - Validate environment variables set correctly

### Phase 3: Client Configuration Integration

1. **Update client-config.json template**:
   - Add rustdocs MCP tools for implementation agents
   - Configure local server connections
   - Ensure Cleo/Tess don't get documentation tools

2. **Test MCP tool availability**:
   - Verify tools load correctly in agent environment
   - Test documentation query functionality
   - Validate tool isolation by agent type

### Phase 4: System Prompt Enhancement

1. **Create documentation-first workflow instructions**
2. **Add to Rex and Blaze system prompts**
3. **Test agents follow documentation-first approach**
4. **Validate improved implementation quality**

## Testing Strategy

### Functional Testing

1. **MCP Server Connectivity**:
   ```bash
   # From agent container
   curl "$MCP_DOCS_SERVER/health"
   curl "$MCP_DOCS_SERVER/api/v1/docs/search?q=tokio"
   ```

2. **Documentation Query Testing**:
   - Test rustdocs_query_rust_docs tool
   - Test rustdocs_search_documentation tool
   - Verify results are relevant and accurate

3. **Agent Behavior Testing**:
   - Create test CodeRun for Rex with documentation requirements
   - Verify agent queries documentation before implementation
   - Confirm implementation follows documented patterns

### Integration Testing

1. **Multi-Agent Workflow Testing**:
   - Ensure Cleo doesn't receive documentation tools
   - Verify Tess doesn't have documentation dependencies
   - Test Rex → Cleo → Tess pipeline with documentation-first Rex

2. **Performance Testing**:
   - Monitor documentation query response times
   - Test concurrent access to MCP server
   - Validate retry logic under server load

### Failure Mode Testing

1. **MCP Server Unavailability**:
   - Test agent behavior when docs server is down
   - Verify graceful degradation to implementation without docs
   - Confirm no agent crashes or blocking

2. **Network Issues**:
   - Test timeout handling
   - Verify retry logic works correctly
   - Validate fallback behavior

## Expected Outcomes

### Implementation Success Metrics

1. **MCP Server Operational**:
   - rustdocs-mcp server deployed and accessible
   - Documentation content up-to-date and comprehensive
   - MCP protocol endpoints responding correctly

2. **Agent Integration Complete**:
   - Rex and Blaze containers configured for documentation queries
   - MCP tools available in implementation agent environments
   - Cleo and Tess unaffected by documentation integration

3. **Documentation-First Workflow Active**:
   - Agents query documentation before implementation
   - Implementation quality improved through prior research
   - Reduced iteration cycles in code review and testing phases

### Quality Improvements Expected

- **Reduced Code Review Issues**: Better adherence to established patterns
- **Fewer Testing Failures**: Implementation follows documented APIs correctly
- **Improved Code Consistency**: Use of standard libraries and approaches
- **Better Integration**: Understanding of existing system architecture

## Monitoring and Maintenance

### MCP Server Health Monitoring

```yaml
# Add health check monitoring
apiVersion: v1
kind: Service
metadata:
  name: rustdocs-mcp-metrics
  namespace: agent-platform
spec:
  selector:
    app: rustdocs-mcp-server
  ports:
  - name: metrics
    port: 9090
    targetPort: 9090
```

### Documentation Content Updates

- **Automated Updates**: Configure CI/CD to update documentation regularly
- **Version Management**: Track Rust crate versions in documentation
- **Content Validation**: Ensure documentation accuracy and completeness

### Usage Analytics

- Track documentation query patterns
- Monitor most-requested documentation topics
- Analyze impact on implementation quality
- Measure reduction in review cycles

## Dependencies

- **Task 6**: Multi-agent orchestration system foundation
- Existing `rustdocs-mcp` server deployment
- Kubernetes DNS resolution for service discovery
- MCP protocol client tools availability

## Future Enhancements

- **Extended Documentation Sources**: Include external API documentation
- **Intelligent Caching**: Cache frequently accessed documentation locally
- **Usage Analytics Dashboard**: Track documentation usage and effectiveness
- **Custom Documentation**: Include project-specific patterns and guidelines
- **Version-Aware Documentation**: Automatically select appropriate documentation versions
