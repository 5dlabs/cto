# Acceptance Criteria: Analyze Existing CodeRun Controller Architecture

## Functional Requirements

### 1. CRD Structure Documentation


- [ ] Complete analysis of CodeRun CRD specification documented


- [ ] All fields relevant to multi-agent support identified


- [ ] `github_app` field usage and capabilities fully mapped


- [ ] Secret and environment variable injection patterns documented


- [ ] Session continuity mechanism (`continue_session`) explained

### 2. Controller Logic Analysis


- [ ] Reconciliation flow documented from CRD submission to pod creation


- [ ] Status-first idempotency pattern fully explained


- [ ] TTL safety mechanisms and cleanup logic documented


- [ ] Finalizer implementation and resource deletion flow mapped


- [ ] Error handling and retry mechanisms identified



### 3. Template System Architecture


- [ ] Complete template file inventory created


- [ ] Handlebars context data structure documented


- [ ] Variable substitution mechanisms explained


- [ ] Agent-specific customization points identified


- [ ] Template loading and rendering pipeline mapped

### 4. Infrastructure Integration


- [ ] Argo Events configuration verified and documented


- [ ] GitHub webhook processing flow mapped


- [ ] Event correlation mechanisms identified


- [ ] External Secrets integration validated


- [ ] Authentication flow from secrets to agents traced

### 5. Discovery Report Generation


- [ ] Comprehensive technical analysis document created (minimum 30 pages)


- [ ] Architecture diagrams included


- [ ] Code examples provided for key patterns


- [ ] Modification requirements clearly prioritized


- [ ] Implementation roadmap with dependencies defined

## Technical Requirements



### Code Coverage
- [ ] All controller source files reviewed:


  - `controller/src/crds/coderun.rs`


  - `controller/src/tasks/code/controller.rs`


  - `controller/src/tasks/code/resources.rs`


  - `controller/src/tasks/code/templates.rs`

### Template Analysis


- [ ] All template files in `infra/charts/controller/claude-templates/` reviewed


- [ ] Handlebars helper functions documented


- [ ] Conditional logic capabilities tested


- [ ] Agent-specific template possibilities identified

### Runtime Validation


- [ ] Test CodeRun CRDs submitted and behavior observed


- [ ] Controller logs analyzed for reconciliation patterns


- [ ] Resource creation verified with `kubectl describe`


- [ ] Pod lifecycle and status updates monitored



## Test Cases

### Test Case 1: CRD Processing Flow
**Objective**: Validate understanding of CRD → Pod flow

**Steps**:


1. Submit a test CodeRun CRD


2. Monitor controller logs


3. Track resource creation (PVC, ConfigMap, Pod)


4. Verify status updates

**Expected Result**: Complete trace of reconciliation flow documented

### Test Case 2: Template Rendering
**Objective**: Verify template system capabilities

**Steps**:


1. Create test CRD with specific `github_app` value


2. Observe generated ConfigMap contents


3. Verify variable substitution


4. Test conditional logic rendering

**Expected Result**: Template customization points identified

### Test Case 3: Session Continuity
**Objective**: Understand `continue_session` behavior

**Steps**:
1. Submit CRD with `continue_session: true`


2. Observe PVC handling


3. Verify workspace persistence


4. Test session restoration

**Expected Result**: Session continuity mechanism documented

### Test Case 4: Webhook Correlation
**Objective**: Validate event processing understanding

**Steps**:


1. Trigger GitHub webhook


2. Monitor Argo Events processing


3. Verify payload extraction


4. Test correlation logic

**Expected Result**: Event correlation mechanism mapped

### Test Case 5: Multi-Agent Differentiation
**Objective**: Confirm agent-specific capabilities

**Steps**:


1. Submit CRDs with different `github_app` values


2. Compare generated resources


3. Verify secret mounting differences


4. Test environment variable injection

**Expected Result**: Agent differentiation mechanisms documented



## Quality Criteria

### Documentation Standards


- [ ] Clear, technical writing with proper terminology


- [ ] Code examples included where relevant


- [ ] Diagrams for complex flows


- [ ] Cross-references to source files


- [ ] Glossary of technical terms

### Completeness


- [ ] All subtasks from task definition completed


- [ ] No unexplored areas in the controller logic


- [ ] All template files reviewed


- [ ] Infrastructure components fully mapped



### Accuracy


- [ ] All findings validated with actual testing


- [ ] No assumptions without verification


- [ ] Code examples tested and working


- [ ] Reconciliation patterns confirmed with logs



## Deliverable Checklist



- [ ] `task-1-discovery-report.md` created (minimum 30 pages)


- [ ] Architecture diagrams included


- [ ] Modification requirements document


- [ ] Implementation roadmap


- [ ] Risk assessment and mitigation strategies


- [ ] Backward compatibility analysis


- [ ] Testing strategy for modifications



## Success Metrics

1. **Comprehensive Coverage**: 100% of controller code analyzed
2. **Documentation Quality**: Technical depth equivalent to 47-page report
3. **Validation**: All findings confirmed through testing
4. **Actionability**: Clear modification path identified
5. **Compatibility**: No breaking changes for existing workflows



## Notes



- This is a discovery and documentation task - no implementation required


- Focus on understanding existing patterns before proposing changes


- Consider the needs of Rex, Cleo, and Tess agents in analysis


- Document rate limiting and operational considerations


- Include security implications of multi-agent support