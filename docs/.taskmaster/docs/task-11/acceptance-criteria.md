# Acceptance Criteria: Setup Tess Kubernetes RBAC



## Overview

This document defines the specific, testable criteria that must be met to consider Task 11 (Setup Tess Kubernetes RBAC) complete. All criteria must pass before the task can be approved and merged.

## Functional Requirements

### FR-1: ServiceAccount Creation
**Requirement**: Tess agent has dedicated ServiceAccount with proper configuration

**Test Cases**:


- [ ] ServiceAccount `coderun-tess` exists in `agent-platform` namespace
- [ ] ServiceAccount has `automountServiceAccountToken: true`


- [ ] ServiceAccount has appropriate labels for identification


- [ ] ServiceAccount token is automatically created and mounted

**Verification**:



```bash
# Test ServiceAccount existence
kubectl get serviceaccount coderun-tess -n agent-platform

# Verify token auto-mounting enabled
kubectl get serviceaccount coderun-tess -n agent-platform -o yaml | grep automountServiceAccountToken

# Check ServiceAccount token secret creation
kubectl get secrets -n agent-platform | grep coderun-tess






```

### FR-2: ClusterRole Permissions
**Requirement**: Comprehensive ClusterRole provides Tess with necessary testing permissions

**Test Cases**:


- [ ] ClusterRole `tess-testing-permissions` exists


- [ ] ClusterRole includes wildcard permissions (lab environment)


- [ ] ClusterRole has proper metadata and labels


- [ ] Alternative production-ready permissions documented

**Verification**:



```bash


# Verify ClusterRole exists
kubectl get clusterrole tess-testing-permissions

# Check permissions scope
kubectl describe clusterrole tess-testing-permissions

# Test permission validation
kubectl auth can-i "*" "*" --as=system:serviceaccount:agent-platform:coderun-tess






```

### FR-3: ClusterRoleBinding Configuration
**Requirement**: ServiceAccount properly bound to ClusterRole

**Test Cases**:


- [ ] ClusterRoleBinding `tess-testing-permissions` exists


- [ ] Binding correctly links ServiceAccount to ClusterRole


- [ ] Binding has proper metadata and labels


- [ ] Binding references correct namespace and ServiceAccount name

**Verification**:



```bash
# Verify ClusterRoleBinding exists
kubectl get clusterrolebinding tess-testing-permissions

# Check binding details
kubectl describe clusterrolebinding tess-testing-permissions

# Validate subject and roleRef linkage
kubectl get clusterrolebinding tess-testing-permissions -o yaml






```

### FR-4: Controller Integration
**Requirement**: CodeRun controller uses agent-specific ServiceAccounts

**Test Cases**:


- [ ] Controller code modified to extract agent name from `github_app`


- [ ] Tess CodeRuns use `coderun-tess` ServiceAccount


- [ ] Other agents maintain their current ServiceAccount behavior


- [ ] ServiceAccount selection logic is extensible for future agents

**Verification**:



```bash
# Create test Tess CodeRun and verify ServiceAccount
kubectl apply -f test-tess-coderun.yaml
kubectl describe coderun test-tess -n agent-platform
kubectl get pod -l app=coderun,github-app=5DLabs-Tess -o yaml | grep serviceAccountName

# Verify other agents still work
kubectl apply -f test-rex-coderun.yaml
kubectl get pod -l app=coderun,github-app=5DLabs-Rex -o yaml | grep serviceAccountName






```

## Security Requirements

### SR-1: Permission Boundaries
**Requirement**: Tess permissions don't affect other agents

**Test Cases**:


- [ ] Rex agents maintain current permission levels


- [ ] Cleo agents maintain current permission levels


- [ ] Blaze agents maintain current permission levels


- [ ] No privilege escalation for non-Tess agents

**Verification**:



```bash
# Test other agent permissions remain unchanged
kubectl auth can-i "*" "*" --as=system:serviceaccount:agent-platform:coderun-default
kubectl auth can-i create pods --as=system:serviceaccount:agent-platform:coderun-default






```

### SR-2: ServiceAccount Token Security
**Requirement**: ServiceAccount tokens properly managed and secured

**Test Cases**:


- [ ] Tokens are automatically rotated according to cluster policy


- [ ] Token mounting works correctly in pods


- [ ] No token leakage in logs or environment variables


- [ ] Proper token permissions validation

**Verification**:



```bash
# Verify token mounting in running pod
kubectl exec -it <tess-pod> -n agent-platform -- ls -la /var/run/secrets/kubernetes.io/serviceaccount/

# Check token can access Kubernetes API
kubectl exec -it <tess-pod> -n agent-platform -- curl -k -H "Authorization: Bearer $(cat /var/run/secrets/kubernetes.io/serviceaccount/token)" https://kubernetes.default.svc/api/v1/namespaces






```

## Integration Requirements

### IR-1: Comprehensive Kubernetes Operations
**Requirement**: Tess can perform all necessary testing operations

**Test Cases**:


- [ ] Create and delete test deployments


- [ ] Read and update ConfigMaps and Secrets


- [ ] Access pod logs and execute commands


- [ ] Create and manage test Services


- [ ] Work with Custom Resources (CodeRun CRDs)

**Verification**:



```bash
# Test basic Kubernetes operations as Tess ServiceAccount
kubectl create deployment test-deploy --image=nginx --dry-run=server --as=system:serviceaccount:agent-platform:coderun-tess
kubectl create configmap test-cm --from-literal=key=value --dry-run=server --as=system:serviceaccount:agent-platform:coderun-tess
kubectl create service clusterip test-svc --tcp=80:80 --dry-run=server --as=system:serviceaccount:agent-platform:coderun-tess






```



