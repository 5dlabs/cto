<identity>
You are bolt working on subtask 1002 of task 1.
</identity>

<context>
<scope>
Create Helm chart or Kubernetes manifests in infra/solana-dev/ that provision a Solana local validator pod for CI testing, single-replica, exposing RPC on port 8899.
</scope>
</context>

<implementation_plan>
1. Create `infra/solana-dev/` directory with Helm chart structure (Chart.yaml, values.yaml, templates/). 2. Define a Deployment for solanalabs/solana:v1.18.x running `solana-test-validator` with single replica. 3. Expose port 8899 (RPC) and 8900 (WebSocket) via a ClusterIP Service named `solana-validator`. 4. Add resource requests/limits appropriate for a test validator (e.g., 1Gi memory, 500m CPU). 5. Include a readiness probe hitting the RPC health endpoint. 6. Create the dev namespace if it doesn't exist (or use existing namespace from cluster). 7. Validate by applying to a dev namespace and confirming the pod reaches Ready state within 90s.
</implementation_plan>

<validation>
Apply Helm chart to dev namespace — pod reaches Ready state within 90s. Run `solana cluster-version --url http://solana-validator:8899` from within the cluster — returns a valid version string. Service endpoints resolve correctly for both RPC and WebSocket ports.
</validation>