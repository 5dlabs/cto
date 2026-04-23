from __future__ import annotations

import importlib
import json
import sys
from collections.abc import AsyncIterator
from pathlib import Path

from fastapi.testclient import TestClient

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))


class StubAgentClient:
    def __init__(self, agent, request_timeout_s: float = 120.0) -> None:
        self.agent = agent

    async def send_and_stream(self, *, session_id: str, text: str) -> AsyncIterator[str]:
        yield f"reply-from-{self.agent.name}"


class StubTTSClient:
    def __init__(self, api_key: str, voice_id: str) -> None:
        self.api_key = api_key
        self.voice_id = voice_id

    @property
    def is_configured(self) -> bool:
        return True

    async def transcribe(self, audio_bytes: bytes, **_: object) -> str:
        return "spoken words" if audio_bytes else ""

    async def stream_tts(self, text: str) -> AsyncIterator[bytes]:
        if text:
            yield f"audio-for-{self.voice_id}".encode()

    def with_voice(self, voice_id: str) -> "StubTTSClient":
        return StubTTSClient(api_key=self.api_key, voice_id=voice_id)


def _recv_non_session(ws):
    """Drain SESSION_STATE frames and return the next non-session JSON frame."""
    while True:
        frame = ws.receive_json()
        if isinstance(frame, dict) and frame.get("type") == "SESSION_STATE":
            continue
        return frame


def load_app(monkeypatch, voice_agents_json: str | None = None, shared_secret: str = ""):
    monkeypatch.setenv("ELEVENLABS_API_KEY", "test-key")
    monkeypatch.setenv("VOICE_BRIDGE_MAX_TURNS", "2")
    monkeypatch.setenv("VOICE_BRIDGE_RATE_WINDOW_S", "60")
    if shared_secret:
        monkeypatch.setenv("VOICE_BRIDGE_SHARED_SECRET", shared_secret)
    else:
        monkeypatch.delenv("VOICE_BRIDGE_SHARED_SECRET", raising=False)
    if voice_agents_json is not None:
        monkeypatch.setenv("VOICE_AGENTS_JSON", voice_agents_json)
    else:
        monkeypatch.delenv("VOICE_AGENTS_JSON", raising=False)

    import app.main as main

    importlib.reload(main)
    monkeypatch.setattr(main, "MorganAgentClient", StubAgentClient)
    monkeypatch.setattr(main, "_BASE_TTS", StubTTSClient(api_key="test-key", voice_id="base-voice"))
    return main.app


def test_unknown_agent_closes_4404(monkeypatch):
    app = load_app(monkeypatch)
    client = TestClient(app)

    with client.websocket_connect("/ws?agent=unknown") as websocket:
        message = websocket.receive()
        assert message["type"] == "websocket.close"
        assert message["code"] == 4404
        assert message["reason"] == "unknown_agent"


def test_agent_routing_and_tts_voice(monkeypatch):
    voice_agents_json = json.dumps(
        {
            "morgan": {
                "model": "openclaw/morgan",
                "voice_id": "voice-morgan",
                "gateway_url": "http://morgan",
                "gateway_token": "token-morgan",
            },
            "hermes": {
                "model": "openclaw/hermes",
                "voice_id": "voice-hermes",
                "gateway_url": "http://hermes",
                "gateway_token": "token-hermes",
            },
        }
    )
    app = load_app(monkeypatch, voice_agents_json=voice_agents_json)
    client = TestClient(app)

    with client.websocket_connect("/ws?agent=hermes") as websocket:
        websocket.send_text('{"type":"start","session_id":"s1"}')
        started = _recv_non_session(websocket)
        assert started == {"type": "started", "session_id": "s1", "agent": "hermes"}

        websocket.send_text('{"type":"text","text":"typed words"}')
        websocket.send_bytes(b"audio")
        websocket.send_text('{"type":"end_utterance"}')

        transcript = _recv_non_session(websocket)
        assert transcript == {
            "type": "transcript",
            "text": "spoken words\ntyped words",
            "agent": "hermes",
        }
        delta = _recv_non_session(websocket)
        assert delta == {"type": "reply_delta", "text": "reply-from-hermes", "agent": "hermes"}
        reply = _recv_non_session(websocket)
        assert reply == {"type": "reply_text", "text": "reply-from-hermes", "agent": "hermes"}
        # drain `speaking` SESSION_STATE emitted just before TTS audio
        speaking = websocket.receive_json()
        assert speaking.get("type") == "SESSION_STATE"
        audio = websocket.receive_bytes()
        assert audio == b"audio-for-voice-hermes"
        done = _recv_non_session(websocket)
        assert done == {"type": "turn_done", "agent": "hermes"}


