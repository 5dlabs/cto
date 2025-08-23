# Autonomous Agent Prompt: Workflow Archival System Implementation

## Mission Statement
You are a data lifecycle management expert tasked with implementing a comprehensive workflow archival and cleanup system for a multi-agent orchestration platform. Your goal is to build automated archival processes that efficiently manage workflow storage, ensure compliance with retention policies, and provide reliable access to historical workflow data.

## System Context
You are working with an Argo Workflows-based system where:
- **Long-running workflows** can persist for weeks, generating substantial metadata and artifacts
- **Multi-agent executions** produce extensive logs, artifacts, and state information across Rex, Cleo, and Tess agents
- **Compliance requirements** mandate retention of certain workflows for audit and regulatory purposes
- **Storage costs** accumulate rapidly without proper lifecycle management
- **Historical access** is needed for debugging, analytics, and compliance reporting

## Primary Objectives

### 1. Artifact Repository Architecture
Design and implement a scalable object storage solution:

**S3-Compatible Storage Setup**:
- Deploy MinIO or configure AWS S3 for workflow artifact storage
- Implement bucket policies for lifecycle management and access control
- Configure compression and deduplication to optimize storage efficiency
- Set up cross-region replication for disaster recovery and high availability

**Argo Workflows Integration**:
- Configure Argo Workflows controller to use artifact repository
- Implement automatic artifact uploading for all workflow outputs
- Set up artifact retention policies aligned with workflow lifecycle
- Ensure artifact metadata correlation with workflow execution records

### 2. Intelligent Retention Policy Engine
Build a sophisticated policy management system:

**Multi-Tiered Retention Policies**:
- Default retention periods based on workflow completion status (success, failure, error)
- Compliance-based retention for audit requirements (7 years for financial, 5 years for security)
- Priority-based retention for critical workflows requiring extended preservation
- Dynamic policy application based on workflow labels, annotations, and execution context

**Policy Override Mechanisms**:
- Legal hold capabilities for litigation and investigation requirements
- Manual retention extensions for ongoing analysis and debugging
- Emergency deletion procedures for security incidents or data breaches
- Automated policy validation and compliance reporting

### 3. Automated Archival Orchestration
Implement comprehensive automation for workflow lifecycle management:

**Time-Based Archival**:
- Automated detection of workflows eligible for archival based on completion time and retention policies
- Staged archival process: hot storage → warm storage → cold storage → deletion
- Batch processing for efficient resource utilization during archival operations
- Incremental archival to minimize system impact and improve performance

**Event-Driven Archival**:
- Immediate archival triggers for specific workflow types or security events
- Integration with external systems for compliance-driven archival requirements
- Workflow completion hooks for automatic artifact collection and initial archival processing
- Real-time policy evaluation and archival decision making

### 4. Archive Query and Restoration Service
Build comprehensive access capabilities for archived workflows:

**Multi-Modal Query Interface**:
- RESTful API for programmatic access to archived workflow metadata
- GraphQL interface for complex queries and data relationships
- Full-text search capabilities across workflow logs and artifact content
- Time-range queries with efficient indexing for performance optimization

**Workflow Restoration Capabilities**:
- Complete workflow restoration from archived state for debugging and reanalysis
- Partial restoration of specific artifacts or logs without full workflow recreation
- Restoration validation to ensure data integrity and completeness
- Progress tracking and status reporting for long-running restoration operations

### 5. Compliance and Audit Framework
Ensure comprehensive governance and auditability:

**Audit Trail Management**:
- Complete logging of all archival, retrieval, and deletion operations
- Immutable audit records with cryptographic integrity verification
- Compliance reporting with automated generation of retention and deletion reports
- Integration with external audit and compliance monitoring systems

**Data Integrity Verification**:
- Checksum validation for all archived artifacts and metadata
- Regular integrity checks with automated corruption detection and alerting
- Backup verification and restoration testing on scheduled intervals
- Chain of custody documentation for legal and compliance requirements

## Technical Implementation Guidelines

### Storage Architecture Patterns
```yaml
# Multi-tier storage configuration
storage_tiers:
  hot_storage:
    access_pattern: "frequent"
    retention_period: "30d"
    storage_class: "STANDARD"

  warm_storage:
    access_pattern: "infrequent"
    retention_period: "1y"
    storage_class: "STANDARD_IA"

  cold_storage:
    access_pattern: "archive"
    retention_period: "7y"
    storage_class: "GLACIER"

  compliance_storage:
    access_pattern: "deep_archive"
    retention_period: "indefinite"
    storage_class: "DEEP_ARCHIVE"
```

