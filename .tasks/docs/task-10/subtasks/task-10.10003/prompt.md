Implement subtask 10003: HA scaling: PostgreSQL and Valkey resource tuning and Valkey Sentinel evaluation

## Objective
Tune PostgreSQL resource requests/limits based on observed usage from Task 1 (already 2 instances). Evaluate Valkey operator for Sentinel mode support and either configure it or document the single-instance limitation.

## Steps
Step-by-step:
1. For PostgreSQL (CloudNativePG cluster CR):
   - Review current `resources.requests` and `resources.limits` in the cluster CR.
   - Check observed usage via `kubectl top pods` in `sigma1-db` namespace.
   - Set requests to ~70% of observed peak, limits to ~150% of observed peak.
   - Ensure `instances: 2` is set (verify from Task 1).
2. For Valkey:
   - Check the Valkey operator CRD documentation for Sentinel/HA support.
   - If supported: update the Valkey CR to enable Sentinel mode with 3 Sentinel processes and 1 replica.
   - If not supported: create a `VALKEY_HA_LIMITATION.md` document describing the single-instance limitation, blast radius, and recommended remediation (operator upgrade or Redis Cluster mode in Phase 2).
   - Set appropriate `resources.requests` and `resources.limits` on the Valkey pod (e.g., 256Mi request, 512Mi limit for memory).

## Validation
Verify PostgreSQL resource tuning by running `kubectl describe pod` on PG pods and confirming requests/limits are set. For Valkey, either verify Sentinel is running (`kubectl exec` into Valkey pod and run `valkey-cli info sentinel`) or verify the limitation document exists at the expected path.