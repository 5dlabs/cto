# Acceptance Criteria: Argo Events GitHub Integration

## EventSource Configuration
- [ ] GitHub EventSource receives webhooks at /events endpoint
- [ ] Webhook secret validation using External Secrets
- [ ] All required event types configured (pull_request, issues, comments, push, workflow_run, security_advisory)
- [ ] Wildcard repository matching for organization-wide coverage

## Sensor Event Routing
- [ ] PR events trigger pr-validation WorkflowTemplate
- [ ] Issue comments trigger coderun-template with rex agent
- [ ] CI failures trigger coderun-template with triage agent  
- [ ] New issues trigger implementation-dag WorkflowTemplate
- [ ] Security events trigger coderun-template with security agent

## Parameter Extraction
- [ ] All sensors extract standard parameters (owner, repo, pr, branch, event)
- [ ] Comment sensors include commentBody and includeComments parameters
- [ ] Agent parameter correctly set for each workflow type
- [ ] Event payload preserved as full JSON string

## Rate Limiting and Security
- [ ] Each sensor has appropriate rate limits (20-30 requests/minute)
- [ ] Webhook signatures validated against configured secret
- [ ] Invalid signatures rejected with appropriate error codes
- [ ] Semaphore limits prevent resource exhaustion

## Integration Testing
- [ ] PR opened/updated events create pr-validation workflows
- [ ] Issue comments create rex workflows with comment context
- [ ] Failed CI checks create triage workflows
- [ ] Security advisories create security agent workflows
- [ ] Rate limiting triggers under load testing

## RBAC and Security
- [ ] Service account has minimal permissions (workflow creation only)
- [ ] Cannot access secrets outside argo-events namespace
- [ ] Network policies restrict unnecessary communication
- [ ] Webhook endpoint accessible only from GitHub IPs