### Archival Workflow Automation
```python
# Example archival orchestration logic
archival_pipeline:
  stages:
    - name: "eligibility_assessment"
      criteria:
        - workflow_status: ["Succeeded", "Failed", "Error"]
        - completion_age: "> 24h"
        - retention_policy_match: true

    - name: "artifact_collection"
      operations:
        - collect_workflow_logs
        - gather_agent_artifacts
        - compress_and_validate
        - generate_manifest

    - name: "metadata_preservation"
      data:
        - workflow_specification
        - execution_timeline
        - resource_utilization
        - compliance_annotations

    - name: "storage_optimization"
      techniques:
        - deduplication
        - compression
        - efficient_indexing
        - metadata_extraction
```

### Query Optimization Strategies
```sql
-- Example query patterns for archived workflow search
-- Time-range queries with efficient indexing
SELECT * FROM workflow_archives
WHERE archival_date BETWEEN ? AND ?
  AND workflow_labels @> ?
  AND retention_policy = ?
ORDER BY archival_date DESC
LIMIT 100;

-- Full-text search across workflow content
SELECT workflow_id, relevance_score
FROM workflow_search_index
WHERE search_vector @@ to_tsquery(?)
  AND archival_tier IN ('warm', 'cold')
ORDER BY relevance_score DESC;
```

## Success Criteria

### Storage Efficiency and Cost Optimization
- **Storage Reduction**: Achieve 60-80% storage reduction through compression and deduplication
- **Tiered Storage**: Automatically transition 90% of archived workflows to appropriate storage tiers
- **Cost Savings**: Reduce storage costs by 40% compared to maintaining all workflows in active storage
- **Cleanup Effectiveness**: Archive 95% of eligible workflows within defined SLA periods

### Performance and Reliability
- **Query Performance**: Archive queries complete within 5 seconds for metadata, 30 seconds for full workflow restoration
- **Archival Speed**: Process 1000+ workflows per hour during bulk archival operations
- **Data Integrity**: Maintain 99.999% data integrity with zero corruption in archived workflows
- **System Availability**: Archive services maintain 99.9% uptime with graceful degradation capabilities

### Compliance and Governance
- **Retention Compliance**: 100% compliance with defined retention policies and regulatory requirements
- **Audit Completeness**: Complete audit trail for all data lifecycle operations with no gaps
- **Recovery Capability**: 99% success rate for workflow restoration from any archived state
- **Policy Enforcement**: Automated enforcement of retention policies with zero manual intervention required

## Implementation Approach

### Phase 1: Foundation Infrastructure
1. **Storage Platform Setup**: Deploy and configure object storage infrastructure with proper security and access controls
2. **Basic Archival Process**: Implement core archival functionality for workflow metadata and artifacts
3. **Policy Framework**: Create configurable retention policy engine with basic rule evaluation

### Phase 2: Automation and Intelligence
4. **Automated Scheduling**: Deploy scheduled archival processes with intelligent batch processing
5. **Advanced Policies**: Implement compliance-based retention with override mechanisms
6. **Quality Assurance**: Add data integrity verification and corruption detection capabilities

### Phase 3: Access and Restoration
7. **Query Services**: Build comprehensive API layer for archived workflow access
8. **Restoration Engine**: Implement complete workflow restoration with validation and progress tracking
9. **User Interfaces**: Create dashboard and CLI tools for archive management and query operations

### Phase 4: Optimization and Governance
10. **Performance Tuning**: Optimize query performance, storage efficiency, and archival throughput
11. **Compliance Integration**: Connect with external compliance and audit systems
12. **Operational Excellence**: Implement comprehensive monitoring, alerting, and operational procedures

## Key Constraints and Considerations

### Regulatory and Compliance Requirements
- Ensure archival processes meet industry-specific retention requirements (SOX, GDPR, HIPAA)
- Implement secure deletion capabilities with cryptographic verification
- Maintain data sovereignty and jurisdictional compliance for cross-border data storage
- Provide litigation hold capabilities with immutable preservation guarantees

### Performance and Scalability Constraints
- Design for handling thousands of workflows with terabytes of artifacts
- Minimize performance impact on active workflow execution during archival operations
- Ensure query performance scales with archive size growth over time
- Balance storage costs with access performance requirements across different tiers

### Security and Access Control
- Implement role-based access control for archived workflow data
- Ensure encryption at rest and in transit for all archived content
- Maintain audit logs with tamper-evident properties for compliance verification
- Integrate with organizational identity and access management systems

### Operational Excellence Requirements
- Design for minimal operational overhead with high automation levels
- Implement comprehensive monitoring and alerting for all archival processes
- Create runbooks and procedures for common operational scenarios
- Ensure disaster recovery and business continuity for archived data

Your expertise in data lifecycle management, storage systems, and compliance frameworks is essential to building a robust archival system that balances efficiency, compliance, and accessibility while supporting the long-term operational needs of the multi-agent workflow orchestration platform.
