#!/usr/bin/env python3
"""Hermes CodeRun presence E2E smoke harness.

The default mode is safe/dry-run: render the CodeRun manifest and print the
validation plan without mutating the cluster. Set SMOKE_MODE=live to create a
short-lived Hermes CodeRun and validate that the controller-rendered
hermes-presence-adapter route accepts a synthetic Discord event through the
central presence router.

Secrets are accepted only via environment or existing Kubernetes Secret refs and
are never printed.
"""
from __future__ import annotations

import argparse
import json
import os
import random
import re
import string
import subprocess
import tempfile
import time
import urllib.error
import urllib.request
from dataclasses import dataclass
from pathlib import Path
from typing import Any


REDACT = "***REDACTED***"
DEFAULT_ROUTER_URL = "http://discord-bridge-http.bots.svc:3200"
DEFAULT_TOKEN_SECRET = "openclaw-discord-tokens"
DEFAULT_TOKEN_KEY = "PRESENCE_SHARED_TOKEN"


def getenv(name: str, default: str | None = None) -> str:
    value = os.environ.get(name, default)
    if value is None or value == "":
        raise SystemExit(f"{name} is required")
    return value


def optional_env(name: str, default: str | None = None) -> str | None:
    value = os.environ.get(name, default)
    if value is None:
        return None
    value = value.strip()
    return value or None


def bool_env(name: str, default: bool = False) -> bool:
    value = os.environ.get(name)
    if value is None:
        return default
    return value.lower() in {"1", "true", "yes", "y", "on"}


def rand_suffix(length: int = 8) -> str:
    return "".join(random.choices(string.ascii_lowercase + string.digits, k=length))


def sanitize_k8s_name(value: str, max_len: int = 63) -> str:
    value = re.sub(r"[^a-z0-9-]+", "-", value.lower()).strip("-")
    value = re.sub(r"-+", "-", value)
    return (value[:max_len].strip("-") or "smoke")


def redact_text(value: str, secrets: list[str]) -> str:
    redacted = value
    for secret in secrets:
        if secret:
            redacted = redacted.replace(secret, REDACT)
    redacted = re.sub(r'(Authorization:\s*Bearer\s+)[^\s\'\"]+', rf"\\1{REDACT}", redacted, flags=re.IGNORECASE)
    redacted = re.sub(r"(--from-literal=[^=]+=)[^\s]+", rf"\1{REDACT}", redacted)
    return redacted


@dataclass
class Config:
    mode: str
    namespace: str
    run_id: str
    coderun_name: str
    router_url: str
    token: str | None
    token_secret_name: str
    token_secret_key: str
    account_id: str
    channel_id: str
    thread_id: str | None
    message_id: str
    project_id: str
    task_id: str
    agent_id: str
    service: str
    repository_url: str
    docs_repository_url: str
    model: str
    cli_type: str
    provider: str | None
    prompt: str
    wait_timeout: int
    keep_coderun: bool
    keep_namespace: bool
    create_namespace: bool

    @property
    def secrets(self) -> list[str]:
        return [self.token or ""]


