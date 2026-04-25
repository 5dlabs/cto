from __future__ import annotations

import sys
from pathlib import Path

from morgan_avatar_agent.config import AgentConfig
from morgan_avatar_agent.providers import (
    _openai_compatible_base_url,
    build_stt,
    build_turn_detection,
)


def test_placeholder_image_is_preferred_when_enabled(monkeypatch) -> None:
    monkeypatch.setenv("MORGAN_USE_PLACEHOLDER_IMAGE", "true")
    monkeypatch.setenv("MORGAN_IMAGE_URL", "https://example.com/real.png")
    monkeypatch.setenv("MORGAN_PLACEHOLDER_IMAGE_URL", "https://example.com/placeholder.png")
    monkeypatch.setenv("MORGAN_LLM_BACKEND", "inference")

    config = AgentConfig.from_env(project_root=Path("/tmp/project"))

    assert config.avatar_image_url == "https://example.com/placeholder.png"


def test_openclaw_validation_requires_base_url_and_api_key(monkeypatch) -> None:
    monkeypatch.delenv("MORGAN_LLM_BASE_URL", raising=False)
    monkeypatch.delenv("MORGAN_LLM_API_KEY", raising=False)
    monkeypatch.delenv("OPENCLAW_GATEWAY_URL", raising=False)
    monkeypatch.delenv("OPENCLAW_TOKEN", raising=False)
    monkeypatch.setenv("MORGAN_LLM_BACKEND", "openclaw")
    monkeypatch.setenv("MORGAN_PLACEHOLDER_IMAGE_URL", "https://example.com/placeholder.png")

    config = AgentConfig.from_env(project_root=Path("/tmp/project"))

    try:
        config.validate()
    except ValueError as exc:
        assert "MORGAN_LLM_BASE_URL" in str(exc)
    else:
        raise AssertionError("Expected validation to fail when OpenClaw settings are missing.")


def test_lemonslice_agent_id_satisfies_avatar_validation(monkeypatch) -> None:
    monkeypatch.setenv("MORGAN_LLM_BACKEND", "inference")
    monkeypatch.setenv("MORGAN_AVATAR_MODE", "lemonslice")
    monkeypatch.setenv("MORGAN_LEMONSLICE_AGENT_ID", "agent_123")
    monkeypatch.delenv("MORGAN_IMAGE_URL", raising=False)
    monkeypatch.delenv("MORGAN_PLACEHOLDER_IMAGE_URL", raising=False)

    config = AgentConfig.from_env(project_root=Path("/tmp/project"))

    config.validate()
    assert config.has_lemonslice_agent_id is True


def test_disabled_avatar_mode_skips_lemonslice_validation(monkeypatch) -> None:
    monkeypatch.setenv("MORGAN_LLM_BACKEND", "inference")
    monkeypatch.setenv("MORGAN_AVATAR_MODE", "disabled")
    monkeypatch.delenv("MORGAN_LEMONSLICE_AGENT_ID", raising=False)
    monkeypatch.delenv("MORGAN_IMAGE_URL", raising=False)
    monkeypatch.delenv("MORGAN_PLACEHOLDER_IMAGE_URL", raising=False)

    config = AgentConfig.from_env(project_root=Path("/tmp/project"))

    config.validate()
    assert config.avatar_mode == "disabled"


def test_invalid_avatar_mode_fails_validation(monkeypatch) -> None:
    monkeypatch.setenv("MORGAN_LLM_BACKEND", "inference")
    monkeypatch.setenv("MORGAN_AVATAR_MODE", "broken")
    monkeypatch.setenv("MORGAN_PLACEHOLDER_IMAGE_URL", "https://example.com/placeholder.png")

    config = AgentConfig.from_env(project_root=Path("/tmp/project"))

    try:
        config.validate()
    except ValueError as exc:
        assert "MORGAN_AVATAR_MODE" in str(exc)
    else:
        raise AssertionError("Expected validation to fail for an unsupported avatar mode.")


