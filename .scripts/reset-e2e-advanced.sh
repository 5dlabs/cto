#!/bin/bash
set -euo pipefail

# Advanced E2E Environment Reset Script
# This script supports multiple repository initialization strategies (template, minimal)
# Templates are regular directories (NOT git submodules) to avoid crosstalk issues

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "${SCRIPT_DIR}/.." && pwd )"
CONFIG_FILE="${SCRIPT_DIR}/e2e-reset-config.yaml"

# Default configuration
NAMESPACE="${NAMESPACE:-cto}"
TEST_REPO="${TEST_REPO:-cto-parallel-test}"
TEST_REPO_ORG="${TEST_REPO_ORG:-5dlabs}"
REPO_STRATEGY="${REPO_STRATEGY:-template}"

# Function to print colored messages
print_step() {
    echo -e "${BLUE}==>${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_info() {
    echo -e "${MAGENTA}ℹ${NC} $1"
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --strategy)
                REPO_STRATEGY="$2"
                shift 2
                ;;
            --skip-k8s)
                SKIP_K8S=true
                shift
                ;;
            --skip-github)
                SKIP_GITHUB=true
                shift
                ;;
            --force)
                FORCE=true
                shift
                ;;
            --help)
                show_help
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
}

# Show help message
show_help() {
    cat << EOF
Usage: $0 [OPTIONS]

Reset E2E testing environment for CTO platform

OPTIONS:
    --strategy <type>   Repository initialization strategy (template|minimal)
                       Default: template
    --skip-k8s         Skip Kubernetes resource cleanup
    --skip-github      Skip GitHub repository operations
    --force            Skip confirmation prompts
    --help             Show this help message

STRATEGIES:
    template   - Use a local template directory (fastest, recommended)
                 Template is a regular directory, NOT a git submodule
    minimal    - Create minimal structure from scratch (smallest)

CONFIG:
    Configuration is read from: $CONFIG_FILE

EXAMPLES:
    # Full reset with template strategy
    $0

    # Use minimal strategy
    $0 --strategy minimal

    # Only reset Kubernetes resources
    $0 --skip-github

    # Force reset without confirmations
    $0 --force
EOF
}

# Note: init_with_submodule removed - we no longer use git submodules
# to avoid crosstalk issues. Use template or minimal strategies instead.

# Initialize repository with template strategy
# Workflow: init git in template dir -> push to GitHub -> delete template -> clone to test location
init_with_template() {
    local repo_path="$1"
    local template_path="$2"
    
    print_step "Initializing repository with template strategy..."
    
    if [ ! -d "$template_path" ]; then
        print_error "Template directory not found: $template_path"
        print_info "Falling back to minimal strategy"
        init_with_minimal "$repo_path"
        return
    fi
    
    # Step 1: Initialize git in the template directory itself
    print_info "Initializing git in template directory..."
    cd "$template_path"
    
    # Remove any existing .git directory first
    rm -rf .git
    
    # Initialize git
    git init
    git branch -M main
    git add .
    git commit -m "Initial commit - E2E test from template"
    
    # Add remote and push
    git remote add origin "git@github.com:${TEST_REPO_ORG}/${TEST_REPO}.git" 2>/dev/null || \
        git remote set-url origin "git@github.com:${TEST_REPO_ORG}/${TEST_REPO}.git"
    
    print_info "Pushing template to GitHub..."
    git push -u origin main --force
    
    print_success "Template pushed to GitHub"
    
    # Step 2: Delete the template directory (it's ephemeral)
    print_info "Cleaning up template directory..."
    cd "$PROJECT_ROOT"
    rm -rf "$template_path"
    
    # Step 3: Clone from GitHub to the actual test location
    print_info "Cloning from GitHub to test location: $repo_path"
    rm -rf "$repo_path"
    git clone "git@github.com:${TEST_REPO_ORG}/${TEST_REPO}.git" "$repo_path"
    
    print_success "Repository initialized from template and cloned to test location"
}

