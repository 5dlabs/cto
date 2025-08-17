# Task 23: Setup Workflow Archival

## Overview

This task implements comprehensive workflow archival and cleanup policies for the multi-agent orchestration system. The archival system manages the complete lifecycle of workflows from active execution through long-term storage and eventual cleanup, ensuring efficient resource utilization while maintaining compliance and audit requirements.

## Technical Implementation

### 1. Argo Workflows Artifact Repository Configuration

#### MinIO/S3 Artifact Repository Setup
```yaml
# MinIO deployment for workflow artifact storage
apiVersion: apps/v1
kind: Deployment
metadata:
  name: minio-artifact-repository
  namespace: argo
spec:
  replicas: 1
  selector:
    matchLabels:
      app: minio-artifact-repository
  template:
    metadata:
      labels:
        app: minio-artifact-repository
    spec:
      containers:
      - name: minio
        image: minio/minio:latest
        command:
        - /bin/bash
        - -c
        args:
        - minio server /data --console-address :9001
        env:
        - name: MINIO_ACCESS_KEY
          valueFrom:
            secretKeyRef:
              name: minio-credentials
              key: access-key
        - name: MINIO_SECRET_KEY
          valueFrom:
            secretKeyRef:
              name: minio-credentials  
              key: secret-key
        ports:
        - containerPort: 9000
        - containerPort: 9001
        volumeMounts:
        - name: data
          mountPath: /data
        resources:
          requests:
            cpu: 500m
            memory: 1Gi
          limits:
            cpu: 1000m
            memory: 2Gi
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: minio-storage
```

#### Argo Workflows Artifact Repository Configuration
```yaml
# Workflow artifact repository configuration
apiVersion: v1
kind: ConfigMap
metadata:
  name: workflow-controller-configmap
  namespace: argo
data:
  config: |
    artifactRepository:
      s3:
        endpoint: minio-artifact-repository:9000
        bucket: workflow-artifacts
        insecure: true
        accessKeySecret:
          name: minio-credentials
          key: access-key
        secretKeySecret:
          name: minio-credentials
          key: secret-key
        
    # Workflow archival configuration  
    archiveTTL: 2592000  # 30 days in seconds
    
    # Garbage collection policy
    retentionPolicy:
      completed: 2592000  # 30 days for completed workflows
      failed: 7776000     # 90 days for failed workflows (longer for debugging)
      error: 7776000      # 90 days for errored workflows
      
    # Archive compression and metadata
    persistence:
      archive: true
      compress: true
      metadata:
        - name
        - namespace  
        - labels
        - annotations
        - creationTimestamp
        - startedAt
        - finishedAt
        - phase
        - duration
```

### 2. Workflow Lifecycle Management

#### Workflow Archive CRD
```yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition  
metadata:
  name: workflowarchives.argoproj.io
spec:
  group: argoproj.io
  versions:
  - name: v1alpha1
    served: true
    storage: true
    schema:
      openAPIV3Schema:
        type: object
        properties:
          spec:
            type: object
            properties:
              workflow:
                type: object
                description: "Original workflow specification"
              archivalDate:
                type: string
                format: date-time
                description: "When workflow was archived"
              retentionDate:
                type: string
                format: date-time
                description: "When workflow should be deleted"
              reason:
                type: string
                description: "Reason for archival (TTL, manual, policy)"
  scope: Namespaced
  names:
    plural: workflowarchives
    singular: workflowarchive
    kind: WorkflowArchive
```

