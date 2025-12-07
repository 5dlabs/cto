# Kind Local Deployment

This folder contains Helm values files for deploying CTO services to a local Kind cluster.

## Prerequisites

1. Kind cluster running (see `infra/scripts/kind-setup.sh`)
2. Docker images built and loaded:
   ```bash
   # Build and load images to kind
   docker build -t ghcr.io/5dlabs/tools:kind-local -f infra/images/tools/Dockerfile.kind .
   docker build -t ghcr.io/5dlabs/healer:kind-local -f infra/images/healer/Dockerfile.kind .
   docker build -t ghcr.io/5dlabs/openmemory:kind-local -f infra/images/openmemory/Dockerfile .
   
   kind load docker-image ghcr.io/5dlabs/tools:kind-local
   kind load docker-image ghcr.io/5dlabs/healer:kind-local
   kind load docker-image ghcr.io/5dlabs/openmemory:kind-local
   ```

## Deployment

Deploy all services:
```bash
./deploy.sh
```

Or deploy individually:
```bash
# Tools server
helm upgrade --install tools ../../infra/charts/tools \
  -f tools-values.yaml \
  -n cto --create-namespace

# Healer resources (RBAC, PVC, ConfigMap)
kubectl apply -f healer-resources.yaml

# Healer server
helm upgrade --install healer ../../infra/charts/universal-app \
  -f healer-values.yaml \
  -n cto

# OpenMemory server
helm upgrade --install openmemory ../../infra/charts/openmemory \
  -f openmemory-values.yaml \
  -n cto
```

## Uninstall

```bash
./uninstall.sh
```

## Services

| Service | Description | Port |
|---------|-------------|------|
| tools | MCP server aggregator/proxy | 3000 |
| healer | Self-healing platform monitor | 8080 |
| openmemory | Long-term memory for AI agents | 8080 |

## Port Forwarding

```bash
# Tools server
kubectl port-forward svc/tools -n cto 3000:3000

# Healer server
kubectl port-forward svc/healer -n cto 8080:8080

# OpenMemory server
kubectl port-forward svc/openmemory -n cto 8081:8080
```



