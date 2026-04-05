Implement subtask 3014: Create Dockerfile and Kubernetes deployment manifests

## Objective
Create multi-stage Dockerfile for the RMS service and Kubernetes Deployment, Service, and ConfigMap reference manifests for the sigma1 namespace.

## Steps
1. Create `Dockerfile`:
   - Stage 1 (builder): `FROM golang:1.22-alpine AS builder`, install ca-certificates, copy go.mod/go.sum, run `go mod download`, copy source, run `CGO_ENABLED=0 GOOS=linux go build -o /rms-server ./cmd/rms-server/`
   - Stage 2 (runtime): `FROM gcr.io/distroless/static-debian12`, copy binary from builder, copy migration files to /migrations, EXPOSE 8080 8081, ENTRYPOINT ["/rms-server"]
2. Create `deploy/deployment.yaml`:
   - Namespace: sigma1
   - Deployment: rms-server, 2 replicas
   - Container: image from registry, ports 8080 (http) and 8081 (grpc)
   - `envFrom: [{configMapRef: {name: sigma1-infra-endpoints}}]`
   - Resources: requests 128Mi/125m, limits 256Mi/250m
   - Liveness probe: httpGet /health/live port 8080, initialDelaySeconds 5
   - Readiness probe: httpGet /health/ready port 8080, initialDelaySeconds 10
3. Create `deploy/service.yaml`:
   - Service type ClusterIP, ports: 8080 (http), 8081 (grpc)
   - Named ports for Istio/service mesh compatibility
4. Create `deploy/rbac-configmap.yaml`: example `sigma1-rbac-roles` ConfigMap with default role definitions.
5. Add `.dockerignore` to exclude unnecessary files.

## Validation
Verify Docker build completes successfully and produces an image under 50MB. Verify `kubectl apply --dry-run=client -f deploy/` succeeds for all manifests. Verify container starts and both ports respond (health check on 8080, gRPC reflection on 8081).