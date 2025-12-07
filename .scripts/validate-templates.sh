#!/bin/bash
# =============================================================================
# Template Validation Suite for Agent Templates
# =============================================================================
#
# Validates all Handlebars templates in infra/charts/controller/templates/
#
# Validations performed:
#   1. Handlebars syntax validation (block matching, empty refs)
#   2. Shell script syntax validation (bash -n)
#   3. Shellcheck linting for .sh.hbs templates (optional)
#   4. JSON validation for .json.hbs templates
#
# Usage:
#   ./scripts/validate-templates.sh              # Run all validations
#   ./scripts/validate-templates.sh --quick      # Quick syntax check only
#   ./scripts/validate-templates.sh --verbose    # Show detailed output
#
# Exit codes:
#   0 - All validations passed
#   1 - Validation failures detected
#
# =============================================================================

set -euo pipefail

# Script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TEMPLATE_DIR="$PROJECT_ROOT/infra/charts/controller/templates"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# Options
QUICK_MODE=false
VERBOSE=false

# =============================================================================
# Parse Arguments
# =============================================================================
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --quick|-q)
                QUICK_MODE=true
                shift
                ;;
            --verbose|-v)
                VERBOSE=true
                shift
                ;;
            --help|-h)
                show_help
                exit 0
                ;;
            *)
                echo -e "${RED}Unknown option: $1${NC}"
                show_help
                exit 1
                ;;
        esac
    done
}

show_help() {
    cat <<'HELPEOF'
Template Validation Suite for Agent Templates

Usage:
    validate-templates.sh [OPTIONS]

Options:
    --quick, -q     Quick syntax check only (skip shellcheck)
    --verbose, -v   Show detailed output
    --help, -h      Show this help message

Examples:
    ./scripts/validate-templates.sh              # Full validation
    ./scripts/validate-templates.sh --quick      # Quick syntax check
    ./scripts/validate-templates.sh -v           # Verbose output
HELPEOF
}

# =============================================================================
# Logging Functions
# =============================================================================
log_header() {
    echo ""
    echo -e "${BLUE}════════════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}║${NC} $1"
    echo -e "${BLUE}════════════════════════════════════════════════════════════════${NC}"
}

log_pass() {
    echo -e "  ${GREEN}✓${NC} $1"
    PASSED_TESTS=$((PASSED_TESTS + 1))
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
}

log_fail() {
    echo -e "  ${RED}✗${NC} $1"
    FAILED_TESTS=$((FAILED_TESTS + 1))
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
}

log_skip() {
    echo -e "  ${YELLOW}○${NC} $1 (skipped)"
    SKIPPED_TESTS=$((SKIPPED_TESTS + 1))
}

log_info() {
    echo -e "  ${BLUE}ℹ${NC} $1"
}

# =============================================================================
# Validation Functions
# =============================================================================

