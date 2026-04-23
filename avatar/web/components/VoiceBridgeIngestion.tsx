"use client";

import type { RefObject } from "react";
import { useCallback, useEffect, useRef } from "react";
import type { TalkingHeadHandle } from "@/components/TalkingHeadView";
import type { AvatarRuntimeAdapter, VoiceBridgeFrame } from "@/lib/avatar-state";
import type { AlignmentFrame } from "@/lib/lipsync/elevenlabs-to-oculus";

type VoiceBridgeIngestionProps = {
  adapter: AvatarRuntimeAdapter;
  bridgeUrl?: string | null;
  enabled?: boolean;
  /**
   * Optional handle to the TalkingHead instance. When the active adapter is
   * `talkinghead`, each completed utterance is handed off to
   * `talkingHeadRef.current.speakUtterance(...)` along with its alignment
   * frame. Unset / mismatched adapters cause the MP3 stream to be ignored.
   */
  talkingHeadRef?: RefObject<TalkingHeadHandle | null>;
};

const BASE64_CHAR_REGEX = /^[A-Za-z0-9+/=\r\n]+$/;

/**
 * The voice-bridge has two audio wire formats depending on whether
 * ElevenLabs alignment is enabled:
 *   1. Raw MP3 bytes in a binary WebSocket frame.
 *   2. Base64-ASCII of MP3 bytes in a binary WebSocket frame (current
 *      behaviour when VOICE_BRIDGE_ENABLE_ALIGNMENT=1 — the bridge calls
 *      `send_bytes(audio_base64)` with the base64 string, which Starlette
 *      encodes as UTF-8 on the wire).
 *
 * We sniff the first few bytes: MP3 starts with an MPEG frame sync
 * (0xFF 0xFB / 0xFA / 0xF3 / 0xF2) or an ID3 tag ("ID3"). If neither,
 * assume base64 ASCII and decode.
 */
function normaliseAudioChunk(chunk: Uint8Array): Uint8Array {
  if (chunk.length === 0) {
    return chunk;
  }

  const first = chunk[0];
  const isMpegSync =
    first === 0xff &&
    chunk.length > 1 &&
    (chunk[1] & 0xe0) === 0xe0;
  const isId3 =
    chunk.length >= 3 &&
    chunk[0] === 0x49 &&
    chunk[1] === 0x44 &&
    chunk[2] === 0x33;

  if (isMpegSync || isId3) {
    return chunk;
  }

  // Fall through to base64. Decode using atob since the payload is ASCII.
  try {
    const ascii = new TextDecoder("ascii").decode(chunk);
    if (!BASE64_CHAR_REGEX.test(ascii.trim())) {
      return chunk; // give up and let decodeAudioData surface the error
    }
    const binary = atob(ascii);
    const decoded = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i += 1) {
      decoded[i] = binary.charCodeAt(i);
    }
    return decoded;
  } catch {
    return chunk;
  }
}

function concatBytes(chunks: Uint8Array[]): Uint8Array {
  let total = 0;
  for (const c of chunks) {
    total += c.length;
  }
  const out = new Uint8Array(total);
  let offset = 0;
  for (const c of chunks) {
    out.set(c, offset);
    offset += c.length;
  }
  return out;
}

/**
 * Connects to the voice-bridge WebSocket and feeds frames into the
 * active AvatarRuntimeAdapter. Gracefully handles missing bridge URL,
 * connection failures, and adapter changes.
 *
 * For TalkingHead runtimes, also buffers MP3 audio chunks + the matching
 * ElevenLabs alignment frame per utterance, decodes to an `AudioBuffer`,
 * and dispatches to the TalkingHead instance via `speakUtterance`.
 *
 * When `enabled` is false or `bridgeUrl` is null/empty, does nothing —
 * the deterministic adapter continues to work without any bridge connection.
 */
