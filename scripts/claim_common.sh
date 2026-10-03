#!/usr/bin/env bash
# Shared launcher for the paper's individually runnable claim scripts.
#
# Each claim computes exactly the measurements its paper checks use and nothing
# else: the reference-seeded roots and smallest Weil eigenvalue for run claims,
# and the natural evenness with the low parity-sector separations and the
# even-state spectral isolation for evenness claims. scripts/claim_summary.py
# then reports PASS or FAIL for the claim's documented checks.

CLAIM_REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$CLAIM_REPO_ROOT"

# A claim reads no research or target configuration. The Toolkit parses
# several XC_RESEARCH_* settings during primary admission, so every inherited
# XC_RESEARCH_* and XC_TARGET_* value is cleared; the claim's own working
# budget is then declared by its script (claim_working_budget).
for CLAIM_INHERITED_NAME in $(compgen -e); do
  case "$CLAIM_INHERITED_NAME" in
    XC_RESEARCH_*|XC_TARGET_*) unset "$CLAIM_INHERITED_NAME" ;;
  esac
done
unset CLAIM_INHERITED_NAME

# claim_working_budget GIB declares the Toolkit's primary admission budget for
# this claim. The Toolkit reads it as XC_RESEARCH_WORKING_BYTES for Tau and
# source assembly and for the ground-index proof, despite the variable's name.
# Each script's value covers the conservative admission estimate of every one
# of its configurations; run the claim on a machine with at least that much
# memory available.
claim_working_budget() {
  if [[ ! "${1:-}" =~ ^[1-9][0-9]{0,4}$ ]]; then
    echo "claim_working_budget requires a whole number of GiB" >&2
    exit 2
  fi
  export XC_RESEARCH_WORKING_BYTES=$(( $1 << 30 ))
}

PUBLICATION=${PUBLICATION:-none}
BENCHMARK_REPORT=${BENCHMARK_REPORT:-}
BENCHMARK_BASELINE=${BENCHMARK_BASELINE:-}
BENCHMARK_LABEL=${BENCHMARK_LABEL:-}
BENCHMARK_COMPARISON_MODE=${BENCHMARK_COMPARISON_MODE:-}
GL_ROOT_PARALLEL=${XC_GL_ROOT_PARALLEL:-false}
JOURNAL_ARGS=()
PREFLIGHT_ONLY=false
while (($# > 0)); do
  case "$1" in
    --preflight-only)
      PREFLIGHT_ONLY=true
      shift
      ;;
    --capture-output)
      if (($# < 2)); then echo "$1 requires a directory" >&2; exit 2; fi
      JOURNAL_ARGS+=("$1" "$2")
      shift 2
      ;;
    --publish)
      PUBLICATION=execute
      shift
      ;;
    --publish-plan)
      PUBLICATION=plan
      shift
      ;;
    --verify-cache)
      export XC_CACHE_MODE=verify
      shift
      ;;
    --parallel-gl-roots)
      GL_ROOT_PARALLEL=true
      shift
      ;;
    --benchmark-report|--benchmark-baseline|--benchmark-label|--benchmark-comparison-mode)
      if (($# < 2)); then echo "$1 requires a value" >&2; exit 2; fi
      case "$1" in
        --benchmark-report) BENCHMARK_REPORT=$2 ;;
        --benchmark-baseline) BENCHMARK_BASELINE=$2 ;;
        --benchmark-label) BENCHMARK_LABEL=$2 ;;
        --benchmark-comparison-mode) BENCHMARK_COMPARISON_MODE=$2 ;;
      esac
      shift 2
      ;;
    --help|-h)
      echo "Usage: bash ${BASH_SOURCE[1]} [--publish | --publish-plan] [--capture-output DIR] [--verify-cache] [--parallel-gl-roots] [--preflight-only] [--benchmark-report PATH [--benchmark-baseline PATH] [--benchmark-label LABEL] [--benchmark-comparison-mode MODE]]"
      echo "  Each claim runs its fixed configurations and computes only the measurements"
      echo "  its paper checks use. scripts/claim_summary.py reports PASS or FAIL."
      echo "  PUBLICATION: off by default. --publish publishes this claim's artifacts (author"
      echo "               credentials and XC_PUBLISH_AUTHOR_EMAIL required); --publish-plan stages"
      echo "               and plans the same publication without pushing. Claim 1a publishes to"
      echo "               public and private; every other claim publishes to private only."
      echo "  JOURNALS: --capture-output DIR is the parent of the private claim journals"
      echo "  PREFLIGHT: --preflight-only checks source admission without computing"
      echo "  CACHE VALIDATION: --verify-cache recomputes and compares claim artifacts; disabled by default"
      echo "  EXPERIMENTAL GL ROOTS: --parallel-gl-roots or XC_GL_ROOT_PARALLEL=true; native Linux only, never WSL"
      echo "  BENCHMARK: one process/report; gl-root-policy-delta permits only the intentional"
      echo "             serial/parallel GL-root difference"
      exit 0
      ;;
    *)
      echo "Unknown claim-script argument: $1" >&2
      echo "Use --help for supported controls." >&2
      exit 2
      ;;
  esac
