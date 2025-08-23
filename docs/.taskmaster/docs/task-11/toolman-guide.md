# Toolman Guide: Setup Tess Kubernetes RBAC



## Overview

This guide provides comprehensive instructions for implementing Kubernetes RBAC for the Tess agent, including tool usage, best practices, and troubleshooting guidance for autonomous agents working on this task.

## Tool Recommendations

### Primary Tools for RBAC Implementation

#### 1. Kubernetes Resource Management
- **kubernetes_listResources**: List existing RBAC resources
- **kubernetes_describeResource**: Inspect ServiceAccounts and ClusterRoles
- **kubernetes_createResource**: Create RBAC manifests
- **kubernetes_updateResource**: Modify existing RBAC configurations
- **kubernetes_deleteResource**: Clean up test resources

#### 2. Information Research
- **brave-search_brave_web_search**: Research Kubernetes RBAC best practices
- **memory_create_entities**: Store configuration patterns and decisions
- **memory_query_entities**: Retrieve stored RBAC knowledge

### Tool Usage Patterns

#### Phase 1: Discovery and Analysis



```bash
# Use kubernetes_listResources to understand current setup
kubectl get serviceaccounts -n agent-platform
kubectl get clusterroles | grep -i agent
kubectl get clusterrolebindings | grep -i agent

# Use kubernetes_describeResource for detailed analysis
kubectl describe serviceaccount coderun-default -n agent-platform






```

#### Phase 2: RBAC Resource Creation



```yaml
# Use kubernetes_createResource to apply RBAC manifests
# ServiceAccount creation
apiVersion: v1
kind: ServiceAccount
metadata:
  name: coderun-tess
  namespace: agent-platform
  labels:
    app: tess-agent
    component: rbac
automountServiceAccountToken: true






```

#### Phase 3: Validation and Testing



```bash
# Use kubernetes_listResources to verify creation
kubectl get serviceaccount coderun-tess -n agent-platform

# Use brave-search for troubleshooting specific issues
# Search: "kubernetes serviceaccount token not mounting"
# Search: "clusterrolebinding not working"






```



## Best Practices

### 1. Resource Naming Conventions
- **ServiceAccounts**: `coderun-{agent-name}` (e.g., `coderun-tess`)
- **ClusterRoles**: `{agent-name}-{purpose}-permissions` (e.g., `tess-testing-permissions`)
- **ClusterRoleBindings**: Match ClusterRole name for consistency



### 2. Label Strategy



```yaml
metadata:
  labels:
    app: tess-agent
    component: rbac
    managed-by: taskmaster
    version: v1






```

### 3. Progressive Permission Implementation



```yaml
# Start with broad permissions for lab environment
rules:
- apiGroups: ["*"]
  resources: ["*"]
  verbs: ["*"]

# Document production-ready alternative


# rules:
# - apiGroups: ["", "apps", "networking.k8s.io"]


#   resources: ["*"]


#   verbs: ["get", "list", "create", "update", "delete"]






```

### 4. Security Considerations


- Always include comments explaining permission scope


- Document the principle of least privilege strategy


- Include monitoring and audit considerations


- Plan for permission refinement based on actual usage

## Common Workflows



### Workflow 1: Create Complete RBAC Setup



1. **Research Phase**
   ```bash
   # Use brave-search_brave_web_search
   Search: "kubernetes rbac serviceaccount clusterrole best practices"
   Search: "kubernetes agent permissions testing environment"





```



2. **Discovery Phase**
   ```bash
   # Use kubernetes_listResources
   kubectl get serviceaccounts -n agent-platform
   kubectl get clusterroles | grep -E '(agent|coderun)'
   kubectl get clusterrolebindings | grep -E '(agent|coderun)'





```



3. **Implementation Phase**
   ```yaml
   # Use kubernetes_createResource to apply each resource
   # 1. ServiceAccount
   # 2. ClusterRole
   # 3. ClusterRoleBinding





```



4. **Validation Phase**
   ```bash
   # Use kubernetes_describeResource for verification
   kubectl describe serviceaccount coderun-tess -n agent-platform
   kubectl describe clusterrole tess-testing-permissions
   kubectl describe clusterrolebinding tess-testing-permissions





```

### Workflow 2: Controller Code Modification



1. **Code Analysis**
   ```bash
   # Research current controller implementation
   # Look for ServiceAccount handling logic
   # Identify where github_app field is processed





```



2. **Implementation Strategy**
   ```rust
   // Add agent-specific ServiceAccount logic
   fn get_service_account_name(github_app: &str) -> String {
       match github_app {
           "5DLabs-Tess" => "coderun-tess".to_string(),
           "5DLabs-Cleo" => "coderun-cleo".to_string(),
           _ => "coderun-default".to_string(),
       }
   }





```



3. **Testing Approach**
   ```bash
   # Create test CodeRuns for different agents
   # Verify correct ServiceAccount assignment
   # Check pod specifications include proper ServiceAccount





```

### Workflow 3: Permission Testing and Validation



1. **Basic Permission Testing**
   ```bash
   # Use kubernetes_listResources to test access
   kubectl auth can-i "*" "*" --as=system:serviceaccount:agent-platform:coderun-tess
   kubectl auth can-i create pods --as=system:serviceaccount:agent-platform:coderun-tess
   kubectl auth can-i delete deployments --as=system:serviceaccount:agent-platform:coderun-tess





```



