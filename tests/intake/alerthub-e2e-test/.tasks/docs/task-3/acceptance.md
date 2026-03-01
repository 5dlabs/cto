# Acceptance Criteria: Task 3

- [ ] Deploy Redis operator and Valkey instance for caching and rate limiting
- [ ] Valkey instance is running, accepts redis-cli connections, can set/get keys, persistence is working
- [ ] All requirements implemented
- [ ] Tests passing (`helm lint charts/*` exits 0)
- [ ] Lints passing (`kubectl apply --dry-run=client -f . -R` exits 0)
- [ ] Formatted (`yamllint .` exits 0)
- [ ] Build succeeds (`helm template charts/*` exits 0)
- [ ] PR created and ready for review
