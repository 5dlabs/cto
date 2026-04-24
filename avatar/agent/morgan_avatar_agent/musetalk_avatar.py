from __future__ import annotations

import asyncio
import logging
from collections.abc import Iterable
from dataclasses import dataclass
from typing import Any

from .musetalk_inference import AudioChunk, MuseTalkInferenceEngine, RenderedFrame

logger = logging.getLogger(__name__)


@dataclass(frozen=True)
class PublishedFrame:
    index: int
    timestamp_ms: float
    width: int
    height: int
    payload_size: int


class MuseTalkAvatarSession:
    """LiveKit publish pipeline for the self-hosted MuseTalk avatar mode.

    When ``livekit.rtc`` is available at runtime the session creates a
    ``VideoSource`` / ``LocalVideoTrack`` pair and publishes it to the room,
    driving the source from the inference engine. In CI / offline environments
    where ``livekit.rtc`` is not importable we keep the previous test-friendly
    no-op behaviour and just record the room name for observability.
    """

    def __init__(self, inference: MuseTalkInferenceEngine) -> None:
        self._inference = inference
        self._source: Any | None = None
        self._track: Any | None = None
        self._pump_task: asyncio.Task[None] | None = None
        self._room: Any | None = None
        self._stop_event: asyncio.Event | None = None
        self._external_video_active = False

    def warmup(self) -> None:
        self._inference.warmup()

    def publishable_frames(self, audio_chunks: Iterable[AudioChunk]) -> Iterable[PublishedFrame]:
        for frame in self._inference.stream_frames(audio_chunks):
            yield self._to_published_frame(frame)

    async def start(self, session: Any, room: Any) -> None:
        self.warmup()
        self._room = room
        if hasattr(room, "name"):
            session._musetalk_room = room.name

        try:
            from livekit import rtc  # type: ignore[import-not-found]
        except ImportError:
            logger.info("musetalk.avatar.start livekit.rtc unavailable; skipping RTC publish")
            return

        try:
            self._source = rtc.VideoSource(
                self._inference.frame_width, self._inference.frame_height
            )
            self._track = rtc.LocalVideoTrack.create_video_track("avatar", self._source)
            options = rtc.TrackPublishOptions(source=rtc.TrackSource.SOURCE_CAMERA)
            await room.local_participant.publish_track(self._track, options)
            self._stop_event = asyncio.Event()
            self._pump_task = asyncio.create_task(self._idle_pump(rtc))
            logger.info(
                "musetalk.avatar.published width=%d height=%d fps=%d",
                self._inference.frame_width,
                self._inference.frame_height,
                self._inference.target_fps,
            )
        except Exception:
            logger.exception("musetalk.avatar.start_failed")

    async def stop(self) -> None:
        if self._stop_event is not None:
            self._stop_event.set()
        if self._pump_task is not None:
            self._pump_task.cancel()
            try:
                await self._pump_task
            except (asyncio.CancelledError, Exception):  # pragma: no cover - defensive
                pass
            self._pump_task = None
        if self._track is not None and self._room is not None:
            try:
                await self._room.local_participant.unpublish_track(self._track.sid)
            except Exception:  # pragma: no cover - defensive
                logger.exception("musetalk.avatar.unpublish_failed")
        self._track = None
        self._source = None
        self._stop_event = None

    async def render_utterance(
        self,
        audio_url: str,
        *,
        audio_hash: str | None = None,
        duration_ms: float | None = None,
    ) -> int:
        """Render one utterance via the inference engine and push to the source.

        Returns the number of frames actually pushed. Safe to call even when the
        RTC source is not wired (no-op).
        """
        frames_pushed = 0
        async for frame in self._inference.render_and_stream(
            audio_url, audio_hash=audio_hash, duration_ms=duration_ms
        ):
            if self._source is None:
                frames_pushed += 1
                continue
            self._push_frame(frame)
            frames_pushed += 1
        return frames_pushed

    def set_external_video_active(self, active: bool) -> None:
        self._external_video_active = active

    def push_rgba_frame(
        self,
        *,
        width: int,
        height: int,
        rgba: bytes,
        index: int = 0,
        timestamp_ms: float = 0.0,
    ) -> bool:
        if self._source is None:
            return False
        self._push_frame(
            RenderedFrame(
                index=index,
                width=width,
                height=height,
                rgba=rgba,
                timestamp_ms=timestamp_ms,
            )
        )
        return True

    async def _idle_pump(self, rtc: Any) -> None:
        assert self._stop_event is not None
        frame_interval = 1.0 / max(self._inference.target_fps, 1)
        idle_chunk = AudioChunk(samples=[], sample_rate=16000, duration_ms=frame_interval * 1000)
        frame_index = 0
        while not self._stop_event.is_set():
            if self._external_video_active:
                await asyncio.sleep(frame_interval)
                continue
            for frame in self._inference.stream_frames([idle_chunk]):
                if self._stop_event.is_set():
                    return
                if self._source is not None:
                    try:
                        self._push_frame(frame)
                    except Exception:  # pragma: no cover - defensive
                        logger.exception("musetalk.avatar.idle_push_failed")
                frame_index += 1
                await asyncio.sleep(frame_interval)

    def _push_frame(self, frame: RenderedFrame) -> None:
        try:
            from livekit import rtc  # type: ignore[import-not-found]
        except ImportError:  # pragma: no cover - start() guards this path
            return
        video_frame = rtc.VideoFrame(
            frame.width,
            frame.height,
            rtc.VideoBufferType.RGBA,
            frame.rgba,
        )
        assert self._source is not None
        self._source.capture_frame(video_frame)

    @staticmethod
    def _to_published_frame(frame: RenderedFrame) -> PublishedFrame:
        return PublishedFrame(
            index=frame.index,
            timestamp_ms=frame.timestamp_ms,
            width=frame.width,
            height=frame.height,
            payload_size=len(frame.rgba),
        )
