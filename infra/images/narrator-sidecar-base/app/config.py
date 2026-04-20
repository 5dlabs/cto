import os
from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    backend: str = "musetalk"  # musetalk | hunyuan
    port: int = 8081
    voice_id: str = "rex"
    llm_model: str = "gpt-4o-mini"
    openai_api_key: str = ""
    tts_model_path: str = "/models/tts"
    voice_sample_path: str = "/workspace/voice_clone_sample.mp3"
    raw_stream_path: str = ""  # $OPENCLAW_RAW_STREAM_PATH
    interrupt_path: str = "/workspace/.narrator/interrupt.jsonl"
    # Rolling window of ACP events fed to narrator LLM
    narrator_window_size: int = 20
    # Seconds between narrator LLM calls when stream is idle
    narrator_poll_interval: float = 3.0

    model_config = {"env_prefix": "", "case_sensitive": False}

    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        # Allow env override via OPENCLAW_RAW_STREAM_PATH
        if not self.raw_stream_path:
            self.raw_stream_path = os.environ.get("OPENCLAW_RAW_STREAM_PATH", "/workspace/acp-stream.ndjson")


settings = Settings()
