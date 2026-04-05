## Acceptance Criteria

- [ ] 1. JWT validation test: each service accepts requests with valid service token (correct signature, not expired) and rejects requests with expired/invalid tokens returning 401. 2. Audit log test: perform a customer data read on Equipment Catalog, verify audit_log entry created with correct service_name, action, entity_type, actor_service. 3. GDPR export test: trigger data export for test customer, verify JSON file in R2 contains data from all services, signed URL works, data_export_requests record created. 4. GDPR deletion test: trigger deletion for test customer, verify vetting data removed (404 on GET), finance records anonymized (PII fields nulled but record exists), data_deletion_requests shows all services purged. 5. Pod security test: attempt to deploy a pod with privileged: true in sigma1 namespace, verify rejected by admission controller. 6. Secret rotation test: trigger JWT key rotation CronJob, verify new tokens issued, old tokens rejected after grace period, services restart cleanly with new tokens. 7. Alert test: scale equipment-catalog to 0 replicas, verify AlertManager fires critical alert within 1 minute. 8. Rate limit test: send 101 requests from single IP to api.sigma-1.com within 1 minute, verify 429 response on request 101.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.