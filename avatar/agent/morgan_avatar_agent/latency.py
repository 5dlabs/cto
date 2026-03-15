from __future__ import annotations

import json
import math
import time
import uuid
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any


def _now() -> float:
    return time.time()


def _coerce_text(value: Any) -> str | None:
    if value is None:
        return None
    text = str(value).strip()
    return text or None


def _coerce_float(value: Any) -> float | None:
    if value is None:
        return None
    try:
        return float(value)
    except (TypeError, ValueError):
        return None


@dataclass
class TurnRecord:
    turn_id: str
    room_name: str
    started_at: float
    is_greeting: bool = False
    user_listening_at: float | None = None
    transcript: str | None = None
    transcript_language: str | None = None
    speech_source: str | None = None
    speech_created_at: float | None = None
    assistant_text: str | None = None
    agent_state: str | None = None
    end_of_utterance_delay_s: float | None = None
    transcription_delay_s: float | None = None
    llm_ttft_s: float | None = None
    llm_duration_s: float | None = None
    tts_ttfb_s: float | None = None
    tts_audio_duration_s: float | None = None
    estimated_eot_to_first_audio_s: float | None = None
    greeting_latency_s: float | None = None
    eot_components_present: list[str] = field(default_factory=list)
    assistant_interrupted: bool | None = None
    error: str | None = None
    details: dict[str, Any] = field(default_factory=dict)

    def finalize(self) -> None:
        parts = {
            "eou": self.end_of_utterance_delay_s,
            "stt": self.transcription_delay_s,
            "llm": self.llm_ttft_s,
            "tts": self.tts_ttfb_s,
        }
        present = {k: v for k, v in parts.items() if v is not None}
        self.eot_components_present = list(present.keys())

        if present:
            self.estimated_eot_to_first_audio_s = sum(present.values())

        if self.is_greeting and self.tts_ttfb_s is not None:
            self.greeting_latency_s = _now() - self.started_at


class LatencyRecorder:
    def __init__(
        self,
        base_dir: Path,
        room_name: str,
        config_snapshot: dict[str, Any] | None = None,
    ):
        self.room_name = room_name
        self.base_dir = Path(base_dir)
        self.base_dir.mkdir(parents=True, exist_ok=True)
        self.records_path = self.base_dir / f"{room_name}-latency.ndjson"
        self.summary_path = self.base_dir / f"{room_name}-summary.json"
        self.records: list[TurnRecord] = []
        self.current_turn: TurnRecord | None = None
        self.config_snapshot = config_snapshot or {}
        self._write_session_header()

    def _write_session_header(self) -> None:
        header = {
            "record_type": "session_start",
            "room_name": self.room_name,
            "timestamp": _now(),
            "config": self.config_snapshot,
        }
        with self.records_path.open("a", encoding="utf-8") as handle:
            handle.write(json.dumps(header, sort_keys=True) + "\n")

    def _new_turn(self) -> TurnRecord:
        return TurnRecord(
            turn_id=str(uuid.uuid4()),
            room_name=self.room_name,
            started_at=_now(),
        )

    def _ensure_turn(self) -> TurnRecord:
        if self.current_turn is None:
            self.current_turn = self._new_turn()
        return self.current_turn

    def handle_user_state(self, event: Any) -> None:
        new_state = _coerce_text(getattr(event, "new_state", None))
        old_state = _coerce_text(getattr(event, "old_state", None))

        if new_state == "speaking":
            self.current_turn = self._new_turn()
            self.current_turn.details["state_transition"] = f"{old_state}->{new_state}"
            return

        if new_state == "listening":
            turn = self._ensure_turn()
            turn.user_listening_at = _now()
            turn.details["state_transition"] = f"{old_state}->{new_state}"

    def handle_user_transcribed(self, event: Any) -> None:
        is_final = bool(getattr(event, "is_final", False))
        if not is_final:
            return

        turn = self._ensure_turn()
        turn.transcript = _coerce_text(getattr(event, "transcript", None))
        turn.transcript_language = _coerce_text(getattr(event, "language", None))

    def handle_speech_created(self, event: Any) -> None:
        turn = self._ensure_turn()
        turn.speech_source = _coerce_text(getattr(event, "source", None))
        turn.speech_created_at = _now()
        if not self.records and turn.transcript is None:
            turn.is_greeting = True

    def handle_agent_state(self, event: Any) -> None:
        turn = self._ensure_turn()
        turn.agent_state = _coerce_text(getattr(event, "new_state", None))

    def handle_conversation_item(self, event: Any) -> None:
        item = getattr(event, "item", None)
        role = _coerce_text(getattr(item, "role", None))
        if role != "assistant":
            return

        turn = self._ensure_turn()
        turn.assistant_text = _coerce_text(getattr(item, "text_content", None))
        interrupted = getattr(item, "interrupted", None)
        if interrupted is not None:
            turn.assistant_interrupted = bool(interrupted)

    def handle_metrics(self, metric: Any) -> None:
        turn = self._ensure_turn()
        metric_type = _coerce_text(getattr(metric, "type", None)) or type(metric).__name__.lower()

        if "eou" in metric_type:
            turn.end_of_utterance_delay_s = _coerce_float(
                getattr(metric, "end_of_utterance_delay", None)
            )
            turn.transcription_delay_s = _coerce_float(
                getattr(metric, "transcription_delay", None)
            )
        elif "llm" in metric_type:
            turn.llm_ttft_s = _coerce_float(getattr(metric, "ttft", None))
            turn.llm_duration_s = _coerce_float(getattr(metric, "duration", None))
        elif "tts" in metric_type:
            turn.tts_ttfb_s = _coerce_float(getattr(metric, "ttfb", None))
            turn.tts_audio_duration_s = _coerce_float(getattr(metric, "audio_duration", None))
            self.finalize_turn()

    def handle_close(self, error: Any = None) -> None:
        if self.current_turn and self.current_turn not in self.records:
            if error is not None:
                self.current_turn.error = _coerce_text(error)
            self.finalize_turn(force=True)
        self.write_summary()

    def finalize_turn(self, force: bool = False) -> None:
        if self.current_turn is None:
            return

        self.current_turn.finalize()
        if not force and self.current_turn.tts_ttfb_s is None:
            return

        record = self.current_turn
        self.records.append(record)
        serialized = {"record_type": "turn", **asdict(record)}
        with self.records_path.open("a", encoding="utf-8") as handle:
            handle.write(json.dumps(serialized, sort_keys=True) + "\n")
        self.current_turn = None

    def write_summary(self) -> dict[str, Any]:
        summary = summarize_records(self.records)
        payload = {
            "room_name": self.room_name,
            "generated_at": _now(),
            "summary": summary,
        }
        with self.summary_path.open("w", encoding="utf-8") as handle:
            json.dump(payload, handle, indent=2, sort_keys=True)
        return payload


