# Acceptance Criteria: MCP GitHub Comments Tool

## Service Implementation
- [ ] HTTP service running on port 8080 with health endpoint
- [ ] Endpoints for review comments, issue comments, and combined retrieval  
- [ ] GitHub API integration using installation tokens
- [ ] Request/response logging without sensitive data

## Response Processing
- [ ] Normalized schema across all comment types
- [ ] Chronological sorting with threading support
- [ ] Optional summarization modes (brief/compact)
- [ ] Pagination handling for large comment threads

## Performance and Caching
- [ ] ETag-based caching reduces duplicate API calls
- [ ] Response time under 2 seconds for typical comment threads
- [ ] Memory usage stable under concurrent requests
- [ ] Circuit breaker for GitHub API failures

## Integration
- [ ] MCP tool registration in requirements.yaml
- [ ] coderun-template integration for comment events
- [ ] Token mounting from GitHub App token generator
- [ ] Proper RBAC and security configurations

## Testing
- [ ] Unit tests for normalization and pagination
- [ ] Integration tests with real GitHub API
- [ ] Performance tests with large comment threads
- [ ] End-to-end workflow integration tests