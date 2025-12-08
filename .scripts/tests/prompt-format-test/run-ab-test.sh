#!/bin/bash
# =============================================================================
# A/B Test: XML vs Markdown Prompt Formats Across CLIs
# =============================================================================
# This script runs the same task with different prompt formats and CLIs,
# then compares results to determine which format performs better.
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
TEST_DIR="$REPO_ROOT/.prompt-ab-test"
RESULTS_DIR="$TEST_DIR/results"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Test configuration
FORMATS=("markdown" "xml")
CLIS=("claude" "codex")  # Available: claude, codex, gemini, opencode, cursor
MAX_RETRIES=2
TIMEOUT_SECONDS=180  # 3 minutes per test (simple task)

# Expected solution for validation
EXPECTED_FILE="src/calculator.rs"

echo "════════════════════════════════════════════════════════════════"
echo "║ Prompt Format A/B Test"
echo "║ Formats: ${FORMATS[*]}"
echo "║ CLIs: ${CLIS[*]}"
echo "════════════════════════════════════════════════════════════════"
echo ""

# Setup
rm -rf "$TEST_DIR"
mkdir -p "$RESULTS_DIR"

# Initialize results CSV
RESULTS_CSV="$RESULTS_DIR/results.csv"
echo "cli,format,success,retries,time_seconds,files_created,tests_passed,clippy_passed,acceptance_pct" > "$RESULTS_CSV"

# Function to create test workspace
create_workspace() {
    local name="$1"
    local workspace="$TEST_DIR/workspaces/$name"
    
    mkdir -p "$workspace/src"
    
    # Create minimal Cargo.toml with [workspace] to isolate from parent
    cat > "$workspace/Cargo.toml" << 'EOF'
[package]
name = "calculator-test"
version = "0.1.0"
edition = "2021"

[dependencies]

[workspace]
EOF
    
    # Create src/lib.rs
    cat > "$workspace/src/lib.rs" << 'EOF'
// Calculator module will be added here
EOF
    
    echo "$workspace"
}

# Function to validate results
validate_results() {
    local workspace="$1"
    local result_file="$2"
    
    local success=0
    local files_created=0
    local tests_passed=0
    local clippy_passed=0
    local acceptance_pct=0
    local total_criteria=11
    local passed_criteria=0
    
    cd "$workspace"
    
    # Check if calculator.rs exists
    if [ -f "src/calculator.rs" ]; then
        ((files_created++))
        ((passed_criteria++))
        
        # Check for each function
        if grep -q "pub fn add" src/calculator.rs; then ((passed_criteria++)); fi
        if grep -q "pub fn subtract" src/calculator.rs; then ((passed_criteria++)); fi
        if grep -q "pub fn multiply" src/calculator.rs; then ((passed_criteria++)); fi
        if grep -q "pub fn divide" src/calculator.rs; then ((passed_criteria++)); fi
        if grep -q "Result<" src/calculator.rs; then ((passed_criteria++)); fi
        if grep -q "division by zero" src/calculator.rs; then ((passed_criteria++)); fi
        if grep -q "#\[cfg(test)\]" src/calculator.rs; then ((passed_criteria++)); fi
    fi
    
    # Check module registration
    if grep -q "mod calculator" src/lib.rs 2>/dev/null || grep -q "mod calculator" src/main.rs 2>/dev/null; then
        ((passed_criteria++))
    fi
    
    # Run cargo test
    if [ -f "src/calculator.rs" ]; then
        if cargo test --quiet 2>/dev/null; then
            tests_passed=1
            ((passed_criteria++))
        fi
        
        # Run clippy (check only, no warnings as errors for test)
        if cargo clippy --quiet 2>/dev/null; then
            clippy_passed=1
            ((passed_criteria++))
        fi
    fi
    
    # Calculate acceptance percentage
    acceptance_pct=$((passed_criteria * 100 / total_criteria))
    
    # Determine overall success (>= 90% acceptance)
    if [ "$acceptance_pct" -ge 90 ]; then
        success=1
    fi
    
    echo "$success,$files_created,$tests_passed,$clippy_passed,$acceptance_pct" > "$result_file"
}

