#!/bin/bash
# Run Docker runtime benchmark on remote Mac
# Execute this from your MAIN Mac

set -euo pipefail

REMOTE_HOST="${REMOTE_HOST:-jonathon@192.168.1.90}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=========================================="
echo "Remote Docker Runtime Benchmark"
echo "=========================================="
echo "Remote host: $REMOTE_HOST"
echo ""

# Check connectivity
echo "Checking connectivity to remote Mac..."
if ! ssh -o ConnectTimeout=10 "$REMOTE_HOST" "echo 'Connected!'" 2>/dev/null; then
    echo ""
    echo "❌ Cannot connect to $REMOTE_HOST"
    echo ""
    echo "Please ensure:"
    echo "  1. Remote Mac is awake (not sleeping)"
    echo "  2. Remote Login is enabled:"
    echo "     System Preferences > Sharing > Remote Login"
    echo "  3. Your SSH key is authorized on the remote Mac"
    echo ""
    exit 1
fi

echo "✅ Connected to remote Mac"
echo ""

# Copy scripts to remote
echo "Copying benchmark scripts to remote Mac..."
scp -q "$SCRIPT_DIR/setup-remote-runtimes.sh" "$REMOTE_HOST:/tmp/"
scp -q "$SCRIPT_DIR/benchmark-docker-runtimes.sh" "$REMOTE_HOST:/tmp/"
ssh "$REMOTE_HOST" "chmod +x /tmp/*.sh"
echo "✅ Scripts copied"
echo ""

# Run setup
echo "Running setup on remote Mac..."
ssh -t "$REMOTE_HOST" "/tmp/setup-remote-runtimes.sh"
echo ""

# Ask before running benchmark (it takes a while)
read -p "Setup complete. Run benchmark now? This will take 30-60 minutes. (y/n) " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "Starting benchmark..."
    ssh -t "$REMOTE_HOST" "/tmp/benchmark-docker-runtimes.sh"
    
    # Copy results back
    echo ""
    echo "Copying results..."
    RESULTS_DIR="$SCRIPT_DIR/results"
    mkdir -p "$RESULTS_DIR"
    scp "$REMOTE_HOST:/tmp/benchmark-results-*.txt" "$RESULTS_DIR/" 2>/dev/null || true
    echo "✅ Results saved to $RESULTS_DIR/"
else
    echo "Benchmark skipped. Run manually with:"
    echo "  ssh $REMOTE_HOST '/tmp/benchmark-docker-runtimes.sh'"
fi





