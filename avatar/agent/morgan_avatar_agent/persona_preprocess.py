"""
Persona preprocessing worker for MuseTalk avatar pipeline.

Subscribes to NATS subject `avatar.persona.preprocess` with durable consumer.
Runs face detection → bbox crop → VAE encode → landmarks → writes artefacts.
"""

from __future__ import annotations

import json
import os
import pickle
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from morgan_avatar_agent.config import AgentConfig

# Create settings singleton from environment
settings = AgentConfig.from_env()

# Optional imports for actual preprocessing (commented until GPU available)
# import cv2
# import numpy as np
# import torch
# from PIL import Image


@dataclass
class PersonaStatus:
    """Status document for a persona."""

    state: str = "pending"  # pending | preprocessing | ready | failed
    error: str | None = None
    progress_percent: int = 0
    artefacts: list[str] = field(default_factory=list)

    def to_dict(self) -> dict[str, Any]:
        return {
            "state": self.state,
            "error": self.error,
            "progress_percent": self.progress_percent,
            "artefacts": self.artefacts,
        }

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "PersonaStatus":
        return cls(
            state=data.get("state", "pending"),
            error=data.get("error"),
            progress_percent=data.get("progress_percent", 0),
            artefacts=data.get("artefacts", []),
        )


class PersonaStorage:
    """Manages persona storage on PVC."""

    def __init__(self, base_path: str | Path | None = None) -> None:
        self.base_path = Path(base_path or settings.personas_pvc_path or "/personas")

    def persona_dir(self, persona_id: str) -> Path:
        """Get the directory path for a persona."""
        return self.base_path / persona_id

    def ensure_persona_dir(self, persona_id: str) -> Path:
        """Ensure persona directory exists with required subdirs."""
        pdir = self.persona_dir(persona_id)
        for subdir in ["latents", "landmarks", "mask"]:
            (pdir / subdir).mkdir(parents=True, exist_ok=True)
        return pdir

    def read_status(self, persona_id: str) -> PersonaStatus:
        """Read status.json for a persona."""
        status_path = self.persona_dir(persona_id) / "status.json"
        if not status_path.exists():
            return PersonaStatus()
        try:
            with open(status_path, "r") as f:
                return PersonaStatus.from_dict(json.load(f))
        except (json.JSONDecodeError, IOError):
            return PersonaStatus(state="failed", error="Corrupted status.json")

    def write_status(self, persona_id: str, status: PersonaStatus) -> None:
        """Write status.json for a persona."""
        status_path = self.ensure_persona_dir(persona_id) / "status.json"
        with open(status_path, "w") as f:
            json.dump(status.to_dict(), f, indent=2)

    def write_metadata(
        self, persona_id: str, name: str, source_type: str, uploaded_by: str
    ) -> None:
        """Write metadata.json for a persona."""
        meta_path = self.ensure_persona_dir(persona_id) / "metadata.json"
        metadata = {
            "id": persona_id,
            "name": name,
            "source_type": source_type,  # image | video
            "uploaded_by": uploaded_by,
            "created_at": json.dumps({"__type": "datetime", "iso": "now"}),
        }
        with open(meta_path, "w") as f:
            json.dump(metadata, f, indent=2)

    def list_personas(self) -> list[dict[str, Any]]:
        """List all personas with their status."""
        if not self.base_path.exists():
            return []
        personas = []
        for persona_dir in self.base_path.iterdir():
            if persona_dir.is_dir():
                persona_id = persona_dir.name
                status = self.read_status(persona_id)
                metadata = {}
                meta_path = persona_dir / "metadata.json"
                if meta_path.exists():
                    try:
                        with open(meta_path, "r") as f:
                            metadata = json.load(f)
                    except json.JSONDecodeError:
                        pass
                personas.append(
                    {
                        "id": persona_id,
                        "name": metadata.get("name", persona_id),
                        "state": status.state,
                        "error": status.error,
                        "progress_percent": status.progress_percent,
                        "created_at": metadata.get("created_at"),
                    }
                )
        return personas

    def get_source_path(self, persona_id: str) -> Path | None:
        """Get the source file path for a persona (png, jpg, or mp4)."""
        pdir = self.persona_dir(persona_id)
        for ext in [".png", ".jpg", ".jpeg", ".mp4", ".webm"]:
            src = pdir / f"source{ext}"
            if src.exists():
                return src
        return None

    def delete_persona(self, persona_id: str) -> bool:
        """Delete a persona and all its artefacts."""
        import shutil

        pdir = self.persona_dir(persona_id)
        if pdir.exists():
            shutil.rmtree(pdir)
            return True
        return False


