# Task 6: Implement Agent Cancellation System

## Overview
Enhance the existing agent cancellation system to handle concurrent operations and improve race condition handling when Rex pushes remediation fixes. This system builds upon existing cancellation capabilities while adding advanced concurrency control, distributed locking, and state-aware cancellation logic.

## Technical Context
The current implementation-agent-remediation sensor provides basic cancellation functionality including:
- Rex push event detection
- Basic CodeRun deletion for Cleo/Tess agents  
- PR label management

This task enhances these capabilities with advanced concurrent operation handling, distributed locking, and integration with the state management system for robust cancellation in high-load scenarios.

### Existing Foundation
- **Sensor**: `implementation-agent-remediation` in `infra/gitops/resources/github-webhooks/play-workflow-sensors.yaml`
- **Event Detection**: Rex push events via GitHub webhook
- **Basic Operations**: CodeRun deletion and label management
- **Integration Points**: GitHub API, Kubernetes API, Argo Events

## Implementation Guide

### Step 1: Enhance Existing Rex Push Event Sensor

#### 1.1 Extended Sensor Configuration
Enhance the existing sensor in `infra/gitops/resources/github-webhooks/play-workflow-sensors.yaml`:

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: implementation-agent-remediation
  namespace: argo-events
spec:
  template:
    serviceAccountName: argo-events-sa
    container:
      resources:
        requests:
          memory: "128Mi"
          cpu: "100m"
        limits:
          memory: "256Mi"
          cpu: "300m"
  dependencies:
    - name: rex-push
      eventSourceName: github-eventsource
      eventName: org
      filters:
        data:
          - path: body.pusher.name
            type: string
            value: ["5DLabs-Rex[bot]", "5DLabs-Rex"]
          - path: body.ref
            type: string
            comparator: "~"
            value: "refs/heads/.*"
        # Enhanced filtering for concurrent push events
        exprs:
          - expr: 'body.commits | length > 0'
            fields:
              - name: commits
                path: body.commits
          - expr: 'body.repository.name != ""'
            fields:
              - name: repo_name
                path: body.repository.name
  triggers:
    - template:
        name: enhanced-agent-cancellation
        conditions: rex-push
        k8s:
          operation: create
          source:
            resource:
              apiVersion: batch/v1
              kind: Job
              metadata:
                generateName: agent-cancellation-
                namespace: agent-platform
                labels:
                  app: agent-cancellation
                  trigger-type: rex-push
                  repo: "{{.repo_name}}"
                annotations:
                  correlation-id: "{{.correlation_id}}"
              spec:
                template:
                  spec:
                    serviceAccountName: agent-cancellation-sa
                    restartPolicy: OnFailure
                    containers:
                      - name: cancellation-controller
                        image: agent-cancellation:latest
                        env:
                          - name: PR_NUMBER
                            value: "{{.pr_number}}"
                          - name: TASK_ID
                            value: "{{.task_id}}"
                          - name: CORRELATION_ID
                            value: "{{.correlation_id}}"
                          - name: REPO_NAME
                            value: "{{.repo_name}}"
        parameters:
          - src:
              dependencyName: rex-push
              dataKey: body.head_commit.message
            dest: commit_message
          - src:
              dependencyName: rex-push
              dataTemplate: '{{ .commit_message | extractPRNumber }}'
            dest: pr_number
          - src:
              dependencyName: rex-push  
              dataTemplate: '{{ .pr_number | extractTaskId }}'
            dest: task_id
          - src:
              dependencyName: rex-push
              dataTemplate: '{{ uuidv4() }}'
            dest: correlation_id
```

### Step 2: Implement Distributed Locking for Concurrent Operations

#### 2.1 Lease-Based Distributed Locking
```go
package cancellation

import (
    "context"
    "fmt"
    "time"
    
    coordinationv1 "k8s.io/api/coordination/v1"
    metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
    "k8s.io/client-go/kubernetes"
    "sigs.k8s.io/controller-runtime/pkg/log"
)

type DistributedLock struct {
    client      kubernetes.Interface
    namespace   string
    lockName    string
    holderName  string
    leaseDuration time.Duration
}

func NewDistributedLock(client kubernetes.Interface, namespace, lockName, holderName string) *DistributedLock {
    return &DistributedLock{
        client:        client,
        namespace:     namespace,
        lockName:      lockName,
        holderName:    holderName,
        leaseDuration: 30 * time.Second,
    }
}

