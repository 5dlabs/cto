# Toolman Guide: Task Association Validation System



## Overview

This guide provides comprehensive instructions for using tools and utilities required for implementing and managing the Task Association Validation System. The system uses multiple tool categories to ensure robust validation between GitHub workflows and Task Master tasks.



## Tool Categories

### 1. Workflow Orchestration Tools



#### Argo Workflows (`argo_workflows`)

**Purpose**: Manage validation workflow execution and templates

**Key Commands**:



```bash
# Submit validation workflow
argo submit task-validation-template.yaml \


  --parameter pr-number=123 \
  --parameter pr-labels='[{"name":"task-26"}]' \


  --parameter branch-ref="task-26-fix-validation"

# Monitor workflow progress
argo get validation-26-abc123



# View workflow logs
argo logs validation-26-abc123

# List all validation workflows
argo list --label taskmaster.io/type=validation






```

**Best Practices**:


- Use workflow parameters for dynamic values


- Implement proper resource limits


- Add meaningful labels for filtering


- Use structured logging for debugging

#### Argo Events (`argo_events`)

**Purpose**: Configure sensors for GitHub webhook processing

**Key Commands**:



```bash
# Apply sensor configuration
kubectl apply -f task-validation-sensor.yaml

# Check sensor status
kubectl get sensors task-validation-sensor -o yaml

# View sensor logs
kubectl logs -f sensor-controller-manager-xxx -n argo-events



# Test webhook delivery
curl -X POST http://sensor-endpoint/webhook \
  -H "Content-Type: application/json" \


  -d @test-pr-payload.json






```

**Configuration Example**:



```yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: task-validation-sensor
spec:
  dependencies:
  - name: github-pr
    eventSourceName: github-eventsource
    eventName: pull-request
  triggers:
  - template:
      name: validate-task-association
      argoWorkflow:
        operation: submit
        source:
          resource:
            apiVersion: argoproj.io/v1alpha1
            kind: Workflow






```

### 2. GitHub Integration Tools



#### GitHub API (`github_api`)

**Purpose**: Interact with GitHub for PR validation and comments

**Authentication Setup**:



```bash
# Create GitHub token secret
kubectl create secret generic github-token \


  --from-literal=token="ghp_xxxxxxxxxxxxxxxxxxxx" \


  --namespace=taskmaster

# Verify token permissions
curl -H "Authorization: token $GITHUB_TOKEN" \
  https://api.github.com/user






```

**Common Operations**:



```bash
# Get PR information
curl -H "Authorization: token $GITHUB_TOKEN" \
  https://api.github.com/repos/5dlabs/cto/pulls/123

# Create PR comment
curl -X POST \
  -H "Authorization: token $GITHUB_TOKEN" \
  -d '{"body":"Validation failed - check task labels"}' \
  https://api.github.com/repos/5dlabs/cto/issues/123/comments



# List PR labels
curl -H "Authorization: token $GITHUB_TOKEN" \
  https://api.github.com/repos/5dlabs/cto/issues/123/labels






```

**Error Handling**:


- Implement exponential backoff for rate limits


- Check HTTP status codes before processing responses


- Use conditional requests to minimize API usage


- Cache responses when appropriate

### 3. Data Processing Tools

#### JSON Processor (`json_processor`)

**Purpose**: Parse webhook payloads and extract task IDs

**Installation**:



```bash
# Install jq
apt-get update && apt-get install -y jq

# Verify installation
jq --version






```

**Task ID Extraction Patterns**:



```bash


# Extract from PR labels
echo '$PR_PAYLOAD' | jq -r '
  .pull_request.labels[] |
  select(.name | startswith("task-")) |
  .name |
  split("-")[1]
' | head -1

# Extract branch reference
echo '$PR_PAYLOAD' | jq -r '.pull_request.head.ref'



# Parse marker file
cat docs/.taskmaster/current-task.json | jq -r '.task_id'



# Validate JSON schema
echo '$JSON' | jq -e 'has("task_id") and has("started_at") and has("agent")'






```

**Advanced JQ Patterns**:



```bash
# Filter and transform labels
jq '.pull_request.labels | map(select(.name | test("^task-\\d+$")))'



# Extract all task IDs
jq -r '.pull_request.labels[] |
       select(.name | startswith("task-")) |
       .name |
       capture("task-(?<id>\\d+)") |
       .id'

# Conditional extraction
jq -r 'if .pull_request.labels then
         (.pull_request.labels[] | select(.name | startswith("task-")).name)
       else
         empty
       end'






```



#### Regex Processor (`regex_processor`)

**Purpose**: Parse branch names and validate patterns

**Branch Name Validation**:



