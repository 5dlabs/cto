#!/bin/bash
# Fully automated batch setup script for multiple GitHub agents
# Only requires GitHub PAT tokens - SSH keys are auto-generated and added to GitHub

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PLATFORM_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
AGENTS_DIR="$PLATFORM_ROOT/agents"
NAMESPACE="cto"
DRY_RUN=""
VERBOSE=""
AUTO_GENERATE=""

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

usage() {
    cat << EOF
Fully automated GitHub agent setup for 5D Labs Platform

USAGE:
    $0 [OPTIONS]

OPTIONS:
    --auto-generate         Generate SSH keys AND auto-add to GitHub via API (RECOMMENDED)
    --namespace <name>      Kubernetes namespace (default: orchestrator)
    --dry-run              Show commands without executing
    --verbose              Show detailed output
    --help                 Show this help message

DIRECTORY STRUCTURE:
    agents/
    ├── agent-name-1/
    │   └── .env            # TOKEN=ghp_xxxxx (only this required!)
    └── agent-name-2/
        └── .env            # TOKEN=ghp_xxxxx (only this required!)

AUTOMATED WORKFLOW:
    1. Create agent directories with .env files containing GitHub PAT tokens
    2. Run: $0 --auto-generate
    3. Script automatically:
       ✓ Generates SSH key pairs for each agent
       ✓ Adds public keys to GitHub accounts via API
       ✓ Creates Kubernetes secrets
    4. You're done! No manual steps needed.

GITHUB PAT REQUIREMENTS:
    For Classic PATs: 'write:gpg_key' scope
    For Fine-grained PATs: 'Git SSH keys' user permissions (write)

EXAMPLES:
    # Fully automated setup (recommended)
    $0 --auto-generate

    # Dry run to see what would happen
    $0 --auto-generate --dry-run

    # Setup in different namespace with verbose output
    $0 --auto-generate --namespace my-orchestrator --verbose

    # Create agent structure quickly
    for agent in pm0-5dlabs qa0-5dlabs swe-1-5dlabs swe-2-5dlabs SWE-2-5dlabs; do
        mkdir -p agents/\$agent
        echo "TOKEN=ghp_YOUR_TOKEN_HERE" > agents/\$agent/.env
    done

EOF
}

log() {
    if [[ -n "$VERBOSE" ]]; then
        echo -e "${BLUE}[DEBUG]${NC} $*" >&2
    fi
}

info() {
    echo -e "${GREEN}[INFO]${NC} $*"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $*" >&2
}

error() {
    echo -e "${RED}[ERROR]${NC} $*" >&2
}

# Function to add SSH key to GitHub via API
add_ssh_key_to_github() {
    local agent_name="$1"
    local token="$2"
    local public_key_file="$3"

    local title="5D Labs Platform Agent: $agent_name"

    log "Adding SSH key to GitHub for agent: $agent_name"

    if [[ -n "$DRY_RUN" ]]; then
        echo "DRY RUN: Would add SSH key to GitHub via API"
        echo "  Title: $title"
        echo "  Key: ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAASIMULATED... ${agent_name}@5dlabs.platform"
        return 0
    fi

    if [[ ! -f "$public_key_file" ]]; then
        error "Public key file not found: $public_key_file"
        return 1
    fi

    local public_key_content
    public_key_content=$(cat "$public_key_file")

    # GitHub API call to add SSH key
    local response
    local http_code

    response=$(curl -s -w "\n%{http_code}" \
        -X POST \
        -H "Accept: application/vnd.github+json" \
        -H "Authorization: Bearer $token" \
        -H "X-GitHub-Api-Version: 2022-11-28" \
        https://api.github.com/user/keys \
        -d "{\"title\":\"$title\",\"key\":\"$public_key_content\"}")

    http_code=$(echo "$response" | tail -n1)
    response_body=$(echo "$response" | head -n -1)

    log "GitHub API response code: $http_code"
    log "GitHub API response: $response_body"

    case $http_code in
        201)
            info "✅ Successfully added SSH key to GitHub for: $agent_name"
            return 0
            ;;
        422)
            if echo "$response_body" | grep -q "key is already in use"; then
                warn "SSH key already exists in GitHub for: $agent_name (skipping)"
                return 0
            else
                error "GitHub API validation error for $agent_name: $response_body"
                return 1
            fi
            ;;
        401)
            error "GitHub API authentication failed for $agent_name. Check your PAT token and scopes."
            error "Required scopes: Classic PAT needs 'write:gpg_key', Fine-grained PAT needs 'Git SSH keys' write permission"
            return 1
            ;;
        403)
            error "GitHub API forbidden for $agent_name. Check your PAT token permissions."
            return 1
            ;;
        *)
            error "GitHub API error for $agent_name (HTTP $http_code): $response_body"
            return 1
            ;;
    esac
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --namespace)
            NAMESPACE="$2"
            shift 2
            ;;
        --auto-generate)
            AUTO_GENERATE="true"
            shift
            ;;
        --dry-run)
            DRY_RUN="true"
            shift
            ;;
        --verbose)
            VERBOSE="true"
            shift
            ;;
        --help)
            usage
            exit 0
            ;;
        *)
            error "Unknown option: $1"
            echo "Use --help for usage information" >&2
            exit 1
            ;;
    esac
