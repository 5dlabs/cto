# Autonomous Agent Prompt: Setup GitHub Webhook Infrastructure for Remediation Loop

## Your Mission
You are tasked with creating and deploying an Argo Events Sensor that will detect PR comments containing QA feedback and trigger automated remediation workflows. This sensor must integrate with existing GitHub webhook infrastructure and generate CodeRun resources for the Rex agent.

## Context
The platform has an existing GitHub webhook EventSource and multiple working sensors. Your job is to add a new remediation-specific sensor that:
- Listens for PR comments with '🔴 Required Changes' format
- Extracts task IDs from PR labels
- Creates CodeRun resources with remediation configuration
- Integrates seamlessly with existing infrastructure

## Required Actions

### 1. Create Sensor Configuration
Generate a new Argo Events Sensor YAML file at `infra/gitops/resources/github-webhooks/remediation-feedback-sensor.yaml` with:
- Event source dependency on existing `github-eventsource`
- Filtering for issue_comment events containing '🔴 Required Changes'
- Task ID extraction from PR labels
- CodeRun resource generation trigger

### 2. Implement Event Filtering
Configure comprehensive event filtering:
- Filter for 'created' action on issue_comment events
- Validate comment contains feedback marker
- Ensure comment is on a pull request (not issue)
- Optionally validate authorized comment authors

### 3. Configure CodeRun Generation
Set up the Kubernetes resource trigger to:
- Generate CodeRun CRDs in agent-platform namespace
- Set REMEDIATION_MODE=true environment variable
- Pass comment ID, PR number, and task ID as parameters
- Configure appropriate labels and metadata
- Enable session continuation for Rex agent

### 4. Deploy and Validate
- Apply the sensor configuration to the cluster
- Verify sensor pod starts successfully
- Check connection to existing EventSource
- Validate RBAC permissions for CodeRun creation
- Test event flow from webhook to sensor

### 5. Test End-to-End Flow
- Create test PR with task label
- Post comment with required feedback format
- Verify sensor processes the event
- Confirm CodeRun creation with correct parameters
- Test edge cases and error conditions

## Technical Requirements

### Sensor Specification
```yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: pr-comment-remediation
  namespace: argo-events
```

### Event Filtering Rules
- Event type: `issue_comment`
- Action: `created`
- Comment body contains: `🔴 Required Changes`
- Issue has pull_request field (not null)

### CodeRun Resource Template
- Kind: `CodeRun`
- Namespace: `agent-platform`
- GitHub App: `5DLabs-Rex`
- Environment variables:
  - `REMEDIATION_MODE=true`
  - `FEEDBACK_COMMENT_ID`
  - `ITERATION_COUNT`

### Integration Points
- Existing EventSource: `infra/gitops/resources/github-webhooks/eventsource.yaml`
- Service Account: `argo-events-sa`
- Target namespace: `agent-platform`

## Implementation Checklist

- [ ] Create sensor YAML configuration file
- [ ] Configure event source dependency
- [ ] Implement event filtering logic
- [ ] Set up CodeRun resource trigger
- [ ] Add task ID extraction logic
- [ ] Configure environment variables
- [ ] Deploy sensor to cluster
- [ ] Verify sensor pod status
- [ ] Test webhook event processing
- [ ] Validate CodeRun creation
- [ ] Test with real PR comment
- [ ] Document configuration

## Expected Outputs

1. **Sensor Configuration File**: Complete YAML at specified location
2. **Deployed Sensor**: Running pod in argo-events namespace
3. **Working Integration**: Events flow from GitHub to CodeRun creation
4. **Test Results**: Successful end-to-end test with PR comment
5. **Documentation**: Configuration details and troubleshooting guide

## Success Validation

Your implementation is successful when:
1. Sensor pod is running without errors
2. PR comments with feedback trigger sensor
3. CodeRun resources are created with correct configuration
4. Rex agent receives remediation context
5. No interference with existing sensors
6. All test cases pass

## Common Pitfalls to Avoid

- Don't hardcode namespace or service account names
- Ensure proper escaping of special characters in regex
- Validate JSON paths before deployment
- Test RBAC permissions before assuming they work
- Handle edge cases like missing labels or malformed comments
- Avoid blocking existing sensor operations

## Resources and References

- Argo Events Documentation: https://argoproj.github.io/argo-events/
- Existing sensor examples in `infra/gitops/resources/github-webhooks/`
- CodeRun CRD specification in platform documentation
- GitHub webhook event payload reference

## Support and Troubleshooting

If you encounter issues:
1. Check sensor logs: `kubectl logs -n argo-events deployment/pr-comment-remediation-sensor`
2. Verify EventSource connection: `kubectl get eventsources -n argo-events`
3. Review webhook delivery in GitHub settings
4. Validate RBAC: `kubectl auth can-i create coderuns --as=system:serviceaccount:argo-events:argo-events-sa -n agent-platform`
5. Test with simplified filter conditions first

Begin by examining the existing sensor configurations in the github-webhooks directory to understand the patterns and conventions used. Then create your sensor following the same patterns while adding the remediation-specific logic.