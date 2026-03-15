from __future__ import annotations

from pathlib import Path

from morgan_avatar_agent.config import AgentConfig


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
    monkeypatch.setenv("MORGAN_LEMONSLICE_AGENT_ID", "agent_123")
    monkeypatch.delenv("MORGAN_IMAGE_URL", raising=False)
    monkeypatch.delenv("MORGAN_PLACEHOLDER_IMAGE_URL", raising=False)

    config = AgentConfig.from_env(project_root=Path("/tmp/project"))

    config.validate()
    assert config.has_lemonslice_agent_id is True
