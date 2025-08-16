# Task 5: Implementation DAG WorkflowTemplate (Rex → Clippy → QA → Deploy → Acceptance)

## Overview

This task implements a comprehensive end-to-end implementation workflow as an Argo DAG that takes tasks/issues from conception through deployment and acceptance testing. The workflow orchestrates the complete development lifecycle including implementation, code quality assurance, testing, deployment, and acceptance verification as specified in Functional Requirement 3 (FR3).

## Architecture

The implementation DAG consists of five sequential stages with proper dependencies:

1. **Rex Implementation**: AI-powered code implementation from task specifications
2. **Clippy Formatting**: Code formatting and linting for quality assurance  
3. **QA Testing**: Comprehensive testing including unit, integration, and system tests
4. **Deployment**: Automated deployment to preview/staging environments
5. **Acceptance Testing**: Black-box validation of deployed functionality

## Key Features

### End-to-End Automation
- **Complete Pipeline**: From task specification to deployed, tested functionality
- **Quality Gates**: Each stage validates previous work before proceeding
- **Environment Management**: Automated preview environment creation and cleanup
- **Evidence Collection**: Comprehensive artifact collection at each stage

### Deployment Integration
- **Preview Environments**: Per-PR namespace deployment with Helm
- **Service Discovery**: Automatic endpoint detection and URL generation
- **Infrastructure as Code**: Declarative environment configuration
- **Cleanup Automation**: Automatic resource cleanup on failure or completion

### Acceptance Testing
- **Black-box Validation**: External validation of deployed services
- **Performance Testing**: Response time and throughput validation
- **Health Monitoring**: Service health checks and dependency validation
- **Evidence Archival**: Permanent storage of acceptance test results

## Implementation Details

### DAG Structure

```yaml
apiVersion: argoproj.io/v1alpha1
kind: WorkflowTemplate
metadata:
  name: implementation-dag
spec:
  entrypoint: dag
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
            templateRef:
              name: coderun-template
              template: coderun-main
            arguments:
              parameters:
                - name: github-app
                  value: rex
          - name: clippy-format
            dependencies: [implement]
            templateRef:
              name: coderun-template
              template: coderun-main
            arguments:
              parameters:
                - name: github-app
                  value: clippy
          - name: qa-testing
            dependencies: [clippy-format]
            templateRef:
              name: coderun-template
              template: coderun-main
            arguments:
              parameters:
                - name: github-app
                  value: qa
          - name: deploy
            dependencies: [qa-testing]
            template: deploy
            arguments:
              parameters:
                - name: NS
                  value: "{{tasks.set-params.outputs.parameters.NS}}"
                - name: RELEASE
                  value: "{{tasks.set-params.outputs.parameters.RELEASE}}"
          - name: acceptance
            dependencies: [deploy]
            template: acceptance
            arguments:
              artifacts:
                - name: deploy-urls
                  from: "{{tasks.deploy.outputs.artifacts.deploy-urls}}"
```

### Environment Parameter Generation

The workflow begins with parameter computation for consistent naming:

```yaml
- name: set-params
  script:
    image: alpine:3.20
    command: ["/bin/sh", "-lc"]
    source: |
      set -euo pipefail
      NS="impl-pr-{{workflow.parameters.pr}}"
      RELEASE="{{workflow.parameters.repo}}-pr-{{workflow.parameters.pr}}"
      DOMAIN="{{workflow.parameters.domain}}"
      IMAGE_TAG="{{workflow.parameters.imageTag}}"
      
      echo -n "$NS" > /tmp/ns
      echo -n "$RELEASE" > /tmp/release
      echo -n "$DOMAIN" > /tmp/domain
      echo -n "$IMAGE_TAG" > /tmp/image_tag
      
      echo "Generated parameters: NS=$NS, RELEASE=$RELEASE, DOMAIN=$DOMAIN"
  outputs:
    parameters:
      - name: NS
        valueFrom: {path: /tmp/ns}
      - name: RELEASE
        valueFrom: {path: /tmp/release}
      - name: DOMAIN
        valueFrom: {path: /tmp/domain}
      - name: IMAGE_TAG
        valueFrom: {path: /tmp/image_tag}
```

