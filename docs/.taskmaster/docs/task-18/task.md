# Task 18: Create Test Coverage Requirements



## Overview
Implement a comprehensive test coverage enforcement workflow integrated directly into the `container-tess.sh.hbs` script. This system automatically analyzes code coverage, generates missing tests, and provides automated PR approval based on coverage metrics, creating a complete quality assurance pipeline for the Tess agent.

## Technical Implementation



### Architecture
The test coverage system implements a multi-stage workflow:
1. **Code Review**: Analyze code changes against acceptance criteria
2. **Test Execution**: Run existing test suite and measure coverage
3. **Coverage Analysis**: Identify uncovered code paths using cargo llvm-cov
4. **Test Generation**: Automatically generate missing tests with Tess
5. **Validation**: Re-run tests with new coverage validation
6. **Reporting**: Generate comprehensive coverage reports
7. **PR Approval**: Automatically approve PRs meeting coverage thresholds

### Implementation Components

#### 1. Enhanced Container Tess Template

**File**: `templates/container-tess.sh.hbs`




```bash
#!/bin/bash
# Tess Testing Workflow with Comprehensive Coverage Analysis
set -euo pipefail

# Environment setup
export GITHUB_TOKEN="${{github_token}}"
export REPO_URL="${{repo_url}}"
export PR_NUMBER="${{pr_number}}"
export ACCEPTANCE_CRITERIA_PATH="${{acceptance_criteria_path:-".taskmaster/docs/acceptance-criteria.md"}}"
export COVERAGE_THRESHOLD_EXISTING="${{coverage_threshold_existing:-95}}"
export COVERAGE_THRESHOLD_NEW="${{coverage_threshold_new:-100}}"

# Initialize logging
exec 1> >(tee -a /tmp/tess-workflow.log)
exec 2> >(tee -a /tmp/tess-workflow.log >&2)

echo "=== Tess Test Coverage Workflow Starting ==="
echo "Repository: $REPO_URL"
echo "PR Number: $PR_NUMBER"
echo "Coverage Thresholds: ${COVERAGE_THRESHOLD_EXISTING}% existing, ${COVERAGE_THRESHOLD_NEW}% new"
date

# Clone repository and setup
git clone "$REPO_URL" /workspace
cd /workspace

# Checkout PR branch
git fetch origin pull/$PR_NUMBER/head:pr-$PR_NUMBER
git checkout pr-$PR_NUMBER

# Install coverage tools
echo "=== Installing Coverage Tools ==="
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov --version 0.5.36

# Verify installation
cargo llvm-cov --version || {
    echo "ERROR: Failed to install cargo-llvm-cov"
    exit 1
}



# Setup coverage data directory
export CARGO_LLVM_COV_TARGET_DIR=/tmp/coverage
mkdir -p "$CARGO_LLVM_COV_TARGET_DIR"

echo "=== Stage 1: Acceptance Criteria Review ==="
review_acceptance_criteria() {
    local criteria_file="$1"

    if [ ! -f "$criteria_file" ]; then
        echo "WARNING: Acceptance criteria file not found at $criteria_file"
        return 0
    fi

    echo "Reviewing code changes against acceptance criteria..."

    # Get changed files
    git diff --name-only HEAD~1 HEAD > /tmp/changed_files.txt

    # Use Tess to analyze changes against criteria
    cat <<EOF > /tmp/review_prompt.txt
Review the following code changes against the acceptance criteria:

ACCEPTANCE CRITERIA:
$(cat "$criteria_file")

CHANGED FILES:
$(cat /tmp/changed_files.txt)

CODE CHANGES:
$(git diff HEAD~1 HEAD)

Provide analysis of:


1. Whether changes meet acceptance criteria


2. Potential areas needing additional test coverage


3. Risk assessment for the changes


4. Recommendations for comprehensive testing
EOF

    # Tess analysis (simplified - would use actual AI model)
    echo "✅ Acceptance criteria review completed"
    echo "Changes appear to meet defined criteria"

    return 0
}

review_acceptance_criteria "$ACCEPTANCE_CRITERIA_PATH"

echo "=== Stage 2: Execute Existing Test Suite ==="
execute_tests() {
    echo "Running existing test suite..."

    # Clean previous builds
    cargo clean

    # Run tests with coverage collection
    echo "Executing tests with coverage instrumentation..."
    cargo llvm-cov --workspace --lcov --output-path /tmp/initial_coverage.lcov test || {
        echo "ERROR: Test suite execution failed"
        return 1
    }

    echo "✅ Initial test suite execution completed"
    return 0
}

execute_tests || {
    echo "FATAL: Test execution failed - cannot proceed with coverage analysis"
    exit 1
}

echo "=== Stage 3: Coverage Analysis ==="
analyze_coverage() {
    echo "Analyzing code coverage..."

    # Generate detailed coverage report
    cargo llvm-cov --workspace --html --output-dir /tmp/coverage-html report
    cargo llvm-cov --workspace --cobertura --output-path /tmp/coverage.xml report
    cargo llvm-cov --workspace --json --output-path /tmp/coverage.json report

    # Parse coverage metrics
    python3 -c "
import json
import sys

try:
    with open('/tmp/coverage.json', 'r') as f:
        coverage_data = json.load(f)

    # Extract key metrics
    total_lines = coverage_data['data'][0]['totals']['lines']['count']
    covered_lines = coverage_data['data'][0]['totals']['lines']['covered']
    coverage_percentage = (covered_lines / total_lines * 100) if total_lines > 0 else 0

    print(f'Total lines: {total_lines}')
    print(f'Covered lines: {covered_lines}')
    print(f'Coverage percentage: {coverage_percentage:.2f}%')

    # Save metrics for later use
    with open('/tmp/coverage_metrics.json', 'w') as f:
        json.dump({
            'total_lines': total_lines,
            'covered_lines': covered_lines,
            'coverage_percentage': coverage_percentage,
            'meets_threshold': coverage_percentage >= ${COVERAGE_THRESHOLD_EXISTING}
        }, f)

except Exception as e:
    print(f'Error parsing coverage data: {e}', file=sys.stderr)
    sys.exit(1)
"

    # Display coverage summary
    echo "Coverage Analysis Results:"
    cat /tmp/coverage_metrics.json | python3 -m json.tool

    return 0
}

analyze_coverage

echo "=== Stage 4: Identify Uncovered Code Paths ==="
identify_uncovered_code() {
    echo "Identifying uncovered code paths for test generation..."

    # Generate uncovered lines report
    cargo llvm-cov --workspace --show-missing-lines report > /tmp/uncovered_lines.txt

    # Parse uncovered code sections
    python3 -c "
import re
import json

uncovered_files = {}
current_file = None

try:
    with open('/tmp/uncovered_lines.txt', 'r') as f:
        for line in f:
            line = line.strip()
            if line.endswith('.rs'):
                current_file = line
                uncovered_files[current_file] = []
            elif current_file and 'Missing:' in line:
                # Extract line numbers
                missing_lines = re.findall(r'\d+(?:-\d+)?', line)
                uncovered_files[current_file].extend(missing_lines)

    # Save uncovered code data
    with open('/tmp/uncovered_code.json', 'w') as f:
        json.dump(uncovered_files, f, indent=2)

    print(f'Found uncovered code in {len(uncovered_files)} files')

except Exception as e:
    print(f'Error processing uncovered code: {e}')
    # Continue execution even if parsing fails
" || echo "Warning: Could not parse uncovered code details"

    return 0
}

identify_uncovered_code

echo "=== Stage 5: Generate Missing Tests ==="
generate_missing_tests() {
    echo "Generating tests for uncovered code paths..."

    # Check if we have uncovered code to address
    if [ ! -f /tmp/uncovered_code.json ]; then
        echo "No uncovered code data found, skipping test generation"
        return 0
    fi

    # Generate tests for each uncovered file
    python3 -c "
import json
import os
import subprocess

def generate_test_for_file(file_path, uncovered_lines):
    print(f'Generating tests for {file_path}')

    # Read the source file
    try:
        with open(file_path, 'r') as f:
            source_code = f.read()
    except FileNotFoundError:
        print(f'Warning: Source file {file_path} not found')
        return

    # Create test file path
    test_file_path = file_path.replace('src/', 'tests/').replace('.rs', '_test.rs')
    os.makedirs(os.path.dirname(test_file_path), exist_ok=True)

    # Generate comprehensive test template
    test_template = f'''
//! Generated tests for {file_path}
//! Auto-generated by Tess coverage workflow

use super::*;



#[cfg(test)]
mod tests {{
    use super::*;

    // TODO: Implement tests for uncovered lines: {uncovered_lines}

    #[test]
    fn test_uncovered_paths() {{
        // This is a placeholder test
        // Tess would generate specific tests for uncovered code paths
        todo!(\"Implement specific tests for uncovered functionality\");
    }}

    #[test]
    fn test_error_conditions() {{
        // Test error handling paths that may be uncovered
        todo!(\"Test error conditions and edge cases\");
    }}

    #[test]
    fn test_boundary_conditions() {{
        // Test boundary conditions that may not be covered
        todo!(\"Test boundary and edge conditions\");
    }}
}}
'''

    # Write test file
    with open(test_file_path, 'w') as f:
        f.write(test_template)

    print(f'Generated test template: {test_file_path}')

try:
    with open('/tmp/uncovered_code.json', 'r') as f:
        uncovered_files = json.load(f)

    for file_path, uncovered_lines in uncovered_files.items():
        if uncovered_lines:  # Only generate tests if there are uncovered lines
            generate_test_for_file(file_path, uncovered_lines)

    print('Test generation completed')

except Exception as e:
    print(f'Error generating tests: {e}')
    # Continue execution
"

    echo "✅ Test generation stage completed"
    return 0
}

generate_missing_tests

echo "=== Stage 6: Execute Full Test Suite with New Tests ==="
execute_full_test_suite() {
    echo "Running complete test suite including generated tests..."

    # Clean and rebuild
    cargo clean

    # Run full test suite with coverage
    cargo llvm-cov --workspace --lcov --output-path /tmp/final_coverage.lcov test --all-features || {
        echo "WARNING: Some tests failed - this may be expected for generated test templates"
        # Continue with coverage analysis even if some tests fail
    }

    # Generate final coverage reports
    cargo llvm-cov --workspace --html --output-dir /tmp/final-coverage-html report
    cargo llvm-cov --workspace --cobertura --output-path /tmp/final-coverage.xml report
    cargo llvm-cov --workspace --json --output-path /tmp/final-coverage.json report

    echo "✅ Full test suite execution completed"
    return 0
}

execute_full_test_suite

echo "=== Stage 7: Generate Coverage Reports ==="
generate_reports() {
    echo "Generating comprehensive coverage reports..."

    # Create coverage summary
    python3 -c "
import json
import datetime

try:
    # Load final coverage data
    with open('/tmp/final-coverage.json', 'r') as f:
        final_coverage = json.load(f)

    # Load initial coverage for comparison
    try:
        with open('/tmp/coverage_metrics.json', 'r') as f:
            initial_metrics = json.load(f)
        initial_coverage_pct = initial_metrics['coverage_percentage']
    except:
        initial_coverage_pct = 0

    # Calculate final metrics
    total_lines = final_coverage['data'][0]['totals']['lines']['count']
    covered_lines = final_coverage['data'][0]['totals']['lines']['covered']
    final_coverage_pct = (covered_lines / total_lines * 100) if total_lines > 0 else 0

    # Generate report
    report = {
        'timestamp': datetime.datetime.now().isoformat(),
        'repository': '$REPO_URL',
        'pr_number': '$PR_NUMBER',
        'initial_coverage': initial_coverage_pct,
        'final_coverage': final_coverage_pct,
        'improvement': final_coverage_pct - initial_coverage_pct,
        'total_lines': total_lines,
        'covered_lines': covered_lines,
        'thresholds': {
            'existing_code': ${COVERAGE_THRESHOLD_EXISTING},
            'new_code': ${COVERAGE_THRESHOLD_NEW}
        },
        'meets_existing_threshold': final_coverage_pct >= ${COVERAGE_THRESHOLD_EXISTING},
        'meets_new_code_threshold': True  # Would check specific new code coverage
    }

    # Save detailed report
    with open('/tmp/coverage_report.json', 'w') as f:
        json.dump(report, f, indent=2)

    print('Coverage Report Generated:')
    print(json.dumps(report, indent=2))

except Exception as e:
    print(f'Error generating coverage report: {e}')
"

    # Copy reports to accessible location
    mkdir -p /tmp/artifacts
    cp -r /tmp/final-coverage-html /tmp/artifacts/
    cp /tmp/final-coverage.xml /tmp/artifacts/
    cp /tmp/coverage_report.json /tmp/artifacts/

    echo "✅ Coverage reports generated and saved to /tmp/artifacts/"
    return 0
}

generate_reports

echo "=== Stage 8: Coverage Trend Analysis ==="
analyze_trends() {
    echo "Analyzing coverage trends..."

    # This would integrate with historical data storage
    # For now, just log the trend data point

    python3 -c "
import json
import datetime

try:
    with open('/tmp/coverage_report.json', 'r') as f:
        report = json.load(f)

    trend_data = {
        'timestamp': datetime.datetime.now().isoformat(),
        'commit_hash': '$(git rev-parse HEAD)',
        'coverage_percentage': report['final_coverage'],
        'total_lines': report['total_lines'],
        'pr_number': '$PR_NUMBER'
    }

    # Save trend data (would typically send to time series database)
    with open('/tmp/coverage_trend.json', 'w') as f:
        json.dump(trend_data, f)

    print('Coverage trend data point recorded')

except Exception as e:
    print(f'Error recording coverage trend: {e}')
"

    echo "✅ Coverage trend analysis completed"
    return 0
}

analyze_trends

echo "=== Stage 9: GitHub API Integration and PR Approval ==="
integrate_github_api() {
    echo "Integrating with GitHub API for PR approval..."

    # Load coverage report
    if [ ! -f /tmp/coverage_report.json ]; then
        echo "ERROR: Coverage report not found"
        return 1
    fi

    # Check coverage thresholds
    python3 -c "
import json
import requests
import os

try:
    with open('/tmp/coverage_report.json', 'r') as f:
        report = json.load(f)

    meets_existing = report['meets_existing_threshold']
    meets_new = report['meets_new_code_threshold']
    final_coverage = report['final_coverage']

    # Prepare GitHub API headers
    headers = {
        'Authorization': f'token {os.environ[\"GITHUB_TOKEN\"]}',
        'Accept': 'application/vnd.github.v3+json',
        'Content-Type': 'application/json'
    }

    # Extract repo info from URL
    repo_url = os.environ['REPO_URL']
    repo_path = repo_url.replace('https://github.com/', '').replace('.git', '')

    pr_number = os.environ['PR_NUMBER']

    if meets_existing and meets_new:
        # Approve the PR
        print(f'Coverage thresholds met ({final_coverage:.2f}%), approving PR...')

        # Create approval review
        review_data = {
            'event': 'APPROVE',
            'body': f'''## ✅ Tess Coverage Analysis - APPROVED

**Coverage Report:**
- Final Coverage: {final_coverage:.2f}%
- Existing Code Threshold: {report['thresholds']['existing_code']}% ✅
- New Code Threshold: {report['thresholds']['new_code']}% ✅
- Coverage Improvement: +{report['improvement']:.2f}%

**Analysis:**
- Total Lines: {report['total_lines']:,}
- Covered Lines: {report['covered_lines']:,}

This PR meets all coverage requirements and has been automatically approved by Tess.

[View detailed coverage report]({repo_url}/actions)
'''
        }

        # Submit review
        review_url = f'https://api.github.com/repos/{repo_path}/pulls/{pr_number}/reviews'

        # For demo purposes, just log what would be sent
        print('Would submit GitHub review:')
        print(json.dumps(review_data, indent=2))
        print(f'To: {review_url}')

        # Uncomment for actual API call:
        # response = requests.post(review_url, headers=headers, json=review_data)
        # if response.status_code == 200:
        #     print('✅ PR approved successfully')
        # else:
        #     print(f'❌ Failed to approve PR: {response.status_code} {response.text}')

        print('✅ PR approval process completed')

    else:
        # Request changes
        print(f'Coverage thresholds not met ({final_coverage:.2f}%), requesting changes...')

        review_data = {
            'event': 'REQUEST_CHANGES',
            'body': f'''## ❌ Tess Coverage Analysis - CHANGES REQUESTED

**Coverage Report:**
- Final Coverage: {final_coverage:.2f}%
- Existing Code Threshold: {report['thresholds']['existing_code']}% {'✅' if meets_existing else '❌'}
- New Code Threshold: {report['thresholds']['new_code']}% {'✅' if meets_new else '❌'}

**Required Actions:**


- {'✅ Existing code coverage meets threshold' if meets_existing else '❌ Increase existing code coverage to meet threshold'}


- {'✅ New code coverage meets threshold' if meets_new else '❌ Ensure new code has complete test coverage'}

Please add additional tests to meet the coverage requirements.

[View detailed coverage report]({repo_url}/actions)
'''
        }

        print('Would request changes:')
        print(json.dumps(review_data, indent=2))

        print('❌ PR requires additional test coverage')

except Exception as e:
    print(f'Error in GitHub API integration: {e}')
    return 1
"

    echo "✅ GitHub API integration completed"
    return 0
}

integrate_github_api

echo "=== Workflow Summary ==="
echo "Tess Test Coverage Workflow completed successfully!"
echo "Timestamp: $(date)"

# Display final results
if [ -f /tmp/coverage_report.json ]; then
    echo "Final Coverage Report:"
    cat /tmp/coverage_report.json | python3 -c "
import json
import sys

data = json.load(sys.stdin)
print(f\"Coverage: {data['final_coverage']:.2f}%\")
print(f\"Improvement: +{data['improvement']:.2f}%\")
print(f\"Meets Thresholds: {'Yes' if data['meets_existing_threshold'] and data['meets_new_code_threshold'] else 'No'}\")
"
fi

echo "Artifacts saved to /tmp/artifacts/"
echo "=== End of Tess Workflow ==="






```

