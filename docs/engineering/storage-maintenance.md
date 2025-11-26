---
title: Shared Storage Maintenance Jobs
---

# Shared Storage Maintenance Jobs

We now ship two Kubernetes CronJobs that keep high-churn PVCs sized
correctly without needing manual intervention.  Both jobs are deployed
via Argo CD so any changes should happen through PRs in this repository.

## Runner cache pruning

- **Path:** `infra/runner-cache/runner-cache-pruner.yaml`
- **Namespace:** `arc-runners`
- **Schedule:** Every 6 hours
- **What it does:** Mounts the `runner-cache-pvc` directly and removes all
  cached data so the PVC never silently grows until the underlying
  `local-path` disk fills up.

You can adjust the cadence by editing the Cron expression inside the
manifest.  The job purposely runs with small CPU/memory requests so it
co-exists with the ARC runners on the same node.

## Workspace PVC cleanup

- **Path:** `infra/workspace-maintenance/workspace-cleaner.yaml`
- **Namespace:** `cto`
- **Schedule:** Hourly at `:15`
- **What it does:** Lists PVCs with the `workspace-type=shared` label,
  skips any that are still mounted by running pods, and deletes the rest
  once they have existed for longer than `RETENTION_HOURS` (default
  `12`).

The CronJob exposes two environment variables that can be tweaked
without touching the cleanup script:

| Variable        | Default | Meaning                                         |
|-----------------|---------|-------------------------------------------------|
| `RETENTION_HOURS` | `12`  | Minimum age (in hours) before a PVC is eligible |
| `MAX_DELETIONS`   | `15`  | Safety cap per run to avoid deleting too much   |

PVCs can opt out of automated cleanup by adding the label
`workspace.cleanup/skip: "true"`.

