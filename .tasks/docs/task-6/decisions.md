## Decision Points

- Canary phase detection mechanism: The task specifies 'Production with <10% traffic → canary' but does not define how the service determines its traffic percentage. Options include: reading a label/annotation from the Kubernetes deployment, an explicit environment variable, or querying an ingress/service mesh. This needs to be decided before implementing rollout phase tracking.
- Redis dependency for sliding window counters: The rollback trigger conditions require Redis-backed persistence for sliding window counters. Should this use the existing shared Redis instance (if one exists), a dedicated Redis instance, or should in-memory counters suffice for v1 with the understanding that counters reset on restart?

## Coordination Notes

- Agent owner: nova
- Primary stack: Bun/Elysia