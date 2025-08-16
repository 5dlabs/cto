# Autonomous Implementation Prompt: Implementation DAG (Rex → Clippy → QA → Deploy → Acceptance)

## Mission Statement

You are implementing a complete end-to-end development workflow that takes tasks from specification to deployed, tested functionality. Your goal is to create a production-ready DAG that orchestrates the entire implementation lifecycle while ensuring no auto-merge occurs and comprehensive evidence collection for manual QA approval.

## Context

This system implements FR3 requirements for a complete implementation flow. The DAG must coordinate AI agents, quality gates, deployment automation, and acceptance testing while maintaining strict separation between automated approval and merge decisions.

## Technical Requirements

### Must Implement

1. **Five-Stage DAG Pipeline**
   - implement (Rex) → clippy-format (Clippy) → qa-testing (QA) → deploy → acceptance
   - Sequential dependencies with proper error propagation
   - Parameter passing between stages for consistency

2. **Preview Environment Management**
   - Per-PR namespace creation (impl-pr-{number})
   - Helm-based deployment with value overrides
   - Service discovery and URL extraction
   - Resource labeling for cleanup and identification

3. **Acceptance Testing Framework**
   - Black-box HTTP testing of deployed services
   - Performance threshold enforcement (success rate, latency)
   - Evidence collection (responses, logs, metrics)
   - Configurable thresholds and test scenarios

4. **QA Approval Without Auto-Merge**
   - GitHub PR approval on successful completion
   - Comprehensive evidence archival and linking
   - No merge API calls under any circumstances
   - Manual review facilitation through artifacts

5. **Resource Lifecycle Management**
   - Cleanup on failure (helm uninstall, namespace deletion)
   - TTL-based cleanup for successful deployments
   - Resource quota and limit enforcement
   - Proper RBAC for multi-namespace operations

### DAG Structure Requirements

```yaml
apiVersion: argoproj.io/v1alpha1
kind: WorkflowTemplate
metadata:
  name: implementation-dag
spec:
  entrypoint: dag
  onExit: cleanup
  arguments:
    parameters:
      - name: owner
      - name: repo
      - name: pr
      - name: commit
        value: ""
      - name: domain
        value: "preview.example.com"
      - name: chartPath
        value: "charts/app"
      - name: valuesFile
        value: "values-preview.yaml"
      - name: imageTag
        value: "pr-{{workflow.parameters.pr}}"
  templates:
    - name: dag
      dag:
        tasks:
          - name: set-params
            template: set-params
          - name: implement
            dependencies: [set-params]
            templateRef: {name: coderun-template, template: coderun-main}
            arguments:
              parameters: [{name: github-app, value: rex}]
          - name: clippy-format
            dependencies: [implement]
            templateRef: {name: coderun-template, template: coderun-main}
            arguments:
              parameters: [{name: github-app, value: clippy}]
          - name: qa-testing
            dependencies: [clippy-format]
            templateRef: {name: coderun-template, template: coderun-main}
            arguments:
              parameters: [{name: github-app, value: qa}]
          - name: deploy
            dependencies: [qa-testing]
            template: deploy
          - name: acceptance
            dependencies: [deploy]
            template: acceptance
```

### Environment Parameter Generation

Create `set-params` template that generates consistent naming:

```bash
set -euo pipefail
NS="impl-pr-{{workflow.parameters.pr}}"
RELEASE="{{workflow.parameters.repo}}-pr-{{workflow.parameters.pr}}"
DOMAIN="{{workflow.parameters.domain}}"
IMAGE_TAG="{{workflow.parameters.imageTag}}"

# Validate parameters
[ -n "$NS" ] || { echo "ERROR: Namespace cannot be empty"; exit 1; }
[ -n "$RELEASE" ] || { echo "ERROR: Release name cannot be empty"; exit 1; }

# Output for downstream consumption
echo -n "$NS" > /tmp/ns
echo -n "$RELEASE" > /tmp/release
echo -n "$DOMAIN" > /tmp/domain
echo -n "$IMAGE_TAG" > /tmp/image_tag
```

