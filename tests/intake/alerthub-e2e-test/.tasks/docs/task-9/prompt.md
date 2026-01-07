# Task 9: Deploy Integration Service (Nova - Bun/Elysia + Effect)

**Agent**: nova | **Language**: typescript

## Role

You are a Senior Node.js Engineer with expertise in server-side JavaScript and APIs implementing Task 9.

## Goal

Create Kubernetes manifests and deploy the Integration Service to the cluster

## Requirements

1. Create Kubernetes manifests in k8s/integration-service/:
   - deployment.yaml: Deployment with 2 replicas, resource limits (256Mi-512Mi memory, 200m-400m CPU), health check (readiness: /health), env vars from ConfigMaps and Secrets
   - service.yaml: ClusterIP service on port 3000
   - hpa.yaml: HorizontalPodAutoscaler targeting 70% CPU, min 2, max 8 replicas
   - configmap.yaml: ConfigMap with MONGODB_URL, RABBITMQ_URL, KAFKA_BOOTSTRAP_SERVERS
   - secret.yaml: Secret with SLACK_CLIENT_SECRET, SENDGRID_API_KEY, FCM_SERVICE_ACCOUNT

2. Create Dockerfile in integration-service/:
   FROM oven/bun:1.1
   WORKDIR /app
   COPY package.json bun.lockb ./
   RUN bun install --frozen-lockfile --production
   COPY . .
   EXPOSE 3000
   CMD ["bun", "run", "src/index.ts"]

3. Build and push Docker image:
   docker build -t alerthub/integration-service:latest .
   docker push alerthub/integration-service:latest

4. Apply Kubernetes manifests:
   kubectl apply -f k8s/integration-service/

5. Verify deployment:
   kubectl get pods -l app=integration-service
   kubectl logs -l app=integration-service --tail=100
   kubectl port-forward svc/integration-service 3000:3000
   curl http://localhost:3000/health

6. Verify Kafka consumer:
   - Check logs for "Kafka consumer started"
   - Publish test message to alerthub.notifications.created
   - Verify message is consumed and processed

7. Verify RabbitMQ consumer:
   - Check logs for "RabbitMQ consumer started"
   - Publish test message to integration.slack.delivery queue
   - Verify message is consumed and delivered

## Acceptance Criteria

1. Verify pods are running and ready
2. Test health check endpoint
3. Test integration CRUD endpoints
4. Test delivery endpoint with mock integration
5. Verify Kafka consumer processes messages
6. Verify RabbitMQ consumer processes tasks
7. Test retry logic with failing deliveries
8. Verify Effect error handling
9. Check logs for errors
10. Load test delivery throughput

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-9): Deploy Integration Service (Nova - Bun/Elysia + Effect)`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 3
