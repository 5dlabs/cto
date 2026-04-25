from __future__ import annotations

import httpx
from livekit.agents import inference
from livekit.plugins import cartesia, deepgram, elevenlabs, openai

from .config import AgentConfig


def _openai_compatible_base_url(base_url: str) -> str:
    base = base_url.rstrip("/")
    if base.endswith("/v1"):
        return base
    return base + "/v1"


def build_llm(config: AgentConfig, *, session_id: str):
    if config.llm_backend == "inference":
        return inference.LLM(model=config.llm_model)

    base_url = _openai_compatible_base_url(config.llm_base_url)
    extra_headers = {}
    if config.llm_agent_id:
        extra_headers["x-openclaw-agent-id"] = config.llm_agent_id

    return openai.LLM(
        model=config.llm_model,
        base_url=base_url,
        api_key=config.llm_api_key,
        extra_headers=extra_headers,
        user=f"{config.llm_user_prefix}:{session_id}",
        timeout=httpx.Timeout(
            connect=config.llm_connect_timeout_seconds,
            read=config.llm_read_timeout_seconds,
            write=config.llm_write_timeout_seconds,
            pool=config.llm_pool_timeout_seconds,
        ),
    )


def build_stt(config: AgentConfig):
    match config.stt_mode:
        case "openai-realtime":
            return openai.STT(
                model="gpt-4o-mini-transcribe",
                language=config.stt_language,
                use_realtime=True,
            )
        case "openai-transcribe":
            return openai.STT(
                model="gpt-4o-mini-transcribe",
                language=config.stt_language,
                use_realtime=False,
            )
        case "deepgram-flux":
            return deepgram.STTv2(
                model=config.deepgram_flux_model,
                eager_eot_threshold=config.deepgram_eager_eot_threshold,
                eot_threshold=config.deepgram_eot_threshold,
                eot_timeout_ms=config.deepgram_eot_timeout_ms,
                keyterms=config.deepgram_keyterms,
            )
        case "deepgram-nova":
            return deepgram.STT(
                model=config.deepgram_nova_model,
                language=config.stt_language,
                endpointing_ms=config.deepgram_endpointing_ms,
                keyterms=config.deepgram_keyterms,
            )
        case "livekit-nova":
            return inference.STT(model="deepgram/nova-3", language=config.stt_language)
        case _:
            return inference.STT(model="deepgram/flux-general", language=config.stt_language)


def build_turn_detection(config: AgentConfig):
    if "flux" in config.stt_mode or config.stt_mode == "openai-realtime":
        return "stt"

    # The multilingual turn detector loads a local ONNX model. Production Flux
    # STT uses provider-side endpointing, so keep this import off the startup
    # path unless a non-Flux STT mode explicitly needs the local detector.
    from livekit.plugins.turn_detector.multilingual import MultilingualModel

    return MultilingualModel()


def build_tts(config: AgentConfig):
    match config.tts_mode:
        case "cartesia":
            return cartesia.TTS(
                model=config.cartesia_model,
                voice=config.cartesia_voice_id,
                speed=config.cartesia_speed,
                language=config.tts_language,
            )
        case "livekit-elevenlabs":
            return inference.TTS(
                model=f"elevenlabs/{config.eleven_model}",
                voice=config.eleven_voice_id,
                language=config.tts_language,
            )
        case "livekit-cartesia":
            return inference.TTS(
                model=f"cartesia/{config.cartesia_model}",
                voice=config.cartesia_voice_id,
                language=config.tts_language,
            )
        case _:
            return elevenlabs.TTS(
                voice_id=config.eleven_voice_id,
                model=config.eleven_model,
                streaming_latency=config.eleven_streaming_latency,
                chunk_length_schedule=config.eleven_chunk_length_schedule,
                language=config.tts_language,
            )
