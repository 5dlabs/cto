#!/bin/bash

echo "Testing OpenAI CLI installation..."

# Test if openai command is available
if command -v openai >/dev/null 2>&1; then
    echo "✅ OpenAI CLI is installed"
    openai --version
else
    echo "❌ OpenAI CLI not found"
    exit 1
fi

# Test if Python OpenAI package is available
if python3 -c "import openai; print(f'✅ OpenAI Python package version: {openai.__version__}')" 2>/dev/null; then
    echo "✅ OpenAI Python package is available"
else
    echo "❌ OpenAI Python package not found"
    exit 1
fi

# Test if Node.js OpenAI package is available
if node -e "const openai = require('openai'); console.log('✅ OpenAI Node.js package is available')" 2>/dev/null; then
    echo "✅ OpenAI Node.js package is available"
else
    echo "❌ OpenAI Node.js package not found"
    exit 1
fi

echo "🎉 All OpenAI tools are properly installed!"