```bash
# Test branch patterns
echo "task-26-fix-bug" | grep -E "^task-([0-9]+).*$"
echo "feature/task-26" | grep -E "^feature/task-([0-9]+)$"

# Extract task ID using sed
echo "task-26-fix-validation" | sed -n 's/^.*task-\([0-9]\+\).*$/\1/p'

# Validate complex patterns
branch_regex="^(?:feature/)?task-([0-9]+)(?:-.*)?$"
echo "$BRANCH_NAME" | grep -P "$branch_regex"






```

**Pattern Testing**:



```bash
# Test various branch formats
branches=(
  "task-26"
  "task-26-fix-bug"
  "feature/task-26"
  "hotfix/task-15-urgent"
  "invalid-branch"
)

for branch in "${branches[@]}"; do
  if echo "$branch" | grep -qE "^(feature/)?task-([0-9]+)"; then
    id=$(echo "$branch" | sed -n 's/^.*task-\([0-9]\+\).*$/\1/p')
    echo "$branch -> $id"
  else
    echo "$branch -> INVALID"
  fi
done






```

### 4. Git Operations Tools

#### Git Operations (`git_operations`)

**Purpose**: Manage marker files and repository operations

**Marker File Management**:



```bash


# Create marker file
cat > docs/.taskmaster/current-task.json <<EOF
{
  "task_id": "26",
  "started_at": "$(date -Iseconds)",
  "agent": "rex",
  "workflow_id": "$WORKFLOW_NAME",
  "commit_sha": "$(git rev-parse HEAD)"
}
EOF

# Stage and commit
git add docs/.taskmaster/current-task.json
git commit -m "chore: Set current task marker for task-26"

# Push changes
git push origin HEAD






```

**Repository Operations**:



```bash


# Check git status
git status --porcelain

# Verify clean working tree
if [[ -z "$(git status --porcelain)" ]]; then
  echo "Working tree is clean"
else
  echo "Working tree has changes"
fi

# Get current commit
COMMIT_SHA=$(git rev-parse HEAD)
SHORT_SHA=$(git rev-parse --short HEAD)

# Create branch for marker file
git checkout -b "marker/task-26-$(date +%s)"






```

### 5. Monitoring Tools



#### Prometheus (`prometheus`)

**Purpose**: Collect validation metrics and monitoring data

**Metric Collection**:



```bash
# Push validation metrics
cat <<EOF | curl -X POST --data-binary @- \
  http://pushgateway:9091/metrics/job/task-validation
task_validation_total{status="success",task_id="26"} 1
task_validation_duration_seconds{task_id="26"} 15.2
task_validation_errors_total{type="branch_mismatch"} 0
EOF

# Query validation metrics
curl "http://prometheus:9090/api/v1/query?query=task_validation_total"

# Check validation error rate
curl "http://prometheus:9090/api/v1/query?query=rate(task_validation_errors_total[5m])"






```

**Custom Metrics**:
- `task_validation_total`: Counter for validation attempts
- `task_validation_duration_seconds`: Histogram for validation latency
- `task_validation_errors_total`: Counter for validation errors by type
- `task_marker_files_created_total`: Counter for marker file creation

### 6. Kubernetes Integration Tools

#### Kubernetes API (`kubernetes_api`)

**Purpose**: Manage secrets, configmaps, and workflow resources

**Secret Management**:



```bash
# Create validation secrets
kubectl create secret generic task-validation-config \


  --from-literal=github-token="$GITHUB_TOKEN" \


  --from-literal=webhook-secret="$WEBHOOK_SECRET"



# Update secret
kubectl patch secret task-validation-config \
  -p='{"data":{"github-token":"'$(echo -n "$NEW_TOKEN" | base64)'"}}'

# Mount secrets in workflow
kubectl apply -f - <<EOF
apiVersion: v1
kind: Secret
metadata:
  name: task-validation-secret
data:
  github-token: $(echo -n "$GITHUB_TOKEN" | base64)
  webhook-secret: $(echo -n "$WEBHOOK_SECRET" | base64)
EOF






```

**Workflow Management**:



```bash


# Get workflow status
kubectl get workflow validation-26-abc123 -o jsonpath='{.status.phase}'



# Watch workflow progress
kubectl get workflow validation-26-abc123 -w



# Delete completed workflows
kubectl delete workflow -l taskmaster.io/cleanup=true

# Scale workflow controller
kubectl scale deployment workflow-controller --replicas=3






```



## Best Practices

### Tool Configuration

1. **Environment Setup**:



```bash
# Set up tool environment
export GITHUB_TOKEN="ghp_xxxxxxxxxxxxxxxxxxxx"
export ARGO_SERVER="argo-server.taskmaster.local:2746"
export PROMETHEUS_URL="http://prometheus:9090"

# Verify tool connectivity
argo version
jq --version
git --version
kubectl version --client






```

2. **Error Handling**:



