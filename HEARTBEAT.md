# HEARTBEAT.md - Swarm Coordination

## On Each Heartbeat

1. **Check worker inboxes** for completed work
2. **Review TaskList()** for task statuses
3. **Spawn workers** if tasks are unblocked and no worker assigned
4. **Merge completed work** — review and commit
5. **Unblock downstream tasks** as dependencies complete

## Swarm Health Commands

```javascript
TaskList()  // See all tasks
Teammate({ operation: "write", target_agent_id: "worker-name", value: "status?" })
```

## Never HEARTBEAT_OK While

- Workers are still running
- Tasks remain pending or in_progress
- Any phase is incomplete

## Completion

When ALL tasks in ALL phases are `completed`:
```
<swarm>COMPLETE</swarm>
```

Then you may reply HEARTBEAT_OK.
