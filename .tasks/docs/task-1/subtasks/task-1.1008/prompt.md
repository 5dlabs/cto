Implement subtask 1008: Deploy NATS with JetStream via Helm Subchart per Namespace

## Objective
Deploy a single-replica NATS server using nats/nats as a Helm subchart dependency in each namespace with JetStream enabled for durable message delivery.

## Steps
Step-by-step:
1. Add `nats/nats` as a dependency in `Chart.yaml` with condition `nats.enabled` (default true) and alias `nats`.
2. Configure in `values.yaml` defaults:
   - `nats.config.jetstream.enabled: true`
   - `nats.config.jetstream.memStorage.enabled: true`, `size: 256Mi`
   - `nats.config.jetstream.fileStorage.enabled: true`, `size: 1Gi`
   - `nats.config.cluster.enabled: false`
   - `nats.natsBox.enabled: true` (for testing)
   - `nats.nameOverride: hermes-nats`
3. Run `helm dependency update charts/hermes-infra` to fetch the subchart.
4. Connection string pattern: `nats://hermes-nats.{{ .Values.namespace }}.svc:4222` — document for downstream wiring.
5. Apply standard labels via subchart configuration.
6. Verify: `helm template --debug` renders NATS resources correctly.

## Validation
`kubectl get pods -n hermes-staging -l app.kubernetes.io/name=hermes-nats` shows 1 running pod. Using nats-box: `nats server info` shows JetStream enabled. A test publish/subscribe cycle succeeds: create a stream, publish a message, consume it. Same for production namespace.