#### Archive Controller Implementation
```go
// Archive controller for workflow lifecycle management
type ArchiveController struct {
    client        client.Client
    argoClient    argoclientset.Interface
    artifactRepo  ArtifactRepository
    retentionConfig RetentionConfig
}

type RetentionConfig struct {
    CompletedTTL time.Duration `json:"completedTTL"`
    FailedTTL    time.Duration `json:"failedTTL"`
    ErrorTTL     time.Duration `json:"errorTTL"`
    ArchiveBucket string       `json:"archiveBucket"`
}

func (ac *ArchiveController) archiveWorkflow(ctx context.Context, wf *wfv1.Workflow) error {
    // 1. Create archive entry
    archive := &v1alpha1.WorkflowArchive{
        ObjectMeta: metav1.ObjectMeta{
            Name:      fmt.Sprintf("%s-%d", wf.Name, time.Now().Unix()),
            Namespace: wf.Namespace,
            Labels:    wf.Labels,
        },
        Spec: v1alpha1.WorkflowArchiveSpec{
            Workflow:      *wf,
            ArchivalDate:  metav1.Now(),
            RetentionDate: metav1.NewTime(time.Now().Add(ac.getRetentionPeriod(wf))),
            Reason:        "TTL",
        },
    }
    
    // 2. Store workflow artifacts in object storage
    if err := ac.storeWorkflowArtifacts(ctx, wf); err != nil {
        return fmt.Errorf("failed to store artifacts: %w", err)
    }
    
    // 3. Create archive record
    if err := ac.client.Create(ctx, archive); err != nil {
        return fmt.Errorf("failed to create archive: %w", err)
    }
    
    // 4. Delete original workflow
    if err := ac.argoClient.ArgoprojV1alpha1().
        Workflows(wf.Namespace).
        Delete(ctx, wf.Name, metav1.DeleteOptions{}); err != nil {
        return fmt.Errorf("failed to delete workflow: %w", err)
    }
    
    return nil
}

func (ac *ArchiveController) getRetentionPeriod(wf *wfv1.Workflow) time.Duration {
    switch wf.Status.Phase {
    case wfv1.WorkflowSucceeded:
        return ac.retentionConfig.CompletedTTL
    case wfv1.WorkflowFailed:
        return ac.retentionConfig.FailedTTL
    case wfv1.WorkflowError:
        return ac.retentionConfig.ErrorTTL
    default:
        return ac.retentionConfig.CompletedTTL
    }
}
```

### 3. Compliance-Based Retention Policies

#### Retention Policy Engine
```yaml
# Retention policy configuration
apiVersion: v1
kind: ConfigMap
metadata:
  name: workflow-retention-policies
  namespace: argo
data:
  policies.yaml: |
    default_retention:
      completed_workflows: "30d"
      failed_workflows: "90d"
      error_workflows: "90d"
      
    compliance_requirements:
      audit_retention: "7y"     # 7 years for audit compliance
      security_events: "5y"     # 5 years for security events
      financial_data: "7y"      # 7 years for financial compliance
      
    policy_overrides:
      critical_workflows:
        selector:
          labels:
            priority: "critical"
        retention: "1y"
        
      production_deployments:
        selector:
          labels:
            environment: "production"
        retention: "6m"
        
      security_workflows:
        selector:
          labels:
            category: "security"
        retention: "5y"
        archive_immediately: true
```

#### Policy Enforcement Controller
```python
# Python-based retention policy enforcement
import kubernetes
from datetime import datetime, timedelta
import yaml

class RetentionPolicyController:
    def __init__(self, kubeconfig_path=None):
        self.k8s_client = kubernetes.config.load_kube_config(kubeconfig_path)
        self.custom_api = kubernetes.client.CustomObjectsApi()
        self.policies = self.load_retention_policies()
    
    def load_retention_policies(self):
        """Load retention policies from ConfigMap"""
        v1 = kubernetes.client.CoreV1Api()
        cm = v1.read_namespaced_config_map(
            name="workflow-retention-policies",
            namespace="argo"
        )
        return yaml.safe_load(cm.data["policies.yaml"])
    
    def evaluate_retention_policy(self, workflow):
        """Determine retention period for a workflow"""
        base_retention = self.get_base_retention(workflow["status"]["phase"])
        
        # Check for policy overrides
        for policy_name, policy in self.policies.get("policy_overrides", {}).items():
            if self.matches_selector(workflow, policy["selector"]):
                return self.parse_duration(policy["retention"])
        
        return self.parse_duration(base_retention)
    
    def archive_eligible_workflows(self):
        """Archive workflows that have exceeded their retention period"""
        workflows = self.custom_api.list_namespaced_custom_object(
            group="argoproj.io",
            version="v1alpha1", 
            namespace="argo",
            plural="workflows"
        )
        
        for workflow in workflows["items"]:
            if self.should_archive(workflow):
                self.archive_workflow(workflow)
    
    def should_archive(self, workflow):
        """Check if workflow should be archived"""
        if workflow["status"]["phase"] in ["Running", "Pending"]:
            return False
            
        finished_at = datetime.fromisoformat(
            workflow["status"]["finishedAt"].replace("Z", "+00:00")
        )
        
        retention_period = self.evaluate_retention_policy(workflow)
        archive_date = finished_at + retention_period
        
        return datetime.now() >= archive_date
```

