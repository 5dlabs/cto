# Objective: intake

Submit intake for AlertHub and verify tasks.json is generated with testStrategy for each task.

## Gates
- intake-coderun-created: `kubectl get coderuns -n cto -o json | jq -e '[.items[] | select(.spec.runType == "intake")] | length > 0'`
- linear-sidecar-running: `kubectl get pods -n cto -l workflow-type=intake -o json | jq -e '.items[0].spec.containers | map(select(.name == "linear-sidecar" or .name == "linear-sync")) | length > 0'`
- intake-succeeded: `kubectl get coderuns -n cto -o json | jq -e '.items[] | select(.spec.runType == "intake") | .status.phase == "Succeeded"'`
- linear-issues-created: `curl -sf http://localhost:8081/api/linear/project/prd-alerthub-e2e-test/issues | jq -e '.issues | length > 0'`
- linear-issues-have-subtasks: `curl -sf http://localhost:8081/api/linear/project/prd-alerthub-e2e-test/issues | jq -e '.issues | map(select(.children | length > 0)) | length > 0'`
- linear-activities-posted: `curl -sf http://localhost:8081/api/linear/project/prd-alerthub-e2e-test/activities | jq -e '.activities | length > 0'`
- tasks-json-exists: `gh api repos/5dlabs/prd-prd-alerthub-e2e-test/contents/prd-prd-alerthub-e2e-test/.tasks/tasks/tasks.json --silent`
- tasks-have-test-strategy: `gh api repos/5dlabs/prd-prd-alerthub-e2e-test/contents/prd-prd-alerthub-e2e-test/.tasks/tasks/tasks.json --jq '.content' | base64 -d | jq -e '.tasks | all(.testStrategy != null and .testStrategy != "")'`
- task-docs-created: `gh api repos/5dlabs/prd-prd-alerthub-e2e-test/contents/prd-prd-alerthub-e2e-test/.tasks/docs --silent | jq -e 'length > 0'`
- cto-config-attached-to-issue: `curl -sf http://localhost:8081/api/linear/project/prd-alerthub-e2e-test/config | jq -e '.config != null and .config.agents != null'`
- cto-config-configmap-exists: `kubectl get configmap -n cto cto-config-prd-alerthub-e2e-test -o json | jq -e '.data["cto-config.json"] != null'`
- cto-config-sync-test: `BEFORE=$(kubectl get configmap -n cto cto-config-prd-alerthub-e2e-test -o json | jq -r '.metadata.resourceVersion'); curl -sf -X POST http://localhost:8081/api/linear/test-config-sync -d '{"project":"prd-alerthub-e2e-test","agent":"blaze","addTool":"context7"}' -H 'Content-Type: application/json'; sleep 10; AFTER=$(kubectl get configmap -n cto cto-config-prd-alerthub-e2e-test -o json | jq -r '.metadata.resourceVersion'); echo "Before: $BEFORE, After: $AFTER"; [ "$BEFORE" != "$AFTER" ]`
- auto-append-deploy-enabled: `cat ${ROOT_DIR}/cto-config.json | jq -e '.defaults.intake.autoAppendDeployTask == true' 2>/dev/null || echo 'Config check - may not be enabled'`
- deploy-task-appended: `gh api repos/5dlabs/prd-prd-alerthub-e2e-test/contents/prd-prd-alerthub-e2e-test/.tasks/tasks/tasks.json --jq '.content' | base64 -d | jq -e '.tasks[-1].agent_hint == "bolt" and (.tasks[-1].title | test("deploy|Deploy"; "i"))'`
- tool-filter-verified: `kubectl logs -n cto -l workflow-type=intake --tail=200 | grep -iE 'declared tools|Tool inventory|tools resolved|All declared tools'`

## Evidence
- Record command output in lifecycle-test/report.json
- Update lifecycle-test/progress.txt with the outcome
