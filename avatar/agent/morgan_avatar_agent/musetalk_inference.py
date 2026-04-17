from __future__ import annotations

import math
import time
from collections.abc import Iterable
from dataclasses import dataclass
from pathlib import Path


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
    ) -> None:
        self.persona_id = persona_id
        self.personas_root = personas_root
        self.target_fps = target_fps
        self.frame_width = frame_width
        self.frame_height = frame_height
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

    def _render_rgba(self, frame_index: int, local_index: int, total_frames: int) -> bytes:
        mouth = int(80 + 120 * math.sin((local_index / max(total_frames, 1)) * math.pi))
        r = (frame_index * 13) % 255
        g = mouth % 255
        b = (255 - r) % 255
        pixel = bytes((r, g, b, 255))
        return pixel * (self.frame_width * self.frame_height)
