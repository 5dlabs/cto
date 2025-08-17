# Toolman Guide: Agent-Specific Container Scripts for Multi-Agent Workflows

## Overview

This guide provides comprehensive instructions for developing, testing, and deploying agent-specific container scripts that implement distinct workflows for Rex, Cleo, and Tess agents. The scripts enable specialized functionality while maintaining integration with the existing Task Master infrastructure.

## Tool Categories

### 1. Development and Scripting Tools

#### Bash Shell (`bash_shell`)

**Purpose**: Core script execution environment with advanced error handling

**Script Development Best Practices**:
```bash
#!/bin/bash
# Agent-specific container script template
# Use strict error handling
set -euo pipefail

# Global configuration
AGENT_TYPE="${AGENT_TYPE:-unknown}"
WORKSPACE_PATH="${WORKSPACE_PATH:-/workspace}"
LOG_FILE="${WORKSPACE_PATH}/${AGENT_TYPE}.log"

# Logging functions
log() {
    echo "[$(date -Iseconds)] [$AGENT_TYPE] $*" | tee -a "$LOG_FILE"
}

error() {
    echo "[$(date -Iseconds)] [$AGENT_TYPE] ERROR: $*" | tee -a "$LOG_FILE" >&2
}

debug() {
    if [[ "${DEBUG:-false}" == "true" ]]; then
        echo "[$(date -Iseconds)] [$AGENT_TYPE] DEBUG: $*" | tee -a "$LOG_FILE"
    fi
}

# Error handling with cleanup
cleanup() {
    local exit_code=$?
    if [[ $exit_code -ne 0 ]]; then
        error "Script failed with exit code $exit_code at line ${BASH_LINENO[1]}"
        # Perform cleanup operations
        cleanup_workspace
    fi
    exit $exit_code
}

trap cleanup EXIT ERR

# Template variable validation
validate_template_vars() {
    local required_vars=("GITHUB_APP" "TASK_ID" "WORKSPACE_PATH")
    
    for var in "${required_vars[@]}"; do
        if [[ -z "${!var:-}" ]]; then
            error "Required template variable $var is not set"
            return 1
        fi
    done
}

# Retry logic for network operations
retry_command() {
    local max_retries=3
    local delay=2
    local command="$1"
    
    for ((i=1; i<=max_retries; i++)); do
        if eval "$command"; then
            return 0
        else
            if [[ $i -eq $max_retries ]]; then
                error "Command failed after $max_retries attempts: $command"
                return 1
            fi
            log "Attempt $i failed, retrying in ${delay}s..."
            sleep "$delay"
            delay=$((delay * 2))
        fi
    done
}

# Main workflow template
main_workflow() {
    log "Starting ${AGENT_TYPE} agent workflow"
    
    validate_template_vars
    initialize_environment
    execute_agent_workflow
    finalize_results
    
    log "${AGENT_TYPE} agent workflow completed successfully"
}

# Execute main workflow
main_workflow "$@"
```

**Advanced Error Handling**:
```bash
# Comprehensive error handling for agent scripts
handle_github_api_error() {
    local http_code="$1"
    local response="$2"
    
    case "$http_code" in
        403)
            if echo "$response" | jq -e '.message | contains("rate limit")' > /dev/null 2>&1; then
                error "GitHub API rate limit exceeded"
                local reset_time=$(echo "$response" | jq -r '.reset_time // "unknown"')
                error "Rate limit resets at: $reset_time"
                return 2  # Rate limit error
            else
                error "GitHub API forbidden: $(echo "$response" | jq -r '.message // "unknown"')"
                return 1  # Permission error
            fi
            ;;
        404)
            error "GitHub resource not found: $(echo "$response" | jq -r '.message // "unknown"')"
            return 1
            ;;
        422)
            error "GitHub API validation error: $(echo "$response" | jq -r '.message // "unknown"')"
            return 1
            ;;
        *)
            error "GitHub API error (HTTP $http_code): $(echo "$response" | jq -r '.message // "unknown"')"
            return 1
            ;;
    esac
}
```

### 2. Rex Agent Tools

#### MCP Documentation Server (`mcp_documentation_server`)

**Purpose**: Documentation queries and context retrieval for implementation guidance

