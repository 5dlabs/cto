"""
ElevenLabs client: STT (Scribe) + streaming TTS (Flash v2.5).

Deliberately thin. Real network calls are stubbed until we finalise the
voice ID and Scribe billing posture. Returning a clear placeholder lets the
WebSocket plumbing above be exercised end-to-end without an API key.
"""

from __future__ import annotations

import logging
from typing import AsyncIterator

import httpx

log = logging.getLogger("voice-bridge.eleven")

_API_BASE = "https://api.elevenlabs.io/v1"


class ElevenLabsClient:
    def __init__(self, api_key: str, voice_id: str) -> None:
        self._api_key = api_key
        self._voice_id = voice_id

    @property
    def is_configured(self) -> bool:
        return bool(self._api_key and self._voice_id)

    async def transcribe(
        self,
        audio_bytes: bytes,
        *,
        content_type: str = "audio/webm",
        filename: str = "turn.webm",
        language_code: str | None = None,
    ) -> str:
        """ElevenLabs Scribe STT.

        Sends a single turn's audio blob to
        `POST /v1/speech-to-text` as multipart/form-data with
        `model_id=scribe_v1`. Accepts any container ffmpeg can decode
        (opus-webm from MediaRecorder, pcm16 wav, mp3, etc.).
        """
        if not self._api_key:
            log.warning("transcribe(): no API key; returning empty transcript")
            return ""
        if not audio_bytes:
            return ""

        url = f"{_API_BASE}/speech-to-text"
        headers = {"xi-api-key": self._api_key, "accept": "application/json"}
        data: dict[str, str] = {"model_id": "scribe_v1"}
        if language_code:
            data["language_code"] = language_code
        files = {"file": (filename, audio_bytes, content_type)}

        try:
            async with httpx.AsyncClient(timeout=60.0) as client:
                resp = await client.post(url, headers=headers, data=data, files=files)
                if resp.status_code >= 400:
                    body = resp.text[:400]
                    log.warning("scribe %d: %s", resp.status_code, body)
                    return ""
                payload = resp.json()
                text = payload.get("text") or ""
                return text.strip()
        except httpx.HTTPError as exc:
            log.warning("scribe request failed: %s", exc)
            return ""

    async def stream_tts(self, text: str) -> AsyncIterator[bytes]:
        """ElevenLabs Flash v2.5 streaming TTS → MP3 chunks."""
        if not self.is_configured or not text:
            return
        url = f"{_API_BASE}/text-to-speech/{self._voice_id}/stream"
        headers = {
            "xi-api-key": self._api_key,
            "accept": "audio/mpeg",
            "content-type": "application/json",
        }
        payload = {
            "text": text,
            "model_id": "eleven_flash_v2_5",
            "output_format": "mp3_22050_32",
        }
        async with httpx.AsyncClient(timeout=60.0) as client:
            async with client.stream("POST", url, headers=headers, json=payload) as resp:
                resp.raise_for_status()
                async for chunk in resp.aiter_bytes():
                    if chunk:
                        yield chunk
