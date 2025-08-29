# Task 10: Implement Security and RBAC Controls

## Overview
Set up comprehensive security controls, RBAC permissions, and validation for the remediation system. This implementation establishes a robust security foundation including minimal GitHub token permissions, Kubernetes RBAC configuration, input validation, rate limiting, and comprehensive audit logging.

## Technical Context
The Agent Remediation Loop handles sensitive operations including GitHub repository access, Kubernetes resource management, and automated code modifications. A comprehensive security framework is essential to prevent unauthorized access, protect against malicious inputs, and maintain audit trails for compliance and troubleshooting.

### Security Scope
- **GitHub Integration Security**: Token scoping, API access controls
- **Kubernetes RBAC**: Minimal privilege access controls
- **Input Validation**: Protection against injection attacks and malicious content
- **Rate Limiting**: Protection against abuse and resource exhaustion
- **Audit Logging**: Comprehensive security event tracking
- **Access Control**: Authorized reviewer allowlists and permission validation

## Implementation Guide

### Step 1: Configure Minimal GitHub Token Permissions

#### 1.1 GitHub App Permissions Configuration
```yaml
# GitHub App Permissions Configuration
name: "Agent Remediation Loop"
description: "Automated remediation system for code quality issues"

# Repository permissions (minimal required)
permissions:
  contents: read                    # Read repository content for task context
  issues: write                    # Comment on PRs for escalation/status
  pull_requests: write             # Access PR information and post comments  
  metadata: read                   # Access repository metadata
  statuses: read                   # Read commit/PR status checks
  discussions: write               # Post to discussions if needed

# Organization permissions (if needed)
organization_permissions:
  members: read                    # Validate team membership for authorization

# Events the app can receive
events:
  - issue_comment                  # PR comment events
  - pull_request                   # PR events
  - pull_request_review           # PR review events
  - status                        # Status check events
```

#### 1.2 Token Management and Validation
```go
package security

import (
    "context"
    "fmt"
    "net/http"
    "time"
    
    "github.com/google/go-github/v50/github"
    "go.uber.org/zap"
    "golang.org/x/oauth2"
)

type GitHubTokenManager struct {
    client      *github.Client
    logger      *zap.Logger
    tokenScopes []string
}

func NewGitHubTokenManager(token string, logger *zap.Logger) (*GitHubTokenManager, error) {
    ts := oauth2.StaticTokenSource(&oauth2.Token{AccessToken: token})
    tc := oauth2.NewClient(context.Background(), ts)
    
    client := github.NewClient(tc)
    
    // Validate token and get scopes
    scopes, err := validateToken(client)
    if err != nil {
        return nil, fmt.Errorf("failed to validate GitHub token: %w", err)
    }
    
    logger.Info("GitHub token validated", 
        zap.Strings("scopes", scopes))
    
    return &GitHubTokenManager{
        client:      client,
        logger:      logger,
        tokenScopes: scopes,
    }, nil
}

func validateToken(client *github.Client) ([]string, error) {
    ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
    defer cancel()
    
    // Test token by getting user info
    _, resp, err := client.Users.Get(ctx, "")
    if err != nil {
        return nil, fmt.Errorf("token validation failed: %w", err)
    }
    
    // Extract token scopes from response headers
    scopes := []string{}
    if scopeHeader := resp.Header.Get("X-OAuth-Scopes"); scopeHeader != "" {
        scopes = strings.Split(strings.ReplaceAll(scopeHeader, " ", ""), ",")
    }
    
    // Validate required scopes
    requiredScopes := []string{"repo", "write:discussion"}
    for _, required := range requiredScopes {
        if !contains(scopes, required) {
            return nil, fmt.Errorf("token missing required scope: %s", required)
        }
    }
    
    return scopes, nil
}

func (gtm *GitHubTokenManager) ValidatePermissions(ctx context.Context, owner, repo string) error {
    // Check repository access
    _, _, err := gtm.client.Repositories.Get(ctx, owner, repo)
    if err != nil {
        gtm.logger.Warn("Repository access validation failed",
            zap.String("owner", owner),
            zap.String("repo", repo),
            zap.Error(err))
        return fmt.Errorf("insufficient repository permissions: %w", err)
    }
    
    // Validate specific permissions by attempting operations
    return gtm.validateOperationalPermissions(ctx, owner, repo)
}

func (gtm *GitHubTokenManager) validateOperationalPermissions(ctx context.Context, owner, repo string) error {
    // Test issue comment permission (non-destructive test)
    issues, _, err := gtm.client.Issues.ListByRepo(ctx, owner, repo, &github.IssueListByRepoOptions{
        State: github.String("open"),
        Sort:  github.String("created"),
        Direction: github.String("desc"),
        ListOptions: github.ListOptions{PerPage: 1},
    })
    
    if err != nil {
        return fmt.Errorf("cannot read issues: %w", err)
    }
    
    gtm.logger.Info("GitHub permissions validated",
        zap.String("owner", owner),
        zap.String("repo", repo),
        zap.Int("accessible_issues", len(issues)))
    
    return nil
}
```

