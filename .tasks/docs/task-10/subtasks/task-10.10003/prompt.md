<identity>
You are bolt working on subtask 10003 of task 10.
</identity>

<context>
<scope>
Update the Valkey Deployment or StatefulSet manifest to replicas: 2. If Valkey supports primary/replica mode, configure replication. If standalone, run 2 independent instances behind a service. Apply and verify both pods are Running.
</scope>
</context>

<implementation_plan>
Locate the Valkey manifest in helm/sigma1/templates/valkey.yaml. Set `spec.replicas: 2`. If the Valkey chart supports replication via `--replicaof` or equivalent, add the replica configuration pointing to the primary pod's ClusterIP service. Update the sigma1-infra-endpoints ConfigMap VALKEY_URL to point to the primary service. Run `helm upgrade sigma1 ./helm/sigma1 -n sigma1`. Verify `kubectl get pods -n sigma1 -l app=valkey` shows 2 Running pods.
</implementation_plan>

<validation>
`kubectl get pods -n sigma1 -l app=valkey` shows 2 pods in Running state. `kubectl exec` into valkey-0 and run `REDIS-CLI INFO replication` — confirms role:master with 1 connected slave (or equivalent Valkey output).
</validation>