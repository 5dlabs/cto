from __future__ import annotations

import asyncio
import logging

from morgan_avatar_agent.startup import (
    is_expected_startup_disconnect,
    room_is_disconnected,
    run_startup_step,
)


def test_expected_startup_disconnect_matches_greeting_close() -> None:
    assert is_expected_startup_disconnect(
        RuntimeError("AgentSession is closing, cannot use say()")
    )


def test_expected_startup_disconnect_matches_participant_wait_disconnect() -> None:
    assert is_expected_startup_disconnect(
        RuntimeError("room disconnected while waiting for participant")
    )


def test_expected_startup_disconnect_rejects_other_runtime_errors() -> None:
    assert not is_expected_startup_disconnect(RuntimeError("provider failed to start"))


def test_room_is_disconnected_uses_connection_state_name() -> None:
    state = type("State", (), {"name": "CONN_DISCONNECTED"})()
    room = type("Room", (), {"connection_state": state})()

    assert room_is_disconnected(room)


def test_run_startup_step_swallows_only_expected_disconnect(caplog) -> None:
    async def action() -> None:
        raise RuntimeError("AgentSession is closing, cannot use say()")

    with caplog.at_level(logging.INFO):
        completed = asyncio.run(run_startup_step("greeting", action, logging.getLogger(__name__)))

    assert completed is False
    assert "startup.greeting.skip" in caplog.text


def test_run_startup_step_reraises_unexpected_runtime_error() -> None:
    async def action() -> None:
        raise RuntimeError("provider failed to start")

    try:
        asyncio.run(run_startup_step("avatar", action, logging.getLogger(__name__)))
    except RuntimeError as exc:
        assert str(exc) == "provider failed to start"
    else:
        raise AssertionError("Expected unexpected RuntimeError to be re-raised.")

