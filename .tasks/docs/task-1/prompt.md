<identity>
You are bolt, the Kubernetes/Helm implementation agent. You own task 1 end-to-end.
</identity>

<context>
<task_overview>
Task 1: Provision Core Infrastructure (Bolt - Kubernetes/Helm)
Bootstrap all foundational cluster resources for the Sigma-1 platform: namespace creation, CloudNative-PG PostgreSQL 16 cluster with six schemas, Opstree Valkey 7.2 instance, Cloudflare R2 bucket configuration, Signal-CLI pod with PVC, Kubernetes Secrets for all external APIs, and a sigma1-infra-endpoints ConfigMap aggregating all connection strings for downstream services.
Priority: high
Dependencies: None
</task_overview>
</context>

<implementation_plan>
1. Create namespaces: sigma1, databases, openclaw, signal.
2. Deploy CloudNative-PG Cluster CR named sigma1-postgres in databases namespace: PostgreSQL 16, 1 replica (dev), 50Gi storage, database sigma1, owner sigma1_user. Bootstrap initdb with extensions: uuid-ossp, pgcrypto, pg_trgm.
3. Post-cluster init Job: CREATE SCHEMA catalog; CREATE SCHEMA rms; CREATE SCHEMA finance; CREATE SCHEMA vetting; CREATE SCHEMA social; CREATE SCHEMA audit;. Create per-service roles: sigma1_catalog, sigma1_rms, sigma1_finance, sigma1_vetting, sigma1_social each with USAGE on their respective schema only.
4. Deploy Opstree Redis CR named sigma1-valkey in databases namespace using image valkey/valkey:7.2-alpine, single replica.
5. Create Kubernetes Secrets in sigma1 namespace: sigma1-stripe-secret (STRIPE_SECRET_KEY, STRIPE_WEBHOOK_SECRET), sigma1-opencorporates-secret (OPENCORPORATES_API_KEY), sigma1-linkedin-secret (LINKEDIN_CLIENT_ID, LINKEDIN_CLIENT_SECRET), sigma1-google-secret (GOOGLE_REVIEWS_API_KEY, GOOGLE_CALENDAR_CLIENT_ID, GOOGLE_CALENDAR_CLIENT_SECRET), sigma1-elevenlabs-secret (ELEVENLABS_API_KEY), sigma1-twilio-secret (TWILIO_ACCOUNT_SID, TWILIO_AUTH_TOKEN, TWILIO_PHONE_NUMBER), sigma1-openai-secret (OPENAI_API_KEY), sigma1-cloudflare-secret (CF_R2_ACCESS_KEY_ID, CF_R2_SECRET_ACCESS_KEY, CF_R2_ENDPOINT, CF_R2_BUCKET_NAME), sigma1-jwt-secret (JWT_SECRET).
6. Deploy Signal-CLI pod in signal namespace: image signalapp/signal-cli:latest, PVC signal-cli-data (5Gi RWO) mounted at /home/signal-cli/.local/share/signal-cli. Expose as ClusterIP Service signal-cli-svc:7583 (JSON-RPC port). Document manual registration step required before first use.
7. Create ConfigMap sigma1-infra-endpoints in sigma1 namespace with keys: CNPG_SIGMA1_URL=postgresql://sigma1_user:$(PGPASSWORD)@sigma1-postgres-rw.databases.svc.cluster.local:5432/sigma1, VALKEY_SIGMA1_URL=redis://sigma1-valkey.databases.svc.cluster.local:6379, R2_ENDPOINT=$(CF_R2_ENDPOINT), R2_BUCKET=$(CF_R2_BUCKET_NAME), SIGNAL_CLI_URL=http://signal-cli-svc.signal.svc.cluster.local:7583.
8. Apply ArgoCD Application CR pointing to the sigma1/infra Helm chart in the GitOps repo.
9. Validate all pods reach Running state; validate ConfigMap keys are populated; validate schema creation Job completes successfully.
</implementation_plan>

<acceptance_criteria>
1. kubectl get pods -n databases shows sigma1-postgres-1 and sigma1-valkey-0 in Running state within 5 minutes of apply.
2. psql -U sigma1_user -d sigma1 -c '\dn' returns catalog, rms, finance, vetting, social, audit schemas.
3. redis-cli -u $VALKEY_URL ping returns PONG.
4. kubectl get secret -n sigma1 shows all 9 secrets present.
5. kubectl get pod -n signal shows signal-cli pod Running.
6. kubectl get configmap sigma1-infra-endpoints -n sigma1 -o yaml contains all 5 required keys.
7. ArgoCD application shows Synced and Healthy.

See also: acceptance.md in this task directory for the checklist version.
</acceptance_criteria>

<subtasks>
- Create Kubernetes namespaces: sigma1, databases, openclaw, signal: Define and apply Namespace manifests for all four namespaces required by the Sigma-1 platform. These namespaces gate every subsequent resource deployment.
- Deploy CloudNative-PG PostgreSQL 16 Cluster CR (sigma1-postgres): Define and apply the CloudNative-PG Cluster custom resource for sigma1-postgres in the databases namespace with PostgreSQL 16, 1 replica (dev), 50Gi storage, and required extensions.
- Run post-cluster schema init Job: create six schemas and per-service roles: Deploy a Kubernetes Job that connects to sigma1-postgres and creates the six application schemas (catalog, rms, finance, vetting, social, audit) plus per-service PostgreSQL roles with scoped USAGE grants.
- Deploy Opstree Valkey 7.2 single-replica instance (sigma1-valkey): Define and apply the Opstree Redis/Valkey operator CR for sigma1-valkey in the databases namespace using valkey/valkey:7.2-alpine, single replica.
- Create all nine Kubernetes Secrets in the sigma1 namespace: Define Helm secret templates (or ExternalSecrets if using ESO) for all nine external API secrets required by the platform in the sigma1 namespace.
- Deploy Signal-CLI pod, PVC, and ClusterIP Service in the signal namespace: Create a PersistentVolumeClaim, Pod (or Deployment), and ClusterIP Service for Signal-CLI in the signal namespace with a 5Gi RWO volume mounted at the Signal data path.
- Create sigma1-infra-endpoints ConfigMap aggregating all connection strings: Define the sigma1-infra-endpoints ConfigMap in the sigma1 namespace with the five required connection string keys referencing in-cluster DNS addresses for PostgreSQL, Valkey, R2, and Signal-CLI.
- Apply ArgoCD Application CR for the sigma1/infra Helm chart and validate full stack: Create the ArgoCD Application custom resource pointing to the sigma1/infra Helm chart in the GitOps repo, sync it, and run the full validation suite confirming all pods, secrets, schemas, and the ConfigMap are healthy.
</subtasks>