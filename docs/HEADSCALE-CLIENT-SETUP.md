# Headscale Client Setup Guide

## Current Status

- ✅ Tailscale installed on this Mac (userspace mode)
- ✅ Headscale tunnel binding merged to `develop` ([PR #3257](https://github.com/5dlabs/cto/pull/3257))
- ❌ Cluster appears offline (HTTP 530 from all tunnel endpoints)

## Step 1: Check/Start the Cluster

From a machine on the home LAN (192.168.1.x network):

```bash
# Check if the node is reachable
ping 192.168.1.77

# If not responding, physically check:
# - Power to the node
# - Network cable connected
# - Any error lights on the hardware
```

## Step 2: Verify Cluster is Running

Once the node is pingable:

```bash
# Check if Kubernetes API is responding
kubectl --kubeconfig ~/.kube/config get nodes

# Check ArgoCD is syncing
kubectl get applications -n argocd

# Force sync the CTO app to pick up Headscale binding
kubectl -n argocd patch app cto --type merge -p '{"operation": {"sync": {}}}'
# Or via ArgoCD CLI:
argocd app sync cto
```

## Step 3: Verify Headscale is Running

```bash
# Check Headscale pod
kubectl get pods -n headscale

# Check Headscale service
kubectl get svc -n headscale

# Check logs if needed
kubectl logs -n headscale deploy/headscale
```

## Step 4: Verify Cloudflare Tunnel

```bash
# Check tunnel operator
kubectl get pods -n operators | grep cloudflare

# Check tunnel bindings
kubectl get tunnelbindings -A

# Verify Headscale binding exists
kubectl get tunnelbinding headscale-tunnel -n headscale -o yaml
```

## Step 5: Create Headscale Auth Key

```bash
# Create admin user (if not exists)
kubectl exec -n headscale deploy/headscale -- headscale users create admin

# Create a reusable auth key (30 days expiration)
kubectl exec -n headscale deploy/headscale -- headscale preauthkeys create --user admin --reusable --expiration 720h

# Save this key! You'll need it for client connection
```

## Step 6: Test Tunnel Access

From any internet-connected machine:

```bash
# Test Headscale is accessible via tunnel
curl -I https://headscale.5dlabs.ai

# Should return HTTP 200 or redirect, NOT 530/1033
```

## Step 7: Connect Client (on remote Mac)

Once Headscale is accessible via tunnel, run on the remote Mac:

```bash
# If using userspace networking (already running):
tailscale --socket=/tmp/tailscaled.sock up \
  --login-server https://headscale.5dlabs.ai \
  --authkey <YOUR_AUTH_KEY_FROM_STEP_5>

# Verify connection
tailscale --socket=/tmp/tailscaled.sock status
```

## Step 8: Verify Cluster Access via VPN

Once connected to Headscale:

```bash
# Test cluster API access
kubectl get nodes

# Should now work from anywhere!
```

---

## Quick Reference

| Service | URL | Purpose |
|---------|-----|---------|
| Headscale | https://headscale.5dlabs.ai | VPN control server |
| ArgoCD | https://argocd.5dlabs.ai | GitOps dashboard |
| Grafana | https://grafana.5dlabs.ai | Monitoring |

| Cluster Info | Value |
|--------------|-------|
| Node IP | 192.168.1.77 |
| K8s API | https://192.168.1.77:6443 |
| Headscale Namespace | headscale |

## Troubleshooting

### Tunnel returns 530/1033
- Cluster is offline or tunnel pod isn't running
- Check: `kubectl get pods -n operators | grep cloudflare`

### ArgoCD not syncing
- Force sync: `argocd app sync cto`
- Check app health: `argocd app get cto`

### Headscale pod not starting
- Check logs: `kubectl logs -n headscale deploy/headscale`
- Check PVC: `kubectl get pvc -n headscale`

### Can't create auth key
- Ensure user exists: `kubectl exec -n headscale deploy/headscale -- headscale users list`
- Check Headscale logs for errors

### Tailscale won't connect
- Verify Headscale URL is accessible: `curl https://headscale.5dlabs.ai`
- Check auth key hasn't expired
- Try with `--reset` flag to clear state
