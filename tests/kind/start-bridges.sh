#!/bin/bash
# Start socat bridges to expose Kind services to localhost
# This is needed for native binaries on macOS since the host can't reach Docker networks directly

set -e

KIND_NODE_IP="172.18.0.2"

echo "🌉 Starting socat bridges for Kind cluster..."
echo ""

# Function to start a bridge
start_bridge() {
    local name=$1
    local host_port=$2
    local kind_port=$3
    
    # Stop existing bridge if running
    docker rm -f "${name}-bridge" 2>/dev/null || true
    
    # Start new bridge
    docker run -d --rm \
        --name "${name}-bridge" \
        --network kind \
        -p "${host_port}:${host_port}" \
        alpine/socat tcp-listen:${host_port},fork,reuseaddr tcp-connect:${KIND_NODE_IP}:${kind_port}
    
    echo "  ✅ ${name}: localhost:${host_port} → ${KIND_NODE_IP}:${kind_port}"
}

# Start bridges for each service
start_bridge "tools" 3001 30001
start_bridge "prometheus" 9090 30090
start_bridge "grafana" 3000 30030
start_bridge "loki" 3100 30100
start_bridge "argo-workflows" 2746 30081
start_bridge "healer" 8080 30080
start_bridge "openmemory" 8082 30082

echo ""
echo "🎉 All bridges started!"
echo ""
echo "📋 Access services at:"
echo "   Tools:          http://localhost:3001"
echo "   Prometheus:     http://localhost:9090"
echo "   Grafana:        http://localhost:3000 (admin/admin)"
echo "   Loki:           http://localhost:3100"
echo "   Argo Workflows: http://localhost:2746"
echo "   Healer:         http://localhost:8080"
echo "   OpenMemory:     http://localhost:8082"
echo ""
echo "To stop all bridges: docker rm -f \$(docker ps -q --filter 'name=.*-bridge')"



