# Publishing & Deployment

You are publishing the completed application and making it accessible via public URLs.

## 🚨 CRITICAL FIRST STEP - MERGE ALL PRS 🚨

**⛔ YOU MUST COMPLETE THIS BEFORE DEPLOYING ⛔**

### STEP 1: MERGE ALL OUTSTANDING PRS TO MAIN (MANDATORY)

**Before deploying, you MUST ensure ALL task PRs are merged to main branch.**

**Check all PRs:**
```bash
# List all open PRs for this project
gh pr list --state open --json number,title,labels,reviews,statusCheckRollup

# For each PR, verify:
# 1. All CI checks passing
# 2. All required reviews approved
# 3. No merge conflicts
```

**Merge approved PRs:**
```bash
# Merge each approved PR
gh pr merge <PR_NUMBER> --squash --delete-branch

# Verify main is up to date
git checkout main
git pull origin main
```

**⚠️ DO NOT PROCEED TO DEPLOYMENT UNTIL ALL PRS ARE MERGED! ⚠️**

---

## Steps to Execute (After All PRs Merged)

### 2. Build Production Image

```bash
# Ensure you're on latest main
git checkout main
git pull origin main

# Build Docker image
docker build -t <service-name>:latest .

# Tag for registry
docker tag <service-name>:latest ghcr.io/<org>/<service-name>:latest
docker tag <service-name>:latest ghcr.io/<org>/<service-name>:v1.0.0

# Push to registry
docker push ghcr.io/<org>/<service-name>:latest
docker push ghcr.io/<org>/<service-name>:v1.0.0
```

### 3. Create Kubernetes Manifests

Create deployment manifest (`k8s/deployment.yaml`):
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: <service-name>
  namespace: agent-platform
spec:
  replicas: 2
  selector:
    matchLabels:
      app: <service-name>
  template:
    metadata:
      labels:
        app: <service-name>
    spec:
      containers:
      - name: app
        image: ghcr.io/<org>/<service-name>:latest
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: <service-name>-secrets
              key: database-url
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: <service-name>
  namespace: agent-platform
spec:
  selector:
    app: <service-name>
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8080
```

### 4. Set Up Ngrok Ingress

Create Ngrok ingress (`k8s/ingress.yaml`):
```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: <service-name>-ngrok
  namespace: agent-platform
  annotations:
    k8s.ngrok.com/modules: ngrok-module-set-<service>
spec:
  ingressClassName: ngrok
  rules:
  - host: <service-name>.ngrok.app
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: <service-name>
            port:
              number: 80
```

### 5. Deploy to Kubernetes

```bash
# Apply manifests
kubectl apply -f k8s/deployment.yaml
kubectl apply -f k8s/ingress.yaml

# Wait for deployment
kubectl rollout status deployment/<service-name> -n agent-platform

# Verify pods are running
kubectl get pods -n agent-platform -l app=<service-name>
```

### 6. Get Public URLs

```bash
# Get Ngrok URL
kubectl get ingress <service-name>-ngrok -n agent-platform -o jsonpath='{.status.loadBalancer.ingress[0].hostname}'

# Or check Ngrok dashboard
kubectl get ngrokingress -n agent-platform

# Test the URL
NGROK_URL=$(kubectl get ingress <service-name>-ngrok -n agent-platform -o jsonpath='{.status.loadBalancer.ingress[0].hostname}')
curl -f https://$NGROK_URL/health
```

### 7. Smoke Test Deployed Application

```bash
# Test health endpoint
curl https://<ngrok-url>/health

# Test API endpoints
curl https://<ngrok-url>/api/products

# Test authentication flow
curl -X POST https://<ngrok-url>/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"test","password":"test123"}'
```

### 8. Create Deployment Report

Document the deployment:
```markdown
## Deployment Report

**Service:** <service-name>
**Version:** v1.0.0
**Namespace:** agent-platform
**Date:** <current-date>

### Deployment Status
- ✅ All PRs merged to main
- ✅ Docker image built and pushed
- ✅ Kubernetes deployment successful
- ✅ Pods healthy: X/X running
- ✅ Ngrok ingress configured

### Public URLs
- **Main URL:** https://<ngrok-url>
- **Health Check:** https://<ngrok-url>/health
- **API Docs:** https://<ngrok-url>/api/docs (if available)

### Endpoints Tested
- ✅ GET /health - 200 OK
- ✅ GET /api/products - 200 OK
- ✅ POST /api/auth/login - 200 OK (with valid creds)
- ✅ POST /api/cart/add - 200 OK (with auth token)

### Configuration
- Database: Connected
- Environment: Production
- Replicas: 2
- Resources: Default limits

### Access Instructions
Users can access the application at:
- **URL:** https://<ngrok-url>
- **API Base:** https://<ngrok-url>/api
- **Health:** https://<ngrok-url>/health
```

## Success Criteria

Your task is complete when:

1. ✅ **ALL task PRs merged to main** (MANDATORY FIRST)
2. ✅ Docker image built and pushed to registry
3. ✅ Kubernetes deployment created and healthy
4. ✅ Ngrok ingress configured and accessible
5. ✅ Public URL obtained and verified
6. ✅ Smoke tests pass on live deployment
7. ✅ Deployment report created with URLs
8. ✅ Application publicly accessible

**Remember: PR merging is THE FIRST STEP - nothing else happens until all PRs are merged!**

## Output

Provide the public Ngrok URLs where the application can be accessed.

