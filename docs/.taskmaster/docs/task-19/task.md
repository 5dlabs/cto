# Task 19: Implement PR Approval Workflow



## Overview
Build an automated PR approval workflow that triggers after Tess validation completion, integrating with GitHub's branch protection rules and human review checkpoints. This system provides automated quality gates while maintaining human oversight for critical decisions.

## Technical Implementation



### Architecture
The PR approval workflow implements a multi-stage approval process:
1. **Tess Validation**: Automated testing and coverage analysis
2. **Automated Approval**: Tess approves PRs meeting 120% satisfaction criteria
3. **Event Detection**: Sensor detects Tess approval events
4. **Workflow Resumption**: Suspended workflow resumes after approval
5. **Human Review Gate**: Human reviewer validates before merge
6. **Branch Protection**: GitHub rules enforce approval requirements
7. **Automated Merge**: Optional automated merge after all approvals

### Implementation Components



#### 1. Tess PR Approval Logic

**File**: `controller/src/github/tess_approval.rs`




```rust
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;



#[derive(Debug, Serialize, Deserialize)]
pub struct TessApprovalCriteria {
    pub test_coverage_threshold: f64,
    pub code_quality_score: f64,
    pub acceptance_criteria_met: bool,
    pub security_scan_passed: bool,
    pub performance_regression: bool,
    pub breaking_changes: bool,
}



#[derive(Debug, Serialize, Deserialize)]
pub struct TessValidationResult {
    pub pr_number: u32,
    pub repository: String,
    pub validation_timestamp: DateTime<Utc>,
    pub overall_score: f64,
    pub criteria: TessApprovalCriteria,
    pub detailed_results: HashMap<String, serde_json::Value>,
    pub recommendation: ApprovalRecommendation,
}



#[derive(Debug, Serialize, Deserialize)]
pub enum ApprovalRecommendation {
    Approve,
    RequestChanges,
    RequiresHumanReview,
}

pub struct TessApprovalEngine {
    github_client: super::GitHubClient,
    approval_threshold: f64,
}

impl TessApprovalEngine {
    pub fn new(github_client: super::GitHubClient, approval_threshold: f64) -> Self {
        Self {
            github_client,
            approval_threshold,
        }
    }

    pub async fn evaluate_pr_for_approval(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u32,
        validation_result: &TessValidationResult,
    ) -> Result<ApprovalDecision> {
        // Calculate comprehensive approval score
        let approval_score = self.calculate_approval_score(validation_result)?;

        // Make approval decision based on criteria
        let decision = if approval_score >= self.approval_threshold {
            if self.has_blocking_issues(validation_result) {
                ApprovalDecision::RequiresHumanReview {
                    reason: "High score but blocking issues detected".to_string(),
                    score: approval_score,
                }
            } else {
                ApprovalDecision::AutoApprove {
                    score: approval_score,
                    criteria_met: self.get_met_criteria(validation_result),
                }
            }
        } else {
            ApprovalDecision::RequestChanges {
                score: approval_score,
                failed_criteria: self.get_failed_criteria(validation_result),
                required_improvements: self.generate_improvement_recommendations(validation_result),
            }
        };

        // Execute the approval decision
        self.execute_approval_decision(repo_owner, repo_name, pr_number, &decision).await?;

        Ok(decision)
    }

    fn calculate_approval_score(&self, result: &TessValidationResult) -> Result<f64> {
        let criteria = &result.criteria;

        // Weighted scoring system
        let mut score = 0.0;
        let mut total_weight = 0.0;

        // Coverage score (30% weight)
        if criteria.test_coverage_threshold >= 95.0 {
            score += 30.0;
        } else {
            score += (criteria.test_coverage_threshold / 95.0) * 30.0;
        }
        total_weight += 30.0;

        // Code quality score (25% weight)
        score += (criteria.code_quality_score / 100.0) * 25.0;
        total_weight += 25.0;

        // Acceptance criteria (20% weight)
        if criteria.acceptance_criteria_met {
            score += 20.0;
        }
        total_weight += 20.0;

        // Security scan (15% weight)
        if criteria.security_scan_passed {
            score += 15.0;
        }
        total_weight += 15.0;

        // Performance regression (10% weight, negative scoring)
        if !criteria.performance_regression {
            score += 10.0;
        }
        total_weight += 10.0;

        // Breaking changes penalty
        if criteria.breaking_changes {
            score -= 15.0; // Significant penalty
        }

        Ok((score / total_weight) * 100.0)
    }

    fn has_blocking_issues(&self, result: &TessValidationResult) -> bool {
        let criteria = &result.criteria;

        // Critical blocking conditions
        !criteria.security_scan_passed ||
        criteria.breaking_changes ||
        criteria.test_coverage_threshold < 70.0 // Minimum acceptable coverage
    }

    async fn execute_approval_decision(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u32,
        decision: &ApprovalDecision,
    ) -> Result<()> {
        match decision {
            ApprovalDecision::AutoApprove { score, criteria_met } => {
                let review_body = self.generate_approval_review_body(*score, criteria_met);
                self.github_client.submit_pr_review(
                    repo_owner,
                    repo_name,
                    pr_number,
                    super::ReviewEvent::Approve,
                    &review_body,
                ).await?;

                // Add approval label
                self.github_client.add_labels(
                    repo_owner,
                    repo_name,
                    pr_number,
                    vec!["tess-approved".to_string()],
                ).await?;

                println!("✅ PR #{} automatically approved by Tess (score: {:.1}%)", pr_number, score);
            }

            ApprovalDecision::RequestChanges { score, failed_criteria, required_improvements } => {
                let review_body = self.generate_changes_requested_body(*score, failed_criteria, required_improvements);
                self.github_client.submit_pr_review(
                    repo_owner,
                    repo_name,
                    pr_number,
                    super::ReviewEvent::RequestChanges,
                    &review_body,
                ).await?;

                // Add needs-changes label
                self.github_client.add_labels(
                    repo_owner,
                    repo_name,
                    pr_number,
                    vec!["needs-changes".to_string()],
                ).await?;

                println!("❌ PR #{} requires changes (score: {:.1}%)", pr_number, score);
            }

            ApprovalDecision::RequiresHumanReview { reason, score } => {
                let review_body = format!(
                    "## 🤖 Tess Analysis - Human Review Required\n\n\
                     **Approval Score:** {:.1}%\n\
                     **Review Reason:** {}\n\n\
                     This PR meets most automated criteria but requires human review \
                     before final approval. Please review the detailed analysis below.",
                    score, reason
                );

                self.github_client.submit_pr_review(
                    repo_owner,
                    repo_name,
                    pr_number,
                    super::ReviewEvent::Comment,
                    &review_body,
                ).await?;

                // Add human-review-required label
                self.github_client.add_labels(
                    repo_owner,
                    repo_name,
                    pr_number,
                    vec!["human-review-required".to_string()],
                ).await?;

                println!("👀 PR #{} requires human review (score: {:.1}%)", pr_number, score);
            }
        }

        Ok(())
    }

    fn generate_approval_review_body(&self, score: f64, criteria_met: &[String]) -> String {
        format!(
            "## ✅ Tess Analysis - APPROVED\n\n\
             **Overall Score:** {:.1}%\n\
             **Approval Threshold:** {:.1}%\n\n\
             **Criteria Met:**\n{}\n\n\
             This PR has successfully passed all automated quality checks and \
             meets the requirements for automatic approval.\n\n\
             📊 [View Detailed Analysis]({{analysis_url}})",
            score,
            self.approval_threshold,
            criteria_met.iter()
                .map(|c| format!("- ✅ {}", c))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    fn generate_changes_requested_body(
        &self,
        score: f64,
        failed_criteria: &[String],
        improvements: &[String],
    ) -> String {
        format!(
            "## ❌ Tess Analysis - CHANGES REQUESTED\n\n\
             **Overall Score:** {:.1}%\n\
             **Required Score:** {:.1}%\n\n\
             **Failed Criteria:**\n{}\n\n\
             **Required Improvements:**\n{}\n\n\
             Please address these issues and push updated changes for re-evaluation.\n\n\
             📊 [View Detailed Analysis]({{analysis_url}})",
            score,
            self.approval_threshold,
            failed_criteria.iter()
                .map(|c| format!("- ❌ {}", c))
                .collect::<Vec<_>>()
                .join("\n"),
            improvements.iter()
                .map(|i| format!("- 🔧 {}", i))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}



#[derive(Debug)]
pub enum ApprovalDecision {
    AutoApprove {
        score: f64,
        criteria_met: Vec<String>,
    },
    RequestChanges {
        score: f64,
        failed_criteria: Vec<String>,
        required_improvements: Vec<String>,
    },
    RequiresHumanReview {
        reason: String,
        score: f64,
    },
}






```

