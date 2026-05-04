#!/usr/bin/env python3
"""Mini centralized Discord control-plane smoke for Morgan + a task channel.

This synthetic in-cluster smoke uses the deployed bridge contract without
requiring a live Discord message or a full CodeRun. It validates the route shape
we want for the end state:

- DM/home-style direct inbound to Morgan routes to Morgan.
- Task channel direct inbound to Rex/routes Sigma One task 1 routes to the task worker.
- Ambient task-channel Discord-style routing is expected to fail closed once the
  live bridge exposes /presence/discord-events.

Secrets are accepted via env and never printed.
"""
from __future__ import annotations

import json
import os
import random
import string
import subprocess
import textwrap
import time
import urllib.error
import urllib.request
from typing import Any


def getenv(name: str, default: str | None = None) -> str:
    value = os.environ.get(name, default)
    if value is None or value == "":
        raise SystemExit(f"{name} is required")
    return value


ROUTER_URL = os.environ.get("PRESENCE_ROUTER_URL", "http://discord-bridge-http.bots.svc:3200").rstrip("/")
TOKEN = getenv("PRESENCE_SHARED_TOKEN")
RUN_ID = os.environ.get("SMOKE_RUN_ID") or "morgan-task-" + "".join(random.choices(string.ascii_lowercase + string.digits, k=8))
NS = os.environ.get("SMOKE_NAMESPACE", f"cto-presence-smoke-{RUN_ID}")
ACCOUNT_ID = os.environ.get("SMOKE_ACCOUNT_ID", "control-plane-smoke-account")
DM_CHANNEL_ID = os.environ.get("SMOKE_DM_CHANNEL_ID", f"dm-{RUN_ID}")
TASK_CHANNEL_ID = os.environ.get("SMOKE_TASK_CHANNEL_ID", f"task-chan-sigma-1-task-1-{RUN_ID}")
PROJECT_ID = os.environ.get("SMOKE_PROJECT_ID", "sigma-1")
TASK_ID = os.environ.get("SMOKE_TASK_ID", "1")
MORGAN_ROUTE = f"{RUN_ID}-morgan"
TASK_ROUTE = f"{RUN_ID}-sigma-task-1"


def run(cmd: list[str], *, input_text: str | None = None, check: bool = True) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, input=input_text, text=True, capture_output=True, check=check)


def request(method: str, path: str, payload: dict[str, Any] | None = None, auth: bool = True) -> tuple[int, str]:
    data = None if payload is None else json.dumps(payload).encode()
    headers = {"Content-Type": "application/json"}
    if auth:
        headers["Authorization"] = f"Bearer {TOKEN}"
    req = urllib.request.Request(f"{ROUTER_URL}{path}", data=data, headers=headers, method=method)
    try:
        with urllib.request.urlopen(req, timeout=20) as resp:
            return resp.status, resp.read().decode()
    except urllib.error.HTTPError as exc:
        return exc.code, exc.read().decode(errors="replace")


def apply_yaml(yaml_text: str) -> None:
    run(["kubectl", "apply", "-f", "-"], input_text=yaml_text)


