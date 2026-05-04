# Morgan Sidecar Source Decision

Date: 2026-05-04

## Status

Accepted for CTO control-plane Wave 2B stub implementation.

## Decision

Build the first Hermes-compatible Morgan sidecar in `5dlabs/cto`, under a new bounded app package such as `apps/morgan-agent-sidecar`, and publish it as:

```text
ghcr.io/5dlabs/morgan-agent-sidecar:<semver-or-sha>
```

Keep `5dlabs/morgan-meet` as the Morgan-specific flagship demo/product plan and product asset source. Reconcile contracts with it continuously, but do not make the first Hermes CodeRun sidecar depend on Morgan Meet shipping its own runtime package.

This selects the `cto` repository for the first runnable sidecar source location, not a long-term rejection of a later shared Aperture package. After the stub sidecar passes a Hermes CodeRun smoke, promote the stable MCP/workspace contracts into a shared Aperture/Morgan package if reuse pressure is real.

## Why this is the right next step

The control-plane validation matrix is currently bottlenecked on Morgan rows `M-01` through `M-08`: there is design and routing evidence, but no durable sidecar image/package, no CodeRun attachment, no MCP discovery, no `meet-init` gate, and no `/workspace` stream proof.

The available repo evidence points in two directions:

- `5dlabs/morgan-meet` is now explicitly scoped as an OpenClaw-first Morgan Meet demo. Its PRD says Hermes/OpenCode/acpx support is an Aperture roadmap item, not the first Morgan Meet demo.
- `5dlabs/cto` already owns the Hermes CodeRun harness, the `hermes-presence-adapter` sidecar pattern, the controller rendering path, the validation matrix, and the live smoke harnesses needed to prove a Hermes sidecar.

Choosing `cto` for the first stub avoids waiting on a separate product-demo repo to become a reusable runtime package. It also keeps the initial implementation close to the CodeRun rendering and GitOps validation surfaces that must change anyway.

## Scope of the first implementation

The first `apps/morgan-agent-sidecar` implementation should be a stub/provider-free sidecar that proves the runtime contract only:

- `GET /healthz` returns ready when `/workspace` stream paths are writable.
- MCP endpoint exposes deterministic Morgan tools:
  - `morgan_session_start`
  - `morgan_session_stop`
  - `morgan_session_status`
  - `morgan_say`
  - `morgan_set_state`
  - `morgan_events_tail`
- Meeting aliases are present for Morgan Meet compatibility:
  - `meet_join`
  - `meet_leave`
  - `meet_get_status`
  - `meet_stream_audio` may return a controlled `not_implemented`/stub result until audio ingestion is wired.
- It writes per-run streams under the run workspace:
  - `morgan-events.jsonl`
  - `morgan-commands.jsonl`
  - `morgan-status.json`
- It never receives Discord bot tokens and does not call Discord APIs.
- It defaults to symbolic/audio-text stub mode when provider secrets are absent.

Provider integrations such as LiveKit, LemonSlice, Recall, Google Meet, and OpenClaw connector work remain out of scope for the stub. They should be added only after CodeRun sidecar attachment, MCP discovery, and workspace stream validation are green.

## Consequences

### Positive

- Unblocks `M-01` immediately as an accepted repo/package decision.
- Makes `M-02` through `M-08` implementable in the same repo that owns controller, harness, smoke scripts, and matrix evidence.
- Allows a no-secret, no-provider, repeatable CodeRun smoke before touching meeting credentials or avatar providers.
- Preserves Morgan Meet as the demo/product narrative while CTO proves the reusable Hermes runtime adapter.

### Tradeoffs

- There will be a short-term contract synchronization obligation between CTO docs/code and `5dlabs/morgan-meet` product docs.
- If Aperture becomes a separate reusable platform repo/package, code may later move out of `cto`.
- The first sidecar image is intentionally a stub and will not by itself prove live meeting/avatar UX.

## Implementation handoff

1. Create `apps/morgan-agent-sidecar` with TypeScript or Python, matching existing repo app conventions where practical.
2. Add local tests for health, MCP/tool listing, JSONL append/read, status snapshot writes, and SIGTERM/session stop behavior.
3. Add a Dockerfile and GitHub image publish workflow only after local tests/build are green.
4. Add controller/harness wiring behind env-driven enablement:
   - `MORGAN_AGENT_ENABLED=true`
   - `MORGAN_PROVIDER_MODE=stub`
   - `MORGAN_MCP_URL=http://127.0.0.1:4000/mcp`
   - stream path env vars under `/workspace/<run-subdir>/`.
5. Add a dry-run/live smoke harness that creates a Hermes CodeRun with Morgan enabled, verifies the `morgan-agent` sidecar, checks MCP discovery or health, and confirms stream files exist.
6. Keep all validation evidence no-secret: print resource names, route IDs, pod names, and redacted summaries only.

## Validation impact

This decision is documentation evidence for `M-01` only. It does not prove a sidecar image/package, CodeRun attachment, MCP discovery, `meet-init`, or workspace streams. Those rows remain `BLOCKED` or `NOT_STARTED` until implementation and live/stub smoke evidence exists.
