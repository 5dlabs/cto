# Task ID: 4
# Title: Implement Agent-Specific PVC Naming
# Status: pending
# Dependencies: 1
# Priority: high
# Description: Modify the Rust controller to extract agent names from github_app field and implement workspace-{service}-{agent} PVC naming pattern
# Details:
Update controller/src/tasks/code/resources.rs to parse github_app field (e.g., '5DLabs-Rex' → 'rex'). Implement extract_agent_name() function using regex or string manipulation. Modify PVC creation logic to use format!('workspace-{}-{}', code_run.spec.service, agent_name). Ensure backward compatibility by checking for existing PVCs with old naming. Update controller reconciliation to handle both naming patterns during transition period. Consider using kube-rs PersistentVolumeClaim API for idempotent creation.

# Test Strategy:
Unit test extract_agent_name() with various GitHub App formats. Integration test PVC creation with different agents. Verify workspace isolation between Rex, Cleo, and Tess. Confirm existing workflows continue working with legacy PVC names.