#### 2. Argo Workflows PR Approval Sensor

**File**: `workflows/pr-approval-sensor.yaml`




```yaml
apiVersion: argoproj.io/v1alpha1
kind: Sensor
metadata:
  name: pr-approval-sensor
  namespace: taskmaster
spec:
  template:
    serviceAccountName: argo-events-sa
  dependencies:
  - name: tess-approval
    eventSourceName: github-webhook
    eventName: pull_request_review
    filters:
      data:
      - path: body.review.state
        type: string
        value:


        - approved
      - path: body.review.user.login
        type: string
        value:


        - "5DLabs-Tess[bot]"
    transform:
      jq: |
        {
          "repository": .body.repository.full_name,
          "pr_number": .body.number,
          "review_id": .body.review.id,
          "approved_by": .body.review.user.login,
          "approval_timestamp": .body.review.submitted_at,
          "review_body": .body.review.body
        }

  triggers:
  - template:
      name: resume-workflow-on-approval
      argoWorkflow:
        operation: resume
        source:
          resource:
            apiVersion: argoproj.io/v1alpha1
            kind: Workflow
            metadata:
              generateName: resume-pr-workflow-
            spec:
              entrypoint: handle-tess-approval
              arguments:
                parameters:
                - name: repository
                  value: "{{.Input.repository}}"
                - name: pr-number
                  value: "{{.Input.pr_number}}"
                - name: review-id
                  value: "{{.Input.review_id}}"
                - name: approval-timestamp
                  value: "{{.Input.approval_timestamp}}"

              templates:
              - name: handle-tess-approval
                inputs:
                  parameters:
                  - name: repository
                  - name: pr-number
                  - name: review-id
                  - name: approval-timestamp
                steps:
                - - name: find-waiting-workflow
                    template: find-suspended-workflow
                    arguments:
                      parameters:
                      - name: repository
                        value: "{{inputs.parameters.repository}}"
                      - name: pr-number
                        value: "{{inputs.parameters.pr-number}}"

                - - name: resume-workflow
                    template: resume-suspended-workflow
                    arguments:
                      parameters:
                      - name: workflow-name
                        value: "{{steps.find-waiting-workflow.outputs.parameters.workflow-name}}"
                      - name: approval-data
                        value: |
                          {
                            "tess_approved": true,
                            "approval_timestamp": "{{inputs.parameters.approval-timestamp}}",
                            "review_id": "{{inputs.parameters.review-id}}"
                          }

              - name: find-suspended-workflow
                inputs:
                  parameters:
                  - name: repository
                  - name: pr-number
                outputs:
                  parameters:
                  - name: workflow-name
                    valueFrom:
                      path: /tmp/workflow-name.txt
                script:
                  image: argoproj/argocd:v2.8.0
                  command: [sh]
                  source: |
                    set -e

                    REPO="{{inputs.parameters.repository}}"
                    PR_NUM="{{inputs.parameters.pr-number}}"

                    echo "Searching for suspended workflow for PR #$PR_NUM in $REPO"

                    # Find workflow suspended at pr-approval checkpoint
                    WORKFLOW_NAME=$(kubectl get workflows -n taskmaster \


                      -l "taskmaster.io/pr-number=$PR_NUM" \


                      -l "taskmaster.io/repository=$REPO" \


                      -o jsonpath='{.items[?(@.status.phase=="Running")].metadata.name}' | head -1)

                    if [ -z "$WORKFLOW_NAME" ]; then
                      echo "No suspended workflow found for PR #$PR_NUM"
                      exit 1
                    fi

                    echo "Found workflow: $WORKFLOW_NAME"
                    echo "$WORKFLOW_NAME" > /tmp/workflow-name.txt

              - name: resume-suspended-workflow
                inputs:
                  parameters:
                  - name: workflow-name
                  - name: approval-data
                script:
                  image: argoproj/workflow-controller:v3.4.4
                  command: [sh]
                  source: |
                    set -e

                    WORKFLOW_NAME="{{inputs.parameters.workflow-name}}"
                    APPROVAL_DATA='{{inputs.parameters.approval-data}}'

                    echo "Resuming workflow: $WORKFLOW_NAME"
                    echo "Approval data: $APPROVAL_DATA"

                    # Resume the workflow by setting the approval parameter
                    kubectl patch workflow "$WORKFLOW_NAME" -n taskmaster --type='merge' -p='{
                      "spec": {
                        "suspend": null,
                        "arguments": {
                          "parameters": [
                            {
                              "name": "tess-approval-data",
                              "value": "'"$APPROVAL_DATA"'"
                            }
                          ]
                        }
                      }
                    }'

                    echo "✅ Workflow resumed successfully"

  - template:
      name: notification-trigger
      http:
        url: http://notification-service.taskmaster.svc.cluster.local/webhook
        payload:
        - src:
            dataTemplate: |
              {
                "event": "tess_approval",
                "repository": "{{.Input.repository}}",
                "pr_number": {{.Input.pr_number}},
                "timestamp": "{{.Input.approval_timestamp}}",
                "message": "Tess has approved PR #{{.Input.pr_number}} in {{.Input.repository}}"
              }
          dest: body
        method: POST
        headers:
          Content-Type: application/json






```