### IR-2: Cross-Namespace Access
**Requirement**: Tess can access resources across namespaces for testing

**Test Cases**:


- [ ] Access resources in agent-platform namespace


- [ ] Access resources in default namespace (if needed)


- [ ] Access resources in application namespaces for testing


- [ ] List and describe resources across namespaces

**Verification**:



```bash
# Test cross-namespace resource access
kubectl get pods --all-namespaces --as=system:serviceaccount:agent-platform:coderun-tess
kubectl get deployments -n default --as=system:serviceaccount:agent-platform:coderun-tess






```

### IR-3: Admin Interface Access
**Requirement**: Tess can access admin interfaces for comprehensive testing

**Test Cases**:


- [ ] Access to Argo CD resources (if deployed)


- [ ] Access to database resources (if accessible via K8s)


- [ ] Access to monitoring resources (if deployed)


- [ ] Access to ingress and networking resources

**Verification**:



```bash
# Test access to various admin resources (adjust based on cluster setup)
kubectl get applications.argoproj.io --all-namespaces --as=system:serviceaccount:agent-platform:coderun-tess
kubectl get ingresses --all-namespaces --as=system:serviceaccount:agent-platform:coderun-tess






```

## Performance Requirements

### PR-1: Controller Performance
**Requirement**: ServiceAccount selection doesn't impact controller performance

**Test Cases**:


- [ ] CodeRun creation time remains within acceptable limits


- [ ] No significant increase in controller resource usage


- [ ] Concurrent CodeRun creation handles ServiceAccount selection correctly


- [ ] Controller startup time not significantly impacted

**Verification**:



```bash
# Monitor controller performance
kubectl top pods -n agent-platform -l app=controller

# Time CodeRun creation with different agents
time kubectl apply -f test-coderun-tess.yaml
time kubectl apply -f test-coderun-rex.yaml






```

## Operational Requirements

### OR-1: Monitoring and Observability
**Requirement**: RBAC changes are observable and monitorable

**Test Cases**:


- [ ] ServiceAccount authentication events logged


- [ ] Permission denied events captured


- [ ] Controller logs show ServiceAccount selection decisions


- [ ] RBAC resources visible in cluster monitoring

**Verification**:



```bash
# Check audit logs for RBAC events
kubectl logs -n kube-system -l component=kube-apiserver | grep -i rbac

# Monitor controller logs
kubectl logs -n agent-platform -l app=controller | grep -i serviceaccount






```

### OR-2: Documentation and Maintenance
**Requirement**: RBAC configuration is properly documented

**Test Cases**:


- [ ] RBAC manifest files properly commented


- [ ] Controller code changes documented


- [ ] Production refinement strategy documented


- [ ] Troubleshooting guide available

**Verification**:


- Review code comments and documentation


- Verify README updates or documentation files


- Check inline comments in YAML manifests

## Regression Testing

### RT-1: Existing Agent Functionality
**Requirement**: No impact on existing agent operations

**Test Cases**:


- [ ] Rex agents continue to work without changes


- [ ] Blaze agents continue to work without changes


- [ ] Existing CodeRun workflows complete successfully


- [ ] No new errors in controller logs

**Verification**:



```bash
# Test existing agent functionality
kubectl apply -f existing-rex-workflow.yaml
kubectl apply -f existing-blaze-workflow.yaml

# Monitor for errors
kubectl logs -n agent-platform -l app=controller --tail=100






```



### RT-2: Backward Compatibility
**Requirement**: Changes are backward compatible with existing configurations

**Test Cases**:


- [ ] Existing CRD specifications work without modification


- [ ] Legacy ServiceAccount usage continues to function


- [ ] No breaking changes in controller API


- [ ] Existing workflows don't require updates

## Edge Cases and Error Handling

### EC-1: ServiceAccount Missing
**Requirement**: Graceful handling when ServiceAccount doesn't exist

**Test Cases**:


- [ ] Controller provides clear error messages


- [ ] Pod creation fails gracefully with helpful errors


- [ ] No controller crashes or restarts


- [ ] Error conditions properly logged

### EC-2: Permission Denied Scenarios
**Requirement**: Clear error reporting for permission issues

**Test Cases**:


- [ ] Permission denied errors clearly reported


- [ ] Tess operations fail gracefully when permissions insufficient


- [ ] Error messages include troubleshooting guidance


- [ ] No security information leakage in errors

## Final Validation Checklist

Before considering Task 11 complete:



- [ ] All functional requirements (FR-1 through FR-4) pass


- [ ] All security requirements (SR-1 through SR-2) pass


- [ ] All integration requirements (IR-1 through IR-3) pass


- [ ] Performance requirements (PR-1) validated


- [ ] Operational requirements (OR-1 through OR-2) met


- [ ] Regression testing (RT-1 through RT-2) successful


- [ ] Edge cases (EC-1 through EC-2) handled properly


- [ ] Code review completed and approved


- [ ] Documentation updated and reviewed


- [ ] Changes tested in isolated environment


- [ ] Ready for integration with multi-agent orchestration system



## Success Metrics



1. **100% test case pass rate** - All test cases must pass


2. **Zero regression issues** - No impact on existing functionality


3. **Performance within 5% of baseline** - No significant performance degradation


4. **Clear audit trail** - All RBAC changes properly logged and documented


5. **Security boundaries validated** - Permission isolation confirmed between agents

When all acceptance criteria are met, Task 11 provides the secure foundation for Tess to perform comprehensive Kubernetes testing within the multi-agent orchestration system.