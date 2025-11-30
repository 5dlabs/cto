# Test Scenario: Introduce Bad Git Operation in Rex Template

## Objective

Introduce a deliberate git error into Rex's container template to test the alert monitoring system. The error should be realistic enough that Factory needs to analyze logs to understand what went wrong.

## Task

1. Create a new branch: `test/rex-git-failure-simulation`

2. Edit the file: `infra/charts/controller/agent-templates/code/factory/container-rex.sh.hbs`

3. Add the following BEFORE the `factory_container_base` partial include:

```handlebars
#!/bin/bash
# ============================================================================
# DELIBERATE BUG FOR ALERT SYSTEM TESTING
# This simulates a git authentication/permission failure
# The monitor should detect this failure and Factory should analyze the cause
# ============================================================================

echo "🧪 [TEST] Simulating git operation failure..."

# Attempt to push to a non-existent remote - this will fail
git push nonexistent-remote main 2>&1 || {
  echo "❌ ERROR: Failed to push to remote"
  echo "This is a DELIBERATE test failure for the alert monitoring system"
  echo "Factory should identify: bad remote configuration"
  exit 1
}

# The rest of the template follows...
{{> factory_container_base
    agent_banner="🔧 Rex Factory implementation workflow starting"
    agent_completion_message="✅ Rex Factory implementation complete"
    force_writes=true}}
```

4. Commit with message: `test: introduce deliberate git failure in Rex for alert system testing`

5. Create a PR to `main` with:
   - Title: `[TEST] Deliberate Rex Git Failure for Alert Monitoring`
   - Body: 
     ```
     ## Purpose
     This PR introduces a deliberate failure in Rex's startup to test the alert monitoring system.
     
     ## Expected Behavior
     - Rex pod will fail with exit code 1
     - Alert A7 (Pod Failure) should trigger
     - Factory should analyze the logs and identify the bad git remote as the cause
     
     ## After Testing
     This PR should be CLOSED (not merged) after validating the alert system works.
     ```

## DO NOT MERGE

This is a test PR only. Close it after testing is complete.

## Verification

After the PR is created, run:
```bash
cd /Users/jonathonfritz/code/work-projects/5dlabs/cto/monitor
cargo run -- alert-watch --namespace agent-platform --prompts-dir prompts --dry-run
```

Then trigger a Rex workflow and watch for the A7 alert.

