#!/bin/bash
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/.." && pwd )"
NAMESPACE="agent-platform"
TEST_REPO="cto-parallel-test"
TEST_REPO_ORG="5dlabs"
TEST_REPO_PATH="/Users/jonathonfritz/code/work-projects/5dlabs/cto-parallel-test"
TEMPLATE_REPO_PATH="${PROJECT_ROOT}/testing/cto-parallel-test"
GITHUB_URL="https://github.com/${TEST_REPO_ORG}/${TEST_REPO}"

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

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check required tools
check_prerequisites() {
    print_step "Checking prerequisites..."
    
    local missing_tools=()
    
    if ! command_exists kubectl; then
        missing_tools+=("kubectl")
    fi
    
    if ! command_exists gh; then
        missing_tools+=("gh (GitHub CLI)")
    fi
    
    if ! command_exists argo; then
        print_warning "argo CLI not found - will use kubectl for workflow deletion"
    fi
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        print_error "Missing required tools: ${missing_tools[*]}"
        echo "Please install the missing tools and try again."
        exit 1
    fi
    
    # Check GitHub CLI authentication
    if ! gh auth status >/dev/null 2>&1; then
        print_error "GitHub CLI is not authenticated. Run 'gh auth login' first."
        exit 1
    fi
    
    print_success "All prerequisites met"
}

# Delete all Argo Workflows
delete_workflows() {
    print_step "Deleting all Argo Workflows..."
    
    # Get all workflows
    local workflows=$(kubectl get workflows -n "$NAMESPACE" -o name 2>/dev/null || true)
    
    if [ -z "$workflows" ]; then
        print_warning "No workflows found in namespace $NAMESPACE"
    else
        echo "$workflows" | while IFS= read -r workflow; do
            if [ -n "$workflow" ]; then
                echo "  Deleting: $workflow"
                kubectl delete "$workflow" -n "$NAMESPACE" --force --grace-period=0 2>/dev/null || true
            fi
        done
        print_success "All workflows deleted"
    fi
    
    # Also clean up workflow templates if needed
    print_step "Checking for workflow templates..."
    local templates=$(kubectl get workflowtemplates -n "$NAMESPACE" -o name 2>/dev/null || true)
    if [ -n "$templates" ]; then
        print_warning "Found workflow templates (not deleting, but listing):"
        echo "$templates"
    fi
}

# Delete all pods in agent-platform namespace
delete_pods() {
    print_step "Deleting all pods in namespace $NAMESPACE..."
    
    # Get all pods
    local pods=$(kubectl get pods -n "$NAMESPACE" -o name 2>/dev/null || true)
    
    if [ -z "$pods" ]; then
        print_warning "No pods found in namespace $NAMESPACE"
    else
        echo "$pods" | while IFS= read -r pod; do
            if [ -n "$pod" ]; then
                echo "  Deleting: $pod"
                kubectl delete "$pod" -n "$NAMESPACE" --force --grace-period=0 2>/dev/null || true
            fi
        done
        print_success "All pods deleted"
    fi
}

# Clean up ConfigMaps and other resources
cleanup_resources() {
    print_step "Cleaning up ConfigMaps and other test resources..."
    
    # Delete test-related ConfigMaps (preserve system ones)
    local configmaps=$(kubectl get configmaps -n "$NAMESPACE" -o name | grep -E "play-|test-|coderun-|docsrun-" 2>/dev/null || true)
    
    if [ -n "$configmaps" ]; then
        echo "$configmaps" | while IFS= read -r cm; do
            if [ -n "$cm" ]; then
                echo "  Deleting ConfigMap: $cm"
                kubectl delete "$cm" -n "$NAMESPACE" --force --grace-period=0 2>/dev/null || true
            fi
        done
        print_success "Test ConfigMaps cleaned up"
    else
        print_warning "No test ConfigMaps found"
    fi
    
    # Clean up any test PVCs
    local pvcs=$(kubectl get pvc -n "$NAMESPACE" -o name | grep -E "workspace-play-|workspace-test-" 2>/dev/null || true)
    
    if [ -n "$pvcs" ]; then
        echo "$pvcs" | while IFS= read -r pvc; do
            if [ -n "$pvc" ]; then
                echo "  Deleting PVC: $pvc"
                kubectl delete "$pvc" -n "$NAMESPACE" --force --grace-period=0 2>/dev/null || true
            fi
        done
        print_success "Test PVCs cleaned up"
    fi
}