class PersonaPreprocessor:
    """Preprocesses persona source files for MuseTalk inference."""

    def __init__(self, storage: PersonaStorage | None = None) -> None:
        self.storage = storage or PersonaStorage()
        self.max_retries = 3

    def preprocess(self, persona_id: str) -> PersonaStatus:
        """
        Run full preprocessing pipeline for a persona.

        Steps:
        1. Load source file (image or video)
        2. Face detection and bbox extraction
        3. Crop to face region
        4. VAE encode to latents
        5. Extract landmarks
        6. Generate preview video (5s)
        7. Write status.json as "ready"
        """
        # Update status to preprocessing
        status = PersonaStatus(state="preprocessing", progress_percent=0)
        self.storage.write_status(persona_id, status)

        try:
            source_path = self.storage.get_source_path(persona_id)
            if not source_path:
                raise ValueError(f"No source file found for persona {persona_id}")

            # Phase 3.5: Stub implementation - actual CV/ML code when GPU available
            # For now, simulate the pipeline steps
            status.progress_percent = 10
            self.storage.write_status(persona_id, status)

            # Step 1: Face detection (stub)
            self._detect_face(persona_id, source_path)
            status.progress_percent = 30
            self.storage.write_status(persona_id, status)

            # Step 2: Crop and resize (stub)
            self._crop_face(persona_id)
            status.progress_percent = 50
            self.storage.write_status(persona_id, status)

            # Step 3: VAE encode (stub)
            self._vae_encode(persona_id)
            status.progress_percent = 70
            self.storage.write_status(persona_id, status)

            # Step 4: Extract landmarks (stub)
            self._extract_landmarks(persona_id)
            status.progress_percent = 85
            self.storage.write_status(persona_id, status)

            # Step 5: Generate preview (stub)
            self._generate_preview(persona_id)
            status.progress_percent = 100

            # Mark as ready
            status.state = "ready"
            status.artefacts = [
                "source.png",
                "metadata.json",
                "latents/",
                "landmarks/",
                "coords.pkl",
                "mask/",
                "preview.mp4",
                "status.json",
            ]
            self.storage.write_status(persona_id, status)

            return status

        except Exception as e:
            status.state = "failed"
            status.error = str(e)
            self.storage.write_status(persona_id, status)
            raise

    def _detect_face(self, persona_id: str, source_path: Path) -> None:
        """Detect face in source file. Stub - actual implementation with GPU."""
        # TODO: Use MediaPipe or dlib for face detection
        # For now, write a placeholder bbox file
        pdir = self.storage.persona_dir(persona_id)
        bbox = {"x": 100, "y": 100, "width": 256, "height": 256}
        with open(pdir / "bbox.json", "w") as f:
            json.dump(bbox, f)

    def _crop_face(self, persona_id: str) -> None:
        """Crop face region from source. Stub - actual implementation with GPU."""
        # TODO: Use OpenCV/PIL to crop and resize to 256x256 or 512x512
        pass

    def _vae_encode(self, persona_id: str) -> None:
        """VAE encode to latents. Stub - actual implementation with GPU."""
        # TODO: Load MuseTalk VAE and encode frames to latents
        # Write latents to latents/ directory
        pdir = self.storage.persona_dir(persona_id)
        latents_dir = pdir / "latents"
        # Placeholder: create empty files to indicate structure
        (latents_dir / "frame_0000.pkl").touch()

    def _extract_landmarks(self, persona_id: str) -> None:
        """Extract facial landmarks. Stub - actual implementation with GPU."""
        # TODO: Use MediaPipe Face Mesh for 468 landmarks
        pdir = self.storage.persona_dir(persona_id)
        landmarks_dir = pdir / "landmarks"
        coords = {"coords": [[0.0, 0.0]] * 468}  # Placeholder
        with open(pdir / "coords.pkl", "wb") as f:
            pickle.dump(coords, f)
        (landmarks_dir / "frame_0000.json").touch()

    def _generate_preview(self, persona_id: str) -> None:
        """Generate 5s preview video. Stub - actual implementation with GPU."""
        # TODO: Use ffmpeg to generate silent 5s video from processed frames
        pdir = self.storage.persona_dir(persona_id)
        # Placeholder: create empty preview file
        (pdir / "preview.mp4").touch()


# NATS consumer stub - will be implemented with actual NATS client
# when the messaging infrastructure is confirmed

# Fallback: K8s Job API
# If NATS causes backpressure/ordering issues, we can switch to
# creating Kubernetes Jobs for each preprocessing task.


def run_preprocessing_job(persona_id: str, namespace: str = "avatar") -> dict[str, Any]:
    """
    Create a Kubernetes Job to run preprocessing for a persona.
    Fallback when NATS consumer is not suitable.
    """
    import uuid

    job_name = f"preprocess-{persona_id}-{uuid.uuid4().hex[:8]}"

    # Job manifest template
    job_manifest = {
        "apiVersion": "batch/v1",
        "kind": "Job",
        "metadata": {"name": job_name, "namespace": namespace},
        "spec": {
            "template": {
                "spec": {
                    "containers": [
                        {
                            "name": "preprocessor",
                            "image": "ghcr.io/5dlabs/musetalk-worker:phase4-bootstrap",
                            "command": [
                                "python",
                                "-m",
                                "morgan_avatar_agent.persona_preprocess",
                                "--persona-id",
                                persona_id,
                            ],
                            "volumeMounts": [
                                {
                                    "name": "personas-pvc",
                                    "mountPath": "/personas",
                                }
                            ],
                        }
                    ],
                    "volumes": [
                        {
                            "name": "personas-pvc",
                            "persistentVolumeClaim": {"claimName": "personas-pvc"},
                        }
                    ],
                    "restartPolicy": "Never",
                }
            },
            "backoffLimit": 3,
        },
    }

    return job_manifest


def main() -> None:
    """CLI entry point for preprocessing."""
    import argparse

    parser = argparse.ArgumentParser(description="Preprocess persona for MuseTalk")
    parser.add_argument("--persona-id", required=True, help="Persona ID to preprocess")
    parser.add_argument(
        "--pvc-path", default="/personas", help="Base path for personas PVC"
    )
    args = parser.parse_args()

    storage = PersonaStorage(args.pvc_path)
    preprocessor = PersonaPreprocessor(storage)

    print(f"Starting preprocessing for persona: {args.persona_id}")
    try:
        status = preprocessor.preprocess(args.persona_id)
        print(f"Preprocessing complete: {status.state}")
        if status.error:
            print(f"Error: {status.error}")
    except Exception as e:
        print(f"Preprocessing failed: {e}")
        raise


if __name__ == "__main__":
    main()
