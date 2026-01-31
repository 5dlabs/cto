# MongoDB Production Readiness Checklist

Complete this checklist before deploying MongoDB to production.

## Pre-Deployment

### Infrastructure Requirements

- [ ] Kubernetes cluster version 1.24+ verified
- [ ] Percona MongoDB Operator v1.16.0 installed
- [ ] Sufficient cluster resources available
  - [ ] CPU: 3+ cores per replica (9+ total)
  - [ ] Memory: 4GB+ per replica (12GB+ total)
  - [ ] Storage: 60GB+ persistent storage (20GB × 3 replicas)
- [ ] Storage class supports dynamic provisioning
- [ ] Storage class supports volume expansion
- [ ] Multiple availability zones configured
- [ ] Network policies supported (if required)

### Security Configuration

- [ ] **All default passwords changed** in secrets.yaml
- [ ] Passwords meet complexity requirements (16+ chars, mixed case, numbers, symbols)
- [ ] Secrets stored in external secret manager (Vault, AWS Secrets Manager, etc.)
- [ ] S3 backup credentials configured with least privilege
- [ ] Database users follow principle of least privilege
- [ ] Network policies configured and tested
- [ ] TLS/SSL certificates generated (if required)
- [ ] Certificate rotation procedure documented
- [ ] Audit logging enabled (if required)
- [ ] RBAC policies configured for operator

### Backup Configuration

- [ ] S3 bucket created and accessible
- [ ] S3 credentials tested and verified
- [ ] Backup schedules configured (daily + weekly)
- [ ] PITR (Point-in-Time Recovery) enabled
- [ ] Backup retention policies set
- [ ] Backup encryption configured (if required)
- [ ] Backup monitoring alerts configured
- [ ] Restore procedure tested successfully
- [ ] Automated backup verification enabled
- [ ] Off-site backup replication configured (optional)

### Monitoring & Alerting

- [ ] PMM Server deployed and accessible
- [ ] PMM agent configured in cluster
- [ ] Prometheus/Grafana integration (if using)
- [ ] ServiceMonitor/PodMonitor created (if using Prometheus Operator)
- [ ] Critical alerts configured:
  - [ ] Pod not ready
  - [ ] High CPU usage (>80%)
  - [ ] High memory usage (>85%)
  - [ ] High disk usage (>85%)
  - [ ] Replication lag (>10s)
  - [ ] Backup failures
  - [ ] Connection pool exhaustion
- [ ] Alert routing configured (email, Slack, PagerDuty, etc.)
- [ ] Dashboards created and shared with team
- [ ] Log aggregation configured (ELK, Loki, etc.)

### High Availability

- [ ] 3+ replica set members configured
- [ ] Anti-affinity rules configured
- [ ] PodDisruptionBudget set (maxUnavailable: 1)
- [ ] Multi-AZ deployment verified
- [ ] Automatic failover tested
- [ ] Split-brain prevention verified
- [ ] Network partition handling tested

### Performance Tuning

- [ ] Resource requests/limits tuned for workload
- [ ] WiredTiger cache size configured (50-60% of pod memory)
- [ ] Storage I/O tested (IOPS, throughput)
- [ ] Connection pool sizing configured
- [ ] Query performance baseline established
- [ ] Index strategy documented
- [ ] Slow query logging configured
- [ ] Operation profiling enabled

## Deployment

### Deployment Process

- [ ] Maintenance window scheduled
- [ ] Stakeholders notified
- [ ] Rollback plan prepared
- [ ] Namespace created
- [ ] Secrets applied
- [ ] ConfigMap applied
- [ ] Cluster manifest applied
- [ ] Services created

### Verification

- [ ] All pods running and ready
- [ ] Replica set initialized
- [ ] Primary elected
- [ ] All replicas syncing
- [ ] Services accessible
- [ ] PMM monitoring active
- [ ] Backups scheduled
- [ ] Logs flowing to aggregation system