# Initialize/update submodule template repository
ensure_submodule_ready() {
    print_step "Checking submodule template repository..."
    
    # Ensure testing directory exists
    mkdir -p "$(dirname "$TEMPLATE_REPO_PATH")"
    
    # Check if submodule is initialized
    if [ ! -d "$TEMPLATE_REPO_PATH" ] || [ ! -f "$TEMPLATE_REPO_PATH/.git" ]; then
        print_step "Initializing submodule..."
        cd "$PROJECT_ROOT"
        
        # Initialize submodule if .gitmodules exists
        if [ -f ".gitmodules" ] && grep -q "testing/cto-parallel-test" .gitmodules 2>/dev/null; then
            git submodule update --init --recursive testing/cto-parallel-test 2>/dev/null || {
                print_warning "Submodule initialization failed. Will clone directly."
                # Fallback: clone directly if submodule fails
                if [ ! -d "$TEMPLATE_REPO_PATH" ]; then
                    git clone "$GITHUB_URL" "$TEMPLATE_REPO_PATH" 2>/dev/null || {
                        print_error "Failed to clone template repository"
                        return 1
                    }
                fi
            }
        else
            # Submodule not configured, clone directly
            if [ ! -d "$TEMPLATE_REPO_PATH" ]; then
                print_step "Cloning template repository..."
                git clone "$GITHUB_URL" "$TEMPLATE_REPO_PATH" || {
                    print_error "Failed to clone template repository"
                    return 1
                }
            fi
        fi
        
        print_success "Template repository ready at $TEMPLATE_REPO_PATH"
    else
        print_step "Updating submodule to latest..."
        cd "$PROJECT_ROOT"
        git submodule update --remote testing/cto-parallel-test 2>/dev/null || true
        print_success "Template repository ready at $TEMPLATE_REPO_PATH"
    fi
}

# Delete GitHub repository
delete_github_repo() {
    print_step "Deleting GitHub repository ${TEST_REPO_ORG}/${TEST_REPO}..."
    
    # Check if repo exists
    if gh repo view "${TEST_REPO_ORG}/${TEST_REPO}" >/dev/null 2>&1; then
        print_warning "About to delete repository: ${GITHUB_URL}"
        read -p "Are you sure you want to delete this repository? (y/N): " -n 1 -r
        echo
        
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            gh repo delete "${TEST_REPO_ORG}/${TEST_REPO}" --yes
            print_success "GitHub repository deleted"
        else
            print_warning "Skipping GitHub repository deletion"
            return 1
        fi
    else
        print_warning "GitHub repository ${TEST_REPO_ORG}/${TEST_REPO} does not exist"
    fi
    
    # Remove local repository
    if [ -d "$TEST_REPO_PATH" ]; then
        print_step "Removing local repository..."
        rm -rf "$TEST_REPO_PATH"
        print_success "Local repository removed"
    fi
    
    return 0
}

