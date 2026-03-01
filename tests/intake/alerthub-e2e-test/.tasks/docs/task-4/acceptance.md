# Acceptance Criteria: Task 4

- [ ] Deploy Strimzi operator and Kafka cluster for async event processing
- [ ] Kafka cluster is running, topics exist with correct partitions, can produce/consume messages via kafka-console tools
- [ ] All requirements implemented
- [ ] Tests passing (`helm lint charts/*` exits 0)
- [ ] Lints passing (`kubectl apply --dry-run=client -f . -R` exits 0)
- [ ] Formatted (`yamllint .` exits 0)
- [ ] Build succeeds (`helm template charts/*` exits 0)
- [ ] PR created and ready for review
