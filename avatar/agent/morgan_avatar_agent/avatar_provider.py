from __future__ import annotations

import logging
from typing import Any, Protocol

from livekit.agents._exceptions import APIStatusError

from .config import AgentConfig
from .echomimic_avatar import build_echomimic_avatar_session
from .musetalk_avatar import MuseTalkAvatarSession
from .musetalk_inference import MuseTalkInferenceEngine

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


class MuseTalkProvider:
    def __init__(self, config: AgentConfig) -> None:
        self._config = config
        self._nats_client: Any | None = None
        self._session: MuseTalkAvatarSession | None = None

    async def start(self, session: Any, room: Any) -> None:
        logger.info(
            "MORGAN_AVATAR_MODE=musetalk selected, starting self-hosted avatar pipeline "
            "(use_stub=%s, nats_url=%s)",
            self._config.musetalk_use_stub,
            self._config.nats_url,
        )
        nats_client = None
        if not self._config.musetalk_use_stub:
            from .musetalk_nats_client import MuseTalkNatsClient, MuseTalkNatsError

            try:
                nats_client = MuseTalkNatsClient(
                    url=self._config.nats_url,
                    request_subject=self._config.nats_request_subject,
                    result_subject=self._config.nats_result_subject,
                    stream=self._config.nats_stream,
                    request_timeout_s=self._config.musetalk_request_timeout_s,
                )
                await nats_client.connect()
            except MuseTalkNatsError as exc:
                logger.warning(
                    "Failed to connect MuseTalk NATS client (%s); falling back to stub frames",
                    exc,
                )
                nats_client = None

        self._nats_client = nats_client
        self._session = MuseTalkAvatarSession(
            MuseTalkInferenceEngine(
                persona_id=self._config.persona_id,
                personas_root=self._config.personas_root,
                target_fps=self._config.musetalk_target_fps,
                frame_width=self._config.musetalk_frame_width,
                frame_height=self._config.musetalk_frame_height,
                nats_client=nats_client,
                reference_image_url=self._config.musetalk_reference_image_url,
                use_stub=self._config.musetalk_use_stub or nats_client is None,
            )
        )
        await self._session.start(session, room)

    async def stop(self) -> None:
        if self._session is not None:
            await self._session.stop()
        close = getattr(self._nats_client, "close", None)
        if close is not None:
            await close()


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
        case "musetalk":
            return MuseTalkProvider(config)
        case "echomimic":
            return build_echomimic_avatar_session(config)
        case _:
            raise ValueError(f"Unsupported MORGAN_AVATAR_MODE: {config.avatar_mode}")
