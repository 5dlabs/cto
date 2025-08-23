

# Claude Code Docker Image

A containerized development environment with Claude Code CLI for AI-powered coding assistance.



## Features

- **Claude Code CLI**: AI-powered coding assistant from Anthropic
- **Development Tools**: Node.js 20, git, zsh, fzf, and more
- **Ready for Tasks**: Pre-configured for automated development workflows
- **Multi-platform**: Supports both AMD64 and ARM64 architectures



## Usage

### Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `ANTHROPIC_API_KEY` | ✅ | API key for Claude Code access |
| `TZ` | ❌ | Timezone setting (default: system timezone) |

### Building the Image





```bash


# Build locally
docker build -t claude-code:latest .

# Build with specific Claude version
docker build --build-arg CLAUDE_CODE_VERSION=1.2.3 -t claude-code:latest .








```

### Running Locally





```bash
# Interactive development environment
docker run -it \


  -e ANTHROPIC_API_KEY="your-api-key" \
  -v $(pwd):/workspace \
  claude-code:latest

# With timezone setting
docker run -it \


  -e ANTHROPIC_API_KEY="your-api-key" \


  -e TZ="America/New_York" \
  -v $(pwd):/workspace \
  claude-code:latest








```

### Kubernetes TaskRun Integration





```yaml
apiVersion: tekton.dev/v1beta1
kind: TaskRun
metadata:
  name: claude-development-task
spec:
  taskSpec:
    steps:
    - name: claude-code
      image: claude-code:latest
      env:
      - name: ANTHROPIC_API_KEY
        valueFrom:
          secretKeyRef:
            name: claude-secrets
            key: api-key
      script: |
        #!/bin/bash
        # Your development tasks here
        claude-code "Generate documentation for this project"








```

## Integration with Orchestrator

This image is used by the orchestrator platform for:

1. **Documentation Generation**: Automated project documentation
2. **Code Analysis**: Intelligent code review and suggestions
3. **Task Automation**: AI-powered development task execution
4. **Testing**: Automated test generation and execution



## API Key Setup

Get your Claude API key from [Anthropic](https://console.anthropic.com) and add it to your secrets:





```bash
kubectl create secret generic claude-secrets \


  --from-literal=api-key="your-claude-api-key"








```

## Included Tools



- Node.js 20


- npm/npx


- git


- zsh with powerline10k


- fzf (fuzzy finder)


- gh (GitHub CLI)


- jq (JSON processor)


- Standard development utilities