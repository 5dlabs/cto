"""Structured JSON logging for avatar session protocol metrics.

Mirrors the client-side ``avatar/web/lib/avatar-metrics.ts`` recorder. The
voice-bridge image does **not** pin ``prometheus_client``, so instead of
exposing a metrics endpoint we emit structured JSON log events on the
``avatar.metrics`` logger. Downstream collectors (Loki / Promtail / fluent-bit)
can parse these events and convert to Prometheus series if needed.

Metric names and targets are taken from
``docs/specs/avatar-session-protocol.md``.
"""
from __future__ import annotations

import json
import logging
import time
from dataclasses import dataclass, field
from typing import Any, Callable, Dict, Optional

logger = logging.getLogger("avatar.metrics")

CONNECTION_LATENCY_TARGET_MS = 2000
AUDIO_LATENCY_TARGET_MS = 1500
ERROR_RECOVERY_TARGET_MS = 5000


def _now_ms() -> float:
    return time.monotonic() * 1000.0


def _emit(metric: str, value_ms: float, target_ms: int, **extra: Any) -> None:
    """Emit a single structured metric event.

    The payload is JSON-serialised into the log message so it can be ingested
    by log-based metric pipelines regardless of handler configuration.
    """
    payload: Dict[str, Any] = {
        "event": "avatar_metric",
        "metric": metric,
        "value_ms": round(float(value_ms), 3),
        "target_ms": target_ms,
        "within_target": float(value_ms) <= float(target_ms),
    }
    for key, val in extra.items():
        if val is not None:
            payload[key] = val
    try:
        logger.info(json.dumps(payload, default=str))
    except (TypeError, ValueError):  # pragma: no cover - defensive
        logger.info("avatar_metric=%s value_ms=%.3f", metric, value_ms)


def record_connection_latency_ms(value_ms: float, **extra: Any) -> None:
    """Record metric 1: room.connected_at - connection_requested_at."""
    _emit("connection_latency_ms", value_ms, CONNECTION_LATENCY_TARGET_MS, **extra)


def record_audio_latency_ms(value_ms: float, **extra: Any) -> None:
    """Record metric 2: audio_ready_at - room.connected_at."""
    _emit("audio_latency_ms", value_ms, AUDIO_LATENCY_TARGET_MS, **extra)


def record_error_recovery_ms(value_ms: float, **extra: Any) -> None:
    """Record metric 5: recovered_at - error_observed_at."""
    _emit("error_recovery_ms", value_ms, ERROR_RECOVERY_TARGET_MS, **extra)


@dataclass
class SessionTimer:
    """Helper to capture monotonic timestamps keyed by label.

    Usage::

        timer = SessionTimer()
        timer.mark("connect_start")
        timer.mark("room_connected")
        timer.record_delta(
            "connect_start",
            "room_connected",
            record_connection_latency_ms,
            session_id=session_id,
        )
    """

    marks: Dict[str, float] = field(default_factory=dict)

    def mark(self, label: str, at_ms: Optional[float] = None) -> float:
        ts = _now_ms() if at_ms is None else float(at_ms)
        self.marks[label] = ts
        return ts

    def get(self, label: str) -> Optional[float]:
        return self.marks.get(label)

    def delta_ms(self, start: str, end: str) -> Optional[float]:
        a = self.marks.get(start)
        b = self.marks.get(end)
        if a is None or b is None:
            return None
        return max(0.0, b - a)

    def record_delta(
        self,
        start: str,
        end: str,
        emitter: Callable[..., None],
        **extra: Any,
    ) -> Optional[float]:
        delta = self.delta_ms(start, end)
        if delta is None:
            return None
        emitter(delta, **extra)
        return delta


__all__ = [
    "AUDIO_LATENCY_TARGET_MS",
    "CONNECTION_LATENCY_TARGET_MS",
    "ERROR_RECOVERY_TARGET_MS",
    "SessionTimer",
    "record_audio_latency_ms",
    "record_connection_latency_ms",
    "record_error_recovery_ms",
]
