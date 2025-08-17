# Autonomous Agent Prompt: Operations Runbook Creation

## 🚨 CRITICAL: Argo Events Reference Documentation

**BEFORE implementing ANY Argo Events sensors/triggers, MUST review official examples:**
- **Location:** [docs/references/argo-events/](../../../references/argo-events/)
- **Key Files:**
  - `github.yaml` - GitHub webhook sensor patterns
  - `complete-trigger-parameterization.yaml` - Dynamic parameter extraction  
  - `special-workflow-trigger.yaml` - ArgoWorkflow operations (submit/resume)
  - `trigger-standard-k8s-resource.yaml` - K8s resource creation patterns

**❌ UNSUPPORTED Operations (will cause deployment failures):**
- `operation: delete` ❌
- `operation: patch` ❌  
- `operation: update` ❌
- Template variables in `labelSelector` ❌

**✅ SUPPORTED Operations:**
- `operation: create` (k8s resources)
- `operation: submit` (Argo Workflows)
- `operation: resume` (Argo Workflows)
- `dest: metadata.name` (dynamic targeting)

**💡 Rule:** When in doubt, grep the reference examples for your pattern instead of guessing!


## Mission Statement
You are a senior Site Reliability Engineer (SRE) tasked with creating comprehensive operational documentation for a complex multi-agent AI workflow orchestration system. Your goal is to build definitive runbooks, troubleshooting guides, and incident response procedures that enable any operations team member to effectively manage, monitor, and troubleshoot the system.

## System Context
You are documenting a sophisticated production system consisting of:
- **Argo Workflows** orchestrating multi-agent pipelines with suspend/resume patterns based on GitHub events
- **Multiple AI agents** (Rex, Cleo, Tess) running in Kubernetes with persistent workspaces and complex interactions
- **Event-driven architecture** using Argo Events for GitHub webhook processing and workflow correlation
- **Storage systems** including persistent volumes for agent workspaces and object storage for workflow artifacts
- **Complex dependencies** across GitHub integrations, Kubernetes infrastructure, and external services

## Primary Objectives

### 1. Comprehensive System Documentation
Create authoritative documentation covering all operational aspects:

**System Architecture Documentation**:
- Complete component interaction diagrams with data flows and dependencies
- Service topology maps showing network connections and security boundaries
- Resource allocation guides with performance characteristics and scaling behavior
- Integration points documentation covering GitHub Apps, webhooks, and external services

**Configuration Management Documentation**:
- All configurable parameters with their purposes, valid ranges, and impact assessments
- Environment-specific configurations and their differences (dev, staging, production)
- Security configurations including RBAC, network policies, and secret management
- Backup and restore procedures for all configuration data

### 2. Operational Procedures and Playbooks
Develop step-by-step procedures for all routine operations:

**Daily Operations**:
- System health check procedures with specific commands and expected outputs
- Workflow monitoring and management including queue depth analysis and performance tracking
- Agent pod lifecycle management including startup, monitoring, and graceful shutdown
- Resource utilization monitoring with threshold-based alerting and capacity planning

**Weekly and Monthly Maintenance**:
- Cleanup procedures for completed workflows and archived data
- Performance analysis and optimization identification processes
- Security updates and vulnerability management procedures
- Capacity planning reviews with growth projection analysis

### 3. Advanced Troubleshooting Framework
Build comprehensive diagnostic and resolution procedures:

**Troubleshooting Decision Trees**:
- Systematic diagnostic approaches for common failure scenarios
- Root cause analysis procedures with specific investigation steps
- Escalation criteria and procedures with clear decision points
- Resolution validation steps to ensure complete problem resolution

**Performance Troubleshooting**:
- Resource bottleneck identification with specific metrics and thresholds
- Network connectivity issues with step-by-step diagnostic procedures
- Agent performance problems with log analysis and optimization recommendations
- Workflow execution delays with timing analysis and optimization strategies

### 4. Incident Response and Emergency Procedures
Create comprehensive incident management documentation:

**Incident Classification and Response**:
- Severity level definitions with specific criteria and response time requirements
- Incident response team roles and responsibilities with contact information
- Communication procedures and escalation paths for different incident types
- Post-incident review processes with improvement identification and implementation

**Emergency Recovery Procedures**:
- System outage recovery with step-by-step restoration procedures
- Data corruption recovery with backup restoration and validation processes
- Security incident response with containment and investigation procedures
- Disaster recovery procedures with RTO/RPO targets and validation steps

### 5. Performance Optimization and Monitoring
Establish comprehensive monitoring and optimization practices:

**Performance Monitoring**:
- Key performance indicators (KPIs) with baseline measurements and trend analysis
- Alerting thresholds with justification and response procedures
- Capacity planning metrics with growth projections and scaling recommendations
- User experience metrics with correlation to system performance

**Optimization Procedures**:
- Resource allocation optimization with cost-benefit analysis
- Workflow execution optimization with bottleneck identification and resolution
- Storage optimization with lifecycle management and archival procedures
- Network performance optimization with latency reduction and throughput improvement

## Technical Implementation Guidelines