**MCP Integration Functions**:
```bash
# MCP server integration for Rex agent
MCP_SERVER_URL="${MCP_SERVER_URL:-http://mcp-server:8080}"
MCP_TIMEOUT="${MCP_TIMEOUT:-30}"

check_mcp_connectivity() {
    log "Checking MCP server connectivity"
    
    if curl -f --max-time "$MCP_TIMEOUT" "$MCP_SERVER_URL/health" > /dev/null 2>&1; then
        log "MCP server is accessible"
        return 0
    else
        error "MCP server not accessible at $MCP_SERVER_URL"
        return 1
    fi
}

query_mcp_documentation() {
    local query="$1"
    local output_file="$2"
    
    log "Querying MCP documentation: $query"
    
    local request_payload=$(jq -n \
        --arg query "$query" \
        --argjson include_examples true \
        '{query: $query, include_examples: $include_examples}')
    
    local response=$(curl -s -X POST "$MCP_SERVER_URL/api/v1/query" \
        -H "Content-Type: application/json" \
        -d "$request_payload" \
        --max-time "$MCP_TIMEOUT")
    
    if [[ $? -eq 0 ]] && echo "$response" | jq -e '.content' > /dev/null 2>&1; then
        echo "$response" | jq -r '.content' >> "$output_file"
        
        # Extract code examples if available
        if echo "$response" | jq -e '.examples' > /dev/null 2>&1; then
            echo "$response" | jq -r '.examples[]' >> "${WORKSPACE_PATH}/code_examples.rs"
        fi
        
        log "Documentation retrieved and saved to $output_file"
        return 0
    else
        error "Failed to query MCP documentation: $response"
        return 1
    fi
}

analyze_task_requirements() {
    local task_file="$1"
    local context_file="$2"
    
    log "Analyzing task requirements from $task_file"
    
    # Extract key technologies and concepts
    local technologies=$(grep -i -E "(rust|cargo|kubernetes|docker|api|http|json|yaml)" "$task_file" | head -10)
    local patterns=$(grep -i -E "(pattern|architecture|design|structure)" "$task_file" | head -10)
    
    # Query documentation for each technology
    while IFS= read -r tech_line; do
        if [[ -n "$tech_line" ]]; then
            local tech=$(echo "$tech_line" | grep -oiE "\b(rust|cargo|kubernetes|docker|api)\b" | head -1)
            if [[ -n "$tech" ]]; then
                retry_command "query_mcp_documentation 'best practices for ${tech,,}' '$context_file'"
                sleep 1  # Rate limiting
            fi
        fi
    done <<< "$technologies"
    
    # Query architectural patterns
    retry_command "query_mcp_documentation 'architecture patterns and design principles' '$context_file'"
    retry_command "query_mcp_documentation 'error handling patterns in Rust' '$context_file'"
    retry_command "query_mcp_documentation 'testing strategies and best practices' '$context_file'"
}

generate_implementation_plan() {
    local task_file="$1"
    local context_file="$2"
    local plan_file="$3"
    
    log "Generating implementation plan based on documentation"
    
    cat > "$plan_file" <<EOF
# Implementation Plan for Task ${TASK_ID}
Generated by Rex Agent at $(date -Iseconds)

## Task Overview
$(head -20 "$task_file")

## Documentation Context
Based on MCP server queries, the following patterns and practices should be followed:

$(head -50 "$context_file")

## Implementation Approach
1. **Architecture Design**
   - Follow documented patterns from MCP server
   - Implement proper error handling as per Rust guidelines
   - Use established API design principles

2. **Code Structure**
   - Organize modules according to documented conventions
   - Implement comprehensive inline documentation
   - Follow documented naming conventions

3. **Testing Strategy**
   - Unit tests following documented patterns
   - Integration tests for API endpoints
   - Property-based testing where appropriate

## Implementation Steps
$(generate_step_by_step_approach)

## Quality Assurance
- Code review against documentation standards
- Automated testing with comprehensive coverage
- Performance validation against benchmarks
EOF
    
    log "Implementation plan generated: $plan_file"
}

generate_step_by_step_approach() {
    echo "1. Set up project structure following Rust conventions"
    echo "2. Implement core functionality with comprehensive documentation"
    echo "3. Add error handling based on documented patterns"
    echo "4. Create comprehensive test suite"
    echo "5. Generate API documentation with examples"
    echo "6. Validate implementation against requirements"
    echo "7. Performance testing and optimization"
    echo "8. Final documentation review and updates"
}
```

### 3. Cleo Agent Tools

#### Rust Formatting Tools (`rustfmt`, `clippy`)

**Purpose**: Code quality enforcement and automated formatting

