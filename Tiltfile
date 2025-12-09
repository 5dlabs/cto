# =============================================================================
# CTO Platform - Tilt Development Configuration
# =============================================================================
# All 7 services build in PARALLEL for maximum speed.
#
# Services: PM, Controller, Tools, Healer, Research, TweakCN, OpenMemory
#
# Prerequisites:
#   1. Enable dev registry: ./scripts/argocd-dev-mode.sh enable
#   2. Start Tilt: tilt up
#
# Access the Tilt UI at http://localhost:10350
#
# To speed up builds:
#   - Increase Docker Desktop CPU/memory (Settings → Resources)
#   - Builds use shared cargo registry cache
# =============================================================================

allow_k8s_contexts([
    'talos-home',
    'admin@simple-cluster',
    'kind-kind',
    'minikube',
    'docker-desktop',
])

# =============================================================================
# Configuration
# =============================================================================
LOCAL_REGISTRY = os.getenv('LOCAL_REGISTRY', '192.168.1.72:30500')
NAMESPACE = 'cto'
DEV_TAG = 'tilt-dev'

# =============================================================================
# Build Commands
# =============================================================================

def build_cmd(name, dockerfile):
    """Generate a build+push command for a service."""
    return '''
        set -e
        echo "🔨 Building %s..."
        DOCKER_BUILDKIT=1 docker build \
            --platform linux/amd64 \
            --build-arg BUILDKIT_INLINE_CACHE=1 \
            -t %s/%s:%s \
            -f %s \
            . && \
        echo "📤 Pushing %s..." && \
        docker push %s/%s:%s && \
        echo "✅ %s complete"
    ''' % (name, LOCAL_REGISTRY, name, DEV_TAG, dockerfile, name, LOCAL_REGISTRY, name, DEV_TAG, name)

def deploy_cmd(deployment_name):
    """Generate a rollout restart command."""
    return 'kubectl rollout restart deployment/%s -n %s && kubectl rollout status deployment/%s -n %s --timeout=180s' % (deployment_name, NAMESPACE, deployment_name, NAMESPACE)

# =============================================================================
# Core Services - PARALLEL BUILDS
# =============================================================================
# All builds run in parallel (no resource_deps between builds)
# Shared cargo registry cache means dependencies download once

# PM Server
local_resource(
    'build-pm',
    cmd=build_cmd('pm', 'infra/images/pm-server/Dockerfile.build'),
    deps=['crates/pm/', 'Cargo.toml', 'Cargo.lock'],
    ignore=['**/target/'],
    labels=['build'],
)

# Controller
local_resource(
    'build-controller',
    cmd=build_cmd('controller', 'infra/images/controller/Dockerfile.kind'),
    deps=['crates/controller/', 'templates/', 'Cargo.toml', 'Cargo.lock'],
    ignore=['**/target/'],
    labels=['build'],
)

# Tools
local_resource(
    'build-tools',
    cmd=build_cmd('tools', 'infra/images/tools/Dockerfile.kind'),
    deps=['crates/tools/', 'crates/mcp/', 'Cargo.toml', 'Cargo.lock'],
    ignore=['**/target/'],
    labels=['build'],
)

# Healer
local_resource(
    'build-healer',
    cmd=build_cmd('healer', 'infra/images/healer/Dockerfile.kind'),
    deps=['crates/healer/', 'Cargo.toml', 'Cargo.lock'],
    ignore=['**/target/'],
    labels=['build'],
)

# Research
local_resource(
    'build-research',
    cmd=build_cmd('research', 'infra/images/research/Dockerfile'),
    deps=['crates/research/', 'Cargo.toml', 'Cargo.lock'],
    ignore=['**/target/'],
    labels=['build'],
)

# TweakCN
local_resource(
    'build-tweakcn',
    cmd=build_cmd('tweakcn', 'infra/images/tweakcn/Dockerfile'),
    deps=['crates/tweakcn/', 'Cargo.toml', 'Cargo.lock'] if os.path.exists('crates/tweakcn') else [],
    ignore=['**/target/'],
    labels=['build'],
    auto_init=False,  # Manual trigger - may not have crate
    trigger_mode=TRIGGER_MODE_MANUAL,
)

# OpenMemory
local_resource(
    'build-openmemory',
    cmd=build_cmd('openmemory', 'infra/images/openmemory/Dockerfile'),
    deps=['crates/openmemory/', 'Cargo.toml', 'Cargo.lock'] if os.path.exists('crates/openmemory') else [],
    ignore=['**/target/'],
    labels=['build'],
    auto_init=False,  # Manual trigger - may not have crate
    trigger_mode=TRIGGER_MODE_MANUAL,
)

# =============================================================================
# Deploys (After Builds)
# =============================================================================

local_resource(
    'deploy-pm',
    cmd=deploy_cmd('cto-pm'),
    resource_deps=['build-pm'],
    labels=['deploy'],
)

local_resource(
    'deploy-controller',
    cmd=deploy_cmd('cto-controller'),
    resource_deps=['build-controller'],
    labels=['deploy'],
)

local_resource(
    'deploy-tools',
    cmd=deploy_cmd('cto-tools'),
    resource_deps=['build-tools'],
    labels=['deploy'],
)

local_resource(
    'deploy-healer',
    cmd=deploy_cmd('cto-healer'),
    resource_deps=['build-healer'],
    labels=['deploy'],
)

local_resource(
    'deploy-research',
    cmd='echo "Research is a CronJob - uses new image on next run"',
    resource_deps=['build-research'],
    labels=['deploy'],
)

local_resource(
    'deploy-tweakcn',
    cmd=deploy_cmd('tweakcn'),
    resource_deps=['build-tweakcn'],
    labels=['deploy'],
    auto_init=False,
)

local_resource(
    'deploy-openmemory',
    cmd=deploy_cmd('cto-openmemory'),
    resource_deps=['build-openmemory'],
    labels=['deploy'],
    auto_init=False,
)

# =============================================================================
# Dev Tools (Manual)
# =============================================================================

local_resource(
    'cargo-check',
    cmd='cargo check --workspace',
    labels=['dev'],
    auto_init=False,
    trigger_mode=TRIGGER_MODE_MANUAL,
)

local_resource(
    'cargo-clippy', 
    cmd='cargo clippy --workspace -- -D warnings',
    labels=['dev'],
    auto_init=False,
    trigger_mode=TRIGGER_MODE_MANUAL,
)

local_resource(
    'cargo-test',
    cmd='cargo test --workspace',
    labels=['dev'],
    auto_init=False,
    trigger_mode=TRIGGER_MODE_MANUAL,
)

# =============================================================================
# Port Forwards
# =============================================================================

local_resource(
    'port-forwards',
    serve_cmd='kubectl port-forward svc/cto-controller -n cto 8080:8080 & kubectl port-forward svc/cto-tools -n cto 3000:3000 & kubectl port-forward svc/cto-pm -n cto 3001:3000 & wait',
    labels=['infra'],
    auto_init=False,
)

# =============================================================================
# Settings - PARALLEL BUILDS
# =============================================================================

update_settings(
    max_parallel_updates=7,  # All 7 services can build in parallel
)