### Step 2: Set Up Kubernetes RBAC for Agent Operations

#### 2.1 Service Account and Role Configuration
```yaml
# Service Account for Remediation Controller
apiVersion: v1
kind: ServiceAccount
metadata:
  name: remediation-controller-sa
  namespace: agent-platform
  labels:
    app: agent-remediation-loop
    component: controller
---
# Role for Remediation Operations
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  namespace: agent-platform
  name: remediation-controller-role
  labels:
    app: agent-remediation-loop
rules:
  # ConfigMap permissions for state management
  - apiGroups: [""]
    resources: ["configmaps"]
    verbs: ["get", "list", "create", "update", "patch", "delete"]
    resourceNames: [] # Limit to specific ConfigMaps if needed
  
  # Secret permissions for GitHub tokens
  - apiGroups: [""]
    resources: ["secrets"]
    verbs: ["get", "list"]
    resourceNames: ["github-token", "github-webhook-secret"]
  
  # CodeRun CRD permissions
  - apiGroups: ["platform.5dlabs.com"]
    resources: ["coderuns"]
    verbs: ["get", "list", "create", "update", "patch", "delete", "watch"]
  
  # Event permissions for status updates
  - apiGroups: [""]
    resources: ["events"]
    verbs: ["create", "patch"]
  
  # Lease permissions for distributed locking
  - apiGroups: ["coordination.k8s.io"]
    resources: ["leases"]
    verbs: ["get", "list", "create", "update", "patch", "delete"]

---
# RoleBinding
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: remediation-controller-binding
  namespace: agent-platform
  labels:
    app: agent-remediation-loop
subjects:
  - kind: ServiceAccount
    name: remediation-controller-sa
    namespace: agent-platform
roleRef:
  kind: Role
  name: remediation-controller-role
  apiGroup: rbac.authorization.k8s.io

---
# ClusterRole for limited cluster-wide permissions
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: remediation-controller-cluster-role
  labels:
    app: agent-remediation-loop
rules:
  # Node information for system health checks
  - apiGroups: [""]
    resources: ["nodes"]
    verbs: ["get", "list"]
  
  # Namespace access for multi-namespace operations (if needed)
  - apiGroups: [""]
    resources: ["namespaces"]
    verbs: ["get", "list"]

---
# ClusterRoleBinding
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: remediation-controller-cluster-binding
  labels:
    app: agent-remediation-loop
subjects:
  - kind: ServiceAccount
    name: remediation-controller-sa
    namespace: agent-platform
roleRef:
  kind: ClusterRole
  name: remediation-controller-cluster-role
  apiGroup: rbac.authorization.k8s.io
```

