"""MuseTalk NATS worker — consumes render requests, produces avatar videos.

Message protocol (JSON on avatar.render.request):
{
  "request_id": "uuid",
  "persona_id": "morgan",
  "reference_image_url": "https://...",
  "audio_url": "https://...",
  "audio_hash": "sha256-of-audio",
  "fps": 25,
  "callback_subject": "avatar.render.result"
}

Result (JSON on avatar.render.result):
{
  "request_id": "uuid",
  "persona_id": "morgan",
  "video_url": "https://...",
  "render_time_s": 12.3,
  "cached": false,
  "error": null
}
"""

import asyncio
import json
import logging
import os
import signal
import tempfile
import time
from hashlib import sha256
from pathlib import Path

import aiohttp.web
import nats
from nats.js.api import ConsumerConfig, DeliverPolicy, AckPolicy
from prometheus_client import Counter, Histogram, Gauge, generate_latest

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s %(levelname)s [%(name)s] %(message)s",
)
log = logging.getLogger("worker")

# Prometheus metrics
RENDER_REQUESTS = Counter("musetalk_render_requests_total", "Total render requests", ["persona_id", "status"])
RENDER_DURATION = Histogram("musetalk_render_duration_seconds", "Render duration", buckets=[1, 5, 10, 30, 60, 120, 300])
GPU_MEMORY = Gauge("musetalk_gpu_memory_bytes", "GPU memory used")
QUEUE_DEPTH = Gauge("musetalk_queue_pending", "Pending messages in queue")

# Global state
_healthy = False
_ready = False
_shutdown = asyncio.Event()


async def download_file(url: str, dest: str):
    """Download a file from URL to local path."""
    import aiohttp
    async with aiohttp.ClientSession() as session:
        async with session.get(url) as resp:
            resp.raise_for_status()
            with open(dest, "wb") as f:
                async for chunk in resp.content.iter_chunked(8192):
                    f.write(chunk)


async def handle_message(msg, nc):
    """Process a single render request."""
    try:
        data = json.loads(msg.data.decode())
        request_id = data.get("request_id", "unknown")
        persona_id = data.get("persona_id", "unknown")
        log.info("Processing request %s for persona %s", request_id, persona_id)

        result_subject = data.get("callback_subject", os.environ.get("NATS_RESULT_SUBJECT", "avatar.render.result"))

        # Check render cache
        cache_enabled = os.environ.get("CACHE_ENABLED", "true").lower() == "true"
        audio_hash = data.get("audio_hash", "")
        cache_key = f"{persona_id}:{audio_hash}"

        # TODO: check S3/R2 cache for existing render

        with tempfile.TemporaryDirectory(dir="/tmp/renders") as tmpdir:
            # Download reference image and audio
            ref_path = os.path.join(tmpdir, "reference.png")
            audio_path = os.path.join(tmpdir, "audio.wav")
            output_path = os.path.join(tmpdir, "output.mp4")

            await download_file(data["reference_image_url"], ref_path)
            await download_file(data["audio_url"], audio_path)

            # Run render (CPU-bound, offload to thread)
            from render import render_avatar
            loop = asyncio.get_event_loop()
            t0 = time.time()
            result = await loop.run_in_executor(
                None,
                render_avatar,
                ref_path,
                audio_path,
                output_path,
                data.get("fps", 25),
            )
            duration = time.time() - t0

            RENDER_DURATION.observe(duration)
            RENDER_REQUESTS.labels(persona_id=persona_id, status="success").inc()

            # TODO: upload output.mp4 to S3/R2 and get signed URL
            video_url = f"file://{output_path}"  # placeholder

            response = {
                "request_id": request_id,
                "persona_id": persona_id,
                "video_url": video_url,
                "render_time_s": round(duration, 2),
                "cached": False,
                "error": None,
            }

        await nc.publish(result_subject, json.dumps(response).encode())
        await msg.ack()
        log.info("Completed request %s in %.1fs", request_id, duration)

    except Exception as e:
        log.error("Failed to process request: %s", e, exc_info=True)
        RENDER_REQUESTS.labels(persona_id=data.get("persona_id", "unknown"), status="error").inc()

        error_response = {
            "request_id": data.get("request_id", "unknown"),
            "persona_id": data.get("persona_id", "unknown"),
            "video_url": None,
            "render_time_s": 0,
            "cached": False,
            "error": str(e),
        }
        result_subject = data.get("callback_subject", os.environ.get("NATS_RESULT_SUBJECT", "avatar.render.result"))
        await nc.publish(result_subject, json.dumps(error_response).encode())
        await msg.nak()


