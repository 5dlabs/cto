Implement subtask 7008: Create Morgan Kubernetes Deployment with PVC, secrets, and Cloudflare Tunnel annotation

## Objective
Author the Kubernetes Deployment manifest for Morgan in the openclaw namespace, including the 10Gi PVC, envFrom references to all required secrets and ConfigMaps, liveness probe, and Cloudflare Tunnel annotation for the Signal webhook.

## Steps
1. Create k8s/openclaw/morgan-deployment.yaml.
2. Deployment spec: namespace: openclaw, replicas: 1, selector: app=morgan.
3. Container spec: image: openclaw/agent:latest (or pinned tag), args: ['--agent-dir', '/agents/morgan'].
4. Volume mounts: /workspace from PVC morgan-workspace, /agents/morgan from ConfigMap or mounted from repo via init container.
5. PVC: create k8s/openclaw/morgan-pvc.yaml — PersistentVolumeClaim name: morgan-workspace, storageClassName: standard, accessModes: ReadWriteOnce, storage: 10Gi.
6. envFrom: sigma1-infra-endpoints ConfigMap (from Task 2), plus secretRef entries for: elevenlabs-secret (ELEVENLABS_API_KEY, ELEVENLABS_VOICE_ID), twilio-secret (TWILIO_PHONE_NUMBER, TWILIO_SIP_DOMAIN), openai-secret (OPENAI_API_KEY), signal-secret (SIGNAL_PHONE_NUMBER).
7. Liveness probe: httpGet path: /health/live, port: 8081 (OpenClaw admin port), initialDelaySeconds: 30, periodSeconds: 15.
8. Readiness probe: httpGet path: /health/ready, port: 8081, initialDelaySeconds: 15.
9. Annotations: cloudflare-tunnel/enabled: 'true', cloudflare-tunnel/hostname: '${MORGAN_TUNNEL_HOSTNAME}', cloudflare-tunnel/service: 'http://morgan-chat-svc:3000' (also routes /voice/inbound externally).
10. Apply: `kubectl apply -f k8s/openclaw/morgan-pvc.yaml -f k8s/openclaw/morgan-deployment.yaml`.
11. Wait for rollout: `kubectl rollout status deployment/morgan -n openclaw`.

## Validation
`kubectl get deployment morgan -n openclaw` shows 1/1 READY. `kubectl get pvc morgan-workspace -n openclaw` shows Bound. `kubectl describe deployment morgan -n openclaw` shows all expected envFrom sources listed. `kubectl exec -n openclaw deployment/morgan -- wget -qO- http://localhost:8081/health/live` returns HTTP 200.