# Recreate GitHub repository
recreate_github_repo() {
    print_step "Recreating GitHub repository ${TEST_REPO_ORG}/${TEST_REPO}..."
    
    # Create new repository
    gh repo create "${TEST_REPO_ORG}/${TEST_REPO}" --private --clone=false || {
        print_error "Failed to create repository. It may already exist."
        return 1
    }
    
    print_success "GitHub repository created"
    
    # Restore from template if available
    if [ -d "$TEMPLATE_REPO_PATH" ]; then
        print_step "Restoring repository from submodule template..."
        
        # Copy template to test repo location, excluding .git (submodule file)
        rsync -av --exclude='.git' "$TEMPLATE_REPO_PATH/" "$TEST_REPO_PATH/" || \
            cp -r "$TEMPLATE_REPO_PATH"/* "$TEST_REPO_PATH/" 2>/dev/null || true
        
        # Also copy hidden files (except .git)
        if [ -d "$TEMPLATE_REPO_PATH" ]; then
            find "$TEMPLATE_REPO_PATH" -maxdepth 1 -name ".*" ! -name ".git" ! -name "." ! -name ".." -exec cp -r {} "$TEST_REPO_PATH/" \; 2>/dev/null || true
        fi
        
        # Initialize git and push to GitHub
        cd "$TEST_REPO_PATH"
        git init
        git add .
        git commit -m "Initial commit - E2E test reset" || {
            print_warning "No changes to commit (repository may be empty)"
        }
        git branch -M main
        git remote add origin "git@github.com:${TEST_REPO_ORG}/${TEST_REPO}.git" 2>/dev/null || \
            git remote set-url origin "git@github.com:${TEST_REPO_ORG}/${TEST_REPO}.git"
        git push -u origin main --force
        
        print_success "Repository restored from submodule template and pushed to GitHub"
    else
        print_step "Cloning empty repository..."
        git clone "git@github.com:${TEST_REPO_ORG}/${TEST_REPO}.git" "$TEST_REPO_PATH"
        
        # Add minimal structure
        cd "$TEST_REPO_PATH"
        mkdir -p docs/.taskmaster/docs
        
        # Create minimal cto-config.json
        cat > cto-config.json <<'EOF'
{
  "version": "1.0.0",
  "project": "cto-parallel-test",
  "description": "Test repository for CTO platform E2E testing"
}
EOF
        
        # Create a simple PRD for testing
        cat > docs/.taskmaster/docs/simple-prd.txt <<'EOF'
# Simple Test PRD

Build a minimal test application with the following requirements:

1. Create a simple "Hello World" HTTP server
2. Add basic health check endpoint
3. Include simple unit tests
4. Add basic documentation

Technical Requirements:
- Use Go or Rust
- Include Dockerfile
- Add GitHub Actions CI/CD

This is a minimal PRD designed for quick E2E testing of the CTO platform.
EOF
        
        git add .
        git commit -m "Initial commit - Minimal E2E test structure"
        git push -u origin main
        
        print_success "Minimal test repository created and pushed"
    fi
    
    cd - >/dev/null
}

# Show summary
show_summary() {
    echo
    print_step "Environment Reset Summary"
    echo "=========================="
    echo "✓ Deleted all Argo Workflows in $NAMESPACE"
    echo "✓ Deleted all pods in $NAMESPACE"
    echo "✓ Cleaned up test ConfigMaps and PVCs"
    echo "✓ Deleted and recreated GitHub repository: ${GITHUB_URL}"
    echo "✓ Test repository ready at: $TEST_REPO_PATH"
    
    if [ -d "$TEMPLATE_REPO_PATH" ]; then
        echo "✓ Template preserved at: $TEMPLATE_REPO_PATH"
    fi
    
    echo
    print_success "E2E environment successfully reset!"
    echo
    echo "Next steps:"
    echo "1. Update any CTO configuration to point to the fresh repository"
    echo "2. Run your E2E tests with: cto play --task-id <task-id>"
    echo "3. Monitor with: kubectl logs -f -l workflow -n agent-platform"
}

# Main execution
main() {
    echo "==================================="
    echo "  E2E Environment Reset Script"
    echo "==================================="
    echo
    
    check_prerequisites
    
    # Save template before deletion (if first run)
    ensure_submodule_ready
    
    # Kubernetes cleanup (do this first)
    delete_workflows
    delete_pods
    cleanup_resources
    
    # GitHub repository reset (always do this last)
    delete_github_repo
    recreate_github_repo
    
    # Show summary
    show_summary
}

# Handle script interruption
trap 'print_error "Script interrupted"; exit 1' INT TERM

# Run main function
main "$@"