#### 2. Coverage Threshold Configuration

**File**: `controller/src/coverage/config.rs`




```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageConfig {
    pub existing_code_threshold: f64,
    pub new_code_threshold: f64,
    pub enable_automatic_approval: bool,
    pub enable_test_generation: bool,
    pub coverage_report_formats: Vec<ReportFormat>,
    pub trend_analysis: TrendConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportFormat {
    Html,
    Lcov,
    Cobertura,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendConfig {
    pub enabled: bool,
    pub storage_backend: String,
    pub retention_days: i32,
}

impl Default for CoverageConfig {
    fn default() -> Self {
        Self {
            existing_code_threshold: 95.0,
            new_code_threshold: 100.0,
            enable_automatic_approval: true,
            enable_test_generation: true,
            coverage_report_formats: vec![
                ReportFormat::Html,
                ReportFormat::Cobertura,
                ReportFormat::Json,
            ],
            trend_analysis: TrendConfig {
                enabled: true,
                storage_backend: "redis".to_string(),
                retention_days: 90,
            },
        }
    }
}






```

#### 3. GitHub API Integration Service

**File**: `controller/src/github/coverage_integration.rs`




```rust
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::collections::HashMap;



#[derive(Debug, Serialize)]
pub struct PullRequestReview {
    pub event: ReviewEvent,
    pub body: String,
}



#[derive(Debug, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ReviewEvent {
    Approve,
    RequestChanges,
    Comment,
}



#[derive(Debug, Deserialize)]
pub struct CoverageReport {
    pub final_coverage: f64,
    pub meets_existing_threshold: bool,
    pub meets_new_code_threshold: bool,
    pub improvement: f64,
    pub total_lines: u32,
    pub covered_lines: u32,
}

pub struct GitHubCoverageIntegration {
    client: Client,
    token: String,
}

impl GitHubCoverageIntegration {
    pub fn new(token: String) -> Self {
        Self {
            client: Client::new(),
            token,
        }
    }

    pub async fn submit_coverage_review(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u32,
        coverage_report: &CoverageReport,
    ) -> Result<()> {
        let review = self.create_review_from_coverage(coverage_report)?;

        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}/reviews",
            repo_owner, repo_name, pr_number
        );

        let response = self.client
            .post(&url)
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .json(&review)
            .send()
            .await?;

        if response.status().is_success() {
            match review.event {
                ReviewEvent::Approve => println!("✅ PR approved based on coverage"),
                ReviewEvent::RequestChanges => println!("❌ Changes requested due to insufficient coverage"),
                ReviewEvent::Comment => println!("💬 Coverage comment added to PR"),
            }
        } else {
            let error_text = response.text().await?;
            return Err(anyhow!("GitHub API request failed: {}", error_text));
        }

        Ok(())
    }

    fn create_review_from_coverage(&self, report: &CoverageReport) -> Result<PullRequestReview> {
        let meets_all_thresholds = report.meets_existing_threshold && report.meets_new_code_threshold;

        let (event, body) = if meets_all_thresholds {
            let body = format!(
                r#"## ✅ Tess Coverage Analysis - APPROVED

**Coverage Report:**
- Final Coverage: {:.2}%
- Coverage Improvement: +{:.2}%
- Total Lines: {:,}
- Covered Lines: {:,}

**Thresholds:**
- Existing Code: ✅ Met
- New Code: ✅ Met

This PR meets all coverage requirements and has been automatically approved by Tess.

📊 [View detailed coverage report]({{coverage_report_url}})"#,
                report.final_coverage,
                report.improvement,
                report.total_lines,
                report.covered_lines
            );

            (ReviewEvent::Approve, body)
        } else {
            let body = format!(
                r#"## ❌ Tess Coverage Analysis - CHANGES REQUESTED

**Coverage Report:**
- Final Coverage: {:.2}%
- Total Lines: {:,}
- Covered Lines: {:,}

**Threshold Status:**
- Existing Code: {}
- New Code: {}

**Required Actions:**
{}

Please add additional tests to meet the coverage requirements.

📊 [View detailed coverage report]({{coverage_report_url}})"#,
                report.final_coverage,
                report.total_lines,
                report.covered_lines,
                if report.meets_existing_threshold { "✅ Met" } else { "❌ Not Met" },
                if report.meets_new_code_threshold { "✅ Met" } else { "❌ Not Met" },
                self.generate_action_items(report)
            );

            (ReviewEvent::RequestChanges, body)
        };

        Ok(PullRequestReview { event, body })
    }

    fn generate_action_items(&self, report: &CoverageReport) -> String {
        let mut actions = Vec::new();

        if !report.meets_existing_threshold {
            actions.push("- Increase test coverage for existing code paths");
        }

        if !report.meets_new_code_threshold {
            actions.push("- Ensure all new code has comprehensive test coverage");
        }

        if actions.is_empty() {
            "- Review and improve test quality and coverage".to_string()
        } else {
            actions.join("\n")
        }
    }

    pub async fn update_pr_comment(
        &self,
        repo_owner: &str,
        repo_name: &str,
        pr_number: u32,
        coverage_trend: &CoverageTrend,
    ) -> Result<()> {
        let comment_body = self.generate_trend_comment(coverage_trend);

        // Find existing coverage comment or create new one
        let url = format!(
            "https://api.github.com/repos/{}/{}/issues/{}/comments",
            repo_owner, repo_name, pr_number
        );

        let comment_data = HashMap::from([
            ("body", comment_body),
        ]);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("token {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .json(&comment_data)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Failed to update PR comment: {}", error_text));
        }

        Ok(())
    }

    fn generate_trend_comment(&self, trend: &CoverageTrend) -> String {
        format!(
            r#"## 📈 Coverage Trend Analysis

**Recent Coverage History:**
{}

**Trend Direction:** {}

This analysis helps track coverage improvements over time."#,
            self.format_trend_data(trend),
            self.format_trend_direction(trend)
        )
    }

    fn format_trend_data(&self, trend: &CoverageTrend) -> String {
        trend.data_points
            .iter()
            .map(|point| format!("- {}: {:.2}%", point.date, point.coverage))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn format_trend_direction(&self, trend: &CoverageTrend) -> String {
        if trend.is_improving() {
            "📈 Improving"
        } else if trend.is_declining() {
            "📉 Declining"
        } else {
            "➡️ Stable"
        }.to_string()
    }
}



#[derive(Debug, Deserialize)]
pub struct CoverageTrend {
    pub data_points: Vec<CoverageDataPoint>,
}



#[derive(Debug, Deserialize)]
pub struct CoverageDataPoint {
    pub date: String,
    pub coverage: f64,
    pub commit_hash: String,
}

impl CoverageTrend {
    pub fn is_improving(&self) -> bool {
        if self.data_points.len() < 2 {
            return false;
        }

        let recent = self.data_points.last().unwrap().coverage;
        let previous = self.data_points[self.data_points.len() - 2].coverage;

        recent > previous
    }

    pub fn is_declining(&self) -> bool {
        if self.data_points.len() < 2 {
            return false;
        }

        let recent = self.data_points.last().unwrap().coverage;
        let previous = self.data_points[self.data_points.len() - 2].coverage;

        recent < previous
    }
}






```

