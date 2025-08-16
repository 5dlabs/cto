# Task 7: MCP Tool for PR Comment Retrieval

## Overview
This task implements a lightweight MCP (Model Context Protocol) tool for efficient retrieval of PR and review comments, enabling AI agents like Rex to access conversation context when responding to feedback. The service provides normalized, paginated access to GitHub comments with optional summarization.

## Architecture
- **HTTP Service**: Lightweight Go/Node.js service exposing REST endpoints
- **GitHub API Integration**: Reads installation tokens from token generator (Task 2)
- **Response Normalization**: Unified comment format across different GitHub comment types
- **Caching**: ETag-based caching for performance optimization
- **MCP Integration**: Registered as tool in requirements.yaml for agent access

## Key Features
### Comment Types Supported
- **Review Comments**: Code-specific comments on PR diffs
- **Issue Comments**: General PR discussion comments
- **Timeline Events**: PR state changes, reviews, commits (optional)

### Response Processing
- **Normalization**: Consistent schema across comment types
- **Sorting**: Chronological ordering with threading support
- **Summarization**: Token-budget friendly summaries (brief/compact modes)
- **Pagination**: Efficient handling of large comment threads

## Implementation

### Service Endpoints
```go
// GET /repos/{owner}/{repo}/pulls/{number}/comments (review comments)
// GET /repos/{owner}/{repo}/issues/{number}/comments (issue comments)
// GET /repos/{owner}/{repo}/pulls/{number}/comments/combined (all types)
```

### Normalized Response Schema
```json
{
  "comments": [
    {
      "id": "string",
      "type": "review|issue|timeline",
      "author": {"login": "string", "type": "User|Bot"},
      "created_at": "RFC3339",
      "updated_at": "RFC3339", 
      "body": "string",
      "url": "string",
      "html_url": "string",
      "file_path": "string?",
      "line": "number?",
      "commit_id": "string?",
      "in_reply_to_id": "string?",
      "review_state": "approved|changes_requested|commented?"
    }
  ],
  "summary": {
    "text": "string",
    "stats": {
      "total_comments": "number",
      "review_comments": "number", 
      "issue_comments": "number",
      "characters": "number"
    }
  }
}
```

### GitHub API Integration
```go
func (s *Service) fetchComments(owner, repo, number string) (*CommentResponse, error) {
    token, err := s.readToken("/var/run/github/token")
    if err != nil {
        return nil, err
    }
    
    client := github.NewClient(nil).WithAuthToken(token)
    
    // Fetch review comments
    reviewComments, err := s.fetchAllPages(client.PullRequests.ListComments, 
        context.Background(), owner, repo, number)
    
    // Fetch issue comments  
    issueComments, err := s.fetchAllPages(client.Issues.ListComments,
        context.Background(), owner, repo, number)
    
    return s.normalizeAndSort(reviewComments, issueComments), nil
}
```

### ETag Caching Implementation
```go
func (s *Service) handleWithETag(w http.ResponseWriter, r *http.Request, 
    key string, fetcher func() ([]byte, error)) {
    
    ifNoneMatch := r.Header.Get("If-None-Match")
    
    if etag, exists := s.cache.Get(key); exists && etag == ifNoneMatch {
        w.WriteHeader(http.StatusNotModified)
        return
    }
    
    data, err := fetcher()
    if err != nil {
        http.Error(w, err.Error(), http.StatusInternalServerError)
        return
    }
    
    etag := fmt.Sprintf(`"${hash(data)}"`)
    s.cache.Set(key, etag)
    
    w.Header().Set("ETag", etag)
    w.Header().Set("Content-Type", "application/json")
    w.Write(data)
}
```

### Workflow Integration
In coderun-template, when github-app=rex and event is comment-related:

```yaml
- name: fetch-comments
  container:
    image: curlimages/curl:latest
    command: ["/bin/sh", "-c"]
    args:
      - |
        curl -s "http://mcp-github-comments/repos/{{workflow.parameters.owner}}/{{workflow.parameters.repo}}/pulls/{{workflow.parameters.pr}}/comments/combined?summarize=compact" \
          -o /work/comments.json
  outputs:
    parameters:
      - name: comments_path
        value: "/work/comments.json"
```

