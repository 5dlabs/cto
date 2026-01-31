# Deploy MongoDB Cluster

<task>
<objective>Deploy a Percona MongoDB cluster for storing integration configurations</objective>

<context>
The Percona MongoDB operator is already installed in the cluster. Your job is to create a MongoDB cluster instance using the operator's CRDs.
</context>

<requirements>
- Create a `PerconaServerMongoDB` CR (Percona operator custom resource)
- Database purpose: integration configurations storage
- Configure persistent storage with appropriate PVC
- Set resource limits (CPU/memory)
- Enable monitoring for Prometheus scraping
- Configure replica set for high availability
</requirements>

<deliverables>
- `mongodb-cluster.yaml` - The PerconaServerMongoDB CR manifest
- Applied to cluster and pods running
</deliverables>

<acceptance_criteria>
- [ ] MongoDB cluster pods are Running
- [ ] Replica set is formed and healthy
- [ ] PVC is bound with persistent storage
- [ ] Database is accessible for connections
</acceptance_criteria>
</task>
