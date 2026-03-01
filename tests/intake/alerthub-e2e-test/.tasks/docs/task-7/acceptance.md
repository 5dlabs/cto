# Acceptance Criteria: Task 7

- [ ] Deploy SeaweedFS for S3-compatible object storage of attachments and exports
- [ ] SeaweedFS is running, S3 API responds, can create/delete objects in test bucket using AWS CLI
- [ ] All requirements implemented
- [ ] Tests passing (`helm lint charts/*` exits 0)
- [ ] Lints passing (`kubectl apply --dry-run=client -f . -R` exits 0)
- [ ] Formatted (`yamllint .` exits 0)
- [ ] Build succeeds (`helm template charts/*` exits 0)
- [ ] PR created and ready for review
