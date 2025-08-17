# Task ID: 2
# Title: Setup Argo Events Infrastructure
# Status: in-progress
# Dependencies: None
# Priority: high
# Description: Create and configure Argo Events Sensors for multi-agent workflow orchestration using existing EventBus and EventSource infrastructure
# Details:
Infrastructure (EventBus, EventSource, test sensor) is already deployed and functional. Create four specialized Sensors for agent coordination: 1) Multi-agent workflow resume sensor to handle PR creation events and resume workflows after Rex completes, 2) Ready-for-QA label sensor to detect PR labeling and resume after Cleo, 3) PR approval sensor to handle approval events and resume after Tess, 4) Rex remediation sensor to detect Rex pushes and cancel/restart the QA pipeline. All sensors should use the existing 'github' EventSource and 'argo' EventBus. Reference github-demo-sensor.yaml for patterns. Focus on proper webhook payload field correlation and workflow label selectors for targeting specific suspended workflows.

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