**Formatting Pipeline Implementation**:
```bash
# Comprehensive formatting and quality pipeline for Cleo
RUSTFMT_CONFIG="${WORKSPACE_PATH}/rustfmt.toml"
CLIPPY_CONFIG="${WORKSPACE_PATH}/clippy.toml"

setup_formatting_tools() {
    log "Setting up formatting and linting tools"
    
    # Ensure rustfmt is available
    if ! rustfmt --version > /dev/null 2>&1; then
        log "Installing rustfmt component"
        rustup component add rustfmt
    fi
    
    # Ensure clippy is available
    if ! cargo clippy --version > /dev/null 2>&1; then
        log "Installing clippy component"
        rustup component add clippy
    fi
    
    # Create rustfmt configuration if not exists
    if [[ ! -f "$RUSTFMT_CONFIG" ]]; then
        create_rustfmt_config
    fi
}

create_rustfmt_config() {
    cat > "$RUSTFMT_CONFIG" <<EOF
# Rustfmt configuration for TaskMaster projects
max_width = 100
hard_tabs = false
tab_spaces = 4
newline_style = "Unix"
use_small_heuristics = "Default"
indent_style = "Block"
wrap_comments = true
comment_width = 80
normalize_comments = true
normalize_doc_attributes = true
format_strings = true
format_macro_matchers = true
format_code_in_doc_comments = true
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
EOF
    
    log "Created rustfmt configuration: $RUSTFMT_CONFIG"
}

check_formatting_status() {
    local status_file="$1"
    
    log "Checking code formatting status"
    
    if cargo fmt -- --check > /tmp/fmt_check.log 2>&1; then
        log "Code is already properly formatted"
        echo '{"status": "formatted", "issues": []}' > "$status_file"
        return 0
    else
        log "Formatting issues detected"
        
        # Parse formatting issues from output
        local issues=()
        while IFS= read -r line; do
            if [[ "$line" =~ ^Diff\ in\ (.*)\ at\ line ]]; then
                local file=$(echo "$line" | sed 's/^Diff in \(.*\) at line.*/\1/')
                issues+=("\"$file\"")
            fi
        done < /tmp/fmt_check.log
        
        local issues_json=$(printf '%s\n' "${issues[@]}" | jq -R . | jq -s .)
        echo "{\"status\": \"needs_formatting\", \"issues\": $issues_json}" > "$status_file"
        
        return 1
    fi
}

apply_formatting() {
    local report_file="$1"
    
    log "Applying code formatting"
    
    # Capture files before formatting
    local files_before=$(find src -name "*.rs" -exec md5sum {} \; | sort)
    
    # Apply formatting
    if cargo fmt; then
        log "Formatting applied successfully"
        
        # Capture files after formatting
        local files_after=$(find src -name "*.rs" -exec md5sum {} \; | sort)
        
        # Generate diff report
        if [[ "$files_before" != "$files_after" ]]; then
            generate_formatting_report "$report_file"
        else
            log "No formatting changes were needed"
        fi
        
        return 0
    else
        error "Failed to apply formatting"
        return 1
    fi
}

run_clippy_analysis() {
    local report_file="$1"
    local json_file="$2"
    
    log "Running clippy analysis"
    
    # Run clippy with JSON output
    local clippy_output
    if clippy_output=$(cargo clippy --message-format=json --all-targets --all-features -- -D warnings 2>&1); then
        log "Clippy analysis completed with no issues"
        echo '{"status": "clean", "warnings": [], "errors": []}' > "$json_file"
        return 0
    else
        log "Clippy found issues"
        
        # Parse and categorize clippy output
        echo "$clippy_output" | jq -s '
        {
            "status": "issues_found",
            "timestamp": now | strftime("%Y-%m-%dT%H:%M:%SZ"),
            "warnings": [.[] | select(.reason == "compiler-message" and .message.level == "warning")],
            "errors": [.[] | select(.reason == "compiler-message" and .message.level == "error")],
            "suggestions": [.[] | select(.reason == "compiler-message" and (.message.spans[]?.suggestion_applicability // "") == "MachineApplicable")]
        }' > "$json_file"
        
        generate_clippy_report "$clippy_output" "$report_file"
        return 1
    fi
}

apply_clippy_fixes() {
    log "Applying automatic clippy fixes"
    
    # Apply fixes that can be automatically applied
    if cargo clippy --fix --allow-dirty --allow-staged --all-targets --all-features -- -D warnings; then
        log "Automatic clippy fixes applied"
        
        # Show what was changed
        if [[ -n "$(git diff --name-only)" ]]; then
            log "Files modified by clippy fixes:"
            git diff --name-only | while read -r file; do
                log "  - $file"
            done
        fi
        
        return 0
    else
        log "Some clippy issues require manual intervention"
        return 1
    fi
}

organize_imports() {
    log "Organizing imports and cleaning dead code"
    
    # Use cargo-sort if available, otherwise basic organization
    if command -v cargo-sort > /dev/null 2>&1; then
        cargo sort
    else
        # Basic import organization using cargo fmt with import settings
        cargo fmt -- --config imports_granularity=Crate,group_imports=StdExternalCrate
    fi
    
    # Remove unused imports
    cargo +nightly fix --edition-idioms 2>/dev/null || true
}

generate_quality_summary() {
    local summary_file="$1"
    
    log "Generating quality summary"
    
    # Count various metrics
    local loc=$(find src -name "*.rs" -exec wc -l {} + | tail -1 | awk '{print $1}' || echo "0")
    local files=$(find src -name "*.rs" | wc -l || echo "0")
    local todo_count=$(find src -name "*.rs" -exec grep -c "TODO\|FIXME\|XXX" {} + 2>/dev/null | awk '{sum += $1} END {print sum+0}')
    local panic_count=$(find src -name "*.rs" -exec grep -c "panic!\|unwrap()" {} + 2>/dev/null | awk '{sum += $1} END {print sum+0}')
    
    cat > "$summary_file" <<EOF
{
    "timestamp": "$(date -Iseconds)",
    "agent": "cleo",
    "task_id": "${TASK_ID}",
    "metrics": {
        "lines_of_code": $loc,
        "rust_files": $files,
        "todo_items": $todo_count,
        "potential_panics": $panic_count
    },
    "changes_applied": {
        "formatting": true,
        "clippy_fixes": true,
        "import_organization": true,
        "dead_code_removal": true
    },
    "quality_score": $(calculate_quality_score "$loc" "$todo_count" "$panic_count")
}
EOF
}

calculate_quality_score() {
    local loc="$1"
    local todos="$2"
    local panics="$3"
    
    # Simple quality scoring algorithm
    local base_score=100
    local todo_penalty=$((todos * 2))
    local panic_penalty=$((panics * 5))
    
    local score=$((base_score - todo_penalty - panic_penalty))
    
    if [[ $score -lt 0 ]]; then
        score=0
    fi
    
    echo "$score"
}
```