def test_musetalk_product_mode_is_disabled(monkeypatch) -> None:
    monkeypatch.setenv("MORGAN_LLM_BACKEND", "inference")
    monkeypatch.setenv("MORGAN_AVATAR_MODE", "musetalk")

    config = AgentConfig.from_env(project_root=Path("/tmp/project"))

    try:
        config.validate()
    except ValueError as exc:
        assert "MORGAN_AVATAR_MODE" in str(exc)
    else:
        raise AssertionError("Expected validation to reject MuseTalk as a product mode.")


def test_echomimic_mode_requires_app_url(monkeypatch) -> None:
    monkeypatch.setenv("MORGAN_LLM_BACKEND", "inference")
    monkeypatch.setenv("MORGAN_AVATAR_MODE", "echomimic")
    monkeypatch.delenv("MORGAN_ECHOMIMIC_APP_URL", raising=False)
    monkeypatch.delenv("ECHOMIMIC_APP_URL", raising=False)

    config = AgentConfig.from_env(project_root=Path("/tmp/project"))

    try:
        config.validate()
    except ValueError as exc:
        assert "MORGAN_ECHOMIMIC_APP_URL" in str(exc)
    else:
        raise AssertionError("Expected validation to fail without EchoMimic app URL.")


def test_echomimic_defaults_are_loaded(monkeypatch) -> None:
    monkeypatch.setenv("MORGAN_LLM_BACKEND", "inference")
    monkeypatch.setenv("MORGAN_AVATAR_MODE", "echomimic")
    monkeypatch.setenv("MORGAN_ECHOMIMIC_APP_URL", "https://echomimic.example")
    monkeypatch.setenv("MORGAN_ECHOMIMIC_VIDEO_LENGTH", "65")
    monkeypatch.setenv("MORGAN_ECHOMIMIC_WEIGHT_DTYPE", "float16")

    config = AgentConfig.from_env(project_root=Path("/tmp/project"))

    config.validate()
    assert config.avatar_mode == "echomimic"
    assert config.echomimic_app_url == "https://echomimic.example"
    assert config.echomimic_video_length == 65
    assert config.echomimic_weight_dtype == "float16"
    assert "golden retriever" in config.echomimic_prompt


def test_openai_compatible_base_url_avoids_double_v1() -> None:
    assert _openai_compatible_base_url("https://morgan.5dlabs.ai") == "https://morgan.5dlabs.ai/v1"
    assert (
        _openai_compatible_base_url("https://morgan.5dlabs.ai/v1") == "https://morgan.5dlabs.ai/v1"
    )


def test_flux_stt_uses_provider_turn_detection_without_local_model(monkeypatch) -> None:
    sys.modules.pop("livekit.plugins.turn_detector.multilingual", None)
    monkeypatch.setenv("MORGAN_LLM_BACKEND", "inference")
    monkeypatch.setenv("MORGAN_STT_MODE", "livekit-flux")

    config = AgentConfig.from_env(project_root=Path("/tmp/project"))

    assert build_turn_detection(config) == "stt"
    assert "livekit.plugins.turn_detector.multilingual" not in sys.modules


def test_elevenlabs_scribe_uses_provider_turn_detection_without_local_model(monkeypatch) -> None:
    sys.modules.pop("livekit.plugins.turn_detector.multilingual", None)
    monkeypatch.setenv("MORGAN_LLM_BACKEND", "inference")
    monkeypatch.setenv("MORGAN_STT_MODE", "elevenlabs-scribe")

    config = AgentConfig.from_env(project_root=Path("/tmp/project"))

    assert build_turn_detection(config) == "stt"
    assert "livekit.plugins.turn_detector.multilingual" not in sys.modules


def test_elevenlabs_scribe_builds_realtime_stt(monkeypatch) -> None:
    monkeypatch.setenv("MORGAN_LLM_BACKEND", "inference")
    monkeypatch.setenv("MORGAN_STT_MODE", "elevenlabs-scribe")
    monkeypatch.setenv("ELEVEN_API_KEY", "test-key")

    config = AgentConfig.from_env(project_root=Path("/tmp/project"))

    assert build_stt(config).model == "scribe_v2_realtime"
