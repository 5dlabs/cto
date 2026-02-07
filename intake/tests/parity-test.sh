#!/usr/bin/env bash
# Phase 1 Parity Test: Compare old intake-agent vs new intake-util output.
# Both systems should produce byte-identical files on disk.

set -euo pipefail

OLD_BINARY="/Users/jonathonfritz/agents/lobster-intake/apps/intake-agent/dist/intake-agent"
NEW_BINARY="/Users/jonathonfritz/agents/lobster-intake/intake/util/intake-util"
FIXTURES_DIR="/Users/jonathonfritz/agents/lobster-intake/intake/tests/fixtures"

PASS=0
FAIL=0
ERRORS=""

run_test() {
  local size="$1"
  local fixture="$FIXTURES_DIR/tasks-${size}.json"

  if [ ! -f "$fixture" ]; then
    echo "SKIP: $fixture not found"
    return
  fi

  local old_docs="/tmp/parity-old-docs-${size}"
  local new_docs="/tmp/parity-new-docs-${size}"
  local old_prompts="/tmp/parity-old-prompts-${size}"
  local new_prompts="/tmp/parity-new-prompts-${size}"

  # Clean output directories
  rm -rf "$old_docs" "$new_docs" "$old_prompts" "$new_prompts"

  # Read tasks content
  local tasks
  tasks=$(cat "$fixture")

  # --- Run old system (stdin JSON envelope protocol) ---
  echo "{\"operation\":\"generate_docs\",\"payload\":{\"tasks\":${tasks},\"base_path\":\"${old_docs}\",\"project_root\":\"${old_docs}\"}}" \
    | "$OLD_BINARY" > /dev/null 2>&1

  echo "{\"operation\":\"generate_prompts\",\"payload\":{\"tasks\":${tasks},\"output_dir\":\"${old_prompts}\",\"project_name\":\"test\"}}" \
    | "$OLD_BINARY" > /dev/null 2>&1

  # --- Run new system (CLI subcommands) ---
  "$NEW_BINARY" generate-docs --task-json "$fixture" --base-path "$new_docs" > /dev/null 2>&1

  "$NEW_BINARY" generate-prompts --task-json "$fixture" --output-dir "$new_prompts" --project-name test > /dev/null 2>&1

  # --- Compare generate-docs output ---
  local docs_diff
  if docs_diff=$(diff -r "$old_docs" "$new_docs" 2>&1); then
    echo "  PASS: generate-docs ($size)"
    PASS=$((PASS + 1))
  else
    echo "  FAIL: generate-docs ($size)"
    ERRORS="${ERRORS}\n--- generate-docs ${size} ---\n${docs_diff}\n"
    FAIL=$((FAIL + 1))
  fi

  # --- Compare generate-prompts output ---
  local prompts_diff
  if prompts_diff=$(diff -r "$old_prompts" "$new_prompts" 2>&1); then
    echo "  PASS: generate-prompts ($size)"
    PASS=$((PASS + 1))
  else
    echo "  FAIL: generate-prompts ($size)"
    ERRORS="${ERRORS}\n--- generate-prompts ${size} ---\n${prompts_diff}\n"
    FAIL=$((FAIL + 1))
  fi
}

echo "=== Phase 1 Parity Tests ==="
echo ""

for size in small medium large; do
  echo "[$size fixture]"
  run_test "$size"
  echo ""
done

echo "=== Results ==="
echo "PASS: $PASS  FAIL: $FAIL  TOTAL: $((PASS + FAIL))"

if [ $FAIL -gt 0 ]; then
  echo ""
  echo "=== Diff Details ==="
  echo -e "$ERRORS"
  exit 1
fi

echo ""
echo "All parity tests passed."
exit 0