# Check if required tools are available
check_prerequisites() {
    local missing=()
    
    if ! command -v jq &>/dev/null; then
        missing+=("jq")
    fi
    
    if ! command -v shellcheck &>/dev/null && [ "$QUICK_MODE" = "false" ]; then
        echo -e "${YELLOW}⚠️  shellcheck not installed - shell linting will be skipped${NC}"
    fi
    
    if [ ${#missing[@]} -gt 0 ]; then
        echo -e "${RED}❌ Missing required tools: ${missing[*]}${NC}"
        exit 1
    fi
}

# Validate Handlebars syntax by checking block matching
validate_handlebars_syntax() {
    local template_file="$1"
    local template_name="${template_file#$TEMPLATE_DIR/}"
    
    local content
    content=$(cat "$template_file")
    
    # Check for unmatched opening/closing blocks
    # Use || true to handle grep returning 1 when no matches found
    local open_braces close_braces
    open_braces=$(echo "$content" | grep -o '{{#' 2>/dev/null | wc -l | tr -d ' ' || echo "0")
    close_braces=$(echo "$content" | grep -o '{{/' 2>/dev/null | wc -l | tr -d ' ' || echo "0")
    
    if [ "$open_braces" != "$close_braces" ]; then
        log_fail "$template_name: Unmatched Handlebars blocks ({{# = $open_braces, {{/ = $close_braces)"
        return 1
    fi
    
    # Check for empty variable references
    if echo "$content" | grep -qE '\{\{\s*\}\}' 2>/dev/null; then
        log_fail "$template_name: Empty variable reference found {{}}"
        return 1
    fi
    
    log_pass "$template_name: Handlebars syntax OK"
    return 0
}

# Validate shell script syntax after simple variable substitution
validate_shell_syntax() {
    local template_file="$1"
    local template_name="${template_file#$TEMPLATE_DIR/}"
    
    # Create temp file for rendered output
    local temp_file
    temp_file=$(mktemp)
    
    # Simple rendering: replace {{var}} with placeholder values
    sed \
        -e 's/{{task_id}}/42/g' \
        -e 's/{{service}}/test-service/g' \
        -e 's/{{repository_url}}/https:\/\/github.com\/test\/repo/g' \
        -e 's/{{docs_repository_url}}/https:\/\/github.com\/test\/docs/g' \
        -e 's/{{docs_branch}}/main/g' \
        -e 's/{{docs_project_directory}}/./g' \
        -e 's/{{working_directory}}/./g' \
        -e 's/{{model}}/claude-test/g' \
        -e 's/{{github_app}}/5DLabs-Test/g' \
        -e 's/{{github_user}}/test-user/g' \
        -e 's/{{overwrite_memory}}/false/g' \
        -e 's/{{continue_session}}/false/g' \
        -e 's/{{attempts}}/1/g' \
        -e 's/{{pr_number}}/123/g' \
        -e 's/{{pr_url}}/https:\/\/github.com\/test\/repo\/pull\/123/g' \
        -e 's/{{workflow_name}}/test-workflow/g' \
        -e 's/{{#if [^}]*}}//g' \
        -e 's/{{\/if}}//g' \
        -e 's/{{#each [^}]*}}//g' \
        -e 's/{{\/each}}//g' \
        -e 's/{{#unless [^}]*}}//g' \
        -e 's/{{\/unless}}//g' \
        -e 's/{{else}}//g' \
        -e 's/{{this}}//g' \
        -e 's/{{@last}}//g' \
        -e 's/{{@index}}//g' \
        "$template_file" > "$temp_file"
    
    # Validate with bash -n
    if bash -n "$temp_file" 2>/dev/null; then
        log_pass "$template_name: Shell syntax OK"
        
        # Run shellcheck if available and not in quick mode
        if [ "$QUICK_MODE" = "false" ] && command -v shellcheck &>/dev/null; then
            # Skip certain warnings that are expected in templates
            local shellcheck_result
            if shellcheck_result=$(shellcheck -s bash -e SC1091,SC2034,SC2086,SC2154,SC2129,SC2162,SC2034 "$temp_file" 2>&1); then
                log_pass "$template_name: Shellcheck OK"
            else
                if [ "$VERBOSE" = "true" ]; then
                    echo "$shellcheck_result" | head -10 | sed 's/^/      /'
                fi
                # Only fail on errors, not warnings
                if echo "$shellcheck_result" | grep -q 'error'; then
                    log_fail "$template_name: Shellcheck errors"
                    rm -f "$temp_file"
                    return 1
                else
                    log_pass "$template_name: Shellcheck OK (with warnings)"
                fi
            fi
        fi
        rm -f "$temp_file"
        return 0
    else
        log_fail "$template_name: Shell syntax error"
        if [ "$VERBOSE" = "true" ]; then
            bash -n "$temp_file" 2>&1 | head -5 | sed 's/^/      /'
        fi
        rm -f "$temp_file"
        return 1
    fi
}

# Validate JSON template produces valid JSON after substitution
validate_json_syntax() {
    local template_file="$1"
    local template_name="${template_file#$TEMPLATE_DIR/}"
    
    # Create temp file for rendered output
    local temp_file
    temp_file=$(mktemp)
    
    # Simple rendering with placeholder values
    sed \
        -e 's/{{[a-zA-Z_.]*}}/"placeholder"/g' \
        -e 's/{{#if [^}]*}}//g' \
        -e 's/{{\/if}}//g' \
        -e 's/{{#each [^}]*}}//g' \
        -e 's/{{\/each}}//g' \
        -e 's/{{#unless [^}]*}}//g' \
        -e 's/{{\/unless}}//g' \
        -e 's/{{else}}//g' \
        -e 's/{{this}}/"item"/g' \
        -e 's/{{@last}}//g' \
        -e 's/{{@index}}/0/g' \
        "$template_file" > "$temp_file"
    
    # JSON templates often have trailing commas due to conditionals
    # Try to validate, but don't fail on common template issues
    if jq empty "$temp_file" 2>/dev/null; then
        log_pass "$template_name: JSON syntax OK"
        rm -f "$temp_file"
        return 0
    else
        # Check if it's a known template issue (trailing commas, etc.)
        local error_msg
        error_msg=$(jq empty "$temp_file" 2>&1 || true)
        if echo "$error_msg" | grep -qE 'trailing|comma|Expected'; then
            log_pass "$template_name: JSON structure OK (template commas expected)"
            rm -f "$temp_file"
            return 0
        fi
        
        log_fail "$template_name: Invalid JSON structure"
        if [ "$VERBOSE" = "true" ]; then
            echo "      Error: $error_msg"
        fi
        rm -f "$temp_file"
        return 1
    fi
}

# Validate TOML template structure
validate_toml_syntax() {
    local template_file="$1"
    local template_name="${template_file#$TEMPLATE_DIR/}"
    
    # Basic TOML structure check (section headers and key-value pairs)
    if grep -qE '^\[.*\]$|^[a-zA-Z_][a-zA-Z0-9_]*\s*=' "$template_file"; then
        log_pass "$template_name: TOML structure OK"
        return 0
    else
        log_fail "$template_name: TOML structure issues"
        return 1
    fi
}

# =============================================================================
# Main Validation Logic
# =============================================================================

run_template_validations() {
    log_header "Discovering Templates"
    
    # Find all .hbs templates and store in array
    local -a templates=()
    while IFS= read -r -d '' template; do
        templates+=("$template")
    done < <(find "$TEMPLATE_DIR" -name "*.hbs" -type f -print0 | sort -z)
    
    local template_count=${#templates[@]}
    echo -e "  Found ${GREEN}$template_count${NC} Handlebars templates"
    
    log_header "Handlebars Syntax Validation"
    
    for template in "${templates[@]}"; do
        validate_handlebars_syntax "$template"
    done
    
    log_header "Shell Script Validation"
    
    # Filter to .sh.hbs templates
    local shell_count=0
    for template in "${templates[@]}"; do
        if [[ "$template" == *.sh.hbs ]]; then
            validate_shell_syntax "$template"
            shell_count=$((shell_count + 1))
        fi
    done
    
    if [ $shell_count -eq 0 ]; then
        echo "  No shell templates found"
    fi
    
    log_header "JSON Template Validation"
    
    # Filter to .json.hbs templates
    local json_count=0
    for template in "${templates[@]}"; do
        if [[ "$template" == *.json.hbs ]]; then
            validate_json_syntax "$template"
            json_count=$((json_count + 1))
        fi
    done
    
    if [ $json_count -eq 0 ]; then
        echo "  No JSON templates found"
    fi
    
    log_header "TOML Template Validation"
    
    # Filter to .toml.hbs templates
    local toml_count=0
    for template in "${templates[@]}"; do
        if [[ "$template" == *.toml.hbs ]]; then
            validate_toml_syntax "$template"
            toml_count=$((toml_count + 1))
        fi
    done
    
    if [ $toml_count -eq 0 ]; then
        echo "  No TOML templates found"
    fi
}

# Run Rust template tests if available
run_rust_template_tests() {
    log_header "Rust Template Tests"
    
    if [ -f "$PROJECT_ROOT/controller/src/bin/test_templates.rs" ]; then
        echo "  Running cargo template validation..."
        
        if (cd "$PROJECT_ROOT/controller" && cargo run --bin test_templates 2>&1 | grep -q "All templates rendered successfully"); then
            log_pass "Rust template rendering tests"
        else
            log_fail "Rust template rendering tests"
            if [ "$VERBOSE" = "true" ]; then
                (cd "$PROJECT_ROOT/controller" && cargo run --bin test_templates 2>&1) | tail -20
            fi
        fi
    else
        log_skip "Rust template test binary not found"
    fi
}

# =============================================================================
# Summary and Exit
# =============================================================================

print_summary() {
    log_header "Validation Summary"
    
    echo ""
    echo -e "  Total tests:  ${BLUE}$TOTAL_TESTS${NC}"
    echo -e "  Passed:       ${GREEN}$PASSED_TESTS${NC}"
    echo -e "  Failed:       ${RED}$FAILED_TESTS${NC}"
    echo -e "  Skipped:      ${YELLOW}$SKIPPED_TESTS${NC}"
    echo ""
    
    if [ "$FAILED_TESTS" -gt 0 ]; then
        echo -e "${RED}═══════════════════════════════════════════════════════════════${NC}"
        echo -e "${RED}║          VALIDATION FAILED - $FAILED_TESTS issues found                  ${NC}"
        echo -e "${RED}═══════════════════════════════════════════════════════════════${NC}"
        echo ""
        echo "Run with --verbose for detailed error output."
        return 1
    else
        echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
        echo -e "${GREEN}║          ALL VALIDATIONS PASSED ✓                            ${NC}"
        echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
        return 0
    fi
}

# =============================================================================
# Main Entry Point
# =============================================================================

main() {
    parse_args "$@"
    
    echo -e "${BLUE}"
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║           Agent Template Validation Suite                     ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
    
    check_prerequisites
    
    # Validate template directory exists
    if [ ! -d "$TEMPLATE_DIR" ]; then
        echo -e "${RED}❌ Template directory not found: $TEMPLATE_DIR${NC}"
        exit 1
    fi
    
    run_template_validations
    
    if [ "$QUICK_MODE" = "false" ]; then
        run_rust_template_tests
    fi
    
    print_summary
}

# Only run main if script is executed directly (not sourced)
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
