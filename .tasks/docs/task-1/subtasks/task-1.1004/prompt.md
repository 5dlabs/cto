<identity>
You are bolt working on subtask 1004 of task 1.
</identity>

<context>
<scope>
Define and apply the Opstree Redis/Valkey operator CR for sigma1-valkey in the databases namespace using valkey/valkey:7.2-alpine, single replica.
</scope>
</context>

<implementation_plan>
Create sigma1/infra/templates/valkey.yaml. Use the Opstree Redis operator CR (apiVersion: redis.redis.opstreelabs.in/v1beta2, kind: Redis or RedisCluster — use standalone Redis kind for single replica). metadata.name: sigma1-valkey, metadata.namespace: databases. spec.kubernetesConfig.image: valkey/valkey:7.2-alpine. spec.redisConfig or equivalent: no auth for dev (document production auth requirement). spec.storage: use default ephemeral or 5Gi PVC depending on operator defaults. Ensure the Opstree Redis operator is installed (document as prerequisite). Apply via Helm. Wait: kubectl wait redis/sigma1-valkey -n databases --for=condition=Ready --timeout=120s (or equivalent operator status condition).
</implementation_plan>

<validation>
kubectl get pods -n databases shows sigma1-valkey-0 in Running state. redis-cli -h sigma1-valkey.databases.svc.cluster.local -p 6379 ping returns PONG from within the cluster (run via kubectl exec into a temporary pod).
</validation>