# Initialize repository with minimal strategy
init_with_minimal() {
    local repo_path="$1"
    
    print_step "Initializing repository with minimal strategy..."
    
    cd "$repo_path"
    
    # Create directory structure
    mkdir -p docs/.tasks/docs
    mkdir -p src
    mkdir -p tests
    
    # Create cto-config.json
    cat > cto-config.json <<'EOF'
{
  "version": "1.0.0",
  "project": "cto-parallel-test",
  "description": "Test repository for CTO platform E2E testing",
  "github": {
    "org": "5dlabs",
    "repo": "cto-parallel-test"
  }
}
EOF
    
    # Create a comprehensive test PRD
    cat > docs/.tasks/docs/test-prd.txt <<'EOF'
# CTO Platform E2E Test PRD

## Project Overview
Build a test application to validate the CTO platform's multi-agent workflow capabilities.

## Core Requirements

### 1. Web Application
- Create a simple REST API server
- Implement CRUD operations for a "Task" entity
- Add authentication using JWT tokens
- Include OpenAPI/Swagger documentation

### 2. Database Integration
- Use PostgreSQL for data persistence
- Implement database migrations
- Add connection pooling
- Include transaction support

### 3. Testing Suite
- Unit tests with >80% coverage
- Integration tests for API endpoints
- End-to-end tests for critical user flows
- Performance benchmarks for key operations

### 4. DevOps & Deployment
- Multi-stage Dockerfile
- Kubernetes manifests (Deployment, Service, Ingress)
- GitHub Actions CI/CD pipeline
- Helm chart for deployment

### 5. Monitoring & Observability
- Structured logging with correlation IDs
- Prometheus metrics exposition
- Health and readiness endpoints
- Distributed tracing with OpenTelemetry

### 6. Security
- Input validation and sanitization
- Rate limiting
- CORS configuration
- Security headers (HSTS, CSP, etc.)

## Technical Stack
- Language: Go or Rust (prefer Rust for memory safety)
- Framework: Actix-web (Rust) or Gin (Go)
- Database: PostgreSQL 15+
- Cache: Redis
- Container: Docker with multi-stage builds
- Orchestration: Kubernetes 1.28+

## Success Criteria
1. All agents (Rex, Cleo, Tess) successfully complete their stages
2. Code passes all quality gates
3. Application deploys successfully to Kubernetes
4. All tests pass in CI/CD pipeline
5. Documentation is complete and accurate

## Timeline
This is designed to be completed in a single CTO platform run, testing:
- Rex: Implementation of all features
- Cleo: Code quality, linting, formatting, unit tests
- Tess: End-to-end testing in live Kubernetes environment
EOF

    # Create README
    cat > README.md <<'EOF'
# CTO Parallel Test Repository

This repository is used for end-to-end testing of the CTO multi-agent platform.

## Purpose

- Validate multi-agent workflow orchestration
- Test Rex → Cleo → Tess pipeline
- Verify quality gates and automated reviews
- Ensure proper Kubernetes deployment and testing

## Structure

```
.
├── docs/
│   └── .tasks/
│       └── docs/
│           └── test-prd.txt    # Product Requirements Document
├── src/                         # Source code (generated by Rex)
├── tests/                       # Test suites (validated by Cleo/Tess)
├── k8s/                        # Kubernetes manifests
├── .github/                    # GitHub Actions workflows
└── cto-config.json             # CTO configuration
```

## Usage

This repository is automatically managed by the CTO platform during E2E testing.

To trigger a test:
```bash
cto play --task-id <task-id>
```

## Reset

To reset this repository for a fresh test:
```bash
./scripts/reset-e2e-environment.sh
```
EOF
    
    # Create .gitignore
    cat > .gitignore <<'EOF'
# Build artifacts
target/
dist/
*.exe
*.dll
*.so
*.dylib

# IDE
.idea/
.vscode/
*.swp
*.swo
*~

# Dependencies
node_modules/
vendor/

# Environment
.env
.env.local

# Logs
*.log
logs/

# Test coverage
coverage/
*.coverage
.coverage

# OS
.DS_Store
Thumbs.db

# Kubernetes
*.kubeconfig
EOF
    
    git init
    git branch -M main
    git add .
    git commit -m "Initial commit - Minimal E2E test structure"
    
    print_success "Minimal repository structure created"
}

