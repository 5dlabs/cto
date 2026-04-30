#!/usr/bin/env python3
"""Patch Hermes v2026.4.23 Discord /sethome command for user-installed Discord contexts.

The upstream image defines /sethome without explicit Discord installation contexts,
which can leave Discord's global command registered only for guild installs. In a
user-installed DM context Discord rejects the interaction before Hermes sees it
with "Unknown integration". This patch is idempotent and also teaches Hermes'
safe sync diff to compare existing command contexts/integration_types so it
recreates stale global commands when these fields differ.
"""
from __future__ import annotations

from pathlib import Path

path = Path("/opt/hermes/gateway/platforms/discord.py")
text = path.read_text()

old_sethome = '''        @tree.command(name="sethome", description="Set this chat as the home channel")
        async def slash_sethome(interaction: discord.Interaction):
            await self._run_simple_slash(interaction, "/sethome")
'''
new_sethome = '''        @tree.command(name="sethome", description="Set this chat as the home channel")
        @discord.app_commands.allowed_contexts(guilds=True, dms=True, private_channels=True)
        @discord.app_commands.allowed_installs(guilds=True, users=True)
        async def slash_sethome(interaction: discord.Interaction):
            await self._run_simple_slash(interaction, "/sethome")
'''
if new_sethome not in text:
    if old_sethome not in text:
        raise SystemExit("sethome command block not found")
    text = text.replace(old_sethome, new_sethome, 1)

old_payload_tail = '''        default_permissions = getattr(command, "default_member_permissions", None)
        if default_permissions is not None:
            payload["default_member_permissions"] = getattr(
                default_permissions, "value", default_permissions
            )
        return payload

    def _canonicalize_app_command_option(self, payload: Dict[str, Any]) -> Dict[str, Any]:
'''
new_payload_tail = '''        default_permissions = getattr(command, "default_member_permissions", None)
        if default_permissions is not None:
            payload["default_member_permissions"] = getattr(
                default_permissions, "value", default_permissions
            )
        contexts = getattr(command, "contexts", None)
        if contexts is None:
            allowed_contexts = getattr(command, "allowed_contexts", None)
            if allowed_contexts is not None and hasattr(allowed_contexts, "to_array"):
                contexts = allowed_contexts.to_array()
        if contexts is not None:
            payload["contexts"] = contexts

        integration_types = getattr(command, "integration_types", None)
        if integration_types is None:
            allowed_installs = getattr(command, "allowed_installs", None)
            if allowed_installs is not None and hasattr(allowed_installs, "to_array"):
                integration_types = allowed_installs.to_array()
        if integration_types is not None:
            payload["integration_types"] = integration_types

        return payload

    def _canonicalize_app_command_option(self, payload: Dict[str, Any]) -> Dict[str, Any]:
'''
if new_payload_tail not in text:
    if old_payload_tail not in text:
        raise SystemExit("existing command payload block not found")
    text = text.replace(old_payload_tail, new_payload_tail, 1)

path.write_text(text)
print("patched discord.py for /sethome user installs")
