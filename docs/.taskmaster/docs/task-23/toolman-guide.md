

# Task 23 Tool Usage Guide: Workflow Archival System

## Tool Categories and Usage

### Kubernetes Resource Management Tools
**Primary Purpose**: Deploy and manage archival system components and monitor workflow lifecycle

#### kubernetes_createResource & kubernetes_updateResource



```bash
# Deploy MinIO for artifact storage
kubernetes_createResource --file=./manifests/minio-deployment.yaml



# Create WorkflowArchive CRD
kubernetes_createResource --file=./manifests/workflow-archive-crd.yaml

# Deploy archival controller
kubernetes_createResource --file=./manifests/archive-controller.yaml

# Update Argo Workflows config for artifact repository
kubernetes_updateResource --file=./manifests/workflow-controller-configmap.yaml






```

**Best Practices**:


- Deploy archival components in separate namespace for isolation


- Use persistent volumes with appropriate storage classes for durability


- Configure resource limits to prevent archival operations from impacting active workflows


- Implement proper RBAC for archival system components

#### kubernetes_listResources & kubernetes_describeResource



```bash
# Monitor workflow archival status
kubernetes_listResources --api-version=argoproj.io/v1alpha1 --kind=WorkflowArchive --all-namespaces=true



# Check workflows eligible for archival
kubernetes_listResources --api-version=argoproj.io/v1alpha1 --kind=Workflow --field-selector=status.phase=Succeeded

# Examine archive controller status
kubernetes_describeResource --kind=Deployment --name=archive-controller --namespace=archival-system

# Review MinIO storage status
kubernetes_listPods --namespace=archival-system --selector=app=minio






```

**Monitoring Focus**:


- Track number of workflows in various archival states


- Monitor storage utilization and capacity planning


- Verify archival controller health and processing rates


- Check artifact repository connectivity and performance

#### kubernetes_deleteResource



```bash
# Clean up test archives during development
kubernetes_deleteResource --api-version=argoproj.io/v1alpha1 --kind=WorkflowArchive --name=test-archive-*

# Remove expired archives based on retention policies
kubernetes_deleteResource --api-version=argoproj.io/v1alpha1 --kind=WorkflowArchive --field-selector=spec.retentionDate<$(date -d "yesterday" -u +%Y-%m-%dT%H:%M:%SZ)

# Emergency cleanup of storage-intensive workflows
kubernetes_deleteResource --api-version=argoproj.io/v1alpha1 --kind=Workflow --label-selector=size=large,archive-eligible=true






```

**Safety Guidelines**:


- Always verify retention policies before deleting archives


- Use dry-run mode for bulk deletion operations


- Maintain audit logs of all deletion activities


- Coordinate with compliance team for regulated data deletion

#### kubernetes_getPodLogs



```bash
# Monitor archival controller processing
kubernetes_getPodLogs --pod-name=archive-controller-* --namespace=archival-system --follow=true

# Check MinIO storage operations
kubernetes_getPodLogs --pod-name=minio-* --namespace=archival-system --grep="bucket\|upload\|download"

# Analyze archival job execution
kubernetes_getPodLogs --job-name=workflow-cleanup-* --namespace=archival-system






```

**Log Analysis Focus**:


- Track archival processing rates and performance bottlenecks


- Identify storage errors or connectivity issues


- Monitor retention policy application and compliance


- Debug failed archival operations and recovery procedures

### Research and Documentation Tools



#### brave-search_brave_web_search



```bash


# Research archival best practices
brave-search_brave_web_search --query="Kubernetes workflow archival MinIO lifecycle management"

# Find retention policy implementations
brave-search_brave_web_search --query="compliance retention policies automation legal hold"

# Research storage optimization techniques
brave-search_brave_web_search --query="object storage compression deduplication lifecycle policies"






```

**Research Areas**:


- Industry best practices for workflow data archival and retention


- Compliance frameworks and regulatory requirements for data retention


- Storage optimization and cost management strategies


- Disaster recovery and business continuity for archived data

### Memory and Context Management

#### memory_create_entities & memory_query_entities



