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

# Build core services (controller, pm-server, healer) in release mode with verbose progress
# Use this for faster local dev builds - shows compile progress and uses sccache
build-release-fast:
    @echo "Building release binaries with verbose progress (using sccache)..."
    cargo build --release --bin agent-controller --bin pm-server --bin healer -vv

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

# Run healer Play API server (for MCP integration)
dev-healer-play-api:
    @echo "Starting healer Play API server..."
    cargo run --bin healer -- play-api --addr 0.0.0.0:8083

# Run healer Play API with release build
healer-play-api:
    ./target/release/healer play-api --addr 0.0.0.0:8083

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
# mprocs (TUI for monitoring services) - RECOMMENDED
# =============================================================================

# Kill processes on dev ports (8080, 8081, 8082, 8083, 3000)
kill-ports:
    @echo "🧹 Cleaning up stale processes..."
    -lsof -ti :8080 | xargs kill -9 2>/dev/null || true
    -lsof -ti :8081 | xargs kill -9 2>/dev/null || true
    -lsof -ti :8082 | xargs kill -9 2>/dev/null || true
    -lsof -ti :8083 | xargs kill -9 2>/dev/null || true
    -lsof -ti :3000 | xargs kill -9 2>/dev/null || true
    @sleep 1
    @echo "✅ Ports cleared"

# Start all services with mprocs TUI (recommended)
mp: kill-ports
    @echo "Starting services with mprocs TUI..."
    @if [ -f .env.local ]; then \
        echo "Sourcing .env.local to load environment variables..."; \
        set -a; \
        source .env.local; \
        set +a; \
    else \
        echo "⚠️  .env.local not found - some services may not work"; \
    fi
    mprocs

# =============================================================================
# launchd Services (background daemon mode - no terminal required)
# =============================================================================

# Install and start launchd services (runs in background, auto-restarts on rebuild)
launchd-install:
    @echo "Installing CTO services as launchd daemons..."
    @echo "Prerequisites: fswatch (brew install fswatch), release binaries (cargo build --release)"
    ./scripts/launchd-setup.sh install

# Uninstall launchd services
launchd-uninstall:
    ./scripts/launchd-setup.sh uninstall

# Show launchd service status
launchd-status:
    ./scripts/launchd-setup.sh status

# Tail launchd service logs
launchd-logs:
    ./scripts/launchd-setup.sh logs

# Restart all launchd services
launchd-restart:
    ./scripts/launchd-setup.sh restart

# Start launchd services (if stopped)
launchd-start:
    ./scripts/launchd-setup.sh start

# Stop launchd services (without unloading)
launchd-stop:
    ./scripts/launchd-setup.sh stop

# Build release binaries and let launchd auto-restart (main development workflow)
build-and-restart: build-release
    @echo "✅ Release binaries built - launchd watcher will auto-restart services"