done

# Publication destinations are fixed per claim. Claim 1a is the public
# headline reproduction; every other claim is private.
case "$(basename "${BASH_SOURCE[1]}" .sh)" in
  claim1a_lambda13) CLAIM_PUBLISH_TARGET=both ;;
  *) CLAIM_PUBLISH_TARGET=private ;;
esac
if [[ "$PREFLIGHT_ONLY" == "true" ]]; then
  PUBLICATION=none
fi
case "$PUBLICATION" in
  none)
    # Inherited managed-session settings must not bypass --publish or the
    # per-claim destination policy, even when this script is run directly.
    export XC_PUBLISH_TARGET=none XC_PUBLISH_EXECUTE=false
    ;;
  plan|execute)
    if [[ "$PUBLICATION" == "execute" && -z "${XC_PUBLISH_AUTHOR_EMAIL:-}" ]]; then
      echo "--publish requires XC_PUBLISH_AUTHOR_EMAIL to be set" >&2
      exit 1
    fi
    export XC_RUN_PROFILE=author XC_PUBLISH_TARGET="$CLAIM_PUBLISH_TARGET"
    if [[ "$PUBLICATION" == "execute" ]]; then
      export XC_PUBLISH_EXECUTE=true
    else
      export XC_PUBLISH_EXECUTE=false
    fi
    echo "  publication: $CLAIM_PUBLISH_TARGET ($PUBLICATION)"
    ;;
  *)
    echo "PUBLICATION must be none, plan, or execute" >&2
    exit 1
    ;;
esac

if [[ -n "$BENCHMARK_BASELINE" && -z "$BENCHMARK_REPORT" ]]; then
  echo "BENCHMARK_BASELINE requires BENCHMARK_REPORT" >&2
  exit 1
fi
if [[ -n "$BENCHMARK_LABEL" && -z "$BENCHMARK_REPORT" ]]; then
  echo "BENCHMARK_LABEL requires BENCHMARK_REPORT" >&2
  exit 1
fi
if [[ -n "$BENCHMARK_COMPARISON_MODE" && -z "$BENCHMARK_BASELINE" ]]; then
  echo "BENCHMARK_COMPARISON_MODE requires BENCHMARK_BASELINE" >&2
  exit 1
fi
case "$BENCHMARK_COMPARISON_MODE" in
  ""|same-policy|gl-root-policy-delta)
    ;;
  *)
    echo "BENCHMARK_COMPARISON_MODE must be same-policy or gl-root-policy-delta" >&2
    exit 1
    ;;
esac

case "$GL_ROOT_PARALLEL" in
  1|[Tt][Rr][Uu][Ee]|[Yy][Ee][Ss]|[Oo][Nn])
    GL_ROOT_PARALLEL_ENABLED=true
    ;;
  0|[Ff][Aa][Ll][Ss][Ee]|[Nn][Oo]|[Oo][Ff][Ff]|"")
    GL_ROOT_PARALLEL_ENABLED=false
    ;;
  *)
    echo "XC_GL_ROOT_PARALLEL must be true/false, yes/no, on/off, or 1/0" >&2
    exit 1
    ;;
esac

BENCHMARK_ARGS=()
if [[ -n "$BENCHMARK_REPORT" ]]; then
  BENCHMARK_ARGS+=(--benchmark-report "$BENCHMARK_REPORT")
fi
if [[ -n "$BENCHMARK_BASELINE" ]]; then
  BENCHMARK_ARGS+=(--benchmark-baseline "$BENCHMARK_BASELINE")
fi
if [[ -n "$BENCHMARK_LABEL" ]]; then
  BENCHMARK_ARGS+=(--benchmark-label "$BENCHMARK_LABEL")
fi
if [[ -n "$BENCHMARK_COMPARISON_MODE" ]]; then
  BENCHMARK_ARGS+=(--benchmark-comparison-mode "$BENCHMARK_COMPARISON_MODE")
fi

RUNTIME_ARGS=()
if [[ "$GL_ROOT_PARALLEL_ENABLED" == "true" ]]; then
  RUNTIME_ARGS+=(--parallel-gl-roots)
fi