#### 2.2 RBAC Validation Implementation
```go
package security

import (
    "context"
    "fmt"
    
    authv1 "k8s.io/api/authorization/v1"
    metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
    "k8s.io/client-go/kubernetes"
    "go.uber.org/zap"
)

type RBACValidator struct {
    client    kubernetes.Interface
    namespace string
    logger    *zap.Logger
}

func NewRBACValidator(client kubernetes.Interface, namespace string, logger *zap.Logger) *RBACValidator {
    return &RBACValidator{
        client:    client,
        namespace: namespace,
        logger:    logger,
    }
}

func (rv *RBACValidator) ValidatePermissions(ctx context.Context) error {
    requiredPermissions := []Permission{
        {Resource: "configmaps", Verbs: []string{"get", "list", "create", "update", "patch"}},
        {Resource: "coderuns", Group: "platform.5dlabs.com", Verbs: []string{"get", "list", "create", "delete"}},
        {Resource: "leases", Group: "coordination.k8s.io", Verbs: []string{"get", "create", "update"}},
        {Resource: "secrets", Verbs: []string{"get", "list"}},
        {Resource: "events", Verbs: []string{"create"}},
    }
    
    for _, perm := range requiredPermissions {
        if err := rv.checkPermission(ctx, perm); err != nil {
            return fmt.Errorf("RBAC validation failed for %s: %w", perm.Resource, err)
        }
    }
    
    rv.logger.Info("RBAC permissions validated successfully")
    return nil
}

type Permission struct {
    Group     string
    Resource  string
    Verbs     []string
    Namespace string
}

func (rv *RBACValidator) checkPermission(ctx context.Context, perm Permission) error {
    namespace := perm.Namespace
    if namespace == "" {
        namespace = rv.namespace
    }
    
    for _, verb := range perm.Verbs {
        review := &authv1.SelfSubjectAccessReview{
            Spec: authv1.SelfSubjectAccessReviewSpec{
                ResourceAttributes: &authv1.ResourceAttributes{
                    Namespace: namespace,
                    Verb:      verb,
                    Group:     perm.Group,
                    Resource:  perm.Resource,
                },
            },
        }
        
        result, err := rv.client.AuthorizationV1().SelfSubjectAccessReviews().Create(ctx, review, metav1.CreateOptions{})
        if err != nil {
            return fmt.Errorf("permission check failed: %w", err)
        }
        
        if !result.Status.Allowed {
            return fmt.Errorf("permission denied: %s %s in %s (reason: %s)", 
                verb, perm.Resource, namespace, result.Status.Reason)
        }
        
        rv.logger.Debug("Permission validated",
            zap.String("verb", verb),
            zap.String("resource", perm.Resource),
            zap.String("namespace", namespace))
    }
    
    return nil
}
```

### Step 3: Implement Comment Validation and Sanitization

