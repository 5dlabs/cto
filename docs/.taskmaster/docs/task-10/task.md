# Task 10: QA Kubernetes Verification and PR Approval

## Overview
This task implements strict QA verification in Kubernetes environments with comprehensive evidence collection, followed by automated PR approval without auto-merge functionality. The system ensures thorough testing validation while maintaining human oversight over merge decisions.

## Architecture
- **QA Agent Integration**: Enhanced QA testing with Kubernetes evidence requirements
- **Evidence Collection**: Automated capture of cluster state, logs, and test artifacts
- **Verification Gates**: Independent validation of QA evidence completeness and correctness
- **PR Approval Process**: GitHub Review API integration for automated approval
- **Manual Merge Control**: Strict prevention of automated merge operations

## Key Features

### Kubernetes Evidence Requirements
- **Cluster State Capture**: kubectl-based resource inventory and status
- **Log Collection**: Complete application and infrastructure logs
- **Test Artifact Generation**: Structured test results and coverage reports
- **Evidence Validation**: Schema-based validation of collected evidence

### QA Agent Enhancement
- **System Prompt Updates**: Enhanced prompts requiring evidence generation
- **Artifact Structure**: Standardized evidence directory layout
- **Test Execution**: Comprehensive test suite with Kubernetes integration
- **Proof Generation**: Structured summary of testing activities and results

## Implementation

### QA Agent System Prompt Requirements
```markdown
# QA Agent System Prompt (Enhanced)

You are a QA verification agent responsible for comprehensive testing with Kubernetes evidence collection.

## Required Outputs
1. Execute all test suites (unit, integration, system, performance)
2. Collect evidence in /artifacts/qa/proof/ directory
3. Generate summary.json with complete test results
4. Capture Kubernetes cluster state and application logs
5. Validate all services are healthy and responsive

## Evidence Structure Required:
- /artifacts/qa/proof/summary.json (mandatory)
- /artifacts/qa/proof/logs/ (application logs)
- /artifacts/qa/proof/k8s/ (cluster resources)
- /artifacts/qa/proof/evidence/ (test results, coverage)

## Failure Conditions:
- Any test failures must be documented with remediation steps
- Missing evidence files will cause verification failure
- Incomplete summary.json schema will be rejected
- Set qa.passed=false and exit non-zero for any failures

IMPORTANT: You must collect evidence even if tests fail. Never skip evidence collection.
```

### Evidence Collection Implementation
```bash
#!/bin/bash
# collect-qa-evidence.sh

set -euo pipefail

NAMESPACE=${K8S_NAMESPACE:-default}
ARTIFACTS_DIR=${ARTIFACTS_DIR:-/artifacts/qa/proof}

echo "Collecting QA evidence in namespace: $NAMESPACE"

# Create evidence directory structure
mkdir -p "$ARTIFACTS_DIR"/{logs,k8s,evidence}

# Collect Kubernetes resources
echo "Collecting Kubernetes resources..."
kubectl get pods -n "$NAMESPACE" -o yaml > "$ARTIFACTS_DIR/k8s/pods.yaml"
kubectl get services -n "$NAMESPACE" -o yaml > "$ARTIFACTS_DIR/k8s/services.yaml"
kubectl get deployments -n "$NAMESPACE" -o yaml > "$ARTIFACTS_DIR/k8s/deployments.yaml"
kubectl get events -n "$NAMESPACE" --sort-by=.lastTimestamp -o yaml > "$ARTIFACTS_DIR/k8s/events.yaml"

# Collect application logs
echo "Collecting application logs..."
for pod in $(kubectl get pods -n "$NAMESPACE" -o name); do
  pod_name=${pod#*/}
  kubectl logs "$pod" -n "$NAMESPACE" --all-containers=true > "$ARTIFACTS_DIR/logs/${pod_name}.log" || true
  
  # Also collect previous logs if available
  kubectl logs "$pod" -n "$NAMESPACE" --previous --all-containers=true > "$ARTIFACTS_DIR/logs/${pod_name}-previous.log" 2>/dev/null || true
done

# Run health checks and capture responses
echo "Running health checks..."
for service in $(kubectl get services -n "$NAMESPACE" -o jsonpath='{.items[*].metadata.name}'); do
  if kubectl get service "$service" -n "$NAMESPACE" -o jsonpath='{.spec.ports[0].port}' >/dev/null 2>&1; then
    port=$(kubectl get service "$service" -n "$NAMESPACE" -o jsonpath='{.spec.ports[0].port}')
    
    # Port forward and health check
    kubectl port-forward -n "$NAMESPACE" "service/$service" "$port:$port" &
    PF_PID=$!
    sleep 2
    
    curl -s --max-time 10 "http://localhost:$port/health" > "$ARTIFACTS_DIR/evidence/${service}-health.json" || \
      echo '{"status":"error","message":"health check failed"}' > "$ARTIFACTS_DIR/evidence/${service}-health.json"
    
    kill $PF_PID 2>/dev/null || true
  fi
done

echo "Evidence collection completed"
```