#### 3. Main Workflow with Approval Gates

**File**: `workflows/pr-workflow-with-approval.yaml`




```yaml
apiVersion: argoproj.io/v1alpha1
kind: WorkflowTemplate
metadata:
  name: pr-workflow-with-approval
  namespace: taskmaster
spec:
  entrypoint: pr-processing-pipeline

  arguments:
    parameters:
    - name: repository
    - name: pr-number
    - name: github-token
    - name: human-review-required
      value: "true"

  templates:
  - name: pr-processing-pipeline
    dag:
      tasks:
      - name: tess-validation
        template: run-tess-validation
        arguments:
          parameters:
          - name: repository
            value: "{{workflow.parameters.repository}}"
          - name: pr-number
            value: "{{workflow.parameters.pr-number}}"

      - name: wait-for-tess-approval
        dependencies: [tess-validation]
        template: wait-for-approval
        arguments:
          parameters:
          - name: repository
            value: "{{workflow.parameters.repository}}"
          - name: pr-number
            value: "{{workflow.parameters.pr-number}}"
          - name: validation-result
            value: "{{tasks.tess-validation.outputs.parameters.validation-result}}"

      - name: human-review-gate
        dependencies: [wait-for-tess-approval]
        template: human-review-checkpoint
        when: "{{workflow.parameters.human-review-required}} == 'true'"
        arguments:
          parameters:
          - name: repository
            value: "{{workflow.parameters.repository}}"
          - name: pr-number
            value: "{{workflow.parameters.pr-number}}"
          - name: tess-approval
            value: "{{tasks.wait-for-tess-approval.outputs.parameters.approval-status}}"

      - name: final-approval
        dependencies: [human-review-gate]
        template: finalize-pr-approval
        arguments:
          parameters:
          - name: repository
            value: "{{workflow.parameters.repository}}"
          - name: pr-number
            value: "{{workflow.parameters.pr-number}}"
          - name: approval-chain
            value: |
              {
                "tess_approval": "{{tasks.wait-for-tess-approval.outputs.parameters.approval-status}}",
                "human_review": "{{tasks.human-review-gate.outputs.parameters.review-status}}"
              }

  - name: run-tess-validation
    inputs:
      parameters:
      - name: repository
      - name: pr-number
    outputs:
      parameters:
      - name: validation-result
        valueFrom:
          path: /tmp/validation-result.json
    container:
      image: taskmaster/tess:latest
      command: [bash, /scripts/container-tess.sh.hbs]
      env:
      - name: GITHUB_TOKEN
        value: "{{workflow.parameters.github-token}}"
      - name: REPO_URL
        value: "https://github.com/{{inputs.parameters.repository}}.git"
      - name: PR_NUMBER
        value: "{{inputs.parameters.pr-number}}"
      volumeMounts:
      - name: validation-output
        mountPath: /tmp
    volumes:
    - name: validation-output
      emptyDir: {}

  - name: wait-for-approval
    inputs:
      parameters:
      - name: repository
      - name: pr-number
      - name: validation-result
    outputs:
      parameters:
      - name: approval-status
        valueFrom:
          path: /tmp/approval-status.json
    suspend: {}  # This will be resumed by the sensor
    container:
      image: alpine:3.18
      command: [sh]
      args:


      - -c


      - |
        echo "Waiting for Tess approval on PR #{{inputs.parameters.pr-number}}"
        echo "Validation result: {{inputs.parameters.validation-result}}"

        # This container will be suspended until Tess approves
        # The sensor will resume the workflow and provide approval data

        if [ -n "$TESS_APPROVAL_DATA" ]; then
          echo "Tess approval received!"
          echo "$TESS_APPROVAL_DATA" > /tmp/approval-status.json
        else
          echo "No approval data received"
          echo '{"status": "timeout", "approved": false}' > /tmp/approval-status.json
        fi
    activeDeadlineSeconds: 3600  # 1 hour timeout

  - name: human-review-checkpoint
    inputs:
      parameters:
      - name: repository
      - name: pr-number
      - name: tess-approval
    outputs:
      parameters:
      - name: review-status
        valueFrom:
          path: /tmp/human-review-status.json
    script:
      image: curlimages/curl:8.4.0
      command: [sh]
      source: |
        set -e

        REPO="{{inputs.parameters.repository}}"
        PR_NUM="{{inputs.parameters.pr-number}}"
        TESS_APPROVAL='{{inputs.parameters.tess-approval}}'

        echo "Human review checkpoint for PR #$PR_NUM"
        echo "Tess approval status: $TESS_APPROVAL"

        # Check if human approval already exists
        HUMAN_APPROVALS=$(curl -s \
          -H "Authorization: token $GITHUB_TOKEN" \
          -H "Accept: application/vnd.github.v3+json" \
          "https://api.github.com/repos/$REPO/pulls/$PR_NUM/reviews" | \
          jq '[.[] | select(.state == "APPROVED" and .user.type == "User")] | length')

        if [ "$HUMAN_APPROVALS" -gt 0 ]; then
          echo "Human approval found"
          echo '{"status": "approved", "reviewer": "human", "timestamp": "'$(date -u +%Y-%m-%dT%H:%M:%SZ)'"}' > /tmp/human-review-status.json
        else
          echo "Waiting for human review..."

          # Add comment requesting human review
          curl -s -X POST \
            -H "Authorization: token $GITHUB_TOKEN" \
            -H "Accept: application/vnd.github.v3+json" \
            "https://api.github.com/repos/$REPO/issues/$PR_NUM/comments" \


            -d '{
              "body": "🤖 **Tess Approval Complete - Human Review Required**\n\nTess has completed automated validation and approved this PR. A human reviewer must now approve before merging.\n\n**Next Steps:**\n- Review the Tess validation results\n- Verify the changes meet project standards\n- Approve the PR if everything looks good\n\n**Tess Approval:** ✅ Approved\n**Coverage:** See detailed report in PR review"
            }'

          # For this demo, we'll assume human review is required but not wait
          echo '{"status": "pending", "message": "Human review requested"}' > /tmp/human-review-status.json
        fi

  - name: finalize-pr-approval
    inputs:
      parameters:
      - name: repository
      - name: pr-number
      - name: approval-chain
    script:
      image: curlimages/curl:8.4.0
      command: [sh]
      source: |
        set -e

        REPO="{{inputs.parameters.repository}}"
        PR_NUM="{{inputs.parameters.pr-number}}"
        APPROVAL_CHAIN='{{inputs.parameters.approval-chain}}'

        echo "Finalizing approval for PR #$PR_NUM"
        echo "Approval chain: $APPROVAL_CHAIN"

        # Verify all required approvals
        TESS_APPROVED=$(echo "$APPROVAL_CHAIN" | jq -r '.tess_approval.approved // false')
        HUMAN_REVIEWED=$(echo "$APPROVAL_CHAIN" | jq -r '.human_review.status == "approved"')

        if [ "$TESS_APPROVED" = "true" ] && [ "$HUMAN_REVIEWED" = "true" ]; then
          echo "✅ All approvals complete - PR ready for merge"

          # Add final approval label
          curl -s -X POST \
            -H "Authorization: token $GITHUB_TOKEN" \
            -H "Accept: application/vnd.github.v3+json" \
            "https://api.github.com/repos/$REPO/issues/$PR_NUM/labels" \
            -d '{"labels": ["ready-to-merge"]}'

          # Optional: Auto-merge if enabled
          # This would check branch protection rules and merge if possible

        else
          echo "❌ Missing required approvals"
          echo "Tess approved: $TESS_APPROVED"
          echo "Human reviewed: $HUMAN_REVIEWED"
        fi






```

