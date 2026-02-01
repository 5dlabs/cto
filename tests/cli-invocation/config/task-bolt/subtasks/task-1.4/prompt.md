# Deploy SeaweedFS Cluster

<task>
<agent>seaweedfs-deployer</agent>
<objective>Deploy a SeaweedFS cluster for distributed object storage</objective>

<context>
SeaweedFS operator or Helm chart is available. Your job is to deploy a SeaweedFS cluster for object storage to support file handling.
</context>

<requirements>
- Deploy SeaweedFS master server(s)
- Deploy SeaweedFS volume server(s)
- Configure data replication for durability
- Set up load balancing for volume servers
- Configure persistent storage for volumes
- Set resource limits (CPU/memory)
</requirements>

<deliverables>
- `seaweedfs-master.yaml` - Master server deployment
- `seaweedfs-volume.yaml` - Volume server deployment
- Applied to cluster and pods running
</deliverables>

<acceptance_criteria>
- [ ] SeaweedFS master pods are Running
- [ ] SeaweedFS volume pods are Running
- [ ] PVC is bound for volume storage
- [ ] Object storage API is accessible
</acceptance_criteria>
</task>