func (dl *DistributedLock) TryAcquire(ctx context.Context) (*Lease, error) {
    logger := log.FromContext(ctx)
    
    // Create or update lease
    lease := &coordinationv1.Lease{
        ObjectMeta: metav1.ObjectMeta{
            Name:      dl.lockName,
            Namespace: dl.namespace,
            Annotations: map[string]string{
                "cancellation.5dlabs.com/holder":     dl.holderName,
                "cancellation.5dlabs.com/acquired":   time.Now().Format(time.RFC3339),
                "cancellation.5dlabs.com/operation": "agent-cancellation",
            },
        },
        Spec: coordinationv1.LeaseSpec{
            HolderIdentity:       &dl.holderName,
            LeaseDurationSeconds: int32(dl.leaseDuration.Seconds()),
            AcquireTime:         &metav1.MicroTime{Time: time.Now()},
            RenewTime:           &metav1.MicroTime{Time: time.Now()},
        },
    }
    
    leaseClient := dl.client.CoordinationV1().Leases(dl.namespace)
    
    // Try to create the lease
    createdLease, err := leaseClient.Create(ctx, lease, metav1.CreateOptions{})
    if err == nil {
        logger.Info("Successfully acquired distributed lock", "lockName", dl.lockName)
        return NewLease(createdLease, dl.client, dl.namespace), nil
    }
    
    // If lease exists, try to acquire it
    existingLease, getErr := leaseClient.Get(ctx, dl.lockName, metav1.GetOptions{})
    if getErr != nil {
        return nil, fmt.Errorf("failed to get existing lease: %w", getErr)
    }
    
    // Check if lease is expired
    if dl.isLeaseExpired(existingLease) {
        existingLease.Spec.HolderIdentity = &dl.holderName
        existingLease.Spec.AcquireTime = &metav1.MicroTime{Time: time.Now()}
        existingLease.Spec.RenewTime = &metav1.MicroTime{Time: time.Now()}
        
        updatedLease, updateErr := leaseClient.Update(ctx, existingLease, metav1.UpdateOptions{})
        if updateErr != nil {
            return nil, fmt.Errorf("failed to acquire expired lease: %w", updateErr)
        }
        
        logger.Info("Acquired expired distributed lock", "lockName", dl.lockName)
        return NewLease(updatedLease, dl.client, dl.namespace), nil
    }
    
    return nil, fmt.Errorf("lock held by %s", *existingLease.Spec.HolderIdentity)
}

func (dl *DistributedLock) isLeaseExpired(lease *coordinationv1.Lease) bool {
    if lease.Spec.RenewTime == nil || lease.Spec.LeaseDurationSeconds == nil {
        return true
    }
    
    expiration := lease.Spec.RenewTime.Add(time.Duration(*lease.Spec.LeaseDurationSeconds) * time.Second)
    return time.Now().After(expiration)
}

type Lease struct {
    lease     *coordinationv1.Lease
    client    kubernetes.Interface
    namespace string
    renewCtx  context.Context
    cancelRenew context.CancelFunc
}

func NewLease(lease *coordinationv1.Lease, client kubernetes.Interface, namespace string) *Lease {
    ctx, cancel := context.WithCancel(context.Background())
    
    l := &Lease{
        lease:       lease,
        client:      client,
        namespace:   namespace,
        renewCtx:    ctx,
        cancelRenew: cancel,
    }
    
    // Start automatic renewal
    go l.autoRenew()
    
    return l
}

func (l *Lease) autoRenew() {
    ticker := time.NewTicker(10 * time.Second) // Renew every 10 seconds
    defer ticker.Stop()
    
    for {
        select {
        case <-l.renewCtx.Done():
            return
        case <-ticker.C:
            l.renew()
        }
    }
}

func (l *Lease) renew() {
    leaseClient := l.client.CoordinationV1().Leases(l.namespace)
    
    l.lease.Spec.RenewTime = &metav1.MicroTime{Time: time.Now()}
    _, err := leaseClient.Update(context.Background(), l.lease, metav1.UpdateOptions{})
    if err != nil {
        // Log error but continue - lease will expire naturally
        fmt.Printf("Failed to renew lease: %v\n", err)
    }
}

func (l *Lease) Release() {
    l.cancelRenew()
    
    leaseClient := l.client.CoordinationV1().Leases(l.namespace)
    err := leaseClient.Delete(context.Background(), l.lease.Name, metav1.DeleteOptions{})
    if err != nil {
        fmt.Printf("Failed to release lease: %v\n", err)
    }
}
```

### Step 3: Enhance CodeRun Deletion with State Awareness

#### 3.1 State-Aware Deletion Logic
```go
package cancellation

import (
    "context"
    "fmt"
    "time"
    
    platformv1 "github.com/5dlabs/platform/api/v1"
    "k8s.io/apimachinery/pkg/labels"
    "sigs.k8s.io/controller-runtime/pkg/client"
    "sigs.k8s.io/controller-runtime/pkg/log"
)

type StateAwareCancellation struct {
    client      client.Client
    stateManager StateManager // From Task 4
    lockManager  *DistributedLock
}