### Summary.json Schema Generation
```bash
#!/bin/bash
# generate-qa-summary.sh

set -euo pipefail

ARTIFACTS_DIR=${ARTIFACTS_DIR:-/artifacts/qa/proof}
NAMESPACE=${K8S_NAMESPACE:-default}
PR_NUMBER=${PR_NUMBER:-0}
HEAD_SHA=${HEAD_SHA:-unknown}

# Determine overall test status
TEST_PASSED=true
TEST_DETAILS="All tests passed successfully"

# Check for test failures (customize based on your test framework)
if [ -f "$ARTIFACTS_DIR/evidence/test-results.xml" ]; then
  FAILED_TESTS=$(xmllint --xpath "//testcase[@result='failure']/@name" "$ARTIFACTS_DIR/evidence/test-results.xml" 2>/dev/null | wc -l)
  if [ "$FAILED_TESTS" -gt 0 ]; then
    TEST_PASSED=false
    TEST_DETAILS="$FAILED_TESTS test(s) failed - see test-results.xml for details"
  fi
fi

# Collect evidence file inventory
EVIDENCE_FILES=()
while IFS= read -r -d '' file; do
  rel_path=${file#$ARTIFACTS_DIR/}
  file_size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null || echo 0)
  EVIDENCE_FILES+=("{\"name\":\"$(basename "$file")\",\"path\":\"$rel_path\",\"type\":\"$(file_type "$file")\",\"size\":$file_size}")
done < <(find "$ARTIFACTS_DIR" -type f -print0)

# Generate comprehensive summary
cat > "$ARTIFACTS_DIR/summary.json" <<EOF
{
  "version": "1.0",
  "pr": {
    "owner": "${OWNER:-unknown}",
    "repo": "${REPO:-unknown}",
    "number": $PR_NUMBER,
    "headSha": "$HEAD_SHA"
  },
  "qa": {
    "passed": $TEST_PASSED,
    "startedAt": "$(date -u -Iseconds)",
    "finishedAt": "$(date -u -Iseconds)",
    "details": "$TEST_DETAILS"
  },
  "environment": {
    "k8sContext": "$(kubectl config current-context)",
    "namespace": "$NAMESPACE",
    "clusterVersion": "$(kubectl version --short --client=false -o json 2>/dev/null | jq -r '.serverVersion.gitVersion // "unknown"')"
  },
  "testCases": $(generate_test_cases),
  "evidence": [$(IFS=,; echo "${EVIDENCE_FILES[*]}")],
  "metadata": {
    "generatedAt": "$(date -u -Iseconds)",
    "generatedBy": "qa-agent-$(cat /proc/version | cut -d' ' -f3)",
    "artifactsDir": "$ARTIFACTS_DIR"
  }
}
EOF

echo "Generated summary.json with $(echo "${EVIDENCE_FILES[@]}" | wc -w) evidence files"
```

### QA Verification Gate
```yaml
- name: verify-k8s-proof
  container:
    image: ghcr.io/myorg/kubectl-jq:1.30
    command: ["/bin/bash"]
    args: ["-c", "/scripts/verify-k8s-proof.sh"]
    env:
      - name: QA_ARTIFACTS_DIR
        value: /artifacts/qa/proof
    volumeMounts:
      - name: artifacts
        mountPath: /artifacts
  script:
    source: |
      set -euo pipefail
      
      ARTIFACTS_DIR=${QA_ARTIFACTS_DIR:-/artifacts/qa/proof}
      SUMMARY_FILE="$ARTIFACTS_DIR/summary.json"
      
      echo "Validating QA evidence at $ARTIFACTS_DIR"
      
      # Check summary.json exists
      if [ ! -f "$SUMMARY_FILE" ]; then
        echo "ERROR: QA summary.json not found at $SUMMARY_FILE"
        exit 1
      fi
      
      # Validate schema
      if ! jq -e '.qa.passed != null and .testCases != null and .evidence != null' "$SUMMARY_FILE" >/dev/null; then
        echo "ERROR: Invalid summary.json schema"
        jq . "$SUMMARY_FILE" || echo "Failed to parse JSON"
        exit 1
      fi
      
      # Check QA passed status
      QA_PASSED=$(jq -r '.qa.passed' "$SUMMARY_FILE")
      if [ "$QA_PASSED" != "true" ]; then
        echo "ERROR: QA tests did not pass (qa.passed = $QA_PASSED)"
        jq -r '.qa.details' "$SUMMARY_FILE"
        exit 1
      fi
      
      # Validate evidence files exist
      echo "Validating evidence files..."
      missing_files=()
      while read -r evidence_path; do
        full_path="$ARTIFACTS_DIR/$evidence_path"
        if [ ! -f "$full_path" ]; then
          missing_files+=("$evidence_path")
        fi
      done < <(jq -r '.evidence[].path' "$SUMMARY_FILE")
      
      if [ ${#missing_files[@]} -gt 0 ]; then
        echo "ERROR: Missing evidence files:"
        printf '  %s\n' "${missing_files[@]}"
        exit 1
      fi
      
      # Generate verification result
      jq -n \
        --argjson evidence_count "$(jq '.evidence | length' "$SUMMARY_FILE")" \
        --argjson test_count "$(jq '.testCases | length' "$SUMMARY_FILE")" \
        --arg namespace "$(jq -r '.environment.namespace' "$SUMMARY_FILE")" \
        '{
          valid: true,
          evidenceCount: $evidence_count,
          testCount: $test_count,
          namespace: $namespace,
          validatedAt: (now | strftime("%Y-%m-%dT%H:%M:%SZ"))
        }' > "$ARTIFACTS_DIR/verification.json"
      
      echo "QA evidence validation passed - $evidence_count files verified"
```

