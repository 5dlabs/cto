//! Health check definitions.
//!
//! This module contains the definitions for validation checks that the
//! AI agent will run against the cluster.

/// Health check definitions for the validation agent.
#[allow(dead_code)] // Reference material for AI agent prompt
///
/// These are provided as context to Claude so it knows what to check.
pub const HEALTH_CHECKS: &str = r#"
# Cluster Health Checks

## 1. Node Health
- All nodes show STATUS=Ready
- Node versions match expected Kubernetes version
- No node conditions indicating problems

## 2. Node Connectivity  
- Cilium reports all nodes reachable
- All endpoints reachable (critical for pod networking)
- WireGuard peers established (if encryption enabled)

## 3. Pod-to-Pod Networking
- Can exec into pods on ALL nodes from control plane
- Pods can communicate across nodes
- Service discovery works

## 4. Storage Health
- Default storage class exists
- All PVCs are Bound (not Pending)
- local-path-provisioner healthy
- Mayastor healthy (if deployed)

## 5. DNS Health
- CoreDNS pods running on all nodes
- DNS resolution works from pods
- External DNS resolution works

## 6. GitOps Health
- ArgoCD server healthy
- ArgoCD repo-server can fetch repos
- All applications synced
- All applications healthy

## 7. Observability Health
- Prometheus scraping targets
- Loki ingesting logs
- Grafana accessible

## 8. Security Health
- cert-manager issuing certificates
- External Secrets Operator connected to vault
- Pod Security Standards enforced
"#;
