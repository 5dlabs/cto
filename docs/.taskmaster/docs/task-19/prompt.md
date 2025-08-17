# Task 19: PR Approval Workflow - Autonomous Implementation Prompt

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


## Objective
Build a comprehensive automated PR approval workflow that integrates Tess validation results with GitHub's branch protection system. Create a multi-stage approval process with automated Tess approval (120% satisfaction threshold), event-driven workflow resumption, and human review checkpoints.

## Context
You are implementing the final stage of the Task Master quality assurance pipeline. After Tess completes testing and coverage analysis, the system must automatically evaluate PR quality and provide appropriate approvals while maintaining human oversight for critical decisions.

## Core Implementation Requirements

### 1. Tess Approval Engine
**Location**: `controller/src/github/tess_approval.rs`

Implement comprehensive PR evaluation system:
```rust
pub struct TessApprovalEngine {
    github_client: GitHubClient,
    approval_threshold: f64,  // 120% for automatic approval
}

pub struct TessApprovalCriteria {
    pub test_coverage_threshold: f64,      // From Task 18 coverage analysis
    pub code_quality_score: f64,          // Static analysis results
    pub acceptance_criteria_met: bool,     // Requirements compliance
    pub security_scan_passed: bool,       // Security vulnerability check
    pub performance_regression: bool,      // Performance impact analysis
    pub breaking_changes: bool,            // API/interface changes
}
```

Key functions to implement:
- `calculate_approval_score()` - Weighted scoring algorithm (120% threshold)
- `evaluate_pr_for_approval()` - Complete PR evaluation and decision making
- `has_blocking_issues()` - Identify critical blocking conditions
- `execute_approval_decision()` - Submit GitHub reviews and labels

### 2. Weighted Scoring Algorithm
Implement comprehensive scoring system:
- **Test Coverage (30% weight)**: Score based on coverage percentage vs. 95% target
- **Code Quality (25% weight)**: Static analysis and code quality metrics
- **Acceptance Criteria (20% weight)**: Requirements compliance check
- **Security Scan (15% weight)**: Vulnerability and security analysis
- **Performance Impact (10% weight)**: Performance regression detection
- **Breaking Changes Penalty**: Significant deduction for API changes

Score calculation example:
```rust
fn calculate_approval_score(&self, result: &TessValidationResult) -> Result<f64> {
    let mut score = 0.0;
    
    // Coverage scoring (30% weight)
    if criteria.test_coverage_threshold >= 95.0 {
        score += 30.0;
    } else {
        score += (criteria.test_coverage_threshold / 95.0) * 30.0;
    }
    
    // Additional weighted components...
    // Breaking changes penalty
    if criteria.breaking_changes {
        score -= 15.0;  // Significant penalty
    }
    
    Ok(score)
}
```

### 3. Argo Workflows PR Approval Sensor
**Location**: `workflows/pr-approval-sensor.yaml`

Create event-driven sensor for Tess approval detection:
- **Event Source**: GitHub webhook for `pull_request_review` events
- **Filter Conditions**: Review state = "approved" AND user = "5DLabs-Tess[bot]"
- **Trigger Action**: Resume suspended workflows waiting for approval
- **Data Transformation**: Extract PR number, repository, and approval timestamp

Sensor configuration:
```yaml
dependencies:
- name: tess-approval
  eventSourceName: github-webhook
  eventName: pull_request_review
  filters:
    data:
    - path: body.review.state
      value: ["approved"]
    - path: body.review.user.login
      value: ["5DLabs-Tess[bot]"]
```

### 4. Main Workflow with Approval Gates
**Location**: `workflows/pr-workflow-with-approval.yaml`

Design comprehensive workflow with multiple approval stages:
1. **Tess Validation**: Execute complete testing and analysis
2. **Wait for Approval**: Suspend workflow until Tess approves
3. **Human Review Gate**: Optional human review checkpoint
4. **Final Approval**: Combine all approvals and finalize

Workflow structure:
```yaml
templates:
- name: pr-processing-pipeline
  dag:
    tasks:
    - name: tess-validation
      template: run-tess-validation
    - name: wait-for-tess-approval
      dependencies: [tess-validation]
      template: wait-for-approval
    - name: human-review-gate
      dependencies: [wait-for-tess-approval]
      template: human-review-checkpoint
    - name: final-approval
      dependencies: [human-review-gate]
      template: finalize-pr-approval
```

### 5. GitHub Branch Protection Integration
**Location**: `scripts/setup-branch-protection.sh`

Configure GitHub branch protection rules:
- **Required Approvals**: Minimum 2 approving reviews (Tess + Human)
- **Status Checks**: Require Tess validation to pass
- **Dismiss Stale Reviews**: Auto-dismiss when new commits pushed
- **Code Owner Reviews**: Require code owner approval
- **Admin Enforcement**: Apply rules to administrators

Branch protection API call:
```bash
curl -X PUT \
  "https://api.github.com/repos/$OWNER/$REPO/branches/$BRANCH/protection" \
  -d '{
    "required_pull_request_reviews": {
      "required_approving_review_count": 2,
      "dismiss_stale_reviews": true,
      "require_code_owner_reviews": true
    },
    "required_status_checks": {
      "strict": true,
      "contexts": ["tess-validation"]
    }
  }'
```

