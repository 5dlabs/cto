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
        started = websocket.receive_json()
        assert started == {"type": "started", "session_id": "s1", "agent": "hermes"}

        websocket.send_text('{"type":"text","text":"typed words"}')
        websocket.send_bytes(b"audio")
        websocket.send_text('{"type":"end_utterance"}')

        transcript = websocket.receive_json()
        assert transcript == {
            "type": "transcript",
            "text": "spoken words\ntyped words",
            "agent": "hermes",
        }
        delta = websocket.receive_json()
        assert delta == {"type": "reply_delta", "text": "reply-from-hermes", "agent": "hermes"}
        reply = websocket.receive_json()
        assert reply == {"type": "reply_text", "text": "reply-from-hermes", "agent": "hermes"}
        audio = websocket.receive_bytes()
        assert audio == b"audio-for-voice-hermes"
        done = websocket.receive_json()
        assert done == {"type": "turn_done", "agent": "hermes"}


def test_shared_secret_auth_rejects(monkeypatch):
    app = load_app(monkeypatch, shared_secret="topsecret")
    client = TestClient(app)

    with client.websocket_connect("/ws") as websocket:
        msg = websocket.receive_json()
        assert msg == {"type": "error", "error": "unauthorized"}
        close = websocket.receive()
        assert close["type"] == "websocket.close"
        assert close["code"] == 4401


def test_rate_limit_rejects_excess_turns(monkeypatch):
    app = load_app(monkeypatch)
    client = TestClient(app)

    with client.websocket_connect("/ws") as websocket:
        websocket.send_text('{"type":"start","session_id":"s1"}')
        websocket.receive_json()

        websocket.send_text('{"type":"text","text":"one"}')
        websocket.send_text('{"type":"end_utterance"}')
        websocket.receive_json()
        websocket.receive_json()
        websocket.receive_json()
        websocket.receive_bytes()
        websocket.receive_json()

        websocket.send_text('{"type":"text","text":"two"}')
        websocket.send_text('{"type":"end_utterance"}')
        websocket.receive_json()
        websocket.receive_json()
        websocket.receive_json()
        websocket.receive_bytes()
        websocket.receive_json()

        websocket.send_text('{"type":"text","text":"three"}')
        websocket.send_text('{"type":"end_utterance"}')
        assert websocket.receive_json() == {"type": "error", "error": "rate_limited"}
