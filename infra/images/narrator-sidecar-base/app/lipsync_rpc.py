"""NATS RPC client for lipsync rendering via MuseTalk/Hunyuan workers."""

from __future__ import annotations

import asyncio
import json
import logging
import uuid
from typing import Any

import nats

from app.config import settings

logger = logging.getLogger(__name__)


class LipsyncRPC:
    """Publish lipsync render requests to NATS and await results."""

    def __init__(self, nats_url: str | None = None):
        self._nats_url = nats_url or "nats://nats.messaging.svc.cluster.local:4222"
        self._nc: nats.NATS | None = None
        self._pending: dict[str, asyncio.Future[dict[str, Any]]] = {}
        self._backend = settings.backend  # musetalk or hunyuan
        self._subject = "avatar.render.request"
        self._result_subject = "avatar.render.result"

    async def connect(self):
        """Connect to NATS and subscribe to result subject."""
        self._nc = await nats.connect(self._nats_url)
        await self._nc.subscribe(self._result_subject, cb=self._handle_result)
        logger.info("LipsyncRPC connected to %s (backend=%s)", self._nats_url, self._backend)

    async def _handle_result(self, msg):
        """Handle incoming render result."""
        try:
            data = json.loads(msg.data.decode())
            request_id = data.get("request_id")
            if request_id and request_id in self._pending:
                future = self._pending.pop(request_id)
                if not future.done():
                    future.set_result(data)
        except Exception as e:
            logger.error("Error handling result: %s", e)

    async def render(
        self,
        reference_image_url: str,
        audio_url: str,
        persona_id: str = "blaze",
        fps: int = 25,
        timeout: float = 300.0,
    ) -> dict[str, Any]:
        """
        Publish a lipsync render request and await the result.

        Args:
            reference_image_url: URL to reference image (person's face)
            audio_url: URL to audio file (TTS output)
            persona_id: Persona identifier
            fps: Target FPS for output video
            timeout: Seconds to wait for result

        Returns:
            Dict with video_url, render_time_s, error, etc.
        """
        if not self._nc:
            await self.connect()

        request_id = str(uuid.uuid4())
        future: asyncio.Future[dict[str, Any]] = asyncio.get_event_loop().create_future()
        self._pending[request_id] = future

        payload = {
            "request_id": request_id,
            "persona_id": persona_id,
            "reference_image_url": reference_image_url,
            "audio_url": audio_url,
            "fps": fps,
            "callback_subject": self._result_subject,
        }

        await self._nc.publish(self._subject, json.dumps(payload).encode())
        logger.info("Published lipsync request %s (backend=%s)", request_id, self._backend)

        try:
            result = await asyncio.wait_for(future, timeout=timeout)
            logger.info("Got lipsync result for %s: %s", request_id, result.get("error", "ok"))
            return result
        except asyncio.TimeoutError:
            self._pending.pop(request_id, None)
            logger.error("Lipsync request %s timed out after %.1fs", request_id, timeout)
            return {"request_id": request_id, "error": "timeout", "video_url": None}
        except Exception as e:
            self._pending.pop(request_id, None)
            logger.error("Lipsync request %s failed: %s", request_id, e)
            return {"request_id": request_id, "error": str(e), "video_url": None}

    async def close(self):
        """Close NATS connection."""
        if self._nc:
            await self._nc.close()
            logger.info("LipsyncRPC closed")


# Module-level singleton
_lipsync: LipsyncRPC | None = None


async def get_lipsync_rpc() -> LipsyncRPC:
    """Get or create the LipsyncRPC singleton."""
    global _lipsync
    if _lipsync is None:
        _lipsync = LipsyncRPC()
        await _lipsync.connect()
    return _lipsync


async def render_lipsync(
    reference_image_url: str,
    audio_url: str,
    persona_id: str = "blaze",
    fps: int = 25,
    timeout: float = 300.0,
) -> dict[str, Any]:
    """Convenience function to render lipsync via NATS."""
    rpc = await get_lipsync_rpc()
    return await rpc.render(reference_image_url, audio_url, persona_id, fps, timeout)
