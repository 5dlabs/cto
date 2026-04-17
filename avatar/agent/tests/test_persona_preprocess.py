"""Tests for persona preprocessing module."""

from __future__ import annotations

import json
import os
import tempfile
from pathlib import Path

import pytest

from morgan_avatar_agent.persona_preprocess import (
    PersonaPreprocessor,
    PersonaStatus,
    PersonaStorage,
    run_preprocessing_job,
)


@pytest.fixture
def temp_storage():
    """Create a temporary storage directory for testing."""
    with tempfile.TemporaryDirectory() as tmpdir:
        yield PersonaStorage(tmpdir)


class TestPersonaStatus:
    """Test PersonaStatus dataclass."""

    def test_default_status(self):
        status = PersonaStatus()
        assert status.state == "pending"
        assert status.error is None
        assert status.progress_percent == 0
        assert status.artefacts == []

    def test_to_dict(self):
        status = PersonaStatus(
            state="ready",
            error=None,
            progress_percent=100,
            artefacts=["source.png", "metadata.json"],
        )
        data = status.to_dict()
        assert data["state"] == "ready"
        assert data["progress_percent"] == 100
        assert data["artefacts"] == ["source.png", "metadata.json"]

    def test_from_dict(self):
        data = {
            "state": "failed",
            "error": "Test error",
            "progress_percent": 50,
            "artefacts": ["file1"],
        }
        status = PersonaStatus.from_dict(data)
        assert status.state == "failed"
        assert status.error == "Test error"
        assert status.progress_percent == 50
        assert status.artefacts == ["file1"]


class TestPersonaStorage:
    """Test PersonaStorage class."""

    def test_ensure_persona_dir(self, temp_storage):
        persona_id = "test-persona"
        pdir = temp_storage.ensure_persona_dir(persona_id)

        assert pdir.exists()
        assert (pdir / "latents").exists()
        assert (pdir / "landmarks").exists()
        assert (pdir / "mask").exists()

    def test_write_and_read_status(self, temp_storage):
        persona_id = "test-persona"
        status = PersonaStatus(state="preprocessing", progress_percent=50)

        temp_storage.write_status(persona_id, status)
        read = temp_storage.read_status(persona_id)

        assert read.state == "preprocessing"
        assert read.progress_percent == 50

    def test_read_missing_status(self, temp_storage):
        status = temp_storage.read_status("nonexistent")
        assert status.state == "pending"

    def test_write_metadata(self, temp_storage):
        persona_id = "test-persona"
        temp_storage.write_metadata(persona_id, "Test Name", "image", "admin")

        meta_path = temp_storage.persona_dir(persona_id) / "metadata.json"
        assert meta_path.exists()

        with open(meta_path) as f:
            data = json.load(f)
        assert data["name"] == "Test Name"
        assert data["source_type"] == "image"
        assert data["uploaded_by"] == "admin"

    def test_list_personas(self, temp_storage):
        # Create two personas
        temp_storage.write_metadata("persona-1", "First", "image", "admin")
        temp_storage.write_status("persona-1", PersonaStatus(state="ready"))

        temp_storage.write_metadata("persona-2", "Second", "video", "admin")
        temp_storage.write_status("persona-2", PersonaStatus(state="preprocessing"))

        personas = temp_storage.list_personas()

        assert len(personas) == 2
        assert personas[0]["id"] in ["persona-1", "persona-2"]

    def test_delete_persona(self, temp_storage):
        persona_id = "to-delete"
        temp_storage.ensure_persona_dir(persona_id)

        assert temp_storage.delete_persona(persona_id) is True
        assert not temp_storage.persona_dir(persona_id).exists()

    def test_delete_nonexistent(self, temp_storage):
        assert temp_storage.delete_persona("nonexistent") is False

    def test_get_source_path(self, temp_storage):
        persona_id = "test-source"
        pdir = temp_storage.ensure_persona_dir(persona_id)

        # Create source file
        (pdir / "source.png").touch()

        path = temp_storage.get_source_path(persona_id)
        assert path is not None
        assert path.name == "source.png"


class TestPersonaPreprocessor:
    """Test PersonaPreprocessor class."""

    def test_preprocess_no_source(self, temp_storage):
        preprocessor = PersonaPreprocessor(temp_storage)

        with pytest.raises(ValueError, match="No source file found"):
            preprocessor.preprocess("nonexistent")

    def test_preprocess_success(self, temp_storage):
        persona_id = "test-preprocess"
        pdir = temp_storage.ensure_persona_dir(persona_id)

        # Create a dummy source file
        (pdir / "source.png").touch()

        preprocessor = PersonaPreprocessor(temp_storage)
        status = preprocessor.preprocess(persona_id)

        assert status.state == "ready"
        assert status.progress_percent == 100
        assert "preview.mp4" in status.artefacts

        # Verify status was written
        read_status = temp_storage.read_status(persona_id)
        assert read_status.state == "ready"


class TestK8sJob:
    """Test Kubernetes Job generation."""

    def test_run_preprocessing_job(self):
        job = run_preprocessing_job("test-persona", "avatar")

        assert job["apiVersion"] == "batch/v1"
        assert job["kind"] == "Job"
        assert job["metadata"]["namespace"] == "avatar"
        assert "preprocess-test-persona" in job["metadata"]["name"]

        container = job["spec"]["template"]["spec"]["containers"][0]
        assert container["name"] == "preprocessor"
        assert "musetalk-worker" in container["image"]
        assert "--persona-id" in container["command"]
        assert "test-persona" in container["command"]
