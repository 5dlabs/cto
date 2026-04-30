#!/usr/bin/env bash
set -euo pipefail

ROUTER_URL="${PRESENCE_ROUTER_URL:-http://127.0.0.1:3200}"
TOKEN="${PRESENCE_SHARED_TOKEN:-}"
MODE="${PRESENCE_SMOKE_MODE:-cluster}"
NS="${PRESENCE_SMOKE_NAMESPACE:-cto-presence-smoke}"
ROUTE_ID="presence-smoke-$(date +%s)"
AGENT_ID="${PRESENCE_SMOKE_AGENT_ID:-rex}"
CODERUN_ID="${PRESENCE_SMOKE_CODERUN_ID:-presence-smoke-coderun}"
PROJECT_ID="${PRESENCE_SMOKE_PROJECT_ID:-presence-smoke-project}"
TASK_ID="${PRESENCE_SMOKE_TASK_ID:-presence-smoke-task}"
CHANNEL_ID="${PRESENCE_SMOKE_CHANNEL_ID:-presence-smoke-channel}"
ACCOUNT_ID="${PRESENCE_SMOKE_ACCOUNT_ID:-presence-smoke-account}"
EVENT_TEXT="presence smoke $(date +%s)"

if [[ -z "${TOKEN}" ]]; then
  echo "PRESENCE_SHARED_TOKEN is required" >&2
  exit 2
fi

auth_curl() {
  curl -fsS -H "Authorization: Bearer ${TOKEN}" "$@"
}

cleanup_route() {
  auth_curl -X DELETE "${ROUTER_URL}/presence/routes/${ROUTE_ID}" >/dev/null 2>&1 || true
}

cleanup_cluster() {
  if [[ "${MODE}" == "cluster" ]]; then
    kubectl delete namespace "${NS}" --ignore-not-found >/dev/null 2>&1 || true
  fi
}

trap 'cleanup_route; cleanup_cluster' EXIT

echo "Checking presence router health at ${ROUTER_URL}"
curl -fsS "${ROUTER_URL}/health" >/dev/null

echo "Verifying presence auth boundary"
unauth_code="$(curl -sS -o /dev/null -w '%{http_code}' "${ROUTER_URL}/presence/routes")"
if [[ "${unauth_code}" != "401" ]]; then
  echo "Expected unauthenticated /presence/routes to return 401, got ${unauth_code}" >&2
  exit 1
fi
auth_curl "${ROUTER_URL}/presence/routes" >/dev/null

if [[ "${MODE}" == "cluster" ]]; then
  echo "Starting disposable in-cluster worker ${NS}/presence-smoke-worker"
  kubectl create namespace "${NS}" --dry-run=client -o yaml | kubectl apply -f - >/dev/null
  kubectl -n "${NS}" create secret generic presence-smoke-token \
    --from-literal=token="${TOKEN}" \
    --dry-run=client -o yaml | kubectl apply -f - >/dev/null
  kubectl -n "${NS}" apply -f - >/dev/null <<'YAML'
apiVersion: v1
kind: Pod
metadata:
  name: presence-smoke-worker
  labels:
    app: presence-smoke-worker
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
      command:
        - node
        - -e
        - |
          const http = require("node:http");
          http.createServer((req, res) => {
            let body = "";
            req.on("data", (chunk) => body += chunk);
            req.on("end", () => {
              const expected = `Bearer ${process.env.EXPECT_TOKEN}`;
              if (req.headers.authorization !== expected) {
                res.writeHead(401, {"content-type": "application/json"});
                res.end(JSON.stringify({error: "unauthorized"}));
                return;
              }
              console.log(`PRESENCE_SMOKE_EVENT ${body}`);
              res.writeHead(202, {"content-type": "application/json"});
              res.end(JSON.stringify({accepted: true}));
            });
          }).listen(8080, "0.0.0.0");
      ports:
        - containerPort: 8080
---
apiVersion: v1
kind: Service
metadata:
  name: presence-smoke-worker
spec:
  selector:
    app: presence-smoke-worker
  ports:
    - port: 8080
      targetPort: 8080
YAML
  kubectl -n "${NS}" wait --for=condition=Ready pod/presence-smoke-worker --timeout=90s >/dev/null
  WORKER_URL="http://presence-smoke-worker.${NS}.svc:8080"
else
  WORKER_URL="${PRESENCE_SMOKE_WORKER_URL:?PRESENCE_SMOKE_WORKER_URL is required when PRESENCE_SMOKE_MODE is not cluster}"
fi

echo "Registering route ${ROUTE_ID}"
auth_curl -X POST "${ROUTER_URL}/presence/routes" \
  -H "Content-Type: application/json" \
  --data-binary @- >/dev/null <<JSON
{
  "route_id": "${ROUTE_ID}",
  "runtime": "hermes",
  "agent_id": "${AGENT_ID}",
  "project_id": "${PROJECT_ID}",
  "task_id": "${TASK_ID}",
  "coderun_id": "${CODERUN_ID}",
  "worker_url": "${WORKER_URL}",
  "session_key": "presence-smoke-session",
  "discord": {
    "account_id": "${ACCOUNT_ID}",
    "channel_id": "${CHANNEL_ID}"
  }
}
JSON

echo "Sending routed inbound event"
auth_curl -X POST "${ROUTER_URL}/presence/inbound" \
  -H "Content-Type: application/json" \
  --data-binary @- >/dev/null <<JSON
{
  "schema": "cto.presence.v1",
  "event_type": "message",
  "runtime": "hermes",
  "agent_id": "${AGENT_ID}",
  "project_id": "${PROJECT_ID}",
  "task_id": "${TASK_ID}",
  "coderun_id": "${CODERUN_ID}",
  "discord": {
    "account_id": "${ACCOUNT_ID}",
    "channel_id": "${CHANNEL_ID}",
    "message_id": "presence-smoke-message"
  },
  "text": "${EVENT_TEXT}"
}
JSON

if [[ "${MODE}" == "cluster" ]]; then
  echo "Verifying worker received the event"
  for _ in $(seq 1 20); do
    if kubectl -n "${NS}" logs pod/presence-smoke-worker | grep -F "${EVENT_TEXT}" >/dev/null; then
      echo "Presence smoke passed"
      exit 0
    fi
    sleep 1
  done
  echo "Timed out waiting for worker event log" >&2
  exit 1
fi

echo "Presence smoke passed"
