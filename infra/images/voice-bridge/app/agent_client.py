"""
Morgan agent adapter.

Morgan (in-cluster OpenClaw StatefulSet) uses the nats-messenger plugin.
Inbound turns are published to `agent.morgan.inbox` (JSON-encoded
AgentMessage with a NATS `reply` inbox). The plugin enqueues the message
as a system event into Morgan's agent session via `enqueueSystemEvent`
and optionally wakes the agent via `requestHeartbeatNow` when the
priority is `urgent`. Replies come back either on:

  - the original `reply` inbox (when Morgan invokes the nats-messenger
    tool with the captured reply context), or
  - `agent.all.broadcast` / `agent.{sender}.inbox` for autonomous
    messages.

For the voice bridge we rely on the request/reply path: publish an
AgentMessage with a unique reply inbox, subscribe to that inbox, and
stream every reply chunk back to the WS client. The first reply arriving
within `reply_timeout_s` becomes the token stream; subsequent messages
on the inbox are also forwarded until the bridge sees an `end: true`
marker or the timeout elapses.

If the NATS client fails to connect (e.g. the Morgan StatefulSet is not
yet running), we fall back to a single stub token so the WS round-trip
still completes and the UI can render a clear error state.
"""

from __future__ import annotations

import asyncio
import json
import logging
import secrets
from typing import Any, AsyncIterator

try:
    from nats.aio.client import Client as NatsClient  # type: ignore
    from nats.aio.msg import Msg as NatsMsg  # type: ignore
except Exception:  # pragma: no cover - optional dep guard
    NatsClient = None  # type: ignore[assignment]
    NatsMsg = None  # type: ignore[assignment]

log = logging.getLogger("voice-bridge.agent")


class MorganAgentClient:
    def __init__(
        self,
        nats_url: str,
        inbox_subject: str = "agent.morgan.inbox",
        replies_subject: str | None = None,
        agent_name: str = "voice-bridge",
        reply_timeout_s: float = 30.0,
    ) -> None:
        self._nats_url = nats_url
        self._inbox_subject = inbox_subject
        # When set, we ALSO subscribe to this fixed subject for replies,
        # in addition to the per-request reply inbox. Leave None for pure
        # request/reply semantics.
        self._fixed_replies_subject = replies_subject
        self._agent_name = agent_name
        self._reply_timeout_s = reply_timeout_s
        self._nc: Any | None = None
        self._connect_lock = asyncio.Lock()

    @property
    def is_configured(self) -> bool:
        return bool(self._nats_url and self._inbox_subject)

    async def _ensure_connected(self) -> Any | None:
        if NatsClient is None:
            return None
        if self._nc is not None and self._nc.is_connected:
            return self._nc
        async with self._connect_lock:
            if self._nc is not None and self._nc.is_connected:
                return self._nc
            nc = NatsClient()
            try:
                await nc.connect(servers=[self._nats_url], connect_timeout=3, max_reconnect_attempts=2)
                self._nc = nc
                log.info("NATS connected: %s", self._nats_url)
                return self._nc
            except Exception as exc:  # noqa: BLE001
                log.warning("NATS connect failed (%s): %s", self._nats_url, exc)
                self._nc = None
                return None

    async def send_and_stream(self, *, session_id: str, text: str) -> AsyncIterator[str]:
        nc = await self._ensure_connected()
        if nc is None:
            # Graceful degrade: caller sees a clear stub response rather
            # than a WS-level error before Morgan is deployed.
            log.info(
                "agent stub (no NATS): session=%s inbox=%s text=%r",
                session_id,
                self._inbox_subject,
                text,
            )
            yield (
                "(voice-bridge: no NATS connection to Morgan yet — your "
                "turn was captured but not routed.)"
            )
            return

        reply_inbox = f"_INBOX.voicebridge.{secrets.token_hex(8)}"
        queue: asyncio.Queue[str | None] = asyncio.Queue()

        async def _on_reply(msg: Any) -> None:
            try:
                payload = msg.data.decode("utf-8", errors="replace")
                try:
                    parsed = json.loads(payload)
                except json.JSONDecodeError:
                    parsed = {"text": payload}
                chunk = parsed.get("text") or parsed.get("content") or ""
                if chunk:
                    await queue.put(chunk)
                if parsed.get("end") is True or parsed.get("final") is True:
                    await queue.put(None)
            except Exception as exc:  # noqa: BLE001
                log.warning("reply handler error: %s", exc)
                await queue.put(None)

        sub = await nc.subscribe(reply_inbox, cb=_on_reply)
        envelope = {
            "from": self._agent_name,
            "to": "morgan",
            "session": session_id,
            "priority": "normal",
            "content": text,
            "kind": "voice_turn",
        }
        try:
            await nc.publish(
                self._inbox_subject,
                json.dumps(envelope).encode("utf-8"),
                reply=reply_inbox,
            )
            await nc.flush()
        except Exception as exc:  # noqa: BLE001
            log.warning("publish failed: %s", exc)
            await sub.unsubscribe()
            yield f"(voice-bridge publish failed: {exc})"
            return

        deadline = asyncio.get_event_loop().time() + self._reply_timeout_s
        try:
            while True:
                remaining = deadline - asyncio.get_event_loop().time()
                if remaining <= 0:
                    break
                try:
                    token = await asyncio.wait_for(queue.get(), timeout=remaining)
                except asyncio.TimeoutError:
                    break
                if token is None:
                    break
                yield token
        finally:
            try:
                await sub.unsubscribe()
            except Exception:  # noqa: BLE001
                pass

    async def close(self) -> None:
        nc = self._nc
        self._nc = None
        if nc is not None:
            try:
                await nc.drain()
            except Exception:  # noqa: BLE001
                pass

