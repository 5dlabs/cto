# Task 11: Deploy Web Console Frontend (Blaze - React/Next.js + Effect)

**Agent**: blaze | **Language**: tsx

## Role

You are a Senior Frontend Engineer with expertise in React, TypeScript, and modern UI/UX implementing Task 11.

## Goal

Build and deploy the Web Console to the cluster with CDN configuration

## Requirements

1. Create Kubernetes manifests in k8s/web-console/:
   - deployment.yaml: Deployment with 2 replicas, resource limits (128Mi-256Mi memory, 100m-200m CPU), health check (readiness: /), env vars for API URLs
   - service.yaml: ClusterIP service on port 3000
   - configmap.yaml: ConfigMap with NEXT_PUBLIC_API_URL, NEXT_PUBLIC_WS_URL

2. Update Dockerfile in web-console/ for production:
   FROM oven/bun:1.1 AS builder
   WORKDIR /app
   COPY package.json bun.lockb ./
   RUN bun install --frozen-lockfile
   COPY . .
   RUN bun run build
   FROM oven/bun:1.1
   WORKDIR /app
   COPY --from=builder /app/.next/standalone ./
   COPY --from=builder /app/.next/static ./.next/static
   COPY --from=builder /app/public ./public
   EXPOSE 3000
   CMD ["bun", "run", "server.js"]

3. Build and push Docker image:
   docker build -t alerthub/web-console:latest .
   docker push alerthub/web-console:latest

4. Apply Kubernetes manifests:
   kubectl apply -f k8s/web-console/

5. Configure Ingress:
   - Create Ingress resource for / path
   - Configure TLS with cert-manager
   - Add Cloudflare Tunnel annotations
   - Enable caching for static assets

6. Configure CDN:
   - Setup Cloudflare zone
   - Configure caching rules for /_next/static/*
   - Enable Brotli compression
   - Configure security headers

## Acceptance Criteria

1. Verify pods are running and ready
2. Test web console loads via port-forward
3. Test all pages render correctly
4. Test WebSocket connection for real-time updates
5. Test API calls to backend services
6. Test form submissions (create integration, configure rule)
7. Test Effect error handling (invalid inputs, API failures)
8. Verify responsive design on mobile
9. Test dark/light theme switching
10. Run Lighthouse performance audit (target: >90 score)

## Constraints

- Match existing codebase patterns and style
- Create PR with atomic, well-described commits
- Include unit tests for new functionality
- PR title: `feat(task-11): Deploy Web Console Frontend (Blaze - React/Next.js + Effect)`

## Resources

- PRD: `.tasks/docs/prd.txt`
- Dependencies: 5, 8, 9, 10
