"""FastAPI application for narrator sidecar."""

from __future__ import annotations

import asyncio
import logging
import os
import signal
import time

import uvicorn
from aiortc import RTCPeerConnection, RTCSessionDescription
from fastapi import FastAPI, HTTPException, Response

from app.acp_parser import ACPEvent, parse_line
from app.config import settings
from app.interrupt import write_interrupt
from app.narrator import Narrator
from app.session import SessionRegistry, SessionState
from app.tailer import tail_acp_stream
from app.tts import TTSEngine
from app.webrtc import NarratorAudioTrack, WebRTCSession

logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s [%(name)s] %(message)s")
log = logging.getLogger("main")

registry = SessionRegistry()
app = FastAPI(title="Narrator Sidecar", version="0.1.0")

_shutdown = asyncio.Event()
_narrator: Narrator | None = None
_tts: TTSEngine | None = None


@app.on_event("startup")
async def startup():
    global _narrator, _tts
    log.info("Narrator sidecar starting (backend=%s, port=%d)", settings.backend, settings.port)
    _narrator = Narrator()
    _tts = TTSEngine(settings.tts_model_path, settings.voice_sample_path)
    # Preload TTS in background
    asyncio.create_task(_tts.preload())


@app.on_event("shutdown")
async def shutdown():
    await registry.cleanup_all()
    log.info("Narrator sidecar shut down")


@app.get("/healthz")
async def healthz():
    return {"status": "ok"}


@app.get("/readyz")
async def readyz(response: Response):
    tts_ready = _tts.ready if _tts else False
    if not tts_ready:
        response.status_code = 503
        return {"status": "starting", "tts_ready": False}
    return {"status": "ready", "tts_ready": True}


@app.get("/info")
async def info():
    return {
        "backend": settings.backend,
        "version": "0.1.0",
        "capabilities": ["audio", "video", "interrupt"],
    }


@app.post("/sessions")
async def create_session(req: dict) -> dict:
    session_id = req.get("session_id", f"session-{int(time.time())}")
    persona_id = req.get("persona_id", "blaze")
    webrtc_offer = req.get("webrtc_offer")

    if not webrtc_offer:
        raise HTTPException(status_code=400, detail="webrtc_offer required")

    state = registry.create(session_id, persona_id)

    # Set up WebRTC
    webrtc = WebRTCSession()
    state.pc = webrtc.pc

    @webrtc.pc.on("iceconnectionstatechange")
    async def on_ice_state():
        log.info("ICE state: %s (session=%s)", webrtc.pc.iceConnectionState, session_id)
        if webrtc.pc.iceConnectionState in ("failed", "closed"):
            await registry.delete(session_id)

    # Handle SDP
    answer = await webrtc.handle_offer(webrtc_offer["sdp"], webrtc_offer["type"])

    # Start tailing ACP stream and generating narration
    state.tailer_task = asyncio.create_task(_run_tailer_and_narrate(state, webrtc))

    return {"session_id": session_id, "webrtc_answer": answer}


@app.delete("/sessions/{session_id}")
async def delete_session(session_id: str):
    state = registry.get(session_id)
    if not state:
        raise HTTPException(status_code=404, detail="session not found")
    await registry.delete(session_id)
    return {"status": "deleted"}


@app.post("/sessions/{session_id}/interrupt")
async def interrupt_session(session_id: str, req: dict):
    state = registry.get(session_id)
    if not state:
        raise HTTPException(status_code=404, detail="session not found")

    text = req.get("text", "")
    source = req.get("source", "text")

    await write_interrupt(session_id, text, source)
    state.set_phrase("Got it, one sec...", "high")

    return {"status": "acknowledged", "session_id": session_id}


@app.get("/sessions/{session_id}/state")
async def get_session_state(session_id: str):
    state = registry.get(session_id)
    if not state:
        raise HTTPException(status_code=404, detail="session not found")

    return {
        "session_id": state.session_id,
        "persona_id": state.persona_id,
        "active": state.active,
        "last_phrase": state.last_phrase,
        "last_urgency": state.last_urgency,
        "last_phrase_time": state.last_phrase_time,
        "acp_events": list(state.acp_events)[-10:],
    }


async def _run_tailer_and_narrate(state: SessionState, webrtc: WebRTCSession):
    """Tail ACP stream, generate phrases, synthesize TTS, push to WebRTC."""
    narrator = _narrator or Narrator()
    tts = _tts

    if tts and not tts.ready:
        await tts.preload()

    try:
        async for line in tail_acp_stream():
            if not state.active:
                break

            event = parse_line(line)
            if event is None:
                continue

            state.add_acp_event(event)

            # Every N events, generate a narration phrase
            if len(state.acp_events) % settings.narrator_window_size == 0:
                events = list(state.acp_events)[-settings.narrator_window_size:]
                result = await narrator.narrate(events)

                if not result.get("silent") and tts:
                    phrase = result.get("phrase", "")
                    urgency = result.get("urgency", "low")
                    state.set_phrase(phrase, urgency)
                    log.info("Narrating: %s (urgency=%s)", phrase, urgency)

                    # Synthesize and push to WebRTC
                    audio = await tts.synthesize(phrase)
                    await webrtc.push_tts_audio(audio)

    except asyncio.CancelledError:
        log.info("Tail task cancelled for session %s", state.session_id)
    except Exception as e:
        log.error("Tail error: %s", e, exc_info=True)


async def main():
    """Entry point."""
    config = uvicorn.Config(app, host="0.0.0.0", port=settings.port, log_level="info")
    server = uvicorn.Server(config)

    loop = asyncio.get_event_loop()
    loop.add_signal_handler(signal.SIGTERM, lambda: _shutdown.set())
    loop.add_signal_handler(signal.SIGINT, lambda: _shutdown.set())

    await server.serve()


if __name__ == "__main__":
    asyncio.run(main())
