//! Remediation playbooks for common issues.
//!
//! This module contains remediation instructions that the AI agent
//! can follow to fix common cluster issues.

/// Remediation playbooks for the validation agent.
#[allow(dead_code)] // Reference material for AI agent prompt
///
/// These are provided as context to Claude so it knows how to fix issues.
pub const REMEDIATION_PLAYBOOK: &str = r#"
# Remediation Playbook

## Node Connectivity Issues

### Symptom: "Endpoints unreachable" in Cilium status

1. Check if Cilium pods are running on all nodes:
   ```bash
   kubectl get pods -n kube-system -l k8s-app=cilium -o wide
   ```

2. Restart Cilium on affected node:
   ```bash
   kubectl delete pod -n kube-system -l k8s-app=cilium --field-selector spec.nodeName=<node>
   ```

3. Check WireGuard status:
   ```bash
   kubectl exec -n kube-system ds/cilium -- cilium encrypt status
   ```

4. If still failing, check Talos firewall:
   - Ensure UDP 51871 (WireGuard) is allowed between nodes
   - Ensure VXLAN port 8472 is allowed

## Pod Stuck in Pending

### Symptom: PVC not binding

1. Check storage class exists:
   ```bash
   kubectl get sc
   ```

2. Check local-path-provisioner:
   ```bash
   kubectl get pods -n local-path-storage
   kubectl logs -n local-path-storage -l app=local-path-provisioner
   ```

3. Check for Pod Security violations:
   ```bash
   kubectl get events -A | grep -i forbidden
   ```

4. Fix: Add privileged labels to namespace:
   ```bash
   kubectl label namespace local-path-storage \
     pod-security.kubernetes.io/enforce=privileged \
     pod-security.kubernetes.io/enforce-version=latest
   ```

## ArgoCD App Not Syncing

### Symptom: Application shows "Unknown" sync status

1. Check ArgoCD controller:
   ```bash
   kubectl get pods -n argocd -l app.kubernetes.io/name=argocd-application-controller
   ```

2. Restart controller if stuck:
   ```bash
   kubectl rollout restart statefulset -n argocd argocd-application-controller
   ```

3. Force refresh application:
   ```bash
   kubectl annotate application <app> -n argocd argocd.argoproj.io/refresh=hard --overwrite
   ```

4. Check repo-server connectivity:
   ```bash
   kubectl logs -n argocd -l app.kubernetes.io/name=argocd-repo-server --tail=50
   ```

## External Secrets Not Created

### Symptom: Kubernetes Secret not created from ExternalSecret

1. Check ESO webhook is reachable:
   ```bash
   kubectl get pods -n external-secrets -l app.kubernetes.io/name=external-secrets-webhook
   ```

2. Check ClusterSecretStore exists:
   ```bash
   kubectl get clustersecretstore
   ```

3. Check OpenBao is unsealed:
   ```bash
   kubectl exec -n openbao openbao-0 -- bao status
   ```

4. If webhook timing out, restart it:
   ```bash
   kubectl rollout restart deployment -n external-secrets external-secrets-webhook
   ```

## DNS Resolution Failing

### Symptom: Pods can't resolve service names

1. Check CoreDNS:
   ```bash
   kubectl get pods -n kube-system -l k8s-app=kube-dns
   kubectl logs -n kube-system -l k8s-app=kube-dns --tail=20
   ```

2. Restart CoreDNS:
   ```bash
   kubectl rollout restart deployment -n kube-system coredns
   ```

3. Check Cilium DNS proxy:
   ```bash
   kubectl exec -n kube-system ds/cilium -- cilium status | grep DNS
   ```

## CrashLoopBackOff Pods

### General approach:

1. Get pod logs:
   ```bash
   kubectl logs <pod> -n <namespace> --previous
   ```

2. Check events:
   ```bash
   kubectl describe pod <pod> -n <namespace>
   ```

3. Common fixes:
   - Missing ConfigMap/Secret → Create or fix ExternalSecret
   - Image pull error → Check imagePullSecrets
   - Resource limits → Increase memory/CPU limits
   - Probe failures → Increase initialDelaySeconds
"#;