# Function to run a single test
run_test() {
    local cli="$1"
    local format="$2"
    local attempt="$3"
    
    local test_name="${cli}-${format}-attempt${attempt}"
    local workspace
    workspace=$(create_workspace "$test_name")
    local prompt_file
    
    if [ "$format" = "xml" ]; then
        prompt_file="$SCRIPT_DIR/task.xml"
    else
        prompt_file="$SCRIPT_DIR/task.md"
    fi
    
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${CYAN}Test: $cli / $format (Attempt $attempt)${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    
    local start_time
    start_time=$(date +%s)
    
    # Copy prompt to workspace
    cp "$prompt_file" "$workspace/task-prompt.${format}"
    cp "$SCRIPT_DIR/acceptance-criteria.md" "$workspace/"
    
    # Here we would invoke the actual CLI
    # For now, simulate with a mock that creates expected files
    # In production, this would be:
    # timeout $TIMEOUT_SECONDS $CLI_COMMAND --prompt "$prompt_file" --workspace "$workspace"
    
    echo "  📁 Workspace: $workspace"
    echo "  📄 Prompt: $prompt_file"
    echo "  ⏱️  Timeout: ${TIMEOUT_SECONDS}s"
    echo ""
    
    # Execute CLI
    if [ "${DRY_RUN:-false}" = "true" ]; then
        echo -e "  ${YELLOW}⚠️  DRY RUN - Simulating CLI execution${NC}"
        
        # Simulate successful implementation for testing the harness
        cat > "$workspace/src/calculator.rs" << 'EOF'
pub fn add(a: i32, b: i32) -> i32 { a + b }
pub fn subtract(a: i32, b: i32) -> i32 { a - b }
pub fn multiply(a: i32, b: i32) -> i32 { a * b }
pub fn divide(a: i32, b: i32) -> Result<i32, &'static str> {
    if b == 0 { Err("division by zero") } else { Ok(a / b) }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_add() { assert_eq!(add(2, 3), 5); }
    #[test] fn test_subtract() { assert_eq!(subtract(5, 3), 2); }
    #[test] fn test_multiply() { assert_eq!(multiply(4, 3), 12); }
    #[test] fn test_divide() { assert_eq!(divide(10, 2), Ok(5)); }
    #[test] fn test_divide_by_zero() { assert_eq!(divide(10, 0), Err("division by zero")); }
}
EOF
        echo "pub mod calculator;" > "$workspace/src/lib.rs"
    else
        # Real CLI execution
        echo -e "  ${GREEN}🚀 Running $cli CLI...${NC}"
        
        local cli_log="$RESULTS_DIR/${test_name}-cli.log"
        
        cd "$workspace"
        
        # Initialize git repo (some CLIs require it)
        git init --quiet 2>/dev/null || true
        git add -A 2>/dev/null || true
        git commit -m "Initial commit" --quiet 2>/dev/null || true
        
        # Resolve CLI paths (some may be aliases)
        local cli_path
        case "$cli" in
            claude) cli_path="${CLAUDE_PATH:-$HOME/.claude/local/claude}" ;;
            codex) cli_path="${CODEX_PATH:-$(which codex 2>/dev/null || echo codex)}" ;;
            opencode) cli_path="${OPENCODE_PATH:-$(which opencode 2>/dev/null || echo opencode)}" ;;
            gemini) cli_path="${GEMINI_PATH:-$(which gemini 2>/dev/null || echo gemini)}" ;;
            *) cli_path="$cli" ;;
        esac
        
        case "$cli" in
            claude)
                # Claude CLI: --print for non-interactive, --dangerously-skip-permissions for automation
                echo "  Command: $cli_path -p --dangerously-skip-permissions \"<prompt>\""
                timeout "$TIMEOUT_SECONDS" "$cli_path" -p --dangerously-skip-permissions "$(cat "$prompt_file")" > "$cli_log" 2>&1 || true
                ;;
            codex)
                # Codex CLI: exec mode for non-interactive
                echo "  Command: $cli_path exec --full-auto \"<prompt>\""
                timeout "$TIMEOUT_SECONDS" "$cli_path" exec --full-auto "$(cat "$prompt_file")" > "$cli_log" 2>&1 || true
                ;;
            opencode)
                # OpenCode CLI
                echo "  Command: $cli_path \"<prompt>\""
                timeout "$TIMEOUT_SECONDS" "$cli_path" "$(cat "$prompt_file")" > "$cli_log" 2>&1 || true
                ;;
            gemini)
                # Gemini CLI
                echo "  Command: $cli_path \"<prompt>\""
                timeout "$TIMEOUT_SECONDS" "$cli_path" "$(cat "$prompt_file")" > "$cli_log" 2>&1 || true
                ;;
            *)
                echo -e "  ${RED}Unknown CLI: $cli${NC}"
                ;;
        esac
        
        local exit_code=$?
        cd - > /dev/null
        
        echo "  📝 Log: $cli_log"
        if [ $exit_code -eq 124 ]; then
            echo -e "  ${YELLOW}⚠️  Timed out after ${TIMEOUT_SECONDS}s${NC}"
        elif [ $exit_code -ne 0 ]; then
            echo -e "  ${YELLOW}⚠️  CLI exited with code $exit_code${NC}"
        fi
    fi
    
    local end_time
    end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    # Validate results
    local validation_file="$RESULTS_DIR/${test_name}-validation.txt"
    validate_results "$workspace" "$validation_file"
    
    # Read validation results
    local validation_result
    validation_result=$(cat "$validation_file")
    
    # Parse results
    local success files_created tests_passed clippy_passed acceptance_pct
    IFS=',' read -r success files_created tests_passed clippy_passed acceptance_pct <<< "$validation_result"
    
    # Record to CSV
    echo "$cli,$format,$success,$attempt,$duration,$files_created,$tests_passed,$clippy_passed,$acceptance_pct" >> "$RESULTS_CSV"
    
    # Display results
    if [ "$success" = "1" ]; then
        echo -e "  ${GREEN}✓ SUCCESS${NC} - Acceptance: ${acceptance_pct}%"
    else
        echo -e "  ${RED}✗ FAILED${NC} - Acceptance: ${acceptance_pct}%"
    fi
    echo "  ⏱️  Duration: ${duration}s"
    echo ""
    
    # Return 0 for success (bash convention), 1 for failure
    if [ "$success" = "1" ]; then
        return 0
    else
        return 1
    fi
}