# Main repository recreation logic
recreate_repository() {
    local repo_path="${TEST_REPO_PATH:-/Users/jonathonfritz/code/work-projects/5dlabs/cto-parallel-test}"
    local template_path="${TEMPLATE_REPO_PATH:-${PROJECT_ROOT}/testing/cto-parallel-test}"
    
    print_step "Recreating repository with strategy: $REPO_STRATEGY"
    
    # Create GitHub repository
    if [ "${SKIP_GITHUB:-false}" != "true" ]; then
        print_step "Creating GitHub repository ${TEST_REPO_ORG}/${TEST_REPO}..."
        
        # Delete if exists and force mode
        if gh repo view "${TEST_REPO_ORG}/${TEST_REPO}" >/dev/null 2>&1; then
            if [ "${FORCE:-false}" = "true" ] || confirm "Delete existing repository?"; then
                gh repo delete "${TEST_REPO_ORG}/${TEST_REPO}" --yes
                print_success "Existing repository deleted"
            else
                print_warning "Keeping existing repository"
                return 1
            fi
        fi
        
        # Create new repository
        local visibility="${REPO_VISIBILITY:-private}"
        gh repo create "${TEST_REPO_ORG}/${TEST_REPO}" \
            "--${visibility}" \
            --description "E2E test repository for CTO platform" \
            --clone=false
        
        print_success "GitHub repository created"
    fi
    
    # Prepare local repository
    rm -rf "$repo_path"
    mkdir -p "$repo_path"
    
    # Initialize based on strategy
    case "$REPO_STRATEGY" in
        template)
            init_with_template "$repo_path" "$template_path"
            ;;
        minimal)
            init_with_minimal "$repo_path"
            ;;
        *)
            print_error "Unknown strategy: $REPO_STRATEGY (valid options: template, minimal)"
            exit 1
            ;;
    esac
    
    # Push to GitHub
    if [ "${SKIP_GITHUB:-false}" != "true" ]; then
        cd "$repo_path"
        git remote add origin "git@github.com:${TEST_REPO_ORG}/${TEST_REPO}.git" 2>/dev/null || true
        git push -u origin main --force
        print_success "Repository pushed to GitHub"
    fi
    
    cd - >/dev/null
}

# Confirmation prompt
confirm() {
    if [ "${FORCE:-false}" = "true" ]; then
        return 0
    fi
    
    local prompt="${1:-Continue?}"
    read -p "$prompt (y/N): " -n 1 -r
    echo
    [[ $REPLY =~ ^[Yy]$ ]]
}

# Kubernetes cleanup function
cleanup_kubernetes() {
    if [ "${SKIP_K8S:-false}" = "true" ]; then
        print_info "Skipping Kubernetes cleanup (--skip-k8s)"
        return
    fi
    
    print_step "Cleaning up Kubernetes resources in namespace: $NAMESPACE"
    
    # Delete workflows
    print_info "Deleting Argo Workflows..."
    kubectl delete workflows --all -n "$NAMESPACE" --force --grace-period=0 2>/dev/null || true
    
    # Delete pods
    print_info "Deleting all pods..."
    kubectl delete pods --all -n "$NAMESPACE" --force --grace-period=0 2>/dev/null || true
    
    # Delete test ConfigMaps
    print_info "Deleting test ConfigMaps..."
    kubectl delete configmaps -n "$NAMESPACE" --force --grace-period=0 \
        -l "app.kubernetes.io/part-of=cto-test" 2>/dev/null || true
    
    # Alternative: Delete by name pattern
    for pattern in "play-" "test-" "coderun-" "docsrun-"; do
        kubectl get configmaps -n "$NAMESPACE" -o name | grep "^configmap/${pattern}" | \
            xargs -r kubectl delete -n "$NAMESPACE" --force --grace-period=0 2>/dev/null || true
    done
    
    # Delete test PVCs
    print_info "Deleting test PVCs..."
    for pattern in "workspace-play-" "workspace-test-"; do
        kubectl get pvc -n "$NAMESPACE" -o name | grep "^persistentvolumeclaim/${pattern}" | \
            xargs -r kubectl delete -n "$NAMESPACE" --force --grace-period=0 2>/dev/null || true
    done
    
    print_success "Kubernetes cleanup complete"
}

# Main execution
main() {
    echo "============================================"
    echo "  Advanced E2E Environment Reset Script"
    echo "============================================"
    echo
    
    parse_args "$@"
    
    print_info "Strategy: $REPO_STRATEGY"
    print_info "Namespace: $NAMESPACE"
    print_info "Repository: ${TEST_REPO_ORG}/${TEST_REPO}"
    echo
    
    if ! confirm "Proceed with environment reset?"; then
        print_warning "Aborted by user"
        exit 0
    fi
    
    # Perform cleanup
    cleanup_kubernetes
    
    # Recreate repository
    recreate_repository
    
    # Summary
    echo
    print_success "E2E environment reset complete!"
    echo
    echo "Repository strategies used:"
    echo "  • Strategy: $REPO_STRATEGY"
    echo
    echo "Next steps:"
    echo "1. Trigger test: cto play --task-id <task-id>"
    echo "2. Monitor: kubectl logs -f -l workflow -n cto"
    echo "3. Check GitHub: https://github.com/${TEST_REPO_ORG}/${TEST_REPO}"
}

# Handle script interruption
trap 'print_error "Script interrupted"; exit 1' INT TERM

# Run main function
main "$@"
