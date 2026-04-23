"use client";

import { useVoiceAssistant } from "@livekit/components-react";
import { useEffect, type RefObject } from "react";
import type { TalkingHeadHandle } from "@/components/TalkingHeadView";
import type { RemoteTrackPublication } from "livekit-client";

type LiveKitAudioBridgeProps = {
  talkingHeadRef: RefObject<TalkingHeadHandle | null>;
};

/**
 * Subscribes to the agent's remote audio track via LiveKit's voice-assistant
 * hook and pipes the underlying MediaStreamTrack into the TalkingHead
 * instance for real-time lip-sync through HeadAudio.
 *
 * Playback is handled separately (see `AssistantAudioRenderer` in Room.tsx),
 * which both delivers audible output to the user and keeps Chrome's
 * MediaStreamTrack "flowing" for the Web Audio graph.
 */
export default function LiveKitAudioBridge({
  talkingHeadRef,
}: LiveKitAudioBridgeProps) {
  const { audioTrack } = useVoiceAssistant();

  // RemoteAudioTrack exposes the raw MediaStreamTrack through its
  // publication. `audioTrack` from useVoiceAssistant is a TrackReference,
  // not the track itself — reach through to the publication.
  const publication = audioTrack?.publication as
    | RemoteTrackPublication
    | undefined;
  const mediaStreamTrack = publication?.track?.mediaStreamTrack ?? null;

  useEffect(() => {
    if (!mediaStreamTrack) {
      talkingHeadRef.current?.detachAudio();
      return;
    }
    talkingHeadRef.current?.attachAudio(mediaStreamTrack);
    return () => {
      talkingHeadRef.current?.detachAudio();
    };
  }, [mediaStreamTrack, talkingHeadRef]);

  return null;
}
