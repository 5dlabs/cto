from __future__ import annotations

import asyncio
import logging
import os

from dotenv import load_dotenv
from livekit import agents
from livekit.agents import (
    APIConnectOptions,
    Agent,
    AgentServer,
    AgentSession,
    JobContext,
    JobProcess,
    MetricsCollectedEvent,
    metrics,
    room_io,
)
from livekit.agents._exceptions import APIStatusError
from livekit.agents.voice.agent_session import SessionConnectOptions
from livekit.plugins import lemonslice, noise_cancellation, silero

from morgan_avatar_agent.config import AgentConfig
from morgan_avatar_agent.latency import LatencyRecorder
from morgan_avatar_agent.providers import build_llm, build_stt, build_tts, build_turn_detection

load_dotenv()

logger = logging.getLogger("morgan-avatar")
server = AgentServer()


def prewarm(proc: JobProcess) -> None:
    proc.userdata["vad"] = silero.VAD.load()


server.setup_fnc = prewarm


class MorganAgent(Agent):
    def __init__(self, config: AgentConfig) -> None:
        super().__init__(instructions=config.system_instructions)
        self._config = config


def _env_bool(name: str, default: bool) -> bool:
    value = os.getenv(name)
    if value is None:
        return default
    return value.strip().lower() in {"1", "true", "yes", "on"}


@server.rtc_session(agent_name="morgan-avatar")
async def entrypoint(ctx: JobContext) -> None:
    config = AgentConfig.from_env()
    config.validate()

    ctx.log_context_fields = {
        "room": ctx.room.name,
        "agent": config.agent_name,
        "stt_mode": config.stt_mode,
        "tts_mode": config.tts_mode,
        "llm_backend": config.llm_backend,
    }

    session = AgentSession(
        stt=build_stt(config),
        llm=build_llm(
            config,
            # Keep the OpenClaw user/session identifier stable for the life of
            # the room so the desktop app can target the same conversation with
            # supplemental pasted context.
            session_id=ctx.room.name,
        ),
        tts=build_tts(config),
        vad=ctx.proc.userdata["vad"],
        turn_detection=build_turn_detection(config),
        preemptive_generation=config.preemptive_generation,
        resume_false_interruption=config.resume_false_interruption,
        false_interruption_timeout=config.false_interruption_timeout,
        aec_warmup_duration=config.aec_warmup_duration,
        use_tts_aligned_transcript=config.use_tts_aligned_transcript,
        min_consecutive_speech_delay=0.0,
        conn_options=SessionConnectOptions(
            llm_conn_options=APIConnectOptions(
                max_retry=3,
                retry_interval=2.0,
                timeout=config.llm_read_timeout_seconds,
            ),
        ),
    )

    usage_collector = metrics.UsageCollector()
    latency = LatencyRecorder(
        config.latency_log_dir,
        ctx.room.name,
        config_snapshot=config.as_dict(),
    )

    @session.on("metrics_collected")
    def on_metrics_collected(event: MetricsCollectedEvent) -> None:
        metrics.log_metrics(event.metrics)
        usage_collector.collect(event.metrics)
        latency.handle_metrics(event.metrics)

    @session.on("user_state_changed")
    def on_user_state_changed(event) -> None:
        latency.handle_user_state(event)

    @session.on("user_input_transcribed")
    def on_user_input_transcribed(event) -> None:
        latency.handle_user_transcribed(event)

    @session.on("speech_created")
    def on_speech_created(event) -> None:
        latency.handle_speech_created(event)

    @session.on("conversation_item_added")
    def on_conversation_item_added(event) -> None:
        latency.handle_conversation_item(event)

    @session.on("agent_state_changed")
    def on_agent_state_changed(event) -> None:
        latency.handle_agent_state(event)

    @session.on("close")
    def on_close(event) -> None:
        latency.handle_close(getattr(event, "error", None))

    async def on_shutdown() -> None:
        logger.info("Usage summary: %s", usage_collector.get_summary())
        latency.write_summary()

    ctx.add_shutdown_callback(on_shutdown)

    allow_audio_only_fallback = _env_bool("MORGAN_ALLOW_AUDIO_ONLY_FALLBACK", True)

    if config.avatar_mode == "lemonslice":
        avatar_kwargs = {
            "agent_prompt": config.avatar_prompt,
            "idle_timeout": config.avatar_idle_timeout,
        }
        if config.has_lemonslice_agent_id:
            avatar_kwargs["agent_id"] = config.lemonslice_agent_id
        else:
            avatar_kwargs["agent_image_url"] = config.avatar_image_url

        avatar = lemonslice.AvatarSession(**avatar_kwargs)

        try:
            await avatar.start(session, room=ctx.room)
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
                if allow_audio_only_fallback and root.status_code == 402 and "insufficient funds" in body_text:
                    logger.warning(
                        "LemonSlice credits unavailable, continuing in audio-only mode "
                        "(set MORGAN_ALLOW_AUDIO_ONLY_FALLBACK=false to fail hard)"
                    )
                else:
                    raise
            else:
                logger.error("LemonSlice session start failed: %r", root)
                if not allow_audio_only_fallback:
                    raise
                logger.warning(
                    "Continuing in audio-only mode after LemonSlice start failure "
                    "(set MORGAN_ALLOW_AUDIO_ONLY_FALLBACK=false to fail hard)"
                )
    elif config.avatar_mode == "disabled":
        logger.info("MORGAN_AVATAR_MODE=disabled, running audio-only session")
    elif config.avatar_mode == "musetalk":
        logger.info("MORGAN_AVATAR_MODE=musetalk selected, video pipeline not wired yet, running audio-only fallback")
    else:
        raise ValueError(f"Unsupported MORGAN_AVATAR_MODE: {config.avatar_mode}")

    audio_input = room_io.AudioInputOptions(
        noise_cancellation=noise_cancellation.BVC() if config.use_noise_cancellation else None,
    )

    await session.start(
        room=ctx.room,
        agent=MorganAgent(config),
        room_options=room_io.RoomOptions(audio_input=audio_input),
    )

    # Give the browser a moment to subscribe to the remote tracks before the greeting starts.
    await asyncio.sleep(1.0)
    await session.say(config.greeting)


if __name__ == "__main__":
    agents.cli.run_app(server)
