from __future__ import annotations

import asyncio
import base64
import json
import logging
import os
import time
from collections import defaultdict, deque
from typing import AsyncIterator

from fastapi import FastAPI, WebSocket, WebSocketDisconnect
from fastapi.middleware.cors import CORSMiddleware
from starlette.websockets import WebSocketState

from .agent_client import MorganAgentClient
from .elevenlabs_client import ElevenLabsClient
from .voice_agents import AgentSpec, get_agent

log = logging.getLogger("voice-bridge")
logging.basicConfig(level=os.environ.get("LOG_LEVEL", "INFO"))

app = FastAPI(title="morgan-voice-bridge", version="0.0.1")

app.add_middleware(
    CORSMiddleware,
    allow_origins=os.environ.get("ALLOWED_ORIGINS", "*").split(","),
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

_BASE_TTS = ElevenLabsClient(
    api_key=os.environ.get("ELEVENLABS_API_KEY", ""),
    voice_id=os.environ.get("MORGAN_VOICE_ID", "iP95p4xoKVk53GoZ742B"),
)
_ALIGNMENT_ENABLED = os.environ.get("VOICE_BRIDGE_ENABLE_ALIGNMENT", "0") == "1"
_AUTH_SHARED_SECRET = os.environ.get("VOICE_BRIDGE_SHARED_SECRET", "")
_RATE_LIMIT_MAX_TURNS = max(1, int(os.environ.get("VOICE_BRIDGE_MAX_TURNS", "20")))
_RATE_LIMIT_WINDOW_S = max(1, int(os.environ.get("VOICE_BRIDGE_RATE_WINDOW_S", "60")))
_turn_windows: dict[str, deque[float]] = defaultdict(deque)
_turn_counters: dict[str, int] = defaultdict(int)


@app.get("/healthz")
async def healthz() -> dict[str, str]:
    return {"status": "ok"}


@app.get("/readyz")
async def readyz() -> dict[str, object]:
    default_agent = get_agent(None)
    return {
        "status": "ok",
        "tts_configured": _BASE_TTS.is_configured,
        "agent_configured": bool(default_agent),
        "default_agent": default_agent.name if default_agent else None,
    }


@app.websocket("/ws")
async def voice_ws(ws: WebSocket) -> None:
    agent_name = ws.query_params.get("agent")
    agent_spec = get_agent(agent_name)
    if agent_spec is None:
        await ws.accept()
        await ws.close(code=4404, reason="unknown_agent")
        return

    if not _is_authorized(ws):
        await ws.accept()
        await ws.send_json({"type": "error", "error": "unauthorized"})
        await ws.close(code=4401, reason="unauthorized")
        return

    await ws.accept()
    session_id: str | None = None
    audio_chunks: list[bytes] = []
    text_addendum = ""
    client_key = _client_key(ws, agent_spec)

    try:
        while True:
            msg = await ws.receive()
            if msg.get("type") == "websocket.disconnect":
                break
            if msg.get("bytes") is not None:
                audio_chunks.append(msg["bytes"])
                continue
            if msg.get("text") is None:
                continue
            frame = json.loads(msg["text"])
            kind = frame.get("type")

            if kind == "start":
                session_id = frame.get("session_id") or "anon"
                audio_chunks = []
                text_addendum = ""
                await ws.send_json(
                    {
                        "type": "started",
                        "session_id": session_id,
                        "agent": agent_spec.name,
                    }
                )
            elif kind == "text":
                text_addendum = f"{text_addendum}\n{frame.get('text', '')}".strip()
            elif kind == "end_utterance":
                if not session_id:
                    await ws.send_json({"type": "error", "error": "no_active_session"})
                    continue
                if _is_rate_limited(client_key):
                    await ws.send_json({"type": "error", "error": "rate_limited"})
                    continue
                await _handle_turn(
                    ws,
                    agent_spec=agent_spec,
                    session_id=session_id,
                    audio_chunks=audio_chunks,
                    text_addendum=text_addendum,
                )
                audio_chunks = []
                text_addendum = ""
            elif kind == "stop":
                break
            else:
                log.warning("unknown frame type: %s", kind)
    except WebSocketDisconnect:
        log.info("client disconnected cleanly (session=%s agent=%s)", session_id, agent_spec.name)
    finally:
        await _safe_close(ws)


def _is_authorized(ws: WebSocket) -> bool:
    if not _AUTH_SHARED_SECRET:
        return True
    token = (
        ws.query_params.get("token")
        or ws.headers.get("x-voice-bridge-token")
        or ws.headers.get("authorization", "").removeprefix("Bearer ").strip()
    )
    if token == _AUTH_SHARED_SECRET:
        return True
    log.warning("rejecting unauthorized websocket request")
    return False


def _client_key(ws: WebSocket, agent_spec: AgentSpec) -> str:
    forwarded_for = ws.headers.get("x-forwarded-for", "")
    ip = forwarded_for.split(",")[0].strip() if forwarded_for else (ws.client.host if ws.client else "unknown")
    return f"{agent_spec.name}:{ip}"


def _is_rate_limited(client_key: str) -> bool:
    now = time.monotonic()
    window = _turn_windows[client_key]
    while window and now - window[0] > _RATE_LIMIT_WINDOW_S:
        window.popleft()
    if len(window) >= _RATE_LIMIT_MAX_TURNS:
        return True
    window.append(now)
    return False


async def _safe_close(ws: WebSocket) -> None:
    try:
        if ws.application_state != WebSocketState.DISCONNECTED:
            await ws.close()
    except RuntimeError:
        pass
    except Exception:
        log.debug("websocket close ignored", exc_info=True)


async def _handle_turn(
    ws: WebSocket,
    *,
    agent_spec: AgentSpec,
    session_id: str,
    audio_chunks: list[bytes],
    text_addendum: str,
) -> None:
    started = time.monotonic()
    transcript = ""
    if audio_chunks:
        content_type = os.environ.get("VOICE_BRIDGE_AUDIO_MIME", "audio/webm")
        filename = os.environ.get("VOICE_BRIDGE_AUDIO_NAME", "turn.webm")
        transcript = await _BASE_TTS.transcribe(
            b"".join(audio_chunks),
            content_type=content_type,
            filename=filename,
        )
    user_text = "\n".join(part for part in (transcript, text_addendum) if part).strip()
    if not user_text:
        await ws.send_json({"type": "error", "error": "empty_utterance"})
        return

    await ws.send_json({"type": "transcript", "text": user_text, "agent": agent_spec.name})

    reply_text_buf: list[str] = []
    agent_client = MorganAgentClient(agent_spec)
    async for token in _stream_agent_reply(agent_client, session_id, user_text):
        reply_text_buf.append(token)
        await ws.send_json({"type": "reply_delta", "text": token, "agent": agent_spec.name})

    full_reply = "".join(reply_text_buf).strip()
    await ws.send_json({"type": "reply_text", "text": full_reply, "agent": agent_spec.name})

    tts_client = _BASE_TTS.with_voice(agent_spec.voice_id)
    tts_bytes = 0

    if _ALIGNMENT_ENABLED:
        async for frame in tts_client.stream_tts_with_timestamps(full_reply):
            audio_b64 = frame.get("audio_base64", "")
            if audio_b64:
                try:
                    mp3_bytes = base64.b64decode(audio_b64)
                except (ValueError, TypeError):
                    log.warning("skipping malformed audio_base64 frame")
                    mp3_bytes = b""
                if mp3_bytes:
                    # Send decoded MP3 on the binary channel so the client
                    # can feed AudioContext.decodeAudioData() directly.
                    await ws.send_bytes(mp3_bytes)
                    tts_bytes += len(mp3_bytes)
            alignment = frame.get("alignment")
            if alignment:
                # ElevenLabs returns per-character timings that cover the
                # characters being spoken in this audio chunk, not the full
                # reply — align the chars array accordingly so the client
                # gets a one-to-one map.
                char_start_s = alignment.get("character_start_times_seconds", [])
                char_end_s = alignment.get("character_end_times_seconds", [])
                align_chars = alignment.get("characters")
                if align_chars is None:
                    align_chars = list(full_reply)[: len(char_start_s)]
                await ws.send_json({
                    "type": "alignment",
                    "atMs": round(alignment.get("audio_start_seconds", 0) * 1000),
                    "chars": list(align_chars),
                    "char_start_ms": [round(t * 1000) for t in char_start_s],
                    "char_end_ms": [round(t * 1000) for t in char_end_s],
                    "agent": agent_spec.name,
                })
    else:
        async for mp3_chunk in tts_client.stream_tts(full_reply):
            tts_bytes += len(mp3_chunk)
            await ws.send_bytes(mp3_chunk)

    _turn_counters[agent_spec.name] += 1
    log.info(
        json.dumps(
            {
                "type": "turn_metrics",
                "agent": agent_spec.name,
                "session_id": session_id,
                "turn_count": _turn_counters[agent_spec.name],
                "audio_chunks": len(audio_chunks),
                "transcript_chars": len(user_text),
                "reply_chars": len(full_reply),
                "tts_bytes": tts_bytes,
                "duration_ms": round((time.monotonic() - started) * 1000, 2),
            }
        )
    )
    await ws.send_json({"type": "turn_done", "agent": agent_spec.name})


async def _stream_agent_reply(
    agent_client: MorganAgentClient,
    session_id: str,
    text: str,
) -> AsyncIterator[str]:
    try:
        async for tok in agent_client.send_and_stream(session_id=session_id, text=text):
            yield tok
    except Exception as exc:
        log.exception("agent stream failed: %s", exc)
        yield f"[voice-bridge error: {exc}]"
        await asyncio.sleep(0)
