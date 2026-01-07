# Task 8: Deploy Notification Router Service (Rex - Rust/Axum)

**Agent**: rex | **Language**: rust

## Role

You are a Senior Rust Engineer with expertise in systems programming and APIs implementing Task 8.

## Goal

Create Kubernetes manifests and deploy the Notification Router service to the cluster

## Requirements

1. Create Kubernetes manifests in k8s/notification-router/:
   - deployment.yaml: Deployment with 2 replicas, resource limits (512Mi-1Gi memory, 250m-500m CPU), health checks (liveness: /health/live, readiness: /health/ready), env vars from ConfigMaps and Secrets
   - service.yaml: ClusterIP service on port 8080
   - hpa.yaml: HorizontalPodAutoscaler targeting 70% CPU, min 2, max 10 replicas
   - configmap.yaml: ConfigMap with POSTGRES_URL, REDIS_URL, KAFKA_BOOTSTRAP_SERVERS
   - secret.yaml: Secret with JWT_SECRET, POSTGRES_PASSWORD

2. Create Dockerfile in notification-router/:
   FROM rust:1.75 AS builder
   WORKDIR /app
   COPY Cargo.toml Cargo.lock ./
   COPY src ./src
   RUN cargo build --release
   FROM debian:bookworm-slim
   RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
   COPY --from=builder /app/target/release/notification-router /usr/local/bin/
   EXPOSE 8080
   CMD ["notification-router"]

3. Build and push Docker image:
   docker build -t alerthub/notification-router:latest .
   docker push alerthub/notification-router:latest

4. Apply Kubernetes manifests:
   kubectl apply -f k8s/notification-router/

5. Verify deployment:
   kubectl get pods -l app=notification-router
   kubectl logs -l app=notification-router --tail=100
   kubectl port-forward svc/notification-router 8080:8080
   curl http://localhost:8080/health/ready

6. Configure Ingress:
   - Create Ingress resource for /api/v1/notifications path
   - Configure TLS with cert-manager
   - Add rate limiting annotations

## Acceptance Criteria

1. Verify pods are running and ready
2. Test health check endpoints
3. Test API endpoints via port-forward
4. Verify metrics endpoint returns data
5. Test HPA scaling (generate load, verify replicas increase)
6. Verify WebSocket connections work
7. Check logs for errors
8. Test database connectivity
9. Test Kafka event publishing
10. Load test with 10,000 req/min

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-8): Deploy Notification Router Service (Rex - Rust/Axum)`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 2
