#!/usr/bin/env bash
# =============================================================================
# local-ci.sh - Run the same checks locally that CI runs
# =============================================================================
# This script mirrors ALL GitHub Actions CI checks:
#   - Rust: fmt, clippy (pedantic), tests
#   - Infrastructure: YAML lint, Helm, schema validation
#   - Security: detect-secrets, basic security checks
#
# Usage:
#   ./scripts/local-ci.sh              # Check changed files only
#   ./scripts/local-ci.sh --all        # Check everything
#   ./scripts/local-ci.sh --rust       # Rust checks only
#   ./scripts/local-ci.sh --infra      # Infrastructure checks only
# =============================================================================

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

# Configuration
CLIPPY_ARGS="-D warnings -W clippy::pedantic"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Track failures
FAILURES=()
WARNINGS=()

# Logging functions
log_info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
log_success() { echo -e "${GREEN}✅ $1${NC}"; }
log_warning() { echo -e "${YELLOW}⚠️  $1${NC}"; WARNINGS+=("$1"); }
log_error() { echo -e "${RED}❌ $1${NC}"; }
log_header() { 
    echo ""
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${CYAN}   $1${NC}"
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
    echo ""
}
log_section() {
    echo ""
    echo -e "${MAGENTA}───────────────────────────────────────────────────────────────${NC}"
    echo -e "${MAGENTA}   $1${NC}"
    echo -e "${MAGENTA}───────────────────────────────────────────────────────────────${NC}"
}

# =============================================================================
# UTILITY FUNCTIONS
# =============================================================================

# Check if a command exists
check_command() {
    local cmd="$1"
    local install_hint="${2:-}"
    
    if ! command -v "$cmd" &>/dev/null; then
        if [ -n "$install_hint" ]; then
            log_warning "$cmd not found. $install_hint"
        else
            log_warning "$cmd not found, skipping related checks"
        fi
        return 1
    fi
    return 0
}

# Get package name from Cargo.toml
get_package_name() {
    local crate_dir="$1"
    local toml="$REPO_ROOT/crates/$crate_dir/Cargo.toml"
    if [ -f "$toml" ]; then
        grep '^name = ' "$toml" | head -1 | sed 's/name = "\(.*\)"/\1/'
    fi
}

# Get list of all Rust crates in the repo (returns package names)
get_all_crates() {
    find "$REPO_ROOT/crates" -name "Cargo.toml" -type f 2>/dev/null | while read -r toml; do
        grep '^name = ' "$toml" | head -1 | sed 's/name = "\(.*\)"/\1/'
    done | sort -u
}

# Get list of changed crates based on git diff (returns package names)
get_changed_crates() {
    local base_ref="${1:-origin/main}"
    local changed_files

    if git rev-parse "$base_ref" >/dev/null 2>&1; then
        changed_files=$(git diff --name-only "$base_ref"...HEAD 2>/dev/null || git diff --name-only HEAD)
    else
        changed_files=$(git diff --name-only HEAD)
    fi

    # Get directory names first, then convert to package names
    local crate_dirs
    crate_dirs=$(echo "$changed_files" | grep -E '^crates/[^/]+/' | sed 's|crates/\([^/]*\)/.*|\1|' | sort -u)

    for dir in $crate_dirs; do
        get_package_name "$dir"
    done
}

# Check if infrastructure files changed
infra_changed() {
    local base_ref="${1:-origin/main}"
    local changed_files
    
    if git rev-parse "$base_ref" >/dev/null 2>&1; then
        changed_files=$(git diff --name-only "$base_ref"...HEAD 2>/dev/null || git diff --name-only HEAD)
    else
        changed_files=$(git diff --name-only HEAD)
    fi
    
    echo "$changed_files" | grep -qE '^infra/'
}