### Deployment Template Implementation

Use DocsRun CR for deployment orchestration:

```yaml
- name: deploy
  inputs:
    parameters:
      - name: NS
      - name: RELEASE
  resource:
    action: create
    manifest: |
      apiVersion: taskmaster.io/v1
      kind: DocsRun
      metadata:
        generateName: docsrun-deploy-
      spec:
        command: ["/bin/sh", "-lc"]
        args:
          - |
            set -euo pipefail
            
            # Create and label namespace
            kubectl get ns ${NS} >/dev/null 2>&1 || kubectl create ns ${NS}
            kubectl label ns ${NS} preview=true pr=${PR} workflow={{workflow.name}} --overwrite
            
            # Deploy application
            helm upgrade --install ${RELEASE} ${CHART_PATH} \
              -n ${NS} \
              -f ${VALUES_FILE} \
              --set image.tag=${IMAGE_TAG} \
              --set ingress.host=pr-${PR}.${DOMAIN} \
              --wait --timeout=10m
            
            # Extract service URLs
            kubectl get ingress -n ${NS} -o json | \
              jq '{
                hosts: [.items[].spec.rules[].host],
                ns: "'${NS}'",
                release: "'${RELEASE}'",
                endpoints: [.items[].spec.rules[] | {host: .host, paths: .http.paths[].path}]
              }' > /tmp/urls.json
        env:
          - name: NS
            value: "{{inputs.parameters.NS}}"
          - name: RELEASE
            value: "{{inputs.parameters.RELEASE}}"
          - name: CHART_PATH
            value: "{{workflow.parameters.chartPath}}"
          - name: VALUES_FILE
            value: "{{workflow.parameters.valuesFile}}"
          - name: IMAGE_TAG
            value: "{{workflow.parameters.imageTag}}"
          - name: PR
            value: "{{workflow.parameters.pr}}"
          - name: DOMAIN
            value: "{{workflow.parameters.domain}}"
  outputs:
    artifacts:
      - name: deploy-urls
        path: /tmp/urls.json
```

### Acceptance Testing Implementation

Comprehensive black-box testing with configurable thresholds:

