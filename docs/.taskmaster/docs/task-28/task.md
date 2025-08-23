# Task 28: Implement Basic Monitoring for Multi-Agent Workflows

## Overview

This task implements basic monitoring for the multi-agent workflow system to track workflow progress and detect simple issues. The focus is on essential monitoring that provides visibility into workflow status without complex alerting infrastructure.

## Technical Requirements

### Basic Monitoring Components

1. **Workflow Status Tracking**
   - Monitor workflow creation and progression
   - Track which stage each workflow is in
   - Simple workflow age tracking (creation time vs. current time)

2. **Basic Alert Conditions**
   - Workflows stuck in same stage for >12 hours
   - Workflows running longer than 7 days
   - Failed agent pods that don't restart

3. **Simple Resource Monitoring**
   - Basic resource usage for agent pods
   - PVC storage usage warnings
   - Simple GitHub API rate limit checks

## Implementation Approach

### Basic Workflow Monitoring

Create a simple Python script that queries Argo Workflows and displays status. This script will:
- List all active workflows with play-orchestration label
- Calculate workflow age and detect stuck/long-running workflows
- Generate simple status reports with counts and alerts

### Basic Resource Monitoring

Create a simple bash script that checks:
- Pod resource usage via kubectl top
- PVC usage warnings
- Failed pods in the agent-platform namespace
- Recent events for debugging

### Basic Alerting

Implement simple alerting via:
- Slack webhook notifications for stuck workflows
- Console/log output as fallback
- CronJob to run checks every 15 minutes

## Implementation Plan

1. **Phase 1: Basic Status Monitoring**
   - Deploy monitoring scripts as Kubernetes CronJobs
   - Set up basic console/log output for status tracking

2. **Phase 2: Simple Alerting**
   - Configure Slack webhook for basic notifications
   - Test alert conditions

3. **Phase 3: Basic Resource Tracking**
   - Implement simple resource usage reporting
   - Add PVC usage monitoring

## Success Criteria

1. **Basic visibility** - Can see current workflow status and age
2. **Stuck workflow detection** - Alerts when workflows are stuck >12 hours
3. **Long-running workflow alerts** - Notifications for workflows >7 days
4. **Resource awareness** - Basic visibility into resource usage
5. **Simple alerting** - Notifications via Slack or console logs

This basic approach provides essential monitoring without the complexity of comprehensive observability infrastructure.