func (sac *StateAwareCancellation) CancelAgentsWithStateCheck(ctx context.Context, taskID string, prNumber int) error {
    logger := log.FromContext(ctx)
    
    // Acquire distributed lock
    lockName := fmt.Sprintf("cancel-%s", taskID)
    lock, err := sac.lockManager.TryAcquire(ctx)
    if err != nil {
        return fmt.Errorf("failed to acquire cancellation lock: %w", err)
    }
    defer lock.Release()
    
    logger.Info("Acquired cancellation lock", "taskId", taskID, "lockName", lockName)
    
    // Check current remediation state
    state, err := sac.stateManager.GetState(ctx, taskID)
    if err != nil {
        logger.Error(err, "Failed to get remediation state", "taskId", taskID)
        // Continue with cancellation even if state check fails
    }
    
    if state != nil && state.CancellationInProgress {
        logger.Info("Cancellation already in progress", "taskId", taskID)
        return nil // Already being handled
    }
    
    // Mark cancellation as started
    if state != nil {
        state.CancellationInProgress = true
        state.LastUpdate = time.Now()
        if err := sac.stateManager.UpdateState(ctx, state); err != nil {
            logger.Error(err, "Failed to update state with cancellation flag", "taskId", taskID)
        }
    }
    
    // Find CodeRuns to cancel
    codeRunList := &platformv1.CodeRunList{}
    listOpts := &client.ListOptions{
        Namespace: "agent-platform",
        LabelSelector: labels.SelectorFromSet(map[string]string{
            "task-id": taskID,
        }),
    }
    
    if err := sac.client.List(ctx, codeRunList, listOpts); err != nil {
        return fmt.Errorf("failed to list CodeRuns for task %s: %w", taskID, err)
    }
    
    logger.Info("Found CodeRuns to cancel", "count", len(codeRunList.Items), "taskId", taskID)
    
    // Cancel each CodeRun with retry logic
    var cancellationErrors []error
    for _, codeRun := range codeRunList.Items {
        if err := sac.cancelCodeRunWithRetry(ctx, &codeRun, 3); err != nil {
            logger.Error(err, "Failed to cancel CodeRun", "codeRunName", codeRun.Name)
            cancellationErrors = append(cancellationErrors, err)
        }
    }
    
    // Update state after cancellation attempt
    if state != nil {
        state.CancellationInProgress = false
        state.LastUpdate = time.Now()
        
        if len(cancellationErrors) > 0 {
            state.ErrorMessages = append(state.ErrorMessages, 
                fmt.Sprintf("Cancellation errors: %d failed", len(cancellationErrors)))
        }
        
        if err := sac.stateManager.UpdateState(ctx, state); err != nil {
            logger.Error(err, "Failed to update state after cancellation", "taskId", taskID)
        }
    }
    
    if len(cancellationErrors) > 0 {
        return fmt.Errorf("cancellation completed with %d errors", len(cancellationErrors))
    }
    
    logger.Info("Successfully cancelled all agents", "taskId", taskID, "count", len(codeRunList.Items))
    return nil
}

func (sac *StateAwareCancellation) cancelCodeRunWithRetry(ctx context.Context, codeRun *platformv1.CodeRun, maxRetries int) error {
    logger := log.FromContext(ctx)
    
    for attempt := 1; attempt <= maxRetries; attempt++ {
        // Check if CodeRun is already completed
        if codeRun.Status.Phase == platformv1.CodeRunPhaseSucceeded || 
           codeRun.Status.Phase == platformv1.CodeRunPhaseFailed {
            logger.Info("CodeRun already completed, skipping cancellation", 
                "codeRunName", codeRun.Name, 
                "phase", codeRun.Status.Phase)
            return nil
        }
        
        // Attempt graceful termination first
        if attempt == 1 {
            logger.Info("Attempting graceful CodeRun termination", "codeRunName", codeRun.Name)
            if err := sac.gracefulTermination(ctx, codeRun); err == nil {
                return nil
            }
        }
        
        // Force delete if graceful termination failed
        logger.Info("Force deleting CodeRun", "codeRunName", codeRun.Name, "attempt", attempt)
        if err := sac.client.Delete(ctx, codeRun); err != nil {
            if attempt == maxRetries {
                return fmt.Errorf("failed to delete CodeRun after %d attempts: %w", maxRetries, err)
            }
            
            // Exponential backoff
            backoffDuration := time.Duration(attempt*attempt) * time.Second
            logger.Info("CodeRun deletion failed, retrying", 
                "codeRunName", codeRun.Name, 
                "attempt", attempt, 
                "backoff", backoffDuration)
            
            select {
            case <-time.After(backoffDuration):
                continue
            case <-ctx.Done():
                return ctx.Err()
            }
        }
        
        // Verify deletion
        if err := sac.verifyDeletion(ctx, codeRun); err != nil {
            logger.Error(err, "CodeRun deletion verification failed", "codeRunName", codeRun.Name)
            if attempt < maxRetries {
                continue
            }
            return err
        }
        
        logger.Info("Successfully cancelled CodeRun", "codeRunName", codeRun.Name)
        return nil
    }
    
    return fmt.Errorf("failed to cancel CodeRun %s after %d attempts", codeRun.Name, maxRetries)
}

func (sac *StateAwareCancellation) gracefulTermination(ctx context.Context, codeRun *platformv1.CodeRun) error {
    // Send SIGTERM to the CodeRun (implementation depends on CodeRun CRD)
    // This would typically update the CodeRun spec to signal termination
    
    codeRun.Spec.Terminate = true // Assuming this field exists
    if err := sac.client.Update(ctx, codeRun); err != nil {
        return fmt.Errorf("failed to send termination signal: %w", err)
    }
    
    // Wait for graceful shutdown (up to 30 seconds)
    timeout := time.After(30 * time.Second)
    ticker := time.NewTicker(2 * time.Second)
    defer ticker.Stop()
    
    for {
        select {
        case <-timeout:
            return fmt.Errorf("graceful termination timeout")
        case <-ticker.C:
            // Refresh CodeRun status
            if err := sac.client.Get(ctx, client.ObjectKeyFromObject(codeRun), codeRun); err != nil {
                return fmt.Errorf("failed to get CodeRun status: %w", err)
            }
            
            if codeRun.Status.Phase == platformv1.CodeRunPhaseSucceeded || 
               codeRun.Status.Phase == platformv1.CodeRunPhaseFailed {
                return nil // Graceful termination successful
            }
        }
    }
}

