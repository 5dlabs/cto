# CTO Platform - Local Development Commands
# ===========================================
# Usage: just <command>
# Run `just --list` to see all available commands

# Default recipe - show help
default:
    @just --list

# =============================================================================
# Development Environment Setup
# =============================================================================

# Install development tools (just, bacon, process-compose)
install-tools:
    @echo "Installing development tools..."
    cargo install just bacon
    brew tap f1bonacc1/tap && brew install f1bonacc1/tap/process-compose || echo "Install process-compose manually: https://github.com/F1bonacc1/process-compose"
    @echo "✅ Development tools installed"

# Sync secrets from 1Password to .env.local
sync-secrets:
    @echo "Syncing secrets from 1Password..."
    ./scripts/sync-secrets-for-dev.sh
    @echo "✅ Secrets synced to .env.local"

# =============================================================================
# Build Commands
# =============================================================================

# Build all binaries in debug mode
build:
    cargo build --workspace

# Build all binaries in release mode
build-release:
    cargo build --workspace --release

# Build a specific binary
build-bin BIN:
    cargo build --bin {{BIN}}

# =============================================================================
# Check & Lint Commands
# =============================================================================

# Run cargo check on all crates
check:
    cargo check --workspace

# Run clippy with pedantic warnings (required before push)
clippy:
    cargo clippy --all-targets -- -D warnings -W clippy::pedantic

# Run cargo fmt check
fmt-check:
    cargo fmt --all --check

# Format all code
fmt:
    cargo fmt --all

# Run all pre-push checks (fmt, clippy, test)
pre-push: fmt-check clippy test
    @echo "✅ All pre-push checks passed"

# =============================================================================
# Test Commands
# =============================================================================

# Run all tests
test:
    cargo test --workspace

# Run tests with output
test-verbose:
    cargo test --workspace -- --nocapture

# Run tests for a specific crate
test-crate CRATE:
    cargo test -p {{CRATE}}

# =============================================================================
# Development Server Commands
# =============================================================================

# Start all services with file watching (requires bacon)
dev:
    @echo "Starting local development environment..."
    @echo "Make sure you have sourced .env.local first!"
    ./scripts/dev-local.sh

# Run PM server locally
dev-pm:
    @echo "Starting PM server..."
    cargo run --bin pm-server

# Run PM server with watching
watch-pm:
    bacon run-pm

# Run controller locally
dev-controller:
    @echo "Starting controller..."
    AGENT_TEMPLATES_PATH=./templates cargo run --bin agent-controller

# Run controller with watching
watch-controller:
    AGENT_TEMPLATES_PATH=./templates bacon run-controller

# Run tools server locally
dev-tools:
    @echo "Starting tools server..."
    SYSTEM_CONFIG_PATH=./infra/charts/cto/templates/tools cargo run --bin tools-server

# Run tools server with watching
watch-tools:
    SYSTEM_CONFIG_PATH=./infra/charts/cto/templates/tools bacon run-tools

# Run healer server locally
dev-healer:
    @echo "Starting healer server..."
    HEALER_TEMPLATES_DIR=./templates/healer CTO_CONFIG_PATH=./cto-config.json cargo run --bin healer -- server

# Run healer with watching
watch-healer:
    HEALER_TEMPLATES_DIR=./templates/healer CTO_CONFIG_PATH=./cto-config.json bacon run-healer

# =============================================================================
# CLI Commands (for testing CLIs locally)
# =============================================================================

# Run intake CLI
intake *ARGS:
    cargo run --bin intake -- {{ARGS}}

# Run research CLI
research *ARGS:
    cargo run --bin research -- {{ARGS}}

# Run healer CLI
healer *ARGS:
    HEALER_TEMPLATES_DIR=./templates/healer CTO_CONFIG_PATH=./cto-config.json cargo run --bin healer -- {{ARGS}}

# Run MCP server (stdio mode)
mcp:
    cargo run --bin mcp

# =============================================================================
# Bacon Watcher Commands (use these for development)
# =============================================================================