### 4. Tess Agent Tools

#### Testing and Coverage Tools (`cargo`, `llvm_cov`, `tarpaulin`, `nextest`)

**Purpose**: Comprehensive testing and coverage analysis

**Testing Pipeline Implementation**:
```bash
# Comprehensive testing pipeline for Tess agent
COVERAGE_THRESHOLD="${COVERAGE_THRESHOLD:-95}"
TEST_TIMEOUT="${TEST_TIMEOUT:-600}"  # 10 minutes

setup_testing_tools() {
    log "Setting up testing and coverage tools"
    
    # Install cargo-llvm-cov if not available
    if ! cargo llvm-cov --version > /dev/null 2>&1; then
        log "Installing cargo-llvm-cov"
        cargo install cargo-llvm-cov
    fi
    
    # Install cargo-nextest for faster testing
    if ! cargo nextest --version > /dev/null 2>&1; then
        log "Installing cargo-nextest"
        cargo install cargo-nextest --locked
    fi
    
    # Install cargo-audit for security scanning
    if ! cargo audit --version > /dev/null 2>&1; then
        log "Installing cargo-audit"
        cargo install cargo-audit
    fi
    
    # Install tarpaulin as fallback coverage tool
    if ! cargo tarpaulin --version > /dev/null 2>&1; then
        log "Installing cargo-tarpaulin"
        cargo install cargo-tarpaulin
    fi
}

run_unit_tests() {
    local results_dir="$1"
    local log_file="$results_dir/unit_tests.log"
    
    log "Running unit tests with cargo nextest"
    
    # Use nextest if available, otherwise regular cargo test
    if command -v cargo-nextest > /dev/null 2>&1; then
        timeout "$TEST_TIMEOUT" cargo nextest run --all-features --no-fail-fast \
            --message-format json > "$log_file" 2>&1
    else
        timeout "$TEST_TIMEOUT" cargo test --all-features --no-fail-fast \
            --message-format json > "$log_file" 2>&1
    fi
    
    local exit_code=$?
    
    if [[ $exit_code -eq 0 ]]; then
        log "Unit tests passed"
        echo "PASSED" > "$results_dir/unit_test_status"
        return 0
    else
        error "Unit tests failed (exit code: $exit_code)"
        echo "FAILED" > "$results_dir/unit_test_status"
        
        # Extract failed test information
        extract_test_failures "$log_file" "$results_dir/failed_tests.json"
        return 1
    fi
}

extract_test_failures() {
    local log_file="$1"
    local output_file="$2"
    
    # Parse JSON test output to extract failures
    grep '^{' "$log_file" | jq -s '
        [.[] | select(.type == "test" and .event == "failed") | {
            name: .name,
            stdout: .stdout,
            message: .message
        }]' > "$output_file"
}

run_integration_tests() {
    local results_dir="$1"
    local log_file="$results_dir/integration_tests.log"
    
    log "Running integration tests"
    
    # Find integration test files
    local integration_tests=$(find . -path "*/tests/*.rs" -type f | wc -l)
    
    if [[ $integration_tests -eq 0 ]]; then
        log "No integration tests found"
        echo "NONE" > "$results_dir/integration_test_status"
        return 0
    fi
    
    if timeout "$TEST_TIMEOUT" cargo test --test '*' --message-format json > "$log_file" 2>&1; then
        log "Integration tests passed"
        echo "PASSED" > "$results_dir/integration_test_status"
        return 0
    else
        error "Integration tests failed"
        echo "FAILED" > "$results_dir/integration_test_status"
        return 1
    fi
}

generate_coverage_report() {
    local coverage_dir="$1"
    local use_tarpaulin="${2:-false}"
    
    log "Generating coverage report"
    
    mkdir -p "$coverage_dir"
    
    if [[ "$use_tarpaulin" == "true" ]] || ! command -v cargo-llvm-cov > /dev/null 2>&1; then
        generate_tarpaulin_coverage "$coverage_dir"
    else
        generate_llvm_coverage "$coverage_dir"
    fi
}

generate_llvm_coverage() {
    local coverage_dir="$1"
    
    log "Using cargo-llvm-cov for coverage analysis"
    
    # Generate LCOV report
    if cargo llvm-cov --all-features --workspace --lcov --output-path "$coverage_dir/lcov.info"; then
        log "LCOV coverage report generated"
        
        # Generate HTML report if genhtml is available
        if command -v genhtml > /dev/null 2>&1; then
            genhtml "$coverage_dir/lcov.info" \
                -o "$coverage_dir/html" \
                --title "Task ${TASK_ID} Coverage Report" \
                --show-details \
                --legend
            log "HTML coverage report generated"
        fi
        
        # Generate JSON summary
        cargo llvm-cov --all-features --workspace --json \
            --output-path "$coverage_dir/coverage.json"
        
        # Extract coverage percentage
        local coverage_percent=$(jq -r '.data[0].totals.lines.percent' "$coverage_dir/coverage.json" 2>/dev/null || echo "0")
        echo "$coverage_percent" > "$coverage_dir/coverage_percent.txt"
        
        log "Coverage: ${coverage_percent}%"
        return 0
    else
        error "Failed to generate coverage with cargo-llvm-cov"
        return 1
    fi
}

generate_tarpaulin_coverage() {
    local coverage_dir="$1"
    
    log "Using cargo-tarpaulin for coverage analysis"
    
    if cargo tarpaulin --all-features --workspace --timeout "$TEST_TIMEOUT" \
        --out xml --out html --output-dir "$coverage_dir"; then
        
        # Convert Tarpaulin XML to LCOV format
        if command -v python3 > /dev/null 2>&1; then
            python3 -c "
import xml.etree.ElementTree as ET
import sys

tree = ET.parse('$coverage_dir/cobertura.xml')
root = tree.getroot()

for package in root.findall('.//package'):
    for class_elem in package.findall('.//class'):
        filename = class_elem.get('filename')
        print(f'SF:{filename}')
        
        for line in class_elem.findall('.//line'):
            line_num = line.get('number')
            hits = line.get('hits', '0')
            print(f'DA:{line_num},{hits}')
        
        print('end_of_record')
" > "$coverage_dir/lcov.info" 2>/dev/null || true
        fi
        
        return 0
    else
        error "Failed to generate coverage with cargo-tarpaulin"
        return 1
    fi
}

validate_coverage_threshold() {
    local coverage_dir="$1"
    local coverage_file="$coverage_dir/coverage_percent.txt"
    
    if [[ ! -f "$coverage_file" ]]; then
        error "Coverage percentage file not found"
        return 1
    fi
    
    local coverage_percent=$(cat "$coverage_file")
    
    log "Validating coverage: ${coverage_percent}% vs ${COVERAGE_THRESHOLD}%"
    
    if (( $(echo "$coverage_percent >= $COVERAGE_THRESHOLD" | bc -l) )); then
        log "✓ Coverage threshold met"
        echo "PASSED" > "$coverage_dir/threshold_status"
        return 0
    else
        error "✗ Coverage below threshold: ${coverage_percent}% < ${COVERAGE_THRESHOLD}%"
        echo "FAILED" > "$coverage_dir/threshold_status"
        return 1
    fi
}

run_security_audit() {
    local audit_log="$1"
    
    log "Running security audit"
    
    if cargo audit --json > "$audit_log" 2>&1; then
        log "Security audit passed"
        echo "PASSED" > "${audit_log}.status"
        return 0
    else
        local vulnerabilities=$(jq -r '.vulnerabilities.count' "$audit_log" 2>/dev/null || echo "unknown")
        error "Security vulnerabilities found: $vulnerabilities"
        echo "FAILED" > "${audit_log}.status"
        return 1
    fi
}

validate_deployment_readiness() {
    local results_dir="$1"
    
    log "Validating deployment readiness"
    
    # Test release build
    if cargo build --release --all-features > "$results_dir/release_build.log" 2>&1; then
        log "✓ Release build successful"
        echo "PASSED" > "$results_dir/release_build_status"
    else
        error "✗ Release build failed"
        echo "FAILED" > "$results_dir/release_build_status"
        return 1
    fi
    
    # Run benchmarks if available
    if [[ -d "benches" ]] || grep -q "\[\[bench\]\]" Cargo.toml 2>/dev/null; then
        log "Running benchmarks"
        if cargo bench > "$results_dir/benchmark_results.txt" 2>&1; then
            log "✓ Benchmarks completed"
            echo "PASSED" > "$results_dir/benchmark_status"
        else
            log "⚠ Benchmarks failed or unavailable"
            echo "FAILED" > "$results_dir/benchmark_status"
        fi
    else
        echo "NONE" > "$results_dir/benchmark_status"
    fi
    
    return 0
}
```

