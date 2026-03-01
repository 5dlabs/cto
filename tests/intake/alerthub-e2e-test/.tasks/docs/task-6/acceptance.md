# Acceptance Criteria: Task 6

- [ ] Deploy RabbitMQ operator for integration delivery task queues
- [ ] RabbitMQ is running, management UI accessible, can publish/consume messages, DLQs are configured
- [ ] All requirements implemented
- [ ] Tests passing (`helm lint charts/*` exits 0)
- [ ] Lints passing (`kubectl apply --dry-run=client -f . -R` exits 0)
- [ ] Formatted (`yamllint .` exits 0)
- [ ] Build succeeds (`helm template charts/*` exits 0)
- [ ] PR created and ready for review
