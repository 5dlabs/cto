# Task 10: Deploy Admin API Service (Grizz - Go/gRPC)

**Agent**: grizz | **Language**: go

## Role

You are a Senior Go Engineer with expertise in concurrent systems and microservices implementing Task 10.

## Goal

Create Kubernetes manifests and deploy the Admin API service to the cluster

## Requirements

1. Create Kubernetes manifests in k8s/admin-api/:
   - deployment.yaml: Deployment with 2 replicas, resource limits (256Mi-512Mi memory, 200m-400m CPU), health checks (liveness: /health, readiness: /health/ready), env vars from ConfigMaps and Secrets
   - service.yaml: ClusterIP service on port 9090 (gRPC), 8080 (REST gateway)
   - hpa.yaml: HorizontalPodAutoscaler targeting 70% CPU, min 2, max 6 replicas
   - configmap.yaml: ConfigMap with POSTGRES_URL, REDIS_URL
   - secret.yaml: Secret with JWT_PRIVATE_KEY, POSTGRES_PASSWORD

2. Create Dockerfile in admin-api/:
   FROM golang:1.22 AS builder
   WORKDIR /app
   COPY go.mod go.sum ./
   RUN go mod download
   COPY . .
   RUN CGO_ENABLED=0 GOOS=linux go build -o admin-api cmd/server/main.go
   FROM alpine:latest
   RUN apk --no-cache add ca-certificates
   COPY --from=builder /app/admin-api /usr/local/bin/
   EXPOSE 9090 8080
   CMD ["admin-api"]

3. Build and push Docker image:
   docker build -t alerthub/admin-api:latest .
   docker push alerthub/admin-api:latest

4. Apply Kubernetes manifests:
   kubectl apply -f k8s/admin-api/

5. Verify deployment:
   kubectl get pods -l app=admin-api
   kubectl logs -l app=admin-api --tail=100
   kubectl port-forward svc/admin-api 9090:9090 8080:8080
   grpcurl -plaintext localhost:9090 list
   curl http://localhost:8080/api/v1/tenants

6. Configure Ingress:
   - Create Ingress resource for /api/v1/tenants, /api/v1/users, /api/v1/rules, /api/v1/analytics paths
   - Configure TLS with cert-manager

## Acceptance Criteria

1. Verify pods are running and ready
2. Test gRPC endpoints with grpcurl
3. Test REST endpoints via grpc-gateway
4. Test JWT authentication with valid/invalid tokens
5. Test RBAC with different roles
6. Test tenant CRUD operations
7. Test user CRUD operations
8. Test rule CRUD operations
9. Test analytics endpoints
10. Verify audit logging

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-10): Deploy Admin API Service (Grizz - Go/gRPC)`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 4
