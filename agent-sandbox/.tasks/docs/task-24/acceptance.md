# Acceptance Criteria: Task 24

- [ ] Create WebSocket connection handler that broadcasts task changes to team members using Redis pub/sub for horizontal scaling.
- [ ] Integration tests: connect WebSocket as team member, create task via REST API, verify WebSocket receives event. Test unauthorized connection rejected. Test connection survives network interruptions with ping/pong. Load test 1000 concurrent connections. Verify events only sent to team members.
- [ ] All requirements implemented
- [ ] Tests passing
- [ ] Code follows conventions
- [ ] PR created and ready for review

## Subtasks

- [ ] 24.1: Implement WebSocket connection upgrade handler with JWT authentication
- [ ] 24.2: Design and implement TaskEvent domain model with JSON serialization
- [ ] 24.3: Build Redis pub/sub infrastructure for task events
- [ ] 24.4: Implement connection manager for tracking active WebSocket connections
- [ ] 24.5: Integrate event publishing into task mutation handlers
- [ ] 24.6: Implement WebSocket message forwarding from Redis to clients
- [ ] 24.7: Add WebSocket connection health monitoring with ping/pong
- [ ] 24.8: Implement graceful shutdown and reconnection handling