```bash
# Implement robust error checking
set -euo pipefail

# Function for API calls with retry
api_call_with_retry() {
  local url="$1"
  local max_retries=3
  local retry=0

  while [[ $retry -lt $max_retries ]]; do
    if curl -f -s "$url"; then
      return 0
    fi
    ((retry++))
    sleep $((2**retry))
  done

  echo "API call failed after $max_retries retries" >&2
  return 1
}






```

3. **Security Practices**:



```bash
# Never log sensitive data
echo "Processing PR #$PR_NUMBER" # OK
echo "GitHub token: $GITHUB_TOKEN" # NEVER DO THIS



# Use secure temporary files
temp_file=$(mktemp)
trap "rm -f $temp_file" EXIT

# Validate inputs
validate_task_id() {
  local task_id="$1"
  if [[ ! "$task_id" =~ ^[0-9]+$ ]]; then
    echo "Invalid task ID: $task_id" >&2
    return 1
  fi
}






```

### Performance Optimization

1. **Parallel Processing**:



```bash
# Extract all validation methods in parallel
extract_label_id &
extract_branch_id &
extract_marker_id &
wait



# Batch API calls
curl_multi_requests() {
  curl -s -H "Authorization: token $GITHUB_TOKEN" \
    "https://api.github.com/repos/5dlabs/cto/pulls/123" &
  curl -s -H "Authorization: token $GITHUB_TOKEN" \
    "https://api.github.com/repos/5dlabs/cto/issues/123/labels" &
  wait
}






```

2. **Caching Strategies**:



```bash
# Cache GitHub API responses
CACHE_DIR="/tmp/github-cache"
cache_key="pr-$PR_NUMBER-$(date +%H)"
cache_file="$CACHE_DIR/$cache_key"

if [[ -f "$cache_file" && $(($(date +%s) - $(stat -c %Y "$cache_file"))) -lt 300 ]]; then
  cat "$cache_file"
else
  github_api_call | tee "$cache_file"
fi






```

### Troubleshooting

#### Common Issues

1. **Validation Failures**:



```bash
# Debug task ID extraction
echo "PR Labels: $(jq -r '.pull_request.labels[].name' <<< "$WEBHOOK_PAYLOAD")"
echo "Branch: $(jq -r '.pull_request.head.ref' <<< "$WEBHOOK_PAYLOAD")"
echo "Marker: $(cat docs/.taskmaster/current-task.json 2>/dev/null || echo 'not found')"

# Test regex patterns
branch="task-26-fix-bug"
if [[ "$branch" =~ ^(feature/)?task-([0-9]+) ]]; then
  echo "Extracted ID: ${BASH_REMATCH[2]}"
else
  echo "No match for branch: $branch"
fi






```

2. **GitHub API Issues**:



```bash


# Check API rate limits
curl -s -I -H "Authorization: token $GITHUB_TOKEN" \
  https://api.github.com/rate_limit | grep -E "X-RateLimit"



# Test webhook payload
curl -X POST http://localhost:8080/webhook \
  -H "Content-Type: application/json" \
  -H "X-GitHub-Event: pull_request" \


  -d @test-pr-payload.json






```

3. **Workflow Debug**:



```bash


# Get detailed workflow status
argo get validation-26-abc123 -o yaml



# Check workflow logs
argo logs validation-26-abc123 --follow



# Debug failed steps
kubectl describe workflow validation-26-abc123






```

### Monitoring and Alerting

1. **Health Checks**:



```bash


# Validate system health
check_validation_system() {
  # Check Argo Workflows
  if ! argo version >/dev/null 2>&1; then
    echo "ERROR: Argo Workflows unavailable"
    return 1
  fi

  # Check GitHub API
  if ! curl -f -s -H "Authorization: token $GITHUB_TOKEN" \
       https://api.github.com/rate_limit >/dev/null; then
    echo "ERROR: GitHub API unavailable"
    return 1
  fi

  # Check Prometheus
  if ! curl -f -s "$PROMETHEUS_URL/api/v1/query?query=up" >/dev/null; then
    echo "ERROR: Prometheus unavailable"
    return 1
  fi

  echo "All systems operational"
  return 0
}






```

2. **Performance Monitoring**:



```bash
# Track validation performance
start_time=$(date +%s.%N)
# ... validation logic ...
end_time=$(date +%s.%N)
duration=$(echo "$end_time - $start_time" | bc)



# Report metrics
echo "task_validation_duration_seconds{task_id=\"$TASK_ID\"} $duration" | \
  curl -X POST --data-binary @- \
  http://pushgateway:9091/metrics/job/task-validation






```

This guide provides comprehensive coverage of all tools required for implementing and maintaining the Task Association Validation System. Follow these patterns and best practices to ensure robust, secure, and maintainable validation workflows.
