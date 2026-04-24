from __future__ import annotations

from pathlib import Path

from morgan_avatar_agent.avatar_provider import (
    DisabledAvatarProvider,
    LemonSliceAvatarProvider,
    MuseTalkProvider,
    build_avatar_provider,
)
from morgan_avatar_agent.config import AgentConfig
from morgan_avatar_agent.echomimic_avatar import EchoMimicAvatarSession


def _config(monkeypatch, mode: str) -> AgentConfig:
    monkeypatch.setenv("MORGAN_LLM_BACKEND", "inference")
    monkeypatch.setenv("MORGAN_AVATAR_MODE", mode)
    if mode == "lemonslice":
        monkeypatch.setenv("MORGAN_LEMONSLICE_AGENT_ID", "agent_123")
    if mode == "echomimic":
        monkeypatch.setenv("MORGAN_ECHOMIMIC_APP_URL", "https://echomimic.example")
    return AgentConfig.from_env(project_root=Path("/tmp/project"))


def test_provider_factory_selects_disabled(monkeypatch) -> None:
    provider = build_avatar_provider(
        _config(monkeypatch, "disabled"),
        allow_audio_only_fallback=True,
    )

    assert isinstance(provider, DisabledAvatarProvider)


def test_provider_factory_selects_lemonslice(monkeypatch) -> None:
    provider = build_avatar_provider(
        _config(monkeypatch, "lemonslice"),
        allow_audio_only_fallback=True,
    )

    assert isinstance(provider, LemonSliceAvatarProvider)


def test_provider_factory_selects_musetalk(monkeypatch) -> None:
    provider = build_avatar_provider(
        _config(monkeypatch, "musetalk"),
        allow_audio_only_fallback=True,
    )

    assert isinstance(provider, MuseTalkProvider)


def test_provider_factory_selects_echomimic(monkeypatch) -> None:
    provider = build_avatar_provider(
        _config(monkeypatch, "echomimic"),
        allow_audio_only_fallback=True,
    )

    assert isinstance(provider, EchoMimicAvatarSession)


def test_echomimic_provider_carries_tuning_options(monkeypatch) -> None:
    monkeypatch.setenv("MORGAN_LLM_BACKEND", "inference")
    monkeypatch.setenv("MORGAN_AVATAR_MODE", "echomimic")
    monkeypatch.setenv("MORGAN_ECHOMIMIC_APP_URL", "https://echomimic.example")
    monkeypatch.setenv("MORGAN_ECHOMIMIC_VIDEO_LENGTH", "147")
    monkeypatch.setenv("MORGAN_ECHOMIMIC_SAMPLE_HEIGHT", "768")
    monkeypatch.setenv("MORGAN_ECHOMIMIC_SAMPLE_WIDTH", "768")
    monkeypatch.setenv("MORGAN_ECHOMIMIC_WEIGHT_DTYPE", "bfloat16")
    config = AgentConfig.from_env(project_root=Path("/tmp/project"))

    provider = build_avatar_provider(config, allow_audio_only_fallback=True)

    assert isinstance(provider, EchoMimicAvatarSession)
    assert provider._optional_form_fields() == {
        "video_length": "147",
        "sample_height": "768",
        "sample_width": "768",
        "weight_dtype": "bfloat16",
    }


def test_echomimic_provider_extracts_assistant_turn_text(monkeypatch) -> None:
    provider = build_avatar_provider(
        _config(monkeypatch, "echomimic"),
        allow_audio_only_fallback=True,
    )

    assert isinstance(provider, EchoMimicAvatarSession)

    item = type(
        "Item",
        (),
        {
            "role": "assistant",
            "interrupted": False,
            "content": ["Hello", type("Part", (), {"text": "there"})()],
        },
    )()
    event = type("Event", (), {"item": item})()

    assert provider._assistant_text_from_event(event) == "Hello there"
