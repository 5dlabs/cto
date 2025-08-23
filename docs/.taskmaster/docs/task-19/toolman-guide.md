# Task 19: PR Approval Workflow - Tool Usage Guide

## Overview
This guide covers the comprehensive toolset required for implementing automated PR approval workflows with Tess validation integration, GitHub branch protection, and human review checkpoints. The implementation spans Rust development, Argo Workflows, GitHub API integration, and event-driven architecture.

## Required Tools

### 1. GitHub API and CLI Tools
**Primary Tools**: `gh`, `curl`, `jq`, `github-cli`

```bash
# GitHub CLI setup and authentication
gh auth login --scopes repo,read:org,write:repo_hook
gh auth status
gh config set git_protocol https

# PR and review operations
gh pr list --state all --limit 50
gh pr view 123 --json reviews,labels,checks
gh pr review 123 --approve --body "Automated approval by Tess"
gh pr review 123 --request-changes --body "Coverage threshold not met"

# Branch protection management
gh api repos/owner/repo/branches/main/protection --method PUT \
  --input branch-protection.json

# Webhook management
gh api repos/owner/repo/hooks --method POST \
  --field name=web \
  --field config[url]=https://webhook.example.com/github \
  --field events[]=pull_request_review
```

**GitHub API Integration Development**:
```bash
# Test API connectivity and permissions
curl -H "Authorization: token $GITHUB_TOKEN" \
     -H "Accept: application/vnd.github.v3+json" \
     https://api.github.com/user

# PR review operations
curl -X POST \
  -H "Authorization: token $GITHUB_TOKEN" \
  -H "Accept: application/vnd.github.v3+json" \
  https://api.github.com/repos/owner/repo/pulls/123/reviews \
  -d '{"event":"APPROVE","body":"Automated approval by Tess"}'

# Check current branch protection
curl -H "Authorization: token $GITHUB_TOKEN" \
     https://api.github.com/repos/owner/repo/branches/main/protection | jq .
```

### 2. Argo Workflows and Events
**Primary Tools**: `argo`, `kubectl`, `yaml-lint`

```bash
# Argo Workflows CLI operations
argo version
argo list --all-namespaces
argo get workflow-name -n taskmaster
argo logs workflow-name -n taskmaster --follow

# Workflow template management
argo template create workflows/pr-workflow-with-approval.yaml
argo template list -n taskmaster
argo submit workflows/pr-workflow-with-approval.yaml \
  --parameter repository=owner/repo \
  --parameter pr-number=123

# Workflow suspension and resumption
argo suspend workflow-name -n taskmaster
argo resume workflow-name -n taskmaster
argo retry workflow-name -n taskmaster
```

**Argo Events Management**:
```bash
# Sensor management
kubectl apply -f workflows/pr-approval-sensor.yaml
kubectl get sensors -n taskmaster
kubectl describe sensor pr-approval-sensor -n taskmaster

# EventBus management
kubectl get eventbus -n taskmaster
kubectl logs -f deployment/eventbus-default-stan -n taskmaster

# Event source monitoring
kubectl get eventsources -n taskmaster
kubectl logs -f deployment/github-webhook-eventsource -n taskmaster
```

### 3. Rust Development Environment
**Primary Tools**: `cargo`, `rust-analyzer`, `serde`

```bash
# Development setup
cargo new --lib approval-engine
cd approval-engine
cargo add serde --features derive
cargo add tokio --features full
cargo add reqwest --features json
cargo add anyhow
cargo add chrono --features serde

# Development workflow
cargo watch -x "check --bin approval-engine"
cargo test --lib approval_engine -- --nocapture
cargo clippy --all-targets --all-features
cargo fmt --all
```

**Approval Engine Development**:
```toml
# Cargo.toml dependencies
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
anyhow = "1.0"
thiserror = "1.0"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
tracing = "0.1"
```

```rust
// Basic approval engine structure
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct TessApprovalCriteria {
    pub test_coverage_threshold: f64,
    pub code_quality_score: f64,
    pub acceptance_criteria_met: bool,
    pub security_scan_passed: bool,
    pub performance_regression: bool,
    pub breaking_changes: bool,
}

pub struct TessApprovalEngine {
    github_client: GitHubClient,
    approval_threshold: f64,
}

impl TessApprovalEngine {
    pub fn new(github_token: String, threshold: f64) -> Self {
        Self {
            github_client: GitHubClient::new(github_token),
            approval_threshold: threshold,
        }
    }

    pub async fn evaluate_pr(&self, repo: &str, pr_number: u32) -> Result<ApprovalDecision> {
        // Implementation
        todo!()
    }
}
```

### 4. Kubernetes and Container Tools
**Primary Tools**: `kubectl`, `docker`, `helm`

