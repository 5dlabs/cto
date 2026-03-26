# CTO Troubleshooting Guide

## Setup Issues

### "Docker not installed" but Docker is running

**Symptoms:** Setup wizard says Docker is not installed even though Docker Desktop is running.

**Solutions:**
1. **Restart Docker**: Quit and restart Docker Desktop
2. **Check Docker socket**: Run `docker ps` in terminal
3. **PATH issue**: Docker CLI might not be in your system PATH
   - macOS: Check `/usr/local/bin/docker` exists
   - Windows: Check Docker Desktop settings → Resources → WSL Integration

### "Kind not installed"

**Solution:** Install Kind:

```bash
# macOS
brew install kind

# Linux
curl -Lo ./kind https://kind.sigs.k8s.io/dl/v0.20.0/kind-linux-amd64
chmod +x ./kind
sudo mv ./kind /usr/local/bin/kind

# Windows (PowerShell)
choco install kind
```

### "Helm not installed"

**Solution:** Install Helm:

```bash
# macOS
brew install helm

# Linux
curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash

# Windows
choco install kubernetes-helm
```

## Cluster Issues

### Cluster creation fails

**Common causes:**

1. **Not enough memory**
   ```bash
   # Check Docker resources
   docker info | grep "Total Memory"
   ```
   Minimum: 4GB, Recommended: 8GB

2. **Port conflict**
   ```bash
   # Check if port 6443 is in use
   lsof -i :6443
   ```
   
3. **Existing cluster**
   ```bash
   # Delete existing cluster
   kind delete cluster --name cto-lite
   ```

### Cluster running but not responding

**Check cluster status:**
```bash
kubectl cluster-info --context kind-cto-lite
kubectl get nodes
```

**Restart cluster:**
```bash
docker restart cto-lite-control-plane
```

## Deployment Issues

### Helm deployment fails

**Check Helm status:**
```bash
helm list -n cto-lite
helm status cto-lite -n cto-lite
```

**Common fixes:**

1. **Delete failed release:**
   ```bash
   helm uninstall cto-lite -n cto-lite
   ```

2. **Check pod status:**
   ```bash
   kubectl get pods -n cto-lite
   kubectl describe pod <pod-name> -n cto-lite
   ```

### Pods stuck in Pending/ImagePullBackOff

**Image not found:**
- Images are pulled from ghcr.io/5dlabs/cto-lite-*
- Check internet connectivity
- Verify you're logged into GHCR if using private images

## Workflow Issues

### Workflow won't start

**Check Argo Workflows:**
```bash
# Check Argo server is running
kubectl get pods -n cto-lite | grep argo

# Check workflow templates
kubectl get workflowtemplates -n cto-lite
```

**Check API key:**
- Verify API key in keychain
- Test key with curl:
  ```bash
  curl https://api.anthropic.com/v1/messages \
    -H "x-api-key: $ANTHROPIC_API_KEY" \
    -H "content-type: application/json"
  ```

### Workflow stuck on agent step

**Check agent pod logs:**
```bash
kubectl logs -n cto-lite -l workflows.argoproj.io/workflow=<workflow-name>
```

**Common issues:**
- API rate limiting (wait and retry)
- Invalid repository URL
- Branch doesn't exist
- No permissions to repository

### Logs not appearing

**Check log aggregation:**
```bash
# Direct pod logs
kubectl logs -n cto-lite <pod-name>

# Via Argo CLI
argo logs <workflow-name> -n cto-lite
```

## App Issues

### App crashes on launch

**macOS:**
- Check Console.app for crash logs
- Reset app data:
  ```bash
  rm -rf ~/Library/Application\ Support/ai.5dlabs.cto-lite
  ```

**Windows:**
- Check Event Viewer
- Reset app data:
  ```powershell
  Remove-Item -Recurse "$env:APPDATA\ai.5dlabs.cto-lite"
  ```

### API keys not saving

**macOS Keychain:**
- Open Keychain Access
- Search for "cto-lite"
- Verify entries exist