```bash
# Store archival configuration baselines
memory_create_entities --entities='[{"type":"archival-baseline","name":"storage-utilization-week1","properties":{"total_workflows":1200,"archived_workflows":800,"storage_savings":"65%"}}]'

# Track retention policy effectiveness
memory_create_entities --entities='[{"type":"retention-metrics","name":"policy-compliance-report","properties":{"compliance_rate":"99.8%","violations":2,"policy_overrides":15}}]'

# Query archival performance patterns
memory_query_entities --query="archival performance metrics and storage optimization results"

# Retrieve compliance and audit information
memory_query_entities --query="retention policy violations and resolution procedures"






```

**Knowledge Management**:


- Maintain repository of archival policies and configuration patterns


- Track storage optimization achievements and cost savings


- Store compliance reporting templates and audit procedures


- Build knowledge base of troubleshooting solutions and operational procedures

## Local Server Integration

### MinIO Client Manager
**Purpose**: Direct management of MinIO object storage for workflow artifacts




```python
# Example MinIO management operations
from minio import Minio

# Initialize MinIO client
client = MinioClient(
    endpoint=os.getenv('MINIO_ENDPOINT'),
    access_key=os.getenv('MINIO_ACCESS_KEY'),
    secret_key=os.getenv('MINIO_SECRET_KEY')
)



# Create bucket with lifecycle policies
client.create_bucket_with_lifecycle('workflow-artifacts', {
    'hot_storage': {'days': 30, 'storage_class': 'STANDARD'},
    'warm_storage': {'days': 365, 'storage_class': 'STANDARD_IA'},
    'cold_storage': {'days': 2555, 'storage_class': 'GLACIER'}  # 7 years
})



# Upload workflow artifacts with metadata
artifact_metadata = {
    'workflow_id': 'play-workflow-123',
    'task_id': '21',
    'agent_type': 'rex',
    'archival_date': '2024-01-15T10:30:00Z'
}

client.upload_workflow_artifacts(
    bucket='workflow-artifacts',
    workflow_id='play-workflow-123',
    artifacts=workflow_artifacts,
    metadata=artifact_metadata
)






```

**Storage Management Capabilities**:


- Bucket lifecycle policy management and optimization


- Artifact upload/download with progress tracking


- Storage utilization analysis and cost optimization


- Data integrity verification and corruption detection

### Archive Manager
**Purpose**: Comprehensive archive lifecycle management with policy enforcement




```python
# Archive management operations
archive_manager = ArchiveManager(
    kubeconfig_path='/path/to/kubeconfig',
    postgres_url=os.getenv('POSTGRES_URL')
)



# Execute archival process for eligible workflows
archival_results = archive_manager.archive_eligible_workflows(
    namespace='argo',
    batch_size=100,
    dry_run=False
)

# Apply retention policies with compliance checking
policy_results = archive_manager.apply_retention_policies(
    policy_config='/config/retention-policies.yaml',
    include_compliance_check=True
)

# Generate compliance reports
compliance_report = archive_manager.generate_compliance_report(
    report_period='monthly',
    include_audit_trail=True,
    export_format='json'
)






```

**Archive Management Features**:


- Automated workflow eligibility assessment and batch processing


- Retention policy evaluation and enforcement automation


- Compliance reporting with audit trail generation


- Archive restoration and validation capabilities

### Retention Policy Engine
**Purpose**: Intelligent policy management with compliance and override handling




```python
# Retention policy management
policy_engine = RetentionPolicyEngine(
    kubeconfig_path=os.getenv('KUBECONFIG'),
    policy_config_path=os.getenv('POLICY_CONFIG_PATH')
)

# Evaluate retention policy for specific workflow
policy_decision = policy_engine.evaluate_policy(
    workflow_metadata={
        'labels': {'priority': 'critical', 'environment': 'production'},
        'annotations': {'compliance.category': 'financial'},
        'phase': 'Succeeded',
        'finishedAt': '2024-01-01T12:00:00Z'
    }
)

# Apply policy overrides (legal hold, manual extension)
override_result = policy_engine.apply_override(
    workflow_id='play-workflow-123',
    override_type='legal_hold',
    reason='litigation pending case #2024-001',
    authorized_by='legal-team@company.com'
)

# Generate policy compliance summary
compliance_summary = policy_engine.generate_compliance_summary(
    time_range='last_quarter',
    include_violations=True,
    group_by=['policy_type', 'compliance_category']
)






```

**Policy Management Features**:


- Multi-tiered policy evaluation with precedence handling


