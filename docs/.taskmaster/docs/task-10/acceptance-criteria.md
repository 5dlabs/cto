# Acceptance Criteria: QA Kubernetes Verification and PR Approval

## QA Agent and Artifact Contract
- [ ] Agent produces /artifacts/qa/proof/summary.json with required schema
- [ ] Directory structure includes logs/, k8s/, and http/ subdirectories
- [ ] summary.json contains qa.passed boolean and artifact references
- [ ] Agent exits non-zero when tests fail (qa.passed=false)
- [ ] All referenced artifact files exist and are properly formatted

## Verification Gates and Compliance
- [ ] verify-k8s-proof validates summary.json existence and schema
- [ ] Gate checks qa.passed=true before allowing progression
- [ ] verify-compliance enforced as required dependency before approve-pr
- [ ] Pipeline fails when summary.json missing or invalid
- [ ] DAG dependencies prevent approve-pr without successful gates

## GitHub PR Approval Integration  
- [ ] approve-pr posts APPROVE review using GitHub App token
- [ ] Review API call uses correct headers and authentication
- [ ] Step fails gracefully on invalid tokens or missing artifacts
- [ ] No merge API endpoints called by any pipeline step
- [ ] GitHub App identity visible on approved reviews

## Evidence Collection System
- [ ] kubectl logs collected for all pods in target namespace
- [ ] kubectl get resources dumped to YAML files (pods, deployments, events, services)
- [ ] HTTP health checks captured with response codes and timing
- [ ] File paths in summary.json match collected artifacts
- [ ] Optional object store upload produces valid presigned URLs

## PR Comment and Communication
- [ ] Summary comment posted to PR with pass/fail status
- [ ] Comment includes links to artifact files or object store URLs
- [ ] Namespace and resource counts included in summary
- [ ] Comment updated or new comment posted per run
- [ ] Failed runs generate comments explaining missing evidence

## Branch Protection and Merge Prevention
- [ ] Repository branch protection settings verified intact
- [ ] No merge API endpoints present in codebase
- [ ] PRs remain open after QA approval
- [ ] Human review still required per branch protection rules
- [ ] QA approval does not trigger auto-merge behavior

## End-to-End Testing Scenarios
- [ ] Success path: valid evidence → gates pass → approval posted → PR not merged
- [ ] Failure path: missing/invalid evidence → gates fail → no approval → workflow fails
- [ ] Token validation: invalid GitHub token → approval step fails with clear error
- [ ] Object store integration: artifacts uploaded and URLs accessible
- [ ] Concurrent runs: multiple PRs processed without artifact collision