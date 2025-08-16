# Toolman Guide: QA Kubernetes Verification and PR Approval

## Overview
Guide for implementing and operating QA verification system with evidence collection and automated PR approval workflow.

## Core Components

### QA Evidence Collection
```bash
# Manual evidence collection
./scripts/collect-qa-evidence.sh \
  --namespace qa-testing \
  --output-dir /artifacts/qa/proof \
  --repo myorg/repo \
  --pr 123

# Automated collection in workflow
env:
  ARTIFACTS_DIR: "/artifacts/qa/proof"
  K8S_NAMESPACE: "qa-testing"
  OWNER: "myorg"
  REPO: "repo"
  PR_NUMBER: "123"
```

### Evidence Directory Structure
```
/artifacts/qa/proof/
├── summary.json          # Required schema file
├── logs/
│   ├── pod1-container.log
│   └── pod2-container.log
├── k8s/
│   ├── pods.yaml
│   ├── deployments.yaml
│   ├── events.yaml
│   └── services.yaml
└── http/
    ├── health.json
    └── health.meta
```

### Summary.json Schema
```json
{
  "version": "1.0",
  "pr": {
    "owner": "myorg",
    "repo": "repo", 
    "number": 123,
    "headSha": "abc123def456"
  },
  "qa": {
    "passed": true,
    "startedAt": "2024-01-01T12:00:00Z",
    "finishedAt": "2024-01-01T12:05:00Z"
  },
  "environment": {
    "k8sContext": "qa-cluster",
    "namespace": "qa-testing"
  },
  "artifacts": [
    {
      "name": "pods",
      "path": "k8s/pods.yaml",
      "contentType": "application/yaml",
      "bytes": 1024,
      "url": "https://s3.amazonaws.com/bucket/path/pods.yaml"
    }
  ],
  "evidence": {
    "k8s": {"resources": ["pods","deployments","events","services"]},
    "logs": [{"pod": "app-pod", "container": "main", "path": "logs/app-pod-main.log"}],
    "httpChecks": [{"name": "health", "url": "http://app/health", "status": 200, "latencyMs": 45}]
  }
}
```

## Workflow Integration

### QA Agent System Prompt
```yaml
# Enhanced qa-testing template
spec:
  templates:
  - name: qa-testing
    script:
      image: qa-agent:latest
      source: |
        #!/bin/bash
        
        # Set up evidence collection
        mkdir -p "$ARTIFACTS_DIR"/{logs,k8s,http}
        
        # Run QA tests and collect evidence
        python /app/qa-agent.py \
          --collect-evidence \
          --output-dir "$ARTIFACTS_DIR" \
          --namespace "$K8S_NAMESPACE"
        
        # Generate summary.json
        ./scripts/generate-summary.sh \
          --status $? \
          --output "$ARTIFACTS_DIR/summary.json"
```

### Verification Gates
```yaml
# verify-k8s-proof step
- name: verify-k8s-proof
  dependencies: [qa-testing]
  script:
    image: qa-validator:latest
    source: |
      #!/bin/bash
      SUMMARY="$ARTIFACTS_DIR/summary.json"
      
      # Validate file exists
      test -f "$SUMMARY" || { echo "Missing summary.json"; exit 1; }
      
      # Validate schema and qa.passed
      jq -e '.qa.passed == true and .pr.number and .environment.namespace' \
        "$SUMMARY" > /dev/null || exit 1
      
      # Validate referenced artifacts exist
      jq -r '.artifacts[].path' "$SUMMARY" | while read path; do
        test -f "$ARTIFACTS_DIR/$path" || { echo "Missing $path"; exit 1; }
      done
```

### PR Approval Step
```yaml
# approve-pr step
- name: approve-pr
  dependencies: [verify-compliance, verify-k8s-proof]
  script:
    image: github-client:latest
    source: |
      #!/bin/bash
      
      # Verify summary exists
      test -f "$SUMMARY_PATH" || { echo "No QA proof"; exit 1; }
      
      # Post APPROVE review
      curl -sS -X POST \
        "https://api.github.com/repos/$OWNER/$REPO/pulls/$PR_NUMBER/reviews" \
        -H "Authorization: Bearer $GITHUB_TOKEN" \
        -H "Accept: application/vnd.github+json" \
        -H "X-GitHub-Api-Version: 2022-11-28" \
        -d '{"event":"APPROVE","body":"QA verification passed with evidence."}'
```

## Evidence Collection Tools

### Kubernetes Resource Collection
```bash
# Collect pod information
kubectl -n "$K8S_NAMESPACE" get pods -o yaml > "$ARTIFACTS_DIR/k8s/pods.yaml"

# Collect deployments 
kubectl -n "$K8S_NAMESPACE" get deployments -o yaml > "$ARTIFACTS_DIR/k8s/deployments.yaml"

# Collect recent events
kubectl -n "$K8S_NAMESPACE" get events \
  --sort-by=.lastTimestamp \
  -o yaml > "$ARTIFACTS_DIR/k8s/events.yaml"

# Collect services
kubectl -n "$K8S_NAMESPACE" get services -o yaml > "$ARTIFACTS_DIR/k8s/services.yaml"
```

### Log Collection
```bash
# Collect logs from all pods
for pod in $(kubectl -n "$K8S_NAMESPACE" get pods -o name); do
  pod_name="${pod##*/}"
  kubectl -n "$K8S_NAMESPACE" logs "$pod" --all-containers=true \
    > "$ARTIFACTS_DIR/logs/${pod_name}.log" 2>/dev/null || true
done
```

### HTTP Health Checks
```bash
# Health check with timing
curl -sS -w '%{http_code} %{time_total}\n' \
  "$HEALTH_URL" \
  -o "$ARTIFACTS_DIR/http/health.json" \
  > "$ARTIFACTS_DIR/http/health.meta" 2>&1
```

