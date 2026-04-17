# Coder PRD — GPU + MuseTalk Deployment (Phase 4 of Self-Hosted Avatar)

**Copy-paste into Coder's Discord channel. Keep full message as one block.**

---

**@Coder** — GPU node provisioning + MuseTalk worker deploy. You have full decision authority: delete+recreate GPU, roll SSH keys, modify ArgoCD apps, push images. No further approval needed unless you exceed $40 CAD/day burn.

## Goal
Bring a V100S GPU node online in our OVH RKE2 cluster (GRA9), confirm the GPU Operator picks it up, then deploy the avatar agent container (MuseTalk 1.5 + CUDA 11.8) onto it so a future phase can publish WebRTC video frames. **Everything below "What Needs GPU" in `plan.md` is yours. Phases 1–3 are not.**

## Context & Preconditions
- Cluster: RKE2 v1.34.5, control plane `https://10.0.0.181:9345`, 4× b3-64 CPU nodes in GRA9, Ubuntu 22.04.
- Quota: approved via OVH ticket `CS15510375` (100 vm pool, Startup Program).
- Project ID: `6093a51de65b458e8b20a7c570a4f2c1`.
- GPU Operator + NFD already deployed via ArgoCD, currently in standby.
- Skills live in `5dlabs/agent-platform` (metal/OVH/RKE2 skills). PR lands in `5dlabs/cto`.
- Budget ceiling: ~$893 CAD/month from $14,377 startup credit. Stay under $40/day; tear down if idle overnight.

### Credentials — All in 1Password `Automation` vault
Coder pod has `OP_SERVICE_ACCOUNT_TOKEN` mounted; `op read` works directly. Three items, all confirmed readable from `openclaw-coder` agent container:

```bash
# OVH CA API (signs every API call)
OVH_AK=$(op read "op://Automation/OVH CA API/application_key")     # 274e538f3a56d176
OVH_AS=$(op read "op://Automation/OVH CA API/application_secret")
OVH_CK=$(op read "op://Automation/OVH CA API/consumer_key")
OVH_PID=$(op read "op://Automation/OVH CA API/project_id")         # 6093a51de65b458e8b20a7c570a4f2c1
OVH_EP=$(op read "op://Automation/OVH CA API/endpoint")            # https://ca.api.ovh.com/1.0

# SSH key (already registered with OVH; reuse, do not rotate)
op read "op://Automation/OVH GRA9 GPU SSH/private_key" > /tmp/gpu_key && chmod 600 /tmp/gpu_key
op read "op://Automation/OVH GRA9 GPU SSH/public_key"  > /tmp/gpu_key.pub
SSH_USER=$(op read "op://Automation/OVH GRA9 GPU SSH/ssh_user")    # ubuntu
SSH_NAME=$(op read "op://Automation/OVH GRA9 GPU SSH/registered_name")  # metal-gra9-gpu-bootstrap-20260416 (already in OVH)

# RKE2 agent join token (Coder pod uses this, no SSH-to-control-plane needed)
RKE2_TOKEN=$(op read "op://Automation/RKE2 Join Token/token")
RKE2_SERVER=$(op read "op://Automation/RKE2 Join Token/server_url")    # https://10.0.0.181:9345
RKE2_VERSION=$(op read "op://Automation/RKE2 Join Token/rke2_version") # v1.34.5+rke2r1
```

### Inline OVH CA signer (no external script dependency)
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

