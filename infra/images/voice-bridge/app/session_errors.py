"""Structured ERROR frame emission for cto-avatar-session/v1.

See ``docs/specs/avatar-session-protocol.md`` §"Frame Types" #4 ERROR
and §"Failure Modes". The legacy ``{type:"error", error: str}`` frame is
kept alongside these structured frames for back-compat.
"""
from __future__ import annotations

import logging
import time
from typing import Any, Optional

from fastapi import WebSocket
from starlette.websockets import WebSocketState

log = logging.getLogger("voice-bridge.session_errors")

PROTOCOL = "cto-avatar-session/v1"

# Error codes per spec. Recoverable == client may retry with backoff.
NETWORK_DISCONNECT = "NETWORK_DISCONNECT"
STT_FAILED = "STT_FAILED"
TTS_FAILED = "TTS_FAILED"
ASSET_LOAD_FAILED = "ASSET_LOAD_FAILED"
SESSION_TIMEOUT = "SESSION_TIMEOUT"
AUTH_FAILED = "AUTH_FAILED"

_DEFAULT_RECOVERABLE: dict[str, bool] = {
    NETWORK_DISCONNECT: True,
    STT_FAILED: True,
    TTS_FAILED: True,
    ASSET_LOAD_FAILED: True,
    SESSION_TIMEOUT: False,
    AUTH_FAILED: False,
}


def build_error_frame(
    *,
    session_id: Optional[str],
    code: str,
    message: str,
    recoverable: Optional[bool] = None,
    timestamp_ms: Optional[int] = None,
) -> dict[str, Any]:
    """Build a spec-shaped ERROR frame dict.

    ``session_id`` may be ``None`` when emitting before the ``start``
    handshake completes (e.g. on auth failure); we substitute the
    sentinel ``"unknown"`` so downstream consumers can still key on it.
    """
    if recoverable is None:
        recoverable = _DEFAULT_RECOVERABLE.get(code, False)
    if timestamp_ms is None:
        timestamp_ms = int(time.time() * 1000)
    return {
        "protocol": PROTOCOL,
        "type": "ERROR",
        "session_id": session_id or "unknown",
        "code": code,
        "message": message,
        "recoverable": recoverable,
        "timestamp_ms": timestamp_ms,
    }


async def emit_error_frame(
    ws: WebSocket,
    *,
    session_id: Optional[str],
    code: str,
    message: str,
    recoverable: Optional[bool] = None,
) -> None:
    """Best-effort send of a structured ERROR frame on ``ws``.

    Swallows send failures — the caller has already decided to tear
    down the connection and we do not want emission failure to mask
    the original error.
    """
    frame = build_error_frame(
        session_id=session_id,
        code=code,
        message=message,
        recoverable=recoverable,
    )
    try:
        if ws.client_state == WebSocketState.DISCONNECTED:
            return
        await ws.send_json(frame)
    except Exception as exc:  # noqa: BLE001 — defensive; connection already failing
        log.debug("failed to emit structured ERROR frame: %s", exc)
