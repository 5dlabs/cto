# Autonomous Implementation Prompt: QA Kubernetes Verification and PR Approval

## Mission Statement
Implement strict QA verification system with evidence collection in Kubernetes and automated PR approval workflow that blocks auto-merging.

## Technical Requirements
1. **QA Agent System** with artifact contract and evidence collection
2. **Verification Gates** validating proof files and compliance enforcement
3. **PR Approval Workflow** using GitHub Review API without merge capability
4. **Evidence Collection** with kubectl logs, resources, and HTTP checks
5. **Artifact Management** with optional object store upload and linking
6. **Branch Protection** ensuring human review requirements remain intact

## Key Implementation Points
- QA agent collects evidence to /artifacts/qa/proof directory structure
- summary.json schema with qa.passed boolean and artifact references
- verify-k8s-proof gate validates evidence before approval
- approve-pr step uses GitHub App token for APPROVE review only
- No merge API endpoints called - relies on branch protection
- Evidence includes kubectl logs, resource dumps, and HTTP responses
- Optional S3/GCS upload with presigned URLs in PR comments

## Success Criteria
- QA verification produces complete evidence artifacts
- Gates prevent approval without valid proof
- GitHub APPROVE reviews posted automatically on success
- Branch protection prevents auto-merge after approval
- Evidence artifacts accessible via links in PR comments
- End-to-end workflow supports both pass/fail scenarios