# Validate the complete invocation before starting a potentially expensive build.
if [[ -z "${BIN+x}" ]]; then
  CLAIM_TARGET_DIR="$CLAIM_REPO_ROOT/target"
  BIN="$CLAIM_TARGET_DIR/release/ccm-reproduction"
  # Cargo's incremental freshness check is quick and prevents an executable
  # built from an older toolkit lockfile, without HP, or in an externally
  # overridden CARGO_TARGET_DIR from being mistaken for the current binary.
  CLAIM_FEATURES=hp
  if [[ "$GL_ROOT_PARALLEL_ENABLED" == "true" ]]; then
    CLAIM_FEATURES=$CLAIM_FEATURES,experimental-gl-root-parallel
  fi
  cargo build --quiet --release --features "$CLAIM_FEATURES" --locked --bin ccm-reproduction \
    --target-dir "$CLAIM_TARGET_DIR"
elif [[ ! -x "$BIN" ]]; then
  echo "Configured reproduction binary is not executable: $BIN" >&2
  exit 1
fi

# A failed configuration must not skip the claim's other independent
# configurations. Each binary exit status is retained, and the script itself
# fails after attempting every configuration.
CLAIM_FAILURES=0
CLAIM_LOGS=()
CLAIM_SCRIPT_NAME=$(basename "${BASH_SOURCE[1]}" .sh)
claim_exit() {
  local status=$?
  trap - EXIT
  if ((${#CLAIM_LOGS[@]} > 0)); then
    local summary="${CLAIM_LOGS[0]%.log}.summary.json"
    if python3 "$CLAIM_REPO_ROOT/scripts/claim_summary.py" --claim "$CLAIM_SCRIPT_NAME" --output "$summary" "${CLAIM_LOGS[@]}"; then :
    else status=1
    fi
  fi
  if ((CLAIM_FAILURES > 0)); then
    if [[ "$PREFLIGHT_ONLY" == "true" ]]; then
      echo "Source admission failed for $CLAIM_FAILURES invocation(s); no claim computation was started." >&2
    else
      echo "Claim script finished with $CLAIM_FAILURES failed invocation(s). See claim logs." >&2
    fi
    exit 1
  fi
  exit "$status"
}
trap claim_exit EXIT

# run_claim run|check-evenness ARGS...
#   run:            reference-seeded roots and the smallest Weil eigenvalue
#                   (claim-level capture; no certification or research).
#   check-evenness: natural evenness plus the low parity-sector separations
#                   and the even-state spectral isolation (gap-level capture).
run_claim() {
  if [[ -z "${XC_RESEARCH_WORKING_BYTES:-}" ]]; then
    echo "The claim script did not declare its working budget (claim_working_budget)" >&2
    exit 2
  fi
  local -a command=("$BIN" "${BENCHMARK_ARGS[@]}" "${RUNTIME_ARGS[@]}" "${JOURNAL_ARGS[@]}" "$@")
  case "${1:-}" in
    run)
      command+=(--research-capture claim --root-acquisition seeded --root-validation off)
      ;;
    check-evenness)
      command+=(--research-capture gap --root-acquisition seeded)
      ;;
    *)
      echo "run_claim requires run or check-evenness" >&2
      CLAIM_FAILURES=$((CLAIM_FAILURES + 1))
      return 0
      ;;
  esac
  if [[ "$PREFLIGHT_ONLY" == "true" ]]; then
    # The exact claim arguments, but never journals, capture sessions or
    # publication in admission mode.
    if "${command[@]}" --preflight-only; then :
    else CLAIM_FAILURES=$((CLAIM_FAILURES + 1))
    fi
    return 0
  fi
  local log_root=${CLAIM_LOG_ROOT:-$CLAIM_REPO_ROOT/.xcelerator-cache/claim-logs}
  mkdir -p "$log_root"
  local log
  log=$(mktemp "$log_root/claim-$(date -u +%Y%m%dT%H%M%S)-XXXXXX.log")
  CLAIM_LOGS+=("$log")
  echo "Claim log: $log"
  local -a statuses
  if XC_PERF_REPORT="${XC_PERF_REPORT:-${log%.log}.performance.json}" "${command[@]}" 2>&1 | tee "$log"; then
    statuses=("${PIPESTATUS[@]}")
  else
    statuses=("${PIPESTATUS[@]}")
  fi
  printf 'binary_exit=%s\nlog_exit=%s\n' "${statuses[0]}" "${statuses[1]}" > "${log%.log}.status"
  if ((statuses[0] != 0 || statuses[1] != 0)); then
    CLAIM_FAILURES=$((CLAIM_FAILURES + 1))
    echo "Claim invocation failed; preserved output and continuing independent configurations: $log" >&2
  fi
  return 0
}