#### 4. Branch Protection Configuration

**File**: `scripts/setup-branch-protection.sh`




```bash
#!/bin/bash
# Setup GitHub branch protection rules for PR approval workflow

set -euo pipefail

REPO_OWNER="${1:-}"
REPO_NAME="${2:-}"
BRANCH="${3:-main}"

if [ -z "$REPO_OWNER" ] || [ -z "$REPO_NAME" ]; then
    echo "Usage: $0 <repo-owner> <repo-name> [branch]"
    echo "Example: $0 myorg myrepo main"
    exit 1
fi

if [ -z "$GITHUB_TOKEN" ]; then
    echo "ERROR: GITHUB_TOKEN environment variable required"
    exit 1
fi

echo "Setting up branch protection for $REPO_OWNER/$REPO_NAME on branch '$BRANCH'"

# Configure branch protection rules
curl -X PUT \
  -H "Authorization: token $GITHUB_TOKEN" \
  -H "Accept: application/vnd.github.v3+json" \
  "https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/branches/$BRANCH/protection" \


  -d '{
    "required_status_checks": {
      "strict": true,
      "contexts": [
        "tess-validation",
        "continuous-integration"
      ]
    },
    "enforce_admins": false,
    "required_pull_request_reviews": {
      "required_approving_review_count": 2,
      "dismiss_stale_reviews": true,
      "require_code_owner_reviews": true,
      "dismissal_restrictions": {
        "users": [],
        "teams": []
      }
    },
    "restrictions": null,
    "allow_force_pushes": false,
    "allow_deletions": false
  }'

echo "✅ Branch protection configured successfully"

# Verify configuration
echo "Current branch protection settings:"
curl -s \
  -H "Authorization: token $GITHUB_TOKEN" \
  -H "Accept: application/vnd.github.v3+json" \
  "https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/branches/$BRANCH/protection" | \
  jq '.required_pull_request_reviews.required_approving_review_count'






```