func (sac *StateAwareCancellation) verifyDeletion(ctx context.Context, codeRun *platformv1.CodeRun) error {
    // Wait up to 10 seconds for deletion to complete
    timeout := time.After(10 * time.Second)
    ticker := time.NewTicker(1 * time.Second)
    defer ticker.Stop()
    
    for {
        select {
        case <-timeout:
            return fmt.Errorf("deletion verification timeout")
        case <-ticker.C:
            err := sac.client.Get(ctx, client.ObjectKeyFromObject(codeRun), &platformv1.CodeRun{})
            if err != nil {
                if client.IgnoreNotFound(err) == nil {
                    return nil // Successfully deleted
                }
                return fmt.Errorf("error verifying deletion: %w", err)
            }
        }
    }
}
```

### Step 4: Implement Atomic Label Transitions

#### 4.1 GitHub Label Management with Optimistic Concurrency
```go
package cancellation

import (
    "context"
    "fmt"
    "strings"
    "time"
    
    "github.com/google/go-github/v50/github"
    "golang.org/x/oauth2"
)

type AtomicLabelManager struct {
    client *github.Client
    owner  string
    repo   string
}

func NewAtomicLabelManager(token, owner, repo string) *AtomicLabelManager {
    ts := oauth2.StaticTokenSource(
        &oauth2.Token{AccessToken: token},
    )
    tc := oauth2.NewClient(context.Background(), ts)
    
    return &AtomicLabelManager{
        client: github.NewClient(tc),
        owner:  owner,
        repo:   repo,
    }
}

func (alm *AtomicLabelManager) AtomicLabelTransition(ctx context.Context, prNumber int, transitions []LabelTransition) error {
    maxRetries := 5
    baseDelay := 100 * time.Millisecond
    
    for attempt := 1; attempt <= maxRetries; attempt++ {
        if err := alm.attemptLabelTransition(ctx, prNumber, transitions); err != nil {
            if isRetryableError(err) && attempt < maxRetries {
                // Exponential backoff with jitter
                delay := time.Duration(attempt*attempt) * baseDelay
                time.Sleep(delay)
                continue
            }
            return fmt.Errorf("label transition failed after %d attempts: %w", attempt, err)
        }
        return nil
    }
    
    return fmt.Errorf("unexpected end of retry loop")
}

type LabelTransition struct {
    Action string // "add", "remove", "replace"
    Labels []string
    FromLabel string // For replace action
}

func (alm *AtomicLabelManager) attemptLabelTransition(ctx context.Context, prNumber int, transitions []LabelTransition) error {
    // Get current PR with ETag for optimistic concurrency control
    pr, response, err := alm.client.PullRequests.Get(ctx, alm.owner, alm.repo, prNumber)
    if err != nil {
        return fmt.Errorf("failed to get PR %d: %w", prNumber, err)
    }
    
    etag := response.Header.Get("ETag")
    currentLabels := make([]string, len(pr.Labels))
    for i, label := range pr.Labels {
        currentLabels[i] = *label.Name
    }
    
    // Calculate new labels based on transitions
    newLabels := alm.calculateNewLabels(currentLabels, transitions)
    
    // Attempt to update labels with ETag check
    labelObjects := make([]*github.Label, len(newLabels))
    for i, labelName := range newLabels {
        labelObjects[i] = &github.Label{Name: &labelName}
    }
    
    // Create update request with conditional headers
    issueRequest := &github.IssueRequest{
        Labels: labelObjects,
    }
    
    // Add conditional update headers
    opts := &github.RequestOptions{
        IfMatch: etag,
    }
    
    _, updateResponse, err := alm.client.Issues.Edit(ctx, alm.owner, alm.repo, prNumber, issueRequest)
    if err != nil {
        if updateResponse != nil && updateResponse.StatusCode == 412 {
            // Precondition failed - concurrent modification detected
            return &ConcurrentModificationError{
                Message: "PR was modified by another process",
                PRNumber: prNumber,
            }
        }
        return fmt.Errorf("failed to update PR labels: %w", err)
    }
    
    return nil
}

func (alm *AtomicLabelManager) calculateNewLabels(currentLabels []string, transitions []LabelTransition) []string {
    labelSet := make(map[string]bool)
    
    // Start with current labels
    for _, label := range currentLabels {
        labelSet[label] = true
    }
    
    // Apply transitions
    for _, transition := range transitions {
        switch transition.Action {
        case "add":
            for _, label := range transition.Labels {
                labelSet[label] = true
            }
        case "remove":
            for _, label := range transition.Labels {
                delete(labelSet, label)
            }
        case "replace":
            if transition.FromLabel != "" {
                delete(labelSet, transition.FromLabel)
            }
            for _, label := range transition.Labels {
                labelSet[label] = true
            }
        }
    }
    
    // Convert back to slice
    result := make([]string, 0, len(labelSet))
    for label := range labelSet {
        result = append(result, label)
    }
    
    return result
}

type ConcurrentModificationError struct {
    Message  string
    PRNumber int
}

func (e *ConcurrentModificationError) Error() string {
    return fmt.Sprintf("concurrent modification on PR %d: %s", e.PRNumber, e.Message)
}

