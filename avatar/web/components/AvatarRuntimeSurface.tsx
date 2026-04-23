import DeterministicAvatar from "@/components/DeterministicAvatar";
import type { AvatarStatePayload } from "@/lib/avatar-state";
import { VideoTrack } from "@livekit/components-react";

type AvatarRuntimeSurfaceProps = {
  compact?: boolean;
  state: AvatarStatePayload;
  videoTrack: unknown | null;
};

export default function AvatarRuntimeSurface({
  compact = false,
  state,
  videoTrack,
}: AvatarRuntimeSurfaceProps) {
  if (videoTrack) {
    return (
      <VideoTrack
        // livekit types are awkward here, but Room already guarantees a valid trackRef
        trackRef={videoTrack as never}
        className="h-full w-full object-cover"
      />
    );
  }

  switch (state.runtime.kind) {
    case "talkinghead":
      return (
        <div className="flex h-full items-center justify-center px-8 text-center text-sm text-slate-300">
          TalkingHead runtime hook is reserved here. Falling back until the 3D runtime is wired.
        </div>
      );
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
}
