# Task 1: Implement Play Runner (Sequential Project Execution)

## Goal
Build a minimal Play Runner that executes tasks in `docs/.taskmaster/docs/` sequentially using the existing per-task Play workflow template.

## Summary
- Discover `task-*` directories (ascending)
- Submit `play-workflow-template` with `task-id=<n>` (concurrency=1)
- Wait for workflow completion via event-driven gates; then advance to next
- Resume-safe/idempotent; can start from an offset
- Basic observability (per-task progress output)

Status: pending