def load_config(args: argparse.Namespace) -> Config:
    run_id = optional_env("SMOKE_RUN_ID") or f"hermes-coderun-{rand_suffix()}"
    run_id = sanitize_k8s_name(run_id, 40)
    namespace = optional_env("SMOKE_NAMESPACE", "cto") or "cto"
    coderun_name = sanitize_k8s_name(optional_env("SMOKE_CODERUN_NAME", run_id) or run_id)
    task_id = optional_env("SMOKE_TASK_ID", "1") or "1"
    channel_id = optional_env("SMOKE_DISCORD_CHANNEL_ID") or f"hermes-coderun-channel-{run_id}"
    return Config(
        mode=args.mode or optional_env("SMOKE_MODE", "dry-run") or "dry-run",
        namespace=namespace,
        run_id=run_id,
        coderun_name=coderun_name,
        router_url=(optional_env("PRESENCE_ROUTER_URL", DEFAULT_ROUTER_URL) or DEFAULT_ROUTER_URL).rstrip("/"),
        token=optional_env("PRESENCE_SHARED_TOKEN"),
        token_secret_name=optional_env("PRESENCE_TOKEN_SECRET_NAME", DEFAULT_TOKEN_SECRET) or DEFAULT_TOKEN_SECRET,
        token_secret_key=optional_env("PRESENCE_TOKEN_SECRET_KEY", DEFAULT_TOKEN_KEY) or DEFAULT_TOKEN_KEY,
        account_id=optional_env("SMOKE_DISCORD_ACCOUNT_ID", "control-plane-smoke-account") or "control-plane-smoke-account",
        channel_id=channel_id,
        thread_id=optional_env("SMOKE_DISCORD_THREAD_ID"),
        message_id=f"{run_id}-msg-1",
        project_id=optional_env("SMOKE_PROJECT_ID", "presence-smoke") or "presence-smoke",
        task_id=task_id,
        agent_id=optional_env("SMOKE_AGENT_ID", "rex") or "rex",
        service=optional_env("SMOKE_SERVICE", "presence-smoke") or "presence-smoke",
        repository_url=optional_env("SMOKE_REPOSITORY_URL", "https://github.com/5dlabs/cto") or "https://github.com/5dlabs/cto",
        docs_repository_url=optional_env("SMOKE_DOCS_REPOSITORY_URL", "https://github.com/5dlabs/cto") or "https://github.com/5dlabs/cto",
        model=optional_env("SMOKE_MODEL", "gpt-5-codex") or "gpt-5-codex",
        cli_type=optional_env("SMOKE_CLI_TYPE", "codex") or "codex",
        provider=optional_env("SMOKE_PROVIDER"),
        prompt=optional_env(
            "SMOKE_PROMPT",
            "Hermes presence smoke: acknowledge this synthetic Discord event and do not modify files.",
        ) or "Hermes presence smoke: acknowledge this synthetic Discord event and do not modify files.",
        wait_timeout=int(optional_env("SMOKE_WAIT_TIMEOUT", "240") or "240"),
        keep_coderun=bool_env("SMOKE_KEEP_CODERUN", False),
        keep_namespace=bool_env("SMOKE_KEEP_NAMESPACE", False),
        create_namespace=bool_env("SMOKE_CREATE_NAMESPACE", False),
    )


def run(cmd: list[str], *, input_text: str | None = None, check: bool = True, cfg: Config | None = None) -> subprocess.CompletedProcess[str]:
    printable = " ".join(cmd)
    print(f"[smoke] $ {redact_text(printable, cfg.secrets if cfg else [])}")
    proc = subprocess.run(cmd, input=input_text, text=True, capture_output=True, check=False)
    if check and proc.returncode != 0:
        stdout = redact_text(proc.stdout, cfg.secrets if cfg else [])
        stderr = redact_text(proc.stderr, cfg.secrets if cfg else [])
        raise SystemExit(f"command failed ({proc.returncode}): {printable}\nSTDOUT:\n{stdout}\nSTDERR:\n{stderr}")
    return proc


def kubectl(cfg: Config, *args: str, input_text: str | None = None, check: bool = True) -> subprocess.CompletedProcess[str]:
    return run(["kubectl", *args], input_text=input_text, check=check, cfg=cfg)


def request(cfg: Config, method: str, path: str, payload: dict[str, Any] | None = None, *, auth: bool = True, timeout: int = 20) -> tuple[int, str]:
    data = None if payload is None else json.dumps(payload).encode("utf-8")
    headers = {"Content-Type": "application/json"}
    if auth:
        if not cfg.token:
            raise SystemExit("PRESENCE_SHARED_TOKEN is required for router HTTP calls in live mode")
        headers["Authorization"] = f"Bearer {cfg.token}"
    req = urllib.request.Request(f"{cfg.router_url}{path}", data=data, headers=headers, method=method)
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            return resp.status, resp.read().decode("utf-8")
    except urllib.error.HTTPError as exc:
        return exc.code, exc.read().decode("utf-8", errors="replace")


