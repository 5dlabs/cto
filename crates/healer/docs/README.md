# Healer Documentation

Healer is the CTO platform's self-healing ops agent — it monitors infrastructure, detects issues, and spawns AI agents to remediate problems.

## Documents

| Document | Description |
|----------|-------------|
| [Current Functionality](./current-functionality.md) | Complete reference of what Healer does today |
| [Desired Functionality](./desired-functionality.md) | Vision and roadmap for Healer's evolution |

## Quick Reference

### What Healer Does Today

- **Reactive Monitoring**: Watches pods, workflows, and CodeRuns via kubectl streams
- **Alert Detection**: 8 alert types (A1-A9) covering container failures, timeouts, stale progress
- **Remediation**: Spawns AI agents (CodeRuns) to investigate and fix issues
- **Deduplication**: Prevents alert spam with smart grouping by workflow family
- **Log Collection**: Queries Loki for historical logs even after pods are GC'd

### Where Healer is Going

- **Proactive Monitoring**: Continuous health checks, not just during workflows
- **Predictive Alerts**: Warn about issues before they happen based on trends
- **Auto-Remediation**: Pre-approved actions (restarts, scaling) without human approval
- **Learning**: Improve remediation success rates over time
- **Dashboard**: Real-time platform health visibility

## Getting Started

```bash
# Build Healer
cd crates/healer
cargo build --release

# Run a full E2E loop with self-healing
healer full --task-id 4 --config cto-config.json --self-healing

# Watch for alerts in dry-run mode
healer alert-watch --namespace cto --dry-run
```

## Contributing

When adding new features to Healer, update these docs:
1. Add implementation details to `current-functionality.md`
2. Check off completed items in `desired-functionality.md`
3. Update this README if the quick reference changes

