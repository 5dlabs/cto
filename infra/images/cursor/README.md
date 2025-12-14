

# Cursor CLI Docker Image

A containerized development environment with Cursor CLI (`cursor-agent`) for AI-powered coding assistance in the terminal.



## Features

- **Cursor CLI**: Terminal-first AI coding assistant (`cursor-agent`)
- **Development Tools**: Node.js 20, git, zsh, fzf, Rust toolchain, and more
- **Ready for Tasks**: Pre-configured for automated development workflows
- **Multi-platform**: Supports both AMD64 and ARM64 architectures



## Usage

### Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `TZ` | ❌ | Timezone setting (default: system timezone) |

### Building the Image





```bash


# Build locally
docker build -t cursor-agent:latest .








```

### Running Locally





```bash
# Interactive development environment
docker run -it \
  -v $(pwd):/workspace \
  cursor-agent:latest

# Example: start an interactive Cursor session
docker run -it \
  -v $(pwd):/workspace \
  cursor-agent:latest \
  cursor-agent "refactor the auth module to use JWT tokens"








```

### Kubernetes TaskRun Integration





```yaml
apiVersion: tekton.dev/v1beta1
kind: TaskRun
metadata:
  name: cursor-cli-task
spec:
  taskSpec:
    steps:
    - name: cursor-agent
      image: cursor-agent:latest
      script: |
        #!/bin/bash
        set -euo pipefail
        cursor-agent -p "review these changes for security issues" --output-format text








```

## Included Tools



- Cursor CLI (`cursor-agent`)


- Node.js 20


- git


- zsh with powerline10k


- fzf (fuzzy finder)


- gh (GitHub CLI)


- jq (JSON processor)


- Standard development utilities


