Implement subtask 3012: Write Kubernetes Deployment and Service manifests for RMS

## Objective
Create Kubernetes Deployment (2 replicas) and ClusterIP Service manifests for the RMS service with correct port configuration, envFrom references, and health probes.

## Steps
Create k8s/rms/deployment.yaml: apiVersion apps/v1, kind Deployment, replicas: 2, image: placeholder (to be set by CI), envFrom: [{configMapRef: {name: sigma1-infra-endpoints}}, {secretRef: {name: sigma1-google-secret}}, {secretRef: {name: sigma1-rms-secret}}]. Container ports: 8080 (rest), 9090 (grpc), 8081 (health). LivenessProbe: httpGet /health/live :8081 initialDelaySeconds 10. ReadinessProbe: httpGet /health/ready :8081 initialDelaySeconds 15. Create k8s/rms/service.yaml: ClusterIP Service exposing port 8080 (name: rest) and port 9090 (name: grpc); health port 8081 not exposed externally. Add resource requests/limits: requests cpu 100m memory 128Mi, limits cpu 500m memory 512Mi.

## Validation
kubectl apply -f k8s/rms/ --dry-run=client exits 0 with no errors. kubectl apply against a dev cluster shows 2/2 pods Ready within 60 seconds. kubectl describe service rms shows ClusterIP with ports 8080 and 9090.