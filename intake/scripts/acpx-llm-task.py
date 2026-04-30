#!/usr/bin/env python3
"""ACPX-backed OpenClaw-compatible llm-task adapter for intake workflows."""
from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any

AGENT_BY_PROVIDER = {
    "github-copilot": "copilot",
    "copilot": "copilot",
    "github": "copilot",
    "gemini": "gemini",
    "google": "gemini",
    "google-gemini": "gemini",
    "anthropic": "claude",
    "claude": "claude",
    "openai": "codex",
    "codex": "codex",
    "gpt": "codex",
    "opencode": "opencode",
    "fireworks": "opencode",
    "factory": "droid",
    "cursor": "cursor",
}

DEFAULT_MODELS = {
    "github-copilot": ["gpt-5.5", "gpt-4.1", "claude-opus-4.7"],
    "gemini": ["gemini-3.1-pro-preview", "gemini-3.0-pro-preview"],
    "anthropic": ["claude-opus-4-7", "claude-sonnet-4-5"],
    "openai": ["gpt-5.5", "gpt-5.1"],
    "opencode": [],
    "factory": [],
    "cursor": [],
}

PROVIDER_AGENT = {
    "github-copilot": "copilot",
    "gemini": "gemini",
    "anthropic": "claude",
    "openai": "codex",
    "opencode": "opencode",
    "factory": "droid",
    "cursor": "cursor",
}


def die(message: str, code: int = 1) -> None:
    print(f"acpx-llm-task: {message}", file=sys.stderr)
    raise SystemExit(code)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tool", required=True)
    parser.add_argument("--action", required=True, choices=("json", "text"))
    parser.add_argument("--args-json")
    parser.add_argument("--args-file")
    return parser.parse_args()


def load_payload(args: argparse.Namespace) -> dict[str, Any]:
    if args.args_json and args.args_file:
        die("use only one of --args-json or --args-file")
    if args.args_file:
        try:
            return json.loads(Path(args.args_file).read_text())
        except Exception as exc:  # noqa: BLE001
            die(f"failed to read --args-file: {exc}")
    if args.args_json:
        try:
            return json.loads(args.args_json)
        except Exception as exc:  # noqa: BLE001
            die(f"failed to parse --args-json: {exc}")
    return {}


def command_available(command: str) -> bool:
    return shutil.which(command) is not None


def provider_capabilities() -> dict[str, Any]:
    acpx_available = command_available(os.environ.get("ACPX_LLM_BIN", "acpx"))
    providers: dict[str, Any] = {}
    for provider, agent in PROVIDER_AGENT.items():
        agent_available = command_available(agent)
        providers[provider] = {
            "available": bool(acpx_available and agent_available),
            "invoke": "acpx",
            "agent": agent,
            "acpx_available": acpx_available,
            "agent_available": agent_available,
            "models": DEFAULT_MODELS.get(provider, []),
        }
    return {"providers": providers}


def read_schema(schema_ref: Any) -> str:
    if not schema_ref:
        return ""
    if isinstance(schema_ref, (dict, list)):
        return json.dumps(schema_ref, indent=2, sort_keys=True)
    schema_text = str(schema_ref)
    path = Path(schema_text)
    if path.exists() and path.is_file():
        try:
            return path.read_text()
        except Exception as exc:  # noqa: BLE001
            return f"[Could not read schema {schema_text}: {exc}]"
    return schema_text


def build_prompt(payload: dict[str, Any], action: str) -> str:
    task_prompt = payload.get("prompt") or payload.get("system") or payload.get("instructions") or ""
    input_payload = payload.get("input", payload.get("payload", {}))
    schema = read_schema(payload.get("schema") or payload.get("json_schema"))
    input_json = json.dumps(input_payload, indent=2, sort_keys=True, ensure_ascii=False)
    mode = "JSON" if action == "json" else "plain text"
    parts = [
        "System:",
        "You are a CTO intake workflow LLM. Return only the requested output.",
        "Do not include markdown fences unless explicitly requested.",
    ]
    if action == "json":
        parts.extend([
            "If JSON is requested, output strictly valid JSON only.",
            "Do not include explanations outside the JSON value.",
        ])
    parts.extend([
        "",
        "User task prompt:",
        str(task_prompt),
        "",
        "Input JSON:",
        input_json,
        "",
        f"Requested output mode: {mode}",
    ])
    if schema:
        parts.extend(["", "Schema JSON:", schema])
    if action == "json":
        parts.extend(["", "Return valid JSON only. Do not use markdown fences."])
    return "\n".join(parts).strip() + "\n"