### 4. Workflow History API

#### Archive Query Service
```go
// RESTful API for querying archived workflows
type ArchiveQueryService struct {
    client     client.Client
    repository ArtifactRepository
}

type WorkflowQuery struct {
    Namespace    string    `json:"namespace,omitempty"`
    Name         string    `json:"name,omitempty"`
    Labels       map[string]string `json:"labels,omitempty"`
    Phase        string    `json:"phase,omitempty"`
    StartTime    time.Time `json:"startTime,omitempty"`
    EndTime      time.Time `json:"endTime,omitempty"`
    Limit        int       `json:"limit,omitempty"`
    Offset       int       `json:"offset,omitempty"`
}

func (aqs *ArchiveQueryService) QueryArchives(ctx context.Context, query WorkflowQuery) (*ArchiveQueryResult, error) {
    // Build selector from query parameters
    listOpts := []client.ListOption{
        client.InNamespace(query.Namespace),
    }
    
    if len(query.Labels) > 0 {
        selector := labels.Set(query.Labels).AsSelector()
        listOpts = append(listOpts, client.MatchingLabelsSelector{Selector: selector})
    }
    
    var archives v1alpha1.WorkflowArchiveList
    if err := aqs.client.List(ctx, &archives, listOpts...); err != nil {
        return nil, fmt.Errorf("failed to query archives: %w", err)
    }
    
    // Filter and paginate results
    filtered := aqs.filterArchives(archives.Items, query)
    paginated := aqs.paginate(filtered, query.Limit, query.Offset)
    
    result := &ArchiveQueryResult{
        Archives:   paginated,
        TotalCount: len(filtered),
        Query:      query,
    }
    
    return result, nil
}

func (aqs *ArchiveQueryService) RestoreWorkflow(ctx context.Context, archiveID string) (*wfv1.Workflow, error) {
    // 1. Retrieve archive record
    archive, err := aqs.getArchiveByID(ctx, archiveID)
    if err != nil {
        return nil, fmt.Errorf("archive not found: %w", err)
    }
    
    // 2. Restore workflow from archive
    workflow := &archive.Spec.Workflow
    workflow.ResourceVersion = ""  // Clear for recreation
    workflow.UID = ""             // Clear for recreation
    
    // 3. Restore artifacts from object storage
    if err := aqs.restoreWorkflowArtifacts(ctx, workflow); err != nil {
        return nil, fmt.Errorf("failed to restore artifacts: %w", err)
    }
    
    return workflow, nil
}
```

#### GraphQL API for Advanced Queries
```graphql
# GraphQL schema for workflow archive queries
type WorkflowArchive {
  id: ID!
  name: String!
  namespace: String!
  labels: [Label!]!
  phase: WorkflowPhase!
  startedAt: DateTime!
  finishedAt: DateTime!
  duration: Int!
  archivalDate: DateTime!
  retentionDate: DateTime!
  artifactLocation: String!
}

type Query {
  workflowArchives(
    namespace: String
    labels: [LabelSelector!]
    phase: WorkflowPhase
    dateRange: DateRange
    pagination: PaginationInput
  ): WorkflowArchiveConnection!
  
  workflowArchive(id: ID!): WorkflowArchive
  
  workflowStatistics(
    timeRange: DateRange!
    groupBy: [StatisticGroupBy!]!
  ): [WorkflowStatistic!]!
}

type Mutation {
  restoreWorkflow(archiveId: ID!): Workflow!
  deleteArchive(archiveId: ID!): Boolean!
  updateRetentionPolicy(
    archiveId: ID!
    newRetentionDate: DateTime!
  ): WorkflowArchive!
}
```

### 5. Automated Garbage Collection

