import type { AvatarRuntimeAdapter, VoiceBridgeFrame } from "@/lib/avatar-state";
import { useCallback, useEffect, useRef } from "react";

type VoiceBridgeIngestionProps = {
  adapter: AvatarRuntimeAdapter;
  bridgeUrl?: string | null;
  enabled?: boolean;
};

/**
 * Connects to the voice-bridge WebSocket and feeds frames into the
 * active AvatarRuntimeAdapter. Gracefully handles missing bridge URL,
 * connection failures, and adapter changes.
 *
 * When `enabled` is false or `bridgeUrl` is null/empty, does nothing —
 * the deterministic adapter continues to work without any bridge connection.
 */
export default function VoiceBridgeIngestion({
  adapter,
  bridgeUrl,
  enabled = true,
}: VoiceBridgeIngestionProps) {
  const wsRef = useRef<WebSocket | null>(null);
  const adapterRef = useRef(adapter);

  // Keep adapter ref current so the onmessage handler sees the latest adapter
  useEffect(() => {
    adapterRef.current = adapter;
  }, [adapter]);

  const connect = useCallback(() => {
    if (!enabled || !bridgeUrl) {
      return;
    }

    if (wsRef.current && wsRef.current.readyState <= WebSocket.OPEN) {
      return; // already connected or connecting
    }

    try {
      const ws = new WebSocket(bridgeUrl);
      wsRef.current = ws;

      ws.onopen = () => {
        // Send a "start" frame to initiate the session
        ws.send(
          JSON.stringify({
            type: "start",
            session_id: `avatar-web-${Date.now()}`,
          }),
        );
      };

      ws.onmessage = (event) => {
        // Voice-bridge sends JSON frames and raw MP3 bytes
        // Only JSON frames are relevant for viseme ingestion
        if (typeof event.data === "string") {
          try {
            const frame = JSON.parse(event.data) as VoiceBridgeFrame;
            if (frame && typeof frame.type === "string") {
              adapterRef.current.ingestBridgeFrame?.(frame);
            }
          } catch {
            // Ignore non-JSON text (shouldn't happen, but be defensive)
          }
        }
        // Raw binary MP3 chunks are ignored — the adapter doesn't need them
      };

      ws.onclose = () => {
        wsRef.current = null;
        // Attempt reconnect after a delay
        setTimeout(connect, 3000);
      };

      ws.onerror = () => {
        wsRef.current = null;
      };
    } catch {
      // WebSocket construction failed (e.g. invalid URL) — silently ignore
    }
  }, [bridgeUrl, enabled]);

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
    };
  }, [connect]);

  return null; // renders nothing visible
}
