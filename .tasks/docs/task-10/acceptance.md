## Acceptance Criteria

- [ ] 1. kubectl get pods -n sigma1 shows all service Deployments with READY=2/2 (or 1/1 for Morgan). 2. kubectl get hpa -n sigma1 shows HPA objects for equipment-catalog, rms, finance, customer-vetting, social-engine with MINPODS=2. 3. CiliumNetworkPolicy enforcement: attempt direct HTTP from finance pod to social-engine pod (kubectl exec curl); connection must be refused (exit code 1 or connection timeout). 4. k6 run --vus 100 --duration 60s availability_check.js: p99 latency < 500ms, error rate < 0.1%. 5. ArgoCD UI shows all Applications in Synced + Healthy state. 6. GitHub Actions: open a test PR, verify Stitch posts review comment, Tess reports coverage >= 80%, Cipher reports no CRITICAL findings, Atlas blocks merge until all checks pass. 7. Grafana dashboard loads and shows real-time metrics for all 5 backend services with non-zero request_rate. 8. GDPR end-to-end: after sigma1_gdpr_delete execution, audit schema contains row with customer_id, deleted_at timestamp, and services_affected array listing all 5 services. 9. CNPG cluster: kubectl get cluster sigma1-postgres -n databases shows instances: 2 and status Ready. 10. Cloudflare Tunnel: curl https://chat.sigma1.com/health returns 200 from Morgan chat service.

## Verification Notes

- [ ] Confirm dependencies are satisfied before implementation.
- [ ] Update tests, docs, and configuration touched by this task.
- [ ] Validate the final behavior against the task objective.