#!/usr/bin/env python3
"""Normalize model catalog data for CTO consumers.

The script accepts raw `mmmodels ... --json` output and writes two stable
artifacts:
1) normalized catalog for cross-app usage
2) OpenClaw chart model map consumed by Helm templates
"""

from __future__ import annotations

import argparse
import json
import subprocess
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


def now_iso() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Normalize mmmodels catalog output.")
    parser.add_argument(
        "--input-json",
        help="Path to raw mmmodels JSON. If omitted, mmmodels is executed.",
    )
    parser.add_argument(
        "--providers",
        default="anthropic,openai",
        help="Comma-separated provider ids to include (default: anthropic,openai).",
    )
    parser.add_argument(
        "--normalized-out",
        default="infra/model-catalog/normalized-model-catalog.json",
        help="Output path for normalized shared artifact.",
    )
    parser.add_argument(
        "--openclaw-out",
        default="infra/charts/openclaw-agent/files/model-catalog.generated.json",
        help="Output path for OpenClaw provider/model map.",
    )
    parser.add_argument(
        "--web-out",
        default="apps/web/src/generated/model-catalog.json",
        help="Output path for web API model list source.",
    )
    parser.add_argument(
        "--generated-at",
        help="Override ISO timestamp for deterministic tests.",
    )
    return parser.parse_args()


def run_mmmodels(provider_id: str) -> list[dict[str, Any]]:
    command = [
        "npx",
        "--yes",
        "mmmodels",
        "list",
        "--provider",
        provider_id,
        "--json",
        "--plain",
    ]
    completed = subprocess.run(command, check=True, capture_output=True, text=True)
    payload = json.loads(completed.stdout)
    if isinstance(payload, list):
        return [entry for entry in payload if isinstance(entry, dict)]
    return []


def load_input(path: Path) -> list[dict[str, Any]]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if isinstance(payload, list):
        return [entry for entry in payload if isinstance(entry, dict)]
    raise ValueError("input JSON must be an array of objects")


def first_string(entry: dict[str, Any], *keys: str) -> str | None:
    for key in keys:
        value = entry.get(key)
        if isinstance(value, str) and value.strip():
            return value.strip()
    return None


def first_int(entry: dict[str, Any], *keys: str) -> int | None:
    for key in keys:
        value = entry.get(key)
        if isinstance(value, int):
            return value
        if isinstance(value, float):
            return int(value)
        if isinstance(value, str) and value.isdigit():
            return int(value)
    return None


def parse_capabilities(entry: dict[str, Any]) -> list[str]:
    capabilities: list[str] = []

    raw_caps = entry.get("capabilities")
    if isinstance(raw_caps, list):
        capabilities.extend(str(cap).strip().lower() for cap in raw_caps if str(cap).strip())

    for key in ("reasoning", "tools", "files", "open", "structured_output", "structured"):
        value = entry.get(key)
        if isinstance(value, bool) and value:
            capabilities.append(key.replace("_output", "").replace("structured", "structured"))

    # Stable order, unique values.
    return sorted(set(capabilities))


@dataclass
class NormalizedModel:
    model_id: str
    model_name: str
    provider_id: str
    status: str | None
    capabilities: list[str]
    context_window: int | None
    max_tokens: int | None
    last_updated: str | None


def normalize_entry(entry: dict[str, Any], provider_hint: str) -> NormalizedModel | None:
    model_id = first_string(entry, "id", "model_id", "modelId")
    if not model_id:
        return None

    model_name = first_string(entry, "name", "model", "display_name") or model_id
    provider_id = first_string(entry, "provider_id", "providerId", "provider") or provider_hint
    status = first_string(entry, "status")
    context_window = first_int(entry, "context_window", "input_limit", "tokens")
    max_tokens = first_int(entry, "max_tokens", "output_limit")
    last_updated = first_string(entry, "last_updated", "release_date")
    capabilities = parse_capabilities(entry)

    return NormalizedModel(
        model_id=model_id,
        model_name=model_name,
        provider_id=provider_id,
        status=status,
        capabilities=capabilities,
        context_window=context_window,
        max_tokens=max_tokens,
        last_updated=last_updated,
    )


