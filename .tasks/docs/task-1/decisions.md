## Decision Points

- PostgreSQL HA/DR Strategy: Decide on initial HA/DR for PostgreSQL (single instance for dev, future plans for HA/DR for production).
- Redis/Valkey Persistence: Decide on persistence strategy for Redis/Valkey (ephemeral for dev, RDB/AOF for production).
- S3/R2 Credential Management: Decide on the method for managing S3/R2 credentials (Kubernetes Secret, IAM roles, etc.).
- Initial Network Policy Scope: Define the initial scope of network policies (namespace-level, pod-level, ingress/egress rules).

## Coordination Notes

- Agent owner: bolt
- Primary stack: Kubernetes/Helm