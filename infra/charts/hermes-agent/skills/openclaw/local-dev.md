# Local Development Services

Your pod includes a full Rust/Go/Python development environment and a service manager for running CTO platform services locally. This lets you build, test, and iterate without pushing to CI.

## CTO Service Manager

The `cto-services` script manages CTO platform services (controller, pm-server, healer) as background processes with PID files and log management.

```bash
# Start all CTO services
cto-services start all

# Start a specific service
cto-services start controller

# Check status of all services
cto-services status

# Restart a service after rebuilding
cto-services restart controller

# Tail logs for a service
cto-services logs controller

# Watch for binary changes and auto-restart
cto-services watch

# Stop everything
cto-services stop all
```

### Services

| Service | Binary | Port | Description |
|---------|--------|------|-------------|
| controller | agent-controller | 8080 | CTO controller (reconciles CodeRuns, renders templates) |
| pm-server | pm-server | 8081 | PM server (project management integration) |
| healer | healer | 8082 | Healer server (self-healing / remediation) |

### Environment Variables

The service manager sets these automatically (override via env):
- `RUST_LOG=info` — Rust log level
- `AGENT_TEMPLATES_PATH` — Path to CTO templates
- `HEALER_TEMPLATES_DIR` — Path to healer templates
- `CTO_CONFIG_PATH` — Path to cto-config.json
- `CONTROLLER_CONFIG_PATH` — Path to controller-config.yaml
- `SERVER_HOST=0.0.0.0` — Bind address

## Building CTO Binaries

```bash
cd /workspace/repos/cto

# Build all binaries in release mode
cargo build --release

# Build a specific binary
cargo build --release --bin agent-controller

# Run tests
cargo test

# Run clippy
cargo clippy --all-targets

# Run tests with nextest (faster parallel execution)
cargo nextest run

# Watch for changes and rebuild
cargo watch -x 'build --release'
```

## Docker Builds

Your pod has Docker-in-Docker access with GHCR credentials pre-configured.

```bash
# Build a Docker image
docker build -t ghcr.io/5dlabs/myimage:latest .

# Push to GHCR (credentials auto-configured)
docker push ghcr.io/5dlabs/myimage:latest

# Use remote BuildKit for faster builds
docker buildx create --name remote --driver remote $BUILDKIT_HOST
docker buildx use remote
docker buildx build --push -t ghcr.io/5dlabs/myimage:latest .

# Check Docker is working
docker info
docker ps
```

## Just Commands

If the CTO repo has a `justfile`, use `just` for common tasks:

```bash
cd /workspace/repos/cto

# List available recipes
just --list

# Common patterns
just build          # Build all
just test           # Run tests
just lint           # Run linters
just fmt            # Format code
```

## Typical Workflow

1. Build the CTO binaries: `cd /workspace/repos/cto && cargo build --release`
2. Start local services: `cto-services start all`
3. Verify they're running: `cto-services status`
4. Make code changes
5. Rebuild: `cargo build --release`
6. Restart affected service: `cto-services restart controller`
7. Or use auto-restart: `cto-services watch` (in background)