export default function VoiceBridgeIngestion({
  adapter,
  bridgeUrl,
  enabled = true,
  talkingHeadRef,
}: VoiceBridgeIngestionProps) {
  const wsRef = useRef<WebSocket | null>(null);
  const adapterRef = useRef(adapter);
  const audioCtxRef = useRef<AudioContext | null>(null);
  const mp3BufferRef = useRef<Uint8Array[]>([]);
  const alignmentRef = useRef<AlignmentFrame | null>(null);
  const connectRef = useRef<() => void>(() => {});

  const isTalkingHead = adapter.kind === "talkinghead";

  useEffect(() => {
    adapterRef.current = adapter;
  }, [adapter]);

  const resetUtteranceBuffers = useCallback(() => {
    mp3BufferRef.current = [];
    alignmentRef.current = null;
  }, []);

  const ensureAudioContext = useCallback((): AudioContext | null => {
    if (audioCtxRef.current) {
      return audioCtxRef.current;
    }
    if (typeof window === "undefined") {
      return null;
    }
    try {
      audioCtxRef.current = new AudioContext({ sampleRate: 22050 });
      return audioCtxRef.current;
    } catch {
      try {
        audioCtxRef.current = new AudioContext();
        return audioCtxRef.current;
      } catch {
        return null;
      }
    }
  }, []);

  const flushUtterance = useCallback(async () => {
    if (!isTalkingHead) {
      resetUtteranceBuffers();
      return;
    }

    const chunks = mp3BufferRef.current;
    const alignment = alignmentRef.current;
    resetUtteranceBuffers();

    if (chunks.length === 0) {
      return;
    }

    const audioCtx = ensureAudioContext();
    if (!audioCtx) {
      return;
    }

    const normalised = chunks.map(normaliseAudioChunk);
    const mp3Bytes = concatBytes(normalised);

    try {
      const audioBuffer = await audioCtx.decodeAudioData(mp3Bytes.buffer as ArrayBuffer);
      const handle = talkingHeadRef?.current;
      if (!handle?.isReady()) {
        return;
      }
      handle.speakUtterance(audioBuffer, alignment);
    } catch (cause) {
      console.warn("[voice-bridge] failed to decode utterance audio", cause);
    }
  }, [ensureAudioContext, isTalkingHead, resetUtteranceBuffers, talkingHeadRef]);

  const connect = useCallback(() => {
    if (!enabled || !bridgeUrl) {
      return;
    }

    if (wsRef.current && wsRef.current.readyState <= WebSocket.OPEN) {
      return;
    }

    try {
      const ws = new WebSocket(bridgeUrl);
      ws.binaryType = "arraybuffer";
      wsRef.current = ws;

      ws.onopen = () => {
        ws.send(
          JSON.stringify({
            type: "start",
            session_id: `avatar-web-${Date.now()}`,
          }),
        );
      };

      ws.onmessage = (event) => {
        if (typeof event.data === "string") {
          try {
            const frame = JSON.parse(event.data) as VoiceBridgeFrame;
            if (!frame || typeof frame.type !== "string") {
              return;
            }

            adapterRef.current.ingestBridgeFrame?.(frame);

            if (isTalkingHead) {
              switch (frame.type) {
                case "transcript":
                  // A fresh user turn landed — if Morgan was mid-reply,
                  // cancel in-flight speech so the new answer can play.
                  talkingHeadRef?.current?.interrupt();
                  resetUtteranceBuffers();
                  break;
                case "reply_text":
                  // Reply text is finalised immediately before TTS starts.
                  // Clear any stray buffers from a prior turn.
                  resetUtteranceBuffers();
                  break;
                case "alignment":
                  alignmentRef.current = frame as AlignmentFrame;
                  break;
                case "turn_done":
                  void flushUtterance();
                  break;
                default:
                  break;
              }
            }
          } catch {
            // Ignore non-JSON text
          }
          return;
        }

        if (!isTalkingHead) {
          return;
        }

        // Binary frame: MP3 chunk (or base64 ASCII of MP3 bytes).
        if (event.data instanceof ArrayBuffer) {
          mp3BufferRef.current.push(new Uint8Array(event.data));
        } else if (event.data instanceof Blob) {
          // Some browsers default to Blob; read synchronously via Response.
          void event.data.arrayBuffer().then((buf) => {
            mp3BufferRef.current.push(new Uint8Array(buf));
          });
        }
      };

      ws.onclose = () => {
        wsRef.current = null;
        // Reconnect via a ref to avoid referring to `connect` before its
        // declaration inside its own useCallback body.
        setTimeout(() => connectRef.current(), 3000);
      };

      ws.onerror = () => {
        wsRef.current = null;
      };
    } catch {
      // WebSocket construction failed (e.g. invalid URL) — silently ignore
    }
  }, [
    bridgeUrl,
    enabled,
    flushUtterance,
    isTalkingHead,
    resetUtteranceBuffers,
    talkingHeadRef,
  ]);

  useEffect(() => {
    connectRef.current = connect;
  }, [connect]);

  useEffect(() => {
    connect();

    return () => {
      if (wsRef.current) {
        try {
          wsRef.current.close();
        } catch {
          // ignore
        }
        wsRef.current = null;
      }
      const ctx = audioCtxRef.current;
      if (ctx && ctx.state !== "closed") {
        void ctx.close().catch(() => {});
      }
      audioCtxRef.current = null;
      mp3BufferRef.current = [];
      alignmentRef.current = null;
    };
  }, [connect]);

  return null;
}
