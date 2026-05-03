#!/usr/bin/env python3
"""Kubernetes exec probe for the Hermes Discord gateway.

Validates the runtime state file written by `hermes gateway run` rather than
checking for a generic process. This catches stale/disconnected Discord gateway
sessions while avoiding PID-file assumptions that do not hold in containers.
"""
from __future__ import annotations

import json
import pathlib
import sys

STATE_PATH = pathlib.Path("/opt/data/gateway_state.json")

try:
    state = json.loads(STATE_PATH.read_text())
except FileNotFoundError:
    print("gateway_state_missing", file=sys.stderr)
    sys.exit(1)
except Exception as exc:  # pragma: no cover - probe diagnostic path
    print(f"gateway_state_unreadable: {exc}", file=sys.stderr)
    sys.exit(1)

if state.get("gateway_state") != "running":
    print(f"gateway_not_running: {state.get('gateway_state')!r}", file=sys.stderr)
    sys.exit(1)

platforms = state.get("platforms") or {}
discord = platforms.get("discord") or {}
if discord.get("state") != "connected":
    print(f"discord_not_connected: {discord.get('state')!r}", file=sys.stderr)
    sys.exit(1)

print("health_ok")