## Testing Strategy

### Unit Tests



```rust


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approval_score_calculation() {
        let criteria = TessApprovalCriteria {
            test_coverage_threshold: 98.0,
            code_quality_score: 85.0,
            acceptance_criteria_met: true,
            security_scan_passed: true,
            performance_regression: false,
            breaking_changes: false,
        };

        let result = TessValidationResult {
            pr_number: 123,
            repository: "test/repo".to_string(),
            validation_timestamp: Utc::now(),
            overall_score: 0.0, // Will be calculated
            criteria,
            detailed_results: HashMap::new(),
            recommendation: ApprovalRecommendation::Approve,
        };

        let engine = TessApprovalEngine::new(
            mock_github_client(),


            120.0  // 120% threshold
        );

        let score = engine.calculate_approval_score(&result).unwrap();
        assert!(score >= 120.0);
    }

    #[test]
    fn test_blocking_issues_detection() {
        let engine = TessApprovalEngine::new(mock_github_client(), 120.0);

        let criteria_with_security_failure = TessApprovalCriteria {
            security_scan_passed: false,
            breaking_changes: false,
            test_coverage_threshold: 95.0,
            // ... other fields
        };

        let result = create_test_validation_result(criteria_with_security_failure);
        assert!(engine.has_blocking_issues(&result));
    }

    #[tokio::test]
    async fn test_pr_approval_workflow() {
        let mut mock_client = mock_github_client();
        mock_client
            .expect_submit_pr_review()
            .times(1)
            .returning(|_, _, _, _, _| Ok(()));

        let engine = TessApprovalEngine::new(mock_client, 120.0);
        let validation_result = create_high_score_validation_result();

        let decision = engine
            .evaluate_pr_for_approval("owner", "repo", 123, &validation_result)
            .await
            .unwrap();

        match decision {
            ApprovalDecision::AutoApprove { score, .. } => {
                assert!(score >= 120.0);
            }
            _ => panic!("Expected auto-approval decision"),
        }
    }
}






```

