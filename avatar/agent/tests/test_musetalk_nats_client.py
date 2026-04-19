from __future__ import annotations

import asyncio
import json
import sys
import types
from typing import Any

import pytest

from morgan_avatar_agent.musetalk_nats_client import (
    MuseTalkNatsClient,
    MuseTalkNatsError,
    RenderResult,
)


class _FakeMsg:
    def __init__(self, data: bytes) -> None:
        self.data = data


class _FakeJetStream:
    def __init__(self) -> None:
        self.published: list[tuple[str, bytes]] = []

    async def publish(self, subject: str, payload: bytes) -> None:
        self.published.append((subject, payload))


class _FakeSub:
    def __init__(self) -> None:
        self.unsubscribed = False

    async def unsubscribe(self) -> None:
        self.unsubscribed = True


class _FakeNC:
    def __init__(self) -> None:
        self._js = _FakeJetStream()
        self._cb: Any | None = None
        self._sub = _FakeSub()
        self.drained = False

    def jetstream(self) -> _FakeJetStream:
        return self._js

    async def subscribe(self, subject: str, cb: Any) -> _FakeSub:
        self._cb = cb
        return self._sub

    async def drain(self) -> None:
        self.drained = True

    async def deliver(self, payload: dict[str, Any]) -> None:
        assert self._cb is not None
        await self._cb(_FakeMsg(json.dumps(payload).encode("utf-8")))


def _install_fake_nats(monkeypatch: pytest.MonkeyPatch) -> _FakeNC:
    fake_nc = _FakeNC()

    async def _connect(_url: str) -> _FakeNC:
        return fake_nc

    module = types.ModuleType("nats")
    module.connect = _connect  # type: ignore[attr-defined]
    monkeypatch.setitem(sys.modules, "nats", module)
    return fake_nc


def test_render_publishes_request_and_resolves_result(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    fake_nc = _install_fake_nats(monkeypatch)

    async def scenario() -> RenderResult:
        client = MuseTalkNatsClient("nats://fake:4222", request_timeout_s=2.0)
        await client.connect()
        task = asyncio.create_task(
            client.render(
                persona_id="morgan",
                reference_image_url="https://example/ref.png",
                audio_url="https://example/a.wav",
                fps=25,
                audio_hash="sha256-deadbeef",
            )
        )
        await asyncio.sleep(0)
        assert fake_nc._js.published, "request should have been published"
        subject, payload = fake_nc._js.published[0]
        assert subject == "avatar.render.request"
        decoded = json.loads(payload.decode("utf-8"))
        assert decoded["persona_id"] == "morgan"
        assert decoded["reference_image_url"] == "https://example/ref.png"
        assert decoded["audio_url"] == "https://example/a.wav"
        assert decoded["fps"] == 25
        assert decoded["callback_subject"] == "avatar.render.result"
        assert decoded["audio_hash"] == "sha256-deadbeef"
        request_id = decoded["request_id"]
        assert request_id

        await fake_nc.deliver(
            {
                "request_id": request_id,
                "persona_id": "morgan",
                "video_url": "file:///x.mp4",
                "render_time_s": 1.5,
                "cached": False,
                "bootstrap_only": False,
                "gpu": "H100",
                "dtype": "fp16",
                "error": None,
            }
        )
        result = await asyncio.wait_for(task, timeout=1.0)
        await client.close()
        assert fake_nc.drained
        return result

    result = asyncio.run(scenario())
    assert result.is_renderable
    assert result.video_url == "file:///x.mp4"


def test_unmatched_request_id_is_ignored(monkeypatch: pytest.MonkeyPatch) -> None:
    fake_nc = _install_fake_nats(monkeypatch)

    async def scenario() -> None:
        client = MuseTalkNatsClient("nats://fake:4222", request_timeout_s=0.3)
        await client.connect()
        task = asyncio.create_task(
            client.render(
                persona_id="morgan",
                reference_image_url="ref",
                audio_url="a",
                fps=25,
            )
        )
        await asyncio.sleep(0)
        await fake_nc.deliver(
            {
                "request_id": "someone-else",
                "persona_id": "morgan",
                "video_url": "file:///nope.mp4",
                "cached": False,
                "bootstrap_only": False,
                "error": None,
            }
        )
        with pytest.raises(MuseTalkNatsError):
            await task

    asyncio.run(scenario())


def test_bootstrap_only_result_is_not_renderable(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    fake_nc = _install_fake_nats(monkeypatch)

    async def scenario() -> RenderResult:
        client = MuseTalkNatsClient("nats://fake:4222", request_timeout_s=2.0)
        await client.connect()
        task = asyncio.create_task(
            client.render(
                persona_id="morgan",
                reference_image_url="ref",
                audio_url="a",
                fps=25,
            )
        )
        await asyncio.sleep(0)
        request_id = json.loads(fake_nc._js.published[0][1].decode("utf-8"))["request_id"]
        await fake_nc.deliver(
            {
                "request_id": request_id,
                "persona_id": "morgan",
                "video_url": None,
                "cached": False,
                "bootstrap_only": True,
                "error": None,
            }
        )
        return await asyncio.wait_for(task, timeout=1.0)

    result = asyncio.run(scenario())
    assert not result.is_renderable
    assert result.bootstrap_only


def test_render_without_connect_raises() -> None:
    async def scenario() -> None:
        client = MuseTalkNatsClient("nats://fake:4222")
        with pytest.raises(MuseTalkNatsError):
            await client.render(
                persona_id="morgan",
                reference_image_url="ref",
                audio_url="a",
                fps=25,
            )

    asyncio.run(scenario())


def test_connect_without_nats_py_raises(monkeypatch: pytest.MonkeyPatch) -> None:
    # Force `import nats` to fail.
    monkeypatch.setitem(sys.modules, "nats", None)  # type: ignore[arg-type]

    async def scenario() -> None:
        client = MuseTalkNatsClient("nats://fake:4222")
        with pytest.raises(MuseTalkNatsError):
            await client.connect()

    asyncio.run(scenario())
