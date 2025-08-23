# Toolman Guide: Create MCP Documentation Server Integration

## Overview

This guide provides comprehensive instructions for integrating the MCP (Model Context Protocol) documentation server with Rex/Blaze implementation agents, enabling a documentation-first workflow that improves code quality and reduces iteration cycles.

## Tool Recommendations

### Primary Tools for MCP Integration

#### 1. Infrastructure Validation
- **kubernetes_listResources**: Check MCP server deployment status
- **kubernetes_describeResource**: Inspect server configuration and health
- **kubernetes_getResource**: Retrieve specific resource details
- **kubernetes_createResource**: Create test resources for validation

#### 2. Documentation Research
- **brave-search_brave_web_search**: Research MCP protocol and Rust documentation patterns
- **memory_create_entities**: Store integration patterns and configurations
- **memory_query_entities**: Retrieve stored MCP knowledge and troubleshooting solutions

#### 3. Testing and Validation
- **kubernetes_describeResource**: Monitor pod logs and container status
- **kubernetes_updateResource**: Apply template changes and test configurations

### Tool Usage Patterns

#### Phase 1: Infrastructure Discovery and Validation

```bash
# Use kubernetes_listResources to check existing MCP deployment
kubectl get deployments -n agent-platform | grep rustdocs-mcp
kubectl get services -n agent-platform | grep rustdocs-mcp
kubectl get pods -n agent-platform -l app=rustdocs-mcp-server

# Use kubernetes_describeResource for detailed analysis
kubectl describe deployment rustdocs-mcp-server -n agent-platform
kubectl describe service rustdocs-mcp-service -n agent-platform
```

#### Phase 2: Connectivity Testing

```bash
# Test MCP server accessibility
kubectl run mcp-test --rm -i --tty --image=alpine/curl --restart=Never -- \
  curl -f "http://rustdocs-mcp-service.agent-platform.svc.cluster.local:8080/health"

# Test documentation endpoints
kubectl run mcp-docs-test --rm -i --tty --image=alpine/curl --restart=Never -- \
  curl "http://rustdocs-mcp-service.agent-platform.svc.cluster.local:8080/api/v1/docs/search?q=serde"
```

#### Phase 3: Template Integration and Testing

```bash
# Create test CodeRuns to validate template changes
kubectl apply -f - <<EOF
apiVersion: agents.platform/v1
kind: CodeRun
metadata:
  name: test-rex-mcp-integration
  namespace: agent-platform
spec:
  github_app: "5DLabs-Rex"
  service: "test-mcp-service"
  continue_session: false
EOF

# Use kubernetes_describeResource to check pod configuration
kubectl describe pod test-rex-mcp-integration -n agent-platform
```

## Best Practices

### 1. Agent-Specific Configuration Strategy

```handlebars
{{!-- Template Pattern for Agent-Specific MCP Integration --}}
{{#if (or (eq github_app "5DLabs-Rex") (eq github_app "5DLabs-Blaze"))}}
# Implementation agents get documentation tools
export MCP_DOCS_SERVER="http://rustdocs-mcp-service.agent-platform.svc.cluster.local:8080"
export DOCS_FIRST_MODE="true"
echo "📚 Documentation-first workflow enabled for {{github_app}}"
{{else}}
# Quality/Testing agents don't need documentation tools
export DOCS_FIRST_MODE="false"
echo "🎯 Specialized agent {{github_app}} - no documentation queries needed"
{{/if}}
```

### 2. Resilient Connectivity Testing

```bash
# Robust connectivity test with retry logic
test_mcp_connectivity() {
    local server_url="$1"
    local max_retries=3
    local retry_delay=2

    for attempt in $(seq 1 $max_retries); do
        echo "🔍 Attempt $attempt/$max_retries: Testing MCP server at $server_url"

        if curl -f -s --connect-timeout 5 "$server_url/health" >/dev/null 2>&1; then
            echo "✅ MCP documentation server accessible"
            return 0
        fi

        if [ $attempt -lt $max_retries ]; then
            echo "⏳ Retrying in ${retry_delay}s..."
            sleep $retry_delay
        fi
    done

    echo "⚠️ MCP server not accessible after $max_retries attempts"
    echo "📝 Proceeding with implementation without documentation queries"
    export DOCS_FIRST_MODE="false"
    return 1
}
```

### 3. Template Conditional Logic

```handlebars
{{!-- Client Configuration Template Pattern --}}
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
        "CONNECTION_TIMEOUT": "10",
        "RETRY_ATTEMPTS": "3"
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

### 4. System Prompt Enhancement Strategy

```markdown
## Documentation-First Implementation Approach

**CRITICAL WORKFLOW**: Before implementing any code, you MUST research using available documentation tools:

### 1. Research Phase (Mandatory)
```
rustdocs_search_documentation("authentication patterns")
rustdocs_search_documentation("error handling tokio")
rustdocs_query_rust_docs("serde::Serialize")
rustdocs_get_crate_info("sqlx")
```

### 2. Pattern Analysis
- Identify existing implementation patterns
- Understand API usage and best practices
- Find reusable components and utilities

