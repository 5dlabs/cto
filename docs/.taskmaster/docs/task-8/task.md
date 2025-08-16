# Task 8: Git Worktrees for Parallel Workspace Isolation

## Overview
This task implements git worktree-based workspace isolation to enable safe parallel execution of independent tasks. The system creates unique worktree directories for each workflow while sharing a common git repository base, providing better performance than full repository clones with proper isolation guarantees.

## Architecture
- **Base Repository**: Shared git repository with no checkout
- **Per-Task Worktrees**: Individual working directories per taskId/PR
- **Optional PVC Isolation**: Complete storage isolation using volumeClaimTemplates
- **Concurrency Controls**: Semaphores and rate limiting for safe parallelization
- **Cleanup Management**: Automatic worktree and PVC cleanup on completion

## Key Features

### Workspace Isolation
- **Git Worktrees**: Separate working directories sharing git object database
- **Unique Paths**: `/work/trees/${TASK_ID}` per workflow execution
- **Branch Checkout**: Each worktree checks out specified ref/branch
- **Conflict Prevention**: No file handle collisions between parallel workflows

### Performance Benefits
- **Shared Objects**: Git object database shared between worktrees
- **Reduced Network**: No duplicate git clone operations
- **Faster Checkout**: Worktree creation vs full clone significantly faster
- **Storage Efficiency**: Shared .git directory reduces storage overhead

## Implementation

### Init Template for Worktree Creation
```yaml
- name: init-worktree
  inputs:
    parameters:
      - {name: owner}
      - {name: repo}
      - {name: ref}
      - {name: taskId}
  script:
    image: alpine/git:2.44.0
    command: [sh, -c]
    source: |
      OWNER={{inputs.parameters.owner}}
      REPO={{inputs.parameters.repo}}
      REF={{inputs.parameters.ref}}
      TASK_ID={{inputs.parameters.taskId}}
      
      set -euxo pipefail
      mkdir -p /work/base /work/trees
      
      # Initialize or update base repository
      if [ ! -d /work/base/.git ]; then
        git clone --no-checkout https://github.com/${OWNER}/${REPO}.git /work/base
      fi
      
      # Fetch latest changes
      git -C /work/base fetch --no-tags --depth=1 origin "$REF"
      
      # Create worktree for this task
      git -C /work/base worktree add "/work/trees/${TASK_ID}" FETCH_HEAD
      
      # Configure git safety
      git config --global --add safe.directory /work/base
      git config --global --add safe.directory "/work/trees/${TASK_ID}"
      
      # Output workspace path for downstream consumption
      echo "/work/trees/${TASK_ID}" > /work/workspace_path
    volumeMounts:
      - name: work
        mountPath: /work
  outputs:
    parameters:
      - name: workspacePath
        valueFrom: {path: /work/workspace_path}
```

### Parameterized Template Integration
```yaml
templates:
- name: coderun-main
  inputs:
    parameters:
      - {name: owner}
      - {name: repo}  
      - {name: ref, value: "refs/heads/main"}
      - {name: taskId, value: "{{=sprig.default(nowEpoch, workflow.parameters.prNumber)}}"}
      - {name: github-app}
      - {name: usePVC, value: "false"}
  dag:
    tasks:
      - name: init
        template: init-worktree
        arguments:
          parameters:
            - {name: owner, value: "{{inputs.parameters.owner}}"}
            - {name: repo, value: "{{inputs.parameters.repo}}"}
            - {name: ref, value: "{{inputs.parameters.ref}}"}
            - {name: taskId, value: "{{inputs.parameters.taskId}}"}
      - name: run
        dependencies: [init]
        template: create-coderun
        arguments:
          parameters:
            - {name: workspacePath, value: "{{tasks.init.outputs.parameters.workspacePath}}"}
            - {name: github-app, value: "{{inputs.parameters.github-app}}"}
```