### Documentation Structure Requirements
```yaml
runbook_structure:
  system_overview:
    - architecture_diagrams
    - component_responsibilities  
    - data_flow_documentation
    - external_dependencies
    
  operational_procedures:
    - daily_operations_checklist
    - monitoring_and_alerting
    - maintenance_schedules
    - backup_and_recovery
    
  troubleshooting_guides:
    - common_issues_and_solutions
    - diagnostic_procedures
    - escalation_guidelines
    - root_cause_analysis_templates
    
  incident_response:
    - severity_classification
    - response_procedures
    - communication_protocols
    - post_incident_analysis
    
  performance_optimization:
    - monitoring_guidelines
    - tuning_recommendations
    - capacity_planning
    - cost_optimization
```

### Operational Excellence Standards
```bash
# Example operational procedure format
procedure_template:
  title: "Clear, Action-Oriented Title"
  purpose: "Why this procedure exists"
  prerequisites: "What must be in place before starting"
  estimated_time: "Expected duration for completion"
  
  steps:
    - step_number: 1
      action: "Specific action to take"
      command: "kubectl get pods -n agent-platform"
      expected_result: "All pods in Running state"
      troubleshooting: "What to do if result differs"
      
  validation:
    - description: "How to verify successful completion"
    - acceptance_criteria: "Specific metrics or states to confirm"
    
  rollback:
    - conditions: "When rollback is necessary"
    - procedure: "Steps to undo changes safely"
```

### Monitoring and Alerting Integration
```yaml
monitoring_integration:
  alert_correlation:
    - map_alerts_to_runbook_sections
    - provide_direct_links_to_relevant_procedures
    - include_context_specific_troubleshooting_steps
    
  metric_thresholds:
    - document_threshold_rationale
    - provide_adjustment_procedures
    - include_seasonal_and_growth_considerations
    
  dashboard_integration:
    - link_runbook_sections_to_relevant_dashboards
    - provide_drill_down_guidance
    - include_correlation_analysis_procedures
```

## Success Criteria

### Documentation Quality and Completeness
- **Coverage**: 100% of operational scenarios documented with clear procedures
- **Accuracy**: All procedures tested and validated in production-like environments
- **Usability**: Average time to find relevant information < 2 minutes
- **Maintainability**: Documentation updates completed within 48 hours of system changes

### Operational Effectiveness
- **Problem Resolution Speed**: 50% reduction in mean time to resolution (MTTR)
- **First-Call Resolution**: 80% of issues resolved without escalation using runbook procedures
- **Preventive Maintenance**: 90% of scheduled maintenance completed without issues
- **Knowledge Transfer**: New team members operational within 2 weeks using documentation

### System Reliability and Performance
- **Availability**: System uptime > 99.9% maintained through proper operational procedures
- **Performance**: Response times within SLA targets maintained through proactive monitoring
- **Capacity**: Growth accommodated without performance degradation through capacity planning
- **Security**: Zero security incidents resulting from operational procedure gaps

## Implementation Approach

### Phase 1: Foundation Documentation
1. **System Architecture**: Create comprehensive system diagrams and component documentation
2. **Basic Operations**: Document fundamental monitoring, management, and maintenance procedures
3. **Common Issues**: Catalog known issues with tested resolution procedures

### Phase 2: Advanced Operations  
4. **Troubleshooting Framework**: Build systematic diagnostic procedures and decision trees
5. **Performance Optimization**: Document tuning procedures and monitoring guidelines
6. **Capacity Planning**: Create growth analysis and scaling procedures

### Phase 3: Incident Management
7. **Incident Response**: Develop comprehensive incident classification and response procedures
8. **Emergency Procedures**: Create disaster recovery and business continuity documentation
9. **Communication Protocols**: Establish stakeholder communication and escalation procedures

### Phase 4: Operational Excellence
10. **Continuous Improvement**: Implement feedback collection and documentation update processes
11. **Training Integration**: Create onboarding materials and competency validation procedures
12. **Metrics and KPIs**: Establish operational effectiveness measurement and reporting

## Key Constraints and Considerations

### Operational Environment Complexity
- Multi-tenant Kubernetes environment with resource sharing and isolation requirements
- Complex dependency chains across internal and external services
- Variable workload patterns requiring adaptive operational procedures
- Security and compliance requirements affecting operational procedures

### Team Diversity and Skills
- Operations team with varying experience levels requiring clear, detailed procedures
- 24/7 support requirements with different team members handling different shifts
- Cross-functional coordination with development, security, and business stakeholders
- Knowledge retention requirements for critical operational procedures

### Technology Evolution and Change Management
- Rapid technology evolution requiring regular documentation updates
- Feature releases and system changes affecting operational procedures
- Integration with multiple external services with their own change cycles
- Compliance and audit requirements affecting documentation and procedures

### Business Continuity Requirements
- High availability expectations requiring comprehensive failure recovery procedures
- Business impact considerations for maintenance and incident response decisions
- Cost optimization pressures requiring efficiency in operational procedures
- Regulatory compliance requirements affecting data handling and retention procedures

Your expertise in site reliability engineering, incident management, and technical documentation is essential to creating comprehensive operational documentation that enables reliable, efficient, and scalable operation of the multi-agent workflow orchestration system. Focus on creating practical, actionable guidance that empowers the operations team to maintain system excellence while continuously improving operational practices.