### 3. Implementation with Documentation References
- Follow documented patterns and conventions
- Include documentation source comments
- Use established APIs and data structures
```

## Common Workflows

### Workflow 1: Complete MCP Integration Setup

1. **Infrastructure Validation Phase**
   ```bash
   # Use kubernetes_listResources to check MCP deployment
   kubectl get deployment rustdocs-mcp-server -n agent-platform
   kubectl get service rustdocs-mcp-service -n agent-platform
   kubectl get pods -l app=rustdocs-mcp-server -n agent-platform

   # Use brave-search_brave_web_search for research
   Search: "MCP Model Context Protocol rust documentation server"
   Search: "kubernetes service discovery documentation integration"
   ```

2. **Connectivity Testing Phase**
   ```bash
   # Test MCP server health endpoint
   kubectl run health-check --rm -i --tty --image=alpine/curl --restart=Never -- \
     curl -v "http://rustdocs-mcp-service.agent-platform.svc.cluster.local:8080/health"

   # Test documentation search functionality
   kubectl run docs-test --rm -i --tty --image=alpine/curl --restart=Never -- \
     curl "http://rustdocs-mcp-service.agent-platform.svc.cluster.local:8080/api/v1/docs/search?q=tokio"
   ```

3. **Template Integration Phase**
   ```bash
   # Update container script templates
   # Modify: infra/charts/controller/claude-templates/code/container-rex.sh.hbs
   # Modify: infra/charts/controller/claude-templates/code/container-blaze.sh.hbs

   # Update client configuration templates
   # Modify: infra/charts/controller/claude-templates/code/client-config.json.hbs

   # Update system prompt templates
   # Modify: infra/charts/controller/claude-templates/agents/rex-system-prompt.md.hbs
   # Modify: infra/charts/controller/claude-templates/agents/blaze-system-prompt.md.hbs
   ```

4. **Validation Phase**
   ```bash
   # Create test CodeRuns for all agent types
   # Use kubernetes_createResource to create test instances
   # Use kubernetes_describeResource to verify configurations
   # Use memory_create_entities to store successful patterns
   ```

### Workflow 2: Agent-Specific Configuration Testing

1. **Rex Configuration Validation**
   ```bash
   # Create Rex test CodeRun
   kubectl apply -f - <<EOF
   apiVersion: agents.platform/v1
   kind: CodeRun
   metadata:
     name: test-rex-mcp-config
     namespace: agent-platform
   spec:
     github_app: "5DLabs-Rex"
     service: "test-service"
     continue_session: false
   EOF

   # Verify MCP configuration
   kubectl exec -it test-rex-mcp-config -- env | grep MCP
   kubectl exec -it test-rex-mcp-config -- cat ~/.config/claude/client-config.json | jq '.remoteTools[] | select(. | startswith("rustdocs"))'
   ```

2. **Cleo Configuration Isolation**
   ```bash
   # Create Cleo test CodeRun
   kubectl apply -f - <<EOF
   apiVersion: agents.platform/v1
   kind: CodeRun
   metadata:
     name: test-cleo-isolation
     namespace: agent-platform
   spec:
     github_app: "5DLabs-Cleo"
     service: "test-service"
     continue_session: false
   EOF

   # Verify NO MCP configuration
   kubectl exec -it test-cleo-isolation -- env | grep MCP || echo "No MCP config (expected)"
   kubectl exec -it test-cleo-isolation -- cat ~/.config/claude/client-config.json | jq '.remoteTools[] | select(. | startswith("rustdocs"))' | wc -l # Should be 0
   ```

### Workflow 3: Error Handling and Resilience Testing

1. **Server Unavailability Testing**
   ```bash
   # Scale down MCP server to simulate unavailability
   kubectl scale deployment rustdocs-mcp-server --replicas=0 -n agent-platform

   # Create Rex CodeRun during server downtime
   kubectl apply -f - <<EOF
   apiVersion: agents.platform/v1
   kind: CodeRun
   metadata:
     name: test-rex-server-down
     namespace: agent-platform
   spec:
     github_app: "5DLabs-Rex"
     service: "test-service"
     continue_session: false
   EOF

   # Verify graceful degradation
   kubectl logs test-rex-server-down -n agent-platform | grep -E '(not accessible|Proceeding without|DOCS_FIRST_MODE=false)'

   # Restore server
   kubectl scale deployment rustdocs-mcp-server --replicas=2 -n agent-platform
   ```

2. **Network Timeout Testing**
   ```bash
   # Monitor connectivity testing during server restarts
   kubectl rollout restart deployment rustdocs-mcp-server -n agent-platform

   # Create CodeRun during rollout
   kubectl apply -f test-coderun-during-restart.yaml

   # Check for timeout handling and retry logic
   kubectl logs <pod-name> -n agent-platform | grep -E '(timeout|retry|attempt)'
   ```

## Troubleshooting Guide

### Issue 1: MCP Server Not Accessible
**Symptoms**: Container logs show "Documentation server not accessible"

**Diagnosis**:
```bash
# Use kubernetes_describeResource to check server status
kubectl describe deployment rustdocs-mcp-server -n agent-platform
kubectl describe service rustdocs-mcp-service -n agent-platform

