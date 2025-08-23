# Open Code Docker Image

A containerized development environment with Open Code CLI for AI-powered coding assistance.



## Features

- **Open Code CLI**: AI-powered coding assistant from SST
- **Development Tools**: Node.js 20, git, zsh, fzf, and more
- **Ready for Tasks**: Pre-configured for automated development workflows
- **Multi-platform**: Supports both AMD64 and ARM64 architectures



## Usage

### Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `ANTHROPIC_API_KEY` | ✅ | API key for Open Code access (uses Claude models) |
| `TZ` | ❌ | Timezone setting (default: system timezone) |

### Building the Image





```bash


# Build locally
docker build -t opencode:latest .

# Build with specific Open Code version
docker build --build-arg VERSION=0.5.5 -t opencode:latest .








```

### Running Locally





```bash
# Interactive development environment
docker run -it \


  -e ANTHROPIC_API_KEY="your-api-key" \
  -v $(pwd):/workspace \
  opencode:latest

# With timezone setting
docker run -it \


  -e ANTHROPIC_API_KEY="your-api-key" \


  -e TZ="America/New_York" \
  -v $(pwd):/workspace \
  opencode:latest








```



### Headless Server Mode

Open Code can run in headless server mode for automated workflows:





```bash


# Start headless server
docker run -d \


  -e ANTHROPIC_API_KEY="your-api-key" \
  -p 4096:4096 \
  opencode:latest opencode serve --port 4096 --hostname 0.0.0.0








```

### Kubernetes TaskRun Integration





```yaml
apiVersion: tekton.dev/v1beta1
kind: TaskRun
metadata:
  name: opencode-development-task
spec:
  taskSpec:
    steps:
    - name: opencode-code
      image: opencode:latest
      env:
      - name: ANTHROPIC_API_KEY
        valueFrom:
          secretKeyRef:
            name: opencode-secrets
            key: api-key
      script: |
        #!/bin/bash
        # Your development tasks here
        opencode run "Generate documentation for this project"








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
kubectl create secret generic opencode-secrets \


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


- Open Code CLI
