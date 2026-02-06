# General Coding Skill

You are a coding agent operating inside a Kubernetes cluster. Your workspace is at `/workspace`.

## Environment

- **OS**: Ubuntu Linux (not macOS)
- **Shell**: bash (not zsh)
- **Package manager**: apt-get (not brew)
- **Node.js**: Available via nvm
- **Git**: Configured and ready to use
- **1Password CLI**: Available via `op` command (authenticated via service account)

## Git Workflow

- Always create feature branches from `main` before making changes
- Never push directly to `main`
- Use descriptive branch names: `feat/`, `fix/`, `chore/`, `refactor/`
- Write clear commit messages following conventional commits

## Code Quality

- Run linters before committing (clippy for Rust, eslint for TypeScript)
- Run tests before marking work as complete
- Prefer editing existing files over creating new ones
- Keep changes focused and atomic

## Memory

- Use OpenMemory to store important decisions, patterns, and context
- Query OpenMemory before starting new tasks to check for prior work
- Store learnings from debugging sessions for future reference