### 6. Approval Decision Types
Implement three decision types based on evaluation:

**Auto-Approve** (Score ≥ 120%, no blocking issues):
- Submit GitHub approval review
- Add "tess-approved" label
- Log successful approval

**Request Changes** (Score < 120%):
- Submit change request review
- Detail failed criteria and required improvements
- Add "needs-changes" label

**Requires Human Review** (Score ≥ 120% but has blocking issues):
- Submit comment review requesting human oversight
- Add "human-review-required" label
- Provide detailed analysis for human reviewer

## Technical Implementation Details

### Workflow Suspension and Resumption
Implement robust workflow suspension mechanism:
```yaml
- name: wait-for-approval
  suspend: {}  # Suspend until sensor triggers resumption
  activeDeadlineSeconds: 3600  # 1 hour timeout
```

Sensor triggers workflow resumption:
```yaml
triggers:
- template:
    argoWorkflow:
      operation: resume
      source:
        # Patch workflow to resume with approval data
```

### Human Review Checkpoint
Implement optional human review stage:
- Check for existing human approvals via GitHub API
- Request human review with informative comment
- Continue workflow based on human approval status
- Support bypassing human review for low-risk changes

### Error Handling and Recovery
- **Timeout Handling**: Workflows timeout after 1 hour if no approval
- **Failed Validation**: Handle cases where Tess validation fails
- **GitHub API Failures**: Implement retry logic for API calls
- **Missing Workflows**: Handle cases where suspended workflows cannot be found

## Integration Points

### 1. Task 18 Integration
Seamlessly integrate with Tess coverage analysis:
- Consume coverage reports and test results
- Use acceptance criteria validation results
- Incorporate security scan outcomes
- Leverage generated test quality metrics

### 2. GitHub API Integration
Comprehensive GitHub integration:
- Submit PR reviews with detailed feedback
- Manage PR labels for workflow states
- Update PR status checks
- Handle GitHub webhook events

### 3. Argo Workflows Integration
Deep integration with workflow system:
- Suspend and resume workflows dynamically
- Pass approval data between workflow steps
- Handle workflow timeouts and failures
- Manage workflow metadata and labels

## Testing Strategy

### Unit Tests
Focus on core approval logic:
- Approval score calculation accuracy
- Decision making logic for different scenarios
- GitHub API integration functions
- Error handling and edge cases

### Integration Tests
End-to-end workflow validation:
- Complete PR workflow from validation to approval
- Sensor trigger and workflow resumption
- Human review checkpoint functionality
- Branch protection rule enforcement

### Scenario Testing
Test various approval scenarios:
- High-quality PRs that auto-approve
- PRs requiring changes due to low scores
- PRs requiring human review despite high scores
- Emergency override and manual approval processes

## Success Criteria

### Functional Requirements
1. **Accurate Scoring**: 120% threshold properly enforced with weighted criteria
2. **Automated Approval**: Successful GitHub review submission for qualifying PRs
3. **Workflow Resumption**: Sensor correctly resumes suspended workflows
4. **Human Review**: Proper human review checkpoint when required
5. **Branch Protection**: GitHub rules enforce approval requirements

### Performance Requirements
1. **Approval Speed**: Decision made within 30 seconds of Tess completion
2. **Sensor Latency**: Workflow resumption within 10 seconds of approval event
3. **API Efficiency**: Minimal GitHub API calls while maintaining functionality
4. **Timeout Handling**: Proper cleanup of suspended workflows after timeout

### Reliability Requirements
1. **High Success Rate**: >99% successful approval processing
2. **Error Recovery**: Graceful handling of API failures and timeouts
3. **Data Consistency**: Approval state accurately reflected across systems
4. **Audit Trail**: Complete logging of all approval decisions and actions

## Configuration Options

### Approval Thresholds
```rust
pub struct ApprovalConfig {
    pub auto_approval_threshold: f64,     // 120.0
    pub human_review_threshold: f64,      // 100.0
    pub blocking_issue_override: bool,    // false
    pub emergency_bypass: bool,           // false
}
```

### Workflow Timeouts
- Tess validation timeout: 30 minutes
- Approval wait timeout: 1 hour
- Human review timeout: 24 hours
- Overall workflow timeout: 48 hours

### Branch Protection Settings
- Required approving review count: 2
- Dismiss stale reviews: true
- Require code owner reviews: true
- Enforce admin compliance: false

## Security and Compliance

### Security Measures
- Secure GitHub token storage and rotation
- Validate approval authenticity (prevent spoofing)
- Audit all approval decisions and overrides
- Implement role-based access for emergency procedures

### Compliance Features
- Complete audit trail of approval decisions
- Immutable approval history
- Compliance reporting for regulatory requirements
- Emergency override procedures with proper authorization

## Monitoring and Observability

### Key Metrics
- Approval decision distribution (auto/changes/human review)
- Average time from validation to approval
- Human review response times
- Workflow suspension and resumption success rates

### Alerting
- Failed approval processing
- Workflow suspension timeouts
- GitHub API rate limiting
- Unusual approval patterns or failures

## Dependencies and Prerequisites
- Task 10 (GitHub integration foundation)
- Task 18 (Tess coverage analysis completion)
- Argo Workflows and Argo Events installation
- GitHub App configuration with appropriate permissions
- Branch protection rule management access