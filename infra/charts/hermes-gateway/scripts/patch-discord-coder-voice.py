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
changed = False

helper_marker = """    async def _run_post_connect_initialization(self) -> None:
        \"\"\"Finish non-critical startup work after Discord is connected.\"\"\"
        if not self._client:
            return
        try:
            sync_policy = self._get_discord_command_sync_policy()
"""

helper_insert = """    async def _run_post_connect_initialization(self) -> None:
        \"\"\"Finish non-critical startup work after Discord is connected.\"\"\"
        if not self._client:
            return
        try:
            cto_guild = discord.Object(id=1490117571977019524)
            self._client.tree.copy_global_to(guild=cto_guild)
            synced = await self._client.tree.sync(guild=cto_guild)
            logger.info(
                \"[%s] Synced %d CTO guild slash command(s)\",
                self.name,
                len(synced),
            )
        except Exception as e:
            logger.warning(\"[%s] CTO guild slash command sync failed: %s\", self.name, e, exc_info=True)
        try:
            sync_policy = self._get_discord_command_sync_policy()
"""

if "Synced %d CTO guild slash command(s)" not in text:
    if helper_marker not in text:
        raise SystemExit("post-connect initialization block not found")
    text = text.replace(helper_marker, helper_insert)
    changed = True

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
    changed = True

if changed:
    path.write_text(text)