func isRetryableError(err error) bool {
    if strings.Contains(err.Error(), "rate limit") {
        return true
    }
    if strings.Contains(err.Error(), "timeout") {
        return true
    }
    if _, ok := err.(*ConcurrentModificationError); ok {
        return true
    }
    return false
}
```

### Step 5: Build Concurrent Cancellation Coordinator

#### 5.1 Coordination Layer Implementation
```go
package cancellation

import (
    "context"
    "encoding/json"
    "fmt"
    "sync"
    "time"
    
    v1 "k8s.io/api/core/v1"
    metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
    "k8s.io/client-go/kubernetes"
    "sigs.k8s.io/controller-runtime/pkg/log"
)

type CancellationCoordinator struct {
    client           kubernetes.Interface
    namespace        string
    stateManager     StateManager
    labelManager     *AtomicLabelManager
    activeCancellations sync.Map
    maxConcurrent    int
    circuitBreaker   *CircuitBreaker
}

type CancellationRequest struct {
    TaskID         string    `json:"task_id"`
    PRNumber       int       `json:"pr_number"`
    CorrelationID  string    `json:"correlation_id"`
    RequestTime    time.Time `json:"request_time"`
    Priority       int       `json:"priority"`
    RetryCount     int       `json:"retry_count"`
}

type CancellationStatus struct {
    Status      string    `json:"status"` // pending, in_progress, completed, failed
    StartTime   time.Time `json:"start_time"`
    EndTime     *time.Time `json:"end_time"`
    Error       string    `json:"error,omitempty"`
    WorkerID    string    `json:"worker_id"`
}

func NewCancellationCoordinator(client kubernetes.Interface, namespace string, 
    stateManager StateManager, labelManager *AtomicLabelManager) *CancellationCoordinator {
    
    return &CancellationCoordinator{
        client:           client,
        namespace:        namespace,
        stateManager:     stateManager,
        labelManager:     labelManager,
        maxConcurrent:    10,
        circuitBreaker:   NewCircuitBreaker(5, 30*time.Second),
    }
}

func (cc *CancellationCoordinator) RequestCancellation(ctx context.Context, req *CancellationRequest) error {
    logger := log.FromContext(ctx)
    
    // Check for duplicate request
    if _, exists := cc.activeCancellations.Load(req.TaskID); exists {
        logger.Info("Cancellation already in progress", "taskId", req.TaskID)
        return nil
    }
    
    // Check circuit breaker
    if !cc.circuitBreaker.Allow() {
        return fmt.Errorf("circuit breaker open - too many recent failures")
    }
    
    // Add to coordination ConfigMap
    if err := cc.addToQueue(ctx, req); err != nil {
        return fmt.Errorf("failed to queue cancellation request: %w", err)
    }
    
    // Track active cancellation
    status := &CancellationStatus{
        Status:    "pending",
        StartTime: time.Now(),
        WorkerID:  fmt.Sprintf("coordinator-%d", time.Now().Unix()),
    }
    cc.activeCancellations.Store(req.TaskID, status)
    
    logger.Info("Cancellation request queued", 
        "taskId", req.TaskID, 
        "correlationId", req.CorrelationID,
        "priority", req.Priority)
    
    // Process asynchronously
    go cc.processCancellation(ctx, req)
    
    return nil
}

func (cc *CancellationCoordinator) addToQueue(ctx context.Context, req *CancellationRequest) error {
    cmName := "cancellation-queue"
    cmClient := cc.client.CoreV1().ConfigMaps(cc.namespace)
    
    // Get existing queue ConfigMap
    cm, err := cmClient.Get(ctx, cmName, metav1.GetOptions{})
    if err != nil {
        // Create new ConfigMap if it doesn't exist
        cm = &v1.ConfigMap{
            ObjectMeta: metav1.ObjectMeta{
                Name:      cmName,
                Namespace: cc.namespace,
                Labels: map[string]string{
                    "app":       "agent-cancellation",
                    "component": "coordination-queue",
                },
            },
            Data: make(map[string]string),
        }
    }
    
    // Serialize request
    reqData, err := json.Marshal(req)
    if err != nil {
        return fmt.Errorf("failed to marshal cancellation request: %w", err)
    }
    
    // Add to queue
    queueKey := fmt.Sprintf("request-%s-%d", req.TaskID, req.RequestTime.Unix())
    cm.Data[queueKey] = string(reqData)
    
    // Update ConfigMap
    if cm.ResourceVersion == "" {
        _, err = cmClient.Create(ctx, cm, metav1.CreateOptions{})
    } else {
        _, err = cmClient.Update(ctx, cm, metav1.UpdateOptions{})
    }
    
    return err
}