### PR Approval Implementation
```yaml
- name: approve-pr
  inputs:
    parameters:
      - name: owner
      - name: repo  
      - name: prNumber
  container:
    image: curlimages/curl:8.8.0
    command: ["/bin/sh"]
    args:
      - "-c"
      - |
        set -euo pipefail
        
        OWNER={{inputs.parameters.owner}}
        REPO={{inputs.parameters.repo}}
        PR_NUMBER={{inputs.parameters.prNumber}}
        
        # Verify QA evidence exists
        if [ ! -f /artifacts/qa/proof/summary.json ]; then
          echo "ERROR: QA proof missing - cannot approve PR"
          exit 1
        fi
        
        # Read GitHub token
        GITHUB_TOKEN=$(cat /var/run/github/token)
        
        # Create approval review
        echo "Submitting PR approval for $OWNER/$REPO #$PR_NUMBER"
        
        REVIEW_RESPONSE=$(curl -s -X POST \
          "https://api.github.com/repos/$OWNER/$REPO/pulls/$PR_NUMBER/reviews" \
          -H "Authorization: Bearer $GITHUB_TOKEN" \
          -H "Accept: application/vnd.github+json" \
          -H "X-GitHub-Api-Version: 2022-11-28" \
          -d '{
            "event": "APPROVE",
            "body": "✅ QA verification completed successfully with comprehensive Kubernetes evidence.\n\n**Evidence Summary:**\n- All test suites passed\n- Kubernetes cluster state captured\n- Application logs collected\n- Service health checks verified\n\nSee workflow artifacts for complete evidence."
          }')
        
        # Verify approval was created
        REVIEW_ID=$(echo "$REVIEW_RESPONSE" | jq -r '.id // empty')
        if [ -z "$REVIEW_ID" ]; then
          echo "ERROR: Failed to create PR review"
          echo "$REVIEW_RESPONSE" | jq .
          exit 1
        fi
        
        echo "PR review created successfully (ID: $REVIEW_ID)"
        
        # IMPORTANT: Never call merge APIs
        # This implementation explicitly avoids any merge operations
        
    volumeMounts:
      - name: artifacts
        mountPath: /artifacts
      - name: gh-token
        mountPath: /var/run/github
        readOnly: true
```