### PVC Isolation Option
```yaml
# When usePVC=true, use volumeClaimTemplates
volumeClaimTemplates:
- metadata:
    name: work
    labels:
      workflow: "{{workflow.name}}"
  spec:
    accessModes: ["ReadWriteOnce"]
    resources:
      requests:
        storage: 10Gi
    storageClassName: fast-ssd

# When usePVC=false, use emptyDir
volumes:
- name: work
  emptyDir: {}
```

### Concurrency Control Implementation
```yaml
# Per-repo/branch semaphore
synchronization:
  semaphore:
    configMapKeyRef:
      name: workflow-semaphores
      key: "{{=sprig.lower(printf \"%s-%s-%s\" inputs.parameters.owner inputs.parameters.repo (regexReplaceAll \"^refs/(heads|tags)/\" inputs.parameters.ref \"\"))}}"
```

### Semaphore ConfigMap Management
```bash
# Create or update semaphore key
BRANCH_SAFE=$(echo "$REF" | sed -E 's#^refs/(heads|tags)/##; s#[^a-zA-Z0-9_.-]#-#g' | tr '[:upper:]' '[:lower:]')
KEY="${OWNER}-${REPO}-${BRANCH_SAFE}"

kubectl -n argo get configmap workflow-semaphores >/dev/null 2>&1 || \
  kubectl -n argo create configmap workflow-semaphores

kubectl -n argo patch configmap workflow-semaphores --type merge \
  -p "{\"data\":{\"${KEY}\":\"1\"}}"
```

### Cleanup Template
```yaml
- name: cleanup
  inputs:
    parameters:
      - {name: taskId}
      - {name: usePVC}
  script:
    image: alpine/git:2.44.0
    command: [sh, -c] 
    source: |
      TASK_ID={{inputs.parameters.taskId}}
      USE_PVC={{inputs.parameters.usePVC}}
      
      set -euxo pipefail
      
      # Clean up PVCs if using persistent storage
      if [ "$USE_PVC" = "true" ]; then
        kubectl delete pvc -l workflow={{workflow.name}} || true
      fi
      
      # Clean up worktree
      BASE=/work/base
      TREE="/work/trees/${TASK_ID}"
      
      if [ -d "$TREE" ]; then
        git -C "$BASE" worktree remove --force "$TREE" || true
      fi
      
      # Prune stale worktree references
      git -C "$BASE" worktree prune || true
      git -C "$BASE" remote prune origin || true
      git -C "$BASE" gc --prune=now --aggressive || true
    volumeMounts:
      - name: work
        mountPath: /work
```

## Rate Limiting Integration

### Argo Events Sensor Configuration
```yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: repo-workflow-sensor
spec:
  rateLimit:
    unit: minute
    requestsPerUnit: 30
  eventBusName: default
  dependencies:
    - name: github-pr
      eventSourceName: github
      eventName: pull_request
      filters:
        expr:
          # Debounce: 15s window per repo/ref
          expression: "debounce(event.repo.full_name + ':' + event.body.pull_request.base.ref, duration('15s'))"
  triggers:
    - template:
        name: submit-workflow
        k8s:
          operation: create
          source:
            resource:
              apiVersion: argoproj.io/v1alpha1
              kind: Workflow
              spec:
                workflowTemplateRef:
                  name: coderun-template
                arguments:
                  parameters:
                    - name: taskId
                      value: "pr-{{(jsonpath \"$.body.pull_request.number\")}}-{{(jsonpath \"$.headers.X-GitHub-Delivery\")}}"
```

## Performance Testing