### Integration Tests



```bash
#!/bin/bash
# Integration test for PR approval workflow

set -euo pipefail

echo "=== PR Approval Workflow Integration Test ==="

# Setup test environment
REPO="test-org/test-repo"
PR_NUMBER="123"
GITHUB_TOKEN="test-token"

# 1. Test Tess validation and approval
echo "Step 1: Running Tess validation..."
VALIDATION_RESULT=$(curl -X POST http://localhost:8080/api/tess/validate \
  -H "Content-Type: application/json" \
  -d "{\"repository\": \"$REPO\", \"pr_number\": $PR_NUMBER}")

echo "Validation result: $VALIDATION_RESULT"



# 2. Check if Tess approved the PR
echo "Step 2: Checking Tess approval..."
APPROVAL_STATUS=$(echo "$VALIDATION_RESULT" | jq -r '.recommendation')

if [ "$APPROVAL_STATUS" = "Approve" ]; then
  echo "✅ Tess approved the PR"
else
  echo "❌ Tess did not approve the PR: $APPROVAL_STATUS"
  exit 1
fi

# 3. Test workflow sensor trigger
echo "Step 3: Triggering approval sensor..."
curl -X POST http://localhost:12000/webhook \
  -H "Content-Type: application/json" \


  -d '{
    "action": "submitted",
    "review": {
      "state": "approved",
      "user": {"login": "5DLabs-Tess[bot]"},
      "submitted_at": "'$(date -u +%Y-%m-%dT%H:%M:%SZ)'"
    },
    "pull_request": {"number": '$PR_NUMBER'},
    "repository": {"full_name": "'$REPO'"}
  }'



# 4. Verify workflow resumed
echo "Step 4: Checking workflow resumption..."
sleep 5  # Allow time for processing

WORKFLOWS=$(kubectl get workflows -n taskmaster -l "taskmaster.io/pr-number=$PR_NUMBER" -o json)
RUNNING_COUNT=$(echo "$WORKFLOWS" | jq '[.items[] | select(.status.phase == "Running")] | length')

if [ "$RUNNING_COUNT" -gt 0 ]; then
  echo "✅ Workflow resumed successfully"
else
  echo "❌ Workflow not resumed"
  exit 1
fi

echo "=== Integration test completed successfully ==="






```

## Performance Considerations

1. **Approval Scoring**: Cache expensive calculations for repeated evaluations
2. **GitHub API Rate Limits**: Implement intelligent rate limiting and batching
3. **Workflow State**: Efficient storage and retrieval of suspended workflows
4. **Sensor Processing**: Optimize event filtering and processing speed

## Security Considerations

1. **Token Management**: Secure storage and rotation of GitHub tokens
2. **Review Authenticity**: Verify review submissions are from legitimate Tess instances
3. **Branch Protection**: Ensure protection rules cannot be bypassed
4. **Human Override**: Provide mechanism for emergency approvals when needed
