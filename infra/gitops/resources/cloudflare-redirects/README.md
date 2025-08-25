# Cloudflare Domain Redirects

This configuration creates redirect rules in Cloudflare to redirect `5dlabs.ai` and `www.5dlabs.ai` to `https://github.com/5dlabs`.

## How It Works

1. **ArgoCD Job**: Runs a Kubernetes Job with ArgoCD sync hooks
2. **Cloudflare API**: Uses existing external-secrets Cloudflare API credentials
3. **Redirect Rules**: Creates HTTP redirect rules at Cloudflare edge level
4. **301 Redirects**: Permanent redirects for SEO-friendly URL forwarding

## Required API Token Permissions

The Cloudflare API token must have these permissions for both DNS (external-dns) and redirect rules:

### DNS Permissions (for external-dns)
- ✅ Zone:Zone:Read
- ✅ Zone:DNS:Edit

### Redirect Permissions (for this job)
- ✅ **Zone:Zone Settings:Edit** ← **REQUIRED FOR REDIRECTS**

## Setup Steps

### 1. Update Cloudflare API Token
1. Go to https://dash.cloudflare.com/profile/api-tokens
2. Find your existing external-dns token
3. Click **"Edit"**
4. Add permission: **Zone:Zone Settings:Edit**
5. Save the token

### 2. Update Secrets (if token changed)
If the token value changed, update it in your secrets backend:
```bash
# Update the cloudflare.api_token value in your secrets backend
# External-secrets will automatically sync the new token
```

### 3. Verify Job Success
```bash
# Check job status
kubectl get jobs -n external-dns

# Check job logs
kubectl logs job/cloudflare-redirect-setup -n external-dns

# Test redirects
curl -I http://5dlabs.ai
curl -I http://www.5dlabs.ai
```

## Expected Behavior

### Success
- Job completes successfully
- `5dlabs.ai` → 301 redirect → `https://github.com/5dlabs`
- `www.5dlabs.ai` → 301 redirect → `https://github.com/5dlabs`

### Common Issues

#### Authentication Error
```
"errors": [{"code": 10000, "message": "Authentication error"}]
```
**Cause**: API token lacks `Zone:Zone Settings:Edit` permission
**Fix**: Add the permission in Cloudflare dashboard

#### Zone Not Found
```
Error: Could not find zone ID for 5dlabs.ai
```
**Cause**: API token lacks `Zone:Zone:Read` permission or wrong zone
**Fix**: Verify token has zone access

## Integration

- **Runs automatically**: ArgoCD sync hook triggers job on deployment
- **Uses existing secrets**: Leverages external-dns Cloudflare credentials
- **No additional infrastructure**: Edge-level redirects, no backend needed
- **Fast redirects**: Handled at Cloudflare edge globally

## Files

- `job.yaml`: Kubernetes Job with ArgoCD sync hook
- `configmap.yaml`: Shell script for Cloudflare API calls
- `kustomization.yaml`: Resource grouping
- `../applications/cloudflare-redirects.yaml`: ArgoCD Application
