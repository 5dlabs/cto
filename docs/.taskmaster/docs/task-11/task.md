# Task 11: Setup Tess Kubernetes RBAC



## Overview

Configure comprehensive Kubernetes Role-Based Access Control (RBAC) permissions for the Tess agent to perform live Kubernetes testing and deployments as part of the multi-agent orchestration system. This task establishes the security foundation for Tess to validate implementations in real Kubernetes environments.

## Context

Tess is the Quality Assurance & Testing specialist agent in the Play Workflow system. Unlike implementation agents (Rex/Blaze) and code quality agents (Cleo), Tess requires extensive Kubernetes permissions to:



- Deploy applications in live environments


- Perform comprehensive regression testing


- Access admin interfaces (Postgres, Redis, Argo CD)


- Validate pod health and resource utilization


- Execute integration testing across services

## Technical Requirements

### 1. ServiceAccount Creation

Create a dedicated ServiceAccount for Tess in the agent-platform namespace:




```yaml
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

### 2. ClusterRole Definition

For the lab environment, create a ClusterRole with cluster-admin permissions. In production, this should be refined to specific permissions:




```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: tess-testing-permissions
  labels:
    app: tess-agent
    component: rbac
rules:
# Lab Environment: Broad permissions for comprehensive testing
- apiGroups: ["*"]
  resources: ["*"]
  verbs: ["*"]

# Production-Ready Alternative (more restrictive):
# - apiGroups: ["", "apps", "extensions", "networking.k8s.io"]


#   resources: ["*"]


#   verbs: ["*"]


# - apiGroups: ["argoproj.io"]


#   resources: ["*"]


#   verbs: ["*"]
# - apiGroups: ["agents.platform"]


#   resources: ["*"]


#   verbs: ["*"]






```

### 3. ClusterRoleBinding

Link the ServiceAccount to the ClusterRole:




```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: tess-testing-permissions
  labels:
    app: tess-agent
    component: rbac
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: tess-testing-permissions
subjects:
- kind: ServiceAccount
  name: coderun-tess
  namespace: agent-platform






```

### 4. Controller Integration

Update the CodeRun CRD controller to use agent-specific ServiceAccounts:




```rust
// In controller/src/tasks/code/controller.rs
fn get_service_account_name(github_app: &str) -> String {
    match github_app {
        "5DLabs-Tess" => "coderun-tess".to_string(),
        "5DLabs-Cleo" => "coderun-cleo".to_string(), // Future enhancement
        _ => "coderun-default".to_string(), // Rex, Blaze, others
    }
}

// Update pod specification
let service_account_name = get_service_account_name(&code_run.spec.github_app);
pod_spec.service_account_name = Some(service_account_name);






```

### 5. RBAC Aggregation (Future Enhancement)

For maintainability in production environments, consider using Kubernetes RBAC aggregation:




```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: tess-base-permissions
  labels:
    rbac.authorization.k8s.io/aggregate-to-tess: "true"
rules:
- apiGroups: [""]
  resources: ["pods", "services", "configmaps"]
  verbs: ["*"]



---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: tess-app-permissions
  labels:
    rbac.authorization.k8s.io/aggregate-to-tess: "true"
rules:
- apiGroups: ["apps"]
  resources: ["deployments", "replicasets"]
  verbs: ["*"]



---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: tess-aggregated-permissions
aggregationRule:
  clusterRoleSelectors:
  - matchLabels:
      rbac.authorization.k8s.io/aggregate-to-tess: "true"
rules: [] # Rules automatically populated by aggregation






```

## Implementation Steps



### Phase 1: Basic RBAC Setup



1. **Create RBAC manifests** in `infra/resources/rbac/tess-rbac.yaml`


2. **Apply RBAC resources** to the cluster


3. **Verify ServiceAccount creation** and token mounting


4. **Test basic Kubernetes access** with the new ServiceAccount

### Phase 2: Controller Integration



1. **Modify controller code** to use agent-specific ServiceAccounts


2. **Update CodeRun CRD handling** to reference serviceAccountName


3. **Test agent pod creation** with correct ServiceAccount


4. **Verify Tess has required permissions** in container environment

### Phase 3: Permission Validation



1. **Test comprehensive Kubernetes operations** Tess requires


2. **Validate access to admin resources** (if available)


3. **Verify permission boundaries** for security


4. **Document any permission refinements** needed

## Security Considerations

### Lab Environment Approach


- **cluster-admin permissions** acceptable for isolated testing


- Focus on functionality over security hardening


- Monitor resource usage and operations

### Production Considerations


- **Principle of least privilege** - grant only necessary permissions


- **Regular permission audits** to prevent privilege creep


- **Network policies** to restrict Tess container network access


- **Resource quotas** to limit impact of Tess operations


- **Audit logging** for all Tess Kubernetes operations

### Permission Refinement Strategy




```yaml
# Start with broad permissions, then restrict based on actual usage
# Monitor Tess operations to identify minimum required permissions:
# 1. Log all Kubernetes API calls made by Tess
# 2. Analyze logs to determine essential permissions
# 3. Create restrictive ClusterRole with only required permissions
# 4. Test Tess functionality with restricted permissions
# 5. Iterate until optimal permission set is achieved






```

## Testing Strategy

### Functional Tests


1. **ServiceAccount token mounting** in Tess pods


2. **Basic Kubernetes operations** (get, list, create, delete)


3. **Cross-namespace access** if required for testing


4. **Custom resource access** (CodeRun CRDs, Argo Workflows)

### Permission Boundary Tests


1. **Verify other agents don't inherit Tess permissions**


2. **Test permission isolation** between agent ServiceAccounts


3. **Validate resource access restrictions** work as expected


4. **Confirm admin interface access** (Postgres, Redis, Argo CD)

### Security Validation


1. **Audit log analysis** for unexpected permission usage


2. **Network policy compliance** testing


3. **Resource quota enforcement** validation


4. **ServiceAccount token rotation** testing

## Monitoring and Alerting



### RBAC Metrics to Track


- ServiceAccount creation and deletion events


- Permission denied errors from Tess operations


- Unusual Kubernetes API usage patterns


- Resource creation/deletion by Tess

### Alerts to Configure


- Tess ServiceAccount authentication failures


- Excessive resource creation by Tess


- Permission escalation attempts


- ServiceAccount token expiration warnings

## Dependencies

- **Task 9**: Multi-agent orchestration system foundation


- Existing agent-platform namespace and infrastructure


- CodeRun CRD controller operational


- Kubernetes cluster with RBAC enabled



## Expected Outcomes



1. **Tess agent has comprehensive Kubernetes permissions** for live testing


2. **ServiceAccount properly configured** and tokens mounted in pods


3. **Controller uses agent-specific ServiceAccounts** based on github_app


4. **Permission boundaries validated** to ensure security


5. **Foundation established** for Tess comprehensive testing capabilities

## Future Enhancements



- **Production permission refinement** based on actual usage patterns


- **RBAC aggregation implementation** for maintainable permissions


- **ServiceAccount for other agents** (Cleo, future agents)


- **Advanced security hardening** (network policies, resource quotas)


- **Automated permission auditing** and compliance checking