## MCP Requirements Integration
```yaml
# requirements.yaml
tools:
  - name: mcp-github-comments
    baseUrl: http://mcp-github-comments
    endpoints:
      - path: /repos/{owner}/{repo}/pulls/{number}/comments/combined
        method: GET
        description: Get all PR comments with optional summarization
        parameters:
          summarize: "none|brief|compact"
          include: "review,issue,timeline"
          max_items: "2000"
```

## Deployment Configuration
### Kubernetes Manifests
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: mcp-github-comments
spec:
  replicas: 2
  selector:
    matchLabels:
      app: mcp-github-comments
  template:
    spec:
      containers:
      - name: service
        image: ghcr.io/myorg/mcp-github-comments:latest
        ports:
        - containerPort: 8080
        env:
        - name: GITHUB_TOKEN_FILE
          value: /var/run/github/token
        volumeMounts:
        - name: gh-token
          mountPath: /var/run/github
          readOnly: true
        readinessProbe:
          httpGet:
            path: /healthz
            port: 8080
        livenessProbe:
          httpGet:
            path: /healthz
            port: 8080
      initContainers:
      - name: gh-token
        image: ghcr.io/myorg/ghapp-token-gen:latest
        env:
        - name: APP_ID
          valueFrom:
            secretKeyRef:
              name: github-app-rex
              key: appId
        - name: PRIVATE_KEY
          valueFrom:
            secretKeyRef:
              name: github-app-rex
              key: privateKey
        volumeMounts:
        - name: gh-token
          mountPath: /var/run/github
      volumes:
      - name: gh-token
        emptyDir: {}
---
apiVersion: v1
kind: Service
metadata:
  name: mcp-github-comments
spec:
  selector:
    app: mcp-github-comments
  ports:
  - port: 80
    targetPort: 8080
```

## Security and Performance

### Security Features
- **Token Security**: Never logs GitHub tokens or sensitive comment content
- **Rate Limiting**: Respects GitHub API rate limits with backoff
- **Input Validation**: Validates owner/repo/number parameters
- **Memory Safety**: Bounded response sizes and timeouts

### Performance Optimizations
- **ETag Caching**: Reduces API calls for unchanged comment threads
- **Pagination Handling**: Efficient processing of large comment threads
- **Connection Pooling**: Reuses HTTP connections to GitHub API
- **Circuit Breaker**: Handles GitHub API failures gracefully

### Monitoring Integration
```go
func (s *Service) instrumentHandler(name string, handler http.Handler) http.Handler {
    return promhttp.InstrumentHandlerDuration(
        s.metrics.requestDuration.MustCurryWith(prometheus.Labels{"handler": name}),
        promhttp.InstrumentHandlerCounter(
            s.metrics.requestsTotal.MustCurryWith(prometheus.Labels{"handler": name}),
            handler,
        ),
    )
}
```

## Testing Strategy

### Unit Tests
- Comment normalization and merging logic
- Pagination handling with Link headers
- ETag cache hit/miss scenarios
- Error handling for GitHub API failures

### Integration Tests
- End-to-end comment retrieval with real GitHub API
- Workflow integration with coderun-template
- Rate limiting behavior under load
- Token refresh and rotation scenarios

### Performance Tests
- Large comment thread processing (>1000 comments)
- Concurrent request handling
- Memory usage with varying comment sizes
- Cache effectiveness measurement

## Dependencies
- GitHub App token generator (Task 2)
- External Secrets Operator for webhook secrets
- Kubernetes cluster with ingress capability
- Container registry for service image
- Monitoring infrastructure (Prometheus/Grafana)

## References
- [GitHub REST API - Comments](https://docs.github.com/en/rest/pulls/comments)
- [Model Context Protocol Specification](https://modelcontextprotocol.io/specification/)
- [HTTP ETag Caching](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/ETag)