# Monitor launchd services with lnav (status + logs in one TUI)
launchd-monitor:
    #!/usr/bin/env bash
    set -euo pipefail
    
    # Check if lnav is installed
    if ! command -v lnav &> /dev/null; then
        echo "lnav not found. Installing via Homebrew..."
        brew install lnav
    fi
    
    # Show current status first
    ./scripts/launchd-setup.sh status
    echo ""
    echo "Press Enter to open log viewer (lnav)..."
    read -r
    
    # Open lnav with all log files
    lnav /tmp/cto-launchd/*.log

# Monitor launchd services with multitail (split pane view)
launchd-multitail:
    #!/usr/bin/env bash
    set -euo pipefail
    
    # Check if multitail is installed
    if ! command -v multitail &> /dev/null; then
        echo "multitail not found. Installing via Homebrew..."
        brew install multitail
    fi
    
    # Open multitail with all log files
    multitail -s 2 /tmp/cto-launchd/controller.log /tmp/cto-launchd/pm-server.log /tmp/cto-launchd/healer.log /tmp/cto-launchd/healer-sensor.log /tmp/cto-launchd/tunnel.log /tmp/cto-launchd/watcher.log

# =============================================================================
# Process Compose (legacy - use mprocs instead)
# =============================================================================

# Start all services with process-compose TUI
pc: kill-ports
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
    
    ERRORS=0
    WARNINGS=0
    
    echo ""
    echo "╔══════════════════════════════════════════════════════════════════════════════╗"
    echo "║                    CTO LOCAL DEVELOPMENT PRE-FLIGHT CHECK                    ║"
    echo "╚══════════════════════════════════════════════════════════════════════════════╝"
    echo ""
    
    echo "═══ 1. SERVICE STATUS ═══"
    printf "%-20s %-12s %-12s\n" "SERVICE" "K8S" "LOCAL"
    for svc in pm controller healer healer-play-api tools; do
      k8s=$(kubectl get deployment cto-$svc -n cto -o jsonpath='{.spec.replicas}' 2>/dev/null || echo "?")
      case $svc in
        pm) port=8081 ;; controller) port=8080 ;; healer) port=8082 ;; healer-play-api) port=8083 ;; tools) port=3000 ;;
      esac
      local_status=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:$port/health 2>/dev/null || echo "000")
      [ "$local_status" = "200" ] && local_status="✅ UP" || { local_status="❌ DOWN"; ERRORS=$((ERRORS+1)); }
      [ "$k8s" = "0" ] && k8s="⬇️ (0)" || k8s="🔴 ($k8s)"
      printf "%-20s %-12s %-12s\n" "$svc" "$k8s" "$local_status"
    done
    
    echo ""
    echo "═══ 2. TUNNEL & WEBHOOKS ═══"
    tunnel=$(curl -s -o /dev/null -w "%{http_code}" https://pm-dev.5dlabs.ai/health 2>/dev/null || echo "000")
    [ "$tunnel" = "200" ] && echo "✅ Tunnel: pm-dev.5dlabs.ai → localhost:8081" || { echo "❌ Tunnel not working (run: just tunnel)"; ERRORS=$((ERRORS+1)); }
    webhook_url=$(gh api repos/5dlabs/cto/hooks 2>/dev/null | jq -r '.[0].config.url' 2>/dev/null || echo "unavailable")
    if [[ "$webhook_url" == *"-dev."* ]]; then
      echo "✅ GitHub Webhook: $webhook_url (dev)"
    else
      echo "⚠️  GitHub Webhook: $webhook_url (run: just webhook-dev)"
      WARNINGS=$((WARNINGS+1))
    fi
    
    echo ""
    echo "═══ 3. CLUSTER ACCESS ═══"
    cluster_info=$(kubectl cluster-info 2>/dev/null | head -1 || true)
    [ -n "$cluster_info" ] && echo "$cluster_info" || { echo "❌ Cannot connect to cluster"; ERRORS=$((ERRORS+1)); }
    kubectl get crd coderuns.agents.platform 2>/dev/null > /dev/null && echo "✅ CodeRun CRD available" || { echo "❌ CodeRun CRD missing"; ERRORS=$((ERRORS+1)); }
    kubectl get crd workflows.argoproj.io 2>/dev/null > /dev/null && echo "✅ Argo Workflow CRD available" || { echo "❌ Argo Workflow CRD missing"; ERRORS=$((ERRORS+1)); }
    
    echo ""
    echo "═══ 4. API KEYS & SECRETS ═══"
    # Check for LINEAR_OAUTH_TOKEN (preferred) - OAuth flow is required for local dev
    if [ -n "${LINEAR_OAUTH_TOKEN:-}" ]; then
      echo "✅ LINEAR_OAUTH_TOKEN (OAuth - preferred)"
    elif [ -n "${LINEAR_API_KEY:-}" ]; then
      echo "⚠️  LINEAR_API_KEY (legacy - OAuth preferred)"
      WARNINGS=$((WARNINGS+1))
    else
      echo "❌ LINEAR_OAUTH_TOKEN (required for OAuth flow)"
      ERRORS=$((ERRORS+1))
    fi
    [ -n "${ANTHROPIC_API_KEY:-}" ] && echo "✅ ANTHROPIC_API_KEY" || { echo "❌ ANTHROPIC_API_KEY"; ERRORS=$((ERRORS+1)); }
    [ -n "${GITHUB_TOKEN:-}" ] && echo "✅ GITHUB_TOKEN" || { echo "❌ GITHUB_TOKEN"; ERRORS=$((ERRORS+1)); }
    [ -n "${LINEAR_WEBHOOK_SECRET:-}" ] && echo "✅ LINEAR_WEBHOOK_SECRET" || { echo "❌ LINEAR_WEBHOOK_SECRET"; ERRORS=$((ERRORS+1)); }
    
    echo ""
    echo "═══ 5. AGENT OAUTH TOKENS ═══"
    for agent in morgan rex blaze bolt atlas cleo cipher tess; do
      var_name="LINEAR_APP_$(echo $agent | tr '[:lower:]' '[:upper:]')_WEBHOOK_SECRET"
      token_var="LINEAR_APP_$(echo $agent | tr '[:lower:]' '[:upper:]')_ACCESS_TOKEN"
      eval "secret=\${$var_name:-}"
      eval "token=\${$token_var:-}"
      [ -n "$secret" ] && secret_status="✓ secret" || secret_status="✗ secret"
      [ -n "$token" ] && token_status="✓ token" || token_status="✗ token"
      if [ -n "$secret" ] && [ -n "$token" ]; then
        echo "✅ $agent: $secret_status, $token_status"
      elif [ -n "$secret" ]; then
        echo "⚠️  $agent: $secret_status, $token_status (need OAuth: /oauth/start?agent=$agent)"
        WARNINGS=$((WARNINGS+1))
      else
        echo "❌ $agent: $secret_status, $token_status"
        ERRORS=$((ERRORS+1))
      fi
    done
    
    echo ""
    echo "═══ 6. LOCAL DEV CONFIGURATION ═══"
    if [ -n "${CTO_PM_SERVER_URL:-}" ]; then
      if [[ "$CTO_PM_SERVER_URL" == *"-dev."* ]] || [[ "$CTO_PM_SERVER_URL" == *"localhost"* ]]; then
        echo "✅ CTO_PM_SERVER_URL = $CTO_PM_SERVER_URL (dev)"
      else
        echo "⚠️  CTO_PM_SERVER_URL = $CTO_PM_SERVER_URL (not dev - may use production!)"
        WARNINGS=$((WARNINGS+1))
      fi
    else
      echo "⚠️  CTO_PM_SERVER_URL not set (will use cto-config.json or default to production)"
      WARNINGS=$((WARNINGS+1))
    fi
    
    if [ -f "cto-config.json" ]; then
      pm_url=$(jq -r '.defaults.linear.pmServerUrl // empty' cto-config.json 2>/dev/null || echo "")
      if [ -n "$pm_url" ]; then
        if [[ "$pm_url" == *"-dev."* ]] || [[ "$pm_url" == *"localhost"* ]]; then
          echo "✅ cto-config.json pmServerUrl = $pm_url (dev)"
        else
          echo "⚠️  cto-config.json pmServerUrl = $pm_url (not dev!)"
          WARNINGS=$((WARNINGS+1))
        fi
      else
        echo "⚠️  cto-config.json has no pmServerUrl (will use default: pm.5dlabs.ai)"
        WARNINGS=$((WARNINGS+1))
      fi
      team_id=$(jq -r '.defaults.linear.teamId // empty' cto-config.json 2>/dev/null || echo "")
      [ -n "$team_id" ] && echo "✅ cto-config.json teamId = $team_id" || { echo "⚠️  No teamId in cto-config.json"; WARNINGS=$((WARNINGS+1)); }
    else
      echo "❌ cto-config.json not found"
      ERRORS=$((ERRORS+1))
    fi
    
    echo ""
    echo "═══ 7. RECENT CODERUN STATUS ═══"
    echo "Last 5 CodeRuns:"
    kubectl get coderuns -n cto --sort-by=.metadata.creationTimestamp -o custom-columns='NAME:.metadata.name,TYPE:.spec.runType,PHASE:.status.phase,AGE:.metadata.creationTimestamp' 2>/dev/null | tail -6 | head -6 || echo "Could not fetch CodeRuns"
    
    failed_count=$(kubectl get coderuns -n cto -o json 2>/dev/null | jq '[.items[] | select(.status.phase == "Failed")] | length' 2>/dev/null || echo "0")
    if [ "$failed_count" -gt 0 ]; then
      echo ""
      echo "⚠️  $failed_count failed CodeRuns in cluster"
      WARNINGS=$((WARNINGS+1))
    fi
    
    echo ""
    echo "════════════════════════════════════════════════════════════════════════════════"
    if [ $ERRORS -eq 0 ] && [ $WARNINGS -eq 0 ]; then
      echo "✅ PRE-FLIGHT PASSED - Ready for intake/play workflows!"
    elif [ $ERRORS -eq 0 ]; then
      echo "⚠️  PRE-FLIGHT PASSED WITH $WARNINGS WARNING(S)"
      echo "   Workflows may run but some features (like Linear activities) may be limited"
    else
      echo "❌ PRE-FLIGHT FAILED - $ERRORS error(s), $WARNINGS warning(s)"
      echo "   Fix the errors above before running intake/play workflows"
    fi
    echo "════════════════════════════════════════════════════════════════════════════════"
    echo ""
    echo "Quick commands:"
    echo "  just mp              # Start all services with mprocs (kills stale ports first)"
    echo "  just tunnel          # Start Cloudflare tunnel"
    echo "  just webhook-dev     # Point GitHub webhook to dev"
    echo "  just cluster-down    # Scale down in-cluster services"

# =============================================================================
# Fast Dev Image Builds (bypass GitHub Actions)
# =============================================================================

# Build and push dev runtime image with local intake binary
dev-runtime-image:
    @echo "Building dev runtime image with local intake..."
    ./scripts/build-dev-image.sh --binary intake --image runtime --push

# Build and push dev Claude image with local intake binary
dev-claude-image:
    @echo "Building dev Claude image with local intake..."
    ./scripts/build-dev-image.sh --binary intake --image claude --push

# Build dev image locally without pushing (for testing)
dev-image-local:
    ./scripts/build-dev-image.sh --binary intake --image runtime

# Build and push all binaries to dev runtime
dev-runtime-all:
    ./scripts/build-dev-image.sh --binary all --image runtime --push

# Install cross-compilation tools for dev builds
install-cross-tools:
    @echo "Installing cross-compilation tools..."
    cargo install cargo-zigbuild
    @echo ""
    @echo "✅ cargo-zigbuild installed"
    @echo ""
    @echo "For GHCR authentication, run:"
    @echo '  echo $$GITHUB_TOKEN | docker login ghcr.io -u YOUR_USERNAME --password-stdin'

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
