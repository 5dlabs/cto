# Toolman Guide: MCP GitHub Comments Tool

## Overview
Guide for using the GitHub comment retrieval MCP tool for accessing PR conversation context.

## Available Tools

### MCP GitHub Comments Service
**Purpose**: Retrieve and normalize PR comments for AI agent context

#### Get Combined Comments
```bash
# Get all comments for PR
curl "http://mcp-github-comments/repos/myorg/myrepo/pulls/123/comments/combined"

# Get with summarization
curl "http://mcp-github-comments/repos/myorg/myrepo/pulls/123/comments/combined?summarize=compact&max_items=100"
```

## Usage in Workflows

### Rex Agent Integration
When Rex agent processes comment events, comments are automatically fetched:

```yaml
# In coderun-template for comment events
- name: fetch-comments
  container:
    image: curlimages/curl:latest
    command: ["/bin/sh", "-c"]
    args:
      - curl -s "http://mcp-github-comments/repos/{{workflow.parameters.owner}}/{{workflow.parameters.repo}}/pulls/{{workflow.parameters.pr}}/comments/combined?summarize=compact" -o /work/comments.json
```

## Testing Tools

### Comment Tester
```bash
# Test comment retrieval
./scripts/test-comment-retrieval.sh --pr 123 --mode combined

# Test with different summarization modes
./scripts/test-comment-retrieval.sh --pr 123 --summarize brief
```

## Troubleshooting

### Common Issues
- **Service unavailable**: Check pod status and token mounting
- **API rate limits**: Monitor GitHub API usage and implement backoff
- **Missing comments**: Verify PR number and repository access

### Debug Commands
```bash
# Check service health
curl http://mcp-github-comments/healthz

# Test token access
kubectl exec deployment/mcp-github-comments -- cat /var/run/github/token

# Check service logs
kubectl logs -f deployment/mcp-github-comments
```