#### 3.1 Input Validation Framework
```go
package security

import (
    "fmt"
    "html"
    "regexp"
    "strings"
    "unicode/utf8"
    
    "go.uber.org/zap"
)

type InputValidator struct {
    logger           *zap.Logger
    maxCommentLength int
    allowedUsers     map[string]bool
    maliciousPatterns []*regexp.Regexp
}

func NewInputValidator(logger *zap.Logger, allowedUsers []string) *InputValidator {
    userMap := make(map[string]bool)
    for _, user := range allowedUsers {
        userMap[strings.ToLower(user)] = true
    }
    
    // Define patterns for potentially malicious content
    maliciousPatterns := []*regexp.Regexp{
        regexp.MustCompile(`(?i)<script[^>]*>.*?</script>`), // Script tags
        regexp.MustCompile(`(?i)javascript:`),               // JavaScript URLs
        regexp.MustCompile(`(?i)on\w+\s*=`),                // Event handlers
        regexp.MustCompile(`\$\{.*?\}`),                     // Template injection
        regexp.MustCompile(`\{\{.*?\}\}`),                   // Mustache injection
        regexp.MustCompile(`(?i)eval\s*\(`),                // Eval calls
        regexp.MustCompile(`(?i)exec\s*\(`),                // Exec calls
    }
    
    return &InputValidator{
        logger:            logger,
        maxCommentLength:  50000, // 50KB max comment size
        allowedUsers:      userMap,
        maliciousPatterns: maliciousPatterns,
    }
}

type StructuredFeedback struct {
    Description string
    Severity    FeedbackSeverity
    IssueType   IssueType
    Author      string
    CommentID   string
    Timestamp   time.Time
}

func (iv *InputValidator) ValidateFeedback(feedback *StructuredFeedback, author string) error {
    // Check authorized reviewers
    if !iv.isAuthorizedReviewer(author) {
        iv.logger.Warn("Unauthorized feedback source",
            zap.String("author", author),
            zap.String("comment_id", feedback.CommentID))
        return fmt.Errorf("unauthorized feedback source: %s", author)
    }
    
    // Validate comment length
    if len(feedback.Description) > iv.maxCommentLength {
        iv.logger.Warn("Comment exceeds maximum length",
            zap.String("author", author),
            zap.Int("length", len(feedback.Description)),
            zap.Int("max_length", iv.maxCommentLength))
        return fmt.Errorf("comment exceeds maximum length of %d characters", iv.maxCommentLength)
    }
    
    // Check for valid UTF-8
    if !utf8.ValidString(feedback.Description) {
        return fmt.Errorf("comment contains invalid UTF-8 characters")
    }
    
    // Sanitize input
    sanitized := iv.sanitizeInput(feedback.Description)
    if sanitized != feedback.Description {
        iv.logger.Info("Input sanitized",
            zap.String("author", author),
            zap.String("comment_id", feedback.CommentID))
        feedback.Description = sanitized
    }
    
    // Check for malicious patterns
    if iv.containsMaliciousPattern(feedback.Description) {
        iv.logger.Warn("Malicious content detected",
            zap.String("author", author),
            zap.String("comment_id", feedback.CommentID))
        return fmt.Errorf("invalid feedback content detected")
    }
    
    // Validate severity and type
    if !iv.isValidSeverity(feedback.Severity) {
        return fmt.Errorf("invalid feedback severity: %v", feedback.Severity)
    }
    
    if !iv.isValidIssueType(feedback.IssueType) {
        return fmt.Errorf("invalid issue type: %v", feedback.IssueType)
    }
    
    iv.logger.Debug("Feedback validation successful",
        zap.String("author", author),
        zap.String("comment_id", feedback.CommentID))
    
    return nil
}

func (iv *InputValidator) isAuthorizedReviewer(author string) bool {
    // Check against authorized users list
    if iv.allowedUsers[strings.ToLower(author)] {
        return true
    }
    
    // Check for bot accounts (5DLabs-Tess, etc.)
    authorLower := strings.ToLower(author)
    if strings.HasPrefix(authorLower, "5dlabs-") || strings.HasSuffix(authorLower, "[bot]") {
        return true
    }
    
    return false
}

func (iv *InputValidator) sanitizeInput(input string) string {
    // HTML escape dangerous characters
    sanitized := html.EscapeString(input)
    
    // Remove or escape shell metacharacters
    shellMetachars := map[string]string{
        "|":  "&#124;",
        "&":  "&amp;",
        ";":  "&#59;",
        "(":  "&#40;",
        ")":  "&#41;",
        "`":  "&#96;",
        "$":  "&#36;",
    }
    
    for char, replacement := range shellMetachars {
        sanitized = strings.ReplaceAll(sanitized, char, replacement)
    }
    
    return sanitized
}

func (iv *InputValidator) containsMaliciousPattern(input string) bool {
    for _, pattern := range iv.maliciousPatterns {
        if pattern.MatchString(input) {
            return true
        }
    }
    return false
}

func (iv *InputValidator) isValidSeverity(severity FeedbackSeverity) bool {
    validSeverities := []FeedbackSeverity{
        FeedbackSeverity.Low,
        FeedbackSeverity.Medium,
        FeedbackSeverity.High,
        FeedbackSeverity.Critical,
    }
    
    for _, valid := range validSeverities {
        if severity == valid {
            return true
        }
    }
    return false
}

func (iv *InputValidator) isValidIssueType(issueType IssueType) bool {
    validTypes := []IssueType{
        IssueType.Bug,
        IssueType.Enhancement,
        IssueType.Documentation,
        IssueType.Performance,
        IssueType.Security,
    }
    
    for _, valid := range validTypes {
        if issueType == valid {
            return true
        }
    }
    return false
}
```

### Step 4: Add Rate Limiting for API Calls

#### 4.1 Rate Limiting Implementation
```go
package security

