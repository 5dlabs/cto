<identity>
You are rex working on subtask 4013 of task 4.
</identity>

<context>
<scope>
Create Kubernetes Deployment (2 replicas) and ClusterIP Service manifests for the Finance service with correct port configuration, envFrom references, health probes, and resource limits.
</scope>
</context>

<implementation_plan>
Create k8s/finance/deployment.yaml: apiVersion apps/v1, kind Deployment, replicas: 2, image: placeholder, envFrom: [{configMapRef: {name: sigma1-infra-endpoints}}, {secretRef: {name: sigma1-finance-secret}}]. sigma1-finance-secret must contain STRIPE_SECRET_KEY, STRIPE_WEBHOOK_SECRET, JWT_SECRET, INTERNAL_API_KEY. Container port: 3000 (api). LivenessProbe: httpGet /health/live :3000 initialDelaySeconds 10 periodSeconds 15. ReadinessProbe: httpGet /health/ready :3000 initialDelaySeconds 20 periodSeconds 10. Resource requests: cpu 100m memory 128Mi; limits: cpu 500m memory 512Mi. Create k8s/finance/service.yaml: ClusterIP Service exposing port 3000 named api.
</implementation_plan>

<validation>
kubectl apply -f k8s/finance/ --dry-run=client exits 0. kubectl apply against dev cluster shows 2/2 pods Ready within 90 seconds. kubectl describe service finance shows ClusterIP with port 3000. kubectl logs shows no startup errors.
</validation>