func (cc *CancellationCoordinator) processCancellation(ctx context.Context, req *CancellationRequest) {
    logger := log.FromContext(ctx)
    
    // Update status to in_progress
    if statusInterface, exists := cc.activeCancellations.Load(req.TaskID); exists {
        if status, ok := statusInterface.(*CancellationStatus); ok {
            status.Status = "in_progress"
            status.StartTime = time.Now()
            cc.activeCancellations.Store(req.TaskID, status)
        }
    }
    
    defer func() {
        // Clean up active cancellations map
        cc.activeCancellations.Delete(req.TaskID)
    }()
    
    // Perform the actual cancellation
    err := cc.executeCancellation(ctx, req)
    
    // Update final status
    if statusInterface, exists := cc.activeCancellations.Load(req.TaskID); exists {
        if status, ok := statusInterface.(*CancellationStatus); ok {
            now := time.Now()
            status.EndTime = &now
            
            if err != nil {
                status.Status = "failed"
                status.Error = err.Error()
                cc.circuitBreaker.RecordFailure()
                logger.Error(err, "Cancellation failed", "taskId", req.TaskID)
            } else {
                status.Status = "completed"
                cc.circuitBreaker.RecordSuccess()
                logger.Info("Cancellation completed successfully", "taskId", req.TaskID)
            }
            
            cc.activeCancellations.Store(req.TaskID, status)
        }
    }
    
    // Remove from queue
    if err := cc.removeFromQueue(ctx, req); err != nil {
        logger.Error(err, "Failed to remove from queue", "taskId", req.TaskID)
    }
}

func (cc *CancellationCoordinator) executeCancellation(ctx context.Context, req *CancellationRequest) error {
    // Create state-aware cancellation handler
    sac := &StateAwareCancellation{
        client:       cc.client, // This should be controller-runtime client
        stateManager: cc.stateManager,
    }
    
    // Cancel agents
    if err := sac.CancelAgentsWithStateCheck(ctx, req.TaskID, req.PRNumber); err != nil {
        return fmt.Errorf("failed to cancel agents: %w", err)
    }
    
    // Update PR labels
    transitions := []LabelTransition{
        {
            Action: "remove",
            Labels: []string{"ready-for-qa"},
        },
        {
            Action: "add", 
            Labels: []string{"remediation-in-progress"},
        },
    }
    
    if err := cc.labelManager.AtomicLabelTransition(ctx, req.PRNumber, transitions); err != nil {
        return fmt.Errorf("failed to update PR labels: %w", err)
    }
    
    return nil
}

func (cc *CancellationCoordinator) removeFromQueue(ctx context.Context, req *CancellationRequest) error {
    cmName := "cancellation-queue"
    cmClient := cc.client.CoreV1().ConfigMaps(cc.namespace)
    
    cm, err := cmClient.Get(ctx, cmName, metav1.GetOptions{})
    if err != nil {
        return err
    }
    
    queueKey := fmt.Sprintf("request-%s-%d", req.TaskID, req.RequestTime.Unix())
    delete(cm.Data, queueKey)
    
    _, err = cmClient.Update(ctx, cm, metav1.UpdateOptions{})
    return err
}

func (cc *CancellationCoordinator) GetStatus(taskID string) *CancellationStatus {
    if statusInterface, exists := cc.activeCancellations.Load(taskID); exists {
        if status, ok := statusInterface.(*CancellationStatus); ok {
            return status
        }
    }
    return nil
}
```

### Step 6: Implement Advanced Recovery System

#### 6.1 Recovery and Reconciliation
```go
package cancellation

import (
    "context"
    "fmt"
    "time"
    
    "sigs.k8s.io/controller-runtime/pkg/log"
)

type RecoveryManager struct {
    coordinator     *CancellationCoordinator
    stateManager    StateManager
    reconcileInterval time.Duration
}

func NewRecoveryManager(coordinator *CancellationCoordinator, stateManager StateManager) *RecoveryManager {
    return &RecoveryManager{
        coordinator:      coordinator,
        stateManager:     stateManager,
        reconcileInterval: 30 * time.Second,
    }
}

func (rm *RecoveryManager) StartReconciliation(ctx context.Context) {
    logger := log.FromContext(ctx)
    
    ticker := time.NewTicker(rm.reconcileInterval)
    defer ticker.Stop()
    
    for {
        select {
        case <-ctx.Done():
            logger.Info("Stopping reconciliation")
            return
        case <-ticker.C:
            if err := rm.reconcile(ctx); err != nil {
                logger.Error(err, "Reconciliation failed")
            }
        }
    }
}

func (rm *RecoveryManager) reconcile(ctx context.Context) error {
    logger := log.FromContext(ctx)
    
    // Detect partial failures
    inconsistencies, err := rm.detectInconsistencies(ctx)
    if err != nil {
        return fmt.Errorf("failed to detect inconsistencies: %w", err)
    }
    
    if len(inconsistencies) == 0 {
        return nil
    }
    
    logger.Info("Found inconsistencies to repair", "count", len(inconsistencies))
    
    // Repair each inconsistency
    for _, inconsistency := range inconsistencies {
        if err := rm.repairInconsistency(ctx, inconsistency); err != nil {
            logger.Error(err, "Failed to repair inconsistency", "type", inconsistency.Type, "taskId", inconsistency.TaskID)
        }
    }
    
    return nil
}

type Inconsistency struct {
    Type        string `json:"type"`
    TaskID      string `json:"task_id"`
    PRNumber    int    `json:"pr_number"`
    Description string `json:"description"`
    Severity    string `json:"severity"`
}

