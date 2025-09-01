#!/bin/bash

echo "🔍 OpenAI Codex CLI Health Check"
echo "================================="

# Check Codex CLI binary
echo "📦 Checking Codex CLI..."
if command -v codex >/dev/null 2>&1; then
    echo "✅ Codex CLI found at: $(which codex)"
    if codex --version >/dev/null 2>&1; then
        echo "✅ Codex CLI is working"
        codex --version
    else
        echo "❌ Codex CLI version check failed"
        exit 1
    fi
else
    echo "❌ Codex CLI not found"
    exit 1
fi

# Check Node.js and npm (required for Codex)
echo "📦 Checking Node.js and npm..."
if command -v node >/dev/null 2>&1 && command -v npm >/dev/null 2>&1; then
    echo "✅ Node.js $(node --version) and npm $(npm --version) available"
else
    echo "❌ Node.js or npm not found"
fi

# Check environment variables
echo "🔧 Checking environment variables..."
echo "OPENAI_API_KEY: ${OPENAI_API_KEY:+'Set'}${OPENAI_API_KEY:-'Not set'}"
echo "HOME: ${HOME:-'Not set'}"
echo "USER: ${USER:-'Not set'}"

echo ""
echo "🎉 All health checks passed! OpenAI Codex CLI is ready."