def test_shared_secret_auth_rejects(monkeypatch):
    app = load_app(monkeypatch, shared_secret="topsecret")
    client = TestClient(app)

    with client.websocket_connect("/ws") as websocket:
        msg = websocket.receive_json()
        assert msg == {"type": "error", "error": "unauthorized"}
        # New: structured ERROR frame emitted alongside legacy frame.
        structured = websocket.receive_json()
        assert structured["protocol"] == "cto-avatar-session/v1"
        assert structured["type"] == "ERROR"
        assert structured["code"] == "AUTH_FAILED"
        assert structured["recoverable"] is False
        assert structured["session_id"] == "unknown"
        close = websocket.receive()
        assert close["type"] == "websocket.close"
        assert close["code"] == 4401


def test_rate_limit_rejects_excess_turns(monkeypatch):
    app = load_app(monkeypatch)
    client = TestClient(app)

    with client.websocket_connect("/ws") as websocket:
        websocket.send_text('{"type":"start","session_id":"s1"}')
        _recv_non_session(websocket)

        websocket.send_text('{"type":"text","text":"one"}')
        websocket.send_text('{"type":"end_utterance"}')
        _recv_non_session(websocket)  # transcript
        _recv_non_session(websocket)  # reply_delta
        _recv_non_session(websocket)  # reply_text
        websocket.receive_json()  # speaking SESSION_STATE
        websocket.receive_bytes()
        _recv_non_session(websocket)  # turn_done

        websocket.send_text('{"type":"text","text":"two"}')
        websocket.send_text('{"type":"end_utterance"}')
        _recv_non_session(websocket)
        _recv_non_session(websocket)
        _recv_non_session(websocket)
        websocket.receive_json()  # speaking SESSION_STATE
        websocket.receive_bytes()
        _recv_non_session(websocket)

        websocket.send_text('{"type":"text","text":"three"}')
        websocket.send_text('{"type":"end_utterance"}')
        assert _recv_non_session(websocket) == {"type": "error", "error": "rate_limited"}


def test_session_state_frames_track_turn_lifecycle(monkeypatch):
    app = load_app(monkeypatch)
    client = TestClient(app)

    with client.websocket_connect("/ws") as websocket:
        def next_session_state():
            while True:
                frame = websocket.receive_json()
                if isinstance(frame, dict) and frame.get("type") == "SESSION_STATE":
                    assert frame["protocol"] == "cto-avatar-session/v1"
                    assert frame["session_id"] == "s1"
                    assert frame["agent_name"] == "morgan"
                    assert isinstance(frame["timestamp_ms"], int)
                    return frame["state"]

        websocket.send_text('{"type":"start","session_id":"s1"}')
        assert next_session_state() == "connecting"
        # started reply, then connected
        started = websocket.receive_json()
        assert started["type"] == "started"
        assert next_session_state() == "connected"

        websocket.send_text('{"type":"text","text":"hello"}')
        websocket.send_text('{"type":"end_utterance"}')
        # transcript, then listening
        transcript = websocket.receive_json()
        assert transcript["type"] == "transcript"
        assert next_session_state() == "listening"
        # reply_delta, reply_text, then speaking
        assert websocket.receive_json()["type"] == "reply_delta"
        assert websocket.receive_json()["type"] == "reply_text"
        assert next_session_state() == "speaking"
        # tts audio, turn_done, then connected
        websocket.receive_bytes()
        assert websocket.receive_json()["type"] == "turn_done"
        assert next_session_state() == "connected"

        websocket.send_text('{"type":"stop"}')
        assert next_session_state() == "disconnecting"
        assert next_session_state() == "idle"


# ---- Structured ERROR frame (session_errors) --------------------------------

def test_build_error_frame_shape_per_code():
    from app.session_errors import (  # type: ignore
        AUTH_FAILED,
        NETWORK_DISCONNECT,
        SESSION_TIMEOUT,
        STT_FAILED,
        TTS_FAILED,
        build_error_frame,
    )

    for code, expected_recoverable in (
        (NETWORK_DISCONNECT, True),
        (STT_FAILED, True),
        (TTS_FAILED, True),
        (AUTH_FAILED, False),
        (SESSION_TIMEOUT, False),
    ):
        frame = build_error_frame(
            session_id="sess-1",
            code=code,
            message="boom",
            timestamp_ms=123,
        )
        assert frame == {
            "protocol": "cto-avatar-session/v1",
            "type": "ERROR",
            "session_id": "sess-1",
            "code": code,
            "message": "boom",
            "recoverable": expected_recoverable,
            "timestamp_ms": 123,
        }


def test_build_error_frame_defaults_session_id_to_unknown_when_none():
    from app.session_errors import (  # type: ignore
        AUTH_FAILED,
        build_error_frame,
    )

    frame = build_error_frame(
        session_id=None,
        code=AUTH_FAILED,
        message="nope",
        timestamp_ms=1,
    )
    assert frame["session_id"] == "unknown"


def test_build_error_frame_explicit_recoverable_override():
    from app.session_errors import (  # type: ignore
        TTS_FAILED,
        build_error_frame,
    )

    frame = build_error_frame(
        session_id="s",
        code=TTS_FAILED,
        message="m",
        recoverable=False,
        timestamp_ms=1,
    )
    assert frame["recoverable"] is False