def coderun_manifest(cfg: Config) -> str:
    manifest: dict[str, Any] = {
        "apiVersion": "agents.platform/v1",
        "kind": "CodeRun",
        "metadata": {
            "name": cfg.coderun_name,
            "namespace": cfg.namespace,
            "labels": {
                "app": "controller",
                "component": "code-runner",
                "smoke.5dlabs.ai/type": "hermes-presence-coderun-e2e",
                "smoke.5dlabs.ai/run-id": cfg.run_id,
            },
        },
        "spec": {
            "runType": "implementation",
            "taskId": int(cfg.task_id) if cfg.task_id.isdigit() else cfg.task_id,
            "projectId": cfg.project_id,
            "service": cfg.service,
            "repositoryUrl": cfg.repository_url,
            "docsRepositoryUrl": cfg.docs_repository_url,
            "workingDirectory": ".",
            "implementationAgent": cfg.agent_id,
            "githubApp": f"5DLabs-{cfg.agent_id.capitalize()}",
            "harnessAgent": "hermes",
            "model": cfg.model,
            "enableDocker": False,
            "enableCodeServer": False,
            "quality": False,
            "security": False,
            "testing": False,
            "deployment": False,
            "cliConfig": {
                "cliType": cfg.cli_type,
                "model": cfg.model,
            },
            "env": {
                "DISCORD_ACCOUNT_ID": cfg.account_id,
                "PRESENCE_DISCORD_CHANNEL_ID": cfg.channel_id,
                "SMOKE_RUN_ID": cfg.run_id,
            },
            "promptModification": cfg.prompt,
            "acceptanceCriteria": "- [ ] Synthetic smoke only; no code changes required.",
        },
    }
    if cfg.provider:
        manifest["spec"]["cliConfig"]["provider"] = cfg.provider
    if cfg.thread_id:
        manifest["spec"]["env"]["DISCORD_THREAD_ID"] = cfg.thread_id
    return to_yaml(manifest)


def yaml_scalar(value: Any) -> str:
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, int):
        return str(value)
    if value is None:
        return "null"
    return json.dumps(str(value))


def to_yaml(value: Any, indent: int = 0) -> str:
    pad = " " * indent
    if isinstance(value, dict):
        lines: list[str] = []
        for key, item in value.items():
            if isinstance(item, (dict, list)):
                lines.append(f"{pad}{key}:")
                lines.append(to_yaml(item, indent + 2))
            else:
                lines.append(f"{pad}{key}: {yaml_scalar(item)}")
        return "\n".join(lines) + ("\n" if indent == 0 else "")
    if isinstance(value, list):
        lines = []
        for item in value:
            if isinstance(item, (dict, list)):
                lines.append(f"{pad}-")
                lines.append(to_yaml(item, indent + 2))
            else:
                lines.append(f"{pad}- {yaml_scalar(item)}")
        return "\n".join(lines)
    return f"{pad}{yaml_scalar(value)}"


def event_payload(cfg: Config) -> dict[str, Any]:
    discord: dict[str, Any] = {
        "account_id": cfg.account_id,
        "channel_id": cfg.channel_id,
        "message_id": cfg.message_id,
        "user_id": "presence-smoke-user",
        "user_name": "Presence Smoke",
        "chat_type": "group",
    }
    if cfg.thread_id:
        discord["thread_id"] = cfg.thread_id
    return {
        "schema": "cto.presence.v1",
        "event_type": "message",
        "runtime": "hermes",
        "agent_id": cfg.agent_id,
        "project_id": cfg.project_id,
        "task_id": cfg.task_id,
        "coderun_id": cfg.coderun_name,
        "discord": discord,
        "text": f"Hermes CodeRun presence E2E smoke {cfg.run_id}",
        "metadata": {"smoke_run_id": cfg.run_id, "source": "presence-smoke-hermes-coderun.py"},
    }


