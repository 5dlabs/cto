# Enhanced Correlation Sensor Deployment Guide



## Overview
This guide provides step-by-step instructions for deploying the enhanced GitHub webhook correlation sensor that implements Task 5 requirements.



## Prerequisites

### 1. Verify Existing Infrastructure




```bash
# Check EventSource
kubectl get eventsource github -n argo

# Check EventBus
kubectl get eventbus default -n argo

# Check Service Account
kubectl get sa argo-events-sa -n argo



# Check GitHub webhook secret
kubectl get secret github-webhook-secret -n argo








```

### 2. Verify Argo Events Version




```bash
kubectl get deploy -n argo-events -o jsonpath='{.items[*].spec.template.spec.containers[*].image}' | grep argoproj
# Should be v1.9.0 or higher for full parameterization support








```

## Deployment Steps

### Step 1: Apply the Enhanced Sensor





```bash
# Deploy the enhanced correlation sensor
kubectl apply -f enhanced-correlation-sensor.yaml

# Verify deployment
kubectl get sensor enhanced-play-workflow-correlation -n argo








```

### Step 2: Verify Sensor Status





```bash
# Check sensor status
kubectl get sensor enhanced-play-workflow-correlation -n argo -o jsonpath='{.status}' | jq '.'



# Expected output should show:


# - "phase": "Deployed"
# - All dependencies active








```

### Step 3: Monitor Sensor Pod





```bash
# Find sensor pod
kubectl get pods -n argo -l sensor-name=enhanced-play-workflow-correlation



# Watch logs
kubectl logs -f $(kubectl get pods -n argo -l sensor-name=enhanced-play-workflow-correlation -o name | head -1) -n argo








```

## Testing the Deployment

### Test 1: PR Creation Event

Create a test workflow first:




```bash
# Create a suspended workflow for testing
cat <<EOF | kubectl apply -f -
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  name: play-task-99-workflow
  namespace: argo
spec:
  entrypoint: main
  templates:
  - name: main
    steps:
    - - name: wait
        template: suspend
  - name: suspend
    suspend: {}
EOF








```

Simulate PR creation webhook:




```bash


# Use the test script (requires GitHub webhook secret)
curl -X POST http://github-eventsource-svc.argo:12000/github/webhook \
  -H 'X-GitHub-Event: pull_request' \
  -H 'Content-Type: application/json' \
  -H "X-Hub-Signature-256: $(echo -n '{"action":"opened","pull_request":{"labels":[{"name":"task-99"}],"head":{"ref":"task-99-test"}}}' | openssl dgst -sha256 -hmac "$(kubectl get secret github-webhook-secret -n argo -o jsonpath='{.data.secret}' | base64 -d)" | sed 's/^.* /sha256=/')" \
  -d '{"action":"opened","pull_request":{"labels":[{"name":"task-99"}],"head":{"ref":"task-99-test"}}}'








```

### Test 2: Ready-for-QA Label Event





```bash
# Simulate ready-for-qa label addition
curl -X POST http://github-eventsource-svc.argo:12000/github/webhook \
  -H 'X-GitHub-Event: pull_request' \
  -H 'Content-Type: application/json' \
  -H "X-Hub-Signature-256: $(echo -n '{"action":"labeled","label":{"name":"ready-for-qa"},"pull_request":{"labels":[{"name":"task-99"},{"name":"ready-for-qa"}]},"sender":{"login":"5DLabs-Cleo[bot]"}}' | openssl dgst -sha256 -hmac "$(kubectl get secret github-webhook-secret -n argo -o jsonpath='{.data.secret}' | base64 -d)" | sed 's/^.* /sha256=/')" \
  -d '{"action":"labeled","label":{"name":"ready-for-qa"},"pull_request":{"labels":[{"name":"task-99"},{"name":"ready-for-qa"}]},"sender":{"login":"5DLabs-Cleo[bot]"}}'








```

### Test 3: Branch Fallback





```bash
# Test with no labels (fallback to branch)
curl -X POST http://github-eventsource-svc.argo:12000/github/webhook \
  -H 'X-GitHub-Event: pull_request' \
  -H 'Content-Type: application/json' \
  -H "X-Hub-Signature-256: $(echo -n '{"action":"opened","pull_request":{"labels":[],"head":{"ref":"task-100-no-labels"}}}' | openssl dgst -sha256 -hmac "$(kubectl get secret github-webhook-secret -n argo -o jsonpath='{.data.secret}' | base64 -d)" | sed 's/^.* /sha256=/')" \
  -d '{"action":"opened","pull_request":{"labels":[],"head":{"ref":"task-100-no-labels"}}}'








```

## Validation Steps

### 1. Check Sensor Processing




```bash
# Watch sensor logs during test
kubectl logs -f $(kubectl get pods -n argo -l sensor-name=enhanced-play-workflow-correlation -o name | head -1) -n argo



# Look for:
# - "Received event"
# - "Processing trigger"


# - "Successfully processed trigger"








```



### 2. Verify Workflow Resume




```bash


# Check if test workflow was resumed
kubectl get workflow play-task-99-workflow -n argo -o jsonpath='{.status.phase}'
# Should change from "Running" (suspended) to "Succeeded" after resume








```

### 3. Check Remediation Logic




```bash
# List remediation cleanup workflows
kubectl get workflows -n argo -l type=remediation

# Check cleanup job execution
kubectl get jobs -n argo -l job-name~="cancel-quality-agents-*"








```

## Troubleshooting

### Issue: Sensor Not Receiving Events

1. Check EventSource status:




```bash
kubectl describe eventsource github -n argo








```

2. Verify webhook endpoint:




```bash
kubectl get svc github-eventsource-svc -n argo








```

3. Test connectivity:




