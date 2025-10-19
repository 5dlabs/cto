#!/bin/bash
set -euo pipefail

# ============================================================================
# Blaze - Kubernetes Deployment Script
# ============================================================================
# Deploys Next.js application to Kubernetes staging namespace
# ============================================================================

NAMESPACE="${NAMESPACE:-agent-platform}"
TASK_ID="${1:-}"
SERVICE_NAME="${2:-frontend-app}"
IMAGE="${DOCKER_IMAGE:-node:20-alpine}"
REPLICAS="${REPLICAS:-1}"

if [ -z "$TASK_ID" ]; then
    echo "Usage: $0 <task-id> [service-name] [replicas]"
    echo "Example: $0 5 task-5-frontend 1"
    exit 1
fi

APP_NAME="task-${TASK_ID}-${SERVICE_NAME}"

echo "🚀 Deploying $APP_NAME to Kubernetes..."
echo "   Namespace: $NAMESPACE"
echo "   Replicas: $REPLICAS"

# Create namespace if it doesn't exist
kubectl create namespace $NAMESPACE --dry-run=client -o yaml | kubectl apply -f -

# Build Next.js app
echo "📦 Building Next.js application..."
if [ -f "package.json" ]; then
    pnpm install
    pnpm build
else
    echo "⚠️  No package.json found, skipping build"
fi

# Create Kubernetes manifests
echo "📝 Creating Kubernetes manifests..."

# Deployment
cat <<EOF | kubectl apply -f -
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ${APP_NAME}
  namespace: ${NAMESPACE}
  labels:
    app: ${APP_NAME}
    task-id: "${TASK_ID}"
    managed-by: blaze
spec:
  replicas: ${REPLICAS}
  selector:
    matchLabels:
      app: ${APP_NAME}
  template:
    metadata:
      labels:
        app: ${APP_NAME}
        task-id: "${TASK_ID}"
    spec:
      containers:
      - name: nextjs
        image: ${IMAGE}
        command: ["/bin/sh", "-c"]
        args:
          - |
            cd /workspace && \
            npm install -g pnpm && \
            pnpm install && \
            pnpm build && \
            pnpm start
        ports:
        - containerPort: 3000
          name: http
        env:
        - name: NODE_ENV
          value: production
        - name: PORT
          value: "3000"
        volumeMounts:
        - name: workspace
          mountPath: /workspace
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /
            port: 3000
          initialDelaySeconds: 10
          periodSeconds: 5
      volumes:
      - name: workspace
        persistentVolumeClaim:
          claimName: ${APP_NAME}-pvc
---
apiVersion: v1
kind: Service
metadata:
  name: ${APP_NAME}-svc
  namespace: ${NAMESPACE}
  labels:
    app: ${APP_NAME}
    task-id: "${TASK_ID}"
    managed-by: blaze
spec:
  selector:
    app: ${APP_NAME}
  ports:
  - name: http
    port: 3000
    targetPort: 3000
    protocol: TCP
  type: ClusterIP
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: ${APP_NAME}-pvc
  namespace: ${NAMESPACE}
  labels:
    app: ${APP_NAME}
    task-id: "${TASK_ID}"
    managed-by: blaze
spec:
  accessModes:
  - ReadWriteOnce
  resources:
    requests:
      storage: 2Gi
EOF

echo "⏳ Waiting for deployment to be ready..."
kubectl rollout status deployment/${APP_NAME} -n ${NAMESPACE} --timeout=300s

echo ""
echo "✅ Deployment successful!"
echo ""
echo "📋 Deployment Details:"
echo "   Name: ${APP_NAME}"
echo "   Namespace: ${NAMESPACE}"
echo "   Service: ${APP_NAME}-svc"
echo ""
echo "🔍 Check status:"
echo "   kubectl get pods -n ${NAMESPACE} -l app=${APP_NAME}"
echo "   kubectl logs -n ${NAMESPACE} -l app=${APP_NAME} --tail=50"
echo ""
echo "🌐 Next step: Setup Ngrok ingress"
echo "   ./scripts/blaze/setup-ngrok-ingress.sh ${APP_NAME} ${APP_NAME}-svc 3000"
echo ""

