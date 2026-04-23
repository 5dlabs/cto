"use client";

import {
  forwardRef,
  useEffect,
  useImperativeHandle,
  useRef,
} from "react";
import type { TalkingHead as TalkingHeadClass } from "@met4citizen/talkinghead";
import type { HeadAudio as HeadAudioClass } from "@met4citizen/headaudio/dist/headaudio.min.mjs";

export type TalkingHeadHandle = {
  /**
   * Pipe a MediaStreamTrack (e.g. LiveKit's RemoteAudioTrack.mediaStreamTrack)
   * into HeadAudio for real-time viseme detection. Replaces any previous
   * attached track.
   */
  attachAudio(track: MediaStreamTrack): void;
  /**
   * Drop the currently attached audio track. Safe to call even if nothing
   * is attached.
   */
  detachAudio(): void;
  /**
   * True once the avatar GLB has loaded AND the HeadAudio worklet +
   * model are ready to process audio.
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

// HeadAudio's onvalue callback writes into head.mtAvatar[key]. These
// internal members aren't declared in the ambient TalkingHead types;
// narrow-cast at the point of use rather than widening the public type.
type TalkingHeadInternal = TalkingHeadClass & {
  audioCtx: AudioContext;
  mtAvatar: Record<string, { newvalue: number; needsUpdate: boolean }>;
  opt: {
    update?: ((delta: number) => void) | null;
  };
};

const HEADAUDIO_WORKLET_URL = "/headaudio/headworklet.min.mjs";
const HEADAUDIO_MODEL_URL = "/headaudio/model-en-mixed.bin";

const TalkingHeadView = forwardRef<TalkingHeadHandle, TalkingHeadViewProps>(
  function TalkingHeadView({ glbUrl, body = "F", onReady, onError }, ref) {
    const containerRef = useRef<HTMLDivElement | null>(null);
    const headRef = useRef<TalkingHeadInternal | null>(null);
    const headAudioRef = useRef<HeadAudioClass | null>(null);
    const sourceRef = useRef<MediaStreamAudioSourceNode | null>(null);
    const readyRef = useRef(false);
    // Track the most recent attach request made before HeadAudio was
    // ready so we can (re)connect it once the pipeline finishes loading.
    const pendingTrackRef = useRef<MediaStreamTrack | null>(null);

    useEffect(() => {
      const container = containerRef.current;
      if (!container) {
        return;
      }

      if (!glbUrl) {
        onError?.(new Error("NEXT_PUBLIC_AVATAR_GLB_URL is not set."));
        return;
      }

      let cancelled = false;

      (async () => {
        try {
          const [thMod, haMod] = await Promise.all([
            import("@met4citizen/talkinghead"),
            import("@met4citizen/headaudio/dist/headaudio.min.mjs"),
          ]);
          if (cancelled) {
            return;
          }

          const head = new thMod.TalkingHead(container, {
            modelFPS: 30,
            dracoEnabled: false,
            // We drive visemes via HeadAudio, not via TalkingHead's
            // per-language text-to-lipsync modules. Leaving this empty
            // keeps the bundle lean.
            lipsyncModules: [],
            cameraView: "upper",
            avatarMood: "neutral",
          }) as TalkingHeadInternal;
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

          // HeadAudio requires its worklet processor registered on the
          // same AudioContext that TalkingHead owns.
          await head.audioCtx.audioWorklet.addModule(HEADAUDIO_WORKLET_URL);

          const headAudio = new haMod.HeadAudio(head.audioCtx, {
            processorOptions: {},
            parameterData: {
              // Slightly permissive VAD: LiveKit audio arriving at
              // the browser tends to be attenuated after network jitter
              // buffering; lower thresholds help the detector activate
              // promptly on agent speech.
              vadGateActiveDb: -45,
              vadGateInactiveDb: -60,
            },
          });

          await headAudio.loadModel(HEADAUDIO_MODEL_URL);
          if (cancelled) {
            return;
          }

          headAudio.onvalue = (key, value) => {
            const target = head.mtAvatar[key];
            if (!target) {
              return;
            }
            target.newvalue = value;
            target.needsUpdate = true;
          };

          // Drive the HeadAudio update tick from TalkingHead's animation
          // loop; it consumes viseme events posted from the audio thread
          // and applies easing before writing to mtAvatar.
          head.opt.update = headAudio.update.bind(headAudio);

          headAudioRef.current = headAudio;
          readyRef.current = true;

          // If a track was queued while we were still loading, wire it
          // up now.
          const pending = pendingTrackRef.current;
          pendingTrackRef.current = null;
          if (pending) {
            try {
              const stream = new MediaStream([pending]);
              const source = head.audioCtx.createMediaStreamSource(stream);
              source.connect(headAudio);
              sourceRef.current = source;
            } catch (cause) {
              console.warn(
                "[talkinghead] failed to attach pending MediaStream",
                cause,
              );
            }
          }

          onReady?.();
        } catch (cause) {
          if (cancelled) {
            return;
          }
          const message =
            cause instanceof Error
              ? cause.message
              : "TalkingHead.showAvatar failed";
          onError?.(cause instanceof Error ? cause : new Error(message));
        }
      })();

      return () => {
        cancelled = true;
        readyRef.current = false;
        if (sourceRef.current) {
          try {
            sourceRef.current.disconnect();
          } catch {
            // ignore
          }
          sourceRef.current = null;
        }
        headAudioRef.current = null;
        const head = headRef.current;
        if (head) {
          try {
            head.stop();
          } catch {
            // ignore teardown errors — instance may never have started
          }
        }
        headRef.current = null;
        pendingTrackRef.current = null;
      };
    }, [body, glbUrl, onError, onReady]);

    useImperativeHandle(
      ref,
      () => ({
        attachAudio(track) {
          const head = headRef.current;
          const headAudio = headAudioRef.current;
          if (!head || !headAudio || !readyRef.current) {
            pendingTrackRef.current = track;
            return;
          }

          // Tear down any previous source so we don't fan-in multiple
          // tracks to the same HeadAudio node.
          if (sourceRef.current) {
            try {
              sourceRef.current.disconnect();
            } catch {
              // ignore
            }
            sourceRef.current = null;
          }

          try {
            const stream = new MediaStream([track]);
            const source = head.audioCtx.createMediaStreamSource(stream);
            source.connect(headAudio);
            sourceRef.current = source;
          } catch (cause) {
            console.warn(
              "[talkinghead] createMediaStreamSource failed",
              cause,
            );
          }
        },
        detachAudio() {
          pendingTrackRef.current = null;
          if (sourceRef.current) {
            try {
              sourceRef.current.disconnect();
            } catch {
              // ignore
            }
            sourceRef.current = null;
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
      </div>
    );
  },
);

export default TalkingHeadView;
