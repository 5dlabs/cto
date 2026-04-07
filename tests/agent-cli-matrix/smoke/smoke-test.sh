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

# ── CLI definitions ──────────────────────────────────────────────────────────
declare -A CLI_MODELS=(
  [claude]="claude-sonnet-4-6"
  [codex]="gpt-5.4"
  [cursor]="composer-2"
  [factory]="claude-opus-4-5-20251101"
)

# Agents that support all 4 CLIs (from matrix.yaml)
ALL_CLI_AGENTS=(rex blaze grizz nova)

# GitHub App names
declare -A GITHUB_APPS=(
  [rex]="5DLabs-Rex"
  [blaze]="5DLabs-Blaze"
  [grizz]="5DLabs-Grizz"
  [nova]="5DLabs-Nova"
  [tap]="5DLabs-Tap"
  [spark]="5DLabs-Spark"
)

# ── Parse args ───────────────────────────────────────────────────────────────
DRY_RUN=false
FILTER_AGENT=""
FILTER_CLI=""
BATCH_SIZE=4
CLEAN=false

while [[ $# -gt 0 ]]; do
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
    awk '{print $1}' | xargs -r kubectl delete coderun -n "$NAMESPACE" || true
  echo "✓ Cleaned"
  exit 0
fi

# ── Generate CRDs ────────────────────────────────────────────────────────────
mkdir -p "$OUTPUT_DIR"
CRDS=()

generate_crd() {
  local agent="$1"
  local cli="$2"
  local model="${CLI_MODELS[$cli]}"
  local github_app="${GITHUB_APPS[$agent]}"
  local task_id=9000  # Smoke test task range
  local crd_name="smoke-${agent}-${cli}-${RUN_ID##*-}"
  local file="$OUTPUT_DIR/${crd_name}.yaml"

  # Smoke test prompt — asks agent to create an identifying file + PR
  local prompt="SMOKE TEST — CLI Verification

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

Keep it simple — this is just verifying the CLI harness works end-to-end."

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
$(echo "$prompt" | sed 's/^/    /')
YAML

  CRDS+=("$file")
  echo "  ✓ ${crd_name} → ${file##*/}"
}

echo "═══════════════════════════════════════════════════════════════"
echo "║ Multi-CLI Smoke Test Generator"
echo "║ Run ID: ${RUN_ID}"
echo "║ Target: ${TARGET_REPO}"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# Determine which agents and CLIs to test
AGENTS=("${ALL_CLI_AGENTS[@]}")
CLIS=(claude codex cursor factory)

if [[ -n "$FILTER_AGENT" ]]; then
  AGENTS=("$FILTER_AGENT")
fi
if [[ -n "$FILTER_CLI" ]]; then
  CLIS=("$FILTER_CLI")
fi

echo "Agents: ${AGENTS[*]}"
echo "CLIs:   ${CLIS[*]}"
echo "Total:  $((${#AGENTS[@]} * ${#CLIS[@]})) CodeRuns"
echo ""

echo "Generating CRDs..."
for agent in "${AGENTS[@]}"; do
  for cli in "${CLIS[@]}"; do
    generate_crd "$agent" "$cli"
  done
done

echo ""
echo "Generated ${#CRDS[@]} CRDs in $OUTPUT_DIR/"

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
for file in "${CRDS[@]}"; do
  kubectl apply -f "$file" 2>&1
  APPLIED=$((APPLIED + 1))

  if (( APPLIED % BATCH_SIZE == 0 )) && (( APPLIED < ${#CRDS[@]} )); then
    echo ""
    echo "⏳ Batch $((APPLIED / BATCH_SIZE)) applied ($APPLIED/${#CRDS[@]}). Waiting 30s..."
    sleep 30
  fi
done

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "║ ✅ Applied ${#CRDS[@]} CodeRuns"
echo "║ Monitor: kubectl get coderun -n ${NAMESPACE} -l smoke-run-id=${RUN_ID}"
echo "║ Logs:    kubectl logs -n ${NAMESPACE} -l smoke-run-id=${RUN_ID} -f"
echo "║ Clean:   $0 --clean"
echo "═══════════════════════════════════════════════════════════════"