def resolve_agent_model(payload: dict[str, Any]) -> tuple[str, str]:
    provider = str(payload.get("provider") or os.environ.get("REAL_LLM_PROVIDER") or "").strip().lower()
    model = str(payload.get("model") or os.environ.get("ACPX_LLM_MODEL") or os.environ.get("REAL_LLM_MODEL") or "").strip()
    agent = os.environ.get("ACPX_LLM_AGENT", "").strip()
    if not agent:
        agent = AGENT_BY_PROVIDER.get(provider, provider or "copilot")
    if os.environ.get("ACPX_LLM_MODEL"):
        model = os.environ["ACPX_LLM_MODEL"].strip()
    return agent, model


def extract_json(text: str) -> Any:
    decoder = json.JSONDecoder()
    stripped = text.strip()
    if not stripped:
        raise ValueError("empty output")
    candidates = [stripped]
    fence = "```"
    if fence in stripped:
        segments = stripped.split(fence)
        candidates.extend(seg.removeprefix("json").strip() for seg in segments)
    for candidate in candidates:
        try:
            value, end = decoder.raw_decode(candidate)
            if candidate[end:].strip() == "":
                return value
        except json.JSONDecodeError:
            pass
    for idx, ch in enumerate(stripped):
        if ch not in "[{":
            continue
        try:
            value, _ = decoder.raw_decode(stripped[idx:])
            return value
        except json.JSONDecodeError:
            continue
    raise ValueError("no valid JSON object or array found")


def run_llm_task(payload: dict[str, Any], action: str) -> int:
    acpx_bin = os.environ.get("ACPX_LLM_BIN", "acpx")
    if not command_available(acpx_bin):
        die(f"ACPX binary not found: {acpx_bin}", 127)
    agent, model = resolve_agent_model(payload)
    cwd = os.environ.get("ACPX_LLM_CWD") or os.environ.get("WORKSPACE") or os.getcwd()
    timeout = os.environ.get("ACPX_LLM_TIMEOUT", "300")
    prompt = build_prompt(payload, action)
    with tempfile.NamedTemporaryFile("w", encoding="utf-8", suffix=".md", prefix="acpx-llm-task.", delete=False) as fh:
        fh.write(prompt)
        prompt_path = fh.name
    try:
        cmd = [
            acpx_bin,
            "--cwd",
            cwd,
            "--non-interactive-permissions",
            "deny",
            "--auth-policy",
            "skip",
            "--timeout",
            timeout,
            "--format",
            "text",
        ]
        if model and agent != "cursor":
            cmd.extend(["--model", model])
        cmd.extend([agent, "exec", "-f", prompt_path])
        proc = subprocess.run(cmd, text=True, capture_output=True, check=False)
        if proc.returncode != 0:
            if proc.stderr:
                print(proc.stderr.strip(), file=sys.stderr)
            die(f"ACPX command failed with exit {proc.returncode}", proc.returncode)
        if action == "text":
            print(proc.stdout.strip())
            return 0
        try:
            value = extract_json(proc.stdout)
        except ValueError as exc:
            if proc.stderr:
                print(proc.stderr.strip(), file=sys.stderr)
            die(f"ACPX output was not valid JSON: {exc}")
        print(json.dumps(value, ensure_ascii=False, separators=(",", ":")))
        return 0
    finally:
        try:
            os.unlink(prompt_path)
        except OSError:
            pass


def main() -> int:
    args = parse_args()
    if args.tool == "provider-capabilities":
        print(json.dumps(provider_capabilities(), ensure_ascii=False, sort_keys=True))
        return 0
    if args.tool != "llm-task":
        die(f"unsupported tool: {args.tool}", 2)
    payload = load_payload(args)
    return run_llm_task(payload, args.action)


if __name__ == "__main__":
    raise SystemExit(main())