def _percentile(values: list[float], percentile: float) -> float | None:
    if not values:
        return None

    ordered = sorted(values)
    index = max(0, math.ceil((percentile / 100) * len(ordered)) - 1)
    return ordered[index]


def _component_stats(
    records: list[TurnRecord], attr: str
) -> dict[str, float | None]:
    values = [
        getattr(r, attr) for r in records if getattr(r, attr, None) is not None
    ]
    return {
        f"{attr}_p50": _percentile(values, 50),
        f"{attr}_p95": _percentile(values, 95),
        f"{attr}_count": len(values),
    }


def summarize_records(records: list[TurnRecord]) -> dict[str, Any]:
    eot_values = [
        r.estimated_eot_to_first_audio_s
        for r in records
        if r.estimated_eot_to_first_audio_s is not None
    ]
    greeting_values = [
        r.greeting_latency_s
        for r in records
        if r.is_greeting and r.greeting_latency_s is not None
    ]
    conv_values = [
        r.estimated_eot_to_first_audio_s
        for r in records
        if not r.is_greeting and r.estimated_eot_to_first_audio_s is not None
    ]
    interrupted = sum(1 for r in records if r.assistant_interrupted)

    summary: dict[str, Any] = {
        "turn_count": len(records),
        "measured_turn_count": len(eot_values),
        "p50_eot_to_first_audio_s": _percentile(eot_values, 50),
        "p95_eot_to_first_audio_s": _percentile(eot_values, 95),
        "fastest_eot_to_first_audio_s": min(eot_values) if eot_values else None,
        "slowest_eot_to_first_audio_s": max(eot_values) if eot_values else None,
        "interrupted_turn_count": interrupted,
        "greeting_count": sum(1 for r in records if r.is_greeting),
        "greeting_latency_p50_s": _percentile(greeting_values, 50),
        "conversational_turn_count": len(conv_values),
        "conversational_p50_eot_s": _percentile(conv_values, 50),
        "conversational_p95_eot_s": _percentile(conv_values, 95),
    }

    for attr in ("llm_ttft_s", "tts_ttfb_s", "end_of_utterance_delay_s", "transcription_delay_s"):
        summary.update(_component_stats(records, attr))

    return summary


def load_turn_records(path: Path) -> list[TurnRecord]:
    known_fields = {f.name for f in TurnRecord.__dataclass_fields__.values()}
    records: list[TurnRecord] = []
    for line in Path(path).read_text(encoding="utf-8").splitlines():
        payload = json.loads(line)
        if payload.get("record_type") != "turn":
            continue
        payload.pop("record_type", None)
        filtered = {k: v for k, v in payload.items() if k in known_fields}
        records.append(TurnRecord(**filtered))
    return records