```bash
# Kubernetes operations for workflow management
kubectl get workflows -n taskmaster -w
kubectl describe workflow pr-workflow-123 -n taskmaster
kubectl logs -f pods/pr-workflow-123-main -n taskmaster

# Configmap and secret management
kubectl create configmap approval-config \
  --from-file=approval-thresholds.yaml \
  -n taskmaster

kubectl create secret generic github-tokens \
  --from-literal=token=$GITHUB_TOKEN \
  -n taskmaster

# Service and ingress management
kubectl apply -f k8s/approval-service.yaml
kubectl port-forward svc/approval-engine 8080:80 -n taskmaster
```

**Container Development**:
```dockerfile
# Dockerfile for approval engine
FROM rust:1.70 as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y curl ca-certificates
COPY --from=builder /app/target/release/approval-engine /usr/local/bin/
CMD ["approval-engine"]
```

### 5. Testing and Validation Tools
**Primary Tools**: `pytest`, `k6`, `postman`, `mockserver`

```bash
# API testing with curl and jq
test_pr_approval() {
  local repo=$1
  local pr_number=$2

  response=$(curl -s -X POST http://localhost:8080/api/approve \
    -H "Content-Type: application/json" \
    -d "{\"repository\": \"$repo\", \"pr_number\": $pr_number}")

  decision=$(echo "$response" | jq -r '.decision')
  score=$(echo "$response" | jq -r '.score')

  echo "PR $pr_number decision: $decision (score: $score)"
}

# Load testing with k6
k6 run --vus 10 --duration 5m pr-approval-load-test.js

# Integration testing
python3 -m pytest tests/integration/test_pr_approval.py -v
```

**Mock GitHub API Server**:
```python
# mock_github.py
from flask import Flask, request, jsonify
import json

app = Flask(__name__)

@app.route('/repos/<owner>/<repo>/pulls/<int:pr_number>/reviews', methods=['POST'])
def submit_review(owner, repo, pr_number):
    review_data = request.json
    print(f"Review for {owner}/{repo}#{pr_number}: {review_data['event']}")

    return jsonify({
        "id": 12345,
        "state": review_data["event"].lower(),
        "body": review_data["body"]
    }), 200

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=3000, debug=True)
```

## Development Workflow

### Phase 1: Approval Engine Development
```bash
# 1. Setup Rust project structure
mkdir -p approval-engine/src/{github,approval,config}
cd approval-engine/

# 2. Implement core approval logic
cargo watch -x "test approval::scoring"

# 3. Test with mock GitHub API
python3 ../mock_github.py &
GITHUB_API_BASE_URL=http://localhost:3000 cargo test

# 4. Integration testing
cargo test --test integration_tests
```

### Phase 2: Workflow Template Development
```bash
# 1. Create and validate workflow templates
mkdir -p workflows/
argo lint workflows/pr-workflow-with-approval.yaml

# 2. Test workflow submission
argo submit --dry-run workflows/pr-workflow-with-approval.yaml \
  --parameter repository=test/repo \
  --parameter pr-number=123

# 3. Test sensor configuration
kubectl apply --dry-run=client -f workflows/pr-approval-sensor.yaml

# 4. End-to-end workflow testing
./test-pr-approval-e2e.sh
```

### Phase 3: GitHub Integration Testing
```bash
# 1. Setup test repository
gh repo create test-pr-approval --private
gh repo clone test-pr-approval
cd test-pr-approval

# 2. Configure branch protection
./scripts/setup-branch-protection.sh test-org test-pr-approval main

# 3. Test PR creation and approval
echo "# Test change" >> README.md
git add README.md && git commit -m "Test PR for approval workflow"
git push origin feature-branch
gh pr create --title "Test PR" --body "Testing approval workflow"

# 4. Test approval workflow
PR_NUMBER=$(gh pr list --json number --jq '.[0].number')
curl -X POST http://localhost:8080/api/evaluate-pr \
  -d "{\"repository\": \"test-org/test-pr-approval\", \"pr_number\": $PR_NUMBER}"
```

### Phase 4: Production Deployment
```bash
# 1. Build and push container image
docker build -t taskmaster/approval-engine:v1.0.0 .
docker push taskmaster/approval-engine:v1.0.0

# 2. Deploy to Kubernetes
helm upgrade --install approval-engine ./helm/approval-engine \
  --set image.tag=v1.0.0 \
  --namespace taskmaster

# 3. Deploy workflow templates
argo template create workflows/pr-workflow-with-approval.yaml
kubectl apply -f workflows/pr-approval-sensor.yaml

# 4. Verify deployment
kubectl get pods -n taskmaster -l app=approval-engine
argo template list -n taskmaster
kubectl get sensors -n taskmaster
```

## Common Issues and Solutions

