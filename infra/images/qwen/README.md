# Qwen Code Agent Image

This Docker image extends the platform runtime base with the Qwen Code CLI tool installed. It provides an AI-powered command-line workflow for developers using Qwen models.

## Building the Image

Build locally:





```bash
docker build -t platform-qwen:latest .








```

Or use buildx for multi-platform:





```bash
docker buildx build --platform linux/amd64,linux/arm64 -t ghcr.io/5dlabs/platform-qwen:latest .








```



## Usage

Run the container with:





```bash
docker run -it --rm platform-qwen:latest








```

This will start the 'qwen' CLI. Mount volumes as needed for workspaces:





```bash
docker run -it --rm -v $(pwd):/workspace platform-qwen:latest








```

For authentication, set environment variables or use volumes for configuration files.

## Dependencies

- Base: platform-runtime
- Node.js: From base image
- Qwen Code: Installed via npm global

See the main platform README for integration details.