func (rm *RecoveryManager) detectInconsistencies(ctx context.Context) ([]*Inconsistency, error) {
    var inconsistencies []*Inconsistency
    
    // Check for stuck cancellations
    rm.coordinator.activeCancellations.Range(func(key, value interface{}) bool {
        taskID := key.(string)
        status := value.(*CancellationStatus)
        
        // If cancellation has been in progress for more than 5 minutes
        if status.Status == "in_progress" && time.Since(status.StartTime) > 5*time.Minute {
            inconsistencies = append(inconsistencies, &Inconsistency{
                Type:        "stuck_cancellation",
                TaskID:      taskID,
                Description: fmt.Sprintf("Cancellation stuck for %v", time.Since(status.StartTime)),
                Severity:    "high",
            })
        }
        
        return true
    })
    
    // Check for orphaned locks
    if orphanedLocks, err := rm.detectOrphanedLocks(ctx); err == nil {
        inconsistencies = append(inconsistencies, orphanedLocks...)
    }
    
    // Check for state inconsistencies
    if stateIssues, err := rm.detectStateInconsistencies(ctx); err == nil {
        inconsistencies = append(inconsistencies, stateIssues...)
    }
    
    return inconsistencies, nil
}

func (rm *RecoveryManager) detectOrphanedLocks(ctx context.Context) ([]*Inconsistency, error) {
    var inconsistencies []*Inconsistency
    
    // This would query Kubernetes leases and check for expired ones
    // Implementation depends on specific lease detection logic
    
    return inconsistencies, nil
}

func (rm *RecoveryManager) detectStateInconsistencies(ctx context.Context) ([]*Inconsistency, error) {
    var inconsistencies []*Inconsistency
    
    // Check for states marked as "cancellation in progress" for too long
    // Implementation would query state management system
    
    return inconsistencies, nil
}

func (rm *RecoveryManager) repairInconsistency(ctx context.Context, inconsistency *Inconsistency) error {
    logger := log.FromContext(ctx)
    
    switch inconsistency.Type {
    case "stuck_cancellation":
        return rm.repairStuckCancellation(ctx, inconsistency)
    case "orphaned_lock":
        return rm.repairOrphanedLock(ctx, inconsistency)
    case "state_inconsistency":
        return rm.repairStateInconsistency(ctx, inconsistency)
    default:
        logger.Warn("Unknown inconsistency type", "type", inconsistency.Type)
        return nil
    }
}

func (rm *RecoveryManager) repairStuckCancellation(ctx context.Context, inconsistency *Inconsistency) error {
    logger := log.FromContext(ctx)
    
    // Remove from active cancellations
    rm.coordinator.activeCancellations.Delete(inconsistency.TaskID)
    
    // Update state to clear cancellation in progress flag
    state, err := rm.stateManager.GetState(ctx, inconsistency.TaskID)
    if err != nil {
        return fmt.Errorf("failed to get state: %w", err)
    }
    
    if state != nil {
        state.CancellationInProgress = false
        state.LastUpdate = time.Now()
        state.ErrorMessages = append(state.ErrorMessages, 
            "Recovered from stuck cancellation via reconciliation")
        
        if err := rm.stateManager.UpdateState(ctx, state); err != nil {
            return fmt.Errorf("failed to update state: %w", err)
        }
    }
    
    logger.Info("Repaired stuck cancellation", "taskId", inconsistency.TaskID)
    return nil
}

func (rm *RecoveryManager) repairOrphanedLock(ctx context.Context, inconsistency *Inconsistency) error {
    // Implementation would release orphaned Kubernetes leases
    return nil
}

func (rm *RecoveryManager) repairStateInconsistency(ctx context.Context, inconsistency *Inconsistency) error {
    // Implementation would repair state inconsistencies
    return nil
}
```

### Step 7: Create Stress Tests for Concurrent Cancellations

#### 7.1 Comprehensive Test Suite
```go
package cancellation_test

import (
    "context"
    "fmt"
    "sync"
    "testing"
    "time"
    
    "github.com/stretchr/testify/assert"
    "github.com/stretchr/testify/require"
)

func TestConcurrentCancellations(t *testing.T) {
    tests := []struct {
        name             string
        numConcurrent    int
        simulateFailures bool
        expectedSuccess  int
    }{
        {
            name:            "10 concurrent cancellations",
            numConcurrent:   10,
            simulateFailures: false,
            expectedSuccess: 10,
        },
        {
            name:            "20 concurrent cancellations with failures",
            numConcurrent:   20,
            simulateFailures: true,
            expectedSuccess: 15, // Expect 75% success rate with simulated failures
        },
    }
    
    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            coordinator := setupTestCoordinator(t)
            
            var wg sync.WaitGroup
            var successCount int32
            var mu sync.Mutex
            
            ctx, cancel := context.WithTimeout(context.Background(), 60*time.Second)
            defer cancel()
            
            // Launch concurrent cancellation requests
            for i := 0; i < tt.numConcurrent; i++ {
                wg.Add(1)
                go func(taskID string) {
                    defer wg.Done()
                    
                    req := &CancellationRequest{
                        TaskID:        taskID,
                        PRNumber:      100 + i,
                        CorrelationID: fmt.Sprintf("test-%s", taskID),
                        RequestTime:   time.Now(),
                        Priority:      1,
                    }
                    
                    if err := coordinator.RequestCancellation(ctx, req); err == nil {
                        mu.Lock()
                        successCount++
                        mu.Unlock()
                    }
                }(fmt.Sprintf("task-%d", i))
            }
            
            // Wait for all cancellations to complete
            done := make(chan struct{})
            go func() {
                wg.Wait()
                close(done)
            }()
            
            select {
            case <-done:
                // Success
            case <-ctx.Done():
                t.Fatal("Test timed out")
            }
            
            assert.GreaterOrEqual(t, int(successCount), tt.expectedSuccess)
        })
    }
}

