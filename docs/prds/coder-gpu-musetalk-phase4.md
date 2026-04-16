# Coder PRD — GPU + MuseTalk Deployment (Phase 4 of Self-Hosted Avatar)

**Copy-paste into Coder's Discord channel. Keep full message as one block.**

---

**@Coder** — GPU node provisioning + MuseTalk worker deploy. You have full decision authority: delete+recreate GPU, roll SSH keys, modify ArgoCD apps, push images. No further approval needed unless you exceed $40 CAD/day burn.

## Goal
Bring a V100S GPU node online in our OVH RKE2 cluster (GRA9), confirm the GPU Operator picks it up, then deploy the avatar agent container (MuseTalk 1.5 + CUDA 11.8) onto it so a future phase can publish WebRTC video frames. **Everything below "What Needs GPU" in `plan.md` is yours. Phases 1–3 are not.**

## Context & Preconditions
- Cluster: RKE2 v1.34.5, control plane `https://10.0.0.181:9345`, 4× b3-64 CPU nodes in GRA9, Ubuntu 22.04.
- Quota: approved via OVH ticket `CS15510375` (100 vm pool, Startup Program).
- Credentials (1Password, biometric):
  - OVH CA API: app key `274e538f3a56d176`, app secret + consumer key in 1P (item name "OVH CA API").
  - SSH key for the node: already generated, public half already uploaded to OVH. Private at `~/.ssh/metal_ovh_gra9_gpu`, 1P item `nt5fdiygmlflxoipmk4oxumoey`. **Reuse it; do not rotate.**
- Project ID: `6093a51de65b458e8b20a7c570a4f2c1`.
- GPU Operator + NFD already deployed via ArgoCD, currently in standby.
- Skills live in `5dlabs/agent-platform` (metal/OVH/RKE2 skills). PR lands in `5dlabs/cto`.
- Budget ceiling: ~$893 CAD/month from $14,377 startup credit. Stay under $40/day; tear down if idle overnight.

## Acceptance Criteria (all must pass)
1. **Instance provisioned**: 1× `t2-45` (V100S 32GB) in `GRA9`, flavor confirmed via `/cloud/project/$PID/instance`.
2. **Joined cluster**: `kubectl get nodes` shows the GPU node `Ready`, RKE2 agent v1.34.5, joined to `https://10.0.0.181:9345`.
3. **GPU Operator healthy**: NFD labels include `feature.node.kubernetes.io/pci-10de.present=true`; `kubectl describe node <gpu-node>` shows `nvidia.com/gpu: 1` allocatable; `nvidia-smi` works in a `nvidia/cuda:11.8.0-base` test pod.
4. **Image built & pushed**: `registry.5dlabs.ai/5dlabs/musetalk-worker:<tag>` — MuseTalk 1.5 repo baked in, CUDA 11.8 base, PyTorch + torchvision + transformers + opencv + livekit-rtc installed. SBOM or at minimum a `docker inspect` in the PR.
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
# OVH CA signer (already on coordinator box; Coder should have own copy)
/tmp/ovh_sig.sh GET /cloud/project/6093a51de65b458e8b20a7c570a4f2c1/flavor | jq '.[] | select(.name=="t2-45")'

# RKE2 agent join (run on new node)
curl -sfL https://get.rke2.io | INSTALL_RKE2_VERSION=v1.34.5+rke2r1 INSTALL_RKE2_TYPE=agent sh -
# /etc/rancher/rke2/config.yaml:
#   server: https://10.0.0.181:9345
#   token: <from control plane /var/lib/rancher/rke2/server/node-token>

# GPU smoke test pod
kubectl run gpu-smoke --rm -it --restart=Never \
  --image=nvidia/cuda:11.8.0-base-ubuntu22.04 \
  --overrides='{"spec":{"nodeSelector":{"feature.node.kubernetes.io/pci-10de.present":"true"},"containers":[{"name":"gpu-smoke","image":"nvidia/cuda:11.8.0-base-ubuntu22.04","command":["nvidia-smi"],"resources":{"limits":{"nvidia.com/gpu":"1"}}}]}}'
```

---

**End of PRD. Reply in thread with ETA and first checkpoint (node provisioned).**
