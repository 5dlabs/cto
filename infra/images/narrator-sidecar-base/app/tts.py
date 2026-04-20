"""XTTS-v2 TTS wrapper — CPU-only, Rex voice cloned from sample."""

import asyncio
import io
import logging
import os
import tempfile
from functools import lru_cache

import numpy as np
import soundfile as sf

logger = logging.getLogger(__name__)

_TTS_MODEL_NAME = "tts_models/multilingual/multi-dataset/xtts_v2"


@lru_cache(maxsize=1)
def _load_tts(model_path: str, voice_sample: str):
    """Load TTS model once; cached for the process lifetime."""
    from TTS.api import TTS  # import deferred — slow to load

    logger.info("Loading XTTS-v2 model (this takes ~30s on CPU)…")
    tts = TTS(
        model_name=_TTS_MODEL_NAME,
        progress_bar=False,
        gpu=False,
    )
    logger.info("XTTS-v2 ready.")
    return tts


class TTSEngine:
    def __init__(self, model_path: str, voice_sample: str):
        self._model_path = model_path
        self._voice_sample = voice_sample
        self._tts = None
        self._lock = asyncio.Lock()
        self._ready = False

    async def preload(self):
        """Warm up the model in a thread so startup doesn't block the event loop."""
        async with self._lock:
            if self._ready:
                return
            loop = asyncio.get_event_loop()
            await loop.run_in_executor(
                None, _load_tts, self._model_path, self._voice_sample
            )
            self._tts = _load_tts(self._model_path, self._voice_sample)
            self._ready = True

    @property
    def ready(self) -> bool:
        return self._ready

    async def synthesize(self, text: str) -> np.ndarray:
        """
        Synthesize `text` using the Rex voice sample.
        Returns float32 PCM audio at 24 kHz (XTTS-v2 native rate).
        """
        if not self._ready:
            raise RuntimeError("TTS engine not ready; call preload() first")

        if not os.path.exists(self._voice_sample):
            raise FileNotFoundError(f"Voice sample not found: {self._voice_sample}")

        loop = asyncio.get_event_loop()

        def _synth():
            with tempfile.NamedTemporaryFile(suffix=".wav", delete=False) as tmp:
                tmp_path = tmp.name
            try:
                self._tts.tts_to_file(
                    text=text,
                    speaker_wav=self._voice_sample,
                    language="en",
                    file_path=tmp_path,
                )
                audio, sr = sf.read(tmp_path, dtype="float32")
            finally:
                os.unlink(tmp_path)
            return audio, sr

        audio, sr = await loop.run_in_executor(None, _synth)
        return audio  # float32 @ 24kHz mono
