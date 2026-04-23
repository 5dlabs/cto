"""SESSION_STATE frame helpers for the cto-avatar-session/v1 protocol.

See ``docs/specs/avatar-session-protocol.md`` for the frame contract. This
module owns the envelope shape only; state transition points are wired in
:mod:`app.main`. The protocol's ``error`` / ``reconnecting`` states are
intentionally **not** emitted from this module — those are owned by the
ERROR-frame and reconnect workstreams.
"""

from __future__ import annotations

import time
from typing import Literal

PROTOCOL: Literal["cto-avatar-session/v1"] = "cto-avatar-session/v1"

AvatarSessionState = Literal[
    "idle",
    "connecting",
    "connected",
    "listening",
    "speaking",
    "disconnecting",
]


def build_session_state_frame(
    *,
    state: AvatarSessionState,
    session_id: str,
    agent_name: str,
    timestamp_ms: int | None = None,
) -> dict[str, object]:
    """Return a ``SESSION_STATE`` envelope matching ``SessionStateFrame`` in TS.

    ``timestamp_ms`` defaults to the current wall clock in epoch milliseconds
    (matching ``Date.now()`` on the web side). Callers may pass an explicit
    value for deterministic tests.
    """

    return {
        "protocol": PROTOCOL,
        "type": "SESSION_STATE",
        "state": state,
        "session_id": session_id,
        "agent_name": agent_name,
        "timestamp_ms": int(time.time() * 1000) if timestamp_ms is None else timestamp_ms,
    }
