from __future__ import annotations

import logging
import math
import time
from collections.abc import AsyncIterator, Iterable
from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from .musetalk_nats_client import MuseTalkNatsClient

logger = logging.getLogger(__name__)


@dataclass(frozen=True)
class AudioChunk:
    samples: list[float]
    sample_rate: int
    duration_ms: float


@dataclass(frozen=True)
class RenderedFrame:
    index: int
    width: int
    height: int
    rgba: bytes
    timestamp_ms: float


@dataclass(frozen=True)
class InferenceStats:
    frames_rendered: int
    elapsed_s: float

    @property
    def fps(self) -> float:
        if self.elapsed_s <= 0:
            return 0.0
        return self.frames_rendered / self.elapsed_s


class MuseTalkInferenceEngine:
    """CPU-testable streaming stub for Phase 3.

    The production GPU path will wrap MuseTalk realtime inference. For now we
    keep the contract explicit and deterministic so the avatar pipeline can be
    wired and regression-tested without CUDA.
    """

    def __init__(
        self,
        persona_id: str,
        personas_root: Path,
        target_fps: int = 30,
        frame_width: int = 512,
        frame_height: int = 512,
        *,
        nats_client: MuseTalkNatsClient | None = None,
        reference_image_url: str = "",
        use_stub: bool = True,
    ) -> None:
        self.persona_id = persona_id
        self.personas_root = personas_root
        self.target_fps = target_fps
        self.frame_width = frame_width
        self.frame_height = frame_height
        self.nats_client = nats_client
        self.reference_image_url = reference_image_url
        self.use_stub = use_stub
        self._persona_path = personas_root / persona_id
        self._latents_path = self._persona_path / "latents"

    @property
    def persona_ready(self) -> bool:
        return self._latents_path.exists()

    def warmup(self) -> None:
        self._persona_path.mkdir(parents=True, exist_ok=True)
        self._latents_path.mkdir(parents=True, exist_ok=True)

    def stream_frames(self, audio_chunks: Iterable[AudioChunk]) -> Iterable[RenderedFrame]:
        frame_index = 0
        frame_duration_ms = 1000 / self.target_fps
        for chunk in audio_chunks:
            expected_frames = max(1, round((chunk.duration_ms / 1000) * self.target_fps))
            for local_index in range(expected_frames):
                rgba = self._render_rgba(frame_index, local_index, expected_frames)
                yield RenderedFrame(
                    index=frame_index,
                    width=self.frame_width,
                    height=self.frame_height,
                    rgba=rgba,
                    timestamp_ms=frame_index * frame_duration_ms,
                )
                frame_index += 1

    def benchmark(self, audio_chunks: Iterable[AudioChunk]) -> InferenceStats:
        start = time.perf_counter()
        frames = list(self.stream_frames(audio_chunks))
        elapsed = time.perf_counter() - start
        return InferenceStats(frames_rendered=len(frames), elapsed_s=elapsed)

    async def render_and_stream(
        self,
        audio_url: str,
        *,
        audio_hash: str | None = None,
        duration_ms: float | None = None,
    ) -> AsyncIterator[RenderedFrame]:
        """Render a full utterance via the MuseTalk worker and stream frames.

        When ``use_stub`` is True or no NATS client is configured, this falls
        back to the deterministic procedural generator so CI/offline paths work.
        The real GPU path publishes a batch request to the worker and returns
        a single MP4 ``video_url``; decoding the MP4 back into ``RenderedFrame``s
        is a follow-up (see ``avatar/agent/contract.md``). For now we log the
        result and fall back to procedural frames so the LiveKit track keeps
        moving end-to-end.
        """
        if self.use_stub or self.nats_client is None or not audio_url:
            for frame in self._stub_frames_for_duration(duration_ms or 1000.0):
                yield frame
            return

        try:
            result = await self.nats_client.render(
                persona_id=self.persona_id,
                reference_image_url=self.reference_image_url,
                audio_url=audio_url,
                fps=self.target_fps,
                audio_hash=audio_hash,
            )
        except Exception:
            logger.exception("musetalk.render.failed persona=%s", self.persona_id)
            for frame in self._stub_frames_for_duration(duration_ms or 1000.0):
                yield frame
            return

        if result.is_error:
            logger.warning(
                "musetalk.render.error persona=%s error=%s", self.persona_id, result.error
            )
            return
        if result.bootstrap_only:
            logger.info("musetalk.render.bootstrap_only persona=%s", self.persona_id)
            return
        if not result.is_renderable:
            logger.warning(
                "musetalk.render.unrenderable persona=%s video_url=%s",
                self.persona_id,
                result.video_url,
            )
            return

        logger.info(
            "musetalk.render.ok persona=%s video_url=%s cached=%s render_time_s=%s",
            self.persona_id,
            result.video_url,
            result.cached,
            result.render_time_s,
        )
        # TODO(musetalk): decode result.video_url into RenderedFrame tiles and
        # yield at target_fps. For now yield stub frames so the published track
        # stays live.
        for frame in self._stub_frames_for_duration(duration_ms or 1000.0):
            yield frame

    def _stub_frames_for_duration(self, duration_ms: float) -> Iterable[RenderedFrame]:
        chunk = AudioChunk(samples=[], sample_rate=16000, duration_ms=duration_ms)
        return self.stream_frames([chunk])

    def _render_rgba(self, frame_index: int, local_index: int, total_frames: int) -> bytes:
        mouth = int(80 + 120 * math.sin((local_index / max(total_frames, 1)) * math.pi))
        r = (frame_index * 13) % 255
        g = mouth % 255
        b = (255 - r) % 255
        pixel = bytes((r, g, b, 255))
        return pixel * (self.frame_width * self.frame_height)