### Deployment Template Implementation

The deploy template uses DocsRun CRs to execute Helm deployments:

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
            
            # Ensure namespace exists
            kubectl get ns ${NS} >/dev/null 2>&1 || kubectl create ns ${NS}
            kubectl label ns ${NS} preview=true pr=${PR} --overwrite
            
            # Deploy with Helm
            helm upgrade --install ${RELEASE} ${CHART_PATH} \
              -n ${NS} \
              -f ${VALUES_FILE} \
              --set image.tag=${IMAGE_TAG} \
              --wait --timeout=10m
            
            # Extract service URLs
            kubectl get ingress -n ${NS} -o json > /tmp/ing.json
            jq -r '[.items[].spec.rules[].host] | {
              hosts: ., 
              ns: "'${NS}'", 
              release: "'${RELEASE}'"
            }' /tmp/ing.json > /tmp/urls.json
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
  outputs:
    artifacts:
      - name: deploy-urls
        path: /tmp/urls.json
```

### Acceptance Testing Implementation

The acceptance template performs comprehensive black-box testing:

```yaml
- name: acceptance
  inputs:
    artifacts:
      - name: deploy-urls
        path: /input/urls.json
    parameters:
      - name: min_success_pct
        value: "0.95"
      - name: max_p95_ms
        value: "1200"
  container:
    image: curlimages/curl:8.8.0
    command: ["/bin/sh", "-lc"]
    args:
      - |
        set -euo pipefail
        mkdir -p /artifacts/acceptance/responses
        
        HOSTS=$(jq -r '.hosts[]' /input/urls.json)
        TOTAL=0
        OK=0
        TIMES=""
        
        for HOST in $HOSTS; do
          for i in $(seq 1 10); do
            START=$(date +%s%3N)
            CODE=$(curl -ksS \
              -o /artifacts/acceptance/responses/${HOST//./_}-$i.txt \
              -w "%{http_code}" \
              https://$HOST/ || true)
            END=$(date +%s%3N)
            LAT=$((END-START))
            TOTAL=$((TOTAL+1))
            
            if [ "$CODE" -lt 500 ] && [ "$CODE" -ge 200 ]; then
              OK=$((OK+1))
              TIMES="$TIMES $LAT"
            fi
            
            echo "$HOST $i $CODE $LAT" >> /artifacts/acceptance/logs.txt
          done
        done
        
        # Calculate P95 latency
        P95=$(printf "%s\n" $TIMES | sort -n | awk '
          BEGIN{c=0} 
          {a[c++]=$1} 
          END{
            if(c==0) {
              print 999999
            } else {
              print a[int(0.95*c)]
            }
          }'
        )
        
        # Calculate success rate
        SUCCESS=$(awk -v o=$OK -v t=$TOTAL 'BEGIN{
          if(t==0) {
            print 0
          } else {
            print o/t
          }
        }')
        
        # Generate report
        jq -n \
          --argjson success "$SUCCESS" \
          --argjson p95 "$P95" \
          --arg ns "{{tasks.deploy.outputs.parameters.NS}}" \
          '{
            ns: $ns,
            success_pct: $success,
            p95_ms: $p95,
            total_requests: '${TOTAL}',
            successful_requests: '${OK}',
            timestamp: (now | strftime("%Y-%m-%dT%H:%M:%SZ"))
          }' > /artifacts/acceptance/report.json
        
        # Validate thresholds
        awk -v s=$SUCCESS -v thr={{inputs.parameters.min_success_pct}} \
            -v p=$P95 -v pthr={{inputs.parameters.max_p95_ms}} 'BEGIN{
          if(s+0 < thr+0 || p+0 > pthr+0) {
            print "Acceptance test failed: success=" s " (min " thr "), p95=" p "ms (max " pthr "ms)"
            exit 2
          } else {
            print "Acceptance test passed: success=" s ", p95=" p "ms"
          }
        }'
  outputs:
    artifacts:
      - name: acceptance-report
        path: /artifacts/acceptance/report.json
      - name: acceptance-logs
        path: /artifacts/acceptance/logs.txt
      - name: acceptance-responses
        path: /artifacts/acceptance/responses