import (
    "context"
    "fmt"
    "sync"
    "time"
    
    "golang.org/x/time/rate"
    "go.uber.org/zap"
)

type RateLimiter struct {
    limiters map[string]*rate.Limiter
    mutex    sync.RWMutex
    logger   *zap.Logger
    
    // Configuration
    requestsPerMin int
    burstSize      int
    cleanupInterval time.Duration
}

func NewRateLimiter(requestsPerMin, burstSize int, logger *zap.Logger) *RateLimiter {
    rl := &RateLimiter{
        limiters:        make(map[string]*rate.Limiter),
        logger:          logger,
        requestsPerMin:  requestsPerMin,
        burstSize:       burstSize,
        cleanupInterval: 15 * time.Minute,
    }
    
    // Start cleanup goroutine
    go rl.cleanupRoutine()
    
    return rl
}

func (rl *RateLimiter) AllowRequest(taskID string) error {
    limiter := rl.getLimiter(taskID)
    
    if !limiter.Allow() {
        rl.logger.Warn("Rate limit exceeded",
            zap.String("task_id", taskID),
            zap.Int("requests_per_min", rl.requestsPerMin))
        return fmt.Errorf("rate limit exceeded for task %s", taskID)
    }
    
    rl.logger.Debug("Request allowed",
        zap.String("task_id", taskID))
    
    return nil
}

func (rl *RateLimiter) CheckRateLimit(ctx context.Context, taskID string) error {
    // Wait for permission with context timeout
    limiter := rl.getLimiter(taskID)
    
    err := limiter.Wait(ctx)
    if err != nil {
        rl.logger.Warn("Rate limit wait failed",
            zap.String("task_id", taskID),
            zap.Error(err))
        return fmt.Errorf("rate limit check failed: %w", err)
    }
    
    return nil
}

func (rl *RateLimiter) getLimiter(taskID string) *rate.Limiter {
    rl.mutex.RLock()
    limiter, exists := rl.limiters[taskID]
    rl.mutex.RUnlock()
    
    if exists {
        return limiter
    }
    
    // Create new limiter
    rl.mutex.Lock()
    defer rl.mutex.Unlock()
    
    // Double-check after acquiring write lock
    if limiter, exists := rl.limiters[taskID]; exists {
        return limiter
    }
    
    // Create rate limiter: requests per minute with burst capacity
    rateLimit := rate.Limit(float64(rl.requestsPerMin) / 60.0) // Convert to per-second
    limiter = rate.NewLimiter(rateLimit, rl.burstSize)
    rl.limiters[taskID] = limiter
    
    rl.logger.Debug("Created rate limiter",
        zap.String("task_id", taskID),
        zap.Float64("rate_per_sec", float64(rateLimit)),
        zap.Int("burst", rl.burstSize))
    
    return limiter
}

func (rl *RateLimiter) cleanupRoutine() {
    ticker := time.NewTicker(rl.cleanupInterval)
    defer ticker.Stop()
    
    for range ticker.C {
        rl.mutex.Lock()
        
        // Remove unused limiters (simple approach - remove all, they'll be recreated if needed)
        // In production, you might want more sophisticated cleanup based on last access time
        rl.limiters = make(map[string]*rate.Limiter)
        
        rl.mutex.Unlock()
        
        rl.logger.Debug("Rate limiter cleanup completed")
    }
}

// GitHub API specific rate limiter
type GitHubRateLimiter struct {
    primaryLimiter   *rate.Limiter
    secondaryLimiter *rate.Limiter
    logger           *zap.Logger
}

func NewGitHubRateLimiter(logger *zap.Logger) *GitHubRateLimiter {
    // GitHub rate limits: 5000 requests/hour for authenticated requests
    // Secondary rate limit: 100 requests/minute for content creation
    primaryRate := rate.Limit(5000.0 / 3600.0)   // ~1.39 requests per second
    secondaryRate := rate.Limit(100.0 / 60.0)    // ~1.67 requests per second
    
    return &GitHubRateLimiter{
        primaryLimiter:   rate.NewLimiter(primaryRate, 100),   // Burst of 100
        secondaryLimiter: rate.NewLimiter(secondaryRate, 10),  // Burst of 10
        logger:           logger,
    }
}