# examples:
# ovh_call GET /cloud/project/$OVH_PID/flavor
# ovh_call GET /cloud/project/$OVH_PID/instance
# ovh_call POST /cloud/project/$OVH_PID/instance '{"flavorId":"...","imageId":"...","name":"gpu-1","region":"GRA9","sshKeyId":"..."}'
```

## Build Environment — READ BEFORE YOU START

You are running inside the `openclaw-coder` pod in the `cto` namespace. Two quirks will bite you if you don't know about them:

### 1. Image build — use the kaniko sidecar, push to `ghcr.io` only
- No docker daemon in this pod. Image builds go through the **`kaniko`** sidecar that shares `/workspace` with your `agent` container.
- Kaniko is pre-authed for `ghcr.io/5dlabs/*` only. Do **not** try to push to `registry.5dlabs.ai` from here — that creds path is not mounted.
- Canonical build skill lives in the cloned repo (not auto-mounted under `/skills/`):
  `infra/charts/openclaw-agent/skills/openclaw/container-builds.md`
  Read it first. TL;DR of the invocation:
  ```bash
  kubectl exec -n cto $(hostname) -c kaniko -- /kaniko/executor \
    --context=/workspace/repos/cto \
    --dockerfile=infra/docker/musetalk-worker/Dockerfile \
    --destination=ghcr.io/5dlabs/musetalk-worker:<tag> \
    --cache=true \
    --cache-repo=ghcr.io/5dlabs/kaniko-cache
  ```

### 2. kubectl — ignore the baked kubeconfig, use the live projected SA token
`~/.kube/config` is a symlink to `/workspace/.kube/config` which carries a **stale token from a prior pod** and will give you `Unauthorized`. Use the live projected ServiceAccount token instead — `openclaw-coder` has cluster-admin-equivalent RBAC:

```bash
alias kc='KUBECONFIG=/dev/null kubectl \
  --token=$(cat /var/run/secrets/kubernetes.io/serviceaccount/token) \
  --certificate-authority=/var/run/secrets/kubernetes.io/serviceaccount/ca.crt \
  --server=https://kubernetes.default.svc'

kc get nodes      # works
kc -n cto get po  # works
```
Or regenerate `/workspace/.kube/config` once at session start using the same projected token. Either way: don't trust the baked kubeconfig.


## Acceptance Criteria (all must pass)

1. **Instance provisioned**: 1× `t2-45` (V100S 32GB) in `GRA9`, flavor confirmed via `/cloud/project/$PID/instance`.
2. **Joined cluster**: `kubectl get nodes` shows the GPU node `Ready`, RKE2 agent v1.34.5, joined to `https://10.0.0.181:9345`.
3. **GPU Operator healthy**: NFD labels include `feature.node.kubernetes.io/pci-10de.present=true`; `kubectl describe node <gpu-node>` shows `nvidia.com/gpu: 1` allocatable; `nvidia-smi` works in a `nvidia/cuda:11.8.0-base` test pod.
4. **Image built & pushed**: `ghcr.io/5dlabs/musetalk-worker:<tag>` — MuseTalk 1.5 repo baked in, CUDA 11.8 base, PyTorch + torchvision + transformers + opencv + livekit-rtc installed. SBOM or at minimum a `docker inspect` in the PR. (Kaniko sidecar is pre-authed to `ghcr.io/5dlabs/*` only — `registry.5dlabs.ai` is not authed in this pod.)
5. **ArgoCD Application deployed**: avatar agent pod scheduled via `nodeSelector: feature.node.kubernetes.io/pci-10de.present: "true"`, `resources.limits.nvidia.com/gpu: 1`. Pod reaches `Running`.
6. **Smoke test in pod**: `kubectl exec` → `nvidia-smi` + `python -c "import torch; print(torch.cuda.is_available(), torch.cuda.get_device_name(0))"` both succeed (prints `True, Tesla V100S-PCIE-32GB` or similar).

## Out of Scope (do NOT do these)
- LiveKit self-host Helm chart (Phase 1 — different agent).
- Admin upload UI / Better Auth (Phase 3.5).
- MuseTalk streaming plugin code in `avatar/agent/` (Phase 3 — code exists or is being authored elsewhere).
- Cutover from LemonSlice (Phase 6).
- End-to-end latency tuning (Phase 5).

## Deliverables
- PR against `5dlabs/cto`:
  - `infra/charts/musetalk-worker/` updated values + image tag bump, OR new chart if none.
  - ArgoCD Application manifest under `infra/argocd/` pinned to GPU node.
  - Dockerfile under `infra/docker/musetalk-worker/` (or wherever GPU images live in the repo convention).
- Short runbook in `avatar/docs/gpu-provisioning.md`: how to destroy + recreate the node, where the SSH key lives, how to roll it.
- Post in Discord when done with: node name, image digest, ArgoCD app sync status, `nvidia-smi` output from the pod.

## Rules
- **Billing**: if the node is unused >8h, tear it down and leave the ArgoCD app paused. Resume on demand.
- **Secrets**: never paste keys in PR / Discord. Reference 1P items by ID.
- **Cross-repo**: if a skill needs to be added, put it in `5dlabs/agent-platform`, but this feature's PR lands in `5dlabs/cto`.
- **Fallback**: if GPU Operator doesn't auto-bind the driver after 15 min, try `kubectl rollout restart -n gpu-operator daemonset/nvidia-driver-daemonset` once, then escalate rather than hand-install drivers.

## Handy Commands (saves Coder time)
```bash
# Find V100S flavor (uses inline ovh_call from above)
ovh_call GET /cloud/project/$OVH_PID/flavor | jq '.[] | select(.name=="t2-45")'

# RKE2 agent join (run on new node via SSH; token comes from 1P, not the control plane)
curl -sfL https://get.rke2.io | INSTALL_RKE2_VERSION="$RKE2_VERSION" INSTALL_RKE2_TYPE=agent sh -
# Then write /etc/rancher/rke2/config.yaml:
#   server: $RKE2_SERVER       # https://10.0.0.181:9345
#   token:  $RKE2_TOKEN        # op read "op://Automation/RKE2 Join Token/token"

# GPU smoke test pod
kubectl run gpu-smoke --rm -it --restart=Never \
  --image=nvidia/cuda:11.8.0-base-ubuntu22.04 \
  --overrides='{"spec":{"nodeSelector":{"feature.node.kubernetes.io/pci-10de.present":"true"},"containers":[{"name":"gpu-smoke","image":"nvidia/cuda:11.8.0-base-ubuntu22.04","command":["nvidia-smi"],"resources":{"limits":{"nvidia.com/gpu":"1"}}}]}}'
```

---

**End of PRD. Reply in thread with ETA and first checkpoint (node provisioned).**
