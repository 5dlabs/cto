"""
Voice bridge for in-cluster Morgan (OpenClaw StatefulSet).

Runs as a sidecar in the openclaw-morgan-openclaw pod. Exposes a WebSocket
endpoint the CTO desktop app connects to through a Cloudflare tunnel:

  ws(s)://morgan-voice.5dlabs.ai/ws

Flow per user turn:
  1. Client opens WS, sends control frame {"type":"start", "session_id":"..."}.
  2. Client streams mic audio (PCM16 or Opus-in-webm chunks) as binary frames.
  3. Client sends {"type":"end_utterance"} when the user stops talking,
     optionally appending {"type":"text", "text":"..."} for keyboard input
     that should be merged with the spoken turn.
  4. Bridge runs STT (ElevenLabs Scribe) → concatenates with any text addendum
     → forwards to Morgan via the agent adapter (see agent_client.py TODO).
  5. Morgan's reply streams back token-by-token → bridge pipes to
     ElevenLabs TTS (Flash v2.5) → sends binary MP3 frames back to client.
  6. Bridge also sends {"type":"transcript", ...} and
     {"type":"reply_text", ...} JSON frames so the UI can render captions.

This skeleton is deliberately minimal: every network side-effect is a TODO
until we (a) confirm Morgan's input channel and (b) provision the tunnel.
"""

from __future__ import annotations

import asyncio
import json
import logging
import os
from typing import AsyncIterator

from fastapi import FastAPI, WebSocket, WebSocketDisconnect
from fastapi.middleware.cors import CORSMiddleware

from .agent_client import MorganAgentClient
from .elevenlabs_client import ElevenLabsClient

log = logging.getLogger("voice-bridge")
logging.basicConfig(level=os.environ.get("LOG_LEVEL", "INFO"))

app = FastAPI(title="morgan-voice-bridge", version="0.0.1")

# CORS left wide open for local dev; the prod tunnel will fence this by hostname.
app.add_middleware(
    CORSMiddleware,
    allow_origins=os.environ.get("ALLOWED_ORIGINS", "*").split(","),
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

tts = ElevenLabsClient(
    api_key=os.environ.get("ELEVENLABS_API_KEY", ""),
    voice_id=os.environ.get("MORGAN_VOICE_ID", "iP95p4xoKVk53GoZ742B"),
)
agent = MorganAgentClient(
    gateway_url=os.environ.get(
        "MORGAN_GATEWAY_URL",
        "http://openclaw-morgan.cto.svc.cluster.local:18789",
    ),
    gateway_token=os.environ.get("MORGAN_GATEWAY_TOKEN", "openclaw-internal"),
    model=os.environ.get("MORGAN_MODEL", "openclaw/morgan"),
)


@app.get("/healthz")
async def healthz() -> dict[str, str]:
    return {"status": "ok"}


@app.get("/readyz")
async def readyz() -> dict[str, object]:
    return {
        "status": "ok",
        "tts_configured": tts.is_configured,
        "agent_configured": agent.is_configured,
    }


@app.websocket("/ws")
async def voice_ws(ws: WebSocket) -> None:
    await ws.accept()
    session_id: str | None = None
    audio_chunks: list[bytes] = []
    text_addendum: str = ""

    try:
        while True:
            msg = await ws.receive()
            if "bytes" in msg and msg["bytes"] is not None:
                audio_chunks.append(msg["bytes"])
                continue
            if "text" not in msg or msg["text"] is None:
                continue
            frame = json.loads(msg["text"])
            kind = frame.get("type")

            if kind == "start":
                session_id = frame.get("session_id") or "anon"
                audio_chunks = []
                text_addendum = ""
                await ws.send_json({"type": "started", "session_id": session_id})
            elif kind == "text":
                text_addendum = f"{text_addendum}\n{frame.get('text', '')}".strip()
            elif kind == "end_utterance":
                if not session_id:
                    await ws.send_json({"type": "error", "error": "no active session"})
                    continue
                await _handle_turn(ws, session_id, audio_chunks, text_addendum)
                audio_chunks = []
                text_addendum = ""
            elif kind == "stop":
                break
            else:
                log.warning("unknown frame type: %s", kind)
    except WebSocketDisconnect:
        log.info("client disconnected (session=%s)", session_id)
    finally:
        try:
            await ws.close()
        except Exception:
            pass


async def _handle_turn(
    ws: WebSocket,
    session_id: str,
    audio_chunks: list[bytes],
    text_addendum: str,
) -> None:
    # 1. STT — send concatenated audio to ElevenLabs Scribe.
    transcript = ""
    if audio_chunks:
        content_type = os.environ.get("VOICE_BRIDGE_AUDIO_MIME", "audio/webm")
        filename = os.environ.get("VOICE_BRIDGE_AUDIO_NAME", "turn.webm")
        transcript = await tts.transcribe(
            b"".join(audio_chunks),
            content_type=content_type,
            filename=filename,
        )
    user_text = "\n".join(p for p in (transcript, text_addendum) if p).strip()
    if not user_text:
        await ws.send_json({"type": "error", "error": "empty utterance"})
        return

    await ws.send_json({"type": "transcript", "text": user_text})

    # 2. Forward to Morgan and stream reply tokens back.
    reply_text_buf: list[str] = []
    async for token in _stream_agent_reply(session_id, user_text):
        reply_text_buf.append(token)
        await ws.send_json({"type": "reply_delta", "text": token})

    full_reply = "".join(reply_text_buf).strip()
    await ws.send_json({"type": "reply_text", "text": full_reply})

    # 3. TTS — stream MP3 chunks back over the socket as binary frames.
    async for mp3_chunk in tts.stream_tts(full_reply):
        await ws.send_bytes(mp3_chunk)

    await ws.send_json({"type": "turn_done"})


async def _stream_agent_reply(session_id: str, text: str) -> AsyncIterator[str]:
    try:
        async for tok in agent.send_and_stream(session_id=session_id, text=text):
            yield tok
    except Exception as exc:  # noqa: BLE001
        log.exception("agent stream failed: %s", exc)
        yield f"[voice-bridge error: {exc}]"
        await asyncio.sleep(0)
