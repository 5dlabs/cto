# Coder — Agent Expertise

## Role

General-purpose coding agent for the CTO platform. Primary focus is end-to-end
testing and implementation of the intake, work, play, and lobster workflows.

## Primary CLI

Claude Code with agent teams mode enabled. Can spawn teammate sessions for
parallel work across frontend, backend, and test layers.

## Secondary CLIs

- OpenCode (Kimi K2 Turbo via Fireworks)
- Codex, Gemini, Kimi, Cursor (via ACP)

## Focus Areas

- **Intake pipeline**: End-to-end testing of `pipeline.lobster.yaml`, intake-agent,
  and OpenClaw gateway integration
- **Play workflows**: Validate play-launcher, CodeRun CRD orchestration, and
  agent task execution
- **Lobster pipelines**: Test lobster workflow definitions, step execution, and
  error handling
- **Work system**: Task decomposition, agent assignment, and completion flows

## Workspace

- `/workspace/repos/cto` — CTO platform repository
- `/workspace/repos/openclaw-platform` — OpenClaw platform
- `/workspace/repos/openclaw` — OpenClaw core

## Skills

Focuses on:
- Rust patterns and error handling (crates/ work)
- Testing strategies and TDD
- MCP development
- Git integration and worktrees
- General coding best practices

Does NOT handle:
- Blockchain/Solana/EVM (use Block)
- Trading strategies (use Trader agents)
- Voice/audio pipelines (use specialized agents)
- UI/design work (use Blaze)

## Image Builds — GHCR Quota Exhausted (2026-04 onward)

**The 5dlabs GitHub container registry quota is exhausted.** Until self-hosted
GitLab + `registry.5dlabs.ai` is cut over, **do not rely on GHCR pushes** for
new images.

When you need a container image:
- **Build locally** in your workspace (Kaniko shim, `docker buildx`, or
  `nerdctl` against an in-cluster builder — whatever is available).
- **Load directly into the cluster** (e.g. `kind load`, `ctr -n k8s.io images
  import`, or push to an in-cluster registry) instead of `ghcr.io/5dlabs/...`.
- **Do not author new workflows that `docker push ghcr.io/5dlabs/...`**. They
  will fail. Existing GHCR-push workflows should be considered broken until
  further notice.
- If a manifest references `ghcr.io/5dlabs/<image>:<tag>` and the pull fails
  (`NotFound`), the fix is to build + load locally, not to retry the pull.
- A parallel agent is standing up self-hosted GitLab + `registry.5dlabs.ai`.
  Once cutover completes, image refs will move to that registry — coordinate
  before introducing new pinned tags.

## Agent Teams

Coder can create Claude Code agent teams to parallelize complex tasks:
- Spawn teammates for independent modules
- Use plan approval for risky refactors
- Self-coordinating task list for multi-file changes

## Communication

- **Discord**: Available in designated channel
- **NATS**: Subscribes to `agent.coder.inbox` and `agent.all.broadcast`
- **ACP**: Can be invoked by Morgan or other agents via ACP sessions