def kubectl(*args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
    return run(["kubectl", *args], check=check)


def cleanup() -> None:
    for route in (MORGAN_ROUTE, TASK_ROUTE):
        try:
            request("DELETE", f"/presence/routes/{route}")
        except Exception:
            pass
    kubectl("delete", "namespace", NS, "--ignore-not-found", check=False)


def post_inbound(name: str, payload: dict[str, Any], expected_status: int = 202) -> dict[str, Any]:
    status, body = request("POST", "/presence/inbound", payload)
    print(f"[smoke] {name}: HTTP {status}")
    if status != expected_status:
        raise SystemExit(f"{name}: expected HTTP {expected_status}, got {status}: {body}")
    return json.loads(body or "{}")


def main() -> int:
    print(f"[smoke] run_id={RUN_ID}")
    print(f"[smoke] router={ROUTER_URL}")
    print(f"[smoke] namespace={NS}")
    print(f"[smoke] project={PROJECT_ID} task={TASK_ID}")

    status, body = request("GET", "/health", auth=False)
    if status != 200:
        raise SystemExit(f"health failed: HTTP {status} {body}")

    unauth_status, _ = request("GET", "/presence/routes", auth=False)
    if unauth_status != 401:
        raise SystemExit(f"expected unauthenticated /presence/routes=401, got {unauth_status}")

    ns_yaml = run(["kubectl", "create", "namespace", NS, "--dry-run=client", "-o", "yaml"]).stdout
    apply_yaml(ns_yaml)

    secret_yaml = run([
        "kubectl", "-n", NS, "create", "secret", "generic", "presence-smoke-token",
        f"--from-literal=token={TOKEN}", "--dry-run=client", "-o", "yaml",
    ]).stdout
    apply_yaml(secret_yaml)

    worker = textwrap.dedent(
        f"""
        apiVersion: v1
        kind: Pod
        metadata:
          name: worker
          namespace: {NS}
          labels:
            app: morgan-task-smoke-worker
        spec:
          containers:
            - name: worker
              image: node:24-alpine
              env:
                - name: EXPECT_TOKEN
                  valueFrom:
                    secretKeyRef:
                      name: presence-smoke-token
                      key: token
              command: ["node", "-e"]
              args:
                - |
                  const http = require("node:http");
                  http.createServer((req, res) => {{
                    let body = "";
                    req.on("data", chunk => body += chunk);
                    req.on("end", () => {{
                      if (req.headers.authorization !== `Bearer ${{process.env.EXPECT_TOKEN}}`) {{
                        res.writeHead(401, {{"content-type":"application/json"}});
                        res.end(JSON.stringify({{error:"unauthorized"}}));
                        return;
                      }}
                      const event = JSON.parse(body || "{{}}");
                      console.log("MORGAN_TASK_SMOKE_EVENT " + JSON.stringify({{
                        agent_id: event.agent_id,
                        project_id: event.project_id,
                        task_id: event.task_id,
                        coderun_id: event.coderun_id,
                        channel_id: event.discord && event.discord.channel_id,
                        chat_type: event.discord && event.discord.chat_type,
                        session_key: event.session_key,
                        text: event.text
                      }}));
                      res.writeHead(202, {{"content-type":"application/json"}});
                      res.end(JSON.stringify({{accepted:true}}));
                    }});
                  }}).listen(8080, "0.0.0.0");
              ports:
                - containerPort: 8080
        ---
        apiVersion: v1
        kind: Service
        metadata:
          name: worker
          namespace: {NS}
        spec:
          selector:
            app: morgan-task-smoke-worker
          ports:
            - port: 8080
              targetPort: 8080
        """
    )
    apply_yaml(worker)
    kubectl("-n", NS, "wait", "--for=condition=Ready", "pod/worker", "--timeout=90s")
    # Give kube-proxy/endpoints a brief moment after Pod Ready; otherwise the bridge
    # can occasionally hit a just-created service before endpoints are routable.
    time.sleep(3)

    worker_url = f"http://worker.{NS}.svc:8080"
    routes = [
        {
            "route_id": MORGAN_ROUTE,
            "runtime": "hermes",
            "agent_id": "morgan",
            "project_id": PROJECT_ID,
            "worker_url": worker_url,
            "session_key": f"dm:{ACCOUNT_ID}:{DM_CHANNEL_ID}:morgan",
            "discord": {"account_id": ACCOUNT_ID, "channel_id": DM_CHANNEL_ID},
        },
        {
            "route_id": TASK_ROUTE,
            "runtime": "hermes",
            "agent_id": "rex",
            "project_id": PROJECT_ID,
            "task_id": TASK_ID,
            "coderun_id": f"sigma-1-task-{TASK_ID}-hermes-smoke",
            "worker_url": worker_url,
            "session_key": f"task:{PROJECT_ID}:{TASK_ID}:{TASK_CHANNEL_ID}",
            "discord": {"account_id": ACCOUNT_ID, "channel_id": TASK_CHANNEL_ID},
        },
    ]
    for route in routes:
        status, body = request("POST", "/presence/routes", route)
        if status not in (200, 201):
            raise SystemExit(f"route register failed {route['route_id']}: HTTP {status} {body}")

    post_inbound(
        "dm_to_morgan",
        {
            "schema": "cto.presence.v1",
            "event_type": "message",
            "runtime": "hermes",
            "agent_id": "morgan",
            "project_id": PROJECT_ID,
            "discord": {
                "account_id": ACCOUNT_ID,
                "channel_id": DM_CHANNEL_ID,
                "message_id": f"{RUN_ID}-dm-1",
                "user_id": "user-smoke",
                "user_name": "Smoke User",
                "chat_type": "dm",
            },
            "text": "Morgan DM smoke: personal/home context should route to Morgan only",
        },
    )

    post_inbound(
        "task_chan_to_rex",
        {
            "schema": "cto.presence.v1",
            "event_type": "message",
            "runtime": "hermes",
            "agent_id": "rex",
            "project_id": PROJECT_ID,
            "task_id": TASK_ID,
            "coderun_id": f"sigma-1-task-{TASK_ID}-hermes-smoke",
            "discord": {
                "account_id": ACCOUNT_ID,
                "channel_id": TASK_CHANNEL_ID,
                "message_id": f"{RUN_ID}-task-1",
                "user_id": "user-smoke",
                "user_name": "Smoke User",
                "chat_type": "group",
            },
            "text": "Task channel smoke: Sigma One task 1 should route to Rex task worker",
        },
    )

    post_inbound(
        "ambient_task_chan_fails_closed_direct_path",
        {
            "schema": "cto.presence.v1",
            "event_type": "message",
            "runtime": "hermes",
            "agent_id": "morgan",
            "project_id": PROJECT_ID,
            "task_id": TASK_ID,
            "discord": {
                "account_id": ACCOUNT_ID,
                "channel_id": TASK_CHANNEL_ID,
                "message_id": f"{RUN_ID}-ambient-1",
                "user_id": "user-smoke",
                "user_name": "Smoke User",
                "chat_type": "group",
            },
            "text": "Ambient task channel smoke: should not route to Morgan because Morgan route is DM scoped",
        },
        expected_status=404,
    )

    discord_event_status, discord_event_body = request("POST", "/presence/discord-events", {
        "schema": "cto.presence.v1",
        "event_type": "message",
        "project_id": PROJECT_ID,
        "task_id": TASK_ID,
        "discord": {
            "account_id": ACCOUNT_ID,
            "channel_id": TASK_CHANNEL_ID,
            "message_id": f"{RUN_ID}-ambient-discord-1",
            "user_id": "user-smoke",
            "user_name": "Smoke User",
            "chat_type": "group",
        },
        "text": "Ambient Discord-style task channel event should fail closed once endpoint is live",
    })
    if discord_event_status == 202:
        deliveries = len(json.loads(discord_event_body or "{}").get("deliveries", []))
        print(f"[smoke] discord-events endpoint live; ambient deliveries={deliveries}")
        if deliveries != 0:
            raise SystemExit(f"ambient Discord-style event unexpectedly delivered: {discord_event_body}")
    elif discord_event_status == 404:
        print("[smoke] discord-events endpoint is not in the currently deployed bridge image; skipped live ambient fanout check")
    else:
        raise SystemExit(f"unexpected /presence/discord-events HTTP {discord_event_status}: {discord_event_body}")

    time.sleep(1)
    logs = kubectl("-n", NS, "logs", "pod/worker").stdout
    for text in ["Morgan DM smoke", "Sigma One task 1"]:
        if text not in logs:
            raise SystemExit(f"worker logs missing expected text: {text}\n{logs}")
    if "Ambient task channel smoke" in logs:
        raise SystemExit("ambient task channel event unexpectedly reached worker")

    print("[smoke] passed")
    print(f"[smoke] DM channel modeled as {DM_CHANNEL_ID}; task chan modeled as {TASK_CHANNEL_ID}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    finally:
        if os.environ.get("SMOKE_KEEP_NAMESPACE") != "1":
            cleanup()