def dedupe_models(models: list[NormalizedModel]) -> list[NormalizedModel]:
    by_id: dict[str, NormalizedModel] = {}
    for model in models:
        existing = by_id.get(model.model_id)
        if existing is None:
            by_id[model.model_id] = model
            continue
        existing_ts = existing.last_updated or ""
        next_ts = model.last_updated or ""
        if next_ts >= existing_ts:
            by_id[model.model_id] = model
    return [by_id[key] for key in sorted(by_id.keys())]


def ensure_parent(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)


def write_json(path: Path, data: dict[str, Any]) -> None:
    ensure_parent(path)
    path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def model_to_normalized_dict(model: NormalizedModel) -> dict[str, Any]:
    return {
        "capabilities": model.capabilities,
        "contextWindow": model.context_window,
        "id": model.model_id,
        "lastUpdated": model.last_updated,
        "maxTokens": model.max_tokens,
        "name": model.model_name,
        "status": model.status,
    }


def model_to_openclaw_dict(model: NormalizedModel) -> dict[str, Any]:
    payload: dict[str, Any] = {
        "id": model.model_id,
        "name": model.model_name,
    }
    if model.context_window is not None:
        payload["contextWindow"] = model.context_window
    if model.max_tokens is not None:
        payload["maxTokens"] = model.max_tokens
    if "reasoning" in model.capabilities:
        payload["reasoning"] = True
    if model.status:
        payload["status"] = model.status
    if model.last_updated:
        payload["lastUpdated"] = model.last_updated
    return payload


def main() -> int:
    args = parse_args()
    generated_at = args.generated_at or now_iso()
    providers = [provider.strip() for provider in args.providers.split(",") if provider.strip()]

    normalized_provider_map: dict[str, list[NormalizedModel]] = {}
    for provider_id in providers:
        raw_entries = (
            load_input(Path(args.input_json))
            if args.input_json
            else run_mmmodels(provider_id)
        )
        normalized_entries: list[NormalizedModel] = []
        for raw in raw_entries:
            normalized = normalize_entry(raw, provider_id)
            if normalized is None:
                continue
            if normalized.provider_id != provider_id:
                continue
            normalized_entries.append(normalized)
        normalized_provider_map[provider_id] = dedupe_models(normalized_entries)

    normalized_payload: dict[str, Any] = {
        "generatedAt": generated_at,
        "schemaVersion": 1,
        "source": "mmmodels",
        "providers": [],
    }
    for provider_id in sorted(normalized_provider_map.keys()):
        models = normalized_provider_map[provider_id]
        normalized_payload["providers"].append(
            {
                "id": provider_id,
                "modelCount": len(models),
                "models": [model_to_normalized_dict(model) for model in models],
            }
        )

    openclaw_payload: dict[str, Any] = {
        "generatedAt": generated_at,
        "schemaVersion": 1,
        "providers": {},
    }
    web_payload: dict[str, Any] = {
        "generatedAt": generated_at,
        "schemaVersion": 1,
        "providers": {},
    }
    for provider_id in sorted(normalized_provider_map.keys()):
        models = normalized_provider_map[provider_id]
        openclaw_payload["providers"][provider_id] = {
            "models": [model_to_openclaw_dict(model) for model in models]
        }
        web_payload["providers"][provider_id] = [model.model_id for model in models]

    write_json(Path(args.normalized_out), normalized_payload)
    write_json(Path(args.openclaw_out), openclaw_payload)
    write_json(Path(args.web_out), web_payload)
    print(f"Wrote {args.normalized_out}")
    print(f"Wrote {args.openclaw_out}")
    print(f"Wrote {args.web_out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