### Worktree vs Clone Benchmarking
```bash
# Benchmark worktree creation
time_worktree() {
  local repo=$1 ref=$2 task_id=$3
  
  start_time=$(date +%s%N)
  
  git -C /work/base fetch --depth=1 origin "$ref"
  git -C /work/base worktree add "/work/trees/$task_id" FETCH_HEAD
  
  end_time=$(date +%s%N)
  echo $((($end_time - $start_time) / 1000000)) # milliseconds
}

# Benchmark full clone
time_clone() {
  local repo=$1 ref=$2 task_id=$3
  
  start_time=$(date +%s%N)
  
  git clone --depth=1 --branch "$ref" "https://github.com/$repo.git" "/tmp/clone-$task_id"
  
  end_time=$(date +%s%N)
  echo $((($end_time - $start_time) / 1000000)) # milliseconds
}

# Run benchmarks
for i in $(seq 1 10); do
  WORKTREE_TIME=$(time_worktree "myorg/myrepo" "main" "bench-$i")
  CLONE_TIME=$(time_clone "myorg/myrepo" "main" "bench-$i")
  echo "Iteration $i: Worktree ${WORKTREE_TIME}ms, Clone ${CLONE_TIME}ms"
done
```

## Parallel Execution Testing
```bash
# Test N parallel workflows
test_parallel_execution() {
  local n=$1
  local repo="myorg/myrepo"
  local ref="refs/heads/main"
  
  echo "Testing $n parallel workflows"
  
  # Submit N workflows simultaneously
  for i in $(seq 1 $n); do
    argo submit --from workflowtemplate/coderun-template \
      -p owner=myorg \
      -p repo=myrepo \
      -p ref="$ref" \
      -p taskId="parallel-test-$i-$(date +%s)" \
      -p github-app=rex &
  done
  
  wait
  echo "All $n workflows submitted"
}

# Test with semaphore limit
kubectl -n argo patch configmap workflow-semaphores --type merge \
  -p '{"data":{"myorg-myrepo-main":"1"}}'

test_parallel_execution 20
```

## Monitoring and Observability

### Key Metrics
- Worktree creation time vs full clone time
- Parallel execution success rates
- Semaphore queue depths and wait times
- Storage utilization (shared vs isolated)
- Cleanup success rates

### Monitoring Implementation
```go
var (
    worktreeCreationDuration = prometheus.NewHistogramVec(
        prometheus.HistogramOpts{
            Name: "git_worktree_creation_duration_seconds",
            Help: "Time taken to create git worktrees",
        },
        []string{"repo", "ref"},
    )
    
    parallelWorkflowsActive = prometheus.NewGaugeVec(
        prometheus.GaugeOpts{
            Name: "parallel_workflows_active",
            Help: "Number of active parallel workflows per repo",
        },
        []string{"repo", "ref"},
    )
)
```

## Security Considerations

### Access Control
- Git operations run as non-root user
- Read-only access to base repository for most operations
- Semaphore keys sanitized to prevent injection
- Network policies restrict git protocol access

### Resource Limits
```yaml
resources:
  requests:
    cpu: 100m
    memory: 256Mi
  limits:
    cpu: 1
    memory: 2Gi
    ephemeral-storage: 5Gi
```

## Troubleshooting Guide

### Common Issues

#### Git Safe Directory Errors
```bash
# Fix: Add directories to git safe list
git config --global --add safe.directory /work/base
git config --global --add safe.directory "/work/trees/${TASK_ID}"
```

#### Worktree Creation Failures
```bash
# Debug: Check base repository status
git -C /work/base status
git -C /work/base worktree list

# Fix: Prune stale worktrees
git -C /work/base worktree prune
```

#### Semaphore Key Issues
```bash
# Debug: Check ConfigMap contents
kubectl get configmap workflow-semaphores -o yaml

# Fix: Update semaphore key format
KEY=$(echo "${OWNER}-${REPO}-${REF}" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9.-]/-/g')
```

## Dependencies
- Alpine/git container image with git 2.44+
- Kubernetes cluster with PVC support (if using persistent storage)
- Argo Workflows with semaphore support
- ConfigMap workflow-semaphores for concurrency control
- Network access to GitHub for git operations

## References
- [Git Worktree Documentation](https://git-scm.com/docs/git-worktree)
- [Argo Workflows Synchronization](https://argoproj.github.io/argo-workflows/synchronization/)
- [Kubernetes Volume Management](https://kubernetes.io/docs/concepts/storage/volumes/)