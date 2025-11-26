#!/bin/bash
# Setup GitHub agent secrets for 5D Labs Platform
# This script creates the SSH and GitHub token secrets that the orchestrator expects

set -euo pipefail

# Default values
NAMESPACE="cto"
DRY_RUN=""
VERBOSE=""

# Function to show usage
usage() {
    cat << EOF
Setup GitHub agent secrets for 5D Labs Platform

USAGE:
    $0 --user <github-user> --ssh-key <path> --token <token> [OPTIONS]

REQUIRED:
    --user <username>       GitHub username (e.g., 'johnsmith')
    --ssh-key <path>        Path to SSH private key (e.g., '~/.ssh/github_key')
    --token <token>         GitHub personal access token (ghp_xxxx)

OPTIONS:
    --namespace <name>      Kubernetes namespace (default: cto)
    --dry-run              Show commands without executing
    --verbose              Show detailed output
    --help                 Show this help message

EXAMPLES:
    # Setup secrets for user 'johnsmith'
    $0 --user johnsmith --ssh-key ~/.ssh/github_johnsmith --token ghp_abc123

    # Dry run to see what would be created
    $0 --user alice --ssh-key ~/.ssh/alice_github --token ghp_xyz789 --dry-run

    # Setup in different namespace
    $0 --user bob --ssh-key ~/.ssh/bob --token ghp_def456 --namespace my-orchestrator

NOTES:
    - SSH key path should point to the PRIVATE key (public key will be derived)
    - GitHub token needs 'repo' permissions for PR creation
    - Secrets will be named: github-ssh-<user> and github-token-<user>
    - Existing secrets will be replaced without warning

EOF
}

# Function for verbose logging
log() {
    if [[ -n "$VERBOSE" ]]; then
        echo "🔧 $*" >&2
    fi
}

# Function to execute or show commands
execute() {
    if [[ -n "$DRY_RUN" ]]; then
        echo "DRY RUN: $*"
    else
        log "Executing: $*"
        eval "$@"
    fi
}

# Parse command line arguments
GITHUB_USER=""
SSH_KEY_PATH=""
GITHUB_TOKEN=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --user)
            GITHUB_USER="$2"
            shift 2
            ;;
        --ssh-key)
            SSH_KEY_PATH="$2"
            shift 2
            ;;
        --token)
            GITHUB_TOKEN="$2"
            shift 2
            ;;
        --namespace)
            NAMESPACE="$2"
            shift 2
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
            echo "❌ Unknown option: $1" >&2
            echo "Use --help for usage information" >&2
            exit 1
            ;;
    esac
done

# Validate required arguments
if [[ -z "$GITHUB_USER" ]]; then
    echo "❌ Error: --user is required" >&2
    echo "Use --help for usage information" >&2
    exit 1
fi

if [[ -z "$SSH_KEY_PATH" ]]; then
    echo "❌ Error: --ssh-key is required" >&2
    echo "Use --help for usage information" >&2
    exit 1
fi

if [[ -z "$GITHUB_TOKEN" ]]; then
    echo "❌ Error: --token is required" >&2
    echo "Use --help for usage information" >&2
    exit 1
fi

# Validate SSH key path
SSH_KEY_PATH=$(eval echo "$SSH_KEY_PATH")  # Expand ~ if present
if [[ ! -f "$SSH_KEY_PATH" ]]; then
    echo "❌ Error: SSH private key not found at: $SSH_KEY_PATH" >&2
    exit 1
fi

# Derive public key path
SSH_PUB_PATH="${SSH_KEY_PATH}.pub"
if [[ ! -f "$SSH_PUB_PATH" ]]; then
    echo "❌ Error: SSH public key not found at: $SSH_PUB_PATH" >&2
    echo "Expected to find public key alongside private key" >&2
    exit 1
fi

# Validate GitHub token format
if [[ ! "$GITHUB_TOKEN" =~ ^ghp_[a-zA-Z0-9_]{36}$ ]]; then
    echo "⚠️  Warning: GitHub token doesn't match expected format (ghp_xxxxx)" >&2
    echo "Continuing anyway..." >&2
fi

# Generate secret names
SSH_SECRET_NAME="github-ssh-${GITHUB_USER}"
TOKEN_SECRET_NAME="github-token-${GITHUB_USER}"

# Show summary
echo "🚀 Setting up GitHub agent secrets"
echo "   User: $GITHUB_USER"
echo "   SSH Key: $SSH_KEY_PATH"
echo "   Namespace: $NAMESPACE"
echo "   SSH Secret: $SSH_SECRET_NAME"
echo "   Token Secret: $TOKEN_SECRET_NAME"
echo ""

if [[ -n "$DRY_RUN" ]]; then
    echo "🔍 DRY RUN MODE - No changes will be made"
    echo ""
fi

# Check if kubectl is available
if ! command -v kubectl >/dev/null 2>&1; then
    echo "❌ Error: kubectl is not installed or not in PATH" >&2
    exit 1
fi

# Check if namespace exists
if [[ -z "$DRY_RUN" ]]; then
    if ! kubectl get namespace "$NAMESPACE" >/dev/null 2>&1; then
        echo "❌ Error: Namespace '$NAMESPACE' does not exist" >&2
        echo "Create it first: kubectl create namespace $NAMESPACE" >&2
        exit 1
    fi
fi

# Create SSH secret
log "Creating SSH secret: $SSH_SECRET_NAME"
execute kubectl create secret generic "$SSH_SECRET_NAME" \
    --namespace="$NAMESPACE" \
    --from-file=ssh-privatekey="$SSH_KEY_PATH" \
    --from-file=ssh-publickey="$SSH_PUB_PATH" \
    --dry-run=client -o yaml \| kubectl apply -f -

# Create GitHub token secret
log "Creating GitHub token secret: $TOKEN_SECRET_NAME"
execute kubectl create secret generic "$TOKEN_SECRET_NAME" \
    --namespace="$NAMESPACE" \
    --from-literal=token="$GITHUB_TOKEN" \
    --dry-run=client -o yaml \| kubectl apply -f -

if [[ -z "$DRY_RUN" ]]; then
    echo ""
    echo "✅ Successfully created agent secrets for user: $GITHUB_USER"
    echo ""
    echo "🔍 Verify secrets:"
    echo "   kubectl get secrets -n $NAMESPACE | grep github-$GITHUB_USER"
    echo ""
    echo "📋 To use this agent in a CodeRun:"
    echo "   spec:"
    echo "     githubUser: \"$GITHUB_USER\""
    echo "     # ... other fields"
else
    echo ""
    echo "✅ Dry run completed successfully"
    echo "Remove --dry-run to execute these commands"
fi