## Object Store Integration

### S3 Upload Configuration
```bash
# Environment variables
export OBJECT_STORE_UPLOAD="true"
export OBJECT_STORE_PROVIDER="s3"
export OBJECT_STORE_BUCKET="qa-evidence"
export OBJECT_STORE_PREFIX="pr-$PR_NUMBER"
export AWS_ACCESS_KEY_ID="..."
export AWS_SECRET_ACCESS_KEY="..."
export AWS_REGION="us-east-1"

# Upload artifacts
aws s3 cp "$ARTIFACTS_DIR" \
  "s3://$OBJECT_STORE_BUCKET/$OBJECT_STORE_PREFIX/" \
  --recursive

# Generate presigned URLs
aws s3 presign \
  "s3://$OBJECT_STORE_BUCKET/$OBJECT_STORE_PREFIX/summary.json" \
  --expires-in 604800  # 7 days
```

### GCS Upload Configuration  
```bash
# Environment variables
export OBJECT_STORE_PROVIDER="gcs"
export OBJECT_STORE_BUCKET="qa-evidence"
export GCS_SERVICE_ACCOUNT_JSON="/secrets/gcs-key.json"

# Upload artifacts
gsutil -m cp -r "$ARTIFACTS_DIR/*" \
  "gs://$OBJECT_STORE_BUCKET/$OBJECT_STORE_PREFIX/"

# Generate signed URLs
gsutil signurl -d 7d "$GCS_SERVICE_ACCOUNT_JSON" \
  "gs://$OBJECT_STORE_BUCKET/$OBJECT_STORE_PREFIX/summary.json"
```

## PR Comment Generation

### Comment Template
```bash
# Generate PR comment
cat > /tmp/pr-comment.json << EOF
{
  "body": "## QA Verification Results\n\n**Status:** $(jq -r '.qa.passed' $SUMMARY)\n**Namespace:** $(jq -r '.environment.namespace' $SUMMARY)\n**Artifacts:** $(jq '.artifacts | length' $SUMMARY) files\n\n### Evidence Links\n$(jq -r '.artifacts[] | \"- [\" + .name + \"](\" + .url + \")\"' $SUMMARY)\n\n**Started:** $(jq -r '.qa.startedAt' $SUMMARY)\n**Finished:** $(jq -r '.qa.finishedAt' $SUMMARY)"
}
EOF

# Post comment
curl -sS -X POST \
  "https://api.github.com/repos/$OWNER/$REPO/issues/$PR_NUMBER/comments" \
  -H "Authorization: Bearer $GITHUB_TOKEN" \
  -H "Accept: application/vnd.github+json" \
  -d @/tmp/pr-comment.json
```

## Troubleshooting

### Common Issues

#### Missing Evidence Files
```bash
# Check evidence collection
ls -la "$ARTIFACTS_DIR"/{logs,k8s,http}/

# Validate summary.json
jq '.' "$ARTIFACTS_DIR/summary.json" || echo "Invalid JSON"

# Check required fields
jq -e '.qa.passed and .pr.number' "$ARTIFACTS_DIR/summary.json"
```

#### GitHub API Failures
```bash
# Test token validity
curl -H "Authorization: Bearer $GITHUB_TOKEN" \
  https://api.github.com/user

# Check PR existence
curl -H "Authorization: Bearer $GITHUB_TOKEN" \
  "https://api.github.com/repos/$OWNER/$REPO/pulls/$PR_NUMBER"

# Validate review permissions
curl -H "Authorization: Bearer $GITHUB_TOKEN" \
  "https://api.github.com/repos/$OWNER/$REPO/pulls/$PR_NUMBER/reviews"
```

#### Kubernetes Access Issues
```bash
# Check namespace access
kubectl auth can-i get pods -n "$K8S_NAMESPACE"

# Verify pod existence
kubectl -n "$K8S_NAMESPACE" get pods

# Test log access
kubectl -n "$K8S_NAMESPACE" logs deployment/app --tail=10
```

### Debug Commands

#### Evidence Validation
```bash
# Validate all artifacts referenced in summary
./scripts/validate-qa-proof.sh \
  --summary-path "$ARTIFACTS_DIR/summary.json" \
  --strict

# Check file sizes and formats
find "$ARTIFACTS_DIR" -type f -exec file {} \; | head -10
```

#### Branch Protection Status
```bash
# Check branch protection (requires admin token)
curl -H "Authorization: Bearer $ADMIN_TOKEN" \
  "https://api.github.com/repos/$OWNER/$REPO/branches/main/protection"

# List required status checks
curl -H "Authorization: Bearer $ADMIN_TOKEN" \
  "https://api.github.com/repos/$OWNER/$REPO/branches/main/protection/required_status_checks"
```

#### Object Store Access
```bash
# Test S3 connectivity
aws s3 ls "s3://$OBJECT_STORE_BUCKET/" --region "$AWS_REGION"

# Test GCS connectivity  
gsutil ls "gs://$OBJECT_STORE_BUCKET/"

# Validate presigned URL
curl -I "$PRESIGNED_URL"
```

## Best Practices

### Security
- Use GitHub App tokens with minimal required permissions
- Store object store credentials in Kubernetes secrets
- Implement artifact retention policies
- Audit evidence access patterns

### Performance
- Limit log collection to recent entries
- Compress artifacts before upload
- Use parallel collection where possible  
- Set reasonable timeouts for HTTP checks

### Reliability
- Implement retry logic for API calls
- Handle partial evidence collection gracefully
- Provide fallback paths when object store unavailable
- Monitor evidence collection success rates