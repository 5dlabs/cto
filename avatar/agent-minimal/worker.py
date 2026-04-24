"""
morgan-avatar-minimal  —  pure audio publisher for animation pipeline validation.

No STT · No LLM · No TTS · No avatar session.  Reads a baked-in MP3 and streams
it as a LiveKit audio track so the client-side TalkingHead / HeadAudio worklet
can be exercised end-to-end.
"""
from __future__ import annotations

import asyncio
import logging
from pathlib import Path

from dotenv import load_dotenv
from livekit import agents, rtc
from livekit.agents import AgentServer, JobContext

load_dotenv()
logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(name)s: %(message)s")
logger = logging.getLogger("morgan-avatar-minimal")

SAMPLE_RATE = 48_000
NUM_CHANNELS = 1
SAMPLES_PER_FRAME = 2_400  # 50 ms @ 48 kHz

AUDIO_FILE = Path(__file__).parent / "sample.mp3"

server = AgentServer()


def _decode_mp3_to_pcm(path: Path) -> bytes:
    """Decode MP3 → 16-bit mono 48 kHz raw PCM (blocking, run in executor)."""
    from pydub import AudioSegment  # type: ignore[import-untyped]

    audio = (
        AudioSegment.from_mp3(str(path))
        .set_frame_rate(SAMPLE_RATE)
        .set_channels(NUM_CHANNELS)
        .set_sample_width(2)  # 16-bit LE
    )
    return bytes(audio.raw_data)


async def _stream_pcm(source: rtc.AudioSource, pcm: bytes) -> None:
    stride = SAMPLES_PER_FRAME * NUM_CHANNELS * 2  # bytes per frame
    total_frames = (len(pcm) + stride - 1) // stride
    logger.info("Streaming %d frames (%.1f s)", total_frames, len(pcm) / (SAMPLE_RATE * 2))

    for i, offset in enumerate(range(0, len(pcm), stride)):
        chunk = pcm[offset : offset + stride]
        if len(chunk) < stride:
            chunk = chunk + b"\x00" * (stride - len(chunk))
        frame = rtc.AudioFrame(
            data=chunk,
            sample_rate=SAMPLE_RATE,
            num_channels=NUM_CHANNELS,
            samples_per_channel=SAMPLES_PER_FRAME,
        )
        await source.capture_frame(frame)
        if i % 200 == 0:  # log progress every ~10 s
            logger.info("  … frame %d / %d", i, total_frames)
        await asyncio.sleep(0)  # yield to event loop


@server.rtc_session(agent_name="morgan-avatar")
async def entrypoint(ctx: JobContext) -> None:
    logger.info("job received — room=%s", ctx.room.name)

    await ctx.connect()
    logger.info("connected to room %s", ctx.room.name)

    loop = asyncio.get_event_loop()
    logger.info("decoding %s …", AUDIO_FILE)
    pcm = await loop.run_in_executor(None, _decode_mp3_to_pcm, AUDIO_FILE)
    duration_s = len(pcm) / (SAMPLE_RATE * NUM_CHANNELS * 2)
    logger.info("decoded %.1f s of audio (%d bytes)", duration_s, len(pcm))

    source = rtc.AudioSource(sample_rate=SAMPLE_RATE, num_channels=NUM_CHANNELS)
    track = rtc.LocalAudioTrack.create_audio_track("morgan-audio", source)
    options = rtc.TrackPublishOptions(source=rtc.TrackSource.SOURCE_MICROPHONE)
    pub = await ctx.room.local_participant.publish_track(track, options)
    logger.info("published audio track sid=%s name=%s", pub.sid, pub.name)

    await _stream_pcm(source, pcm)
    logger.info("playback complete — worker staying alive (room=%s)", ctx.room.name)

    while True:
        await asyncio.sleep(60)


if __name__ == "__main__":
    agents.cli.run_app(server)
