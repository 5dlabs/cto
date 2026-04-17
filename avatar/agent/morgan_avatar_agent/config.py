from __future__ import annotations

import os
from dataclasses import dataclass
from pathlib import Path


def _env_bool(name: str, default: bool) -> bool:
    value = os.getenv(name)
    if value is None:
        return default
    return value.strip().lower() in {"1", "true", "yes", "on"}


def _env_float(name: str, default: float) -> float:
    value = os.getenv(name)
    if value is None:
        return default
    return float(value)


def _env_int(name: str, default: int) -> int:
    value = os.getenv(name)
    if value is None:
        return default
    return int(value)


def _env_list(name: str) -> list[str]:
    value = os.getenv(name, "")
    if not value.strip():
        return []
    return [item.strip() for item in value.split(",") if item.strip()]


@dataclass(frozen=True)
class AgentConfig:
    agent_name: str
    greeting: str
    system_instructions: str
    avatar_prompt: str
    avatar_mode: str
    lemonslice_agent_id: str
    image_url: str
    placeholder_image_url: str
    use_placeholder_image: bool
    avatar_idle_timeout: int
    use_noise_cancellation: bool
    latency_log_dir: Path
    llm_backend: str
    llm_model: str
    llm_api_key: str
    llm_base_url: str
    llm_agent_id: str
    llm_user_prefix: str
    llm_connect_timeout_seconds: float
    llm_read_timeout_seconds: float
    llm_write_timeout_seconds: float
    llm_pool_timeout_seconds: float
    stt_mode: str
    stt_language: str
    deepgram_flux_model: str
    deepgram_nova_model: str
    deepgram_endpointing_ms: int
    deepgram_eager_eot_threshold: float
    deepgram_eot_threshold: float
    deepgram_eot_timeout_ms: int
    deepgram_keyterms: list[str]
    tts_mode: str
    tts_language: str
    eleven_voice_id: str
    eleven_model: str
    eleven_streaming_latency: int
    eleven_chunk_length_schedule: list[int]
    cartesia_voice_id: str
    cartesia_model: str
    cartesia_speed: float
    preemptive_generation: bool
    resume_false_interruption: bool
    false_interruption_timeout: float
    aec_warmup_duration: float
    use_tts_aligned_transcript: bool

    @classmethod
    def from_env(cls, project_root: Path | None = None) -> AgentConfig:
        root = project_root or Path(__file__).resolve().parents[2]
        llm_backend = os.getenv("MORGAN_LLM_BACKEND", "openclaw").strip().lower()
        default_preemptive_generation = llm_backend != "openclaw"
        return cls(
            agent_name=os.getenv("MORGAN_AGENT_NAME", "morgan-avatar"),
            greeting=os.getenv(
                "MORGAN_GREETING",
                "Hi, I am Morgan. What are you trying to get done today?",
            ),
            system_instructions=os.getenv(
                "MORGAN_SYSTEM_INSTRUCTIONS",
                (
                    "You are Morgan, a crisp and practical program manager speaking with a CTO. "
                    "Be concise, clear, and collaborative. Keep answers speech-friendly. "
                    "Do not use markdown, bullets, emojis, or special formatting "
                    "in spoken responses. "
                    "Lead with the answer, then ask the next useful question when needed."
                ),
            ),
            avatar_prompt=os.getenv(
                "MORGAN_AVATAR_PROMPT",
                (
                    "A confident, thoughtful program manager speaking naturally "
                    "with calm facial motions."
                ),
            ),
            avatar_mode=os.getenv("MORGAN_AVATAR_MODE", "lemonslice").strip().lower(),
            lemonslice_agent_id=os.getenv("MORGAN_LEMONSLICE_AGENT_ID", "").strip(),
            image_url=os.getenv("MORGAN_IMAGE_URL", "").strip(),
            placeholder_image_url=os.getenv("MORGAN_PLACEHOLDER_IMAGE_URL", "").strip(),
            use_placeholder_image=_env_bool("MORGAN_USE_PLACEHOLDER_IMAGE", False),
            avatar_idle_timeout=_env_int("MORGAN_AVATAR_IDLE_TIMEOUT_SECONDS", 60),
            use_noise_cancellation=_env_bool("MORGAN_USE_NOISE_CANCELLATION", True),
            latency_log_dir=Path(os.getenv("MORGAN_LATENCY_LOG_DIR", root / "agent" / "runs")),
            llm_backend=llm_backend,
            llm_model=os.getenv("MORGAN_LLM_MODEL", "openclaw:main").strip(),
            llm_api_key=(
                os.getenv("MORGAN_LLM_API_KEY")
                or os.getenv("OPENCLAW_TOKEN")
                or os.getenv("OPENAI_API_KEY")
                or ""
            ).strip(),
            llm_base_url=(
                os.getenv("MORGAN_LLM_BASE_URL") or os.getenv("OPENCLAW_GATEWAY_URL") or ""
            ).strip(),
            llm_agent_id=os.getenv("MORGAN_LLM_AGENT_ID", "morgan").strip(),
            llm_user_prefix=os.getenv("MORGAN_LLM_USER_PREFIX", "morgan-avatar").strip(),
            llm_connect_timeout_seconds=_env_float("MORGAN_LLM_CONNECT_TIMEOUT_SECONDS", 10.0),
            llm_read_timeout_seconds=_env_float("MORGAN_LLM_READ_TIMEOUT_SECONDS", 120.0),
            llm_write_timeout_seconds=_env_float("MORGAN_LLM_WRITE_TIMEOUT_SECONDS", 10.0),
            llm_pool_timeout_seconds=_env_float("MORGAN_LLM_POOL_TIMEOUT_SECONDS", 10.0),
            stt_mode=os.getenv("MORGAN_STT_MODE", "livekit-flux").strip().lower(),
            stt_language=os.getenv("MORGAN_STT_LANGUAGE", "en").strip(),
            deepgram_flux_model=os.getenv("MORGAN_DEEPGRAM_FLUX_MODEL", "flux-general-en").strip(),
            deepgram_nova_model=os.getenv("MORGAN_DEEPGRAM_NOVA_MODEL", "nova-3").strip(),
            deepgram_endpointing_ms=_env_int("MORGAN_DEEPGRAM_ENDPOINTING_MS", 25),
            deepgram_eager_eot_threshold=_env_float(
                "MORGAN_DEEPGRAM_EAGER_EOT_THRESHOLD", 0.4
            ),
            deepgram_eot_threshold=_env_float("MORGAN_DEEPGRAM_EOT_THRESHOLD", 0.7),
            deepgram_eot_timeout_ms=_env_int("MORGAN_DEEPGRAM_EOT_TIMEOUT_MS", 1500),
            deepgram_keyterms=_env_list("MORGAN_DEEPGRAM_KEYTERMS"),
            tts_mode=os.getenv("MORGAN_TTS_MODE", "elevenlabs").strip().lower(),
            tts_language=os.getenv("MORGAN_TTS_LANGUAGE", "en").strip(),
            eleven_voice_id=os.getenv("MORGAN_ELEVEN_VOICE_ID", "iP95p4xoKVk53GoZ742B").strip(),
            eleven_model=os.getenv("MORGAN_ELEVEN_MODEL", "eleven_flash_v2_5").strip(),
            eleven_streaming_latency=_env_int("MORGAN_ELEVEN_STREAMING_LATENCY", 3),
            eleven_chunk_length_schedule=[
                int(value)
                for value in os.getenv("MORGAN_ELEVEN_CHUNK_LENGTH_SCHEDULE", "80,120,200,260")
                .split(",")
                if value.strip()
            ],
            cartesia_voice_id=os.getenv(
                "MORGAN_CARTESIA_VOICE_ID", "9626c31c-bec5-4cca-baa8-f8ba9e84c8bc"
            ).strip(),
            cartesia_model=os.getenv("MORGAN_CARTESIA_MODEL", "sonic-turbo").strip(),
            cartesia_speed=_env_float("MORGAN_CARTESIA_SPEED", 1.0),
            preemptive_generation=_env_bool(
                "MORGAN_PREEMPTIVE_GENERATION", default_preemptive_generation
            ),
            resume_false_interruption=_env_bool("MORGAN_RESUME_FALSE_INTERRUPTION", True),
            false_interruption_timeout=_env_float("MORGAN_FALSE_INTERRUPTION_TIMEOUT", 1.0),
            aec_warmup_duration=_env_float("MORGAN_AEC_WARMUP_DURATION", 3.0),
            use_tts_aligned_transcript=_env_bool("MORGAN_USE_TTS_ALIGNED_TRANSCRIPT", True),
        )

    @property
    def avatar_image_url(self) -> str:
        if self.use_placeholder_image and self.placeholder_image_url:
            return self.placeholder_image_url
        return self.image_url or self.placeholder_image_url

    @property
    def has_lemonslice_agent_id(self) -> bool:
        return bool(self.lemonslice_agent_id)

    def validate(self) -> None:
        if self.avatar_mode not in {"lemonslice", "disabled", "musetalk"}:
            raise ValueError(
                "MORGAN_AVATAR_MODE must be one of: lemonslice, disabled, musetalk."
            )

        if self.avatar_mode == "lemonslice" and not self.has_lemonslice_agent_id and not self.avatar_image_url:
            raise ValueError(
                "Set MORGAN_LEMONSLICE_AGENT_ID or MORGAN_IMAGE_URL / "
                "MORGAN_PLACEHOLDER_IMAGE_URL so LemonSlice can render the avatar."
            )

        if self.llm_backend == "openclaw":
            if not self.llm_base_url:
                raise ValueError(
                    "Set MORGAN_LLM_BASE_URL or OPENCLAW_GATEWAY_URL for the "
                    "OpenClaw-compatible LLM."
                )
            if not self.llm_api_key:
                raise ValueError(
                    "Set MORGAN_LLM_API_KEY or OPENCLAW_TOKEN for the OpenClaw-compatible LLM."
                )

    def as_dict(self) -> dict[str, str | int | float | bool | list[int] | list[str]]:
        return {
            "agent_name": self.agent_name,
            "avatar_mode": self.avatar_mode,
            "llm_backend": self.llm_backend,
            "llm_model": self.llm_model,
            "stt_mode": self.stt_mode,
            "stt_language": self.stt_language,
            "tts_mode": self.tts_mode,
            "tts_language": self.tts_language,
            "use_placeholder_image": self.use_placeholder_image,
            "avatar_idle_timeout": self.avatar_idle_timeout,
            "use_noise_cancellation": self.use_noise_cancellation,
            "preemptive_generation": self.preemptive_generation,
            "resume_false_interruption": self.resume_false_interruption,
            "false_interruption_timeout": self.false_interruption_timeout,
            "aec_warmup_duration": self.aec_warmup_duration,
            "use_tts_aligned_transcript": self.use_tts_aligned_transcript,
            "deepgram_keyterms": self.deepgram_keyterms,
            "eleven_chunk_length_schedule": self.eleven_chunk_length_schedule,
            "has_lemonslice_agent_id": self.has_lemonslice_agent_id,
        }