2. **Comprehensive Testing**
   ```bash
   # Create test resources to validate permissions
   kubectl create deployment test-tess --image=nginx --dry-run=server --as=system:serviceaccount:agent-platform:coderun-tess
   kubectl create configmap test-cm --from-literal=key=value --dry-run=server --as=system:serviceaccount:agent-platform:coderun-tess





```



3. **Cross-Agent Permission Validation**
   ```bash
   # Ensure other agents don't inherit Tess permissions
   kubectl auth can-i "*" "*" --as=system:serviceaccount:agent-platform:coderun-default





```

## Troubleshooting Guide

### Issue 1: ServiceAccount Not Created
**Symptoms**: Pod creation fails with ServiceAccount not found

**Diagnosis**:



```bash
# Use kubernetes_listResources to check existence
kubectl get serviceaccounts -n agent-platform | grep tess

# Use kubernetes_describeResource for details
kubectl describe serviceaccount coderun-tess -n agent-platform






```

**Resolution**:
1. Verify namespace exists: `kubectl get namespaces | grep agent-platform`


2. Check RBAC manifest syntax and apply again


3. Verify adequate permissions to create ServiceAccounts

### Issue 2: Token Not Mounting in Pods
**Symptoms**: Pods start but can't access Kubernetes API

**Diagnosis**:



```bash
# Check ServiceAccount configuration
kubectl get serviceaccount coderun-tess -n agent-platform -o yaml | grep automountServiceAccountToken

# Inspect pod token mount
kubectl describe pod <pod-name> -n agent-platform | grep -A5 "Mounts:"






```

**Resolution**:
1. Ensure `automountServiceAccountToken: true` in ServiceAccount


2. Verify pod spec includes correct `serviceAccountName`


3. Check for PodSecurityPolicies blocking token mounting

### Issue 3: Permission Denied Errors
**Symptoms**: Tess operations fail with 403 Forbidden

**Diagnosis**:



```bash
# Test specific permissions
kubectl auth can-i <verb> <resource> --as=system:serviceaccount:agent-platform:coderun-tess

# Check ClusterRoleBinding
kubectl describe clusterrolebinding tess-testing-permissions






```

**Resolution**:


1. Verify ClusterRoleBinding links correct ServiceAccount and ClusterRole


2. Check ClusterRole has necessary permissions


3. Ensure ServiceAccount is in correct namespace

### Issue 4: Controller Not Using Correct ServiceAccount
**Symptoms**: Pods created with wrong or default ServiceAccount

**Diagnosis**:



```bash
# Check pod ServiceAccount assignment
kubectl get pods -l github-app=5DLabs-Tess -o yaml | grep serviceAccountName

# Review controller logs
kubectl logs -n agent-platform -l app=controller | grep -i serviceaccount






```

**Resolution**:


1. Verify controller code changes were deployed


2. Check github_app field extraction logic


3. Restart controller pods if needed


4. Validate CodeRun CRD includes correct github_app value



## Tool-Specific Tips

### kubernetes_* Tools


- Always use `--dry-run=server` for testing resource creation
- Use `--as=system:serviceaccount:namespace:name` for permission testing


- Include `--output=yaml` for detailed resource inspection


- Use label selectors to filter resources efficiently



### brave-search_brave_web_search


- Search for official Kubernetes documentation first


- Include version-specific information (e.g., "kubernetes 1.28 rbac")


- Look for troubleshooting guides and common issues


- Research security best practices and compliance requirements



### memory_* Tools


- Store successful RBAC patterns for reuse


- Document permission requirements discovered during testing


- Keep track of troubleshooting solutions


- Record security considerations and decisions



## Quality Checks

### Pre-Implementation Checklist


- [ ] Current RBAC setup analyzed and documented


- [ ] Agent permission requirements clearly defined


- [ ] ServiceAccount naming convention established


- [ ] Security implications reviewed

### Implementation Checklist


- [ ] ServiceAccount created with proper configuration


- [ ] ClusterRole includes necessary permissions


- [ ] ClusterRoleBinding correctly links ServiceAccount and ClusterRole


- [ ] Controller code modified to use agent-specific ServiceAccounts


- [ ] All resources have appropriate labels and metadata

### Post-Implementation Checklist


- [ ] Tess operations work with new permissions


- [ ] Other agents continue to function correctly


- [ ] Permission boundaries validated


- [ ] Documentation updated


- [ ] Monitoring and alerting configured

## Success Indicators

1. **Functional Success**:


   - Tess ServiceAccount created successfully


   - Controller uses correct ServiceAccount per agent


   - All required permissions functional

2. **Security Success**:


   - Permission isolation between agents maintained


   - No privilege escalation for other agents


   - Security boundaries properly enforced

3. **Integration Success**:


   - No impact on existing agent functionality


   - Smooth integration with multi-agent orchestration


   - Clear path for future agent additions

## Additional Resources

### Documentation References


- Kubernetes RBAC Official Documentation


- ServiceAccount Best Practices Guide


- ClusterRole and ClusterRoleBinding Examples


- Security Hardening Guidelines

### Code References


- Controller ServiceAccount handling logic


- CodeRun CRD specification


- Agent identification patterns


- Template system integration points

This guide provides the foundation for successfully implementing Tess Kubernetes RBAC while maintaining security and integration with the broader multi-agent system.