```bash
#!/bin/bash
set -euo pipefail

URLS_FILE="/input/urls.json"
OUTPUT_DIR="/artifacts/acceptance"
MIN_SUCCESS_PCT="{{inputs.parameters.min_success_pct}}"
MAX_P95_MS="{{inputs.parameters.max_p95_ms}}"

mkdir -p "$OUTPUT_DIR"/{responses,logs}

# Parse deployment URLs
HOSTS=$(jq -r '.hosts[]?' "$URLS_FILE" 2>/dev/null || echo "")
if [ -z "$HOSTS" ]; then
  echo "ERROR: No hosts found in deployment URLs"
  exit 1
fi

# Execute acceptance tests
TOTAL_REQUESTS=0
SUCCESSFUL_REQUESTS=0
RESPONSE_TIMES=()

for HOST in $HOSTS; do
  echo "Testing host: $HOST"
  
  for i in $(seq 1 10); do
    START_MS=$(date +%s%3N)
    
    HTTP_CODE=$(curl -ksS \
      --max-time 30 \
      -o "$OUTPUT_DIR/responses/${HOST//\./_}-${i}.txt" \
      -w "%{http_code}" \
      "https://$HOST/" || echo "000")
    
    END_MS=$(date +%s%3N)
    RESPONSE_TIME=$((END_MS - START_MS))
    
    TOTAL_REQUESTS=$((TOTAL_REQUESTS + 1))
    
    if [ "$HTTP_CODE" -ge 200 ] && [ "$HTTP_CODE" -lt 500 ]; then
      SUCCESSFUL_REQUESTS=$((SUCCESSFUL_REQUESTS + 1))
      RESPONSE_TIMES+=("$RESPONSE_TIME")
    fi
    
    # Log each request
    echo "$(date -Iseconds) $HOST request_$i $HTTP_CODE ${RESPONSE_TIME}ms" >> "$OUTPUT_DIR/logs/requests.log"
  done
done

# Calculate metrics
if [ ${#RESPONSE_TIMES[@]} -eq 0 ]; then
  P95_MS=99999
else
  # Sort response times and calculate P95
  IFS=$'\n' SORTED_TIMES=($(sort -n <<<"${RESPONSE_TIMES[*]}"))
  P95_INDEX=$(( (${#SORTED_TIMES[@]} * 95 + 99) / 100 - 1 ))
  P95_MS=${SORTED_TIMES[$P95_INDEX]}
fi

SUCCESS_RATE=$(awk "BEGIN {printf \"%.3f\", $SUCCESSFUL_REQUESTS / $TOTAL_REQUESTS}")

# Generate comprehensive report
jq -n \
  --argjson success_pct "$SUCCESS_RATE" \
  --argjson p95_ms "$P95_MS" \
  --argjson total_requests "$TOTAL_REQUESTS" \
  --argjson successful_requests "$SUCCESSFUL_REQUESTS" \
  --argjson failed_requests "$((TOTAL_REQUESTS - SUCCESSFUL_REQUESTS))" \
  --arg timestamp "$(date -Iseconds)" \
  --arg namespace "$(jq -r '.ns' "$URLS_FILE")" \
  '{
    summary: {
      success_pct: $success_pct,
      p95_ms: $p95_ms,
      total_requests: $total_requests,
      successful_requests: $successful_requests,
      failed_requests: $failed_requests
    },
    thresholds: {
      min_success_pct: '${MIN_SUCCESS_PCT}',
      max_p95_ms: '${MAX_P95_MS}'
    },
    environment: {
      namespace: $namespace,
      timestamp: $timestamp
    }
  }' > "$OUTPUT_DIR/report.json"

# Validate against thresholds
SUCCESS_CHECK=$(awk "BEGIN {print ($SUCCESS_RATE >= $MIN_SUCCESS_PCT) ? \"PASS\" : \"FAIL\"}")
LATENCY_CHECK=$(awk "BEGIN {print ($P95_MS <= $MAX_P95_MS) ? \"PASS\" : \"FAIL\"}")

echo "Acceptance test results:"
echo "  Success rate: $SUCCESS_RATE (threshold: $MIN_SUCCESS_PCT) - $SUCCESS_CHECK"
echo "  P95 latency: ${P95_MS}ms (threshold: ${MAX_P95_MS}ms) - $LATENCY_CHECK"

if [ "$SUCCESS_CHECK" = "FAIL" ] || [ "$LATENCY_CHECK" = "FAIL" ]; then
  echo "ERROR: Acceptance test thresholds not met"
  exit 1
fi

echo "Acceptance tests passed all thresholds"
```

## Implementation Approach

### Phase 1: DAG Structure and Parameter Management

1. **Create implementation-dag.yaml with proper dependencies**
2. **Implement set-params template for consistent naming**
3. **Wire templateRef calls to coderun-template for agent steps**
4. **Test DAG structure and parameter passing**

### Phase 2: Deployment Integration

1. **Build deployment template using DocsRun CRs**
2. **Implement Helm deployment with proper value overrides**
3. **Add service discovery and URL extraction**
4. **Test deployment to preview environments**

### Phase 3: Acceptance Testing Framework

1. **Implement HTTP-based black-box testing**
2. **Add configurable thresholds and metrics calculation**
3. **Implement response collection and logging**
4. **Test against deployed preview environments**

### Phase 4: Cleanup and Resource Management

1. **Implement onExit cleanup template**
2. **Add failure-based resource cleanup**
3. **Implement TTL-based cleanup for successful runs**
4. **Test resource lifecycle management**

### Phase 5: QA Approval and Evidence Integration

1. **Implement GitHub PR approval on success**
2. **Add comprehensive artifact collection**
3. **Create evidence linking and documentation**
4. **Verify no auto-merge functionality**