done

# Check if --auto-generate was provided
if [[ -z "$AUTO_GENERATE" ]]; then
    error "Please use --auto-generate for the new fully automated workflow"
    echo ""
    usage
    exit 1
fi

# Check if agents directory exists
if [[ ! -d "$AGENTS_DIR" ]]; then
    error "Agents directory not found: $AGENTS_DIR"
    echo "Please create the agents/ directory structure:" >&2
    echo "  mkdir -p agents/{pm0-5dlabs,qa0-5dlabs,swe-1-5dlabs,swe-2-5dlabs,SWE-2-5dlabs}" >&2
    echo "  echo 'TOKEN=ghp_YOUR_TOKEN_HERE' > agents/pm0-5dlabs/.env" >&2
    echo "  # ... repeat for each agent" >&2
    exit 1
fi

# Check required tools
for tool in kubectl ssh-keygen curl; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        error "$tool is not installed or not in PATH"
        exit 1
    fi
done

# Check if namespace exists (unless dry run)
if [[ -z "$DRY_RUN" ]]; then
    if ! kubectl get namespace "$NAMESPACE" >/dev/null 2>&1; then
        warn "Namespace '$NAMESPACE' does not exist"
        info "Creating namespace: $NAMESPACE"
        kubectl create namespace "$NAMESPACE"
    fi
fi

info "🚀 Starting fully automated agent setup"
info "   Agents directory: $AGENTS_DIR"
info "   Namespace: $NAMESPACE"
info "   Mode: Auto-generate SSH keys + Auto-add to GitHub"
if [[ -n "$DRY_RUN" ]]; then
    info "   Mode: DRY RUN (no changes will be made)"
fi
echo ""

# Find all agent directories
AGENT_COUNT=0
PROCESSED_COUNT=0
FAILED_COUNT=0
API_ADDED_KEYS=()

