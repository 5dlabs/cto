"""aiortc WebRTC publisher: audio track fed by TTS output, placeholder video."""

import asyncio
import fractions
import logging
from typing import Optional

import av
import numpy as np
from aiortc import MediaStreamTrack, RTCPeerConnection, RTCSessionDescription
from scipy.signal import resample_poly

logger = logging.getLogger(__name__)

# WebRTC opus expects 48kHz; XTTS-v2 outputs 24kHz
_TTS_SAMPLE_RATE = 24_000
_RTC_SAMPLE_RATE = 48_000
_FRAME_SAMPLES = 960  # 20ms @ 48kHz


def _resample_24k_to_48k(audio: np.ndarray) -> np.ndarray:
    """Upsample float32 24kHz mono → 48kHz mono."""
    return resample_poly(audio, up=2, down=1).astype(np.float32)


def _float32_to_s16(audio: np.ndarray) -> bytes:
    clipped = np.clip(audio, -1.0, 1.0)
    return (clipped * 32767).astype(np.int16).tobytes()


class NarratorAudioTrack(MediaStreamTrack):
    """Audio track that drains a shared asyncio.Queue of numpy float32 frames."""

    kind = "audio"

    def __init__(self):
        super().__init__()
        self._queue: asyncio.Queue[np.ndarray] = asyncio.Queue(maxsize=64)
        self._timestamp = 0
        self._buffer = np.zeros(0, dtype=np.float32)

    async def push_audio(self, audio_48k: np.ndarray):
        """Called from the session worker after resampling."""
        await self._queue.put(audio_48k)

    async def recv(self) -> av.AudioFrame:
        # Fill _buffer until we have enough samples for one 20ms frame
        while len(self._buffer) < _FRAME_SAMPLES:
            try:
                chunk = self._queue.get_nowait()
            except asyncio.QueueEmpty:
                # Silence while nothing queued
                chunk = np.zeros(_FRAME_SAMPLES - len(self._buffer), dtype=np.float32)
            self._buffer = np.concatenate([self._buffer, chunk])

        pcm = self._buffer[:_FRAME_SAMPLES]
        self._buffer = self._buffer[_FRAME_SAMPLES:]

        frame = av.AudioFrame(format="s16", layout="mono", samples=_FRAME_SAMPLES)
        frame.planes[0].update(_float32_to_s16(pcm))
        frame.pts = self._timestamp
        frame.sample_rate = _RTC_SAMPLE_RATE
        frame.time_base = fractions.Fraction(1, _RTC_SAMPLE_RATE)
        self._timestamp += _FRAME_SAMPLES
        return frame


class WebRTCSession:
    def __init__(self):
        self.pc: Optional[RTCPeerConnection] = None
        self.audio_track = NarratorAudioTrack()
        self._closed = False

    async def handle_offer(self, sdp: str, sdp_type: str) -> dict:
        """Process SDP offer, return answer dict {sdp, type}."""
        if self.pc:
            await self.pc.close()

        self.pc = RTCPeerConnection()
        self.pc.addTrack(self.audio_track)

        @self.pc.on("connectionstatechange")
        async def on_state():
            logger.info("WebRTC state: %s", self.pc.connectionState)
            if self.pc.connectionState in ("failed", "closed"):
                self._closed = True

        offer = RTCSessionDescription(sdp=sdp, type=sdp_type)
        await self.pc.setRemoteDescription(offer)
        answer = await self.pc.createAnswer()
        await self.pc.setLocalDescription(answer)
        return {"sdp": self.pc.localDescription.sdp, "type": self.pc.localDescription.type}

    async def push_tts_audio(self, audio_24k: np.ndarray):
        """Resample TTS output and push to audio track."""
        audio_48k = _resample_24k_to_48k(audio_24k)
        await self.audio_track.push_audio(audio_48k)

    async def close(self):
        self._closed = True
        if self.pc:
            await self.pc.close()


# Backward compatibility exports
def create_audio_track() -> NarratorAudioTrack:
    """Create a new audio track for WebRTC."""
    return NarratorAudioTrack()


def create_video_track_placeholder():
    """Placeholder video track (not implemented for audio-only PoC)."""
    return None
