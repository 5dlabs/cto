# Tools Docker Image

This directory contains the Dockerfile for the Tools MCP server.

## Overview

The Tools service is a dynamic MCP (Model Context Protocol) tool management and
proxy server. It provides intelligent routing between local and remote MCP
servers for AI development workflows.

## Included Binaries

- **tools-server** - Dynamic MCP tool management and proxy server
- **cto-mcp** - MCP server for Argo Workflows integration in AI development workflows

## Building

The image is built as part of the CTO release workflow. The binary is compiled
separately and copied into the image at build time.

```bash
# Build the tools-server binary
cd /path/to/cto
cargo build --release -p tools --bin tools-server

# Build the Docker image
cd infra/images/tools
cp ../../../target/release/tools-server tools-server-linux
docker build -t ghcr.io/5dlabs/tools:latest .
```

## Runtime Dependencies

The image includes multiple language runtimes to support various MCP servers:

- Node.js (for npx-based MCP servers)
- Python 3 with UV/uvx (for Python MCP servers)
- Go (for Go-based tools)
- Java 17 (for Java-based tools)
- Docker CLI (for container-based MCP servers)
- kubectl (for Kubernetes operations)
- Rust toolchain (for Rust development tools)

## Configuration

The server reads configuration from `/config/servers-config.json` which defines
available MCP servers. Environment variables:

- `PORT` - Server port (default: 3000)
- `PROJECT_DIR` - Configuration directory (default: /config)
- `RUST_LOG` - Log level (default: info)

## Usage

```bash
docker run -d \
  -p 3000:3000 \
  -v /path/to/config:/config \
  ghcr.io/5dlabs/tools:latest
```