def print_dry_run(cfg: Config) -> int:
    print("[smoke] mode=dry-run (no cluster mutation)")
    print(f"[smoke] router={cfg.router_url}")
    print(f"[smoke] namespace={cfg.namespace}")
    print(f"[smoke] coderun={cfg.coderun_name}")
    print("\n--- rendered CodeRun manifest ---")
    print(redact_text(coderun_manifest(cfg), cfg.secrets))
    print("--- synthetic inbound payload ---")
    print(json.dumps(event_payload(cfg), indent=2))
    print(
        "\nTo run live: export PRESENCE_SHARED_TOKEN (or port-forward/use in-cluster router), "
        "then SMOKE_MODE=live scripts/presence-smoke-hermes-coderun.py"
    )
    return 0


def wait_for_route(cfg: Config) -> dict[str, Any]:
    deadline = time.time() + cfg.wait_timeout
    last_body = ""
    while time.time() < deadline:
        status, body = request(cfg, "GET", "/presence/routes")
        last_body = body
        if status != 200:
            raise SystemExit(f"/presence/routes returned HTTP {status}: {redact_text(body, cfg.secrets)}")
        routes = json.loads(body or "{}").get("routes", [])
        for route in routes:
            if route.get("route_id") == cfg.coderun_name:
                print(f"[smoke] route registered: {cfg.coderun_name}")
                return route
        time.sleep(3)
    raise SystemExit(f"timed out waiting for route {cfg.coderun_name}; last routes body: {redact_text(last_body, cfg.secrets)}")


def adapter_pod_from_selector(cfg: Config, selector: str) -> str | None:
    proc = kubectl(cfg, "-n", cfg.namespace, "get", "pod", "-l", selector, "-o", "json", check=False)
    if proc.returncode != 0:
        return None
    items = json.loads(proc.stdout or "{}").get("items", [])
    items.sort(key=lambda pod: pod.get("metadata", {}).get("creationTimestamp", ""), reverse=True)
    for pod in items:
        names = [c.get("name") for c in pod.get("spec", {}).get("containers", [])]
        if "hermes-presence-adapter" in names:
            return pod.get("metadata", {}).get("name")
    return None


def wait_for_adapter_pod(cfg: Config) -> str | None:
    # The controller-rendered Job/Pod may not preserve CR-level smoke labels.
    # Prefer the explicit smoke label, then fall back to labels known to be
    # applied by the CodeRun controller so live smokes can collect adapter-log
    # evidence without guessing pod names or printing secrets.
    selectors = [
        f"smoke.5dlabs.ai/run-id={cfg.run_id}",
        f"cleanup.5dlabs.ai/run={cfg.coderun_name}",
        f"app=controller,component=code-runner,service={cfg.service}",
    ]
    deadline = time.time() + cfg.wait_timeout
    while time.time() < deadline:
        for selector in selectors:
            pod_name = adapter_pod_from_selector(cfg, selector)
            if pod_name:
                print(f"[smoke] adapter pod discovered with selector {selector}: {pod_name}")
                return pod_name
        time.sleep(3)
    return None


def cleanup(cfg: Config) -> None:
    if cfg.mode != "live":
        return
    if not cfg.keep_coderun:
        kubectl(cfg, "-n", cfg.namespace, "delete", "coderun", cfg.coderun_name, "--ignore-not-found", check=False)
    if cfg.token:
        try:
            request(cfg, "DELETE", f"/presence/routes/{cfg.coderun_name}")
        except Exception:
            pass
    if cfg.create_namespace and not cfg.keep_namespace and cfg.namespace != "cto":
        kubectl(cfg, "delete", "namespace", cfg.namespace, "--ignore-not-found", check=False)


