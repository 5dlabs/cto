# Talos Bare Metal Orchestrator

Automated Talos Linux installation on Scaleway bare metal via Rust.

## Quick Start

```bash
# Build
cd talos-orchestrator
cargo build --release

# Download dependencies
./scripts/download-talosctl arm64 1.8.0 ./talosctl
./scripts/download-talos-image arm64 1.8.0 ./talos.raw.img.gz

# Install (requires Scaleway credentials)
export SCALEWAY_PROJECT_ID="your-project-id"
export SCALEWAY_ACCESS_KEY="scw-xxx"
export SCALEWAY_SECRET_KEY="xxx"

./target/release/talos-orchestrator install --server-id "server-uuid" --disk /dev/sda
```

## Binary Usage

```bash
talos-orchestrator install \
  --config config.yaml \
  --server-id "server-uuid" \
  --disk /dev/sda
```

## Configuration

```yaml
# config.yaml
scaleway:
  project_id: "your-project-id"
  access_key: "scw-xxx"
  secret_key: "xxx"

server:
  id: "server-uuid"
  disk: "/dev/sda"

talos:
  version: "1.8.0"
  architecture: "arm64"
```

## Project Structure

```
talos-orchestrator/
├── Cargo.toml
├── src/
│   ├── main.rs           # CLI entrypoint
│   ├── lib.rs            # Library exports
│   ├── error.rs          # Error types
│   ├── config/mod.rs      # Configuration
│   ├── scaleway/         # Scaleway API client
│   ├── ssh/              # Rescue mode SSH
│   ├── talosctl/         # talosctl wrapper
│   └── state/            # State machine
├── scripts/
│   ├── download-talosctl.sh
│   └── download-talos-image.sh
└── config.example.yaml
```

## State Machine

```
ready → rescue_mode → rsync_image → dd_write → installing → booting → bootstrapped
```

## Requirements

- Rust 1.70+
- OpenSSL
- Scaleway bare metal API access
