"use client";

import dynamic from "next/dynamic";
import type { Ref } from "react";
import DeterministicAvatar from "@/components/DeterministicAvatar";
import type { AvatarSessionState, AvatarStatePayload } from "@/lib/avatar-state";
import type { TalkingHeadHandle } from "@/components/TalkingHeadView";
import { VideoTrack } from "@livekit/components-react";

// TalkingHead pulls in three.js and a WebGL context; it has no SSR path,
// so we load it client-side only.
const TalkingHeadView = dynamic(() => import("@/components/TalkingHeadView"), {
  ssr: false,
  loading: () => (
    <div className="flex h-full items-center justify-center px-8 text-center text-sm text-slate-300">
      Loading 3D avatar…
    </div>
  ),
});

type AvatarRuntimeSurfaceProps = {
  compact?: boolean;
  state: AvatarStatePayload;
  videoTrack: unknown | null;
  /**
   * Ref to the active TalkingHead instance. `VoiceBridgeIngestion` calls
   * `current.speakUtterance(...)` once an MP3 utterance has been decoded
   * alongside its alignment frame.
   */
  talkingHeadRef?: Ref<TalkingHeadHandle>;
  /** Ready Player Me (or any ARKit+Oculus rigged) .glb URL. */
  glbUrl?: string;
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
  talkingHeadRef,
  glbUrl,
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
      case "talkinghead":
        if (!glbUrl) {
          return (
            <div className="flex h-full items-center justify-center px-8 text-center text-sm text-slate-300">
              Set <code className="font-mono">NEXT_PUBLIC_AVATAR_GLB_URL</code>{" "}
              to a Ready Player Me .glb to enable the TalkingHead runtime.
            </div>
          );
        }
        return <TalkingHeadView ref={talkingHeadRef} glbUrl={glbUrl} />;
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