### Issue 1: GitHub API Authentication Failures
**Symptoms**: 401/403 errors, authentication failures

**Diagnosis**:
```bash
# Check token validity
gh auth status
curl -I -H "Authorization: token $GITHUB_TOKEN" https://api.github.com/user

# Check token scopes
gh api user -H "Authorization: token $GITHUB_TOKEN" --include | grep x-oauth-scopes

# Verify repository access
gh api repos/owner/repo -H "Authorization: token $GITHUB_TOKEN"
```

**Solutions**:
- Ensure token has required scopes: `repo`, `write:repo_hook`, `read:org`
- Regenerate token if expired or compromised
- Use GitHub App tokens for higher rate limits
- Implement proper token rotation procedures

### Issue 2: Workflow Suspension/Resumption Problems
**Symptoms**: Workflows stuck in suspended state, resume failures

**Diagnosis**:
```bash
# Check workflow status
kubectl get workflow pr-workflow-123 -n taskmaster -o yaml

# Check sensor logs
kubectl logs deployment/pr-approval-sensor -n taskmaster

# Verify eventbus connectivity
kubectl get eventbus -n taskmaster
kubectl logs deployment/eventbus-default-stan -n taskmaster
```

**Solutions**:
- Verify sensor filters match GitHub webhook payload format
- Check eventbus and event source connectivity
- Implement workflow timeout and cleanup procedures
- Add comprehensive logging for debugging

### Issue 3: Approval Scoring Inconsistencies
**Symptoms**: Inconsistent scores, incorrect approval decisions

**Diagnosis**:
```bash
# Test scoring with known inputs
cargo test approval::scoring::test_score_calculation -- --nocapture

# Debug scoring logic
RUST_LOG=debug cargo test approval_engine -- --nocapture

# Validate scoring criteria
./test-scoring-scenarios.sh
```

**Solutions**:
- Add comprehensive unit tests for scoring edge cases
- Implement scoring validation and consistency checks
- Add detailed logging for score calculation steps
- Create test cases for all scoring scenarios

### Issue 4: Sensor Event Processing Failures
**Symptoms**: GitHub events not triggering workflows, missed approvals

**Diagnosis**:
```bash
# Check GitHub webhook delivery
gh api repos/owner/repo/hooks
curl -H "Authorization: token $GITHUB_TOKEN" \
     https://api.github.com/repos/owner/repo/hooks/123/deliveries

# Monitor sensor processing
kubectl logs -f deployment/github-webhook-eventsource -n taskmaster
kubectl logs -f deployment/pr-approval-sensor -n taskmaster

# Test webhook payload processing
curl -X POST http://localhost:12000/webhook \
  -H "Content-Type: application/json" \
  -d @test-webhook-payload.json
```

**Solutions**:
- Verify webhook URL and secret configuration
- Check event source and sensor filter configurations
- Implement event processing retries and error handling
- Add webhook payload validation and logging

## Best Practices

### Approval Engine Design
```rust
// Comprehensive error handling
#[derive(Debug, thiserror::Error)]
pub enum ApprovalError {
    #[error("GitHub API error: {0}")]
    GitHubApi(String),
    #[error("Invalid approval criteria: {0}")]
    InvalidCriteria(String),
    #[error("Scoring calculation failed: {0}")]
    ScoringError(String),
}

// Robust scoring implementation
impl TessApprovalEngine {
    fn calculate_approval_score(&self, criteria: &TessApprovalCriteria) -> Result<f64, ApprovalError> {
        let mut score = 0.0;
        let mut total_weight = 0.0;

        // Coverage scoring with validation
        if criteria.test_coverage_threshold < 0.0 || criteria.test_coverage_threshold > 100.0 {
            return Err(ApprovalError::InvalidCriteria("Invalid coverage percentage".to_string()));
        }

        // Apply weighted scoring...

        Ok((score / total_weight) * 100.0)
    }
}
```

### GitHub Integration Patterns
```rust
// Retry logic for GitHub API calls
use tokio::time::{sleep, Duration};

async fn github_api_with_retry<T, F, Fut>(operation: F, max_retries: u32) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut retries = 0;
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if retries < max_retries => {
                retries += 1;
                sleep(Duration::from_secs(2_u64.pow(retries))).await; // Exponential backoff
            }
            Err(e) => return Err(e),
        }
    }
}
```

### Workflow Template Patterns
```yaml
# Robust workflow template with proper error handling
spec:
  templates:
  - name: approval-step
    retryStrategy:
      limit: 3
      retryPolicy: "OnFailure"
      backoff:
        duration: "30s"
        factor: 2
    activeDeadlineSeconds: 600

    # Proper resource limits
    container:
      resources:
        requests:
          memory: "256Mi"
          cpu: "100m"
        limits:
          memory: "512Mi"
          cpu: "500m"
```