- Legal hold and manual override capabilities


- Compliance monitoring with violation detection and alerting


- Policy effectiveness analysis and optimization recommendations

## Tool Combination Strategies

### Complete Archival Implementation



```bash
# 1. Deploy archival infrastructure
kubernetes_createResource --file=minio-deployment.yaml
kubernetes_createResource --file=archive-controller.yaml

# 2. Configure artifact repository
kubernetes_updateResource --file=workflow-controller-configmap.yaml

# 3. Initialize storage with lifecycle policies
minio_client --create-buckets --apply-lifecycle-policies

# 4. Deploy retention policy engine
retention_policy --load-policies --validate-configuration

# 5. Monitor archival system health
kubernetes_listResources --kind=WorkflowArchive
kubernetes_getPodLogs --pod-name=archive-controller-*






```

### Retention Policy Management



```bash
# 1. Define retention policies based on compliance requirements
retention_policy --create-policy --name=financial_compliance --retention=7y

# 2. Test policy application on sample workflows
retention_policy --test-policy --workflow-selector="labels.category=financial"

# 3. Deploy policies with monitoring
kubernetes_createResource --file=retention-policies-configmap.yaml

# 4. Monitor policy compliance and violations
archive_manager --compliance-report --period=monthly

# 5. Handle policy overrides and exceptions
retention_policy --apply-override --type=legal_hold --workflow-id=specific-workflow






```

### Storage Optimization Workflow



```bash
# 1. Analyze current storage utilization
minio_client --analyze-storage --bucket=workflow-artifacts

# 2. Implement compression and deduplication
archive_manager --optimize-storage --enable-compression --enable-deduplication

# 3. Configure lifecycle transitions
minio_client --configure-lifecycle --hot-to-warm=30d --warm-to-cold=365d

# 4. Monitor optimization effectiveness
memory_query_entities --query="storage optimization metrics over time"

# 5. Generate cost optimization recommendations
archive_manager --cost-analysis --recommendations=true






```

### Archive Query and Restoration



```bash


# 1. Query archived workflows by criteria
archive_manager --query-archives --namespace=argo --labels="task-id=21" --start-date="2024-01-01"

# 2. Export query results for analysis
archive_manager --export-results --format=csv --output=/reports/archive-analysis.csv

# 3. Restore specific workflows for debugging
archive_manager --restore-workflow --archive-id=workflow-archive-123 --target-namespace=debug

# 4. Validate restoration completeness
kubernetes_describeResource --kind=Workflow --name=restored-workflow-123

# 5. Track restoration operations for audit
memory_create_entities --type=restoration-audit --operation-details="[restoration_metadata]"






```

### Compliance and Audit Procedures



```bash
# 1. Generate comprehensive compliance reports
archive_manager --compliance-report --include-audit-trail --export-format=pdf

# 2. Verify data integrity across archives
minio_client --verify-integrity --bucket=workflow-artifacts --check-checksums



# 3. Test disaster recovery procedures
minio_client --test-backup-restore --target-region=dr-region

# 4. Document compliance status for regulators
retention_policy --generate-regulatory-report --framework=sox --period=annual

# 5. Maintain audit trail for all operations
memory_create_entities --type=audit-trail --all-archival-operations






```



## Best Practices Summary

### Storage Management


- Implement tiered storage with appropriate lifecycle policies


- Regular monitoring of storage utilization and cost optimization


- Backup and disaster recovery procedures for critical archived data


- Performance testing to ensure archival operations don't impact active workflows

### Retention Policy Management


- Clear policy documentation with precedence rules and override procedures


- Regular policy review and updates based on regulatory changes


- Automated compliance monitoring with exception handling


- Coordination with legal and compliance teams for policy enforcement

### Operational Excellence


- Comprehensive monitoring and alerting for all archival processes


- Regular testing of restoration procedures and data integrity validation


- Documentation maintenance for operational procedures and troubleshooting


- Capacity planning and cost management for long-term storage growth

### Security and Compliance


- Encryption at rest and in transit for all archived data


- Access control and audit logging for all archival operations


- Regular compliance audits and regulatory reporting


- Secure deletion verification for permanently removed data

This tool guide provides comprehensive guidance for implementing and managing the workflow archival system, ensuring efficient storage utilization, compliance with retention requirements, and reliable access to historical workflow data.
