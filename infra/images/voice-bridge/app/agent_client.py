"""
Morgan agent adapter.

This is the big open question from the plan: how does the in-cluster
`openclaw-morgan-openclaw` StatefulSet receive a new turn, and how does
its streamed reply surface?

Hypotheses (ordered by likelihood based on repo survey):

  1. NATS subject pair (e.g. `openclaw.morgan.input` / `...output`) —
     consistent with how the avatar workers are addressed today.
  2. Linear / Discord bridge intake — agent polls these for new work.
  3. Direct ACP JSON-RPC over a local Unix socket shared via /workspace.

Until we confirm which of these is actually live for Morgan, this class
is a pure stub: it accepts a send_and_stream(session_id, text) call,
logs the intent, and yields a single placeholder token so the WebSocket
plumbing in main.py round-trips cleanly.

Once confirmed, the likely implementation is option (1): subscribe to
`MORGAN_REPLIES_SUBJECT` filtered by session_id, publish to
`MORGAN_INBOX_SUBJECT` with a JSON envelope, stream tokens as they land.
"""

from __future__ import annotations

import asyncio
import logging
from typing import AsyncIterator

log = logging.getLogger("voice-bridge.agent")


class MorganAgentClient:
    def __init__(
        self,
        nats_url: str,
        inbox_subject: str,
        replies_subject: str,
    ) -> None:
        self._nats_url = nats_url
        self._inbox_subject = inbox_subject
        self._replies_subject = replies_subject

    @property
    def is_configured(self) -> bool:
        return bool(self._nats_url and self._inbox_subject and self._replies_subject)

    async def send_and_stream(self, *, session_id: str, text: str) -> AsyncIterator[str]:
        log.info(
            "send_and_stream stub: session=%s inbox=%s replies=%s text=%r",
            session_id,
            self._inbox_subject,
            self._replies_subject,
            text,
        )
        # TODO: wire real NATS request/response once Morgan's subjects are
        # confirmed. For now yield a single placeholder so the WS client
        # sees the full round-trip (transcript → reply → TTS → turn_done).
        yield (
            "(voice-bridge stub) I hear you, but the agent adapter is not "
            "wired to the in-cluster Morgan yet."
        )
        await asyncio.sleep(0)
