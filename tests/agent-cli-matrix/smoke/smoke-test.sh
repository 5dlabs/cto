#!/usr/bin/env bash
# ============================================================================
# Multi-CLI Smoke Test — CodeRun Generator
#
# Creates CodeRun CRDs to test each agent × CLI combination.
# Each agent gets a simple task and creates a PR identifying the CLI + model.
#
# Usage:
#   ./smoke-test.sh                    # Generate + apply all CRDs
#   ./smoke-test.sh --dry-run          # Generate only, don't apply
#   ./smoke-test.sh --agent rex        # Single agent, all CLIs
#   ./smoke-test.sh --cli claude       # All agents, single CLI
#   ./smoke-test.sh --batch 4          # Apply in batches of N
#   ./smoke-test.sh --clean            # Delete all smoke test CodeRuns
# ============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
OUTPUT_DIR="$SCRIPT_DIR/generated"
NAMESPACE="openclaw"
TARGET_REPO="https://github.com/5dlabs/sigma-1.git"
DOCS_REPO="https://github.com/5dlabs/sigma-1.git"
SERVICE="sigma-1"
RUN_ID="smoke-$(date +%Y%m%d-%H%M%S)"

# ── Lookup helpers (bash 3.2 compatible — no associative arrays) ─────────────
get_model() {
  case "$1" in
    claude)  echo "claude-sonnet-4-6" ;;
    codex)   echo "gpt-5.4" ;;
    cursor)  echo "composer-2" ;;
    factory) echo "claude-opus-4-5-20251101" ;;
    *)       echo "unknown"; return 1 ;;
  esac
}

get_github_app() {
  case "$1" in
    rex)   echo "5DLabs-Rex" ;;
    blaze) echo "5DLabs-Blaze" ;;
    grizz) echo "5DLabs-Grizz" ;;
    nova)  echo "5DLabs-Nova" ;;
    tap)   echo "5DLabs-Tap" ;;
    spark) echo "5DLabs-Spark" ;;
    *)     echo "5DLabs-${1^}" ;;
  esac
}

# Agents that support all 4 CLIs (from matrix.yaml)
ALL_CLI_AGENTS="rex blaze grizz nova"

# ── Parse args ───────────────────────────────────────────────────────────────
DRY_RUN=false
FILTER_AGENT=""
FILTER_CLI=""
BATCH_SIZE=4
CLEAN=false

while [ $# -gt 0 ]; do
  case "$1" in
    --dry-run)   DRY_RUN=true; shift ;;
    --agent)     FILTER_AGENT="$2"; shift 2 ;;
    --cli)       FILTER_CLI="$2"; shift 2 ;;
    --batch)     BATCH_SIZE="$2"; shift 2 ;;
    --clean)     CLEAN=true; shift ;;
    *)           echo "Unknown option: $1"; exit 1 ;;
  esac
done

# ── Clean mode ───────────────────────────────────────────────────────────────
if $CLEAN; then
  echo "🧹 Cleaning up smoke test CodeRuns..."
  kubectl get coderun -n "$NAMESPACE" -l "smoke-test=true" --no-headers 2>/dev/null | \
    awk '{print $1}' | xargs kubectl delete coderun -n "$NAMESPACE" 2>/dev/null || true
  echo "✓ Cleaned"
  exit 0
fi

# ── Generate CRDs ────────────────────────────────────────────────────────────
mkdir -p "$OUTPUT_DIR"
CRD_FILES=""
CRD_COUNT=0

