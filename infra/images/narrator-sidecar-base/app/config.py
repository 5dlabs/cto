import os
from pydantic import BaseModel


class Settings(BaseModel):
    backend: str = os.environ.get("BACKEND", "musetalk")  # musetalk | hunyuan
    port: int = int(os.environ.get("PORT", "8081"))
    voice_id: str = os.environ.get("VOICE_ID", "blaze")
    llm_model: str = os.environ.get("LLM_MODEL", "gpt-4o-mini")
    openai_api_key: str = os.environ.get("OPENAI_API_KEY", "")
    tts_model_path: str = os.environ.get("TTS_MODEL_PATH", "/models/tts")
    voice_sample_path: str = os.environ.get("VOICE_SAMPLE_PATH", "/workspace/voice_clone_sample.mp3")
    raw_stream_path: str = os.environ.get("OPENCLAW_RAW_STREAM_PATH", "/workspace/.openclaw/logs/raw-stream.jsonl")
    interrupt_path: str = os.environ.get("INTERRUPT_PATH", "/workspace/.narrator/interrupt.jsonl")
    # Lipsync RPC (NATS) — configurable so musetalk and hunyuan sidecars can target
    # different brokers / subjects when co-deployed in the same pod.
    lipsync_nats_url: str = os.environ.get(
        "LIPSYNC_NATS_URL", "nats://nats.messaging.svc.cluster.local:4222"
    )
    lipsync_subject: str = os.environ.get("LIPSYNC_SUBJECT", "avatar.render.request")
    lipsync_result_subject: str = os.environ.get(
        "LIPSYNC_RESULT_SUBJECT", "avatar.render.result"
    )
    # Rolling window of ACP events fed to narrator LLM
    narrator_window_size: int = 20
    # Seconds between narrator LLM calls when stream is idle
    narrator_poll_interval: float = 3.0


settings = Settings()