func (grl *GitHubRateLimiter) WaitForAPICall(ctx context.Context, isContentCreation bool) error {
    // Always check primary rate limit
    if err := grl.primaryLimiter.Wait(ctx); err != nil {
        return fmt.Errorf("primary rate limit exceeded: %w", err)
    }
    
    // Check secondary rate limit for content creation operations
    if isContentCreation {
        if err := grl.secondaryLimiter.Wait(ctx); err != nil {
            return fmt.Errorf("secondary rate limit exceeded: %w", err)
        }
    }
    
    return nil
}
```

### Step 5: Implement Audit Logging

#### 5.1 Security Audit System
```go
package security

import (
    "context"
    "encoding/json"
    "time"
    
    "go.uber.org/zap"
)

type AuditLogger struct {
    logger *zap.Logger
}

type AuditEvent struct {
    Timestamp     time.Time              `json:"timestamp"`
    EventType     string                 `json:"event_type"`
    Actor         string                 `json:"actor"`
    Action        string                 `json:"action"`
    Resource      string                 `json:"resource"`
    ResourceID    string                 `json:"resource_id,omitempty"`
    TaskID        string                 `json:"task_id,omitempty"`
    PRNumber      int                    `json:"pr_number,omitempty"`
    Success       bool                   `json:"success"`
    ErrorMessage  string                 `json:"error_message,omitempty"`
    IPAddress     string                 `json:"ip_address,omitempty"`
    UserAgent     string                 `json:"user_agent,omitempty"`
    Metadata      map[string]interface{} `json:"metadata,omitempty"`
    Severity      string                 `json:"severity"`
}

func NewAuditLogger(logger *zap.Logger) *AuditLogger {
    return &AuditLogger{logger: logger}
}

func (al *AuditLogger) LogSecurityEvent(ctx context.Context, event AuditEvent) {
    event.Timestamp = time.Now().UTC()
    
    // Extract additional context if available
    if taskID := ctx.Value("task_id"); taskID != nil {
        if event.TaskID == "" {
            event.TaskID = taskID.(string)
        }
    }
    
    if correlationID := ctx.Value("correlation_id"); correlationID != nil {
        if event.Metadata == nil {
            event.Metadata = make(map[string]interface{})
        }
        event.Metadata["correlation_id"] = correlationID.(string)
    }
    
    // Log with appropriate level based on severity
    fields := al.eventToZapFields(event)
    
    switch event.Severity {
    case "critical":
        al.logger.Error("Security audit event", fields...)
    case "high":
        al.logger.Warn("Security audit event", fields...)
    default:
        al.logger.Info("Security audit event", fields...)
    }
}

func (al *AuditLogger) eventToZapFields(event AuditEvent) []zap.Field {
    fields := []zap.Field{
        zap.Time("audit_timestamp", event.Timestamp),
        zap.String("event_type", event.EventType),
        zap.String("actor", event.Actor),
        zap.String("action", event.Action),
        zap.String("resource", event.Resource),
        zap.Bool("success", event.Success),
        zap.String("severity", event.Severity),
    }
    
    if event.ResourceID != "" {
        fields = append(fields, zap.String("resource_id", event.ResourceID))
    }
    
    if event.TaskID != "" {
        fields = append(fields, zap.String("task_id", event.TaskID))
    }
    
    if event.PRNumber > 0 {
        fields = append(fields, zap.Int("pr_number", event.PRNumber))
    }
    
    if event.ErrorMessage != "" {
        fields = append(fields, zap.String("error_message", event.ErrorMessage))
    }
    
    if event.IPAddress != "" {
        fields = append(fields, zap.String("ip_address", event.IPAddress))
    }
    
    if event.UserAgent != "" {
        fields = append(fields, zap.String("user_agent", event.UserAgent))
    }
    
    if event.Metadata != nil {
        if metadataBytes, err := json.Marshal(event.Metadata); err == nil {
            fields = append(fields, zap.String("metadata", string(metadataBytes)))
        }
    }
    
    return fields
}

