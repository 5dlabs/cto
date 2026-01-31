<task>
<agent>postgres-deployer</agent>
<title>Deploy PostgreSQL Cluster</title>
<objective>
Deploy a CloudNative-PG PostgreSQL cluster for the alerthub database
</objective>

<requirements>
- Create a `Cluster` CR (CloudNative-PG custom resource)
- Database name: `alerthub`
- Configure persistent storage with appropriate PVC
</requirements>

<deliverables>
- `postgresql-cluster.yaml` - The Cluster CR manifest
- `postgresql-backup.yaml` - Backup schedule configuration
- Applied to cluster and pods running
</deliverables>
</task>

## Instructions

Create a simple PostgreSQL cluster manifest file. This is a quick test - just create the yaml file and you're done.

Write the file to `postgresql-cluster.yaml` with a basic CloudNative-PG Cluster CR.
