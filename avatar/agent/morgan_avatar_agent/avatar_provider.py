from __future__ import annotations

import logging
from typing import Any, Protocol

from livekit.agents._exceptions import APIStatusError

from .config import AgentConfig
from .echomimic_avatar import build_echomimic_avatar_session

logger = logging.getLogger(__name__)


class AvatarProvider(Protocol):
    async def start(self, session: Any, room: Any) -> None: ...

    async def stop(self) -> None: ...


class DisabledAvatarProvider:
    async def start(self, session: Any, room: Any) -> None:
        del session, room
        logger.info("avatar.provider.disabled")

    async def stop(self) -> None:
        return


class LemonSliceAvatarProvider:
    def __init__(self, config: AgentConfig, *, allow_audio_only_fallback: bool) -> None:
        self._config = config
        self._allow_audio_only_fallback = allow_audio_only_fallback
        self._avatar: Any | None = None

    async def start(self, session: Any, room: Any) -> None:
        try:
            from livekit.plugins import lemonslice  # type: ignore[import-not-found]
        except ImportError:
            logger.exception("lemonslice.provider.import_failed")
            if self._allow_audio_only_fallback:
                logger.warning("Continuing in audio-only mode because LemonSlice is unavailable")
                return
            raise

        avatar_kwargs: dict[str, Any] = {
            "agent_prompt": self._config.avatar_prompt,
            "idle_timeout": self._config.avatar_idle_timeout,
        }
        if self._config.has_lemonslice_agent_id:
            avatar_kwargs["agent_id"] = self._config.lemonslice_agent_id
        else:
            avatar_kwargs["agent_image_url"] = self._config.avatar_image_url

        self._avatar = lemonslice.AvatarSession(**avatar_kwargs)
        try:
            await self._avatar.start(session, room=room)
        except Exception as exc:
            root = exc.__cause__ or exc
            if isinstance(root, APIStatusError):
                logger.error(
                    "LemonSlice session start failed: status=%s body=%r message=%s",
                    root.status_code,
                    root.body,
                    root.message,
                )
                body_text = str(root.body).lower() if root.body is not None else ""
                if (
                    self._allow_audio_only_fallback
                    and root.status_code == 402
                    and "insufficient funds" in body_text
                ):
                    logger.warning(
                        "LemonSlice credits unavailable, continuing in audio-only mode "
                        "(set MORGAN_ALLOW_AUDIO_ONLY_FALLBACK=false to fail hard)"
                    )
                    return
                raise

            logger.error("LemonSlice session start failed: %r", root)
            if not self._allow_audio_only_fallback:
                raise
            logger.warning(
                "Continuing in audio-only mode after LemonSlice start failure "
                "(set MORGAN_ALLOW_AUDIO_ONLY_FALLBACK=false to fail hard)"
            )

    async def stop(self) -> None:
        stop = getattr(self._avatar, "stop", None)
        if stop is not None:
            await stop()


def build_avatar_provider(
    config: AgentConfig,
    *,
    allow_audio_only_fallback: bool,
) -> AvatarProvider:
    match config.avatar_mode:
        case "disabled":
            return DisabledAvatarProvider()
        case "lemonslice":
            return LemonSliceAvatarProvider(
                config,
                allow_audio_only_fallback=allow_audio_only_fallback,
            )
        case "echomimic":
            return build_echomimic_avatar_session(config)
        case _:
            raise ValueError(f"Unsupported MORGAN_AVATAR_MODE: {config.avatar_mode}")