// Specific audit logging methods
func (al *AuditLogger) LogAuthenticationEvent(ctx context.Context, actor, action string, success bool, errorMsg string) {
    event := AuditEvent{
        EventType:    "authentication",
        Actor:        actor,
        Action:       action,
        Resource:     "github_token",
        Success:      success,
        ErrorMessage: errorMsg,
        Severity:     al.determineSeverity("authentication", success),
    }
    
    al.LogSecurityEvent(ctx, event)
}

func (al *AuditLogger) LogAuthorizationEvent(ctx context.Context, actor, action, resource string, success bool, errorMsg string) {
    event := AuditEvent{
        EventType:    "authorization",
        Actor:        actor,
        Action:       action,
        Resource:     resource,
        Success:      success,
        ErrorMessage: errorMsg,
        Severity:     al.determineSeverity("authorization", success),
    }
    
    al.LogSecurityEvent(ctx, event)
}

func (al *AuditLogger) LogInputValidationEvent(ctx context.Context, actor, input string, success bool, errorMsg string) {
    event := AuditEvent{
        EventType:    "input_validation",
        Actor:        actor,
        Action:       "validate_input",
        Resource:     "feedback_comment",
        Success:      success,
        ErrorMessage: errorMsg,
        Severity:     al.determineSeverity("input_validation", success),
        Metadata: map[string]interface{}{
            "input_length": len(input),
        },
    }
    
    al.LogSecurityEvent(ctx, event)
}

func (al *AuditLogger) LogRateLimitEvent(ctx context.Context, taskID, action string, success bool, errorMsg string) {
    event := AuditEvent{
        EventType:    "rate_limiting",
        Actor:        "system",
        Action:       action,
        Resource:     "api_call",
        TaskID:       taskID,
        Success:      success,
        ErrorMessage: errorMsg,
        Severity:     al.determineSeverity("rate_limiting", success),
    }
    
    al.LogSecurityEvent(ctx, event)
}

func (al *AuditLogger) LogPrivilegeEscalation(ctx context.Context, actor, attemptedAction, resource string) {
    event := AuditEvent{
        EventType:   "privilege_escalation",
        Actor:       actor,
        Action:      attemptedAction,
        Resource:    resource,
        Success:     false,
        Severity:    "critical",
        Metadata: map[string]interface{}{
            "alert_type": "security_violation",
        },
    }
    
    al.LogSecurityEvent(ctx, event)
}

func (al *AuditLogger) determineSeverity(eventType string, success bool) string {
    if success {
        return "info"
    }
    
    switch eventType {
    case "authentication", "authorization":
        return "high"
    case "input_validation":
        return "medium"
    case "rate_limiting":
        return "low"
    default:
        return "medium"
    }
}
```

### Step 6: Security Integration and Monitoring

#### 6.1 Security Manager Integration
```go
package security

import (
    "context"
    "fmt"
    
    "go.uber.org/zap"
)

type SecurityManager struct {
    tokenManager   *GitHubTokenManager
    rbacValidator  *RBACValidator
    inputValidator *InputValidator
    rateLimiter    *RateLimiter
    auditLogger    *AuditLogger
    logger         *zap.Logger
}

func NewSecurityManager(
    tokenManager *GitHubTokenManager,
    rbacValidator *RBACValidator,
    inputValidator *InputValidator,
    rateLimiter *RateLimiter,
    auditLogger *AuditLogger,
    logger *zap.Logger,
) *SecurityManager {
    return &SecurityManager{
        tokenManager:   tokenManager,
        rbacValidator:  rbacValidator,
        inputValidator: inputValidator,
        rateLimiter:    rateLimiter,
        auditLogger:    auditLogger,
        logger:         logger,
    }
}

