#!/bin/bash
# Setup script for Docker runtime benchmarking on remote Mac
# Run this ON the remote Mac (192.168.1.90)

set -euo pipefail

echo "=========================================="
echo "Docker Runtime Benchmark Setup"
echo "=========================================="

# Check if Homebrew is installed
if ! command -v brew &> /dev/null; then
    echo "❌ Homebrew not installed. Installing..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
else
    echo "✅ Homebrew installed"
fi

# Check/Install OrbStack
echo ""
echo "--- OrbStack ---"
if command -v orb &> /dev/null; then
    echo "✅ OrbStack already installed"
    orb version
else
    echo "📦 Installing OrbStack..."
    brew install --cask orbstack
    echo "✅ OrbStack installed"
fi

# Check/Install Colima
echo ""
echo "--- Colima ---"
if command -v colima &> /dev/null; then
    echo "✅ Colima already installed"
    colima version
else
    echo "📦 Installing Colima..."
    brew install colima
    echo "✅ Colima installed"
fi

# Check Docker Desktop
echo ""
echo "--- Docker Desktop ---"
if [ -d "/Applications/Docker.app" ]; then
    echo "✅ Docker Desktop installed"
else
    echo "⚠️  Docker Desktop not found"
fi

# Check Docker CLI
if command -v docker &> /dev/null; then
    echo "✅ Docker CLI available"
    docker --version 2>/dev/null || echo "   (not currently running)"
else
    echo "📦 Installing Docker CLI..."
    brew install docker
fi

# Show system info
echo ""
echo "=========================================="
echo "System Information"
echo "=========================================="
echo "CPU: $(sysctl -n machdep.cpu.brand_string)"
echo "Cores: $(sysctl -n hw.ncpu)"
echo "Memory: $(( $(sysctl -n hw.memsize) / 1024 / 1024 / 1024 )) GB"
echo "macOS: $(sw_vers -productVersion)"

echo ""
echo "=========================================="
echo "Setup Complete!"
echo "=========================================="
echo ""
echo "Next steps:"
echo "1. Run: ./benchmark-docker-runtimes.sh"
echo ""