## Validation Requirements

### DAG Execution Testing

**Must Test:**
- Complete success path: all five stages execute in order
- Stage failure scenarios: each stage failure prevents downstream execution
- Parameter passing: consistent naming across all stages
- Resource cleanup: proper cleanup on both failure and success paths

**Test Commands:**
```bash
# Test complete workflow
argo submit --from workflowtemplate/implementation-dag \
  -p owner=testorg -p repo=testrepo -p pr=123 --watch

# Test failure scenarios
# (Introduce failures at each stage and verify proper cleanup)

# Test resource management
kubectl get ns -l preview=true
helm list -A | grep "pr-"
```

### Deployment Testing

**Must Test:**
- Namespace creation and labeling
- Helm deployment with correct parameters
- Service URL extraction and validation
- Resource cleanup on failure

**Test Setup:**
```bash
# Prepare test environment
helm create charts/testapp
kubectl create clusterrolebinding test-admin --clusterrole=cluster-admin --user=system:serviceaccount:workflows:default

# Test deployment
argo submit --from workflowtemplate/implementation-dag \
  -p owner=test -p repo=testapp -p pr=456 \
  -p chartPath=charts/testapp \
  --watch
```

### Acceptance Testing Validation

**Must Test:**
- HTTP endpoint testing with various response codes
- Performance metric calculation (success rate, P95 latency)
- Threshold validation and failure scenarios
- Artifact collection and report generation

## Security Requirements

### RBAC Configuration

```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: implementation-dag-role
rules:
  # Namespace management
  - apiGroups: [""]
    resources: ["namespaces"]
    verbs: ["create", "get", "list", "patch", "update", "delete"]
  # Application deployment
  - apiGroups: ["", "apps", "networking.k8s.io"]
    resources: ["*"]
    verbs: ["*"]
    resourceNames: [] # Restrict to specific namespaces via RoleBinding
  # DocsRun CR management
  - apiGroups: ["taskmaster.io"]
    resources: ["docsruns"]
    verbs: ["create", "get", "list", "watch"]
```

### Resource Security

- **Namespace Isolation**: Each PR gets isolated namespace
- **Network Policies**: Restrict cross-namespace communication  
- **Resource Quotas**: Prevent resource exhaustion
- **Pod Security**: Non-root containers, read-only filesystems

## Performance Requirements

### Execution Targets

- **Set Parameters**: < 10 seconds
- **Agent Steps**: Based on coderun-template performance
- **Deployment**: < 5 minutes for typical applications
- **Acceptance Testing**: < 3 minutes for standard test suite
- **Total Pipeline**: < 30 minutes end-to-end

### Resource Management

```yaml
resources:
  requests:
    cpu: 100m
    memory: 256Mi
  limits:
    cpu: 2
    memory: 4Gi
```

## Success Criteria

Your implementation is complete when:

1. **DAG Execution**: Five-stage pipeline executes with proper dependencies
2. **Agent Integration**: All three agents (Rex, Clippy, QA) integrate correctly
3. **Deployment Automation**: Preview environments deploy successfully with Helm
4. **Acceptance Testing**: Black-box testing validates deployed functionality
5. **Resource Management**: Proper cleanup and lifecycle management
6. **QA Approval**: GitHub PR approval without auto-merge
7. **Evidence Collection**: Comprehensive artifacts for manual review
8. **Security**: RBAC, resource isolation, and security best practices
9. **Performance**: Meets all execution time and resource targets
10. **Testing**: Complete validation of all success and failure scenarios

## Delivery Artifacts

Create these files:
- implementation-dag.yaml - Complete WorkflowTemplate
- RBAC manifests for cluster access
- Test scenarios for all stages
- Helm chart examples and values files
- Documentation and troubleshooting guide

Remember: Never implement auto-merge functionality. The goal is to provide comprehensive evidence for human decision-making while automating the validation and deployment pipeline.