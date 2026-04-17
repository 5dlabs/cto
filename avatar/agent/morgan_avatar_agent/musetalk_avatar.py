from __future__ import annotations

from collections.abc import Iterable
from dataclasses import dataclass
from typing import Any

from .musetalk_inference import AudioChunk, MuseTalkInferenceEngine, RenderedFrame


@dataclass(frozen=True)
class PublishedFrame:
    index: int
    timestamp_ms: float
    width: int
    height: int
    payload_size: int


class MuseTalkAvatarSession:
    """Minimal publish pipeline for the self-hosted MuseTalk avatar mode.

    This class intentionally keeps the Phase 3 contract simple: generate frame
    payloads from audio chunks and expose them through a publishable stream.
    The LiveKit RTC track wiring can wrap these frames directly once the runtime
    dependency is present in the agent image.
    """

    def __init__(self, inference: MuseTalkInferenceEngine) -> None:
        self._inference = inference

    def warmup(self) -> None:
        self._inference.warmup()

    def publishable_frames(self, audio_chunks: Iterable[AudioChunk]) -> Iterable[PublishedFrame]:
        for frame in self._inference.stream_frames(audio_chunks):
            yield self._to_published_frame(frame)

    async def start(self, session: Any, room: Any) -> None:
        # Phase 3 wires the mode selection and frame generator. The actual RTC
        # publication is added by the runtime when livekit.rtc is available in
        # the deployed image.
        self.warmup()
        if hasattr(room, "name"):
            setattr(session, "_musetalk_room", room.name)

    @staticmethod
    def _to_published_frame(frame: RenderedFrame) -> PublishedFrame:
        return PublishedFrame(
            index=frame.index,
            timestamp_ms=frame.timestamp_ms,
            width=frame.width,
            height=frame.height,
            payload_size=len(frame.rgba),
        )
