"""
Morgan agent adapter: OpenAI-compatible HTTP streaming client.

Morgan (in-cluster OpenClaw StatefulSet) exposes an OpenAI-compatible HTTP
gateway on port 18789. We speak `POST /v1/chat/completions` with
`stream: true` and yield assistant content chunks as they arrive.

This is substantially simpler than the previous NATS request/reply path:
the gateway already handles session multiplexing and gives us a familiar
SSE transport.

Env defaults (see main.py):

  MORGAN_GATEWAY_URL   = http://openclaw-morgan.cto.svc.cluster.local:18789
  MORGAN_GATEWAY_TOKEN = openclaw-internal
  MORGAN_MODEL         = openclaw/morgan

If the gateway is unreachable we fall back to a single stub token so the
WS round-trip still completes and the UI can render a clear error state.
"""

from __future__ import annotations

import json
import logging
from typing import AsyncIterator

import httpx

log = logging.getLogger("voice-bridge.agent")


class MorganAgentClient:
    def __init__(
        self,
        gateway_url: str,
        gateway_token: str,
        model: str = "openclaw/morgan",
        request_timeout_s: float = 120.0,
    ) -> None:
        self._gateway_url = gateway_url.rstrip("/")
        self._gateway_token = gateway_token
        self._model = model
        self._request_timeout_s = request_timeout_s

    @property
    def is_configured(self) -> bool:
        return bool(self._gateway_url and self._gateway_token and self._model)

    async def send_and_stream(self, *, session_id: str, text: str) -> AsyncIterator[str]:
        if not self.is_configured:
            log.warning("agent stub: gateway not configured")
            yield "(voice-bridge: Morgan gateway not configured)"
            return

        url = f"{self._gateway_url}/v1/chat/completions"
        headers = {
            "Authorization": f"Bearer {self._gateway_token}",
            "Content-Type": "application/json",
            "Accept": "text/event-stream",
        }
        payload = {
            "model": self._model,
            "stream": True,
            "messages": [{"role": "user", "content": text}],
            "user": f"voice-bridge:{session_id}",
        }

        try:
            async with httpx.AsyncClient(timeout=self._request_timeout_s) as client:
                async with client.stream("POST", url, headers=headers, json=payload) as resp:
                    if resp.status_code >= 400:
                        body = (await resp.aread()).decode("utf-8", "replace")[:400]
                        log.warning("gateway %d: %s", resp.status_code, body)
                        yield f"(voice-bridge: Morgan returned {resp.status_code})"
                        return
                    async for line in resp.aiter_lines():
                        if not line or not line.startswith("data:"):
                            continue
                        data = line[5:].strip()
                        if not data or data == "[DONE]":
                            if data == "[DONE]":
                                return
                            continue
                        try:
                            obj = json.loads(data)
                        except json.JSONDecodeError:
                            continue
                        try:
                            choices = obj.get("choices") or []
                            if not choices:
                                continue
                            delta = choices[0].get("delta") or choices[0].get("message") or {}
                            chunk = delta.get("content") or ""
                        except Exception:  # noqa: BLE001
                            chunk = ""
                        if chunk:
                            yield chunk
        except httpx.HTTPError as exc:
            log.warning("gateway request failed: %s", exc)
            yield f"(voice-bridge: Morgan unreachable: {exc})"

    async def close(self) -> None:  # pragma: no cover - nothing to tear down per-request
        return None
