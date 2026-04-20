"""MuseTalk Test Harness - End-to-end avatar rendering test.

Sends a test render request via NATS and validates the video output.
"""

import asyncio
import json
import logging
import os
import sys
import time
import uuid

import nats
from nats.js.api import StreamConfig

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s %(levelname)s [%(name)s] %(message)s",
)
log = logging.getLogger("test-harness")


async def run_test():
    """Run end-to-end avatar rendering test."""
    nats_url = os.environ.get("NATS_URL", "nats://localhost:4222")
    stream_name = os.environ.get("NATS_STREAM", "AVATAR")
    subject = os.environ.get("NATS_SUBJECT", "avatar.render.request")
    result_subject = os.environ.get("NATS_RESULT_SUBJECT", "avatar.render.result")
    timeout = int(os.environ.get("TEST_TIMEOUT", "300"))

    reference_image_url = os.environ.get(
        "TEST_REFERENCE_IMAGE_URL",
        "https://raw.githubusercontent.com/TMElyralab/MuseTalk/main/data/video/yongen.mp4",
    )
    audio_url = os.environ.get(
        "TEST_AUDIO_URL",
        "https://raw.githubusercontent.com/TMElyralab/MuseTalk/main/data/audio/yongen.wav",
    )
    persona_id = os.environ.get("TEST_PERSONA_ID", "test-yongen")

    log.info("Connecting to NATS at %s", nats_url)
    nc = await nats.connect(nats_url)
    js = nc.jetstream()

    # Ensure stream exists. If the stream pre-exists with a different config
    # (e.g. created by the worker), we accept it as-is rather than trying to
    # reconcile — the worker owns stream provisioning.
    try:
        info = await js.stream_info(stream_name)
        log.info(
            "Stream %s already exists (subjects=%s)",
            stream_name,
            info.config.subjects,
        )
    except Exception:
        try:
            await js.add_stream(
                name=stream_name,
                subjects=[subject, result_subject],
                max_msgs=10000,
                max_bytes=1073741824,  # 1GB
            )
            log.info("Created stream %s", stream_name)
        except nats.js.errors.BadRequestError as e:
            # err_code 10058 = stream name already in use with different config
            if getattr(e, "err_code", None) == 10058:
                log.info(
                    "Stream %s already exists with different config; using as-is",
                    stream_name,
                )
            else:
                raise

    request_id = str(uuid.uuid4())
    request = {
        "request_id": request_id,
        "persona_id": persona_id,
        "reference_image_url": reference_image_url,
        "audio_url": audio_url,
        "audio_hash": "test-audio-hash",
        "fps": 25,
        "callback_subject": result_subject,
    }

    log.info("Sending render request: %s", request_id)
    log.info("Persona: %s", persona_id)
    log.info("Reference image: %s", reference_image_url)
    log.info("Audio: %s", audio_url)

    t0 = time.time()
    await js.publish(subject, json.dumps(request).encode())
    log.info("Request published in %.2fs", time.time() - t0)

    # Subscribe to result
    log.info("Waiting for result on %s (timeout: %ds)...", result_subject, timeout)

    result_received = asyncio.Event()
    result_data = None

    async def result_handler(msg):
        nonlocal result_data
        try:
            data = json.loads(msg.data.decode())
            if data.get("request_id") == request_id:
                result_data = data
                result_received.set()
                await msg.ack()
            else:
                await msg.nak()
        except Exception as e:
            log.error("Error handling result: %s", e)
            await msg.nak()

    sub = await js.subscribe(
        result_subject,
        durable="test-harness",
        cb=result_handler,
    )

    try:
        await asyncio.wait_for(result_received.wait(), timeout=timeout)
    except asyncio.TimeoutError:
        log.error("Timeout waiting for result after %ds", timeout)
        await sub.unsubscribe()
        await nc.drain()
        sys.exit(1)

    await sub.unsubscribe()
    await nc.drain()

    elapsed = time.time() - t0

    # Validate result
    if result_data.get("error"):
        log.error("Render failed: %s", result_data["error"])
        sys.exit(1)

    video_url = result_data.get("video_url")
    if not video_url:
        log.error("No video URL in result")
        sys.exit(1)

    render_time = result_data.get("render_time_s", 0)

    log.info("=" * 60)
    log.info("TEST PASSED!")
    log.info("=" * 60)
    log.info("Request ID: %s", request_id)
    log.info("Video URL: %s", video_url)
    log.info("Render time: %.2fs", render_time)
    log.info("Total time: %.2fs", elapsed)
    log.info("GPU: %s", result_data.get("gpu", "unknown"))
    log.info("Dtype: %s", result_data.get("dtype", "unknown"))
    log.info("Bootstrap only: %s", result_data.get("bootstrap_only", False))
    log.info("=" * 60)

    # Success
    return 0


if __name__ == "__main__":
    try:
        exit_code = asyncio.run(run_test())
        sys.exit(exit_code)
    except Exception as e:
        log.error("Test failed with exception: %s", e, exc_info=True)
        sys.exit(1)