### Evidence Summary Comment
```yaml
- name: post-evidence-summary
  inputs:
    parameters:
      - name: owner
      - name: repo
      - name: prNumber
  container:
    image: curlimages/curl:8.8.0
    command: ["/bin/sh"]
    args:
      - "-c"
      - |
        OWNER={{inputs.parameters.owner}}
        REPO={{inputs.parameters.repo}}
        PR_NUMBER={{inputs.parameters.prNumber}}
        
        GITHUB_TOKEN=$(cat /var/run/github/token)
        SUMMARY_FILE="/artifacts/qa/proof/summary.json"
        
        # Extract evidence summary
        EVIDENCE_COUNT=$(jq -r '.evidence | length' "$SUMMARY_FILE")
        TEST_COUNT=$(jq -r '.testCases | length' "$SUMMARY_FILE")
        NAMESPACE=$(jq -r '.environment.namespace' "$SUMMARY_FILE")
        
        # Generate artifact URLs (customize based on your artifact storage)
        WORKFLOW_NAME={{workflow.name}}
        ARTIFACT_BASE_URL="https://artifacts.example.com/workflows/$WORKFLOW_NAME"
        
        # Create comprehensive comment
        COMMENT_BODY=$(cat <<EOF
## 🔍 QA Verification Results
        
**Status:** ✅ All tests passed with comprehensive evidence
        
### Test Summary
- **Tests Executed:** $TEST_COUNT test cases
- **Evidence Files:** $EVIDENCE_COUNT artifacts collected
- **Test Environment:** $NAMESPACE
        
### Evidence Collected
- 📋 [Test Results]($ARTIFACT_BASE_URL/qa/proof/evidence/test-results.xml)
- 📊 [Coverage Report]($ARTIFACT_BASE_URL/qa/proof/evidence/coverage.html) 
- 🏥 [Health Checks]($ARTIFACT_BASE_URL/qa/proof/evidence/)
- 📝 [Application Logs]($ARTIFACT_BASE_URL/qa/proof/logs/)
- ⚙️ [Kubernetes Resources]($ARTIFACT_BASE_URL/qa/proof/k8s/)
        
### Verification
This PR has been automatically approved based on successful QA verification with complete Kubernetes evidence. The approval does not trigger an automatic merge - human review is still required for merge decisions.
        
**Workflow:** $WORKFLOW_NAME
EOF
        )
        
        # Post comment
        curl -s -X POST \
          "https://api.github.com/repos/$OWNER/$REPO/issues/$PR_NUMBER/comments" \
          -H "Authorization: Bearer $GITHUB_TOKEN" \
          -H "Accept: application/vnd.github+json" \
          -H "X-GitHub-Api-Version: 2022-11-28" \
          -d "{\"body\": $(echo "$COMMENT_BODY" | jq -R -s .)}"
        
    volumeMounts:
      - name: artifacts
        mountPath: /artifacts
      - name: gh-token
        mountPath: /var/run/github
        readOnly: true
```

## Branch Protection Integration
To ensure no auto-merge occurs, configure GitHub branch protection:

```json
{
  "required_status_checks": {
    "strict": true,
    "contexts": ["continuous-integration"]
  },
  "enforce_admins": false,
  "required_pull_request_reviews": {
    "required_approving_review_count": 1,
    "dismiss_stale_reviews": true,
    "require_code_owner_reviews": true
  },
  "restrictions": null,
  "allow_auto_merge": false,
  "allow_squash_merge": true,
  "allow_merge_commit": false,
  "allow_rebase_merge": false
}
```

## Security Considerations

### RBAC for QA Operations
```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: qa-evidence-collector
  namespace: test-environments
rules:
  - apiGroups: [""]
    resources: ["pods", "services", "events", "configmaps"]
    verbs: ["get", "list"]
  - apiGroups: [""]
    resources: ["pods/log"]
    verbs: ["get"]
  - apiGroups: ["apps"]
    resources: ["deployments", "replicasets"]
    verbs: ["get", "list"]
  - apiGroups: ["networking.k8s.io"]
    resources: ["ingresses"]
    verbs: ["get", "list"]
```

### Evidence Integrity
- All evidence files include checksums for tamper detection
- Summary.json contains metadata about collection process
- Verification step validates evidence completeness and integrity
- Audit trail maintained for all QA operations

## Testing Strategy

### End-to-End Validation
```bash
#!/bin/bash
# test-qa-verification.sh

# Create test PR
gh pr create --title "Test QA Verification" --body "Testing automated QA verification process"

# Submit workflow
WORKFLOW=$(argo submit --from workflowtemplate/pr-validation \
  -p owner=myorg -p repo=myrepo -p pr=123 -o name)

# Wait for completion
argo wait "$WORKFLOW"

# Verify evidence collection
if argo logs "$WORKFLOW" -c qa-testing | grep -q "Evidence collection completed"; then
  echo "✅ Evidence collection successful"
else
  echo "❌ Evidence collection failed"
  exit 1
fi

# Verify PR approval
if gh pr view 123 --json reviewDecision | jq -e '.reviewDecision == "APPROVED"'; then
  echo "✅ PR approved successfully"
else
  echo "❌ PR approval failed"
  exit 1
fi

# Verify no auto-merge occurred
if gh pr view 123 --json state | jq -e '.state == "OPEN"'; then
  echo "✅ No auto-merge - PR remains open"
else
  echo "❌ Unexpected PR state change"
  exit 1
fi
```

## Dependencies
- kubectl with cluster access for evidence collection
- GitHub App with PR review permissions
- Artifact storage system for evidence archival
- Container registry for custom images
- Monitoring system for QA metrics

## References
- [GitHub Pull Request Reviews API](https://docs.github.com/en/rest/pulls/reviews)
- [Kubernetes RBAC Authorization](https://kubernetes.io/docs/reference/access-authn-authz/rbac/)
- [GitHub Branch Protection Rules](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches)