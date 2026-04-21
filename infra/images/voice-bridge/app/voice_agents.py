from __future__ import annotations

import json
import os
from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class AgentSpec:
    name: str
    model: str
    voice_id: str
    gateway_url: str
    gateway_token: str


_DEFAULT_AGENTS: dict[str, AgentSpec] = {
    "morgan": AgentSpec(
        name="morgan",
        model=os.environ.get("MORGAN_MODEL", "openclaw/morgan"),
        voice_id=os.environ.get("MORGAN_VOICE_ID", "iP95p4xoKVk53GoZ742B"),
        gateway_url=os.environ.get(
            "MORGAN_GATEWAY_URL",
            "http://openclaw-morgan.cto.svc.cluster.local:18789",
        ),
        gateway_token=os.environ.get("MORGAN_GATEWAY_TOKEN", "openclaw-internal"),
    ),
    "hermes": AgentSpec(
        name="hermes",
        model=os.environ.get("HERMES_MODEL", "openclaw/hermes"),
        voice_id=os.environ.get("HERMES_VOICE_ID", "iP95p4xoKVk53GoZ742B"),
        gateway_url=os.environ.get(
            "HERMES_GATEWAY_URL",
            os.environ.get(
                "MORGAN_GATEWAY_URL",
                "http://openclaw-morgan.cto.svc.cluster.local:18789",
            ),
        ),
        gateway_token=os.environ.get(
            "HERMES_GATEWAY_TOKEN",
            os.environ.get("MORGAN_GATEWAY_TOKEN", "openclaw-internal"),
        ),
    ),
}


def load_voice_agents() -> dict[str, AgentSpec]:
    raw = os.environ.get("VOICE_AGENTS_JSON", "").strip()
    if not raw:
        return _DEFAULT_AGENTS

    payload = json.loads(raw)
    agents: dict[str, AgentSpec] = {}
    for name, cfg in payload.items():
        agents[name] = AgentSpec(
            name=name,
            model=cfg["model"],
            voice_id=cfg["voice_id"],
            gateway_url=cfg["gateway_url"],
            gateway_token=cfg["gateway_token"],
        )
    return agents


def get_agent(name: str | None) -> AgentSpec | None:
    agents = load_voice_agents()
    if not name:
        return agents.get("morgan") or next(iter(agents.values()), None)
    return agents.get(name)
