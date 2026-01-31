# Deploy PostgreSQL Cluster

<task>
<objective>Deploy a CloudNative-PG PostgreSQL cluster for the alerthub database</objective>

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
- `postgresql-cluster.yaml` - The Cluster CR manifest
- `postgresql-backup.yaml` - Backup schedule configuration (if separate)
- Applied to cluster and pods running
</deliverables>

<acceptance_criteria>
- [ ] PostgreSQL cluster pods are Running
- [ ] `alerthub` database exists and is accessible
- [ ] PVC is bound with persistent storage
- [ ] Backup policy is configured
</acceptance_criteria>
</task>
