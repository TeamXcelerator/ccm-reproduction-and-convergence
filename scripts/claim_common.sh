#!/usr/bin/env bash
# Shared zero-configuration launcher for independently runnable claim scripts.

CLAIM_REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$CLAIM_REPO_ROOT"

# Claim scripts accept capture controls directly. Environment variables remain
# available for unattended server jobs, but command-line flags take precedence.
RESEARCH_CAPTURE_LEVEL=${RESEARCH_CAPTURE_LEVEL:-ultra}
RESEARCH_SECTOR_EIGENPAIRS=${RESEARCH_SECTOR_EIGENPAIRS:-8}
ROOT_ACQUISITION_MODE=${ROOT_ACQUISITION_MODE:-seeded}
PARITY_POLICY=${PARITY_POLICY:-even-sector}
ROOT_VALIDATION_LEVEL=${ROOT_VALIDATION_LEVEL:-off}
ROOT_ENCLOSURE_DIGITS=${ROOT_ENCLOSURE_DIGITS:-}
INCLUDE_NEGATIVE_ROOTS=${INCLUDE_NEGATIVE_ROOTS:-false}
ALLOW_ROOT_OVERSUBSCRIPTION=${ALLOW_ROOT_OVERSUBSCRIPTION:-false}
BENCHMARK_REPORT=${BENCHMARK_REPORT:-}
BENCHMARK_BASELINE=${BENCHMARK_BASELINE:-}
BENCHMARK_LABEL=${BENCHMARK_LABEL:-}
BENCHMARK_COMPARISON_MODE=${BENCHMARK_COMPARISON_MODE:-}
CAPTURE_DISTANCE=${CAPTURE_DISTANCE:-false}
DISTANCE_RESOLUTION=${DISTANCE_RESOLUTION:-4000}
DISTANCE_PROFILE_STEPS=${DISTANCE_PROFILE_STEPS:-1000}
GL_ROOT_PARALLEL=${XC_GL_ROOT_PARALLEL:-false}
JOURNAL_ARGS=()
EXPLICIT_CAPTURE_ARGS=()
while (($# > 0)); do
  case "$1" in
    --research-capture)
      if (($# < 2)); then
        echo "--research-capture requires claim, research, gap, maximum, or ultra" >&2
        exit 2
      fi
      RESEARCH_CAPTURE_LEVEL=$2
      shift 2
      ;;
    --capture-output|--capture-prefix-checkpoints|--capture-working-precision-bits|--capture-reduction-max-dimension)
      if (($# < 2)); then echo "$1 requires a value" >&2; exit 2; fi
      JOURNAL_ARGS+=("$1" "$2")
      shift 2
      ;;
    --require-complete-capture)
      JOURNAL_ARGS+=("$1")
      shift
      ;;
    --capture-deviation-decomposition|--capture-prime-power-response|--capture-u-flow-response|--capture-sector-gap-certificate)
      EXPLICIT_CAPTURE_ARGS+=("$1")
      shift
      ;;
    --root-validation)
      if (($# < 2)); then
        echo "--root-validation requires off or certified" >&2
        exit 2
      fi
      ROOT_VALIDATION_LEVEL=$2
      shift 2
      ;;
    --root-acquisition)
      if (($# < 2)); then
        echo "--root-acquisition requires seeded or independent" >&2
        exit 2
      fi
      ROOT_ACQUISITION_MODE=$2
      shift 2
      ;;
    --parity-policy)
      if (($# < 2)); then
        echo "--parity-policy requires natural, adaptive-even, or even-sector" >&2
        exit 2
      fi
      PARITY_POLICY=$2
      shift 2
      ;;
    --root-enclosure-digits)
      if (($# < 2)); then
        echo "--root-enclosure-digits requires a positive integer" >&2
        exit 2
      fi
      ROOT_ENCLOSURE_DIGITS=$2
      shift 2
      ;;
    --research-sector-eigenpairs)
      if (($# < 2)); then
        echo "--research-sector-eigenpairs requires a positive integer" >&2
        exit 2
      fi
      RESEARCH_SECTOR_EIGENPAIRS=$2
      shift 2
      ;;
    --capture-distance)
      CAPTURE_DISTANCE=true
      shift
      ;;
    --distance-resolution)
      if (($# < 2)); then
        echo "--distance-resolution requires a positive integer" >&2
        exit 2
      fi
      DISTANCE_RESOLUTION=$2
      shift 2
      ;;
    --distance-profile-steps)
      if (($# < 2)); then
        echo "--distance-profile-steps requires a positive integer" >&2
        exit 2
      fi
      DISTANCE_PROFILE_STEPS=$2
      shift 2
      ;;
    --include-negative-roots)
      INCLUDE_NEGATIVE_ROOTS=true
      shift
      ;;
    --allow-root-oversubscription)
      ALLOW_ROOT_OVERSUBSCRIPTION=true
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
    --benchmark-report)
      if (($# < 2)); then
        echo "--benchmark-report requires a path" >&2
        exit 2
      fi
      BENCHMARK_REPORT=$2
      shift 2
      ;;
    --benchmark-baseline)
      if (($# < 2)); then
        echo "--benchmark-baseline requires a path" >&2
        exit 2
      fi
      BENCHMARK_BASELINE=$2
      shift 2
      ;;
    --benchmark-label)
      if (($# < 2)); then
        echo "--benchmark-label requires a label" >&2
        exit 2
      fi
      BENCHMARK_LABEL=$2
      shift 2
      ;;
    --benchmark-comparison-mode)
      if (($# < 2)); then
        echo "--benchmark-comparison-mode requires same-policy or gl-root-policy-delta" >&2
        exit 2
      fi
      BENCHMARK_COMPARISON_MODE=$2
      shift 2
      ;;
    --help|-h)
      echo "Usage: bash ${BASH_SOURCE[1]} [--research-capture claim|research|gap|maximum|ultra] [--research-sector-eigenpairs COUNT] [--root-acquisition MODE] [--parity-policy POLICY] [--root-validation LEVEL] [--root-enclosure-digits DIGITS] [--include-negative-roots] [--allow-root-oversubscription] [--capture-distance] [--distance-resolution COUNT] [--distance-profile-steps COUNT] [--verify-cache] [--parallel-gl-roots] [--benchmark-report PATH] [--benchmark-baseline PATH] [--benchmark-label LABEL] [--benchmark-comparison-mode MODE]"
      echo "  LEVEL: claim, research, gap, maximum, or ultra (default)"
      echo "  ROOT ACQUISITION: seeded (default for every claim script) or independent"
      echo "  PARITY POLICY: even-sector (default), natural, or adaptive-even"
      echo "  ROOT VALIDATION: off (default) or certified"
      echo "  ROOT ENCLOSURE: defaults to the claim's display digits; override only when needed"
      echo "  ADVANCED ROOTS: signed and finite-shortfall controls require independent HP discovery"
      echo "  TARGET DISTANCE: --capture-distance retains the eigenfunction profile and its weighted"
      echo "                   distance to tau(u); tune with --distance-resolution and"
      echo "                   --distance-profile-steps. Included by maximum and ultra."
      echo "  JOURNALS: --capture-output DIR, --capture-prefix-checkpoints k,...,"
      echo "            --capture-working-precision-bits BITS, --capture-reduction-max-dimension N"
      echo "  COMPLETENESS: --require-complete-capture fails after preserving partial evidence"
      echo "  EXPLICIT: --capture-deviation-decomposition, --capture-prime-power-response,"
      echo "            --capture-u-flow-response, --capture-sector-gap-certificate (run commands)"
      echo "  CACHE VALIDATION: --verify-cache recomputes and compares claim artifacts; disabled by default"
      echo "  EXPERIMENTAL GL ROOTS: --parallel-gl-roots or XC_GL_ROOT_PARALLEL=true; native Linux only, never WSL"
      echo "  BENCHMARK: one process/report; benchmark sweep configurations separately with distinct paths"
      echo "  BENCHMARK POLICY DELTA: gl-root-policy-delta permits only the intentional serial/parallel GL-root difference"
      exit 0
      ;;
    *)
      echo "Unknown claim-script argument: $1" >&2
      echo "Use --help for supported capture controls." >&2
      exit 2
      ;;
  esac
done

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

if [[ -z "${BIN+x}" ]]; then
  CLAIM_TARGET_DIR="$CLAIM_REPO_ROOT/target"
  BIN="$CLAIM_TARGET_DIR/release/ccm-reproduction"
  # Cargo's incremental freshness check is quick and prevents an executable
  # built from an older toolkit lockfile, without HP, or in an externally
  # overridden CARGO_TARGET_DIR from being mistaken for the current binary.
  CLAIM_FEATURES=hp
  if [[ "$ROOT_VALIDATION_LEVEL" == "certified" || " ${EXPLICIT_CAPTURE_ARGS[*]} " == *" --capture-sector-gap-certificate "* ]]; then
    CLAIM_FEATURES=$CLAIM_FEATURES,root-certification
  fi
  if [[ "$GL_ROOT_PARALLEL_ENABLED" == "true" ]]; then
    CLAIM_FEATURES=$CLAIM_FEATURES,experimental-gl-root-parallel
  fi
  cargo build --quiet --release --features "$CLAIM_FEATURES" --locked --bin ccm-reproduction \
    --target-dir "$CLAIM_TARGET_DIR"
elif [[ ! -x "$BIN" ]]; then
  echo "Configured reproduction binary is not executable: $BIN" >&2
  exit 1
fi

# Ultra is the default research recipe. Lower levels are explicit cost controls.
# Every level preserves the selected root acquisition, parity, and arithmetic.
case "$RESEARCH_CAPTURE_LEVEL" in
  claim|research|gap)
    RESEARCH_CAPTURE_ARGS=(--research-capture "$RESEARCH_CAPTURE_LEVEL")
    ;;
  maximum|ultra)
    # The shared recipe adds complete distance and retained-prefix diagnostics;
    # exact interval certification remains an explicit request.
    RESEARCH_CAPTURE_ARGS=(
      --research-capture "$RESEARCH_CAPTURE_LEVEL"
      --research-sector-eigenpairs "$RESEARCH_SECTOR_EIGENPAIRS"
    )
    ;;
  *)
    echo "RESEARCH_CAPTURE_LEVEL must be claim, research, gap, maximum, or ultra" >&2
    exit 1
    ;;
esac

case "$ROOT_VALIDATION_LEVEL" in
  off|certified)
    ROOT_VALIDATION_ARGS=(
      --root-validation "$ROOT_VALIDATION_LEVEL"
    )
    if [[ -n "$ROOT_ENCLOSURE_DIGITS" ]]; then
      ROOT_VALIDATION_ARGS+=(--root-enclosure-digits "$ROOT_ENCLOSURE_DIGITS")
    fi
    ;;
  *)
    echo "ROOT_VALIDATION_LEVEL must be off or certified" >&2
    exit 1
    ;;
esac

case "$ROOT_ACQUISITION_MODE" in
  seeded|independent)
    ROOT_ACQUISITION_ARGS=(--root-acquisition "$ROOT_ACQUISITION_MODE")
    ;;
  *)
    echo "ROOT_ACQUISITION_MODE must be seeded or independent" >&2
    exit 1
    ;;
esac

case "$PARITY_POLICY" in
  natural|adaptive-even|even-sector)
    PARITY_POLICY_ARGS=(--parity-policy "$PARITY_POLICY")
    ;;
  *)
    echo "PARITY_POLICY must be natural, adaptive-even, or even-sector" >&2
    exit 1
    ;;
