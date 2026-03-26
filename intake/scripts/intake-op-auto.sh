# shellcheck shell=bash
# Sourced by intake shell scripts after ROOT is set to the CTO repo root.
#
# If intake/local.env.op exists (or INTAKE_OP_ENV_FILE) and `op` is on PATH, re-exec the
# current script under `op run` exactly once (INTAKE_OP_WRAPPED prevents recursion).
#
# Disable: INTAKE_OP_AUTO_DISABLE=1

intake_op_auto_wrap() {
  local script_path="$1"
  shift
  local op_env="${INTAKE_OP_ENV_FILE:-$ROOT/intake/local.env.op}"
  [[ "${INTAKE_OP_AUTO_DISABLE:-}" == "1" || "${INTAKE_OP_AUTO_DISABLE:-}" == "true" ]] && return 0
  [[ -f "$op_env" ]] || return 0
  command -v op >/dev/null 2>&1 || {
    echo "intake: $op_env exists but 'op' is not on PATH — install 1Password CLI or set INTAKE_OP_AUTO_DISABLE=1" >&2
    return 0
  }
  [[ "${INTAKE_OP_WRAPPED:-}" == "1" ]] && return 0
  local abs
  abs="$(cd "$(dirname "$script_path")" && pwd)/$(basename "$script_path")"
  exec op run --env-file="$op_env" -- env INTAKE_OP_WRAPPED=1 bash "$abs" "$@"
}
