# OpenHands Docker Image

A containerized development environment with OpenHands for both headless and interactive CLI workflows.



## Features

- **OpenHands modes**: Headless and Interactive CLI
- **Dev Tools**: Node.js 20 via nvm, Rust toolchain, git, zsh+fzf, gh, jq, docker client
- **Multi-arch**: linux/amd64 and linux/arm64 ready



## Build





```bash
docker build -t openhands-agent:latest .








```

## Run (Headless mode)





```bash
docker run -it \
  -e SANDBOX_RUNTIME_CONTAINER_IMAGE=docker.all-hands.dev/all-hands-ai/runtime:0.51-nikolaik \


  -e SANDBOX_USER_ID=$(id -u) \
  -e SANDBOX_VOLUMES=$(pwd):/workspace:rw \


  -e LLM_MODEL="anthropic/claude-sonnet-4-20250514" \


  -e LLM_API_KEY="$ANTHROPIC_API_KEY" \
  -v /var/run/docker.sock:/var/run/docker.sock \
  -v ~/.openhands:/.openhands \
  openhands-agent:latest \
  python -m openhands.core.main -t "write a bash script that prints hi"








```

## Run (Interactive CLI)





```bash
docker run -it \
  -e SANDBOX_RUNTIME_CONTAINER_IMAGE=docker.all-hands.dev/all-hands-ai/runtime:0.51-nikolaik \


  -e SANDBOX_USER_ID=$(id -u) \
  -e SANDBOX_VOLUMES=$(pwd):/workspace:rw \
  -v /var/run/docker.sock:/var/run/docker.sock \
  -v ~/.openhands:/.openhands \
  openhands-agent:latest \
  python -m openhands.cli.main --override-cli-mode true








```

## Model Configuration

- Set provider-prefixed model in `LLM_MODEL`, pass provider key via `LLM_API_KEY`:


  - `LLM_MODEL="anthropic/claude-sonnet-4-20250514"`, `LLM_API_KEY=$ANTHROPIC_API_KEY`


  - `LLM_MODEL="openai/o4-mini"`, `LLM_API_KEY=$OPENAI_API_KEY`


  - `LLM_MODEL="gemini-2.5-pro"`, `LLM_API_KEY=$GOOGLE_API_KEY`


  - `LLM_MODEL="openhands/qwen3-coder-480b"`, `LLM_API_KEY=$OPENHANDS_KEY`
  - Optional: `LLM_BASE_URL`, `LLM_API_VERSION`, `LLM_DISABLE_VISION`, retry knobs