async def run_worker():
    """Main NATS consumer loop."""
    global _healthy, _ready

    nats_url = os.environ.get("NATS_URL", "nats://localhost:4222")
    stream = os.environ.get("NATS_STREAM", "AVATAR")
    subject = os.environ.get("NATS_SUBJECT", "avatar.render.request")
    consumer_name = os.environ.get("NATS_CONSUMER", "musetalk-worker")
    queue = os.environ.get("NATS_QUEUE", "musetalk-workers")
    ack_wait_str = os.environ.get("NATS_ACK_WAIT", "5m")
    max_deliver = int(os.environ.get("NATS_MAX_DELIVER", "3"))

    # Parse ack_wait
    ack_wait = 300  # default 5 minutes
    if ack_wait_str.endswith("m"):
        ack_wait = int(ack_wait_str[:-1]) * 60
    elif ack_wait_str.endswith("s"):
        ack_wait = int(ack_wait_str[:-1])

    log.info("Connecting to NATS at %s", nats_url)
    nc = await nats.connect(nats_url)
    js = nc.jetstream()
    _healthy = True

    # Ensure stream exists
    try:
        await js.find_stream_name_by_subject(subject)
        log.info("Stream %s found for subject %s", stream, subject)
    except Exception:
        log.info("Creating stream %s for subject %s", stream, subject)
        await js.add_stream(name=stream, subjects=[subject, "avatar.render.result"])

    # Pre-load model
    log.info("Pre-loading MuseTalk model...")
    from render import load_model
    load_model()
    _ready = True
    log.info("Model loaded, worker ready")

    # Subscribe with durable consumer
    config = ConsumerConfig(
        durable_name=consumer_name,
        deliver_policy=DeliverPolicy.ALL,
        ack_policy=AckPolicy.EXPLICIT,
        ack_wait=ack_wait,
        max_deliver=max_deliver,
    )

    sub = await js.subscribe(subject, queue=queue, config=config)
    log.info("Subscribed to %s (queue=%s, consumer=%s)", subject, queue, consumer_name)

    try:
        while not _shutdown.is_set():
            try:
                msgs = await sub.fetch(1, timeout=5)
                for msg in msgs:
                    await handle_message(msg, nc)
            except nats.errors.TimeoutError:
                continue
    finally:
        await sub.unsubscribe()
        await nc.drain()
        log.info("Worker shut down")


async def health_handler(request):
    if _healthy:
        return aiohttp.web.Response(text="ok")
    return aiohttp.web.Response(status=503, text="not healthy")


async def ready_handler(request):
    if _ready:
        return aiohttp.web.Response(text="ready")
    return aiohttp.web.Response(status=503, text="not ready")


async def metrics_handler(request):
    # Update GPU memory gauge
    try:
        import torch
        if torch.cuda.is_available():
            GPU_MEMORY.set(torch.cuda.memory_allocated())
    except Exception:
        pass
    return aiohttp.web.Response(
        body=generate_latest(),
        content_type="text/plain; version=0.0.4",
    )


async def run_health_server():
    """HTTP server for liveness/readiness probes and metrics."""
    app = aiohttp.web.Application()
    app.router.add_get("/healthz", health_handler)
    app.router.add_get("/ready", ready_handler)
    app.router.add_get("/metrics", metrics_handler)

    runner = aiohttp.web.AppRunner(app)
    await runner.setup()
    site = aiohttp.web.TCPSite(runner, "0.0.0.0", 8080)
    await site.start()

    metrics_port = int(os.environ.get("METRICS_PORT", "9090"))
    metrics_app = aiohttp.web.Application()
    metrics_app.router.add_get("/metrics", metrics_handler)
    metrics_runner = aiohttp.web.AppRunner(metrics_app)
    await metrics_runner.setup()
    metrics_site = aiohttp.web.TCPSite(metrics_runner, "0.0.0.0", metrics_port)
    await metrics_site.start()

    log.info("Health server on :8080, metrics on :%d", metrics_port)


def handle_shutdown(signum, frame):
    log.info("Received signal %s, shutting down...", signum)
    _shutdown.set()


async def main():
    signal.signal(signal.SIGTERM, handle_shutdown)
    signal.signal(signal.SIGINT, handle_shutdown)

    await run_health_server()
    await run_worker()


if __name__ == "__main__":
    asyncio.run(main())
