#!/usr/bin/env bash
# =============================================================================
# BuildKit Performance Benchmark
# =============================================================================
# Compares build performance with SSD vs RAMDisk (tmpfs) backing storage.
#
# Usage:
#   ./scripts/benchmark-buildkit.sh [--iterations N]
#
# Requirements:
#   - kubectl access to cluster with BuildKit deployed
#   - docker buildx configured to use cluster builder
#
set -euo pipefail

# Configuration
ITERATIONS="${1:-3}"
NAMESPACE="${NAMESPACE:-cto}"
BUILDKIT_STATEFULSET="buildkit"
TEST_IMAGE="benchmark-test"
REGISTRY="${REGISTRY:-192.168.1.72:30500}"

# Colors (RED used in log_error if added later)
# shellcheck disable=SC2034
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[OK]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_header() { echo -e "\n${CYAN}${BOLD}=== $1 ===${NC}\n"; }

# =============================================================================
# Create test Dockerfile
# =============================================================================
create_test_dockerfile() {
    local dir="$1"
    mkdir -p "$dir"
    
    # Create a Dockerfile that does I/O intensive work (simulating Rust build)
    cat > "$dir/Dockerfile" << 'EOF'
# Test Dockerfile for BuildKit benchmark
# Simulates I/O intensive build operations

FROM alpine:3.19 AS builder

# Install build tools
RUN apk add --no-cache gcc musl-dev make

# Create dummy source files (simulates a codebase)
WORKDIR /build
RUN for i in $(seq 1 100); do \
        echo "int func_$i() { return $i; }" > "file_$i.c"; \
    done

# Create a main file
RUN echo '#include <stdio.h>' > main.c && \
    for i in $(seq 1 100); do \
        echo "int func_$i();" >> main.c; \
    done && \
    echo 'int main() {' >> main.c && \
    echo '    int sum = 0;' >> main.c && \
    for i in $(seq 1 100); do \
        echo "    sum += func_$i();" >> main.c; \
    done && \
    echo '    printf("Sum: %d\\n", sum);' >> main.c && \
    echo '    return 0;' >> main.c && \
    echo '}' >> main.c

# Compile all files (I/O intensive - lots of small file reads/writes)
RUN for i in $(seq 1 100); do \
        gcc -c -O2 "file_$i.c" -o "file_$i.o"; \
    done && \
    gcc -c -O2 main.c -o main.o && \
    gcc -o app *.o

# Write some large temporary files (simulates cargo target dir)
RUN dd if=/dev/urandom of=/tmp/large1.bin bs=1M count=50 2>/dev/null && \
    dd if=/dev/urandom of=/tmp/large2.bin bs=1M count=50 2>/dev/null && \
    md5sum /tmp/*.bin > /tmp/checksums.txt

FROM alpine:3.19
COPY --from=builder /build/app /app
CMD ["/app"]
EOF
}

# =============================================================================
# Run a single build and return time in seconds
# =============================================================================
run_build() {
    local label="$1"
    local build_dir="$2"
    local iteration="$3"
    
    # Clear any existing cache for this image
    docker buildx prune -f --filter "label=benchmark=true" 2>/dev/null || true
    
    # Time the build
    local start_time end_time duration
    start_time=$(date +%s.%N)
    
    docker buildx build \
        --no-cache \
        --label "benchmark=true" \
        --platform linux/amd64 \
        -t "${REGISTRY}/cto/${TEST_IMAGE}:${label}-${iteration}" \
        "$build_dir" 2>&1 | tail -5
    
    end_time=$(date +%s.%N)
    duration=$(echo "$end_time - $start_time" | bc)
    
    echo "$duration"
}

# =============================================================================
# Patch BuildKit to use tmpfs
# =============================================================================
enable_tmpfs() {
    log_info "Patching BuildKit to use tmpfs workspace..."
    
    kubectl patch statefulset "$BUILDKIT_STATEFULSET" -n "$NAMESPACE" --type='json' -p='[
        {
            "op": "add",
            "path": "/spec/template/spec/volumes/-",
            "value": {
                "name": "tmpfs-workspace",
                "emptyDir": {
                    "medium": "Memory",
                    "sizeLimit": "16Gi"
                }
            }
        },
        {
            "op": "add",
            "path": "/spec/template/spec/containers/0/volumeMounts/-",
            "value": {
                "name": "tmpfs-workspace",
                "mountPath": "/tmp"
            }
        }
    ]' 2>/dev/null || {
        log_warn "Patch may already be applied or failed"
        return 1
    }
    
    # Wait for rollout
    log_info "Waiting for BuildKit pod to restart..."
    kubectl rollout status statefulset/"$BUILDKIT_STATEFULSET" -n "$NAMESPACE" --timeout=120s
    sleep 5  # Extra buffer for BuildKit to be ready
}

# =============================================================================
# Remove tmpfs patch
# =============================================================================
disable_tmpfs() {
    log_info "Removing tmpfs patch from BuildKit..."
    
    # Get current volumes and volumeMounts, filter out tmpfs-workspace
    kubectl patch statefulset "$BUILDKIT_STATEFULSET" -n "$NAMESPACE" --type='json' -p='[
        {
            "op": "remove",
            "path": "/spec/template/spec/volumes/2"
        },
        {
            "op": "remove", 
            "path": "/spec/template/spec/containers/0/volumeMounts/2"
        }
    ]' 2>/dev/null || {
        log_warn "Could not remove patch (may not be applied)"
        return 0
    }
    
    # Wait for rollout
    log_info "Waiting for BuildKit pod to restart..."
    kubectl rollout status statefulset/"$BUILDKIT_STATEFULSET" -n "$NAMESPACE" --timeout=120s
    sleep 5
}

# =============================================================================
# Main benchmark
# =============================================================================
main() {
    log_header "BuildKit RAMDisk Benchmark"
    
    echo "Configuration:"
    echo "  Iterations: $ITERATIONS"
    echo "  Namespace:  $NAMESPACE"
    echo "  Registry:   $REGISTRY"
    echo ""
    
    # Check prerequisites
    if ! kubectl get statefulset "$BUILDKIT_STATEFULSET" -n "$NAMESPACE" &>/dev/null; then
        log_warn "BuildKit StatefulSet not found in namespace $NAMESPACE"
        echo "Make sure BuildKit is deployed and you have kubectl access."
        exit 1
    fi
    
    if ! docker buildx inspect cluster-builder &>/dev/null; then
        log_warn "cluster-builder not configured. Run ./scripts/setup-cluster-builder.sh first"
        exit 1
    fi
    
    # Create temp directory for test
    local test_dir
    test_dir=$(mktemp -d)
    # shellcheck disable=SC2064
    trap "rm -rf '$test_dir'" EXIT
    
    create_test_dockerfile "$test_dir"
    log_success "Created test Dockerfile in $test_dir"
    
    # Arrays to store results
    declare -a ssd_times
    declare -a tmpfs_times
    
    # ==========================================================================
    # Phase 1: Baseline (SSD) builds
    # ==========================================================================
    log_header "Phase 1: Baseline (SSD Storage)"
    
    for i in $(seq 1 "$ITERATIONS"); do
        log_info "Running SSD build iteration $i/$ITERATIONS..."
        duration=$(run_build "ssd" "$test_dir" "$i")
        ssd_times+=("$duration")
        log_success "SSD Build $i: ${duration}s"
    done
    
    # ==========================================================================
    # Phase 2: Enable tmpfs and rebuild
    # ==========================================================================
    log_header "Phase 2: RAMDisk (tmpfs) Storage"
    
    if enable_tmpfs; then
        for i in $(seq 1 "$ITERATIONS"); do
            log_info "Running tmpfs build iteration $i/$ITERATIONS..."
            duration=$(run_build "tmpfs" "$test_dir" "$i")
            tmpfs_times+=("$duration")
            log_success "tmpfs Build $i: ${duration}s"
        done
        
        # Restore original config
        log_info "Restoring original BuildKit configuration..."
        disable_tmpfs
    else
        log_warn "Could not enable tmpfs, skipping tmpfs benchmarks"
    fi
    
    # ==========================================================================
    # Results
    # ==========================================================================
    log_header "Results"
    
    # Calculate averages
    local ssd_total=0 tmpfs_total=0
    local ssd_count=${#ssd_times[@]}
    local tmpfs_count=${#tmpfs_times[@]}
    
    echo "SSD Build Times:"
    for t in "${ssd_times[@]}"; do
        echo "  - ${t}s"
        ssd_total=$(echo "$ssd_total + $t" | bc)
    done
    
    if [[ $ssd_count -gt 0 ]]; then
        local ssd_avg
        ssd_avg=$(echo "scale=2; $ssd_total / $ssd_count" | bc)
        echo -e "  ${BOLD}Average: ${ssd_avg}s${NC}"
    fi
    
    echo ""
    
    if [[ $tmpfs_count -gt 0 ]]; then
        echo "tmpfs Build Times:"
        for t in "${tmpfs_times[@]}"; do
            echo "  - ${t}s"
            tmpfs_total=$(echo "$tmpfs_total + $t" | bc)
        done
        
        local tmpfs_avg
        tmpfs_avg=$(echo "scale=2; $tmpfs_total / $tmpfs_count" | bc)
        echo -e "  ${BOLD}Average: ${tmpfs_avg}s${NC}"
        
        echo ""
        
        # Calculate improvement
        if [[ $ssd_count -gt 0 ]]; then
            local improvement
            improvement=$(echo "scale=1; (1 - $tmpfs_avg / $ssd_avg) * 100" | bc)
            local speedup
            speedup=$(echo "scale=2; $ssd_avg / $tmpfs_avg" | bc)
            
            echo -e "${CYAN}${BOLD}Performance Comparison:${NC}"
            echo -e "  SSD Average:   ${ssd_avg}s"
            echo -e "  tmpfs Average: ${tmpfs_avg}s"
            echo -e "  ${GREEN}Improvement: ${improvement}% faster (${speedup}x speedup)${NC}"
        fi
    else
        echo "No tmpfs results available"
    fi
    
    echo ""
    log_success "Benchmark complete!"
}

# Handle arguments
case "${1:-}" in
    --help|-h)
        echo "Usage: $0 [--iterations N]"
        echo ""
        echo "Benchmarks BuildKit with SSD vs RAMDisk (tmpfs) storage."
        echo ""
        echo "Options:"
        echo "  --iterations N   Number of build iterations (default: 3)"
        echo ""
        echo "Environment:"
        echo "  NAMESPACE   Kubernetes namespace (default: cto)"
        echo "  REGISTRY    Container registry URL (default: 192.168.1.72:30500)"
        exit 0
        ;;
    --iterations)
        ITERATIONS="${2:-3}"
        shift 2 || true
        ;;
esac

main "$@"

