"""Session state management for narrator sidecar."""

from __future__ import annotations

import asyncio
import logging
import time
from collections import deque
from dataclasses import dataclass, field
from typing import Any

from aiortc import RTCPeerConnection, RTCSessionDescription

log = logging.getLogger("session")

ACP_WINDOW_SIZE = 20


@dataclass
class SessionState:
    """Holds all state for a single narration session."""

    session_id: str
    persona_id: str = "blaze"
    created_at: float = field(default_factory=time.time)
    pc: RTCPeerConnection | None = None
    acp_events: deque[dict[str, Any]] = field(default_factory=lambda: deque(maxlen=ACP_WINDOW_SIZE))
    last_phrase: str | None = None
    last_urgency: str = "low"
    last_phrase_time: float = 0.0
    tailer_task: asyncio.Task | None = None
    narrator_task: asyncio.Task | None = None
    tts_task: asyncio.Task | None = None
    active: bool = True

    def add_acp_event(self, event: dict[str, Any]) -> None:
        self.acp_events.append(event)

    def get_acp_window(self) -> list[dict[str, Any]]:
        return list(self.acp_events)

    def set_phrase(self, phrase: str, urgency: str) -> None:
        self.last_phrase = phrase
        self.last_urgency = urgency
        self.last_phrase_time = time.time()

    async def cleanup(self) -> None:
        self.active = False
        if self.tailer_task and not self.tailer_task.done():
            self.tailer_task.cancel()
        if self.narrator_task and not self.narrator_task.done():
            self.narrator_task.cancel()
        if self.tts_task and not self.tts_task.done():
            self.tts_task.cancel()
        if self.pc:
            await self.pc.close()
        log.info("Session %s cleaned up", self.session_id)


class SessionRegistry:
    """In-memory registry of active sessions."""

    def __init__(self) -> None:
        self._sessions: dict[str, SessionState] = {}

    def create(self, session_id: str, persona_id: str = "blaze") -> SessionState:
        state = SessionState(session_id=session_id, persona_id=persona_id)
        self._sessions[session_id] = state
        log.info("Created session %s (persona=%s)", session_id, persona_id)
        return state

    def get(self, session_id: str) -> SessionState | None:
        return self._sessions.get(session_id)

    async def delete(self, session_id: str) -> None:
        state = self._sessions.pop(session_id, None)
        if state:
            await state.cleanup()
            log.info("Deleted session %s", session_id)

    def list_sessions(self) -> list[dict[str, Any]]:
        return [
            {
                "session_id": s.session_id,
                "persona_id": s.persona_id,
                "active": s.active,
                "last_phrase": s.last_phrase,
                "last_urgency": s.last_urgency,
                "acp_events_count": len(s.acp_events),
            }
            for s in self._sessions.values()
        ]

    async def cleanup_all(self) -> None:
        for session_id in list(self._sessions.keys()):
            await self.delete(session_id)
