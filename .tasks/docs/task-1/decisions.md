## Decision Points

- Should secrets be provisioned via external-secrets operator CRs (if available in the cluster) or sealed-secrets? This depends on which secret management operator is currently deployed and operational.

## Coordination Notes

- Agent owner: bolt
- Primary stack: Kubernetes/Helm