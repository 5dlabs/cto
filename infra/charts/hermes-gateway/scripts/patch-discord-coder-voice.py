#!/usr/bin/env python3
"""Add a Coder-specific /coder command group to Hermes' Discord gateway.

The upstream image already exposes /voice channel, but in CTO Discord that is
hard to discover because several bots expose old or duplicate voice commands.
This runtime patch registers a single clear /coder voice entry point plus
/coder status and /coder leave aliases that proxy to the existing Hermes voice
command handlers.

The patch is intentionally small and idempotent: it edits only the deployed
container copy of /opt/hermes/gateway/platforms/discord.py at pod startup and
keeps all voice behavior in upstream Hermes.
"""
from __future__ import annotations

from pathlib import Path

path = Path("/opt/hermes/gateway/platforms/discord.py")
text = path.read_text()

marker = """        @tree.command(name=\"voice\", description=\"Toggle voice reply mode\")
        @discord.app_commands.describe(mode=\"Voice mode: on, off, tts, channel, leave, or status\")
        @discord.app_commands.choices(mode=[
            discord.app_commands.Choice(name=\"channel — join your voice channel\", value=\"channel\"),
            discord.app_commands.Choice(name=\"leave — leave voice channel\", value=\"leave\"),
            discord.app_commands.Choice(name=\"on — voice reply to voice messages\", value=\"on\"),
            discord.app_commands.Choice(name=\"tts — voice reply to all messages\", value=\"tts\"),
            discord.app_commands.Choice(name=\"off — text only\", value=\"off\"),
            discord.app_commands.Choice(name=\"status — show current mode\", value=\"status\"),
        ])
        async def slash_voice(interaction: discord.Interaction, mode: str = \"\"):
            await self._run_simple_slash(interaction, f\"/voice {mode}\".strip())
"""

insert = marker + """

        coder_group = discord.app_commands.Group(
            name=\"coder\",
            description=\"Talk to Coder\",
        )

        @coder_group.command(name=\"voice\", description=\"Start a live voice session with Coder\")
        async def slash_coder_voice(interaction: discord.Interaction):
            await self._run_simple_slash(
                interaction,
                \"/voice channel\",
                \"Starting Coder voice session~\",
            )

        @coder_group.command(name=\"status\", description=\"Show Coder voice session status\")
        async def slash_coder_status(interaction: discord.Interaction):
            await self._run_simple_slash(interaction, \"/voice status\")

        @coder_group.command(name=\"leave\", description=\"End the Coder voice session\")
        async def slash_coder_leave(interaction: discord.Interaction):
            await self._run_simple_slash(interaction, \"/voice leave\")

        if \"coder\" not in {cmd.name for cmd in tree.get_commands()}:
            tree.add_command(coder_group)
"""

if "coder_group = discord.app_commands.Group(" not in text:
    if marker not in text:
        raise SystemExit("voice slash command block not found")
    text = text.replace(marker, insert)
    path.write_text(text)
