# Autonomous Implementation Prompt: MCP GitHub Comments Tool

## Mission Statement
Implement a lightweight MCP tool for efficient PR comment retrieval that enables AI agents to access conversation context when responding to feedback.

## Technical Requirements
1. **HTTP Service** (Go/Node.js) with REST endpoints for comment retrieval
2. **GitHub API Integration** using installation tokens from Task 2
3. **Response Normalization** with unified schema across comment types  
4. **ETag Caching** for performance optimization
5. **MCP Integration** via requirements.yaml registration

## Key Implementation Points
- Endpoints: /repos/{owner}/{repo}/pulls/{number}/comments/combined
- Token reading from /var/run/github/token (mounted from token generator)
- Normalized response with author, timestamps, content, threading info
- Optional summarization modes (none/brief/compact) for token budgets
- Integration with coderun-template for comment-driven workflows

## Success Criteria
- Service responds correctly to comment retrieval requests
- ETag caching reduces API calls for unchanged threads
- Rex agent can access comment context via MCP tool
- Performance testing shows >2x improvement over individual API calls