```

## Agent Integration Patterns

### Rex Implementation Agent

The Rex agent receives task specifications and implements the required functionality:

**Expected Outputs**:
- Source code implementations
- Unit tests for implemented functionality
- Documentation updates
- Commit messages describing changes

**Integration Points**:
- Task specification from GitHub issues or task management system
- Access to repository context and existing codebase
- Integration with development tools and frameworks

### Clippy Quality Assurance

The Clippy agent ensures code quality and consistency:

**Validation Criteria**:
- Code formatting compliance (rustfmt, prettier, black, gofmt)
- Linting rules adherence (clippy, eslint, golangci-lint, ruff)
- Security best practices enforcement
- Performance optimization recommendations

**Integration Points**:
- Code changes from Rex implementation step
- Project-specific linting configurations
- Automated fix application where possible

### QA Testing Agent  

The QA agent creates and executes comprehensive test suites:

**Test Categories**:
- Unit tests for individual components
- Integration tests for service interactions  
- System tests for end-to-end functionality
- Performance tests for non-functional requirements

**Artifact Generation**:
- Test execution reports (JUnit XML, TAP)
- Code coverage reports  
- Performance benchmark results
- Quality gate compliance evidence

## Deployment Strategies

### Preview Environment Management

**Namespace Strategy**:
- Per-PR namespaces: `impl-pr-{pr-number}`
- Automatic namespace labeling for identification and cleanup
- Resource quotas to prevent resource exhaustion
- Network policies for security isolation

**Helm Deployment Pattern**:
```bash
helm upgrade --install ${RELEASE} charts/app \
  -n ${NAMESPACE} \
  -f values-preview.yaml \
  --set image.tag=pr-${PR_NUMBER} \
  --set ingress.host=pr-${PR_NUMBER}.preview.example.com \
  --wait --timeout=10m
```

**Service Discovery**:
- Automatic ingress detection and URL extraction
- Health check endpoint validation
- Service dependency verification
- Load balancer readiness confirmation

### Infrastructure Integration

**Existing Patterns Compliance**:
- Integration with existing Argo CD applications
- Compliance with organizational Helm chart standards  
- Use of shared infrastructure services (databases, caches)
- Network security policy adherence

**Secret Management**:
- Reuse of existing ExternalSecrets for database credentials
- Application-specific secrets via namespace-scoped ExternalSecrets
- No duplication of GitHub App secrets (inherited from Task 2)
- Proper secret rotation and lifecycle management

## Cleanup and Resource Management

### Automatic Cleanup Implementation

```yaml
onExit: cleanup
templates:
  - name: cleanup
    inputs:
      parameters:
        - name: NS
        - name: RELEASE
    script:
      image: alpine/helm:3.14.4
      command: ["/bin/sh", "-lc"]
      source: |
        set -euo pipefail
        
        if [ "{{workflow.status}}" != "Succeeded" ]; then
          echo "Workflow failed, cleaning up preview environment"
          helm uninstall "{{inputs.parameters.RELEASE}}" \
            -n "{{inputs.parameters.NS}}" || true
          kubectl delete ns "{{inputs.parameters.NS}}" \
            --ignore-not-found=true || true
        else
          echo "Workflow succeeded, leaving preview environment for QA approval"
          # Set TTL for automatic cleanup after 24 hours
          kubectl annotate ns "{{inputs.parameters.NS}}" \
            janitor.stakater.com/expires="$(date -d '+24 hours' -Iseconds)" \
            --overwrite || true
        fi
