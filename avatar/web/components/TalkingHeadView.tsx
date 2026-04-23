"use client";

import {
  forwardRef,
  useEffect,
  useImperativeHandle,
  useRef,
  useState,
} from "react";
// Inside this component we never render with an empty glbUrl — the
// `AvatarRuntimeSurface` guard handles that case — so we avoid a
// setState-in-effect lint warning by not surfacing an empty-url error
// from here.
import type { TalkingHead as TalkingHeadClass } from "@met4citizen/talkinghead";
import {
  alignmentToVisemeTimeline,
  alignmentToWordTimeline,
  type AlignmentFrame,
} from "@/lib/lipsync/elevenlabs-to-oculus";

export type TalkingHeadHandle = {
  /**
   * Play a single reply utterance: audio + ElevenLabs alignment frame,
   * mapped internally to Oculus visemes + word timings.
   */
  speakUtterance(audio: AudioBuffer, alignment: AlignmentFrame | null): void;
  /**
   * Immediately cancel any in-flight audio / lip-sync animation.
   */
  interrupt(): void;
  /**
   * True once the avatar GLB has loaded and the instance is ready to
   * accept `speakUtterance` calls.
   */
  isReady(): boolean;
};

type TalkingHeadViewProps = {
  /** GLB url (Ready Player Me recommended). */
  glbUrl: string;
  /** Avatar body form, defaults to "F". */
  body?: "M" | "F";
  /** Called once the avatar GLB is loaded and the head instance is live. */
  onReady?: () => void;
  /** Called if the avatar fails to load. */
  onError?: (error: Error) => void;
};

const TalkingHeadView = forwardRef<TalkingHeadHandle, TalkingHeadViewProps>(
  function TalkingHeadView({ glbUrl, body = "F", onReady, onError }, ref) {
    const containerRef = useRef<HTMLDivElement | null>(null);
    const headRef = useRef<TalkingHeadClass | null>(null);
    const readyRef = useRef(false);
    const [loadError, setLoadError] = useState<string | null>(null);

    useEffect(() => {
      const container = containerRef.current;
      if (!container) {
        return;
      }

      if (!glbUrl) {
        // Surface missing-URL state lazily so we don't trip
        // react-hooks/set-state-in-effect on first render.
        onError?.(new Error("NEXT_PUBLIC_AVATAR_GLB_URL is not set."));
        return;
      }

      let cancelled = false;

      (async () => {
        try {
          // Load the TalkingHead module lazily so the bundler doesn't
          // statically chase its runtime dynamic imports (it pulls in
          // language-specific lipsync modules by filename at runtime,
          // which Turbopack can't resolve).
          const mod = await import("@met4citizen/talkinghead");
          if (cancelled) {
            return;
          }
          const TalkingHeadCtor = mod.TalkingHead;

          // Empty `lipsyncModules` — we feed Oculus visemes directly
          // from the voice-bridge, so TalkingHead does not need its
          // built-in language engine.
          const head = new TalkingHeadCtor(container, {
            modelFPS: 30,
            dracoEnabled: false,
            lipsyncModules: [],
            cameraView: "upper",
            avatarMood: "neutral",
          });
          headRef.current = head;

          await head.showAvatar({
            url: glbUrl,
            body,
            avatarMood: "neutral",
            avatarIdleEyeContact: 0.3,
            avatarSpeakingEyeContact: 0.6,
            avatarSpeakingHeadMove: 0.4,
          });
          if (cancelled) {
            try {
              head.stop();
            } catch {
              // ignore teardown race
            }
            headRef.current = null;
            return;
          }
          readyRef.current = true;
          onReady?.();
        } catch (cause) {
          if (cancelled) {
            return;
          }
          const message =
            cause instanceof Error
              ? cause.message
              : "TalkingHead.showAvatar failed";
          setLoadError(message);
          onError?.(cause instanceof Error ? cause : new Error(message));
        }
      })();

      return () => {
        cancelled = true;
        readyRef.current = false;
        const head = headRef.current;
        if (head) {
          try {
            head.stop();
          } catch {
            // Ignore teardown errors — the instance may never have started.
          }
        }
        headRef.current = null;
      };
    }, [body, glbUrl, onError, onReady]);

    useImperativeHandle(
      ref,
      () => ({
        speakUtterance(audio, alignment) {
          const head = headRef.current;
          if (!head || !readyRef.current) {
            return;
          }

          const payload: Parameters<typeof head.speakAudio>[0] = { audio };

          if (alignment) {
            const visemeTimeline = alignmentToVisemeTimeline(alignment);
            const wordTimeline = alignmentToWordTimeline(alignment);
            payload.visemes = visemeTimeline.visemes;
            payload.vtimes = visemeTimeline.vtimes;
            payload.vdurations = visemeTimeline.vdurations;
            payload.words = wordTimeline.words;
            payload.wtimes = wordTimeline.wtimes;
            payload.wdurations = wordTimeline.wdurations;
          }

          head.speakAudio(payload);
        },
        interrupt() {
          const head = headRef.current;
          if (!head) {
            return;
          }
          try {
            head.stopSpeaking();
          } catch {
            // stopSpeaking throws if nothing is queued; safe to ignore.
          }
        },
        isReady() {
          return readyRef.current;
        },
      }),
      [],
    );

    return (
      <div className="relative h-full w-full">
        <div
          ref={containerRef}
          className="h-full w-full"
          aria-label="Morgan 3D avatar"
        />
        {loadError ? (
          <div className="absolute inset-x-6 bottom-6 rounded-2xl border border-rose-400/30 bg-rose-950/80 px-4 py-3 text-sm text-rose-100">
            {loadError}
          </div>
        ) : null}
      </div>
    );
  },
);

export default TalkingHeadView;
