Implement an archive step at workflow end (script template) that:
- Verifies `docs/.taskmaster/docs/task-<id>` exists
- Creates `.completed` if missing
- Moves the directory to `.completed/task-<id>`
- Emits a log confirming the move
