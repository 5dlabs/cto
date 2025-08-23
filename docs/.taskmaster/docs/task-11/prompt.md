# Autonomous Agent Prompt: Setup Tess Kubernetes RBAC

## Mission

You are tasked with implementing comprehensive Kubernetes RBAC (Role-Based Access Control) for the Tess agent within the multi-agent orchestration system. Tess is the Quality Assurance & Testing specialist that requires extensive Kubernetes permissions to perform live deployment testing and validation.

## Context

**System Architecture**: Multi-agent Play Workflow with Rex (implementation) → Cleo (code quality) → Tess (testing) → Human approval

**Your Role**: Infrastructure engineer setting up security foundations for automated testing

**Tess Agent Requirements**:


- Live Kubernetes deployment testing


- Comprehensive regression testing in real environments


- Admin access to Postgres, Redis, Argo CD for validation


- Pod health and resource utilization monitoring


- Integration testing across services



## Primary Objectives

### 1. ServiceAccount Configuration
Create `coderun-tess` ServiceAccount in the `agent-platform` namespace with proper token mounting for Kubernetes API access.

### 2. Permission Structure
Implement ClusterRole with comprehensive permissions. For lab environment, use cluster-admin level access; document production refinement strategy.

### 3. Controller Integration
Modify the CodeRun CRD controller to use agent-specific ServiceAccounts based on the `github_app` field.

### 4. Security Boundaries
Ensure other agents (Rex, Cleo, Blaze) don't inherit excessive permissions while Tess has necessary testing access.

## Technical Implementation



### Required RBAC Resources




```yaml
# 1. ServiceAccount for Tess
apiVersion: v1
kind: ServiceAccount
metadata:
  name: coderun-tess
  namespace: agent-platform

# 2. ClusterRole with testing permissions
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: tess-testing-permissions
rules:
- apiGroups: ["*"]
  resources: ["*"]
  verbs: ["*"]  # Lab environment - refine for production

# 3. ClusterRoleBinding
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: tess-testing-permissions
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: tess-testing-permissions
subjects:
- kind: ServiceAccount
  name: coderun-tess
  namespace: agent-platform






```

### Controller Code Changes

Modify the controller to select ServiceAccount based on agent type:




```rust
// Extract agent name from github_app field
fn extract_agent_name(github_app: &str) -> &str {
    match github_app {
        "5DLabs-Tess" => "tess",
        "5DLabs-Cleo" => "cleo",
        _ => "default"
    }
}

// Use agent-specific ServiceAccount
let service_account_name = format!("coderun-{}", extract_agent_name(&code_run.spec.github_app));
pod_spec.service_account_name = Some(service_account_name);






```



## Critical Success Criteria

1. **Functional Validation**:


   - Tess can create, read, update, delete test resources


   - ServiceAccount tokens properly mounted in pods


   - Controller uses correct ServiceAccount per agent

2. **Security Validation**:


   - Other agents (Rex, Cleo) maintain current permission levels


   - Permission boundaries properly enforced


   - No privilege escalation for non-Tess agents

3. **Integration Testing**:


   - Test resource creation/deletion operations


   - Verify cross-namespace access works


   - Validate admin interface accessibility

## Implementation Strategy

### Phase 1: RBAC Resource Creation


1. Create RBAC manifest file in `infra/resources/rbac/`


2. Apply resources to agent-platform namespace


3. Verify ServiceAccount creation and token generation


4. Test basic kubectl operations with ServiceAccount

### Phase 2: Controller Integration


1. Locate CodeRun controller pod creation logic


2. Implement agent-specific ServiceAccount selection


3. Update CRD processing to include serviceAccountName


4. Test controller changes with Tess CodeRun creation

### Phase 3: Validation & Testing


1. Create test CodeRuns for different agents


2. Verify permission boundaries between agents


3. Test Tess comprehensive Kubernetes operations


4. Document any permission adjustments needed



## Key Files to Modify



- `infra/resources/rbac/tess-rbac.yaml` (new)


- `controller/src/tasks/code/controller.rs`


- `controller/src/crds/coderun.rs` (if needed)

## Testing Commands




```bash
# Verify ServiceAccount creation
kubectl get serviceaccount coderun-tess -n agent-platform

# Check ClusterRoleBinding
kubectl describe clusterrolebinding tess-testing-permissions

# Test ServiceAccount permissions
kubectl auth can-i "*" "*" --as=system:serviceaccount:agent-platform:coderun-tess

# Verify token mounting in pod
kubectl describe pod <tess-pod> -n agent-platform | grep -A5 "Mounts:"






```



## Expected Deliverables



1. **RBAC manifest file** with ServiceAccount, ClusterRole, ClusterRoleBinding


2. **Controller code changes** for agent-specific ServiceAccount usage


3. **Verification that Tess has required permissions** for comprehensive testing


4. **Confirmation that other agents** maintain appropriate permission levels


5. **Documentation of permission structure** and future refinement strategy

## Dependencies & Prerequisites



- Agent-platform namespace exists


- CodeRun CRD controller is operational


- Kubernetes cluster has RBAC enabled


- Understanding of multi-agent orchestration system

## Constraints

- **Lab Environment Focus**: Security hardening is secondary to functionality
- **Backward Compatibility**: Don't break existing Rex/Blaze/Cleo operations
- **Future Extensibility**: Design supports additional agents with different permission needs



## Security Notes

**Lab Environment**: cluster-admin permissions acceptable for isolated testing environment

**Production Considerations**: Document strategy for permission refinement:


- Monitor actual Kubernetes operations performed by Tess


- Create restrictive permissions based on real usage patterns


- Implement network policies and resource quotas


- Add audit logging for compliance



## Quality Gates

Before marking complete:


- [ ] Tess ServiceAccount created and functional


- [ ] Controller uses agent-specific ServiceAccounts correctly


- [ ] Permission boundaries tested and validated


- [ ] No regression in other agent functionality


- [ ] Comprehensive testing permissions verified

This RBAC implementation establishes the security foundation for Tess to perform its critical role in live Kubernetes testing within the multi-agent orchestration system.