#### Workflow Cleanup Controller
```yaml
# CronJob for automated workflow cleanup
apiVersion: batch/v1
kind: CronJob
metadata:
  name: workflow-cleanup
  namespace: argo
spec:
  schedule: "0 2 * * *"  # Daily at 2 AM
  jobTemplate:
    spec:
      template:
        spec:
          serviceAccountName: workflow-cleanup
          containers:
          - name: cleanup
            image: workflow-archiver:latest
            command:
            - /bin/bash
            - -c
            args:
            - |
              echo "Starting workflow cleanup process..."
              
              # Archive eligible workflows
              /app/archiver --action=archive --dry-run=false
              
              # Clean up expired archives
              /app/archiver --action=cleanup --dry-run=false
              
              # Generate cleanup report
              /app/archiver --action=report --output=/reports/cleanup-$(date +%Y%m%d).json
              
              echo "Cleanup process completed"
            env:
            - name: KUBECONFIG
              value: /etc/kubeconfig/config
            - name: RETENTION_CONFIG_MAP
              value: workflow-retention-policies
            volumeMounts:
            - name: kubeconfig
              mountPath: /etc/kubeconfig
              readOnly: true
            - name: reports
              mountPath: /reports
          restartPolicy: OnFailure
          volumes:
          - name: kubeconfig
            secret:
              secretName: workflow-cleanup-kubeconfig
          - name: reports
            persistentVolumeClaim:
              claimName: cleanup-reports
```

#### Cleanup Logic Implementation
```python
# Comprehensive cleanup implementation
class WorkflowCleanupManager:
    def __init__(self):
        self.k8s_client = self.init_kubernetes_client()
        self.artifact_client = self.init_artifact_client()
        self.metrics_client = self.init_metrics_client()
    
    def run_cleanup_cycle(self, dry_run=False):
        """Execute complete cleanup cycle"""
        report = {
            "timestamp": datetime.now().isoformat(),
            "dry_run": dry_run,
            "actions": [],
            "statistics": {}
        }
        
        try:
            # 1. Archive completed workflows
            archived_count = self.archive_eligible_workflows(dry_run)
            report["actions"].append({
                "action": "archive",
                "count": archived_count
            })
            
            # 2. Clean up expired archives
            deleted_count = self.cleanup_expired_archives(dry_run)
            report["actions"].append({
                "action": "delete_archives", 
                "count": deleted_count
            })
            
            # 3. Clean up orphaned artifacts
            artifact_cleanup = self.cleanup_orphaned_artifacts(dry_run)
            report["actions"].append({
                "action": "cleanup_artifacts",
                "count": artifact_cleanup["deleted_objects"],
                "size_freed": artifact_cleanup["size_freed"]
            })
            
            # 4. Update metrics
            self.update_cleanup_metrics(report)
            
            # 5. Generate statistics
            report["statistics"] = self.generate_statistics()
            
        except Exception as e:
            report["error"] = str(e)
            self.send_alert(f"Workflow cleanup failed: {e}")
        
        return report
    
    def archive_eligible_workflows(self, dry_run=False):
        """Archive workflows that meet archival criteria"""
        workflows = self.get_archival_candidates()
        archived_count = 0
        
        for workflow in workflows:
            if self.should_archive_workflow(workflow):
                if not dry_run:
                    self.archive_workflow(workflow)
                archived_count += 1
                
        return archived_count
    
    def cleanup_expired_archives(self, dry_run=False):
        """Remove archives that have exceeded retention period"""
        archives = self.get_expired_archives()
        deleted_count = 0
        
        for archive in archives:
            if not dry_run:
                self.delete_archive(archive)
            deleted_count += 1
            
        return deleted_count
    
    def cleanup_orphaned_artifacts(self, dry_run=False):
        """Remove artifacts without corresponding workflows or archives"""
        orphaned_artifacts = self.find_orphaned_artifacts()
        
        deleted_objects = 0
        size_freed = 0
        
        for artifact in orphaned_artifacts:
            if not dry_run:
                self.artifact_client.delete_object(artifact["key"])
            deleted_objects += 1
            size_freed += artifact["size"]
            
        return {
            "deleted_objects": deleted_objects,
            "size_freed": size_freed
        }
```

### 6. Monitoring and Metrics