### 5. GitHub Integration Tools

#### GitHub API (`github_api`)

**Purpose**: PR management, comments, and status updates

**GitHub Integration Functions**:
```bash
# GitHub API integration for all agents
GITHUB_API_URL="${GITHUB_API_URL:-https://api.github.com}"
GITHUB_TOKEN="${GITHUB_TOKEN:-}"
GITHUB_REPOSITORY="${GITHUB_REPOSITORY:-}"

validate_github_config() {
    if [[ -z "$GITHUB_TOKEN" ]]; then
        error "GitHub token not configured"
        return 1
    fi
    
    if [[ -z "$GITHUB_REPOSITORY" ]]; then
        error "GitHub repository not configured"
        return 1
    fi
    
    # Test GitHub API connectivity
    if ! github_api_request "GET" "/rate_limit" > /dev/null 2>&1; then
        error "GitHub API not accessible"
        return 1
    fi
    
    log "GitHub integration configured and accessible"
    return 0
}

github_api_request() {
    local method="$1"
    local endpoint="$2"
    local data="${3:-}"
    
    local curl_args=(
        -X "$method"
        -H "Authorization: token $GITHUB_TOKEN"
        -H "Accept: application/vnd.github.v3+json"
        -H "User-Agent: TaskMaster-Agent"
        --max-time 30
        --silent
        --show-error
        --write-out "%{http_code}"
    )
    
    if [[ -n "$data" ]]; then
        curl_args+=(
            -H "Content-Type: application/json"
            -d "$data"
        )
    fi
    
    local response=$(curl "${curl_args[@]}" "$GITHUB_API_URL$endpoint" 2>/tmp/curl_error)
    local http_code="${response: -3}"
    local body="${response%???}"
    
    if [[ $http_code -ge 200 && $http_code -lt 300 ]]; then
        echo "$body"
        return 0
    else
        handle_github_api_error "$http_code" "$body"
        return 1
    fi
}

get_pr_number_from_git() {
    # Try to extract PR number from git log
    local pr_number=$(git log --oneline -10 | grep -oE '#[0-9]+' | head -1 | sed 's/#//')
    
    if [[ -n "$pr_number" ]]; then
        echo "$pr_number"
        return 0
    fi
    
    # Try to get from git branch if following naming convention
    local branch_name=$(git branch --show-current 2>/dev/null || echo "")
    if [[ "$branch_name" =~ pr-([0-9]+) ]]; then
        echo "${BASH_REMATCH[1]}"
        return 0
    fi
    
    return 1
}

create_pr_comment() {
    local pr_number="$1"
    local comment_body="$2"
    
    if [[ -z "$pr_number" ]]; then
        error "PR number not provided for comment"
        return 1
    fi
    
    log "Creating PR comment on #$pr_number"
    
    local comment_data=$(jq -n --arg body "$comment_body" '{body: $body}')
    
    if github_api_request "POST" "/repos/$GITHUB_REPOSITORY/issues/$pr_number/comments" "$comment_data" > /dev/null; then
        log "PR comment created successfully"
        return 0
    else
        error "Failed to create PR comment"
        return 1
    fi
}

add_pr_labels() {
    local pr_number="$1"
    shift
    local labels=("$@")
    
    if [[ -z "$pr_number" ]]; then
        error "PR number not provided for labeling"
        return 1
    fi
    
    log "Adding labels to PR #$pr_number: ${labels[*]}"
    
    local labels_json=$(printf '%s\n' "${labels[@]}" | jq -R . | jq -s .)
    local label_data=$(jq -n --argjson labels "$labels_json" '{labels: $labels}')
    
    if github_api_request "POST" "/repos/$GITHUB_REPOSITORY/issues/$pr_number/labels" "$label_data" > /dev/null; then
        log "PR labels added successfully"
        return 0
    else
        error "Failed to add PR labels"
        return 1
    fi
}

approve_pr() {
    local pr_number="$1"
    local review_body="$2"
    
    if [[ -z "$pr_number" ]]; then
        error "PR number not provided for approval"
        return 1
    fi
    
    log "Approving PR #$pr_number"
    
    local review_data=$(jq -n \
        --arg body "$review_body" \
        --arg event "APPROVE" \
        '{body: $body, event: $event}')
    
    if github_api_request "POST" "/repos/$GITHUB_REPOSITORY/pulls/$pr_number/reviews" "$review_data" > /dev/null; then
        log "PR approved successfully"
        return 0
    else
        error "Failed to approve PR"
        return 1
    fi
}

check_github_rate_limit() {
    local rate_limit_info
    if rate_limit_info=$(github_api_request "GET" "/rate_limit"); then
        local remaining=$(echo "$rate_limit_info" | jq -r '.rate.remaining')
        local reset_time=$(echo "$rate_limit_info" | jq -r '.rate.reset')
        
        log "GitHub API rate limit: $remaining requests remaining"
        
        if [[ $remaining -lt 100 ]]; then
            local reset_date=$(date -d "@$reset_time" 2>/dev/null || date -r "$reset_time" 2>/dev/null || echo "unknown")
            log "WARNING: GitHub API rate limit low, resets at $reset_date"
        fi
        
        return 0
    else
        error "Failed to check GitHub API rate limit"
        return 1
    fi
}
```