def live(cfg: Config) -> int:
    print("[smoke] mode=live")
    if not cfg.token:
        raise SystemExit("PRESENCE_SHARED_TOKEN is required for live mode; it is never printed")

    status, body = request(cfg, "GET", "/health", auth=False)
    if status != 200:
        raise SystemExit(f"health failed: HTTP {status} {body}")
    unauth_status, _ = request(cfg, "GET", "/presence/routes", auth=False)
    if unauth_status != 401:
        raise SystemExit(f"expected unauthenticated /presence/routes=401, got {unauth_status}")

    if cfg.create_namespace:
        ns_yaml = kubectl(cfg, "create", "namespace", cfg.namespace, "--dry-run=client", "-o", "yaml").stdout
        kubectl(cfg, "apply", "-f", "-", input_text=ns_yaml)

    # Optional convenience for disposable namespaces. In normal cto namespace use
    # the existing presence token secret configured on the controller.
    if cfg.create_namespace and cfg.token:
        secret_yaml = kubectl(
            cfg,
            "-n",
            cfg.namespace,
            "create",
            "secret",
            "generic",
            cfg.token_secret_name,
            f"--from-literal={cfg.token_secret_key}={cfg.token}",
            "--dry-run=client",
            "-o",
            "yaml",
        ).stdout
        kubectl(cfg, "apply", "-f", "-", input_text=secret_yaml)

    manifest = coderun_manifest(cfg)
    with tempfile.NamedTemporaryFile("w", suffix=".yaml", delete=False) as tmp:
        tmp.write(manifest)
        tmp_path = tmp.name
    try:
        kubectl(cfg, "apply", "-f", tmp_path)
    finally:
        Path(tmp_path).unlink(missing_ok=True)

    route = wait_for_route(cfg)
    worker_url = route.get("worker_url", "")
    route_summary = {
        "route_id": route.get("route_id"),
        "runtime": route.get("runtime"),
        "agent_id": route.get("agent_id"),
        "coderun_id": route.get("coderun_id"),
        "project_id": route.get("project_id"),
        "task_id": route.get("task_id"),
        "worker_url_present": bool(worker_url),
    }
    print(f"[smoke] route summary: {json.dumps(route_summary, sort_keys=True)}")
    if not worker_url:
        raise SystemExit(f"registered route missing worker_url: {json.dumps(route, indent=2)}")

    payload = event_payload(cfg)
    print("[smoke] posting synthetic Discord event through /presence/inbound")
    status, body = request(cfg, "POST", "/presence/inbound", payload)
    if status != 202:
        raise SystemExit(f"/presence/inbound expected 202, got HTTP {status}: {redact_text(body, cfg.secrets)}")

    pod_name = wait_for_adapter_pod(cfg)
    if pod_name:
        logs = kubectl(cfg, "-n", cfg.namespace, "logs", f"pod/{pod_name}", "-c", "hermes-presence-adapter", "--tail=200", check=False).stdout
        redacted_logs = redact_text(logs, cfg.secrets)
        if cfg.run_id in redacted_logs or cfg.coderun_name in redacted_logs or "queued" in redacted_logs:
            print(f"[smoke] adapter pod observed: {pod_name}")
        else:
            print(f"[smoke] adapter pod observed without definitive smoke marker in tail logs: {pod_name}")
    else:
        print("[smoke] route delivered, but adapter pod was not found by smoke label before timeout")

    print("[smoke] passed")
    return 0


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mode", choices=["dry-run", "live"], help="Override SMOKE_MODE")
    return parser.parse_args()


def main() -> int:
    cfg = load_config(parse_args())
    if cfg.mode not in {"dry-run", "live"}:
        raise SystemExit("SMOKE_MODE must be dry-run or live")
    try:
        if cfg.mode == "dry-run":
            return print_dry_run(cfg)
        return live(cfg)
    finally:
        cleanup(cfg)


if __name__ == "__main__":
    raise SystemExit(main())
