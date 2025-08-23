# Grok Agent Docker Image

A containerized version of the [Grok CLI](https://github.com/superagent-ai/grok-cli) for use in Kubernetes TaskRuns and automated workflows.



## Features

- **Grok CLI**: Conversational AI powered by Grok-3
- **Smart File Operations**: AI automatically handles file viewing/creation/editing
- **Bash Integration**: Execute shell commands through natural language
- **Git Support**: Built-in git operations with SSH key support
- **Tool Selection**: AI automatically chooses the right tools for tasks



## Usage

### Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `GROK_API_KEY` | ✅ | API key from X.AI for Grok access |
| `GIT_SSH_KEY` | ❌ | Base64-encoded SSH private key for git operations |
| `GIT_USER_NAME` | ❌ | Git user name for commits |
| `GIT_USER_EMAIL` | ❌ | Git user email for commits |
| `REPO_URL` | ❌ | Git repository URL to clone on startup |
| `WORKING_DIR` | ❌ | Directory to change to after cloning |

### Building the Image





```bash


# Build locally
./build.sh

# Build and push to registry
REGISTRY=your-registry.com ./build.sh --push








```

### Running Locally





```bash
# Interactive mode
docker run -it \


  -e GROK_API_KEY="your-api-key" \
  grok-agent:latest



# With git repository
docker run -it \


  -e GROK_API_KEY="your-api-key" \
  -e REPO_URL="git@github.com:user/repo.git" \


  -e GIT_SSH_KEY="$(cat ~/.ssh/id_ed25519 | base64 -w 0)" \


  -e GIT_USER_NAME="Your Name" \


  -e GIT_USER_EMAIL="you@example.com" \
  grok-agent:latest








```

### Kubernetes TaskRun Integration





```yaml
apiVersion: tekton.dev/v1beta1
kind: TaskRun
metadata:
  name: grok-task
spec:
  taskSpec:
    steps:
    - name: grok-agent
      image: grok-agent:latest
      env:
      - name: GROK_API_KEY
        valueFrom:
          secretKeyRef:
            name: grok-secrets
            key: api-key
      - name: REPO_URL
        value: "git@github.com:5dlabs/cto.git"
      script: |
        #!/bin/bash
        grok "Analyze the codebase and suggest improvements"








```

## Integration with Orchestrator

This image can be used as an alternative AI agent in your orchestrator platform:

1. **Documentation Generation**: Use Grok for generating project documentation
2. **Code Analysis**: Analyze codebases and suggest improvements
3. **Task Automation**: Execute complex development tasks through conversation
4. **Testing**: Generate and run tests based on natural language descriptions



## API Key Setup

Get your Grok API key from [X.AI](https://x.ai) and add it to your Kubernetes secrets:





```bash
kubectl create secret generic grok-secrets \


  --from-literal=api-key="your-grok-api-key"








```