## Best Practices and Integration Patterns

### 1. Template Variable Management

```bash
# Template variable handling with validation
handle_template_variables() {
    log "Processing template variables"
    
    # Define required variables per agent
    case "$AGENT_TYPE" in
        "rex")
            REQUIRED_VARS=("GITHUB_APP" "TASK_ID" "WORKSPACE_PATH" "GITHUB_TOKEN" "MCP_SERVER_URL")
            ;;
        "cleo")
            REQUIRED_VARS=("GITHUB_APP" "TASK_ID" "WORKSPACE_PATH" "GITHUB_TOKEN" "QUALITY_RULES")
            ;;
        "tess")
            REQUIRED_VARS=("GITHUB_APP" "TASK_ID" "WORKSPACE_PATH" "GITHUB_TOKEN" "COVERAGE_THRESHOLD")
            ;;
        *)
            REQUIRED_VARS=("GITHUB_APP" "TASK_ID" "WORKSPACE_PATH" "GITHUB_TOKEN")
            ;;
    esac
    
    # Validate all required variables
    local missing_vars=()
    for var in "${REQUIRED_VARS[@]}"; do
        if [[ -z "${!var:-}" ]]; then
            missing_vars+=("$var")
        fi
    done
    
    if [[ ${#missing_vars[@]} -gt 0 ]]; then
        error "Missing required template variables: ${missing_vars[*]}"
        return 1
    fi
    
    # Set defaults for optional variables
    MCP_TIMEOUT="${MCP_TIMEOUT:-30}"
    COVERAGE_THRESHOLD="${COVERAGE_THRESHOLD:-95}"
    TEST_TIMEOUT="${TEST_TIMEOUT:-600}"
    DEBUG="${DEBUG:-false}"
    
    log "Template variables validated successfully"
}
```