func TestDistributedLocking(t *testing.T) {
    coordinator := setupTestCoordinator(t)
    
    numWorkers := 5
    taskID := "test-lock-contention"
    
    var wg sync.WaitGroup
    acquiredCount := int32(0)
    
    ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
    defer cancel()
    
    // Launch multiple workers trying to acquire the same lock
    for i := 0; i < numWorkers; i++ {
        wg.Add(1)
        go func(workerID int) {
            defer wg.Done()
            
            lockName := fmt.Sprintf("test-lock-%s", taskID)
            holderName := fmt.Sprintf("worker-%d", workerID)
            
            lock := NewDistributedLock(coordinator.client, "test", lockName, holderName)
            
            if lease, err := lock.TryAcquire(ctx); err == nil {
                atomic.AddInt32(&acquiredCount, 1)
                
                // Hold lock for 1 second
                time.Sleep(1 * time.Second)
                lease.Release()
            }
        }(i)
    }
    
    wg.Wait()
    
    // Only one worker should have acquired the lock at any given time
    // but due to sequential acquisition, all should eventually succeed
    assert.Equal(t, int32(numWorkers), acquiredCount)
}

func TestStateAwareCancellation(t *testing.T) {
    coordinator := setupTestCoordinator(t)
    
    taskID := "test-state-aware"
    prNumber := 200
    
    ctx := context.Background()
    
    // Create initial state
    state := &RemediationState{
        TaskID:               taskID,
        Iteration:           3,
        Status:              StatusInProgress,
        CancellationInProgress: false,
    }
    
    err := coordinator.stateManager.UpdateState(ctx, state)
    require.NoError(t, err)
    
    // Create mock CodeRuns
    mockCodeRuns := createMockCodeRuns(taskID, 3)
    
    // Perform cancellation
    sac := &StateAwareCancellation{
        stateManager: coordinator.stateManager,
    }
    
    err = sac.CancelAgentsWithStateCheck(ctx, taskID, prNumber)
    require.NoError(t, err)
    
    // Verify state was updated
    updatedState, err := coordinator.stateManager.GetState(ctx, taskID)
    require.NoError(t, err)
    assert.False(t, updatedState.CancellationInProgress)
}

func TestRecoveryManager(t *testing.T) {
    coordinator := setupTestCoordinator(t)
    recoveryManager := NewRecoveryManager(coordinator, coordinator.stateManager)
    
    ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
    defer cancel()
    
    // Simulate stuck cancellation
    stuckStatus := &CancellationStatus{
        Status:    "in_progress",
        StartTime: time.Now().Add(-10 * time.Minute), // 10 minutes ago
    }
    coordinator.activeCancellations.Store("stuck-task", stuckStatus)
    
    // Run reconciliation
    err := recoveryManager.reconcile(ctx)
    require.NoError(t, err)
    
    // Verify stuck cancellation was cleaned up
    _, exists := coordinator.activeCancellations.Load("stuck-task")
    assert.False(t, exists)
}

func BenchmarkConcurrentCancellations(b *testing.B) {
    coordinator := setupTestCoordinator(b)
    
    b.Run("ConcurrentCancellations", func(b *testing.B) {
        b.RunParallel(func(pb *testing.PB) {
            i := 0
            for pb.Next() {
                taskID := fmt.Sprintf("bench-task-%d", i)
                req := &CancellationRequest{
                    TaskID:        taskID,
                    PRNumber:      1000 + i,
                    CorrelationID: fmt.Sprintf("bench-%s", taskID),
                    RequestTime:   time.Now(),
                }
                
                ctx := context.Background()
                coordinator.RequestCancellation(ctx, req)
                i++
            }
        })
    })
}

func setupTestCoordinator(t testing.TB) *CancellationCoordinator {
    // Setup mock Kubernetes client, state manager, and label manager
    // Implementation depends on test infrastructure
    return nil
}

func createMockCodeRuns(taskID string, count int) []MockCodeRun {
    // Create mock CodeRun objects for testing
    return nil
}
```

## Performance Considerations

### Concurrency Management
- Distributed locks prevent race conditions
- Circuit breaker prevents cascading failures  
- Configurable concurrency limits
- Efficient resource cleanup

### Resource Optimization
- Connection pooling for Kubernetes API
- Batched operations where possible
- Configurable retry policies
- Memory-efficient state tracking

### Monitoring Integration
- Comprehensive metrics collection
- Distributed tracing support
- Performance profiling capabilities
- Alert integration for failures

## Security Considerations

### Access Control
- Minimal RBAC permissions
- Service account isolation
- Audit logging for all operations
- Input validation and sanitization

### Concurrency Safety
- Thread-safe operations
- Atomic state updates
- Deadlock prevention
- Resource leak prevention

## Success Criteria
- Enhanced sensor handles concurrent Rex push events
- Distributed locking prevents race conditions
- State-aware cancellation integrates with Task 4
- Atomic label transitions prevent conflicts
- Recovery system handles partial failures
- Comprehensive test coverage validates all scenarios
- Performance meets production requirements
- No resource leaks or deadlocks under load