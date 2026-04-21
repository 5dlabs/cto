from __future__ import annotations

import json
import logging
from typing import AsyncIterator

import httpx

from .voice_agents import AgentSpec

log = logging.getLogger("voice-bridge.agent")


class MorganAgentClient:
    def __init__(
        self,
        agent: AgentSpec,
        request_timeout_s: float = 120.0,
    ) -> None:
        self._agent = agent
        self._request_timeout_s = request_timeout_s

    @property
    def is_configured(self) -> bool:
        return bool(
            self._agent.gateway_url
            and self._agent.gateway_token
            and self._agent.model
        )

    async def send_and_stream(self, *, session_id: str, text: str) -> AsyncIterator[str]:
        if not self.is_configured:
            log.warning("agent stub: gateway not configured for %s", self._agent.name)
            yield f"(voice-bridge: {self._agent.name} gateway not configured)"
            return

        url = f"{self._agent.gateway_url.rstrip('/')}/v1/chat/completions"
        headers = {
            "Authorization": f"Bearer {self._agent.gateway_token}",
            "Content-Type": "application/json",
            "Accept": "text/event-stream",
        }
        payload = {
            "model": self._agent.model,
            "stream": True,
            "messages": [{"role": "user", "content": text}],
            "user": f"voice-bridge:{self._agent.name}:{session_id}",
        }

        try:
            async with httpx.AsyncClient(timeout=self._request_timeout_s) as client:
                async with client.stream("POST", url, headers=headers, json=payload) as resp:
                    if resp.status_code >= 400:
                        body = (await resp.aread()).decode("utf-8", "replace")[:400]
                        log.warning("gateway %s %d: %s", self._agent.name, resp.status_code, body)
                        yield f"(voice-bridge: {self._agent.name} returned {resp.status_code})"
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
                        except Exception:
                            chunk = ""
                        if chunk:
                            yield chunk
        except httpx.HTTPError as exc:
            log.warning("gateway request failed for %s: %s", self._agent.name, exc)
            yield f"(voice-bridge: {self._agent.name} unreachable: {exc})"

    async def close(self) -> None:
        return None
