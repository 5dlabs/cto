"""NATS JetStream client for the MuseTalk worker.

See ``avatar/agent/contract.md`` for the authoritative request / response
schema. This module is intentionally dependency-light: ``nats-py`` is imported
lazily so the rest of the agent (and the CPU stub test-suite) can run on
machines where NATS is not installed.
"""

from __future__ import annotations

import asyncio
import json
import logging
import uuid
from dataclasses import dataclass
from typing import Any

logger = logging.getLogger(__name__)


@dataclass(frozen=True)
class RenderRequest:
    request_id: str
    persona_id: str
    reference_image_url: str
    audio_url: str
    fps: int
    callback_subject: str
    audio_hash: str | None = None

    def to_wire(self) -> dict[str, Any]:
        payload: dict[str, Any] = {
            "request_id": self.request_id,
            "persona_id": self.persona_id,
            "reference_image_url": self.reference_image_url,
            "audio_url": self.audio_url,
            "fps": self.fps,
            "callback_subject": self.callback_subject,
        }
        if self.audio_hash is not None:
            payload["audio_hash"] = self.audio_hash
        return payload


@dataclass(frozen=True)
class RenderResult:
    request_id: str
    persona_id: str
    video_url: str | None
    render_time_s: float | None
    cached: bool
    bootstrap_only: bool
    gpu: str | None
    dtype: str | None
    error: str | None

    @classmethod
    def from_wire(cls, payload: dict[str, Any]) -> RenderResult:
        return cls(
            request_id=str(payload.get("request_id", "")),
            persona_id=str(payload.get("persona_id", "")),
            video_url=payload.get("video_url"),
            render_time_s=payload.get("render_time_s"),
            cached=bool(payload.get("cached", False)),
            bootstrap_only=bool(payload.get("bootstrap_only", False)),
            gpu=payload.get("gpu"),
            dtype=payload.get("dtype"),
            error=payload.get("error"),
        )

    @property
    def is_error(self) -> bool:
        return self.error is not None

    @property
    def is_renderable(self) -> bool:
        return (
            self.error is None
            and not self.bootstrap_only
            and bool(self.video_url)
        )


class MuseTalkNatsError(RuntimeError):
    """Raised when the NATS round-trip to the MuseTalk worker fails."""


class MuseTalkNatsClient:
    """Thin JetStream client for the MuseTalk worker.

    The worker consumes requests via a JetStream pull subscription on the
    ``AVATAR`` stream, so we must publish through ``js.publish``. Results come
    back on core NATS via the request's ``callback_subject``; we subscribe
    once at connect time and route by ``request_id``.
    """

    def __init__(
        self,
        url: str,
        *,
        request_subject: str = "avatar.render.request",
        result_subject: str = "avatar.render.result",
        stream: str = "AVATAR",
        request_timeout_s: float = 60.0,
    ) -> None:
        self._url = url
        self._request_subject = request_subject
        self._result_subject = result_subject
        self._stream = stream
        self._request_timeout_s = request_timeout_s
        self._nc: Any | None = None
        self._js: Any | None = None
        self._sub: Any | None = None
        self._pending: dict[str, asyncio.Future[RenderResult]] = {}
        self._connected = False

    @property
    def connected(self) -> bool:
        return self._connected

    async def connect(self) -> None:
        if self._connected:
            return
        try:
            import nats  # type: ignore[import-not-found]
        except ImportError as exc:  # pragma: no cover - exercised only without nats-py
            raise MuseTalkNatsError(
                "nats-py is not installed; install the 'musetalk' extra or set "
                "MUSETALK_USE_STUB=1 to bypass the worker."
            ) from exc

        logger.info("musetalk.nats.connect url=%s stream=%s", self._url, self._stream)
        self._nc = await nats.connect(self._url)
        self._js = self._nc.jetstream()
        self._sub = await self._nc.subscribe(
            self._result_subject, cb=self._on_result
        )
        self._connected = True

    async def close(self) -> None:
        if not self._connected:
            return
        try:
            if self._sub is not None:
                await self._sub.unsubscribe()
        except Exception:  # pragma: no cover - defensive
            logger.exception("musetalk.nats.unsubscribe_failed")
        try:
            if self._nc is not None:
                await self._nc.drain()
        except Exception:  # pragma: no cover - defensive
            logger.exception("musetalk.nats.drain_failed")
        self._connected = False
        self._nc = None
        self._js = None
        self._sub = None

    async def render(
        self,
        *,
        persona_id: str,
        reference_image_url: str,
        audio_url: str,
        fps: int,
        audio_hash: str | None = None,
        timeout_s: float | None = None,
    ) -> RenderResult:
        if not self._connected or self._js is None:
            raise MuseTalkNatsError("NATS client is not connected; call connect() first.")

        request = RenderRequest(
            request_id=str(uuid.uuid4()),
            persona_id=persona_id,
            reference_image_url=reference_image_url,
            audio_url=audio_url,
            fps=fps,
            callback_subject=self._result_subject,
            audio_hash=audio_hash,
        )
        future: asyncio.Future[RenderResult] = asyncio.get_event_loop().create_future()
        self._pending[request.request_id] = future

        payload = json.dumps(request.to_wire()).encode("utf-8")
        logger.info(
            "musetalk.nats.publish request_id=%s persona=%s",
            request.request_id,
            persona_id,
        )
        try:
            await self._js.publish(self._request_subject, payload)
        except Exception as exc:
            self._pending.pop(request.request_id, None)
            raise MuseTalkNatsError(f"failed to publish render request: {exc}") from exc

        try:
            return await asyncio.wait_for(
                future, timeout=timeout_s or self._request_timeout_s
            )
        except TimeoutError as exc:
            self._pending.pop(request.request_id, None)
            raise MuseTalkNatsError(
                f"musetalk render timed out after "
                f"{timeout_s or self._request_timeout_s}s (request_id={request.request_id})"
            ) from exc

    async def _on_result(self, msg: Any) -> None:
        try:
            payload = json.loads(msg.data.decode("utf-8"))
        except (ValueError, AttributeError):
            logger.exception("musetalk.nats.result_decode_failed")
            return

        result = RenderResult.from_wire(payload)
        future = self._pending.pop(result.request_id, None)
        if future is None:
            logger.debug(
                "musetalk.nats.result_unmatched request_id=%s", result.request_id
            )
            return
        if not future.done():
            future.set_result(result)
