# Acceptance Criteria: Task 2

- [ ] Deploy CloudNative-PG operator and PostgreSQL cluster for structured data storage
- [ ] PostgreSQL cluster is running, accepts connections, alerthub database exists, connection string works from test pod
- [ ] All requirements implemented
- [ ] Tests passing (`helm lint charts/*` exits 0)
- [ ] Lints passing (`kubectl apply --dry-run=client -f . -R` exits 0)
- [ ] Formatted (`yamllint .` exits 0)
- [ ] Build succeeds (`helm template charts/*` exits 0)
- [ ] PR created and ready for review
