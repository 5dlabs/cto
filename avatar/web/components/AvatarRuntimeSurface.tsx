import DeterministicAvatar from "@/components/DeterministicAvatar";
import type { AvatarSessionState, AvatarStatePayload } from "@/lib/avatar-state";
import { VideoTrack } from "@livekit/components-react";

type AvatarRuntimeSurfaceProps = {
  compact?: boolean;
  state: AvatarStatePayload;
  videoTrack: unknown | null;
  /**
   * Latest `cto-avatar-session/v1` SESSION_STATE, if a subscriber is wired.
   * Rendered as a `data-session-state` attribute on a `display:contents`
   * wrapper so the surrounding layout is unchanged. Optional — existing
   * callers that don't yet consume SESSION_STATE frames keep working.
   */
  sessionState?: AvatarSessionState;
};

export default function AvatarRuntimeSurface({
  compact = false,
  state,
  videoTrack,
  sessionState,
}: AvatarRuntimeSurfaceProps) {
  const inner = (() => {
    if (videoTrack) {
      return (
        <VideoTrack
          trackRef={videoTrack as never}
          className="h-full w-full object-cover"
        />
      );
    }

    switch (state.runtime.kind) {
      case "deterministic-fallback":
      default:
        return (
          <DeterministicAvatar
            compact={compact}
            voiceState={state.voiceState}
            latestUserText={state.transcript.latestUserText}
            latestAgentText={state.transcript.latestAgentText}
          />
        );
    }
  })();

  if (sessionState) {
    return (
      <div style={{ display: "contents" }} data-session-state={sessionState}>
        {inner}
      </div>
    );
  }
  return inner;
}
