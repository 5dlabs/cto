# Task ID: 2
# Title: Setup Argo Events Infrastructure
# Status: in-progress
# Dependencies: None
# Priority: high
# Description: Create and configure Argo Events Sensors for multi-agent workflow orchestration using existing EventBus and EventSource infrastructure
# Details:
Infrastructure (EventBus, EventSource, test sensor) is already deployed and functional. Create four specialized Sensors for agent coordination using supported Argo Events patterns:

1) Multi-agent workflow resume sensor (PR created) resumes specific workflows via `argoWorkflow.operation: resume` with parameterized `args.0 = play-task-<id>-workflow` built from PR labels.

2) Ready-for-QA label sensor resumes via the same resume + args pattern when Cleo adds the label, validating actor and action.

3) PR approval sensor resumes via resume + args when Tess approves a PR.

4) Implementation agent remediation sensor submits a short-lived cleanup workflow to delete Cleo/Tess `CodeRun` CRs for the specific task using `kubectl delete -l task-id=<id>`; avoids unsupported delete/patch operations in sensors.

All sensors use the existing 'github' EventSource and 'argo' EventBus. Avoid dynamic `labelSelector` and k8s `delete` operations in sensors; use parameterized resume and workflow-submitted cleanup instead.

# Test Strategy:
Verify each Sensor is deployed with kubectl get sensors -n argo. Test webhook event processing by triggering actual GitHub events (PR creation, labeling, approval, push). Confirm Sensors correctly correlate events using kubectl logs. Validate workflow resumption with suspended test workflows. Test remediation sensor properly cancels running workflows and restarts pipeline. Monitor for rate limiting issues.

# Subtasks:
## 1. Create multi-agent workflow resume sensor [pending]
### Dependencies: None
### Description: Build Sensor for PR created events to resume workflows after Rex
### Details:


## 2. Create ready-for-QA label sensor [pending]
### Dependencies: None
### Description: Build Sensor for PR labeled events to resume workflows after Cleo
### Details:


## 3. Create PR approval sensor [pending]
### Dependencies: None
### Description: Build Sensor for PR approved events to resume workflows after Tess
### Details:


## 4. Create Rex remediation sensor [pending]
### Dependencies: None
### Description: Build Sensor to detect Rex pushes, cancel Cleo/Tess workflows, and restart QA pipeline
### Details:


