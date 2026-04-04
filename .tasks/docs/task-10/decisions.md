## Decision Points

- Container registry choice: ghcr.io vs Cloudflare Container Registry — impacts CI/CD image push configuration and ArgoCD image pull secrets.
- Valkey HA strategy: Sentinel mode (if operator supports) vs documented single-instance limitation — affects availability guarantees and manifest complexity.
- Internal mTLS: implement now with cert-manager or defer to Phase 2 — impacts security posture and operational complexity.
- GDPR orchestrator trigger model: Morgan-triggered Kubernetes Job vs CronJob polling a queue table — affects architecture and reliability guarantees.
- Secret rotation strategy: manual documented process vs external-secrets-operator automation — depends on whether ESO is available in the cluster.
- Alerting destination: PagerDuty vs Signal notification vs both — affects AlertManager receiver configuration.

## Coordination Notes

- Agent owner: bolt
- Primary stack: Kubernetes/Helm