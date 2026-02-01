# Deploy PostgreSQL Cluster

<task>
<agent>postgres-deployer</agent>
<objective>Deploy a CloudNative-PG PostgreSQL cluster for the alerthub database</objective>

<workspace>
Your workspace is `/workspace`. Create all files there.
This is a local test - no actual Kubernetes cluster is available.
Create the YAML manifests that WOULD be applied in a real deployment.
</workspace>

<context>
The CloudNative-PG operator is already installed in the cluster. Your job is to create a PostgreSQL cluster instance using the operator's CRDs.
</context>

<requirements>
- Create a `Cluster` CR (CloudNative-PG custom resource)
- Database name: `alerthub`
- Configure persistent storage with appropriate PVC
- Set resource limits (CPU/memory)
- Enable monitoring annotations for Prometheus scraping
- Configure backup policy using the operator's backup CRDs
</requirements>

<deliverables>
- `/workspace/postgresql-cluster.yaml` - The Cluster CR manifest
- `/workspace/postgresql-backup.yaml` - Backup schedule configuration (if separate)
</deliverables>

<acceptance_criteria>
- [ ] postgresql-cluster.yaml exists with valid CloudNative-PG Cluster CR
- [ ] Database name is set to 'alerthub'
- [ ] Resource limits are configured
- [ ] Backup policy is defined
</acceptance_criteria>
</task>
