## Acceptance Criteria

- [ ] 1. **Namespace existence:** `kubectl get namespace hermes-staging hermes-production` returns both namespaces with correct labels (`hermes.io/environment=staging|production`) and annotations.
- [ ] 2. **RBAC verification:** `kubectl auth can-i --as=system:serviceaccount:hermes-staging:hermes-pipeline-sa --namespace=hermes-staging list pods` returns `yes`; same SA cannot access `hermes-production` (`kubectl auth can-i --as=system:serviceaccount:hermes-staging:hermes-pipeline-sa --namespace=hermes-production list pods` returns `no`).
- [ ] 3. **ResourceQuota enforcement:** `kubectl describe resourcequota -n hermes-staging` shows configured CPU/memory/pod limits; `kubectl describe resourcequota -n hermes-production` shows production-tier limits.
- [ ] 4. **LimitRange presence:** `kubectl get limitrange -n hermes-staging -o yaml` confirms default container limits are set.
- [ ] 5. **CiliumNetworkPolicy isolation:** Deploy a test pod in `hermes-staging` and attempt to curl a service in `hermes-production` — connection must be refused/timed out. Intra-namespace traffic must succeed. Egress to `gitlab-minio-svc.gitlab.svc:9000` must succeed.
- [ ] 6. **MinIO capacity documented:** A capacity report (total, used, free, estimated IOPS) is written to the Helm chart's output notes or a dedicated ConfigMap annotation. If capacity was insufficient, a dedicated MinIO instance is deployed and its endpoint is reflected in the ConfigMap.
- [ ] 7. **MinIO buckets exist and are functional:** `mc ls <alias>/hermes-staging-artifacts` and `mc ls <alias>/hermes-prod-artifacts` succeed. A test object can be PUT, GET (via presigned URL returning HTTP 200), and DELETE from each bucket.
- [ ] 8. **Bucket lifecycle policies active:** `mc ilm ls <alias>/hermes-staging-artifacts` shows expiry rule of 30 days; production shows 90 days.
- [ ] 9. **Bucket quotas set:** `mc admin bucket quota <alias>/hermes-staging-artifacts` confirms quota is configured.
- [ ] 10. **Backing services healthy:** Helm test pods in each namespace pass: Postgres accepts connections and responds to `SELECT 1`; Redis responds to `PING` with `PONG`; NATS accepts a connection and echoes a test publish/subscribe cycle.
- [ ] 11. **ConfigMap completeness:** `kubectl get configmap hermes-infra-endpoints -n hermes-staging -o json | jq '.data | keys'` returns all expected keys (`CNPG_HERMES_URL`, `REDIS_HERMES_URL`, `NATS_HERMES_URL`, `MINIO_ENDPOINT`, `MINIO_BUCKET`, `MINIO_PRESIGN_EXPIRY`, `ENVIRONMENT`). Same for production namespace.
- [ ] 12. **Secrets completeness:** `kubectl get secret hermes-infra-secrets -n hermes-staging -o json | jq '.data | keys'` contains `MINIO_ACCESS_KEY_ID`, `MINIO_SECRET_ACCESS_KEY`, `POSTGRES_URL`, `REDIS_URL`, `NATS_URL`. Values are non-empty (base64-decoded length > 0). Same for production.
- [ ] 13. **No secrets in ConfigMaps:** `kubectl get configmap hermes-infra-endpoints -n hermes-staging -o json | jq '.data'` contains no values matching password/key/secret patterns.
- [ ] 14. **PDB presence (production only):** `kubectl get pdb -n hermes-production` lists PDBs for Postgres, Redis, and NATS with `minAvailable: 1`.
- [ ] 15. **Helm idempotency:** Running `helm upgrade --install` twice in succession produces no errors and no resource drift.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.