## Performance Optimization

### Approval Engine Performance
```rust
// Async processing for multiple criteria
use tokio::join;

impl TessApprovalEngine {
    pub async fn evaluate_pr_parallel(&self, repo: &str, pr_number: u32) -> Result<ApprovalDecision> {
        let (coverage_result, quality_result, security_result) = join!(
            self.analyze_coverage(repo, pr_number),
            self.analyze_code_quality(repo, pr_number),
            self.analyze_security(repo, pr_number)
        );

        let criteria = TessApprovalCriteria {
            test_coverage_threshold: coverage_result?,
            code_quality_score: quality_result?,
            security_scan_passed: security_result?,
            // ... other fields
        };

        self.make_approval_decision(&criteria)
    }
}
```

### GitHub API Optimization
```rust
// Batch API operations
pub async fn get_pr_details_batch(&self, repo: &str, pr_numbers: &[u32]) -> Result<Vec<PrDetails>> {
    let futures = pr_numbers.iter().map(|&pr_number| {
        self.get_pr_details(repo, pr_number)
    });

    let results = futures::future::join_all(futures).await;
    results.into_iter().collect()
}

// GraphQL for efficient data fetching
const PR_DETAILS_QUERY: &str = r#"
query($owner: String!, $name: String!, $number: Int!) {
  repository(owner: $owner, name: $name) {
    pullRequest(number: $number) {
      reviews(first: 10) {
        nodes {
          state
          author {
            login
          }
        }
      }
      labels(first: 10) {
        nodes {
          name
        }
      }
    }
  }
}
"#;
```

## Monitoring and Observability

### Metrics Collection
```rust
// Prometheus metrics
use prometheus::{Counter, Histogram, register_counter, register_histogram};

lazy_static! {
    static ref APPROVAL_DECISIONS: Counter = register_counter!(
        "approval_decisions_total",
        "Total number of approval decisions made"
    ).unwrap();

    static ref APPROVAL_DURATION: Histogram = register_histogram!(
        "approval_decision_duration_seconds",
        "Time taken to make approval decisions"
    ).unwrap();
}

impl TessApprovalEngine {
    pub async fn evaluate_pr_with_metrics(&self, repo: &str, pr_number: u32) -> Result<ApprovalDecision> {
        let _timer = APPROVAL_DURATION.start_timer();

        let decision = self.evaluate_pr(repo, pr_number).await?;

        APPROVAL_DECISIONS.inc();

        Ok(decision)
    }
}
```

### Structured Logging
```rust
// Tracing integration
use tracing::{info, warn, error, instrument};

impl TessApprovalEngine {
    #[instrument(skip(self))]
    pub async fn evaluate_pr(&self, repo: &str, pr_number: u32) -> Result<ApprovalDecision> {
        info!("Starting PR evaluation for {}/#{}", repo, pr_number);

        let criteria = self.gather_approval_criteria(repo, pr_number).await?;
        let score = self.calculate_approval_score(&criteria)?;

        info!("Calculated approval score: {:.1}%", score);

        let decision = self.make_approval_decision(&criteria)?;

        match &decision {
            ApprovalDecision::AutoApprove { score, .. } => {
                info!("Auto-approving PR with score {:.1}%", score);
            }
            ApprovalDecision::RequestChanges { score, failed_criteria } => {
                warn!("Requesting changes for PR (score: {:.1}%): {:?}", score, failed_criteria);
            }
            ApprovalDecision::RequiresHumanReview { reason, score } => {
                info!("Human review required (score: {:.1}%): {}", score, reason);
            }
        }

        Ok(decision)
    }
}
```

## Troubleshooting Checklist

### Pre-Development Setup
- [ ] GitHub CLI authenticated with appropriate scopes
- [ ] Argo Workflows and Argo Events installed in cluster
- [ ] Test repository configured with branch protection
- [ ] Development environment has required tools installed
- [ ] Mock services available for integration testing

### Development Phase
- [ ] Unit tests pass for approval scoring logic
- [ ] GitHub API integration tests succeed
- [ ] Workflow templates validate with `argo lint`
- [ ] Sensor configurations deploy without errors
- [ ] Integration tests demonstrate end-to-end functionality

### Production Deployment
- [ ] Container images build and deploy successfully
- [ ] GitHub webhooks configured and delivering events
- [ ] Workflow templates and sensors deployed correctly
- [ ] Branch protection rules enforced properly
- [ ] Monitoring and alerting functional

### Operational Monitoring
- [ ] PR approval processing completing successfully
- [ ] GitHub API rate limits not exceeded
- [ ] Workflow suspension/resumption working correctly
- [ ] Human review checkpoints functioning
- [ ] Error rates within acceptable limits