```bash
kubectl run test-curl --image=curlimages/curl --rm -it --restart=Never -- \
  curl -v http://github-eventsource-svc.argo:12000/github/webhook








```

### Issue: Correlation Not Working

1. Check sensor logs for errors:




```bash
kubectl logs $(kubectl get pods -n argo -l sensor-name=enhanced-play-workflow-correlation -o name | head -1) -n argo --tail=100








```

2. Verify dataTemplate execution:




```bash
# Add debug output to sensor and redeploy
# Look for template processing errors








```

3. Check workflow naming:




```bash
kubectl get workflows -n argo --show-labels | grep "play-task"








```

### Issue: Workflow Not Resuming

1. Verify workflow is suspended:




```bash
kubectl get workflow play-task-X-workflow -n argo -o yaml | grep -A5 "suspend"








```

2. Check RBAC permissions:




```bash
kubectl auth can-i update workflows --as=system:serviceaccount:argo:argo-events-sa -n argo








```

3. Check for conflicting sensors:




```bash
kubectl get sensors -n argo | grep -E "(play-workflow|multi-agent)"








```

## Migration from Existing Sensors

### Parallel Deployment (Recommended)

1. Deploy enhanced sensor alongside existing ones:




```bash
kubectl apply -f enhanced-correlation-sensor.yaml








```

2. Test with non-production workflows:




```bash
# Create test workflows with high task numbers (900+)
# These won't conflict with production tasks








```

3. Monitor both sensors:




```bash
# Watch both sensor logs
kubectl logs -f -l sensor-name -n argo








```

4. Gradually transition:


   - Update workflow templates to use new correlation


   - Test each agent transition (Rex → Cleo → Tess)


   - Monitor for issues



### Cutover Strategy

1. **Phase 1**: Deploy and test (1-2 days)


   - Deploy enhanced sensor


   - Run parallel with existing sensors


   - Validate with test workflows

2. **Phase 2**: Limited production (3-5 days)


   - Route new workflows to enhanced sensor


   - Keep existing workflows on old sensors


   - Monitor performance and accuracy

3. **Phase 3**: Full migration (1 week)


   - Update all workflow templates


   - Disable old sensors


   - Full production on enhanced sensor

4. **Phase 4**: Cleanup


   - Remove old sensor configurations


   - Update documentation


   - Archive old configurations

## Performance Tuning

### Sensor Resource Limits




```yaml
# Edit sensor to add resource limits
kubectl edit sensor enhanced-play-workflow-correlation -n argo

# Add under spec.template:
resources:
  limits:
    memory: "512Mi"
    cpu: "500m"
  requests:
    memory: "256Mi"
    cpu: "200m"








```

### Retry Configuration
Adjust retry strategy based on webhook reliability:




```yaml
retryStrategy:
  steps: 5          # Increase for unreliable networks
  duration: "30s"   # Increase for slow processing
  factor: 2         # Exponential backoff factor
  jitter: 0.2       # Add randomness to prevent thundering herd








```

### EventBus Scaling
For high webhook volume:




```bash
kubectl edit eventbus default -n argo
# Increase replicas under spec.nats.native.replicas








```

## Monitoring Setup



### Prometheus Metrics




```yaml
# Add to sensor for metrics exposure
kubectl edit sensor enhanced-play-workflow-correlation -n argo

# Under spec.template:
metrics:
  enabled: true
  port: 9090
  path: /metrics








```

### Grafana Dashboard
Create dashboard with:


- Webhook events received/processed


- Correlation success/failure rate


- Workflow resume latency


- Error rate by event type

### Alerting Rules




```yaml
# Key alerts to configure:


- SensorProcessingErrors > 5 in 5m


- CorrelationFailureRate > 10%


- WorkflowResumeLatency > 30s


- WebhookBacklog > 100 events








```

## Security Hardening



### 1. Network Policies




```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: sensor-network-policy
  namespace: argo
spec:
  podSelector:
    matchLabels:
      sensor-name: enhanced-play-workflow-correlation
  policyTypes:


  - Ingress


  - Egress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          app: eventbus
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: argo-server








```

### 2. Webhook Signature Validation
Ensure webhook secret is strong:




```bash
# Generate new secret if needed
kubectl create secret generic github-webhook-secret \


  --from-literal=secret=$(openssl rand -base64 32) \


  -n argo --dry-run=client -o yaml | kubectl apply -f -








```

### 3. RBAC Restrictions
Limit sensor permissions:




```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: sensor-workflow-role
  namespace: argo
rules:
- apiGroups: ["argoproj.io"]
  resources: ["workflows"]
  verbs: ["get", "list", "update"]  # Only what's needed








```



## Rollback Procedure

If issues arise, rollback to previous sensors:





```bash
# 1. Delete enhanced sensor
kubectl delete sensor enhanced-play-workflow-correlation -n argo

# 2. Verify old sensors are active
kubectl get sensors -n argo

# 3. Check webhook processing resumes
kubectl logs -f -l sensor-name -n argo

# 4. Document issues for resolution








```



## Success Criteria

Deployment is successful when:


- [ ] All sensor pods are running


- [ ] Test webhooks process correctly


- [ ] Workflows resume as expected


- [ ] No errors in sensor logs


- [ ] Correlation accuracy is 100%


- [ ] Processing latency < 1 second


- [ ] All test cases pass

## Support and Maintenance



### Regular Checks (Weekly)


- Review sensor logs for warnings


- Check correlation accuracy metrics


- Validate webhook processing times


- Update agent regex for new agents

### Monthly Maintenance


- Review and optimize retry strategies


- Update resource limits based on usage


- Archive old workflow data


- Update test payloads for new patterns



### Quarterly Review


- Performance benchmark comparison


- Security audit of webhook handling


- Documentation updates


- Feature enhancement planning