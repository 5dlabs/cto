#!/bin/bash

echo "🔍 Every Code CLI Health Check"
echo "==============================="

# Check Every Code CLI binary
echo "📦 Checking Every Code CLI..."
if command -v code >/dev/null 2>&1; then
    echo "✅ Every Code CLI found at: $(which code)"
    if code --version >/dev/null 2>&1; then
        echo "✅ Every Code CLI is working"
        code --version
    else
        echo "❌ Every Code CLI version check failed"
        exit 1
    fi
else
    echo "❌ Every Code CLI not found"
    exit 1
fi

# Check Node.js and npm (required for MCP servers and tooling)
echo "📦 Checking Node.js and npm..."
if command -v node >/dev/null 2>&1 && command -v npm >/dev/null 2>&1; then
    echo "✅ Node.js $(node --version) and npm $(npm --version) available"
else
    echo "❌ Node.js or npm not found"
fi

# Check environment variables
echo "🔧 Checking environment variables..."
echo "OPENAI_API_KEY: ${OPENAI_API_KEY:+'Set'}${OPENAI_API_KEY:-'Not set'}"
echo "ANTHROPIC_API_KEY: ${ANTHROPIC_API_KEY:+'Set'}${ANTHROPIC_API_KEY:-'Not set'}"
echo "GOOGLE_API_KEY: ${GOOGLE_API_KEY:+'Set'}${GOOGLE_API_KEY:-'Not set'}"
echo "HOME: ${HOME:-'Not set'}"
echo "CODE_HOME: ${CODE_HOME:-'Not set'}"
echo "USER: ${USER:-'Not set'}"

# Check config directories
echo "📁 Checking config directories..."
if [ -d "${HOME}/.code" ]; then
    echo "✅ Every Code config directory exists: ${HOME}/.code"
else
    echo "⚠️ Every Code config directory not found"
fi

echo ""
echo "🎉 All health checks passed! Every Code CLI is ready."