# Start bacon in check mode (default)
bacon-check:
    bacon

# Start bacon in clippy mode
bacon-clippy:
    bacon clippy

# Start bacon in test mode
bacon-test:
    bacon test

# =============================================================================
# Process Compose (TUI for monitoring services)
# =============================================================================

# Start all services with process-compose TUI (recommended for monitoring)
pc:
    @echo "Starting services with process-compose TUI..."
    @echo "Make sure you have sourced .env.local first!"
    process-compose up --port 8090

# Start process-compose in detached mode
pc-detach:
    process-compose up -d --port 8090

# Stop all process-compose services
pc-down:
    process-compose down --port 8090

# Show process-compose status
pc-status:
    process-compose process list --port 8090

# Attach to running process-compose
pc-attach:
    process-compose attach --port 8090

# =============================================================================
# Docker/Tilt Commands (for comparison/fallback)
# =============================================================================

# Start Tilt development environment (Docker-based)
tilt-up:
    ./scripts/dev.sh up

# Stop Tilt development environment
tilt-down:
    ./scripts/dev.sh down

# Check Tilt status
tilt-status:
    ./scripts/dev.sh status

# =============================================================================
# Cloudflare Tunnel (for receiving webhooks locally)
# =============================================================================

# Start Cloudflare tunnel for local PM development
tunnel:
    @echo "Starting Cloudflare tunnel for pm-dev.5dlabs.ai → localhost:8081"
    @echo "Webhooks will be routed to your local PM server"
    cloudflared tunnel --config config/cloudflared-pm-dev.yaml run

# Start tunnel in background
tunnel-bg:
    @echo "Starting Cloudflare tunnel in background..."
    cloudflared tunnel --config config/cloudflared-pm-dev.yaml run &
    @echo "✅ Tunnel started. pm-dev.5dlabs.ai → localhost:8081"

# Check tunnel status
tunnel-status:
    @echo "=== Cloudflare Tunnel Status ==="
    @cloudflared tunnel list | grep -E "(NAME|pm-local-dev)"
    @echo ""
    @echo "=== DNS Check ==="
    @nslookup pm-dev.5dlabs.ai | grep -A 2 "Name:" || echo "DNS not resolving"

# =============================================================================
# Cluster Management (for local dev)
# =============================================================================

# Scale down cluster services for local development
cluster-down:
    @echo "Scaling down in-cluster services..."
    kubectl scale deployment cto-pm cto-controller cto-healer cto-healer-sensor -n cto --replicas=0
    @echo "✅ In-cluster services scaled to 0"

# Restore cluster services after local development
cluster-up:
    @echo "Restoring in-cluster services..."
    kubectl scale deployment cto-pm cto-controller cto-healer cto-healer-sensor -n cto --replicas=1
    @echo "✅ In-cluster services restored to 1 replica each"
    @echo ""
    @echo "⚠️  Remember to restore the GitHub webhook:"
    @echo "    gh api repos/5dlabs/cto/hooks/585026279 -X PATCH -f 'config[url]=https://pm.5dlabs.ai/webhooks/github'"

# Restore GitHub webhook to production URL
webhook-restore:
    @echo "Restoring GitHub webhook to production URL..."
    gh api repos/5dlabs/cto/hooks/585026279 -X PATCH -f 'config[url]=https://pm.5dlabs.ai/webhooks/github' -f 'config[content_type]=json'
    @echo "✅ GitHub webhook restored to pm.5dlabs.ai"

# Point GitHub webhook to local dev tunnel
webhook-dev:
    @echo "Pointing GitHub webhook to dev tunnel..."
    gh api repos/5dlabs/cto/hooks/585026279 -X PATCH -f 'config[url]=https://pm-dev.5dlabs.ai/webhooks/github' -f 'config[content_type]=json'
    @echo "✅ GitHub webhook now points to pm-dev.5dlabs.ai"

# Show current webhook status
webhook-status:
    @echo "=== GitHub Webhook Status ==="
    @gh api repos/5dlabs/cto/hooks | jq '.[] | {id, url: .config.url, events, active}'

