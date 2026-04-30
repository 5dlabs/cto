#!/usr/bin/env python3
"""Real-model LLM adapter for CTO intake's OpenClaw-compatible llm-task argv.

Supports Anthropic Messages API, Google Gemini generateContent, OpenAI chat
completions, and explicitly configured GitHub Copilot harness commands.
Secrets are read from env only and never printed.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import shlex
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any


class ProviderUnavailable(RuntimeError):
    def __init__(self, provider: str, model: str, reason: str):
        self.provider = provider
        self.model = model
        self.reason = reason
        super().__init__(reason)


def load_payload(ns: argparse.Namespace) -> dict[str, Any]:
    raw = "{}"
    if ns.args_file:
        raw = Path(ns.args_file).read_text()
    elif ns.args_json:
        raw = ns.args_json
    try:
        return json.loads(raw or "{}")
    except Exception as exc:
        print(f"real-llm-invoke: invalid args JSON: {exc}", file=sys.stderr)
        sys.exit(2)


def schema_text(path: str) -> str:
    if not path:
        return ""
    try:
        return Path(path).read_text()[:30000]
    except Exception:
        return ""


def compact(obj: Any, limit: int = 120000) -> str:
    try:
        text = json.dumps(obj, ensure_ascii=False, indent=2)
    except Exception:
        text = str(obj)
    if len(text) > limit:
        return text[:limit] + "\n...[truncated]"
    return text


def build_messages(payload: dict[str, Any], action: str) -> tuple[str, str]:
    prompt = str(payload.get("prompt") or "")
    inp = payload.get("input", {})
    schema = str(payload.get("schema") or "")
    schema_body = schema_text(schema)
    mode = "JSON" if action == "json" else "plain text"
    system = (
        "You are a CTO intake workflow LLM. Return only the requested output, no markdown fences. "
        "If JSON is requested, output strictly valid JSON matching the supplied schema as closely as possible. "
        "Do not include explanations outside JSON."
    )
    user = f"""Task prompt:
{prompt}

Input JSON:
{compact(inp)}

