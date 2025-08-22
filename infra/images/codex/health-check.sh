#!/bin/bash

echo "🔍 OpenAI Codex Agent Health Check"
echo "=================================="

# Activate virtual environment
source /opt/openai/.venv/bin/activate

# Check Python OpenAI SDK
echo "📦 Checking OpenAI Python SDK..."
if python -c "import openai; print(f'✅ OpenAI SDK version: {openai.__version__}')" 2>/dev/null; then
    echo "✅ OpenAI Python SDK is working"
else
    echo "❌ OpenAI Python SDK failed"
    exit 1
fi

# Check OpenAI CLI
echo "🖥️  Checking OpenAI CLI..."
if openai --help >/dev/null 2>&1; then
    echo "✅ OpenAI CLI is working"
else
    echo "❌ OpenAI CLI failed"
    exit 1
fi

# Check Node.js OpenAI SDK
echo "📦 Checking OpenAI Node.js SDK..."
if node -e "const openai = require('openai'); console.log('✅ OpenAI Node.js SDK is available')" 2>/dev/null; then
    echo "✅ OpenAI Node.js SDK is working"
else
    echo "❌ OpenAI Node.js SDK failed"
    exit 1
fi

# Check environment variables
echo "🔧 Checking environment variables..."
echo "OPENAI_API_BASE: ${OPENAI_API_BASE:-'Not set'}"
echo "OPENAI_MODEL: ${OPENAI_MODEL:-'Not set'}"
echo "VIRTUAL_ENV: ${VIRTUAL_ENV:-'Not set'}"

echo ""
echo "🎉 All health checks passed! OpenAI Codex agent is ready."