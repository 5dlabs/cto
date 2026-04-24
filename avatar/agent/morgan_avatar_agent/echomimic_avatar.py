from __future__ import annotations

import asyncio
import logging
import os
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import httpx

from .musetalk_avatar import MuseTalkAvatarSession
from .musetalk_inference import MuseTalkInferenceEngine

logger = logging.getLogger(__name__)


@dataclass(frozen=True)
class EchoMimicRenderOptions:
    app_url: str
    prompt: str
    source_image_path: Path
    timeout_s: float
    video_length: int | None = None
    sample_height: int | None = None
    sample_width: int | None = None
    weight_dtype: str | None = None
    auto_render: bool = True
    tts_mode: str = "elevenlabs"
    eleven_voice_id: str = ""
    eleven_model: str = "eleven_flash_v2_5"


@dataclass(frozen=True)
class EchoMimicRenderResult:
    video_bytes: bytes
    content_type: str
    elapsed_s: str | None
    job_id: str | None


class EchoMimicAvatarSession:
    """OpenClaw/LiveKit adapter for the batch EchoMimic renderer.

    EchoMimic currently returns a complete MP4 after inference, so this session
    starts a visible idle LiveKit video track immediately and exposes
    ``render_utterance`` as the batch render hook. When auto-render is enabled,
    finalized assistant turns are synthesized to audio, rendered through
    EchoMimic, decoded, and published onto the LiveKit video source.
    """

    def __init__(
        self,
        options: EchoMimicRenderOptions,
        *,
        idle_session: MuseTalkAvatarSession,
    ) -> None:
        self._options = options
        self._idle_session = idle_session
        self._render_tasks: set[asyncio.Task[None]] = set()
        self._playback_lock = asyncio.Lock()

    async def start(self, session: Any, room: Any) -> None:
        session._echomimic_app_url = self._options.app_url
        session._echomimic_mode = "batch-mp4"
        logger.info(
            "echomimic.avatar.start app_url=%s source=%s",
            self._options.app_url,
            self._options.source_image_path,
        )
        await self._idle_session.start(session, room)
        if self._options.auto_render:
            self._attach_turn_renderer(session)

    async def stop(self) -> None:
        for task in self._render_tasks:
            task.cancel()
        if self._render_tasks:
            await asyncio.gather(*self._render_tasks, return_exceptions=True)
            self._render_tasks.clear()
        await self._idle_session.stop()

    async def render_utterance(
        self,
        audio_path: str | Path,
        *,
        publish: bool = True,
    ) -> EchoMimicRenderResult:
        source_path = self._options.source_image_path
        audio = Path(audio_path)
        if not source_path.exists():
            raise FileNotFoundError(f"EchoMimic source image does not exist: {source_path}")
        if not audio.exists():
            raise FileNotFoundError(f"EchoMimic audio does not exist: {audio}")

        form_data = {
            "prompt": self._options.prompt,
            **self._optional_form_fields(),
        }
        endpoint = self._options.app_url.rstrip("/") + "/animate"
        logger.info("echomimic.render.start endpoint=%s audio=%s", endpoint, audio)

        async with httpx.AsyncClient(timeout=self._options.timeout_s) as client:
            with source_path.open("rb") as source_file, audio.open("rb") as audio_file:
                response = await client.post(
                    endpoint,
                    data=form_data,
                    files={
                        "source": (source_path.name, source_file, "image/jpeg"),
                        "audio": (audio.name, audio_file, "audio/mpeg"),
                    },
                )
        response.raise_for_status()
        logger.info(
            "echomimic.render.ok job_id=%s elapsed_s=%s bytes=%d",
            response.headers.get("x-echomimic-job-id"),
            response.headers.get("x-echomimic-elapsed-s"),
            len(response.content),
        )
        result = EchoMimicRenderResult(
            video_bytes=response.content,
            content_type=response.headers.get("content-type", "video/mp4"),
            elapsed_s=response.headers.get("x-echomimic-elapsed-s"),
            job_id=response.headers.get("x-echomimic-job-id"),
        )
        if publish:
            await self._publish_mp4_bytes(result.video_bytes)
        return result

    def _optional_form_fields(self) -> dict[str, str]:
        values: dict[str, str] = {}
        if self._options.video_length is not None:
            values["video_length"] = str(self._options.video_length)
        if self._options.sample_height is not None:
            values["sample_height"] = str(self._options.sample_height)
        if self._options.sample_width is not None:
            values["sample_width"] = str(self._options.sample_width)
        if self._options.weight_dtype:
            values["weight_dtype"] = self._options.weight_dtype
        return values

    def _attach_turn_renderer(self, session: Any) -> None:
        @session.on("conversation_item_added")
        def on_conversation_item_added(event: Any) -> None:
            text = self._assistant_text_from_event(event)
            if not text:
                return
            task = asyncio.create_task(self._render_assistant_text(text))
            self._render_tasks.add(task)
            task.add_done_callback(self._on_render_task_done)

    def _on_render_task_done(self, task: asyncio.Task[None]) -> None:
        self._render_tasks.discard(task)
        if task.cancelled():
            return
        if exc := task.exception():
            logger.error(
                "echomimic.turn.render_failed",
                exc_info=(type(exc), exc, exc.__traceback__),
            )

    async def _render_assistant_text(self, text: str) -> None:
        if self._options.tts_mode != "elevenlabs":
            logger.warning(
                "echomimic.turn.skip unsupported_tts_mode=%s",
                self._options.tts_mode,
            )
            return

        api_key = os.getenv("ELEVENLABS_API_KEY", "").strip()
        if not api_key:
            logger.warning("echomimic.turn.skip missing_elevenlabs_api_key")
            return

        with tempfile.TemporaryDirectory(prefix="echomimic-turn-") as tmpdir:
            audio_path = Path(tmpdir) / "turn.mp3"
            await self._synthesize_elevenlabs_audio(text, audio_path, api_key=api_key)
            await self.render_utterance(audio_path, publish=True)

    async def _synthesize_elevenlabs_audio(
        self,
        text: str,
        audio_path: Path,
        *,
        api_key: str,
    ) -> None:
        endpoint = f"https://api.elevenlabs.io/v1/text-to-speech/{self._options.eleven_voice_id}"
        payload = {
            "text": text,
            "model_id": self._options.eleven_model,
        }
        headers = {
            "accept": "audio/mpeg",
            "content-type": "application/json",
            "xi-api-key": api_key,
        }
        async with httpx.AsyncClient(timeout=60.0) as client:
            response = await client.post(
                endpoint,
                params={"output_format": "mp3_44100_128"},
                json=payload,
                headers=headers,
            )
        response.raise_for_status()
        audio_path.write_bytes(response.content)

    async def _publish_mp4_bytes(self, video_bytes: bytes) -> int:
        with tempfile.NamedTemporaryFile(suffix=".mp4") as mp4:
            mp4.write(video_bytes)
            mp4.flush()
            return await self._publish_mp4_path(Path(mp4.name))

    async def _publish_mp4_path(self, video_path: Path) -> int:
        try:
            import cv2  # type: ignore[import-not-found]
        except ImportError:
            logger.exception("echomimic.video.decode_unavailable")
            return 0

        async with self._playback_lock:
            self._idle_session.set_external_video_active(True)
            capture = cv2.VideoCapture(str(video_path))
            try:
                if not capture.isOpened():
                    logger.error("echomimic.video.open_failed path=%s", video_path)
                    return 0

                fps = capture.get(cv2.CAP_PROP_FPS) or 25.0
                frame_interval = 1.0 / max(float(fps), 1.0)
                frames_pushed = 0
                while True:
                    ok, frame = capture.read()
                    if not ok:
                        break
                    rgba = cv2.cvtColor(frame, cv2.COLOR_BGR2RGBA)
                    height, width = rgba.shape[:2]
                    self._idle_session.push_rgba_frame(
                        width=width,
                        height=height,
                        rgba=rgba.tobytes(),
                        index=frames_pushed,
                        timestamp_ms=frames_pushed * frame_interval * 1000,
                    )
                    frames_pushed += 1
                    await asyncio.sleep(frame_interval)
                logger.info("echomimic.video.published frames=%d", frames_pushed)
                return frames_pushed
            finally:
                capture.release()
                self._idle_session.set_external_video_active(False)

    @staticmethod
    def _assistant_text_from_event(event: Any) -> str:
        item = getattr(event, "item", None)
        if getattr(item, "role", None) != "assistant":
            return ""
        if getattr(item, "interrupted", False):
            return ""
        parts: list[str] = []
        for content in getattr(item, "content", []):
            if isinstance(content, str):
                parts.append(content)
            elif hasattr(content, "text"):
                parts.append(str(content.text))
        return " ".join(part.strip() for part in parts if part.strip())


def build_echomimic_avatar_session(config: Any) -> EchoMimicAvatarSession:
    idle_session = MuseTalkAvatarSession(
        MuseTalkInferenceEngine(
            persona_id=config.persona_id,
            personas_root=config.personas_root,
            target_fps=config.musetalk_target_fps,
            frame_width=config.musetalk_frame_width,
            frame_height=config.musetalk_frame_height,
            reference_image_url=config.avatar_image_url,
            use_stub=True,
        )
    )
    return EchoMimicAvatarSession(
        EchoMimicRenderOptions(
            app_url=config.echomimic_app_url,
            prompt=config.echomimic_prompt,
            source_image_path=config.echomimic_source_image_path,
            timeout_s=config.echomimic_request_timeout_s,
            video_length=config.echomimic_video_length,
            sample_height=config.echomimic_sample_height,
            sample_width=config.echomimic_sample_width,
            weight_dtype=config.echomimic_weight_dtype,
            auto_render=config.echomimic_auto_render,
            tts_mode=config.tts_mode,
            eleven_voice_id=config.eleven_voice_id,
            eleven_model=config.eleven_model,
        ),
        idle_session=idle_session,
    )
