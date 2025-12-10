#!/bin/bash
# Docker Runtime Benchmark Script
# Compares Docker Desktop, OrbStack, and Colima for Rust builds
#
# Run this ON the remote Mac after running setup-remote-runtimes.sh

set -euo pipefail

# Configuration
REPO_URL="https://github.com/5dlabs/cto.git"
REPO_DIR="/tmp/cto-benchmark"
DOCKERFILE="infra/images/pm-server/Dockerfile.build"
IMAGE_NAME="benchmark-pm-server"
RESULTS_FILE="/tmp/benchmark-results-$(date +%Y%m%d-%H%M%S).txt"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() { echo -e "${BLUE}[$(date +%H:%M:%S)]${NC} $1"; }
success() { echo -e "${GREEN}✅ $1${NC}"; }
warn() { echo -e "${YELLOW}⚠️  $1${NC}"; }
error() { echo -e "${RED}❌ $1${NC}"; }

# Clone or update repo
setup_repo() {
    log "Setting up repository..."
    if [ -d "$REPO_DIR" ]; then
        cd "$REPO_DIR"
        git fetch origin main
        git reset --hard origin/main
    else
        git clone --depth 1 "$REPO_URL" "$REPO_DIR"
        cd "$REPO_DIR"
    fi
    success "Repository ready at $REPO_DIR"
}

# Time a build
time_build() {
    local runtime=$1
    local build_type=$2
    local start end duration
    
    log "[$runtime] Starting $build_type build..."
    
    start=$(date +%s.%N)
    
    if docker build \
        --platform linux/amd64 \
        -t "$IMAGE_NAME:$runtime-$build_type" \
        -f "$DOCKERFILE" \
        . > /tmp/build-$runtime-$build_type.log 2>&1; then
        
        end=$(date +%s.%N)
        duration=$(echo "$end - $start" | bc)
        success "[$runtime] $build_type build: ${duration}s"
        echo "$runtime,$build_type,$duration" >> "$RESULTS_FILE"
    else
        error "[$runtime] $build_type build failed! Check /tmp/build-$runtime-$build_type.log"
        echo "$runtime,$build_type,FAILED" >> "$RESULTS_FILE"
    fi
}

# Clean Docker cache for cold builds
clean_cache() {
    local runtime=$1
    log "[$runtime] Cleaning build cache..."
    docker builder prune -af > /dev/null 2>&1 || true
    docker image rm -f "$IMAGE_NAME:$runtime-cold" > /dev/null 2>&1 || true
}

# Start a runtime
start_runtime() {
    local runtime=$1
    
    case $runtime in
        "docker-desktop")
            log "Starting Docker Desktop..."
            # Stop others first
            colima stop 2>/dev/null || true
            orb stop 2>/dev/null || true
            sleep 2
            
            open -a Docker
            log "Waiting for Docker Desktop to start..."
            while ! docker info > /dev/null 2>&1; do
                sleep 2
            done
            success "Docker Desktop ready"
            ;;
            
        "orbstack")
            log "Starting OrbStack..."
            # Stop others first
            osascript -e 'quit app "Docker"' 2>/dev/null || true
            colima stop 2>/dev/null || true
            sleep 2
            
            open -a OrbStack
            log "Waiting for OrbStack to start..."
            while ! docker info > /dev/null 2>&1; do
                sleep 2
            done
            success "OrbStack ready"
            ;;
            
        "colima")
            log "Starting Colima..."
            # Stop others first
            osascript -e 'quit app "Docker"' 2>/dev/null || true
            osascript -e 'quit app "OrbStack"' 2>/dev/null || true
            sleep 2
            
            # Start Colima with optimal settings for M1
            colima start \
                --cpu 8 \
                --memory 12 \
                --disk 60 \
                --vm-type vz \
                --vz-rosetta \
                --mount-type virtiofs
            
            success "Colima ready"
            ;;
    esac
    
    # Show Docker info
    docker info 2>/dev/null | grep -E "Server Version|Operating System|CPUs|Total Memory" || true
}

# Stop a runtime
stop_runtime() {
    local runtime=$1
    
    case $runtime in
        "docker-desktop")
            osascript -e 'quit app "Docker"' 2>/dev/null || true
            ;;
        "orbstack")
            osascript -e 'quit app "OrbStack"' 2>/dev/null || true
            ;;
        "colima")
            colima stop 2>/dev/null || true
            ;;
    esac
    sleep 3
}

# Run benchmark for a single runtime
benchmark_runtime() {
    local runtime=$1
    
    echo ""
    echo "=========================================="
    echo "Benchmarking: $runtime"
    echo "=========================================="
    
    start_runtime "$runtime"
    
    cd "$REPO_DIR"
    
    # Cold build (no cache)
    clean_cache "$runtime"
    time_build "$runtime" "cold"
    
    # Warm build (with cache, rebuild same)
    time_build "$runtime" "warm"
    
    # Incremental build (small change)
    log "[$runtime] Making small code change for incremental build..."
    echo "// Benchmark timestamp: $(date)" >> crates/pm/src/main.rs
    time_build "$runtime" "incremental"
    
    # Reset the change
    git checkout crates/pm/src/main.rs
    
    stop_runtime "$runtime"
}

# Print results summary
print_results() {
    echo ""
    echo "=========================================="
    echo "BENCHMARK RESULTS"
    echo "=========================================="
    echo ""
    echo "Runtime,Build Type,Duration (seconds)"
    cat "$RESULTS_FILE"
    echo ""
    echo "Results saved to: $RESULTS_FILE"
    
    # Calculate averages if possible
    echo ""
    echo "--- Summary ---"
    for runtime in docker-desktop orbstack colima; do
        local cold warm incr
        cold=$(grep "^$runtime,cold," "$RESULTS_FILE" | cut -d',' -f3)
        warm=$(grep "^$runtime,warm," "$RESULTS_FILE" | cut -d',' -f3)
        incr=$(grep "^$runtime,incremental," "$RESULTS_FILE" | cut -d',' -f3)
        
        if [ -n "$cold" ] && [ "$cold" != "FAILED" ]; then
            echo "$runtime: cold=${cold}s, warm=${warm}s, incremental=${incr}s"
        fi
    done
}

# Main
main() {
    echo "=========================================="
    echo "Docker Runtime Benchmark"
    echo "$(date)"
    echo "=========================================="
    
    # Initialize results file
    echo "runtime,build_type,duration_seconds" > "$RESULTS_FILE"
    
    # Setup
    setup_repo
    
    # Check what's available
    local runtimes=()
    
    if [ -d "/Applications/Docker.app" ]; then
        runtimes+=("docker-desktop")
    else
        warn "Docker Desktop not installed, skipping"
    fi
    
    if command -v orb &> /dev/null; then
        runtimes+=("orbstack")
    else
        warn "OrbStack not installed, skipping"
    fi
    
    if command -v colima &> /dev/null; then
        runtimes+=("colima")
    else
        warn "Colima not installed, skipping"
    fi
    
    if [ ${#runtimes[@]} -eq 0 ]; then
        error "No Docker runtimes found! Run setup-remote-runtimes.sh first."
        exit 1
    fi
    
    log "Will benchmark: ${runtimes[*]}"
    
    # Run benchmarks
    for runtime in "${runtimes[@]}"; do
        benchmark_runtime "$runtime"
    done
    
    # Print results
    print_results
}

# Run
main "$@"





