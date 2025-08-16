# Autonomous Implementation Prompt: Argo Events GitHub Integration

## Mission Statement
Implement comprehensive GitHub webhook integration using Argo Events that automatically routes repository events to appropriate AI agent workflows with proper parameter extraction, rate limiting, and security validation.

## Technical Requirements
1. **GitHub EventSource** with webhook secret from External Secrets
2. **Event Sensors** for PR, issues, comments, CI failures, and security events
3. **Parameter mapping** with consistent contract across all workflows
4. **Rate limiting** and concurrency controls via semaphores
5. **Security validation** of webhook signatures

## Key Implementation Points
- EventSource handles all GitHub event types with wildcard repository matching
- Sensors filter events and extract parameters using JSONPath and CEL expressions
- All workflows triggered via WorkflowTemplate references with standardized parameters
- Rate limiting prevents event storms (20-30 requests/minute per sensor)
- RBAC provides minimal permissions for workflow creation only

## Success Criteria
- All GitHub event types route to correct workflows
- Parameter extraction works for all event payloads
- Rate limiting and security validation functional
- Integration tests pass with real GitHub webhooks
- No permission escalation or security vulnerabilities