# Check if a crate (package) exists in the workspace
crate_exists() {
    local package="$1"
    # Check if any Cargo.toml in crates/ has this package name
    for toml in "$REPO_ROOT"/crates/*/Cargo.toml; do
        if [ -f "$toml" ] && grep -q "^name = \"$package\"" "$toml"; then
            return 0
        fi
    done
    return 1
}

# =============================================================================
# RUST CHECKS
# =============================================================================

run_rust_fmt() {
    local crate="$1"
    log_info "Checking formatting for $crate..."
    
    if cargo fmt -p "$crate" -- --check 2>&1; then
        log_success "Format check passed for $crate"
        return 0
    else
        log_error "Format check failed for $crate"
        FAILURES+=("rust:fmt:$crate")
        return 1
    fi
}

run_rust_clippy() {
    local crate="$1"
    log_info "Running clippy (pedantic) for $crate..."
    
    # shellcheck disable=SC2086
    if cargo clippy -p "$crate" --all-targets -- $CLIPPY_ARGS 2>&1; then
        log_success "Clippy passed for $crate"
        return 0
    else
        log_error "Clippy failed for $crate"
        FAILURES+=("rust:clippy:$crate")
        return 1
    fi
}

run_rust_tests() {
    local crate="$1"
    log_info "Running tests for $crate..."
    
    local test_cmd
    if command -v cargo-nextest &>/dev/null; then
        test_cmd="cargo nextest run -p $crate"
    else
        test_cmd="cargo test -p $crate --all-targets"
    fi
    
    if $test_cmd 2>&1; then
        log_success "Tests passed for $crate"
        return 0
    else
        log_error "Tests failed for $crate"
        FAILURES+=("rust:test:$crate")
        return 1
    fi
}

check_rust_crate() {
    local crate="$1"
    local skip_tests="${2:-false}"
    
    log_section "Rust crate: $crate"
    
    if ! crate_exists "$crate"; then
        log_warning "Crate '$crate' not found, skipping"
        return 0
    fi
    
    run_rust_fmt "$crate" || true
    run_rust_clippy "$crate" || true
    
    if [ "$skip_tests" = "false" ]; then
        run_rust_tests "$crate" || true
    else
        log_info "Skipping tests for $crate (--skip-tests)"
    fi
}

run_all_rust_checks() {
    local check_all="$1"
    local skip_tests="$2"
    shift 2
    local specific_crates=("$@")
    
    log_header "🦀 Rust Checks"
    
    local crates_to_check=()
    
    if [ ${#specific_crates[@]} -gt 0 ]; then
        crates_to_check=("${specific_crates[@]}")
        log_info "Checking specified crates: ${crates_to_check[*]}"
    elif [ "$check_all" = "true" ]; then
        mapfile -t crates_to_check < <(get_all_crates)
        log_info "Checking all crates: ${crates_to_check[*]:-none}"
    else
        mapfile -t crates_to_check < <(get_changed_crates)
        if [ ${#crates_to_check[@]} -eq 0 ]; then
            log_success "No Rust crates changed"
            return 0
        fi
        log_info "Checking changed crates: ${crates_to_check[*]}"
    fi
    
    for crate in "${crates_to_check[@]}"; do
        check_rust_crate "$crate" "$skip_tests"
    done
}

# =============================================================================
# INFRASTRUCTURE CHECKS
# =============================================================================

run_yaml_lint() {
    log_section "YAML Linting"
    
    if ! check_command "yamllint" "Install with: pip install yamllint"; then
        return 0
    fi
    
    log_info "Running yamllint on infra/gitops..."
    
    if yamllint -c "$REPO_ROOT/.yamllint.yaml" "$REPO_ROOT/infra/gitops" 2>&1; then
        log_success "YAML lint passed"
        return 0
    else
        log_error "YAML lint failed"
        FAILURES+=("infra:yamllint")
        return 1
    fi
}

run_helm_lint() {
    log_section "Helm Chart Validation"
    
    if ! check_command "helm" "Install from: https://helm.sh/docs/intro/install/"; then
        return 0
    fi
    
    local charts_dir="$REPO_ROOT/infra/charts"
    local failed=false
    
    # Find all charts
    for chart in "$charts_dir"/*/; do
        if [ -f "$chart/Chart.yaml" ]; then
            local chart_name
            chart_name=$(basename "$chart")
            log_info "Linting Helm chart: $chart_name"
            
            if helm lint "$chart" 2>&1; then
                log_success "Helm lint passed for $chart_name"
            else
                log_error "Helm lint failed for $chart_name"
                FAILURES+=("infra:helm-lint:$chart_name")
                failed=true
            fi
            
            log_info "Testing template rendering for $chart_name..."
            if helm template "$chart_name" "$chart" > /dev/null 2>&1; then
                log_success "Helm template passed for $chart_name"
            else
                log_error "Helm template failed for $chart_name"
                FAILURES+=("infra:helm-template:$chart_name")
                failed=true
            fi
        fi
    done
    
    [ "$failed" = "false" ]
}

