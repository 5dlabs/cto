Implement subtask 3013: Create Dockerfile and Kubernetes deployment manifests

## Objective
Build the multi-stage Dockerfile and Kubernetes manifests for deploying the RMS service in the sigma1 namespace with proper configuration, secrets, and probes.

## Steps
1. Create `Dockerfile`:
   - Stage 1 (builder): `FROM golang:1.22 AS builder`, WORKDIR /app, COPY go.mod go.sum, RUN go mod download, COPY . ., RUN CGO_ENABLED=0 GOOS=linux go build -o /rms ./cmd/server/
   - Stage 2 (runtime): `FROM gcr.io/distroless/static-debian12`, COPY --from=builder /rms /rms, COPY db/migrations /migrations, EXPOSE 50051 8081, ENTRYPOINT ["/rms"]
2. Create `k8s/deployment.yaml`:
   - Namespace: sigma1, name: rms
   - Replicas: 2
   - Container ports: 50051 (grpc), 8081 (http)
   - `envFrom`: [{configMapRef: {name: sigma1-infra-endpoints}}]
   - `env` from secrets: DB credentials from sigma1-db-credentials, GOOGLE_CALENDAR_SA_KEY from sigma1-gcal-secret, SERVICE_API_KEYS from sigma1-service-api-keys.
   - Liveness probe: httpGet /health/live port 8081, initialDelaySeconds 5, periodSeconds 10.
   - Readiness probe: httpGet /health/ready port 8081, initialDelaySeconds 10, periodSeconds 5.
   - Resources: requests 128Mi/100m, limits 512Mi/500m.
3. Create `k8s/service.yaml`:
   - ClusterIP service exposing ports 50051 (grpc) and 8081 (http).
4. Create `k8s/kustomization.yaml` tying resources together.

## Validation
1) Docker build succeeds: `docker build -t rms:test .` completes without errors. 2) Docker image runs: `docker run rms:test` starts and listens (will fail on DB but should not crash). 3) Image size < 50MB (distroless base). 4) Kubernetes manifests validate: `kubectl apply --dry-run=client -f k8s/` succeeds. 5) Verify probes are configured: parse YAML and confirm liveness/readiness paths and ports.