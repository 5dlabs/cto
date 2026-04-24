from __future__ import annotations

from pathlib import Path

from morgan_avatar_agent.musetalk_avatar import MuseTalkAvatarSession
from morgan_avatar_agent.musetalk_inference import AudioChunk, MuseTalkInferenceEngine


def test_musetalk_mocked_stream_sustains_target_fps(tmp_path: Path) -> None:
    engine = MuseTalkInferenceEngine(
        persona_id="morgan-v1",
        personas_root=tmp_path,
        target_fps=30,
        frame_width=32,
        frame_height=32,
    )
    engine.warmup()

    fixture_audio = [
        AudioChunk(samples=[0.1] * 1600, sample_rate=16000, duration_ms=100.0) for _ in range(10)
    ]

    stats = engine.benchmark(fixture_audio)

    assert stats.frames_rendered >= 28
    assert stats.fps >= 28.0


def test_musetalk_avatar_session_emits_publishable_frames(tmp_path: Path) -> None:
    engine = MuseTalkInferenceEngine(
        persona_id="morgan-v1",
        personas_root=tmp_path,
        target_fps=30,
        frame_width=16,
        frame_height=16,
    )
    session = MuseTalkAvatarSession(engine)

    frames = list(
        session.publishable_frames(
            [AudioChunk(samples=[0.0] * 1600, sample_rate=16000, duration_ms=100.0)]
        )
    )

    assert frames
    assert frames[0].width == 16
    assert frames[0].height == 16
    assert frames[0].payload_size == 16 * 16 * 4


def test_musetalk_avatar_push_rgba_frame_noops_without_source(tmp_path: Path) -> None:
    engine = MuseTalkInferenceEngine(
        persona_id="morgan-v1",
        personas_root=tmp_path,
        target_fps=30,
        frame_width=2,
        frame_height=2,
    )
    session = MuseTalkAvatarSession(engine)

    assert session.push_rgba_frame(width=2, height=2, rgba=bytes(2 * 2 * 4)) is False
