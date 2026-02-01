# Subtask Splitting Guide

## Problem: Combined Subtasks

The intake agent was generating subtasks that combined multiple distinct operations into a single subtask. This violates the principle of **single responsibility** for sub-agents and makes parallel execution less efficient.

## Original State (What Was Wrong)

### Task 1 - Infrastructure Setup

The original subtasks combined multiple unrelated deployments:

```
task-1.1: Deploy database infrastructure (PostgreSQL, MongoDB, Redis/Valkey)
task-1.2: Deploy messaging and storage infrastructure (Kafka, RabbitMQ, SeaweedFS)
task-1.3: Configure Kubernetes infrastructure (namespaces, policies, quotas)
task-1.4: Review and validate infrastructure deployment
```

**Problems:**
1. **task-1.1** combined 3 different databases (PostgreSQL, MongoDB, Redis) - each requires different operators, CRDs, and expertise
2. **task-1.2** combined 3 different systems (Kafka, RabbitMQ, SeaweedFS) - messaging and storage are different domains
3. **task-1.3** combined 3 different Kubernetes concepts (namespaces, policies, quotas) - each has different security implications

## Remediation (How It Was Fixed)

Each combined subtask was split into individual subtasks, each with:
- A single, focused objective
- A dedicated sub-agent with a descriptive name
- Clear acceptance criteria for that one thing

### Fixed Task 1 Structure

```
task-1.1:  Deploy PostgreSQL Cluster        → postgres-deployer
task-1.2:  Deploy MongoDB Cluster           → mongo-deployer
task-1.3:  Deploy Redis/Valkey Instance     → redis-deployer
task-1.4:  Deploy Kafka Cluster             → kafka-deployer
task-1.5:  Deploy RabbitMQ Cluster          → rabbitmq-deployer
task-1.6:  Deploy SeaweedFS Storage         → seaweedfs-deployer
task-1.7:  Configure Kubernetes Namespaces  → namespace-agent
task-1.8:  Configure Network/Security Policies → policy-agent
task-1.9:  Configure Resource Quotas        → quota-agent
task-1.10: Review and Validate Infrastructure → infra-reviewer
```

**Result:** 4 subtasks → 10 subtasks, each doing exactly ONE thing.

## Rules for Intake Agent

### ✅ DO: Create Single-Concern Subtasks

Each subtask should:
1. **Deploy ONE system** (not multiple databases, not multiple services)
2. **Configure ONE aspect** (not namespaces AND policies AND quotas)
3. **Have ONE sub-agent** with a name that clearly describes what it does

### ❌ DON'T: Combine Multiple Systems

Watch for these patterns that indicate a subtask should be split:

| Pattern | Example | Should Split Into |
|---------|---------|-------------------|
| `(X, Y, Z)` in title | `(PostgreSQL, MongoDB, Redis)` | 3 separate subtasks |
| `X and Y` for different systems | `Deploy Kafka and RabbitMQ` | 2 separate subtasks |
| Multiple operators/CRDs | `CloudNative-PG and Percona` | 2 separate subtasks |
| Different domains combined | `messaging and storage` | Separate by domain |

### Detection Heuristic

If the subtask description mentions:
- Multiple operator names (CloudNative-PG, Percona, Strimzi, etc.)
- Multiple technology names in parentheses
- The word "and" connecting different systems
- Multiple CRD types

→ **SPLIT IT** into separate subtasks.

## Sub-Agent Naming Convention

Each sub-agent should have a descriptive name:

```
{technology}-deployer    → for deployment tasks (postgres-deployer, kafka-deployer)
{domain}-agent          → for configuration tasks (policy-agent, quota-agent)
{domain}-reviewer       → for review tasks (infra-reviewer, security-reviewer)
```

## Benefits of Single-Concern Subtasks

1. **Parallel Execution**: Independent subtasks can run simultaneously
2. **Clear Ownership**: Each sub-agent has one job
3. **Easier Debugging**: Failures are isolated to one system
4. **Better Testing**: Each subtask can be tested independently
5. **Reusability**: Sub-agents can be reused across different tasks

## Example: Correct Subtask Structure

```markdown
# Subtask X.Y: Deploy PostgreSQL Cluster

## Agent
postgres-deployer

## Description
Deploy and configure CloudNative-PG PostgreSQL cluster

## Deliverables
- `postgresql-cluster.yaml` - CloudNative-PG Cluster CR

## Acceptance Criteria
- [ ] PostgreSQL cluster pods are Running
- [ ] Database is accessible
- [ ] PVC is bound with persistent storage
```

Note: ONE system, ONE agent, ONE set of deliverables, focused acceptance criteria.
