## Decision Points

- Morgan is held at 1 replica due to Signal-CLI statefulness. The details do not specify whether Morgan should be managed as a Deployment (simpler) or converted to a StatefulSet with a PVC for Signal-CLI data persistence. This choice affects pod restart behavior and data durability.
- The details route only chat.sigma1.com and signals.sigma1.com through Cloudflare Tunnel. It is unclear whether the mobile app's REST API calls to /api/v1/* should also route through the Tunnel or through a separate ingress path. This affects network policy rules and TLS termination strategy.
- The details specify a CronJob that logs a reminder to Loki monthly. This is a manual-reminder approach. An alternative is integrating External Secrets Operator with a secrets manager (Vault, AWS Secrets Manager) for automated rotation. The v1 spec chooses manual reminders but this is an explicit architectural trade-off.

## Coordination Notes

- Agent owner: bolt
- Primary stack: Kubernetes/Helm/Cilium