run_argocd_validation() {
    log_section "ArgoCD Application Validation"
    
    if ! check_command "yq" "Install with: brew install yq"; then
        return 0
    fi
    
    local apps_dir="$REPO_ROOT/infra/gitops/applications"
    local failed=false
    
    if [ ! -d "$apps_dir" ]; then
        log_info "No ArgoCD applications directory found"
        return 0
    fi
    
    for file in "$apps_dir"/*.yaml; do
        [ -f "$file" ] || continue
        local app_name
        app_name=$(basename "$file" .yaml)
        log_info "Validating ArgoCD app: $app_name"
        
        # Check kind
        local kind
        kind=$(yq eval '.kind' "$file")
        if [ "$kind" != "Application" ]; then
            log_error "$app_name: Not an Application (kind: $kind)"
            FAILURES+=("infra:argocd:$app_name:kind")
            failed=true
            continue
        fi
        
        if [ "$failed" = "false" ]; then
            log_success "ArgoCD app valid: $app_name"
        fi
    done
    
    [ "$failed" = "false" ]
}

run_k8s_schema_validation() {
    log_section "Kubernetes Schema Validation"
    
    if ! check_command "kubeconform" "Install with: brew install kubeconform"; then
        return 0
    fi
    
    log_info "Validating Kubernetes resources..."
    
    # Validate standard K8s resources in gitops
    if [ -d "$REPO_ROOT/infra/gitops" ]; then
        if kubeconform -summary -skip Application,RedisFailover,QuestDB,postgresql,Postgresql \
            -ignore-missing-schemas \
            -ignore-filename-pattern 'values\.yaml' \
            "$REPO_ROOT/infra/gitops" 2>&1; then
            log_success "Kubernetes schema validation passed"
        else
            log_warning "Some Kubernetes resources failed schema validation"
        fi
    fi
}

run_security_checks() {
    log_section "Security Checks"
    
    local failed=false
    
    # Check for hardcoded passwords
    # Excludes: secretKeyRef, CHANGE_THIS, PASSWORD, valueFrom, {{ (Go templates), ${ (shell templates)
    log_info "Checking for hardcoded passwords..."
    if grep -r "password:" "$REPO_ROOT/infra/gitops" --include="*.yaml" 2>/dev/null | \
       grep -v "secretKeyRef" | grep -v "CHANGE_THIS" | grep -v "PASSWORD" | grep -v "valueFrom" | \
       grep -v "{{" | grep -v '\${' | grep -q .; then
        log_error "Found potential hardcoded passwords in infra/gitops"
        FAILURES+=("security:hardcoded-password")
        failed=true
    else
        log_success "No hardcoded passwords found"
    fi
    
    # Run detect-secrets if available
    if check_command "detect-secrets"; then
        log_info "Running detect-secrets..."
        if [ -f "$REPO_ROOT/.secrets.baseline" ]; then
            if detect-secrets scan --baseline "$REPO_ROOT/.secrets.baseline" 2>&1 | grep -q "new secrets detected"; then
                log_error "New secrets detected!"
                FAILURES+=("security:detect-secrets")
                failed=true
            else
                log_success "No new secrets detected"
            fi
        fi
    fi
    
    [ "$failed" = "false" ]
}

run_shellcheck() {
    log_section "Shell Script Validation"
    
    if ! check_command "shellcheck" "Install with: brew install shellcheck"; then
        return 0
    fi
    
    log_info "Running shellcheck on scripts..."
    
    local scripts
    scripts=$(find "$REPO_ROOT/infra/scripts" "$REPO_ROOT/scripts" -name "*.sh" -type f 2>/dev/null || true)
    
    if [ -z "$scripts" ]; then
        log_info "No shell scripts found"
        return 0
    fi
    
    for script in $scripts; do
        local script_name
        script_name=$(basename "$script")
        if shellcheck "$script" 2>&1; then
            log_success "shellcheck passed: $script_name"
        else
            log_warning "shellcheck warnings: $script_name"
        fi
    done
}

run_dockerfile_lint() {
    log_section "Dockerfile Validation"
    
    if ! check_command "hadolint" "Install with: brew install hadolint"; then
        return 0
    fi
    
    log_info "Running hadolint on Dockerfiles..."
    
    local dockerfiles
    dockerfiles=$(find "$REPO_ROOT/infra/images" -name "Dockerfile" -type f 2>/dev/null || true)
    
    if [ -z "$dockerfiles" ]; then
        log_info "No Dockerfiles found"
        return 0
    fi
    
    for dockerfile in $dockerfiles; do
        local df_name
        df_name=$(dirname "$dockerfile" | xargs basename)
        if hadolint "$dockerfile" 2>&1; then
            log_success "hadolint passed: $df_name"
        else
            log_warning "hadolint warnings: $df_name"
        fi
    done
}

run_all_infra_checks() {
    local check_all="$1"
    
    log_header "🏗️  Infrastructure Checks"
    
    # Check if we should run infra checks
    if [ "$check_all" != "true" ]; then
        if ! infra_changed; then
            log_success "No infrastructure files changed"
            return 0
        fi
        log_info "Infrastructure files changed, running checks..."
    fi
    
    run_yaml_lint || true
    run_helm_lint || true
    run_argocd_validation || true
    run_k8s_schema_validation || true
    run_security_checks || true
    run_shellcheck || true
    run_dockerfile_lint || true
}

# =============================================================================
# MAIN
# =============================================================================

print_summary() {
    echo ""
    log_header "📊 Summary"
    
    if [ ${#WARNINGS[@]} -gt 0 ]; then
        echo -e "${YELLOW}Warnings:${NC}"
        for warning in "${WARNINGS[@]}"; do
            echo "  ⚠️  $warning"
        done
        echo ""
    fi
    
    if [ ${#FAILURES[@]} -eq 0 ]; then
        log_success "All checks passed!"
        return 0
    else
        log_error "The following checks failed:"
        for failure in "${FAILURES[@]}"; do
            echo "  ❌ $failure"
        done
        return 1
    fi
}

main() {
    cd "$REPO_ROOT"
    
    # Options
    local check_all=false
    local skip_tests=false
    local rust_only=false
    local infra_only=false
    local specific_crates=()
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --all|-a)
                check_all=true
                shift
                ;;
            --skip-tests|-s)
                skip_tests=true
                shift
                ;;
            --rust|-r)
                rust_only=true
                shift
                ;;
            --infra|-i)
                infra_only=true
                shift
                ;;
            --help|-h)
                cat << 'EOF'
Usage: local-ci.sh [OPTIONS] [CRATES...]

Runs the same CI checks locally that GitHub Actions runs.

OPTIONS:
  --all, -a        Check everything (not just changed files)
  --skip-tests, -s Skip running Rust tests (faster)
  --rust, -r       Run only Rust checks
  --infra, -i      Run only infrastructure checks
  --help, -h       Show this help message

EXAMPLES:
  ./scripts/local-ci.sh                    # Check changed files only
  ./scripts/local-ci.sh --all              # Check everything
  ./scripts/local-ci.sh --rust             # Rust checks only
  ./scripts/local-ci.sh --infra            # Infrastructure checks only
  ./scripts/local-ci.sh controller tools   # Check specific crates
  ./scripts/local-ci.sh --skip-tests       # Skip tests for speed
EOF
                exit 0
                ;;
            -*)
                log_error "Unknown option: $1"
                exit 1
                ;;
            *)
                specific_crates+=("$1")
                shift
                ;;
        esac
    done
    
    echo ""
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║                    🔍 Local CI Checks                        ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    
    # Run checks based on options
    if [ "$infra_only" = "true" ]; then
        run_all_infra_checks "$check_all"
    elif [ "$rust_only" = "true" ]; then
        run_all_rust_checks "$check_all" "$skip_tests" "${specific_crates[@]}"
    else
        # Run both
        run_all_rust_checks "$check_all" "$skip_tests" "${specific_crates[@]}"
        run_all_infra_checks "$check_all"
    fi
    
    # Print summary
    if print_summary; then
        exit 0
    else
        exit 1
    fi
}

main "$@"
