```

### Resource Labeling Strategy

All resources are labeled for proper identification and management:

```yaml
metadata:
  labels:
    preview: "true"
    pr: "{{workflow.parameters.pr}}"
    workflow: "{{workflow.name}}"
    repo: "{{workflow.parameters.repo}}"
    managed-by: "implementation-dag"
```

## Security Considerations

### RBAC Configuration

```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: implementation-dag-role
rules:
  - apiGroups: [""]
    resources: ["namespaces"]
    verbs: ["create", "get", "list", "patch", "update"]
  - apiGroups: [""]
    resources: ["pods", "services", "configmaps"]
    verbs: ["create", "get", "list", "patch", "update", "delete"]
  - apiGroups: ["apps"]
    resources: ["deployments", "replicasets"]
    verbs: ["create", "get", "list", "patch", "update", "delete"]
  - apiGroups: ["networking.k8s.io"]
    resources: ["ingresses", "networkpolicies"]
    verbs: ["create", "get", "list", "patch", "update", "delete"]
  - apiGroups: ["taskmaster.io"]
    resources: ["docsruns"]
    verbs: ["create", "get", "list", "watch"]
```

### Network Security

- **Namespace Isolation**: Network policies restrict cross-namespace communication
- **Ingress Security**: TLS termination and security headers configuration
- **Secret Isolation**: Secrets scoped to specific namespaces and applications
- **Service Mesh Integration**: Istio/Linkerd integration for advanced security

### Secret Reuse Strategy

The implementation leverages existing secret management infrastructure:

- **GitHub App Tokens**: Inherited from Task 2 token generator
- **Database Credentials**: Shared ExternalSecrets for common services
- **Registry Access**: Shared image pull secrets for container registries
- **TLS Certificates**: Shared cert-manager certificates for ingress

## Monitoring and Observability

### Workflow Metrics

- **Stage Success Rates**: Success/failure rates for each DAG stage
- **End-to-End Duration**: Total pipeline execution time
- **Environment Provisioning Time**: Deployment step performance
- **Acceptance Test Results**: Success rates and performance metrics

### Application Metrics

- **Service Health**: Health check status for deployed services
- **Response Times**: P50, P95, P99 latency metrics
- **Error Rates**: Application error rates during acceptance testing
- **Resource Utilization**: CPU and memory usage in preview environments

### Alerting Strategy

- **Pipeline Failures**: Immediate alerts for implementation DAG failures
- **Environment Issues**: Alerts for deployment or health check failures
- **Performance Degradation**: Alerts when acceptance tests exceed thresholds
- **Resource Exhaustion**: Alerts for namespace resource quota violations

## Output Parameters and Integration

### Workflow Outputs

```yaml
outputs:
  parameters:
    - name: preview_urls
      valueFrom:
        artifact:
          name: deploy-urls
          from: "{{tasks.deploy.outputs.artifacts.deploy-urls}}"
    - name: acceptance_report_path
      value: "{{workflow.outputs.artifacts.acceptance-report.url}}"
    - name: acceptance_logs_path  
      value: "{{workflow.outputs.artifacts.acceptance-logs.url}}"
    - name: deployment_namespace
      value: "{{tasks.set-params.outputs.parameters.NS}}"
    - name: helm_release
      value: "{{tasks.set-params.outputs.parameters.RELEASE}}"
