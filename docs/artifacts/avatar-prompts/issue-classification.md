# Issue Classification Guide for Avatar Implementation

When debugging or reporting issues during avatar development, classify them clearly:

## Recursive/Runtime Issues

Issues that occur **within** the avatar runtime itself (client-side):

- **Symptoms:** TypeScript errors, build failures, component rendering bugs, state machine loops
- **Examples:** `HostAvatarState` type mismatch, adapter not receiving frames, viseme scaffold producing wrong shapes
- **Where to log:** `docs/artifacts/avatar-prompts/` or commit messages with `fix(avatar-web):` prefix
- **Resolution path:** Fix types, adjust component logic, verify build

## ACP/Harness Issues

Issues that occur **in the agent tooling** used to develop the avatar:

- **Symptoms:** Subagent retry failures, context gaps, session timeouts, tool routing errors
- **Examples:** Copilot/Gemini retries returning "no prior context", acpx session drops, 1Password auth errors in unrelated pipelines
- **Where to log:** This file + commit messages with `fix(acp):` or `chore(subagent):` prefix
- **Resolution path:** Kill stale sessions, verify ACP backend health, restart gateway if needed

## Known Patterns

| Pattern | Classification | Fix |
|---|---|---|
| Subagent retry returns confused/off-task output | ACP/Harness | Kill session, don't retry same task |
| VoiceBridgeFrame type mismatch | Recursive/Runtime | Update avatar-state.ts types |
| `ingestBridgeFrame` never called | Recursive/Runtime | Wire VoiceBridgeIngestion component |
| Subagent drifts to unrelated pipeline (1Password, intake) | ACP/Harness | Kill and ignore; original brief already used |
| Build passes but avatar state stale | Recursive/Runtime | Check adapter.project() dependencies |
| `NEXT_PUBLIC_AVATAR_RUNTIME` not set | Recursive/Runtime | Default is deterministic (correct for Phase 1) |

## Phase 2 Completion Checklist

- [x] VoiceBridgeIngestion component created
- [x] WebSocket connection to voice-bridge established
- [x] `ingestBridgeFrame` wired to adapter
- [x] Derived-text adapter produces visemes from `reply_text`/`reply_delta` frames
- [x] ElevenLabs alignment adapter (`elevenlabs-alignment.ts`)
- [x] `stream_tts_with_timestamps()` in voice-bridge
- [x] Alignment frame type added to `VoiceBridgeFrame`
- [x] Deterministic adapter remains no-op (fallback safe)
- [x] TypeScript clean
- [x] Build passing
- [x] Voice-bridge tests green (4/4)
- [ ] End-to-end viseme demo with real audio (needs live voice-bridge deployment)

## Phase 3 Readiness

- [ ] Asset standardization (rig/blendshape spec, validation)
- [ ] TalkingHead runtime integration
- [ ] Tauri embedding validation
