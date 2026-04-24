from __future__ import annotations

import asyncio
import logging
import os

from dotenv import load_dotenv
from livekit import agents
from livekit.agents import (
    Agent,
    AgentServer,
    AgentSession,
    APIConnectOptions,
    JobContext,
    JobProcess,
    MetricsCollectedEvent,
    metrics,
    room_io,
)
from livekit.agents.voice.agent_session import SessionConnectOptions
from livekit.plugins import noise_cancellation, silero

from morgan_avatar_agent.avatar_provider import AvatarProvider, build_avatar_provider
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

    avatar_provider: AvatarProvider | None = None

    async def on_shutdown() -> None:
        if avatar_provider is not None:
            await avatar_provider.stop()
        logger.info("Usage summary: %s", usage_collector.get_summary())
        latency.write_summary()

    ctx.add_shutdown_callback(on_shutdown)

    allow_audio_only_fallback = _env_bool("MORGAN_ALLOW_AUDIO_ONLY_FALLBACK", True)
    avatar_provider = build_avatar_provider(
        config,
        allow_audio_only_fallback=allow_audio_only_fallback,
    )
    await avatar_provider.start(session, room=ctx.room)

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