### 2. Error Recovery and Resilience

```bash
# Comprehensive error recovery system
implement_error_recovery() {
    # Network operation with exponential backoff
    retry_with_backoff() {
        local max_attempts=5
        local base_delay=2
        local max_delay=60
        local attempt=1
        local command="$1"
        
        while [[ $attempt -le $max_attempts ]]; do
            if eval "$command"; then
                return 0
            fi
            
            if [[ $attempt -eq $max_attempts ]]; then
                error "Command failed after $max_attempts attempts: $command"
                return 1
            fi
            
            local delay=$((base_delay * (2 ** (attempt - 1))))
            if [[ $delay -gt $max_delay ]]; then
                delay=$max_delay
            fi
            
            log "Attempt $attempt failed, retrying in ${delay}s..."
            sleep "$delay"
            ((attempt++))
        done
    }
    
    # Partial failure recovery
    handle_partial_failure() {
        local operation="$1"
        local critical="${2:-false}"
        
        log "Handling partial failure in $operation (critical: $critical)"
        
        if [[ "$critical" == "true" ]]; then
            error "Critical operation failed: $operation"
            return 1
        else
            log "Non-critical operation failed: $operation - continuing"
            return 0
        fi
    }
    
    # Resource cleanup on exit
    cleanup_on_exit() {
        local exit_code=$?
        
        log "Performing cleanup (exit code: $exit_code)"
        
        # Kill background processes
        jobs -p | xargs -r kill 2>/dev/null || true
        
        # Clean temporary files
        rm -f /tmp/agent_* 2>/dev/null || true
        
        # Final status report
        if [[ $exit_code -eq 0 ]]; then
            log "Agent workflow completed successfully"
        else
            error "Agent workflow failed with exit code $exit_code"
        fi
    }
    
    trap cleanup_on_exit EXIT
}
```