# Check pod health
kubectl get pods -l app=rustdocs-mcp-server -n agent-platform
kubectl logs -l app=rustdocs-mcp-server -n agent-platform
```

**Resolution**:
1. Verify MCP server deployment is running
2. Check service configuration and endpoints
3. Test DNS resolution from agent containers
4. Verify network policies don't block traffic

### Issue 2: Agent Has Wrong Tool Configuration
**Symptoms**: Rex missing rustdocs tools or Cleo has rustdocs tools

**Diagnosis**:
```bash
# Check client configuration in agent containers
kubectl exec -it <pod-name> -- cat ~/.config/claude/client-config.json | jq '.remoteTools[]'
kubectl exec -it <pod-name> -- cat ~/.config/claude/client-config.json | jq '.localServers'

# Check environment variables
kubectl exec -it <pod-name> -- env | grep -E '(MCP|DOCS)'
```

**Resolution**:
1. Verify template conditional logic is correct
2. Check github_app field value in CodeRun spec
3. Restart controller to reload templates if needed
4. Validate template rendering with test CodeRuns

### Issue 3: Documentation-First Workflow Not Enforced
**Symptoms**: Agents implement without querying documentation

**Diagnosis**:
```bash
# Check system prompt includes documentation-first instructions
kubectl exec -it <rex-pod> -- cat /workspace/CLAUDE.md | grep -A 10 "Documentation-First"
kubectl exec -it <rex-pod> -- cat /workspace/CLAUDE.md | grep "rustdocs_search_documentation"
```

**Resolution**:
1. Verify system prompt templates updated correctly
2. Check agent type conditional logic in prompts
3. Ensure documentation tools are available and functional
4. Monitor agent behavior for documentation query patterns

### Issue 4: Template Rendering Failures
**Symptoms**: Controller logs show template errors

**Diagnosis**:
```bash
# Check controller logs for template errors
kubectl logs -n agent-platform -l app=controller | grep -i template
kubectl logs -n agent-platform -l app=controller | grep -E '(handlebars|render)'
```

**Resolution**:
1. Validate Handlebars template syntax
2. Check for missing template variables
3. Test template rendering with different github_app values
4. Verify template file structure and permissions

## Tool-Specific Tips

### kubernetes_* Tools
- Use `--watch` flag to monitor resource changes in real-time
- Include `--output=yaml` for detailed resource inspection
- Use label selectors to filter resources efficiently
- Test connectivity from within cluster using test pods

### brave-search_brave_web_search
- Search for official MCP protocol documentation
- Research Kubernetes service discovery patterns
- Look for container script best practices
- Find template system usage examples

### memory_* Tools
- Store successful MCP integration patterns
- Document troubleshooting solutions for common issues
- Keep track of template modifications that work
- Record performance benchmarks and optimization tips

## Quality Checks

### Pre-Implementation Checklist
- [ ] MCP server deployment verified and operational
- [ ] Service discovery configuration tested
- [ ] Agent role requirements clearly defined
- [ ] Template modification strategy planned

### Implementation Checklist
- [ ] Container scripts updated with MCP integration
- [ ] Client configurations include appropriate tools per agent
- [ ] System prompts enforce documentation-first workflow
- [ ] Error handling and retry logic implemented
- [ ] Agent isolation maintained (tools per agent type)

### Post-Implementation Checklist
- [ ] All agent types tested with their expected configurations
- [ ] MCP server accessible from all implementation agents
- [ ] Quality/testing agents unaffected by changes
- [ ] Documentation-first workflow demonstrably improves outcomes
- [ ] Performance impact within acceptable limits

## Success Indicators

1. **Infrastructure Success**:
   - MCP server deployed and accessible
   - Service discovery working correctly
   - Health checks passing consistently

2. **Integration Success**:
   - Rex and Blaze have documentation tools configured
   - Cleo and Tess do NOT have documentation tools
   - Container scripts handle server unavailability gracefully
   - System prompts enforce documentation-first approach

3. **Quality Improvement**:
   - Implementation agents query documentation before coding
   - Reduced code review cycles in Cleo phase
   - Fewer testing failures in Tess phase
   - Better adherence to established patterns

## Performance Optimization

### MCP Server Optimization
- Monitor response times and optimize documentation indexing
- Implement caching for frequently accessed documentation
- Scale MCP server replicas based on usage patterns
- Configure resource limits and requests appropriately

### Agent Optimization
- Minimize documentation query overhead during startup
- Cache documentation results within agent sessions
- Implement intelligent query patterns (specific before general)
- Monitor and optimize tool invocation frequency

## Additional Resources

### Documentation References
- MCP (Model Context Protocol) specification
- Kubernetes service discovery documentation
- Handlebars templating guide
- Container orchestration best practices

### Code References
- MCP server implementation and API
- Controller template system integration
- Agent container script patterns
- Multi-agent workflow coordination

This guide provides the foundation for successfully integrating MCP documentation server with the multi-agent system while maintaining performance, security, and agent specialization.