# Main test loop
echo ""
echo "════════════════════════════════════════════════════════════════"
echo "║ Running Tests"
echo "════════════════════════════════════════════════════════════════"
echo ""

for cli in "${CLIS[@]}"; do
    for format in "${FORMATS[@]}"; do
        attempt=1
        while [ $attempt -le $MAX_RETRIES ]; do
            if run_test "$cli" "$format" "$attempt"; then
                break  # Success, no more retries needed
            fi
            ((attempt++))
        done
    done
done

# Generate summary report
echo ""
echo "════════════════════════════════════════════════════════════════"
echo "║ Results Summary"
echo "════════════════════════════════════════════════════════════════"
echo ""

echo "📊 Results saved to: $RESULTS_CSV"
echo ""

# Parse and display summary
echo "┌─────────┬──────────┬─────────┬──────────┬─────────────────┐"
echo "│   CLI   │  Format  │ Success │ Retries  │ Acceptance %    │"
echo "├─────────┼──────────┼─────────┼──────────┼─────────────────┤"

tail -n +2 "$RESULTS_CSV" | while IFS=',' read -r cli format success retries duration files tests clippy acceptance; do
    success_icon="✗"
    [ "$success" = "1" ] && success_icon="✓"
    printf "│ %-7s │ %-8s │    %s    │    %s     │      %3s%%       │\n" "$cli" "$format" "$success_icon" "$retries" "$acceptance"
done

echo "└─────────┴──────────┴─────────┴──────────┴─────────────────┘"
echo ""

# Calculate format comparison
echo "📈 Format Comparison:"
echo ""

for format in "${FORMATS[@]}"; do
    total=0
    successes=0
    total_acceptance=0
    count=0
    
    while IFS=',' read -r cli fmt success retries duration files tests clippy acceptance; do
        if [ "$fmt" = "$format" ]; then
            ((total++))
            [ "$success" = "1" ] && ((successes++))
            total_acceptance=$((total_acceptance + acceptance))
            ((count++))
        fi
    done < <(tail -n +2 "$RESULTS_CSV")
    
    if [ "$count" -gt 0 ]; then
        avg_acceptance=$((total_acceptance / count))
        success_rate=$((successes * 100 / total))
        echo "  $format: ${success_rate}% success rate, ${avg_acceptance}% avg acceptance"
    fi
done

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "║ Test Complete"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "To run with actual CLIs (not dry run):"
echo "  DRY_RUN=false ./run-ab-test.sh"
echo ""