**Windows Credential Manager:**
- Control Panel → Credential Manager
- Windows Credentials
- Search for "cto-lite"

### Dashboard shows no workflows

**Check kubectl access:**
```bash
kubectl get workflows -n cto-lite
```

**Check cluster connection:**
```bash
kubectl config current-context
# Should show: kind-cto-lite
```

## Network Issues

### GitHub webhooks not working

**Without Cloudflare tunnel:**
- Webhooks won't work (local-only)
- Use manual workflow triggers from Dashboard

**With Cloudflare tunnel:**
1. Verify tunnel is running:
   ```bash
   kubectl get pods -n cto-lite | grep cloudflared
   ```
2. Check tunnel logs:
   ```bash
   kubectl logs -n cto-lite -l app=cloudflared
   ```

## Morgan Call/Video Issues

### "Start a call first so CTO can target the active room."

**Cause:** Shared context was sent before a room was connected.

**Fix:**
1. Open **Call** or **Video**
2. Click **Start call**
3. Wait until the room is connected, then send context again

### "Morgan could not join the room."

Open **Debug** in Call/Video and check **Local runtime** + **Local blockers**.

Common blockers map directly to local health checks:

- `kind cluster 'cto-lite' does not exist`
- `kubectl context 'kind-cto-lite' is missing/unreachable`
- `ingress-nginx controller is not ready in kind`
- `Morgan deployment is not ready in kind`
- `Morgan service is missing in kind`
- `Morgan ingress is missing or not bound to morgan.localhost`
- `cto-tools deployment is not ready`
- `cto-openmemory deployment is not ready`
- `nats deployment is not ready`
- `morgan.localhost is not reachable`

Useful checks:

```bash
kubectl config current-context
kubectl --context kind-cto-lite get deploy -n openclaw openclaw-morgan
kubectl --context kind-cto-lite get svc -n openclaw openclaw-morgan
kubectl --context kind-cto-lite get ingress -n openclaw openclaw-morgan
kubectl --context kind-cto-lite get deploy -n ingress-nginx ingress-nginx-controller
kubectl --context kind-cto-lite get deploy -n cto cto-tools cto-openmemory
kubectl --context kind-cto-lite get deploy -n messaging nats
curl -i http://morgan.localhost/health
```

### Microphone access denied

**Symptom:** Call setup fails immediately or stays disconnected with media/device error.

**Fix:**
1. Allow microphone permissions for CTO in OS privacy settings
2. Retry call start from the Call/Video surface
3. Use the **Session** debug card to confirm `Mic: enabled`

### No video track (audio works)

**Symptom:** Call is connected but video remains pending.

Expected behavior:
- Call mode is voice-only.
- Video mode shows placeholder text until LemonSlice/video track publishes.

Use debug to verify:
- `Audio track: ready`
- `Video track: pending/ready`
- `Recent Morgan errors` for provider/runtime failures

## Reset Everything

### Complete reset (start fresh)

```bash
# 1. Delete Kind cluster
kind delete cluster --name cto-lite

# 2. Clear app data
# macOS
rm -rf ~/Library/Application\ Support/ai.5dlabs.cto-lite

# Windows
Remove-Item -Recurse "$env:APPDATA\ai.5dlabs.cto-lite"

# Linux
rm -rf ~/.config/ai.5dlabs.cto-lite

# 3. Clear keychain entries
# macOS: Use Keychain Access app
# Windows: Use Credential Manager

# 4. Restart Docker
docker system prune -f

# 5. Relaunch CTO
```

## Getting Help

### Collect diagnostic info

```bash
# System info
uname -a
docker version
kind version
helm version
kubectl version

# Cluster status
kubectl cluster-info --context kind-cto-lite
kubectl get all -n cto-lite

# App logs (macOS)
cat ~/Library/Logs/ai.5dlabs.cto-lite/cto-lite.log
```

### Contact support

- **GitHub Issues:** https://github.com/5dlabs/cto/issues
- **Discord:** https://discord.gg/cto
- **Email:** support@5dlabs.ai

Include:
- OS version
- CTO version
- Error messages
- Steps to reproduce
