#!/usr/bin/env bash
# Pipeline monitoring loop — prints status every 2 minutes
set -uo pipefail
cd "${WORKSPACE:-/Users/jonathon/5dlabs/cto}"

LOGS=".intake/logs"

while true; do
  echo "═══════════════════════════════════════════════════"
  echo "  SIGMA-1 PIPELINE STATUS  $(date +%H:%M:%S)"
  echo "═══════════════════════════════════════════════════"

  STEP=$(head -1 "$LOGS/.step-timing" 2>/dev/null || echo "?")
  PHASE=$(sed -n '2p' "$LOGS/.step-timing" 2>/dev/null || echo "?")
  echo "  Phase: $PHASE  |  Step: $STEP"

  starts=$(grep -c step_start "$LOGS/pipeline.jsonl" 2>/dev/null || echo 0)
  ends=$(grep -c step_end "$LOGS/pipeline.jsonl" 2>/dev/null || echo 0)
  active=$((starts - ends))
  llm_total=$(grep -c '"llm_call"' "$LOGS/llm-calls.jsonl" 2>/dev/null || echo 0)
  llm_ok=$(grep '"llm_call"' "$LOGS/llm-calls.jsonl" 2>/dev/null | grep -c '"ok"' || echo 0)
  llm_err=$(grep '"llm_call"' "$LOGS/llm-calls.jsonl" 2>/dev/null | grep -c '"error"' || echo 0)
  echo "  Steps: $starts started / $ends done / $active active"
  echo "  LLM:   $llm_total calls ($llm_ok ok, $llm_err err)"

  echo "  ───────────────────────────────────────────────"
  tail -3 "$LOGS/pipeline.jsonl" 2>/dev/null | python3 -c "
import sys, json
for line in sys.stdin:
    try:
        e = json.loads(line.strip())
        ev = e.get('event','?')
        step = e.get('step_id','?')
        dur = e.get('duration_ms')
        ts = e.get('ts','?')[11:19]
        if dur: print(f'  {ts}  {ev:12s}  {step:35s}  {dur/1000:.0f}s')
        else: print(f'  {ts}  {ev:12s}  {step}')
    except: pass
"

  if grep -q run_complete "$LOGS/runs.jsonl" 2>/dev/null; then
    echo ""
    grep run_complete "$LOGS/runs.jsonl" | python3 -c "
import sys, json
e = json.loads(sys.stdin.readline().strip())
ec = e.get('exit_code', '?')
dur = e.get('duration_sec', '?')
icon = 'PASS' if ec == 0 else 'FAIL'
print(f'  >>> RUN COMPLETE: {icon}  exit={ec}  duration={dur}s')
"
    break
  fi

  alive=$(ps aux 2>/dev/null | grep 'lobster.*pipeline.lobster' | grep -v grep | wc -l)
  if [ "$alive" -eq 0 ]; then
    echo "  >>> WARNING: No lobster process found!"
  fi

  echo ""
  sleep 120
done