```

### Evidence Artifacts

**Rex Implementation Evidence**:
- `/artifacts/rex/commit.txt` - Commit SHA and message
- `/artifacts/rex/changes.diff` - Code changes summary
- `/artifacts/rex/tests.log` - Unit test execution results

**Clippy Quality Evidence**:
- `/artifacts/clippy/report.json` - Linting and formatting results
- `/artifacts/clippy/fixes.diff` - Applied automatic fixes

**QA Testing Evidence**:
- `/artifacts/qa/test-results.xml` - Comprehensive test results
- `/artifacts/qa/coverage.html` - Code coverage reports
- `/artifacts/qa/performance.json` - Performance test results

**Deployment Evidence**:
- `/artifacts/deploy/urls.json` - Service endpoint URLs
- `/artifacts/deploy/resources.yaml` - Deployed Kubernetes resources
- `/artifacts/deploy/helm-status.txt` - Helm deployment status

**Acceptance Evidence**:
- `/artifacts/acceptance/report.json` - Acceptance test summary
- `/artifacts/acceptance/responses/` - HTTP response samples
- `/artifacts/acceptance/performance.json` - Performance metrics

## Integration with FR3 Requirements

### No Auto-Merge Policy

The implementation strictly adheres to the no auto-merge requirement:

- **Manual Approval Required**: QA approval step does not trigger merge
- **Evidence Collection**: Comprehensive evidence for manual review
- **Branch Protection**: Integration with GitHub branch protection rules
- **Audit Trail**: Complete artifact trail for compliance and review

### QA-Only Approval

The workflow generates QA approval without merging:

```bash
# QA approval logic (executed only on success)
if [ "{{workflow.status}}" = "Succeeded" ]; then
  curl -X POST "${GITHUB_API}/repos/${OWNER}/${REPO}/pulls/${PR}/reviews" \
    -H "Authorization: Bearer ${GITHUB_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{
      "event": "APPROVE",
      "body": "Implementation DAG completed successfully. See artifacts for evidence."
    }'
  
  # Post summary comment with links
  curl -X POST "${GITHUB_API}/repos/${OWNER}/${REPO}/issues/${PR}/comments" \
    -H "Authorization: Bearer ${GITHUB_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{
      "body": "## Implementation DAG Results\n\n✅ All stages completed successfully\n\n**Preview Environment**: ['"${PREVIEW_URL}"']('"${PREVIEW_URL}"')\n\n**Artifacts**:\n- [Acceptance Report]('"${ACCEPTANCE_REPORT_URL}"')\n- [Test Results]('"${TEST_RESULTS_URL}"')\n- [Deployment Evidence]('"${DEPLOY_EVIDENCE_URL}"')"
    }'
fi
```

## Future Enhancements

### Planned Features

1. **Progressive Deployment**: Blue/green and canary deployment strategies
2. **Load Testing Integration**: Automated load testing in acceptance phase
3. **Security Scanning**: Integrated vulnerability scanning in deployment
4. **Performance Regression Detection**: Historical performance comparison
5. **Multi-Environment Promotion**: Automatic promotion through staging environments

### Extensibility Points

1. **Custom Deployment Strategies**: Plugin architecture for deployment methods
2. **Additional Test Types**: Contract testing, chaos engineering, security testing
3. **Advanced Monitoring**: APM integration and distributed tracing
4. **Cost Optimization**: Resource usage optimization and cost tracking

## Dependencies

### External Dependencies

- Argo Workflows 3.4+ with DAG support
- Helm 3.14+ for deployment orchestration
- Kubernetes cluster with ingress controller
- Container registry with image storage
- Artifact storage (S3, MinIO, etc.) for evidence

### Internal Dependencies

- coderun-template (Task 3) for agent invocation
- GitHub App token generator (Task 2) for authentication
- External Secrets Operator for credential management
- Monitoring infrastructure for observability

### Tool Dependencies

- kubectl for Kubernetes resource management
- jq for JSON processing and URL extraction
- curl for HTTP-based acceptance testing
- Helm for application deployment and lifecycle

## References

- [Argo Workflows DAG Templates](https://argoproj.github.io/argo-workflows/walk-through/dag/)
- [Helm Charts Best Practices](https://helm.sh/docs/chart_best_practices/)
- [Kubernetes Resource Management](https://kubernetes.io/docs/concepts/policy/resource-quotas/)
- [Preview Environment Patterns](https://kubernetes.io/docs/concepts/overview/working-with-objects/namespaces/)
- [FR3 Functional Requirements Document](../architecture.md#fr3-implementation-flow)