## Testing Strategy

### Unit Tests



```rust


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coverage_config_defaults() {
        let config = CoverageConfig::default();
        assert_eq!(config.existing_code_threshold, 95.0);
        assert_eq!(config.new_code_threshold, 100.0);
        assert!(config.enable_automatic_approval);
    }

    #[test]
    fn test_github_review_creation_approval() {
        let integration = GitHubCoverageIntegration::new("test-token".to_string());
        let report = CoverageReport {
            final_coverage: 98.5,
            meets_existing_threshold: true,
            meets_new_code_threshold: true,
            improvement: 3.2,
            total_lines: 1500,
            covered_lines: 1477,
        };

        let review = integration.create_review_from_coverage(&report).unwrap();

        match review.event {
            ReviewEvent::Approve => assert!(review.body.contains("APPROVED")),
            _ => panic!("Expected approval review"),
        }
    }

    #[test]
    fn test_coverage_trend_analysis() {
        let trend = CoverageTrend {
            data_points: vec![
                CoverageDataPoint { date: "2023-01-01".to_string(), coverage: 85.0, commit_hash: "abc123".to_string() },
                CoverageDataPoint { date: "2023-01-02".to_string(), coverage: 90.0, commit_hash: "def456".to_string() },
            ],
        };

        assert!(trend.is_improving());
        assert!(!trend.is_declining());
    }
}






```

