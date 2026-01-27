# Twingate Setup Summary

## ✅ Completed Setup

### 1. API Configuration
- **API Key**: Stored in 1Password as "Twingate API Key"
- **Subdomain**: `turquoiseelephant631`
- **API Endpoint**: `https://turquoiseelephant631.twingate.com/api/graphql/`

### 2. Remote Network
- **Name**: Latitude
- **ID**: `UmVtb3RlTmV0d29yazoyNzU4MDY=`
- **Status**: ✅ Existing network found

### 3. Connector
- **Name**: giga-octopus
- **ID**: `Q29ubmVjdG9yOjcxOTEwMQ==`
- **Status**: ✅ Using existing connector
- **Tokens**: Generated via GraphQL API

### 4. Resources
- **Cluster Pod Network**: Created
  - **Name**: Cluster Pod Network
  - **ID**: `UmVzb3VyY2U6MzMwMDMyMw==`
  - **Address**: `10.244.0.0/16` (wildcard for all pods)
  - **Protocols**: TCP/UDP/ICMP - ALLOW_ALL

### 5. Kubernetes Configuration

#### ExternalSecrets
- **twingate-api-secret** (operators namespace)
  - Stores `TWINGATE_API_TOKEN` for the operator
  - Source: OpenBao `secret/tools-twingate`
  
- **twingate-connector-tokens** (cto namespace)
  - Stores `TWINGATE_CONNECTOR_ACCESS_TOKEN` and `TWINGATE_CONNECTOR_REFRESH_TOKEN`
  - Source: OpenBao `secret/tools-twingate`

#### Argo CD Applications
- **twingate-operator** (`infra/gitops/applications/operators/twingate-operator.yaml`)
  - Deploys Twingate Kubernetes Operator
  - Uses CRDs for RemoteNetwork, Connector, Resource management
  
- **twingate-connector** (`infra/gitops/applications/networking/twingate-connector.yaml`)
  - Deploys connector pods via Helm chart
  - Uses official Twingate Helm chart: `twingate/connector`
  - Configured with 2 replicas for HA

## 📋 Next Steps

### 1. Store Secrets in OpenBao

Run the script to store all Twingate secrets in OpenBao:

```bash
export TWINGATE_CONNECTOR_ACCESS_TOKEN="eyJhbGciOiJFUzI1NiIsImtpZCI6IjEzdjhMUncxSU9YZGJjeWlaSUFIU3Npa09LWTJhVHhJb2owWWFNUFBnUlEiLCJ0eXAiOiJEQVQifQ.eyJhdWRzIjpudWxsLCJudCI6IkFOIiwiYWlkIjoiNzE5MTAxIiwiZGlkIjoiMjkwNjYwMSIsInJudyI6MTc2OTUyNDE4NSwianRpIjoiMGRhNmJmMGItYWVkNi00MDYxLTk3M2QtMzI2NzY5MmQwNTUxIiwiaXNzIjoidHdpbmdhdGUiLCJhdWQiOiJ0dXJxdW9pc2VlbGVwaGFudDYzMSIsImV4cCI6MTc2OTUyNzU1OCwiaWF0IjoxNzY5NTIzOTU4LCJ2ZXIiOiI0IiwidGlkIjoiMTk0ODciLCJybmV0aWQiOiIyNzU4MDYifQ.bakIpXaJXH4_U9ycni7APtEPixqo7lX5WBOuM-0Z9ouX10wqaQuR7nRkhP2xT82D-P2hVuBG2vRs2C2Z2PTT_g"

export TWINGATE_CONNECTOR_REFRESH_TOKEN="3WbvshI52qdDK1deYKKrz-j8jjXN9fGEXlXwlT-5yBE7HlAigYRCbfzBaRwc_GD0_C5Bb-8zrCnTXcU7uDuce5J9Ehp67aPxLd6safenr_lsEc09jiNRyyRPXoVwXnVU3Q3U3A"

./scripts/store-twingate-secrets.sh
```

Or manually store in OpenBao:

```bash
# Get OpenBao root token
ROOT_TOKEN=$(op item get "OpenBao Unseal Keys - CTO Platform" --format=json | \
  jq -r '.fields[] | select(.label == "password" or .label == "Root Token") | .value')

# Get API key from 1Password
API_KEY=$(op item get "Twingate API Key" --fields credential --reveal)

# Store in OpenBao
kubectl exec -n openbao-system openbao-0 -- env BAO_TOKEN="$ROOT_TOKEN" \
  bao kv put secret/tools-twingate \
  TWINGATE_API_TOKEN="$API_KEY" \
  TWINGATE_CONNECTOR_ACCESS_TOKEN="$TWINGATE_CONNECTOR_ACCESS_TOKEN" \
  TWINGATE_CONNECTOR_REFRESH_TOKEN="$TWINGATE_CONNECTOR_REFRESH_TOKEN"
```

### 2. Deploy via Argo CD

The Argo CD applications will automatically sync once secrets are available:

```bash
# Check operator status
kubectl get application twingate-operator -n argocd

# Check connector status
kubectl get application twingate-connector -n argocd

# Check connector pods
kubectl get pods -n cto -l app.kubernetes.io/name=twingate-connector
```

### 3. Verify Access

Once deployed, Twingate clients can access:
- **Pod Network**: `10.244.0.0/16` (all pods in the cluster)
- **Services**: Via pod network access

## 🔧 Troubleshooting

### Check Connector Status
```bash
# View connector logs
kubectl logs -n cto -l app.kubernetes.io/name=twingate-connector --tail=100

# Check ExternalSecret sync status
kubectl get externalsecret twingate-connector-tokens -n cto
kubectl describe externalsecret twingate-connector-tokens -n cto
```

### Regenerate Connector Tokens
If tokens expire, regenerate via GraphQL API:

```bash
export TWINGATE_API_KEY=$(op item get "Twingate API Key" --fields credential --reveal)
export TWINGATE_SUBDOMAIN=turquoiseelephant631

curl -s "https://${TWINGATE_SUBDOMAIN}.twingate.com/api/graphql/" \
  -H "Content-Type: application/json" \
  -H "X-API-KEY: ${TWINGATE_API_KEY}" \
  -d '{"query":"mutation { connectorGenerateTokens(connectorId: \"Q29ubmVjdG9yOjcxOTEwMQ==\") { connectorTokens { accessToken refreshToken } ok error } }"}' | jq .
```

Then update OpenBao with new tokens.

## 📚 References

- [Twingate API Documentation](https://www.twingate.com/docs/api-overview)
- [Twingate Helm Chart](https://github.com/Twingate/helm-charts)
- [Twingate Kubernetes Operator](https://github.com/Twingate/kubernetes-operator)
