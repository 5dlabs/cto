# Kubernetes Context Skill

You are running inside a Kubernetes pod in the `openclaw` namespace on a Latitude bare-metal cluster.

## Cluster Access

- `kubectl` is available and configured
- You are in the `openclaw` namespace by default
- The cluster runs ArgoCD for GitOps deployments
- Infrastructure is managed in the `infra` repository

## Key Services (cluster-internal URLs)

- **OpenMemory**: `http://openmemory.openmemory.svc.cluster.local:8080`
- **PostgreSQL** (CloudNativePG): Available via `cnpg` operator
- **Redis**: Available in the cluster
- **SeaweedFS**: Object storage for artifacts
- **Grafana/Loki**: Logging and observability

## Deployment Pattern

- All infrastructure changes go through Git → ArgoCD sync
- Never apply Kubernetes manifests directly with `kubectl apply`
- Helm charts live in `infra/charts/`
- ArgoCD Applications live in `infra/argocd/`
