# GPU provisioning runbook for MuseTalk Phase 4

> Archived: the MuseTalk GPU worker is no longer built or deployed as an active
> avatar provider. Current provider work is limited to LemonSlice and EchoMimic.

This runbook covers the GPU node lifecycle for the MuseTalk worker in the OVH GRA9 RKE2 cluster.

## Credentials and references

Read everything from 1Password `Automation` from the `openclaw-coder` pod:

- `OVH CA API`
- `OVH GRA9 GPU SSH`
- `RKE2 Join Token`

Never paste secrets into PRs or chat. Reference the 1Password item names only.

## Required environment

```bash
OVH_AK=$(op read "op://Automation/OVH CA API/application_key")
OVH_AS=$(op read "op://Automation/OVH CA API/application_secret")
OVH_CK=$(op read "op://Automation/OVH CA API/consumer_key")
OVH_PID=$(op read "op://Automation/OVH CA API/project_id")
OVH_EP=$(op read "op://Automation/OVH CA API/endpoint")

RKE2_TOKEN=$(op read "op://Automation/RKE2 Join Token/token")
RKE2_SERVER=$(op read "op://Automation/RKE2 Join Token/server_url")
RKE2_VERSION=$(op read "op://Automation/RKE2 Join Token/rke2_version")

SSH_USER=$(op read "op://Automation/OVH GRA9 GPU SSH/ssh_user")
SSH_NAME=$(op read "op://Automation/OVH GRA9 GPU SSH/registered_name")
op read "op://Automation/OVH GRA9 GPU SSH/private_key" > /tmp/gpu_key
chmod 600 /tmp/gpu_key
```

## OVH signer

```bash
ovh_call() {
  local method="$1" path="$2" body="${3:-}"
  local url="${OVH_EP}${path}"
  local ts
  ts=$(curl -s "${OVH_EP}/auth/time")
  local sig_input="${OVH_AS}+${OVH_CK}+${method}+${url}+${body}+${ts}"
  local sig="\$1\$$(printf '%s' "$sig_input" | sha1sum | cut -d' ' -f1)"
  curl -s -X "$method" "$url" \
    -H "X-Ovh-Application: ${OVH_AK}" \
    -H "X-Ovh-Consumer: ${OVH_CK}" \
    -H "X-Ovh-Timestamp: ${ts}" \
    -H "X-Ovh-Signature: ${sig}" \
    -H "Content-Type: application/json" \
    ${body:+-d "$body"}
}
```

## Create or recreate the GPU node

1. Confirm the `t2-45` flavor is available:

```bash
ovh_call GET /cloud/project/$OVH_PID/flavor | jq '.[] | select(.name=="t2-45")'
```

2. Confirm the SSH key already registered in OVH:

```bash
ovh_call GET /cloud/project/$OVH_PID/sshkey | jq --arg name "$SSH_NAME" '.[] | select(.name==$name)'
```

3. Create the instance in `GRA9` using Ubuntu 22.04 image and the registered SSH key.

4. SSH in and join RKE2:

```bash
curl -sfL https://get.rke2.io | INSTALL_RKE2_VERSION="$RKE2_VERSION" INSTALL_RKE2_TYPE=agent sh -
sudo mkdir -p /etc/rancher/rke2
sudo tee /etc/rancher/rke2/config.yaml >/dev/null <<EOF
server: ${RKE2_SERVER}
token: ${RKE2_TOKEN}
EOF
sudo systemctl enable rke2-agent --now
```

## Verify cluster join

```bash
kubectl get nodes -o wide
kubectl describe node <gpu-node>
```

Pass line:
- node is `Ready`
- node advertises `nvidia.com/gpu: 1`
- node has `feature.node.kubernetes.io/pci-10de.present=true`

## Verify GPU operator

```bash
kubectl get pods -n operators | grep -E 'nvidia|feature'
kubectl describe node <gpu-node> | grep -A5 -E 'Allocatable|nvidia.com/gpu'
```

If the driver daemonset does not bind within 15 minutes:

```bash
kubectl rollout restart -n gpu-operator daemonset/nvidia-driver-daemonset
```

Do that once only, then escalate.

## Smoke test the node

```bash
kubectl run gpu-smoke --rm -it --restart=Never \
  --image=nvidia/cuda:11.8.0-base-ubuntu22.04 \
  --overrides='{"spec":{"nodeSelector":{"feature.node.kubernetes.io/pci-10de.present":"true"},"containers":[{"name":"gpu-smoke","image":"nvidia/cuda:11.8.0-base-ubuntu22.04","command":["nvidia-smi"],"resources":{"limits":{"nvidia.com/gpu":"1"}}}]}}'
```

## Build and deploy MuseTalk worker

Build from the coder pod via the kaniko sidecar, pushing to GHCR as required by the OpenClaw runtime:

```bash
kubectl exec -n bots $(hostname) -c kaniko -- \
  /kaniko/executor \
  --context=/workspace/repos/cto \
  --dockerfile=/workspace/repos/cto/infra/images/musetalk-worker/Dockerfile \
  --destination=ghcr.io/5dlabs/musetalk-worker:phase4-bootstrap \
  --cache=true \
  --cache-repo=ghcr.io/5dlabs/kaniko-cache
```

Then sync and inspect the workload:

```bash
kubectl -n argocd get application musetalk-worker
kubectl -n cto get pods -l app.kubernetes.io/name=musetalk-worker -o wide
kubectl -n cto describe pod -l app.kubernetes.io/name=musetalk-worker
```

Then verify in the worker pod:

```bash
kubectl exec -n cto deploy/musetalk-worker -- nvidia-smi
kubectl exec -n cto deploy/musetalk-worker -- python -c "import torch; print(torch.cuda.is_available(), torch.cuda.get_device_name(0))"
```

## Tear down when idle

If the node is unused for more than 8 hours, pause the ArgoCD app and delete the OVH instance.

1. Pause app sync or scale workload down.
2. Delete the OVH instance with `ovh_call DELETE /cloud/project/$OVH_PID/instance/<instance-id>`.
3. Confirm the node disappears from `kubectl get nodes`.

## SSH key handling

The current key is already registered and should be reused. Do not rotate it unless the key is compromised.

If rotation is required:
- create a new key pair
- register it in OVH
- update the 1Password item `OVH GRA9 GPU SSH`
- recreate the GPU instance with the new registered key