### 3. Performance Monitoring and Optimization

```bash
# Performance monitoring for agent scripts
monitor_performance() {
    local start_time=$(date +%s)
    local operation="$1"
    
    log "Starting performance monitoring for: $operation"
    
    # Memory monitoring
    local initial_memory=$(ps -o rss= -p $$ 2>/dev/null || echo "0")
    
    # Execute operation with monitoring
    "$operation"
    local exit_code=$?
    
    # Calculate metrics
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    local final_memory=$(ps -o rss= -p $$ 2>/dev/null || echo "0")
    local memory_delta=$((final_memory - initial_memory))
    
    log "Performance metrics for $operation:"
    log "  Duration: ${duration}s"
    log "  Memory delta: ${memory_delta}KB"
    log "  Exit code: $exit_code"
    
    # Emit metrics for monitoring systems
    emit_performance_metrics "$operation" "$duration" "$memory_delta" "$exit_code"
    
    return $exit_code
}

emit_performance_metrics() {
    local operation="$1"
    local duration="$2"
    local memory_delta="$3"
    local exit_code="$4"
    
    # Emit metrics to pushgateway if available
    if command -v curl > /dev/null 2>&1 && [[ -n "${PUSHGATEWAY_URL:-}" ]]; then
        cat <<EOF | curl -X POST --data-binary @- "$PUSHGATEWAY_URL/metrics/job/agent-scripts" 2>/dev/null || true
agent_operation_duration_seconds{agent="$AGENT_TYPE",operation="$operation"} $duration
agent_operation_memory_delta_kb{agent="$AGENT_TYPE",operation="$operation"} $memory_delta  
agent_operation_exit_code{agent="$AGENT_TYPE",operation="$operation"} $exit_code
EOF
    fi
}
```

This comprehensive guide provides all the necessary tools, patterns, and best practices for implementing robust agent-specific container scripts that enable the specialized functionality of Rex, Cleo, and Tess agents while maintaining seamless integration with the Task Master platform infrastructure.