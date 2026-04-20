"""FastAPI application for narrator sidecar."""

from __future__ import annotations

import asyncio
import logging
import os
import signal
import time

import uvicorn
from aiortc import RTCPeerConnection, RTCSessionDescription
from fastapi import FastAPI, HTTPException
from fastapi.responses import JSONResponse

from app.config import Settings
from app.interrupt import write_interrupt
from app.narrator import generate_phrase
from app.session import SessionRegistry, SessionState
from app.tailer import tail_acp_stream
from app.tts import synthesize_speech
from app.webrtc import create_audio_track, create_video_track_placeholder

logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s [%(name)s] %(message)s")
log = logging.getLogger("main")

settings = Settings()
registry = SessionRegistry()
app = FastAPI(title="Narrator Sidecar", version="0.1.0")

_shutdown = asyncio.Event()


@app.on_event("startup")
async def startup():
    log.info("Narrator sidecar starting (backend=%s, port=%d)", settings.BACKEND, settings.PORT)


@app.on_event("shutdown")
async def shutdown():
    await registry.cleanup_all()
    log.info("Narrator sidecar shut down")


@app.get("/healthz")
async def healthz():
    return {"status": "ok"}


@app.get("/readyz")
async def readyz():
    return {"status": "ready"}


@app.get("/info")
async def info():
    return {
        "backend": settings.BACKEND,
        "version": "0.1.0",
        "capabilities": ["audio", "video", "interrupt"],
    }


@app.post("/sessions")
async def create_session(req: dict) -> dict:
    session_id = req.get("session_id", f"session-{int(time.time())}")
    persona_id = req.get("persona_id", "rex")
    webrtc_offer = req.get("webrtc_offer")

    if not webrtc_offer:
        raise HTTPException(status_code=400, detail="webrtc_offer required")

    state = registry.create(session_id, persona_id)

    # Set up WebRTC
    pc = RTCPeerConnection()
    state.pc = pc

    @pc.on("iceconnectionstatechange")
    async def on_ice_state():
        log.info("ICE state: %s (session=%s)", pc.iceConnectionState, session_id)
        if pc.iceConnectionState in ("failed", "closed"):
            await registry.delete(session_id)

    # Create media tracks
    audio_track = create_audio_track()
    video_track = create_video_track_placeholder()
    pc.addTrack(audio_track)
    pc.addTrack(video_track)

    # Handle SDP
    offer = RTCSessionDescription(sdp=webrtc_offer["sdp"], type=webrtc_offer["type"])
    await pc.setRemoteDescription(offer)
    answer = await pc.createAnswer()
    await pc.setLocalDescription(answer)

    # Start tailing ACP stream
    state.tailer_task = asyncio.create_task(_run_tailer(state))

    webrtc_answer = {"sdp": pc.localDescription.sdp, "type": pc.localDescription.type}
    return {"session_id": session_id, "webrtc_answer": webrtc_answer}


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


async def _run_tailer(state: SessionState):
    """Tail ACP stream, generate phrases, synthesize TTS, push to WebRTC."""
    stream_path = os.environ.get("OPENCLAW_RAW_STREAM_PATH", "/workspace/.openclaw/acp-stream.ndjson")

    try:
        async for line in tail_acp_stream(stream_path):
            if not state.active:
                break
            state.add_acp_event({"raw": line.strip(), "ts": time.time()})
    except Exception as e:
        log.error("Tail error: %s", e)


async def main():
    """Entry point."""
    config = uvicorn.Config(app, host="0.0.0.0", port=settings.PORT, log_level="info")
    server = uvicorn.Server(config)

    loop = asyncio.get_event_loop()
    loop.add_signal_handler(signal.SIGTERM, lambda: _shutdown.set())
    loop.add_signal_handler(signal.SIGINT, lambda: _shutdown.set())

    await server.serve()


if __name__ == "__main__":
    asyncio.run(main())