generate_crd() {
  local agent="$1"
  local cli="$2"
  local model
  model="$(get_model "$cli")"
  local github_app
  github_app="$(get_github_app "$agent")"
  local task_id=9000
  local crd_name="smoke-${agent}-${cli}-${RUN_ID##*-}"
  local file="$OUTPUT_DIR/${crd_name}.yaml"

  cat > "$file" << YAML
apiVersion: agents.platform/v1
kind: CodeRun
metadata:
  name: ${crd_name}
  namespace: ${NAMESPACE}
  labels:
    smoke-test: "true"
    smoke-run-id: "${RUN_ID}"
    agent: "${agent}"
    cli-type: "${cli}"
spec:
  runType: implementation
  taskId: ${task_id}
  service: ${SERVICE}
  repositoryUrl: ${TARGET_REPO}
  docsRepositoryUrl: ${DOCS_REPO}
  workingDirectory: /workspace
  model: ${model}
  githubApp: ${github_app}
  cliConfig:
    cliType: ${cli}
    model: ${model}
    settings: {}
  env:
    SMOKE_TEST: "true"
    SMOKE_RUN_ID: "${RUN_ID}"
    CTO_CLI_TYPE: "${cli}"
    CTO_CLI_MODEL: "${model}"
  promptModification: |
    SMOKE TEST — CLI Verification

    You are ${agent} running via the ${cli} CLI with model ${model}.

    Your task:
    1. Create a file 'smoke-test/${agent}-${cli}.md' with this content:
       - Agent: ${agent}
       - CLI: ${cli}
       - Model: ${model}
       - Timestamp: (current UTC time)
       - A one-line description of your agent role
    2. Commit with message: 'smoke(${agent}): verify ${cli} CLI [model: ${model}]'
    3. Create a PR with title: '[Smoke] ${agent} via ${cli} (${model})'
    4. In the PR body, include:
       - CLI type: ${cli}
       - Model: ${model}
       - Agent: ${agent}
       - Run ID: ${RUN_ID}

    Keep it simple — this is just verifying the CLI harness works end-to-end.
YAML

  CRD_FILES="$CRD_FILES $file"
  CRD_COUNT=$((CRD_COUNT + 1))
  echo "  ✓ ${crd_name} → ${file##*/}"
}

echo "═══════════════════════════════════════════════════════════════"
echo "║ Multi-CLI Smoke Test Generator"
echo "║ Run ID: ${RUN_ID}"
echo "║ Target: ${TARGET_REPO}"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# Determine which agents and CLIs to test
AGENTS="${FILTER_AGENT:-$ALL_CLI_AGENTS}"
CLIS="${FILTER_CLI:-claude codex cursor factory}"

AGENT_COUNT=0
for _ in $AGENTS; do AGENT_COUNT=$((AGENT_COUNT + 1)); done
CLI_COUNT=0
for _ in $CLIS; do CLI_COUNT=$((CLI_COUNT + 1)); done

echo "Agents: ${AGENTS}"
echo "CLIs:   ${CLIS}"
echo "Total:  $((AGENT_COUNT * CLI_COUNT)) CodeRuns"
echo ""

echo "Generating CRDs..."
for agent in $AGENTS; do
  for cli in $CLIS; do
    generate_crd "$agent" "$cli"
  done
done

echo ""
echo "Generated ${CRD_COUNT} CRDs in $OUTPUT_DIR/"

# ── Apply CRDs ──────────────────────────────────────────────────────────────
if $DRY_RUN; then
  echo ""
  echo "🏁 Dry run — CRDs generated but not applied."
  echo "   Review: ls $OUTPUT_DIR/"
  echo "   Apply:  kubectl apply -f $OUTPUT_DIR/"
  exit 0
fi

echo ""
echo "Applying CRDs in batches of $BATCH_SIZE..."
APPLIED=0
for file in $CRD_FILES; do
  kubectl apply -f "$file" 2>&1
  APPLIED=$((APPLIED + 1))

  remainder=$((APPLIED % BATCH_SIZE))
  if [ "$remainder" -eq 0 ] && [ "$APPLIED" -lt "$CRD_COUNT" ]; then
    echo ""
    echo "⏳ Batch $((APPLIED / BATCH_SIZE)) applied ($APPLIED/${CRD_COUNT}). Waiting 30s..."
    sleep 30
  fi
done

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "║ ✅ Applied ${CRD_COUNT} CodeRuns"
echo "║ Monitor: kubectl get coderun -n ${NAMESPACE} -l smoke-run-id=${RUN_ID}"
echo "║ Logs:    kubectl logs -n ${NAMESPACE} -l smoke-run-id=${RUN_ID} -f"
echo "║ Clean:   $0 --clean"
echo "═══════════════════════════════════════════════════════════════"