### Initial Configuration

- [ ] Admin users created
- [ ] Application users created with appropriate roles
- [ ] Application databases created
- [ ] Initial indexes created
- [ ] Connection strings documented and shared
- [ ] Sample data loaded (if applicable)
- [ ] Application connectivity tested

## Post-Deployment

### Testing

- [ ] Connection from application tested
- [ ] Read operations tested
- [ ] Write operations tested
- [ ] Failover tested (delete primary pod)
- [ ] Backup triggered manually
- [ ] Restore tested to temporary cluster
- [ ] Performance baseline measured
- [ ] Load testing completed
- [ ] Concurrent connection testing
- [ ] Network policy enforcement verified (if applicable)

### Documentation

- [ ] Connection information documented
- [ ] Access procedures documented
- [ ] Backup/restore procedures documented
- [ ] Troubleshooting guide created
- [ ] Disaster recovery plan documented
- [ ] Runbooks created for common operations:
  - [ ] Scale replicas
  - [ ] Increase storage
  - [ ] Upgrade version
  - [ ] Rotate credentials
  - [ ] Emergency procedures
- [ ] Architecture diagram created
- [ ] Change management process defined

### Monitoring Validation

- [ ] All metrics visible in PMM/Grafana
- [ ] Alert rules firing correctly (test alerts)
- [ ] On-call team can access monitoring
- [ ] Log queries documented
- [ ] Dashboard access shared with team
- [ ] Health check endpoints verified

### Team Readiness

- [ ] Team trained on operations
- [ ] On-call rotation established
- [ ] Escalation procedures defined
- [ ] Access permissions granted
- [ ] Emergency contact list updated
- [ ] Knowledge base articles created

## Ongoing Operations

### Daily

- [ ] Check cluster health
- [ ] Review alerts
- [ ] Monitor disk usage
- [ ] Check backup status
- [ ] Review slow query log

### Weekly

- [ ] Review performance metrics
- [ ] Analyze growth trends
- [ ] Test backup restore
- [ ] Review security logs (if enabled)
- [ ] Update documentation

### Monthly

- [ ] Review and update alerts
- [ ] Capacity planning
- [ ] Performance tuning
- [ ] Security audit
- [ ] Disaster recovery drill
- [ ] Update runbooks

### Quarterly

- [ ] Full disaster recovery test
- [ ] Review and update architecture
- [ ] Evaluate new features/versions
- [ ] Team training refresh
- [ ] Cost optimization review

## Compliance & Governance

- [ ] Data residency requirements met
- [ ] Compliance requirements documented (GDPR, HIPAA, etc.)
- [ ] Data retention policies implemented
- [ ] Encryption at rest configured (if required)
- [ ] Encryption in transit configured (if required)
- [ ] Audit logging meets requirements
- [ ] Access controls reviewed and approved
- [ ] Data classification applied
- [ ] Privacy impact assessment completed (if required)

## Cost Optimization

- [ ] Resource utilization reviewed
- [ ] Storage optimization implemented
- [ ] Backup retention optimized
- [ ] Reserved capacity considered (cloud)
- [ ] Cost alerts configured
- [ ] Budget approved

## Emergency Preparedness

- [ ] Emergency procedures documented
- [ ] Emergency contacts updated
- [ ] 24/7 support coverage confirmed
- [ ] Incident response plan created
- [ ] Communication plan established
- [ ] Post-incident review process defined

## Sign-off

### Technical Review

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Database Administrator | | | |
| Platform Engineer | | | |
| Security Team | | | |
| Application Team | | | |

### Management Approval

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Technical Lead | | | |
| Product Owner | | | |
| Operations Manager | | | |

---

## Notes

- This checklist should be completed before production deployment
- Any unchecked items must be documented with justification
- Regular reviews ensure ongoing production readiness
- Update this checklist based on lessons learned

**Version**: 1.0
**Last Updated**: 2024-01-15
**Next Review**: 2024-04-15