# Count agents first
for agent_dir in "$AGENTS_DIR"/*; do
    if [[ -d "$agent_dir" ]]; then
        AGENT_COUNT=$((AGENT_COUNT + 1))
    fi
done

if [[ $AGENT_COUNT -eq 0 ]]; then
    warn "No agent directories found in $AGENTS_DIR"
    echo "Please create agent directories with .env files containing GitHub PAT tokens"
    exit 1
fi

info "📋 Found $AGENT_COUNT agent(s) to process"
echo ""

# Process each agent directory
for agent_dir in "$AGENTS_DIR"/*; do
    if [[ ! -d "$agent_dir" ]]; then
        continue
    fi

    agent_name=$(basename "$agent_dir")
    log "Processing agent directory: $agent_dir"

    # Check for .env file (required)
    env_file="$agent_dir/.env"
    if [[ ! -f "$env_file" ]]; then
        error "Agent '$agent_name': Missing .env file with GitHub PAT token"
        FAILED_COUNT=$((FAILED_COUNT + 1))
        continue
    fi

    # Load token from .env file
    if ! source "$env_file"; then
        error "Agent '$agent_name': Failed to load .env file"
        FAILED_COUNT=$((FAILED_COUNT + 1))
        continue
    fi

    if [[ -z "${TOKEN:-}" ]]; then
        error "Agent '$agent_name': TOKEN not found in .env file"
        FAILED_COUNT=$((FAILED_COUNT + 1))
        continue
    fi

    # Validate token format
    if [[ ! "$TOKEN" =~ ^ghp_[a-zA-Z0-9_]{36}$ ]]; then
        warn "Agent '$agent_name': Token doesn't match expected format (ghp_xxxxx)"
    fi

    # SSH key paths
    ssh_private="$agent_dir/id_ed25519"
    ssh_public="$agent_dir/id_ed25519.pub"

    # Generate SSH keys
    info "🔑 Generating SSH key pair for agent: $agent_name"

    if [[ -z "$DRY_RUN" ]]; then
        # Generate SSH key pair
        ssh-keygen -t ed25519 -f "$ssh_private" -N "" -C "${agent_name}@5dlabs.platform" -q

        if [[ -f "$ssh_private" && -f "$ssh_public" ]]; then
            # Set proper permissions
            chmod 600 "$ssh_private"
            chmod 644 "$ssh_public"
            info "✅ Generated SSH key pair for: $agent_name"
        else
            error "Failed to generate SSH key pair for: $agent_name"
            FAILED_COUNT=$((FAILED_COUNT + 1))
            continue
        fi
    else
        echo "DRY RUN: ssh-keygen -t ed25519 -f $ssh_private -N \"\" -C \"${agent_name}@5dlabs.platform\" -q"
    fi

    # Add SSH key to GitHub via API
    info "🌐 Adding SSH key to GitHub for agent: $agent_name"

    if add_ssh_key_to_github "$agent_name" "$TOKEN" "$ssh_public"; then
        API_ADDED_KEYS+=("$agent_name")
        info "✅ SSH key added to GitHub for: $agent_name"
    else
        error "❌ Failed to add SSH key to GitHub for: $agent_name"
        FAILED_COUNT=$((FAILED_COUNT + 1))
        continue
    fi

    # Create Kubernetes secrets
    info "🔧 Setting up Kubernetes secrets for agent: $agent_name"

    # Call the setup-agent-secrets.sh script
    setup_cmd=(
        "$SCRIPT_DIR/setup-agent-secrets.sh"
        "--user" "$agent_name"
        "--ssh-key" "$ssh_private"
        "--token" "$TOKEN"
        "--namespace" "$NAMESPACE"
    )

    if [[ -n "$DRY_RUN" ]]; then
        setup_cmd+=("--dry-run")
    fi

    if [[ -n "$VERBOSE" ]]; then
        setup_cmd+=("--verbose")
    fi

    log "Executing: ${setup_cmd[*]}"

    if "${setup_cmd[@]}"; then
        info "✅ Successfully processed agent: $agent_name"
        PROCESSED_COUNT=$((PROCESSED_COUNT + 1))
    else
        error "❌ Failed to process agent: $agent_name"
        FAILED_COUNT=$((FAILED_COUNT + 1))
    fi

    echo ""
done

# Display summary of API actions
if [[ ${#API_ADDED_KEYS[@]} -gt 0 ]]; then
    echo "════════════════════════════════════════════════════════════════"
    info "🌐 SSH Keys Successfully Added to GitHub:"
    echo ""

    for agent_name in "${API_ADDED_KEYS[@]}"; do
        echo -e "${GREEN}   ✅ ${agent_name}${NC} - SSH key active in GitHub account"
    done
    echo ""

    info "🎯 All SSH keys are now configured and ready to use!"
    echo ""
fi

# Summary
echo "════════════════════════════════════════════════════════════════"
info "🎉 Fully automated agent setup completed!"
echo ""
info "📊 Summary:"
info "   Total agents found: $AGENT_COUNT"
info "   Successfully processed: $PROCESSED_COUNT"
if [[ $FAILED_COUNT -gt 0 ]]; then
    warn "   Failed: $FAILED_COUNT"
else
    info "   Failed: $FAILED_COUNT"
fi
info "   SSH keys added to GitHub: ${#API_ADDED_KEYS[@]}"
echo ""

if [[ $FAILED_COUNT -eq 0 ]]; then
    info "✅ All agents processed successfully!"
    if [[ -z "$DRY_RUN" ]]; then
        echo ""
        info "🔍 Verify secrets:"
        echo "   kubectl get secrets -n $NAMESPACE | grep github"
        echo ""
        info "🚀 Next steps:"
        echo "   1. Install Helm chart: helm install orchestrator ./infra/charts/orchestrator"
        echo "   2. Create your first task with any of these agents:"
        for agent_dir in "$AGENTS_DIR"/*; do
            if [[ -d "$agent_dir" ]]; then
                agent_name=$(basename "$agent_dir")
                echo "      - githubUser: \"$agent_name\""
            fi
        done
        echo ""
        info "🎯 No manual steps required - everything is automated!"
    else
        info "🏃 Remove --dry-run to execute these commands"
    fi
else
    error "❌ Some agents failed to process. Check the errors above."
    exit 1
fi