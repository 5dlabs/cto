#!/usr/bin/env bash
# =============================================================================
# BuildKit RAMDisk Benchmark - Kubernetes Version
# =============================================================================
# Runs the benchmark directly on the cluster using a Job, avoiding network
# connectivity requirements from the local machine.
#
# Usage:
#   ./scripts/benchmark-buildkit-k8s.sh [ssd|tmpfs|both]
#
set -euo pipefail

NAMESPACE="${NAMESPACE:-cto}"
MODE="${1:-both}"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[OK]${NC} $1"; }
log_header() { echo -e "\n${CYAN}${BOLD}=== $1 ===${NC}\n"; }

# =============================================================================
# Create benchmark Job manifest
# =============================================================================
create_job_manifest() {
    local job_name="$1"
    local use_tmpfs="$2"
    
    local tmpfs_volume=""
    local tmpfs_mount=""
    
    if [[ "$use_tmpfs" == "true" ]]; then
        tmpfs_volume='{"name": "tmpfs-workspace", "emptyDir": {"medium": "Memory", "sizeLimit": "8Gi"}}'
        tmpfs_mount='{"name": "tmpfs-workspace", "mountPath": "/workspace"}'
    fi
    
    cat << EOF
apiVersion: batch/v1
kind: Job
metadata:
  name: ${job_name}
  namespace: ${NAMESPACE}
spec:
  ttlSecondsAfterFinished: 300
  backoffLimit: 0
  template:
    spec:
      restartPolicy: Never
      containers:
        - name: benchmark
          image: moby/buildkit:latest
          command:
            - /bin/sh
            - -c
            - |
              set -e
              echo "=== BuildKit Benchmark (tmpfs=${use_tmpfs}) ==="
              
              if [ "${use_tmpfs}" = "true" ]; then
                WORK_DIR="/workspace"
              else
                WORK_DIR="/var/lib/buildkit/benchmark"
              fi
              mkdir -p "\$WORK_DIR"
              cd "\$WORK_DIR"
              
              # Create test files (simulate I/O intensive build)
              echo "Creating test files..."
              START=\$(date +%s.%N)
              
              # Write 100 small files
              for i in \$(seq 1 100); do
                dd if=/dev/urandom of="small_\$i.bin" bs=1K count=100 2>/dev/null
              done
              
              # Write 5 large files
              for i in \$(seq 1 5); do
                dd if=/dev/urandom of="large_\$i.bin" bs=1M count=50 2>/dev/null
              done
              
              # Read all files
              echo "Reading files..."
              md5sum *.bin > checksums.txt
              
              # Random read/write operations
              echo "Random I/O operations..."
              for i in \$(seq 1 50); do
                cat "small_\$((RANDOM % 100 + 1)).bin" > /dev/null
                dd if=/dev/urandom of="tmp_\$i.bin" bs=1K count=50 2>/dev/null
              done
              
              END=\$(date +%s.%N)
              DURATION=\$(echo "\$END - \$START" | bc)
              
              echo ""
              echo "=== Results ==="
              echo "Mode: ${use_tmpfs:+tmpfs (RAM)}${use_tmpfs:-SSD}"
              echo "Duration: \${DURATION}s"
              echo "Files created: \$(ls -1 *.bin | wc -l)"
              echo "Total size: \$(du -sh . | cut -f1)"
              
              # Cleanup
              rm -rf "\$WORK_DIR"/*
          resources:
            requests:
              cpu: "1"
              memory: "2Gi"
            limits:
              cpu: "4"
              memory: "10Gi"
          volumeMounts:
            - name: buildkit-data
              mountPath: /var/lib/buildkit
            ${tmpfs_mount:+- $tmpfs_mount}
      volumes:
        - name: buildkit-data
          emptyDir: {}
        ${tmpfs_volume:+- $tmpfs_volume}
EOF
}

# =============================================================================
# Run benchmark job
# =============================================================================
run_benchmark() {
    local mode="$1"
    local use_tmpfs="false"
    [[ "$mode" == "tmpfs" ]] && use_tmpfs="true"
    
    local job_name
    job_name="buildkit-benchmark-${mode}-$(date +%s)"
    
    log_info "Creating benchmark job: $job_name (mode: $mode)"
    
    # Create and apply job
    create_job_manifest "$job_name" "$use_tmpfs" | kubectl apply -f -
    
    # Wait for job to complete
    log_info "Waiting for job to complete..."
    kubectl wait --for=condition=complete --timeout=300s "job/$job_name" -n "$NAMESPACE" || {
        log_info "Job may have failed, checking logs..."
    }
    
    # Get logs
    log_header "Results ($mode)"
    kubectl logs "job/$job_name" -n "$NAMESPACE" 2>/dev/null || echo "(no logs available)"
    
    # Cleanup
    kubectl delete job "$job_name" -n "$NAMESPACE" --ignore-not-found=true &>/dev/null
}

# =============================================================================
# Main
# =============================================================================
main() {
    log_header "BuildKit RAMDisk Benchmark (Kubernetes)"
    
    echo "Mode: $MODE"
    echo "Namespace: $NAMESPACE"
    echo ""
    
    case "$MODE" in
        ssd)
            run_benchmark "ssd"
            ;;
        tmpfs)
            run_benchmark "tmpfs"
            ;;
        both|*)
            run_benchmark "ssd"
            echo ""
            run_benchmark "tmpfs"
            
            log_header "Summary"
            echo "Compare the Duration values above to see the performance difference."
            echo "tmpfs (RAM) should be faster for I/O intensive operations."
            ;;
    esac
    
    log_success "Benchmark complete!"
}

case "${1:-}" in
    --help|-h)
        echo "Usage: $0 [ssd|tmpfs|both]"
        echo ""
        echo "Runs BuildKit I/O benchmark directly on the cluster."
        echo ""
        echo "Modes:"
        echo "  ssd    - Run with SSD-backed storage only"
        echo "  tmpfs  - Run with RAM-backed storage only"
        echo "  both   - Run both and compare (default)"
        exit 0
        ;;
esac

main

