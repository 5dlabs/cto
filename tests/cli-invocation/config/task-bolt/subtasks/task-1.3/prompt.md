# Deploy Kafka Cluster

<task>
<agent>kafka-deployer</agent>
<objective>Deploy a Strimzi Kafka cluster for event streaming</objective>

<context>
The Strimzi Kafka operator is already installed in the cluster. Your job is to create a Kafka cluster instance using the operator's CRDs.
</context>

<requirements>
- Create a `Kafka` CR (Strimzi custom resource)
- Configure appropriate topic retention policies
- Set replication factors for high availability
- Configure persistent storage with appropriate PVC
- Set resource limits (CPU/memory)
- Enable monitoring for Prometheus scraping
</requirements>

<deliverables>
- `kafka-cluster.yaml` - The Kafka CR manifest
- Applied to cluster and pods running
</deliverables>

<acceptance_criteria>
- [ ] Kafka broker pods are Running
- [ ] Zookeeper pods are Running (if using ZK mode)
- [ ] PVC is bound with persistent storage
- [ ] Kafka is ready to accept connections
</acceptance_criteria>
</task>
