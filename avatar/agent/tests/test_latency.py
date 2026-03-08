from __future__ import annotations

import json
from pathlib import Path
from types import SimpleNamespace

from morgan_avatar_agent.latency import (
    LatencyRecorder,
    TurnRecord,
    load_turn_records,
    summarize_records,
)


def test_summarize_records_computes_percentiles() -> None:
    records = [
        TurnRecord(
            turn_id="a", room_name="r",
            started_at=0, estimated_eot_to_first_audio_s=0.4,
            assistant_interrupted=False,
        ),
        TurnRecord(
            turn_id="b", room_name="r",
            started_at=0, estimated_eot_to_first_audio_s=0.7,
            assistant_interrupted=True,
        ),
        TurnRecord(
            turn_id="c", room_name="r",
            started_at=0, estimated_eot_to_first_audio_s=0.5,
            assistant_interrupted=False,
        ),
    ]

    summary = summarize_records(records)

    assert summary["turn_count"] == 3
    assert summary["measured_turn_count"] == 3
    assert summary["p50_eot_to_first_audio_s"] == 0.5
    assert summary["p95_eot_to_first_audio_s"] == 0.7
    assert summary["interrupted_turn_count"] == 1


def test_finalize_with_partial_components() -> None:
    turn = TurnRecord(turn_id="p", room_name="r", started_at=0)
    turn.llm_ttft_s = 0.3
    turn.tts_ttfb_s = 0.1
    turn.finalize()

    assert turn.estimated_eot_to_first_audio_s is not None
    assert abs(turn.estimated_eot_to_first_audio_s - 0.4) < 1e-9
    assert sorted(turn.eot_components_present) == ["llm", "tts"]


def test_greeting_turn_latency() -> None:
    turn = TurnRecord(turn_id="g", room_name="r", started_at=100.0, is_greeting=True)
    turn.llm_ttft_s = 0.2
    turn.tts_ttfb_s = 0.08
    turn.finalize()

    assert turn.is_greeting
    assert turn.greeting_latency_s is not None
    assert turn.greeting_latency_s > 0


def test_summarize_includes_greeting_and_component_stats() -> None:
    greeting = TurnRecord(
        turn_id="g", room_name="r", started_at=0, is_greeting=True,
        llm_ttft_s=0.2, tts_ttfb_s=0.08,
        estimated_eot_to_first_audio_s=0.28,
        greeting_latency_s=2.5,
    )
    conv = TurnRecord(
        turn_id="c", room_name="r", started_at=0, is_greeting=False,
        llm_ttft_s=0.3, tts_ttfb_s=0.12,
        end_of_utterance_delay_s=0.05, transcription_delay_s=0.04,
        estimated_eot_to_first_audio_s=0.51,
    )
    summary = summarize_records([greeting, conv])

    assert summary["greeting_count"] == 1
    assert summary["greeting_latency_p50_s"] == 2.5
    assert summary["conversational_turn_count"] == 1
    assert summary["conversational_p50_eot_s"] == 0.51
    assert summary["llm_ttft_s_count"] == 2
    assert summary["tts_ttfb_s_count"] == 2


def test_latency_recorder_writes_turn_record(tmp_path: Path) -> None:
    recorder = LatencyRecorder(tmp_path, "room-123", {"stt_mode": "livekit-flux"})

    recorder.handle_user_state(SimpleNamespace(old_state="speaking", new_state="listening"))
    recorder.handle_user_transcribed(
        SimpleNamespace(is_final=True, transcript="hello Morgan", language="en")
    )
    recorder.handle_metrics(
        SimpleNamespace(
            type="eou_metrics",
            end_of_utterance_delay=0.12,
            transcription_delay=0.08,
        )
    )
    recorder.handle_metrics(SimpleNamespace(type="llm_metrics", ttft=0.22, duration=0.8))
    recorder.handle_metrics(SimpleNamespace(type="tts_metrics", ttfb=0.11, audio_duration=1.4))

    records = recorder.records_path.read_text(encoding="utf-8").splitlines()

    assert len(records) == 2
    assert "hello Morgan" in records[1]


def test_load_turn_records_ignores_unknown_fields(tmp_path: Path) -> None:
    path = tmp_path / "test-latency.ndjson"
    turn_data = {
        "record_type": "turn",
        "turn_id": "x",
        "room_name": "r",
        "started_at": 100.0,
        "llm_ttft_s": 0.3,
        "some_future_field": "ignored",
    }
    path.write_text(json.dumps(turn_data) + "\n", encoding="utf-8")

    records = load_turn_records(path)
    assert len(records) == 1
    assert records[0].llm_ttft_s == 0.3
    assert not hasattr(records[0], "some_future_field")
