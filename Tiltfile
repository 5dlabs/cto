# =============================================================================
# CTO Platform - Tilt Development Configuration
# =============================================================================
# Branch-aware development workflow:
#
#   Feature branches: Local builds → Local registry → Fast iteration
#   develop/main:     Builds disabled → Uses GHCR images from CI
#
# Quick Start:
#   ./scripts/dev.sh up      # Enable dev registry + start Tilt
#   ./scripts/dev.sh down    # Disable dev registry + stop Tilt
#   ./scripts/dev.sh status  # Check current mode
#
# Manual Setup:
#   1. Enable dev registry: ./scripts/argocd-dev-mode.sh enable
#   2. Start Tilt: tilt up
#
# Access the Tilt UI at http://localhost:10350
# =============================================================================

allow_k8s_contexts([
    'talos-home',
    'admin@simple-cluster',
    'kind-kind',
    'minikube',
    'docker-desktop',
])

# =============================================================================
# Branch Detection & Mode Selection
# =============================================================================
# Automatically detect the current branch and set the appropriate mode:
#   - Feature branches: DEV_MODE (local builds enabled)
#   - develop/main: PROD_MODE (use CI-built GHCR images)
#
# Override with environment variable: CTO_DEV_MODE=true|false|auto

def get_git_branch():
    """Get the current git branch name."""
    result = local('git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown"', quiet=True)
    return str(result).strip()

BRANCH = get_git_branch()
PROTECTED_BRANCHES = ['main', 'develop']

# Mode detection: auto (default), true (force dev), false (force prod)
DEV_MODE_ENV = os.getenv('CTO_DEV_MODE', 'auto')

if DEV_MODE_ENV == 'true':
    DEV_MODE = True
elif DEV_MODE_ENV == 'false':
    DEV_MODE = False
else:
    # Auto mode: dev for feature branches, prod for protected branches
    DEV_MODE = BRANCH not in PROTECTED_BRANCHES

# Print mode banner
if DEV_MODE:
    print('')
    print('╔═══════════════════════════════════════════════════════════════════╗')
    print('║  🔧 DEV MODE - Local builds enabled                               ║')
    print('║                                                                   ║')
    print('║  Branch: %-55s ║' % BRANCH)
    print('║  Registry: Local (192.168.1.77:30500)                             ║')
    print('║  Tag: tilt-dev                                                    ║')
    print('║                                                                   ║')
    print('║  Make sure dev registry is enabled:                               ║')
    print('║    ./scripts/argocd-dev-mode.sh enable                            ║')
    print('╚═══════════════════════════════════════════════════════════════════╝')
    print('')
else:
    print('')
    print('╔═══════════════════════════════════════════════════════════════════╗')
    print('║  🚀 PROD MODE - Using GHCR images (builds disabled)               ║')
    print('║                                                                   ║')
    print('║  Branch: %-55s ║' % BRANCH)
    print('║  Registry: ghcr.io/5dlabs/*                                       ║')
    print('║                                                                   ║')
    print('║  Builds are handled by CI/CD on this branch.                      ║')
    print('║  Only deploy commands are available.                              ║')
    print('║                                                                   ║')
    print('║  To force dev mode: CTO_DEV_MODE=true tilt up                     ║')
    print('╚═══════════════════════════════════════════════════════════════════╝')
    print('')

# =============================================================================
# Configuration
# =============================================================================
LOCAL_REGISTRY = os.getenv('LOCAL_REGISTRY', '192.168.1.77:30500')
NAMESPACE = 'cto'
DEV_TAG = os.getenv('DEV_TAG', 'tilt-dev')

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
# Core Services - PARALLEL BUILDS (Dev Mode Only)
# =============================================================================
# All builds run in parallel (no resource_deps between builds)
# Shared cargo registry cache means dependencies download once

if DEV_MODE:
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

    # Runtime with Tasks CLI (for intake workflow)
    local_resource(
        'build-runtime-tasks',
        cmd=build_cmd('runtime', 'infra/images/runtime/Dockerfile.kind-tasks'),
        deps=['crates/tasks/', 'Cargo.toml', 'Cargo.lock'],
        ignore=['**/target/'],
        labels=['build'],
    )

# =============================================================================
# Deploys
# =============================================================================
# In DEV_MODE: deploys depend on builds
# In PROD_MODE: deploys work standalone (restarts pods to pick up new GHCR images)

if DEV_MODE:
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
else:
    # PROD_MODE: Deploy-only resources (no build dependencies)
    local_resource(
        'deploy-pm',
        cmd=deploy_cmd('cto-pm'),
        labels=['deploy'],
        auto_init=False,
        trigger_mode=TRIGGER_MODE_MANUAL,
    )

    local_resource(
        'deploy-controller',
        cmd=deploy_cmd('cto-controller'),
        labels=['deploy'],
        auto_init=False,
        trigger_mode=TRIGGER_MODE_MANUAL,
    )

    local_resource(
        'deploy-tools',
        cmd=deploy_cmd('cto-tools'),
        labels=['deploy'],
        auto_init=False,
        trigger_mode=TRIGGER_MODE_MANUAL,
    )

    local_resource(
        'deploy-healer',
        cmd=deploy_cmd('cto-healer'),
        labels=['deploy'],
        auto_init=False,
        trigger_mode=TRIGGER_MODE_MANUAL,
    )

    local_resource(
        'deploy-research',
        cmd='echo "Research is a CronJob - uses new image on next run"',
        labels=['deploy'],
        auto_init=False,
        trigger_mode=TRIGGER_MODE_MANUAL,
    )

    local_resource(
        'deploy-tweakcn',
        cmd=deploy_cmd('tweakcn'),
        labels=['deploy'],
        auto_init=False,
        trigger_mode=TRIGGER_MODE_MANUAL,
    )

    local_resource(
        'deploy-openmemory',
        cmd=deploy_cmd('cto-openmemory'),
        labels=['deploy'],
        auto_init=False,
        trigger_mode=TRIGGER_MODE_MANUAL,
    )

    # Info resource showing current GHCR images
    local_resource(
        'ghcr-images',
        cmd='''
            echo "📦 Current GHCR images in use:"
            kubectl get deployments -n cto -o jsonpath='{range .items[*]}{.metadata.name}: {.spec.template.spec.containers[0].image}{"\n"}{end}' 2>/dev/null || echo "Could not fetch deployment info"
        ''',
        labels=['info'],
    )

# =============================================================================
# Dev Tools (Manual) - Available in both modes
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
# Port Forwards - Available in both modes
# =============================================================================

local_resource(
    'port-forwards',
    serve_cmd='kubectl port-forward svc/cto-controller -n cto 8080:8080 & kubectl port-forward svc/cto-tools -n cto 3000:3000 & kubectl port-forward svc/cto-pm -n cto 3001:3000 & wait',
    labels=['infra'],
    auto_init=False,
)

# =============================================================================
# Settings
# =============================================================================

if DEV_MODE:
    update_settings(
        max_parallel_updates=7,  # All 7 services can build in parallel
    )
else:
    update_settings(
        max_parallel_updates=3,  # Fewer parallel updates in prod mode
    )