func (sm *SecurityManager) ValidateOperation(ctx context.Context, operation SecurityOperation) error {
    // Rate limiting check
    if err := sm.rateLimiter.CheckRateLimit(ctx, operation.TaskID); err != nil {
        sm.auditLogger.LogRateLimitEvent(ctx, operation.TaskID, operation.Action, false, err.Error())
        return fmt.Errorf("rate limit check failed: %w", err)
    }
    
    // Input validation
    if operation.Feedback != nil {
        if err := sm.inputValidator.ValidateFeedback(operation.Feedback, operation.Actor); err != nil {
            sm.auditLogger.LogInputValidationEvent(ctx, operation.Actor, operation.Feedback.Description, false, err.Error())
            return fmt.Errorf("input validation failed: %w", err)
        }
        sm.auditLogger.LogInputValidationEvent(ctx, operation.Actor, operation.Feedback.Description, true, "")
    }
    
    // GitHub permissions validation
    if operation.RequiresGitHub {
        if err := sm.tokenManager.ValidatePermissions(ctx, operation.Owner, operation.Repo); err != nil {
            sm.auditLogger.LogAuthorizationEvent(ctx, operation.Actor, operation.Action, "github_repo", false, err.Error())
            return fmt.Errorf("GitHub authorization failed: %w", err)
        }
        sm.auditLogger.LogAuthorizationEvent(ctx, operation.Actor, operation.Action, "github_repo", true, "")
    }
    
    // Kubernetes RBAC validation (periodic check)
    // This could be cached and checked less frequently
    
    sm.logger.Info("Security validation passed",
        zap.String("task_id", operation.TaskID),
        zap.String("action", operation.Action),
        zap.String("actor", operation.Actor))
    
    return nil
}

type SecurityOperation struct {
    TaskID         string
    Action         string
    Actor          string
    Owner          string
    Repo           string
    RequiresGitHub bool
    Feedback       *StructuredFeedback
}

func (sm *SecurityManager) InitializeSecurityChecks(ctx context.Context) error {
    sm.logger.Info("Initializing security checks...")
    
    // Validate RBAC permissions
    if err := sm.rbacValidator.ValidatePermissions(ctx); err != nil {
        sm.auditLogger.LogAuthorizationEvent(ctx, "system", "rbac_validation", "kubernetes", false, err.Error())
        return fmt.Errorf("RBAC validation failed: %w", err)
    }
    
    sm.auditLogger.LogAuthorizationEvent(ctx, "system", "rbac_validation", "kubernetes", true, "")
    sm.logger.Info("Security initialization completed successfully")
    
    return nil
}
```

## Security Testing Strategy

### Penetration Testing
1. **Input Validation Testing**: Test with malicious payloads, XSS attempts, injection attacks
2. **Authentication Testing**: Test token validation, scope checking, expired tokens
3. **Authorization Testing**: Test RBAC enforcement, privilege escalation attempts
4. **Rate Limit Testing**: Test bypass attempts, distributed attacks, burst scenarios
5. **Audit Log Testing**: Verify completeness, tamper resistance, log injection attempts

### Security Monitoring
1. **Real-time Alerts**: Failed authentication, authorization violations, rate limit breaches
2. **Anomaly Detection**: Unusual access patterns, privilege escalation attempts
3. **Compliance Monitoring**: Audit log completeness, data retention compliance
4. **Vulnerability Scanning**: Regular security scans of dependencies and configurations

## Compliance Considerations

### Data Protection
- No PII stored in logs or state data
- Secure token storage and rotation
- Audit trail completeness for compliance requirements
- Data retention policies aligned with regulations

### Access Controls
- Principle of least privilege enforced
- Regular access reviews and updates
- Multi-factor authentication requirements
- Audit logging for all privileged operations

## Success Criteria
- GitHub token permissions configured with minimal required scope
- Kubernetes RBAC provides least privilege access to required resources
- Input validation prevents all tested injection attacks
- Rate limiting prevents resource exhaustion and abuse
- Audit logging captures all security-relevant events
- Security controls integrate seamlessly without impacting functionality
- Security monitoring provides proactive threat detection
- Compliance requirements met for audit and data protection