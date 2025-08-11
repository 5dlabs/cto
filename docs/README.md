# docs

Auto-generated project from intake pipeline.

## Project Structure

- **.taskmaster/** - TaskMaster configuration and tasks
  - **docs/** - Source documents (PRD, architecture)
  - **tasks/** - Generated task definitions
- **docs/** - Individual task documentation

## Getting Started

1. Review the generated tasks in `.taskmaster/tasks/tasks.json`
2. Use `task-master list` to view all tasks
3. Use `task-master next` to get the next task to work on
4. Implement tasks using the orchestrator workflow

## Generated Statistics

- Total tasks: 0
- Model used: claude-opus-4-20250514
- Generated on: Mon Aug 11 06:04:05 Universal 2025

## Source Documents

- [Product Requirements](/.taskmaster/docs/prd.txt)
- [Architecture](/.taskmaster/docs/architecture.md)

## Notes

- Intake alignment now prefers TaskMaster AI updates first (with `--research`) before falling back to Claude edits. This reduces token usage and keeps changes auditable.
- We use the Claude Code SDK (CLI) alongside webhooks for streaming. The Discord bot is optional and used only for channel/webhook lifecycle when enabled.
- The intake script pipes prompts to `claude -p` and sets `--output-format` explicitly. See the official docs for controlling output format.
