<identity>
You are bolt working on subtask 1007 of task 1.
</identity>

<context>
<scope>
Define the sigma1-infra-endpoints ConfigMap in the sigma1 namespace with the five required connection string keys referencing in-cluster DNS addresses for PostgreSQL, Valkey, R2, and Signal-CLI.
</scope>
</context>

<implementation_plan>
Create sigma1/infra/templates/infra-endpoints-configmap.yaml. ConfigMap: name sigma1-infra-endpoints, namespace sigma1. Data keys: CNPG_SIGMA1_URL: 'postgresql://sigma1_user@sigma1-postgres-rw.databases.svc.cluster.local:5432/sigma1' (password must be injected at runtime from CNPG secret; document that services should use PGPASSWORD env from sigma1-postgres-app secret alongside this URL). VALKEY_SIGMA1_URL: 'redis://sigma1-valkey.databases.svc.cluster.local:6379'. R2_ENDPOINT: '{{ .Values.cloudflare.r2Endpoint }}' (templated from Helm values, populated from sigma1-cloudflare-secret at deploy time or hardcoded Cloudflare account endpoint). R2_BUCKET: '{{ .Values.cloudflare.r2BucketName }}'. SIGNAL_CLI_URL: 'http://signal-cli-svc.signal.svc.cluster.local:7583'. Add a comment noting that CNPG_SIGMA1_URL intentionally omits the password — services must mount sigma1-postgres-app secret for PGPASSWORD separately.
</implementation_plan>

<validation>
kubectl get configmap sigma1-infra-endpoints -n sigma1 -o yaml contains exactly 5 keys: CNPG_SIGMA1_URL, VALKEY_SIGMA1_URL, R2_ENDPOINT, R2_BUCKET, SIGNAL_CLI_URL. Each value is a non-empty string. CNPG_SIGMA1_URL value contains 'sigma1-postgres-rw.databases.svc.cluster.local'. VALKEY_SIGMA1_URL contains port 6379.
</validation>