#### Archive Metrics Collection
```yaml
# Prometheus monitoring for workflow archival
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: workflow-archive-metrics
  namespace: argo
spec:
  groups:
  - name: workflow.archive.rules
    rules:
    # Archive storage utilization
    - record: workflow_archive_storage_bytes
      expr: sum(minio_bucket_usage_total_bytes{bucket="workflow-artifacts"})
      
    # Archive count by phase
    - record: workflow_archives_total
      expr: count by (phase) (kube_customresource_info{customresource_group="argoproj.io",customresource_kind="WorkflowArchive"})
      
    # Cleanup effectiveness
    - record: workflow_cleanup_rate
      expr: rate(workflow_cleanup_total[1h])
      
    # Storage efficiency
    - record: workflow_storage_efficiency_ratio
      expr: workflow_archive_compressed_bytes / workflow_archive_raw_bytes
      
    # Alerts
    - alert: WorkflowArchiveStorageFull
      expr: workflow_archive_storage_bytes > 0.85 * minio_bucket_capacity_bytes
      for: 10m
      labels:
        severity: warning
      annotations:
        summary: "Workflow archive storage approaching capacity"
        
    - alert: WorkflowCleanupFailed
      expr: time() - workflow_cleanup_last_success_timestamp > 86400  # 24 hours
      labels:
        severity: critical
      annotations:
        summary: "Workflow cleanup has not run successfully in 24 hours"
```

## Implementation Steps

### Phase 1: Infrastructure Setup (Week 1)
1. **Artifact Repository Deployment**
   - Deploy MinIO for S3-compatible object storage
   - Configure Argo Workflows to use artifact repository
   - Test artifact storage and retrieval functionality

2. **Archive CRD and Controller**
   - Deploy WorkflowArchive custom resource definition  
   - Implement basic archive controller functionality
   - Test workflow archival process

### Phase 2: Retention Policies (Week 2)
3. **Policy Configuration**
   - Implement configurable retention policies
   - Add compliance-based retention requirements
   - Create policy override mechanisms

4. **Automated Archival**
   - Deploy CronJob for scheduled archival
   - Implement TTL-based workflow cleanup
   - Add monitoring and alerting

### Phase 3: Query and Restoration (Week 3)
5. **Archive Query API**
   - Implement RESTful API for archive queries
   - Add GraphQL interface for advanced queries
   - Create workflow restoration capabilities

6. **User Interface Integration**
   - Integrate archive queries with Argo UI
   - Add archive management dashboard
   - Implement workflow restoration UI

### Phase 4: Optimization and Monitoring (Week 4)
7. **Performance Optimization**
   - Implement artifact compression and deduplication
   - Optimize storage and query performance
   - Add caching for frequently accessed archives

8. **Comprehensive Monitoring**
   - Deploy metrics collection and alerting
   - Create operational dashboards
   - Implement capacity planning and reporting

## Success Metrics

### Storage Efficiency
- **Compression Ratio**: 60-80% storage reduction through compression
- **Cleanup Effectiveness**: 95% of eligible workflows archived within SLA
- **Storage Utilization**: Maintain 70-80% storage utilization with proper cleanup
- **Query Performance**: Archive queries complete within 5 seconds

### Compliance and Governance  
- **Retention Compliance**: 100% compliance with retention policies
- **Audit Trail**: Complete audit trail for all archival and deletion operations
- **Recovery Capability**: 99% success rate for workflow restoration from archives
- **Data Integrity**: Zero data corruption in archived workflows

### Operational Metrics
- **Automation Level**: 95% of archival operations automated
- **Alert Accuracy**: < 5% false positive rate for archival alerts
- **Recovery Time**: < 15 minutes to restore archived workflows
- **Cost Optimization**: 40% reduction in storage costs through archival

## Dependencies

### Infrastructure Requirements  
- **Object Storage**: MinIO or S3-compatible storage with sufficient capacity
- **Kubernetes Storage**: Persistent volumes for archive metadata and cleanup reports
- **Monitoring Stack**: Prometheus and Grafana for metrics and alerting

### External Dependencies
- **Backup Systems**: Regular backup of archive storage and metadata
- **Compliance Tools**: Integration with compliance monitoring systems
- **Disaster Recovery**: Cross-region replication for critical archive data

## Risk Mitigation

### Data Loss Prevention
- Implement comprehensive backup strategies for archive storage
- Use replication and versioning for critical archive data  
- Maintain audit logs for all archival and deletion operations
- Test restoration procedures regularly

### Performance Impact Mitigation
- Schedule cleanup operations during low-activity periods
- Implement incremental archival to minimize system impact
- Use compression and efficient storage formats
- Monitor and alert on archival performance issues

### Compliance Risk Management
- Regular review and validation of retention policies
- Automated compliance reporting and monitoring
- Legal hold capabilities for litigation and investigation
- Secure deletion verification for expired archives

This comprehensive workflow archival system ensures efficient resource utilization while maintaining compliance requirements and providing reliable access to historical workflow data for audit, debugging, and analysis purposes.