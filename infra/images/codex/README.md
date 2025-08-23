# OpenAI Codex Agent

This image provides OpenAI Codex integration for code generation and analysis tasks.



## Features



- OpenAI CLI tools (installed in dedicated virtualenv)


- OpenAI Python SDK (isolated installation)


- OpenAI Node.js SDK (global installation)


- Codex-specific utilities from the official repository


- Pre-configured environment for OpenAI API access

## Environment Variables

The following environment variables can be configured:



- `OPENAI_API_KEY` - Your OpenAI API key (required)
- `OPENAI_API_BASE` - OpenAI API base URL (default: https://api.openai.com/v1)
- `OPENAI_MODEL` - Default model to use (default: gpt-4)


- `OPENAI_MAX_TOKENS` - Maximum tokens for responses


- `OPENAI_TEMPERATURE` - Temperature setting for model responses



## Usage

### Command Line Interface





```bash
# Test OpenAI CLI
openai --help



# List available models
openai api models.list

# Generate code completion
openai api completions.create -m gpt-4 -p "def fibonacci(n):"








```

### Python SDK





```python
import openai



# Set your API key
openai.api_key = "your-api-key-here"

# Generate code
response = openai.Completion.create(
    engine="gpt-4",
    prompt="def fibonacci(n):",
    max_tokens=100
)








```



### Node.js SDK





```javascript
const { OpenAI } = require('openai');

const openai = new OpenAI({
  apiKey: process.env.OPENAI_API_KEY,
});

// Generate code completion
const completion = await openai.completions.create({
  model: 'gpt-4',
  prompt: 'def fibonacci(n):',
  max_tokens: 100,
});








```

## Building

This image is automatically built from the base runtime image and includes all necessary OpenAI tools and SDKs.

## Integration with GitHub Actions

The image is built automatically when:


- Changes are made to this directory


- Daily scheduled builds run


- Manual workflow dispatch is triggered

Monitor builds at: https://github.com/your-org/your-repo/actions