esac

ADVANCED_ROOT_ARGS=()
if [[ "$INCLUDE_NEGATIVE_ROOTS" == "true" ]]; then
  ADVANCED_ROOT_ARGS+=(--include-negative-roots)
elif [[ "$INCLUDE_NEGATIVE_ROOTS" != "false" ]]; then
  echo "INCLUDE_NEGATIVE_ROOTS must be true or false" >&2
  exit 1
fi
if [[ "$ALLOW_ROOT_OVERSUBSCRIPTION" == "true" ]]; then
  ADVANCED_ROOT_ARGS+=(--allow-root-oversubscription)
elif [[ "$ALLOW_ROOT_OVERSUBSCRIPTION" != "false" ]]; then
  echo "ALLOW_ROOT_OVERSUBSCRIPTION must be true or false" >&2
  exit 1
fi
if ((${#ADVANCED_ROOT_ARGS[@]} > 0)) && [[ "$ROOT_ACQUISITION_MODE" != "independent" ]]; then
  echo "Advanced root controls require ROOT_ACQUISITION_MODE=independent" >&2
  exit 1
fi

# Maximum and Ultra request the complete default distance convention.
# The same grid controls apply to root and evenness claim commands.
JOURNAL_ARGS+=(--capture-grid-resolution "$DISTANCE_RESOLUTION" --capture-profile-steps "$DISTANCE_PROFILE_STEPS")
DISTANCE_ARGS=()
if [[ "$CAPTURE_DISTANCE" == "true" ]]; then
  DISTANCE_ARGS+=(
    --capture-distance
    --distance-resolution "$DISTANCE_RESOLUTION"
    --distance-profile-steps "$DISTANCE_PROFILE_STEPS"
  )
elif [[ "$CAPTURE_DISTANCE" != "false" ]]; then
  echo "CAPTURE_DISTANCE must be true or false" >&2
  exit 1
fi

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

# A primary failure must not skip other independent cases. Each binary exit
# status is retained, and the script itself fails after attempting its cases.
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
    echo "Claim script finished with $CLAIM_FAILURES failed invocation(s). See claim logs." >&2
    exit 1
  fi
  exit "$status"
}
trap claim_exit EXIT

run_research_claim() {
  local -a command=("$BIN" "${BENCHMARK_ARGS[@]}" "${RUNTIME_ARGS[@]}" "${JOURNAL_ARGS[@]}" "$@" "${RESEARCH_CAPTURE_ARGS[@]}")
  if [[ "${1:-}" == "run" ]]; then
    command+=("${ROOT_ACQUISITION_ARGS[@]}" "${PARITY_POLICY_ARGS[@]}" "${ROOT_VALIDATION_ARGS[@]}" "${ADVANCED_ROOT_ARGS[@]}" "${DISTANCE_ARGS[@]}" "${EXPLICIT_CAPTURE_ARGS[@]}")
  elif [[ "${1:-}" == "check-evenness" ]]; then
    command+=("${ROOT_ACQUISITION_ARGS[@]}")
    if ((${#EXPLICIT_CAPTURE_ARGS[@]} > 0)); then
      echo "Explicit diagnostic flags require a run claim; use Ultra for evenness capture." >&2
      CLAIM_FAILURES=$((CLAIM_FAILURES + 1))
      return 0
    fi
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
    echo "Claim invocation failed; preserved output and continuing independent cases: $log" >&2
  fi
  return 0
}
