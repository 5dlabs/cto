# Acceptance Criteria: Task 1

- [ ] Set up Kubernetes cluster with basic operators and namespaces for AlertHub platform
- [ ] Cluster is accessible via kubectl, all namespaces exist, cert-manager webhook responds, network policies allow expected traffic
- [ ] All requirements implemented
- [ ] Tests passing (`helm lint charts/*` exits 0)
- [ ] Lints passing (`kubectl apply --dry-run=client -f . -R` exits 0)
- [ ] Formatted (`yamllint .` exits 0)
- [ ] Build succeeds (`helm template charts/*` exits 0)
- [ ] PR created and ready for review
