## Decision Points

- Should a ClusterRole be created for cross-namespace read access to bridge services, or should all bridge communication remain strictly within the sigma-1 namespace?
- What is the organization-standard refreshInterval for secret rotation — 1h as suggested, or a different cadence?
- Which cluster logging backend should audit logs be shipped to (EFK, Loki, or another solution), and is one already available in the cluster?
- Should secrets be consumed via environment variables (requiring pod restart on rotation) or volume mounts (allowing live reload)? This impacts application-level rotation handling.

## Coordination Notes

- Agent owner: bolt
- Primary stack: Kubernetes/Helm