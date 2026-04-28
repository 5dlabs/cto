from __future__ import annotations

import logging
from collections.abc import Awaitable, Callable

_AGENT_SESSION_CLOSING_PHRASE = "AgentSession is closing"
_AGENT_SESSION_SAY_PHRASE = "cannot use say()"
_ROOM_DISCONNECTED_WAITING_PHRASE = "room disconnected while waiting for participant"


def is_expected_startup_disconnect(exc: RuntimeError) -> bool:
    message = str(exc)
    return (
        _AGENT_SESSION_CLOSING_PHRASE in message and _AGENT_SESSION_SAY_PHRASE in message
    ) or _ROOM_DISCONNECTED_WAITING_PHRASE in message


def room_is_disconnected(room: object) -> bool:
    state = getattr(room, "connection_state", None)
    if callable(state):
        state = state()
    state_name = getattr(state, "name", None)
    return isinstance(state_name, str) and "disconnected" in state_name.lower()


async def run_startup_step(
    step_name: str,
    action: Callable[[], Awaitable[None]],
    logger: logging.Logger,
) -> bool:
    try:
        await action()
    except RuntimeError as exc:
        if is_expected_startup_disconnect(exc):
            logger.info("startup.%s.skip reason=%s", step_name, exc)
            return False
        raise
    return True