Requested output mode: {mode}
Schema path: {schema}
Schema JSON:
{schema_body}
"""
    return system, user


def extract_json(text: str) -> Any:
    text = text.strip()
    if text.startswith("```"):
        text = re.sub(r"^```(?:json)?\s*", "", text)
        text = re.sub(r"\s*```$", "", text)
    try:
        return json.loads(text)
    except Exception:
        pass
    starts = [i for i in (text.find("{"), text.find("[")) if i >= 0]
    if not starts:
        raise ValueError("model returned no JSON object/array")
    start = min(starts)
    opener = text[start]
    closer = "}" if opener == "{" else "]"
    depth = 0
    in_str = False
    esc = False
    for i, ch in enumerate(text[start:], start):
        if in_str:
            if esc:
                esc = False
            elif ch == "\\":
                esc = True
            elif ch == '"':
                in_str = False
        else:
            if ch == '"':
                in_str = True
            elif ch == opener:
                depth += 1
            elif ch == closer:
                depth -= 1
                if depth == 0:
                    return json.loads(text[start : i + 1])
    raise ValueError("could not parse JSON from model output")


def http_json(url: str, headers: dict[str, str], body: dict[str, Any], timeout: int = 180) -> dict[str, Any]:
    data = json.dumps(body).encode()
    req = urllib.request.Request(url, data=data, headers=headers, method="POST")
    last: Exception | None = None
    for attempt in range(4):
        try:
            with urllib.request.urlopen(req, timeout=timeout) as resp:
                return json.loads(resp.read().decode())
        except urllib.error.HTTPError as exc:
            msg = exc.read().decode(errors="replace")[:1000]
            last = RuntimeError(f"HTTP {exc.code}: {msg}")
            if exc.code not in (429, 500, 502, 503, 504):
                break
        except Exception as exc:
            last = exc
        time.sleep(2**attempt)
    raise last or RuntimeError("request failed")


def normalize_provider(provider: str) -> str:
    value = (provider or "").strip().lower()
    aliases = {
        "": "",
        "google": "gemini",
        "google-gemini": "gemini",
        "chatgpt": "openai",
        "gpt": "openai",
        "copilot": "github-copilot",
        "github": "github-copilot",
    }
    return aliases.get(value, value)


def requested_provider_model(payload: dict[str, Any]) -> tuple[str, str, bool]:
    raw_provider = str(payload.get("provider") or "").strip()
    raw_model = str(payload.get("model") or "").strip()
    explicit = bool(raw_provider)
    provider = normalize_provider(raw_provider)
    model = raw_model
    if not provider:
        provider = normalize_provider(os.environ.get("REAL_LLM_PROVIDER", "anthropic"))
        model = model or os.environ.get("REAL_LLM_MODEL", "")
    return provider, model, explicit


def capabilities() -> dict[str, Any]:
    copilot_cmd = os.environ.get("COPILOT_LLM_INVOKE_CMD") or os.environ.get("CTO_COPILOT_LLM_INVOKE_CMD")
    return {
        "providers": {
            "github-copilot": {
                "available": bool(copilot_cmd),
                "models": ["gpt-5.5"],
                "invoke": "configured-command" if copilot_cmd else "unconfigured",
                "requires": "COPILOT_LLM_INVOKE_CMD or CTO_COPILOT_LLM_INVOKE_CMD",
            },
            "openai": {"available": bool(os.environ.get("OPENAI_API_KEY")), "models": [os.environ.get("OPENAI_MODEL") or os.environ.get("CHATGPT_MODEL") or "gpt-5.5"]},
            "anthropic": {"available": bool(os.environ.get("ANTHROPIC_API_KEY")), "models": [os.environ.get("ANTHROPIC_MODEL") or "claude-sonnet-4-5-20250929"]},
            "gemini": {"available": bool(os.environ.get("GEMINI_API_KEY") or os.environ.get("GOOGLE_API_KEY")), "models": [os.environ.get("GEMINI_MODEL") or "gemini-3.1-pro-preview"]},
        }
    }


def call_anthropic(model: str, system: str, user: str, action: str) -> str:
    key = os.environ.get("ANTHROPIC_API_KEY")
    if not key:
        raise ProviderUnavailable("anthropic", model, "ANTHROPIC_API_KEY missing")
    aliases = {
        "claude-opus-4-7-20260610": "claude-opus-4-1-20250805",
        "claude-opus-4-7": "claude-opus-4-1-20250805",
        "claude-sonnet-4-6-20260514": "claude-sonnet-4-5-20250929",
        "claude-sonnet-4-6": "claude-sonnet-4-5-20250929",
        "claude-haiku-4-5-20251001": "claude-haiku-4-5-20251001",
    }
    wire_model = aliases.get(model, model or os.environ.get("ANTHROPIC_MODEL", "claude-sonnet-4-5-20250929"))
    body = {
        "model": wire_model,
        "max_tokens": int(os.environ.get("REAL_LLM_MAX_TOKENS", "6000")),
        "temperature": float(os.environ.get("REAL_LLM_TEMPERATURE", "0.2")),
        "system": system,
        "messages": [{"role": "user", "content": user}],
    }
    if action == "json":
        body["messages"][0]["content"] += "\n\nReturn valid JSON only."
    resp = http_json(
        "https://api.anthropic.com/v1/messages",
        {"content-type": "application/json", "x-api-key": key, "anthropic-version": "2023-06-01"},
        body,
    )
    parts = resp.get("content", [])
    return "".join(part.get("text", "") for part in parts if isinstance(part, dict))


def call_gemini(model: str, system: str, user: str, action: str) -> str:
    key = os.environ.get("GEMINI_API_KEY") or os.environ.get("GOOGLE_API_KEY")
    if not key:
        raise ProviderUnavailable("gemini", model, "GEMINI_API_KEY/GOOGLE_API_KEY missing")
    wire_model = model or os.environ.get("GEMINI_MODEL", "gemini-3.1-pro-preview")
    body = {
        "systemInstruction": {"parts": [{"text": system}]},
        "contents": [{"role": "user", "parts": [{"text": user + ("\n\nReturn valid JSON only." if action == "json" else "")}]}],
        "generationConfig": {"temperature": 0.2, "maxOutputTokens": int(os.environ.get("REAL_LLM_MAX_TOKENS", "6000"))},
    }
    url = f"https://generativelanguage.googleapis.com/v1beta/models/{wire_model}:generateContent?key={key}"
    resp = http_json(url, {"content-type": "application/json"}, body)
    candidates = resp.get("candidates", [])
    if not candidates:
        return ""
    parts = candidates[0].get("content", {}).get("parts", [])
    return "".join(part.get("text", "") for part in parts if isinstance(part, dict))


def call_openai(model: str, system: str, user: str, action: str) -> str:
    key = os.environ.get("OPENAI_API_KEY")
    if not key:
        raise ProviderUnavailable("openai", model, "OPENAI_API_KEY missing")
    wire_model = model or os.environ.get("OPENAI_MODEL") or os.environ.get("CHATGPT_MODEL") or "gpt-5.5"
    body = {
        "model": wire_model,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user + ("\n\nReturn valid JSON only." if action == "json" else "")},
        ],
        "temperature": float(os.environ.get("REAL_LLM_TEMPERATURE", "0.2")),
        "max_tokens": int(os.environ.get("REAL_LLM_MAX_TOKENS", "6000")),
    }
    resp = http_json("https://api.openai.com/v1/chat/completions", {"content-type": "application/json", "authorization": f"Bearer {key}"}, body)
    choices = resp.get("choices", [])
    if not choices:
        return ""
    message = choices[0].get("message", {}) if isinstance(choices[0], dict) else {}
    content = message.get("content", "") if isinstance(message, dict) else ""
    if isinstance(content, list):
        return "".join(part.get("text", "") for part in content if isinstance(part, dict))
    return str(content)


def call_copilot(model: str, system: str, user: str, action: str, payload: dict[str, Any]) -> str:
    cmd_text = os.environ.get("COPILOT_LLM_INVOKE_CMD") or os.environ.get("CTO_COPILOT_LLM_INVOKE_CMD")
    if not cmd_text:
        raise ProviderUnavailable(
            "github-copilot",
            model or "gpt-5.5",
            "No Copilot LLM harness configured. Set COPILOT_LLM_INVOKE_CMD to a command that accepts provider/model/system/user/action JSON on stdin.",
        )
    cmd = shlex.split(cmd_text)
    request = {
        "provider": "github-copilot",
        "model": model or "gpt-5.5",
        "action": action,
        "system": system,
        "user": user,
        "payload": payload,
    }
    try:
        proc = subprocess.run(
            cmd,
            input=json.dumps(request, ensure_ascii=False),
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=int(os.environ.get("COPILOT_LLM_INVOKE_TIMEOUT", "300")),
            check=False,
        )
    except FileNotFoundError as exc:
        raise ProviderUnavailable("github-copilot", model or "gpt-5.5", f"Copilot harness command not found: {exc.filename}") from exc
    if proc.returncode != 0:
        err = (proc.stderr or proc.stdout or "copilot harness failed").strip()[:1000]
        raise RuntimeError(f"github-copilot harness failed: {err}")
    return proc.stdout.strip()


def call_provider(provider: str, model: str, system: str, user: str, action: str, payload: dict[str, Any]) -> str:
    if provider == "github-copilot":
        return call_copilot(model, system, user, action, payload)
    if provider == "gemini":
        return call_gemini(model, system, user, action)
    if provider == "openai":
        return call_openai(model, system, user, action)
    if provider == "anthropic":
        return call_anthropic(model, system, user, action)
    raise ProviderUnavailable(provider, model, f"Unsupported provider: {provider}")


def main() -> int:
    parser = argparse.ArgumentParser(add_help=False)
    parser.add_argument("--tool")
    parser.add_argument("--action", default="json")
    parser.add_argument("--args-json")
    parser.add_argument("--args-file")
    ns, _ = parser.parse_known_args()
    action = ns.action or "json"
    if ns.tool == "provider-capabilities":
        print(json.dumps(capabilities(), ensure_ascii=False))
        return 0
    payload = load_payload(ns)
    system, user = build_messages(payload, action)
    provider, model, _explicit = requested_provider_model(payload)
    try:
        text = call_provider(provider, model, system, user, action, payload)
    except ProviderUnavailable as exc:
        print(
            json.dumps(
                {
                    "ok": False,
                    "error": "provider_unavailable",
                    "requested_provider": exc.provider,
                    "requested_model": exc.model,
                    "reason": exc.reason,
                },
                ensure_ascii=False,
            ),
            file=sys.stderr,
        )
        return 42
    if action == "json":
        try:
            obj = extract_json(text)
        except Exception as exc:
            print(f"real-llm-invoke: model did not return valid JSON: {exc}", file=sys.stderr)
            print(text[:2000], file=sys.stderr)
            return 3
        print(json.dumps(obj, ensure_ascii=False))
    else:
        print(text.strip())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