# Show cluster service status
cluster-status:
    @echo "=== CTO Namespace Services ==="
    @kubectl get deployments -n cto -o custom-columns='NAME:.metadata.name,REPLICAS:.spec.replicas,READY:.status.readyReplicas' | grep -E "(NAME|pm|controller|healer|tools)"

# =============================================================================
# Pre-flight Check
# =============================================================================

# Run comprehensive pre-flight check for local development
preflight:
    #!/usr/bin/env bash
    set -euo pipefail
    source .env.local 2>/dev/null || true
    
    echo ""
    echo "╔══════════════════════════════════════════════════════════════════════════════╗"
    echo "║                    CTO LOCAL DEVELOPMENT PRE-FLIGHT CHECK                    ║"
    echo "╚══════════════════════════════════════════════════════════════════════════════╝"
    echo ""
    
    echo "═══ 1. SERVICE STATUS ═══"
    printf "%-20s %-12s %-12s\n" "SERVICE" "K8S" "LOCAL"
    for svc in pm controller healer tools; do
      k8s=$(kubectl get deployment cto-$svc -n cto -o jsonpath='{.spec.replicas}' 2>/dev/null || echo "?")
      case $svc in
        pm) port=8081 ;; controller) port=8080 ;; healer) port=8082 ;; tools) port=3000 ;;
      esac
      local_status=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:$port/health 2>/dev/null || echo "000")
      [ "$local_status" = "200" ] && local_status="✅ UP" || local_status="❌ DOWN"
      [ "$k8s" = "0" ] && k8s="⬇️ (0)" || k8s="🔴 ($k8s)"
      printf "%-20s %-12s %-12s\n" "$svc" "$k8s" "$local_status"
    done
    
    echo ""
    echo "═══ 2. TUNNEL & WEBHOOKS ═══"
    tunnel=$(curl -s -o /dev/null -w "%{http_code}" https://pm-dev.5dlabs.ai/health 2>/dev/null || echo "000")
    [ "$tunnel" = "200" ] && echo "✅ Tunnel: pm-dev.5dlabs.ai → localhost:8081" || echo "❌ Tunnel not working"
    webhook_url=$(gh api repos/5dlabs/cto/hooks 2>/dev/null | jq -r '.[0].config.url' 2>/dev/null || echo "unavailable")
    echo "📎 GitHub Webhook: $webhook_url"
    
    echo ""
    echo "═══ 3. CLUSTER ACCESS ═══"
    cluster_info=$(kubectl cluster-info 2>/dev/null | head -1 || true)
    [ -n "$cluster_info" ] && echo "$cluster_info" || echo "❌ Cannot connect to cluster"
    kubectl get crd coderuns.agents.platform 2>/dev/null > /dev/null && echo "✅ CodeRun CRD available" || echo "❌ CodeRun CRD missing"
    
    echo ""
    echo "═══ 4. API KEYS ═══"
    [ -n "${LINEAR_API_KEY:-}" ] && echo "✅ LINEAR_API_KEY" || echo "❌ LINEAR_API_KEY"
    [ -n "${ANTHROPIC_API_KEY:-}" ] && echo "✅ ANTHROPIC_API_KEY" || echo "❌ ANTHROPIC_API_KEY"
    [ -n "${GITHUB_TOKEN:-}" ] && echo "✅ GITHUB_TOKEN" || echo "❌ GITHUB_TOKEN"
    
    echo ""
    echo "═══ 5. LOCAL DEV CONFIGURATION ═══"
    # Check CTO_PM_SERVER_URL env var
    if [ -n "${CTO_PM_SERVER_URL:-}" ]; then
      if [[ "$CTO_PM_SERVER_URL" == *"-dev."* ]] || [[ "$CTO_PM_SERVER_URL" == *"localhost"* ]]; then
        echo "✅ CTO_PM_SERVER_URL = $CTO_PM_SERVER_URL (dev)"
      else
        echo "⚠️  CTO_PM_SERVER_URL = $CTO_PM_SERVER_URL (not dev - may use production!)"
      fi
    else
      echo "⚠️  CTO_PM_SERVER_URL not set (will use cto-config.json or default to production)"
    fi
    
    # Check cto-config.json pmServerUrl
    if [ -f "cto-config.json" ]; then
      pm_url=$(jq -r '.defaults.linear.pmServerUrl // empty' cto-config.json 2>/dev/null || echo "")
      if [ -n "$pm_url" ]; then
        if [[ "$pm_url" == *"-dev."* ]] || [[ "$pm_url" == *"localhost"* ]]; then
          echo "✅ cto-config.json pmServerUrl = $pm_url (dev)"
        else
          echo "⚠️  cto-config.json pmServerUrl = $pm_url (not dev!)"
        fi
      else
        echo "⚠️  cto-config.json has no pmServerUrl (will use default: pm.5dlabs.ai)"
      fi
      team_id=$(jq -r '.defaults.linear.teamId // empty' cto-config.json 2>/dev/null || echo "")
      [ -n "$team_id" ] && echo "✅ cto-config.json teamId = $team_id" || echo "⚠️  No teamId in cto-config.json"
    else
      echo "❌ cto-config.json not found"
    fi
    
    echo ""
    echo "═══ PRE-FLIGHT COMPLETE ═══"
    echo ""
    echo "To start local development:"
    echo "  1. just pc          # Start all services with process-compose"
    echo "  2. Wait for tunnel to be healthy"
    echo "  3. Use MCP tools from Cursor"

# =============================================================================
# Utility Commands
# =============================================================================

# Clean build artifacts
clean:
    cargo clean

# Update dependencies
update:
    cargo update

# Check for outdated dependencies
outdated:
    cargo outdated

# Generate documentation
doc:
    cargo doc --workspace --no-deps --open

# Print environment info for debugging
env-info:
    @echo "=== Environment Info ==="
    @echo "KUBECONFIG: ${KUBECONFIG:-not set}"
    @echo "NAMESPACE: ${NAMESPACE:-cto}"
    @echo "RUST_LOG: ${RUST_LOG:-not set}"
    @echo ""
    @echo "=== Kubernetes Context ==="
    @kubectl config current-context 2>/dev/null || echo "kubectl not configured"
    @echo ""
    @echo "=== Cargo Version ==="
    @cargo --version
    @echo ""
    @echo "=== Bacon Version ==="
    @bacon --version 2>/dev/null || echo "bacon not installed (run: just install-tools)"

# Clean up old archived test projects in Linear
cleanup-test-projects:
    #!/usr/bin/env bash
    set -euo pipefail
    source .env.local
    
    echo "Fetching archived test projects..."
    projects=$(curl -s -X POST https://api.linear.app/graphql \
      -H "Authorization: $LINEAR_API_KEY" \
      -H "Content-Type: application/json" \
      -d '{"query": "query { projects(first: 50, includeArchived: true, filter: { name: { containsIgnoreCase: \"tests/intake\" }, archivedAt: { neq: null } }) { nodes { id name archivedAt } } }"}' | jq -r '.data.projects.nodes')
    
    count=$(echo "$projects" | jq 'length')
    echo "Found $count archived test projects"
    
    if [ "$count" -gt 0 ]; then
      echo "Projects to delete:"
      echo "$projects" | jq -r '.[] | "  - \(.name) (archived: \(.archivedAt))"'
      echo ""
      read -p "Delete these projects permanently? (y/N) " -n 1 -r
      echo ""
      if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "$projects" | jq -r '.[].id' | while read id; do
          echo "Deleting project $id..."
          curl -s -X POST https://api.linear.app/graphql \
            -H "Authorization: $LINEAR_API_KEY" \
            -H "Content-Type: application/json" \
            -d "{\"query\": \"mutation { projectDelete(id: \\\"$id\\\") { success } }\"}" | jq -r '.data.projectDelete.success'
        done
        echo "✅ Cleanup complete"
      else
        echo "Aborted"
      fi
    fi