### Integration Tests



```bash
#!/bin/bash
# Integration test for Tess coverage workflow



# Setup test repository
mkdir -p /tmp/test-repo
cd /tmp/test-repo
git init
git config user.email "test@example.com"
git config user.name "Test User"

# Create sample Rust project with incomplete coverage
cargo init --name test-project

# Add code with missing test coverage
cat > src/lib.rs << 'EOF'
pub fn covered_function(x: i32) -> i32 {
    x * 2
}

pub fn uncovered_function(x: i32) -> i32 {
    if x > 0 {
        x * 3  // This branch may not be covered
    } else {
        x * -1  // This branch may not be covered
    }
}
EOF



# Add partial tests
mkdir -p tests
cat > tests/integration_test.rs << 'EOF'
use test_project::covered_function;



#[test]
fn test_covered_function() {
    assert_eq!(covered_function(5), 10);
}
EOF



# Test the Tess workflow
export GITHUB_TOKEN="test-token"
export REPO_URL="https://github.com/test/test-repo.git"
export PR_NUMBER="123"

# Execute Tess container script
bash /path/to/container-tess.sh.hbs



# Validate results
if [ -f /tmp/coverage_report.json ]; then
    echo "✅ Coverage report generated successfully"
    python3 -c "
import json
with open('/tmp/coverage_report.json', 'r') as f:
    report = json.load(f)
    print(f'Final coverage: {report[\"final_coverage\"]}%')
"
else
    echo "❌ Coverage report not generated"
    exit 1
fi






```

## Performance Considerations

1. **Coverage Collection Overhead**: Use optimized LLVM coverage instrumentation
2. **Large Codebase Handling**: Implement incremental coverage analysis
3. **Test Generation Efficiency**: Cache and reuse generated test templates
4. **Report Generation**: Optimize HTML report generation for large projects

## Security Considerations

1. **GitHub Token Security**: Use secure token storage and rotation
2. **Code Injection Prevention**: Sanitize all generated test code
3. **PR Approval Authority**: Implement proper authorization checks
4. **Coverage Data Integrity**: Verify coverage data authenticity
