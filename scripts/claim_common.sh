#!/usr/bin/env bash
# Shared launcher for the paper's individually runnable claim scripts.
#
# By default each claim computes only the measurements its paper checks use:
# the reference-seeded roots and smallest Weil eigenvalue for run claims,
# and the natural evenness with the low parity-sector separations and the
# even-state spectral isolation for evenness claims. scripts/claim_summary.py
# then reports PASS or FAIL for the claim's documented checks. Explicit Ultra
# capture also retains single-run research artifacts for subsequent analysis.

CLAIM_REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$CLAIM_REPO_ROOT"

# claim_working_budget GIB declares the Toolkit's primary admission budget for
# this claim. The Toolkit reads it as XC_RESEARCH_WORKING_BYTES for Tau and
# source assembly and for the ground-index proof, despite the variable's name.
# Each script's value covers the conservative admission estimate of every one
# of its configurations; run the claim on a machine with at least that much
# memory available. XC_CLAIM_MEMORY_GIB explicitly overrides that default; it
# changes resource admission only, never the claim's numerical configuration.
claim_working_budget() {
  if [[ ! "${1:-}" =~ ^[1-9][0-9]{0,4}$ ]]; then
    echo "claim_working_budget requires a whole number of GiB" >&2
    exit 2
  fi
  local effective_gib=${XC_CLAIM_MEMORY_GIB-${1:-}}
  # Stay inside positive signed 64-bit shell arithmetic, and therefore u64.
  # Validate decimal syntax before evaluating anything as shell arithmetic.
  if [[ ! "$effective_gib" =~ ^[1-9][0-9]{0,9}$ ]] || ((effective_gib > 8589934591)); then
    echo "XC_CLAIM_MEMORY_GIB must be an integer from 1 through 8589934591 GiB" >&2
    exit 2
  fi
  export XC_RESEARCH_WORKING_BYTES=$(( effective_gib << 30 ))
  printf 'Claim working budget: %s GiB (%s bytes)\n' "$effective_gib" "$XC_RESEARCH_WORKING_BYTES" >&2
}

PUBLICATION=${PUBLICATION:-none}
PUBLICATION_PER_CONFIGURATION=false
BENCHMARK_REPORT=${BENCHMARK_REPORT:-}
BENCHMARK_BASELINE=${BENCHMARK_BASELINE:-}
BENCHMARK_LABEL=${BENCHMARK_LABEL:-}
BENCHMARK_COMPARISON_MODE=${BENCHMARK_COMPARISON_MODE:-}
GL_ROOT_PARALLEL=${XC_GL_ROOT_PARALLEL:-false}
CLAIM_RESEARCH_CAPTURE=default
CLAIM_SECTOR_EIGENPAIRS=8
CLAIM_SECTOR_EIGENPAIRS_SET=false
CLAIM_ROOT_VALIDATION=off
CLAIM_ROOT_VALIDATION_SET=false
CLAIM_SECTOR_CERTIFICATE=false
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
    --research-capture)
      if (($# < 2)) || [[ "$2" != ultra ]]; then
        echo "--research-capture requires ultra; omit it for the claim's ordinary measurements" >&2
        exit 2
      fi
      CLAIM_RESEARCH_CAPTURE=ultra
      shift 2
      ;;
    --research-sector-eigenpairs)
      if (($# < 2)) || [[ ! "$2" =~ ^[1-9][0-9]*$ ]]; then
        echo "--research-sector-eigenpairs requires a positive integer" >&2
        exit 2
      fi
      CLAIM_SECTOR_EIGENPAIRS=$2
      CLAIM_SECTOR_EIGENPAIRS_SET=true
      shift 2
      ;;
    --root-validation)
      if (($# < 2)) || [[ "$2" != off && "$2" != certified ]]; then
        echo "--root-validation requires off or certified" >&2
        exit 2
      fi
      CLAIM_ROOT_VALIDATION=$2
      CLAIM_ROOT_VALIDATION_SET=true
      shift 2
      ;;
    --capture-sector-gap-certificate)
      CLAIM_SECTOR_CERTIFICATE=true
      shift
      ;;
    --publish)
      PUBLICATION=execute
      shift
      ;;
    --publication-per-configuration)
      PUBLICATION_PER_CONFIGURATION=true
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
      echo "Usage: bash ${BASH_SOURCE[1]} [--publish | --publish-plan] [--capture-output DIR] [--research-capture ultra [--research-sector-eigenpairs COUNT] [--root-validation off|certified] [--capture-sector-gap-certificate]] [--verify-cache] [--parallel-gl-roots] [--preflight-only] [--benchmark-report PATH [--benchmark-baseline PATH] [--benchmark-label LABEL] [--benchmark-comparison-mode MODE]]"
      echo "  Each claim runs its fixed configurations; scripts/claim_summary.py reports"
      echo "  PASS or FAIL for its numerical checks, independently of research completeness."
      echo "  ULTRA: explicitly retain single-run research artifacts and honor configured"
      echo "         XC_RESEARCH_* and XC_TARGET_* inputs. Cohort analysis runs separately."
      echo "         Root and sector certificate switches apply to run commands only."
      echo "  PUBLICATION: off by default. --publish publishes this claim's artifacts (author"
      echo "               credentials and XC_PUBLISH_AUTHOR_EMAIL required); --publish-plan stages"
      echo "               and plans the same publication without pushing. Claim 1a publishes to"
      echo "               public and private; every other claim publishes to private only."
      echo "               --publish stages every configuration, then finalizes once before the"
      echo "               claim summary. --publication-per-configuration retains the old policy."
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

if [[ ${XC_TRANSFORM_MAX_GUARD_BITS+x} ]]; then
  # Bound transform attempts without changing the requested precision. Admit
  # canonical values before invoking a child or evaluating shell arithmetic.
  case "$XC_TRANSFORM_MAX_GUARD_BITS" in
    4096|8192|16384|32768|65536|131072|262144|524288|1048576) ;;
    *) echo 'XC_TRANSFORM_MAX_GUARD_BITS must be a canonical decimal power of two from 4096 through 1048576' >&2; exit 2 ;;
  esac
fi

if [[ ${XC_RESEARCH_PREP_WORKERS+x} ]]; then
  # This controls simultaneous preparation tasks only, not research inputs or
  # numerical identities. Validate before any shell arithmetic or child work.
  CLAIM_PREPARATION_WORKERS=$XC_RESEARCH_PREP_WORKERS
  if [[ ! "$CLAIM_PREPARATION_WORKERS" =~ ^[0-9]+$ ]]; then
    echo 'XC_RESEARCH_PREP_WORKERS must be a positive decimal integer no larger than 18446744073709551615' >&2
    exit 2
  fi
  CLAIM_PREPARATION_WORKERS=${CLAIM_PREPARATION_WORKERS#"${CLAIM_PREPARATION_WORKERS%%[!0]*}"}
  if [[ -z "$CLAIM_PREPARATION_WORKERS" || ${#CLAIM_PREPARATION_WORKERS} -gt 20 || ( ${#CLAIM_PREPARATION_WORKERS} -eq 20 && "$CLAIM_PREPARATION_WORKERS" > 18446744073709551615 ) ]]; then
    echo 'XC_RESEARCH_PREP_WORKERS must be a positive decimal integer no larger than 18446744073709551615' >&2
    exit 2
  fi
  export XC_RESEARCH_PREP_WORKERS=$CLAIM_PREPARATION_WORKERS
  unset CLAIM_PREPARATION_WORKERS
fi

if [[ "$CLAIM_RESEARCH_CAPTURE" != ultra ]]; then
  if [[ "$CLAIM_SECTOR_EIGENPAIRS_SET" == true || "$CLAIM_ROOT_VALIDATION_SET" == true || "$CLAIM_SECTOR_CERTIFICATE" == true ]]; then
    echo "Research and certificate controls require --research-capture ultra" >&2
    exit 2
  fi
  # Ordinary claims remain self-contained even with inherited research inputs;
  # the validated preparation worker cap is an independent execution control.
  # The explicit Ultra route honors its source-bound target/preparer inputs.
  for CLAIM_INHERITED_NAME in $(compgen -e); do
    case "$CLAIM_INHERITED_NAME" in
      XC_RESEARCH_PREP_WORKERS) ;; # Validated operational concurrency only.
      XC_RESEARCH_*|XC_TARGET_*) unset "$CLAIM_INHERITED_NAME" ;;
    esac
  done
  unset CLAIM_INHERITED_NAME
else
  case "$(basename "${BASH_SOURCE[1]}" .sh)" in
    claim1c_lambda1000)
      if [[ "$CLAIM_SECTOR_CERTIFICATE" == true ]]; then
        echo "The claim1c-hp1000 applicability policy excludes the sector-gap certificate" >&2
        exit 2
      fi
      ;;
    claim4*)
      if [[ "$CLAIM_ROOT_VALIDATION_SET" == true || "$CLAIM_SECTOR_CERTIFICATE" == true ]]; then
        echo "Root validation and the sector-gap certificate switches are not supported by check-evenness" >&2
        exit 2
      fi
      ;;
  esac
fi

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
  if [[ "$CLAIM_RESEARCH_CAPTURE" == ultra ]]; then
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

# A failed configuration must not skip the claim's other independent
# configurations. Each binary exit status is retained, and the script itself
# fails after attempting every configuration.
CLAIM_FAILURES=0
CLAIM_LOGS=()
CLAIM_SCRIPT_NAME=$(basename "${BASH_SOURCE[1]}" .sh)
CLAIM_DEFERRED_PUBLICATION=false
CLAIM_PUBLICATION_LOCK=
CLAIM_PUBLICATION_STATUS_FILE=
claim_publication_state() {
  python3 - "$CLAIM_PUBLICATION_STATUS_FILE" "$1" "$XC_PUBLISH_STAGING_ROOT" "$CLAIM_PUBLISH_TARGET" "${#CLAIM_LOGS[@]}" <<'PUBLICATION_STATE'
import json, os, sys, tempfile
from pathlib import Path
path=Path(sys.argv[1])
value=dict(schema_version=1,status=sys.argv[2],staging_root=sys.argv[3],target=sys.argv[4],
           invocations=int(sys.argv[5]),semantics='operational publication status; numerical verdict is separate')
with tempfile.NamedTemporaryFile(mode='w',dir=path.parent,prefix=path.name+'.',delete=False) as output:
    temporary=Path(output.name)
    json.dump(value,output,indent=2); output.write('\n'); output.flush(); os.fsync(output.fileno())
os.replace(temporary,path)
PUBLICATION_STATE
}
claim_prepare_publication() {
  if [[ "$PUBLICATION" != execute || "$PUBLICATION_PER_CONFIGURATION" == true ]]; then return 0; fi
  local log_root=${CLAIM_LOG_ROOT:-$CLAIM_REPO_ROOT/.xcelerator-cache/claim-logs}
  mkdir -p "$log_root"
  if [[ -z "${XC_PUBLISH_STAGING_ROOT:-}" ]]; then
    XC_PUBLISH_STAGING_ROOT=$(mktemp -d "$log_root/claim-publication-staging-XXXXXX")
    export XC_PUBLISH_STAGING_ROOT
  fi
  if [[ -L "$XC_PUBLISH_STAGING_ROOT" ]]; then
    echo "Deferred publication refuses a linked staging root" >&2; return 2
  fi
  mkdir -p "$XC_PUBLISH_STAGING_ROOT"
  local lock="$XC_PUBLISH_STAGING_ROOT/.claim-publication-active"
  if ! mkdir "$lock"; then
    echo "Another claim or an interrupted claim owns publication staging: $XC_PUBLISH_STAGING_ROOT" >&2
    return 2
  fi
  CLAIM_PUBLICATION_LOCK=$lock
  # Launcher identity/head files and empty directory scaffolding are allowed.
  # Never absorb another run's drafts or remove any retained data.
  if [[ -L "$XC_PUBLISH_STAGING_ROOT/drafts" ]] ||
     [[ -e "$XC_PUBLISH_STAGING_ROOT/drafts" && ! -d "$XC_PUBLISH_STAGING_ROOT/drafts" ]] ||
     [[ -d "$XC_PUBLISH_STAGING_ROOT/drafts" && -n "$(find "$XC_PUBLISH_STAGING_ROOT/drafts" -mindepth 1 ! -type d -print -quit)" ]]; then
    echo "Deferred publication requires an empty draft queue; use publish-staged explicitly for retained drafts" >&2
    return 2
  fi
  CLAIM_PUBLICATION_STATUS_FILE=$(mktemp "$log_root/claim-publication-XXXXXX.status.json")
  CLAIM_DEFERRED_PUBLICATION=true
  claim_publication_state pending
  echo "Deferred publication staging: $XC_PUBLISH_STAGING_ROOT"
  echo "Publication status: $CLAIM_PUBLICATION_STATUS_FILE"
}
claim_exit() {
  local status=$? publication_complete=true
  trap - EXIT
  if [[ "$CLAIM_DEFERRED_PUBLICATION" == true ]]; then
    publication_complete=false
    if ((status == 0 && ${#CLAIM_LOGS[@]} > 0)); then
      local publication_log="${CLAIM_PUBLICATION_STATUS_FILE%.status.json}.log"
      local -a publication_statuses
      if XC_PUBLISH_EXECUTE=true "$BIN" publish-staged --staging-root "$XC_PUBLISH_STAGING_ROOT" 2>&1 | tee "$publication_log"; then
        publication_statuses=("${PIPESTATUS[@]}")
      else
        publication_statuses=("${PIPESTATUS[@]}")
      fi
      if ((publication_statuses[0] == 0 && publication_statuses[1] == 0)); then
        if claim_publication_state completed; then publication_complete=true
        else status=1
        fi
      else
        status=1
        if claim_publication_state failed; then :; else status=1; fi
      fi
    else
      if claim_publication_state pending; then :; else
        if ((status == 0)); then status=1; fi
      fi
    fi
    if [[ "$publication_complete" != true ]]; then
      echo "Publication remains pending or failed; retained staging: $XC_PUBLISH_STAGING_ROOT" >&2
      echo "Retry explicitly with the same author/destination environment: $BIN publish-staged --staging-root $XC_PUBLISH_STAGING_ROOT" >&2
    fi
  fi
  if ((${#CLAIM_LOGS[@]} > 0)); then
    local summary="${CLAIM_LOGS[0]%.log}.summary.json"
    if [[ "$publication_complete" == true ]]; then
      if python3 "$CLAIM_REPO_ROOT/scripts/claim_summary.py" --claim "$CLAIM_SCRIPT_NAME" --output "$summary" "${CLAIM_LOGS[@]}"; then :
      elif ((status == 0)); then status=1
      fi
    else
      # Preserve numerical assessment without printing a premature claim PASS.
      if python3 "$CLAIM_REPO_ROOT/scripts/claim_summary.py" --claim "$CLAIM_SCRIPT_NAME" --output "$summary" "${CLAIM_LOGS[@]}" > "${summary%.json}.pending-publication.log" 2>&1; then :
      elif ((status == 0)); then status=1
      fi
    fi
  fi
  if [[ -n "$CLAIM_PUBLICATION_LOCK" ]]; then
    if rmdir "$CLAIM_PUBLICATION_LOCK"; then :; else
      echo "Retained publication lock could not be removed: $CLAIM_PUBLICATION_LOCK" >&2
      if ((status == 0)); then status=1; fi
    fi
  fi
  if ((CLAIM_FAILURES > 0)); then
    if [[ "$PREFLIGHT_ONLY" == "true" ]]; then
      echo "Source admission failed for $CLAIM_FAILURES invocation(s); no claim computation was started." >&2
    else
      echo "Claim script finished with $CLAIM_FAILURES failed invocation(s). See claim logs." >&2
    fi
    if ((status == 0)); then status=1; fi
  fi
  exit "$status"
}
trap claim_exit EXIT
trap 'exit 143' TERM
trap 'exit 130' INT
claim_prepare_publication

# run_claim run|check-evenness ARGS...
#   run:            reference-seeded roots and the smallest Weil eigenvalue
#                   (claim-level capture by default; explicit Ultra optional).
#   check-evenness: natural evenness plus the low parity-sector separations
#                   and the even-state spectral isolation (gap by default).
run_claim() {
  if [[ -z "${XC_RESEARCH_WORKING_BYTES:-}" ]]; then
    echo "The claim script did not declare its working budget (claim_working_budget)" >&2
    exit 2
  fi
  local -a command=("$BIN" "${BENCHMARK_ARGS[@]}" "${RUNTIME_ARGS[@]}" "${JOURNAL_ARGS[@]}" "$@")
  local invocation_publish_execute=${XC_PUBLISH_EXECUTE:-false}
  if [[ "$CLAIM_DEFERRED_PUBLICATION" == true ]]; then
    command+=(--defer-publication)
    # Older sweep scripts toggle execution on their last configuration. Every
    # deferred process stages under the same explicit policy; none finalizes.
    invocation_publish_execute=true
  fi
  local cutoff= digits= parity=even-sector previous= argument capture_policy=full
  if [[ "$CLAIM_RESEARCH_CAPTURE" == ultra ]]; then
    for argument in "$@"; do
      case "$previous" in
        --lambda-sq) cutoff=$argument ;;
        --precision-digits) digits=$argument ;;
        --parity-policy) parity=$argument ;;
      esac
      previous=$argument
    done
    case "$CLAIM_SCRIPT_NAME" in
      claim1c_lambda1000) capture_policy=claim1c-hp1000 ;;
      claim8_natural_eigenvector)
        if [[ "$parity" == natural ]]; then capture_policy=claim8-natural; fi
        ;;
    esac
    command+=(--capture-policy "$capture_policy")
  fi
  case "${1:-}" in
    run)
      if [[ "$CLAIM_RESEARCH_CAPTURE" == ultra ]]; then
        command+=(--research-capture ultra --research-sector-eigenpairs "$CLAIM_SECTOR_EIGENPAIRS" --root-acquisition seeded --root-validation "$CLAIM_ROOT_VALIDATION")
        if [[ "$CLAIM_SECTOR_CERTIFICATE" == true ]]; then
          command+=(--capture-sector-gap-certificate)
        fi
      else
        command+=(--research-capture claim --root-acquisition seeded --root-validation off)
      fi
      ;;
    check-evenness)
      if [[ "$CLAIM_RESEARCH_CAPTURE" == ultra ]]; then
        command+=(--research-capture ultra --research-sector-eigenpairs "$CLAIM_SECTOR_EIGENPAIRS" --root-acquisition seeded)
      else
        command+=(--research-capture gap --root-acquisition seeded)
      fi
      ;;
    *)
      echo "run_claim requires run or check-evenness" >&2
      CLAIM_FAILURES=$((CLAIM_FAILURES + 1))
      return 0
      ;;
  esac
  if [[ "$CLAIM_RESEARCH_CAPTURE" == ultra && -n "${XC_TARGET_SPEC_DIR:-}" ]]; then
    if [[ ! "$cutoff" =~ ^[0-9]+$ || ! "$digits" =~ ^[0-9]+$ ]]; then
      echo "Ultra target selection requires the claim's explicit cutoff and precision" >&2
      CLAIM_FAILURES=$((CLAIM_FAILURES + 1))
      return 0
    fi
    export XC_TARGET_SPEC_FILE="${XC_TARGET_SPEC_DIR%/}/c${cutoff}-d${digits}.json"
    if [[ ! -f "$XC_TARGET_SPEC_FILE" ]]; then
      echo "Prepared target specification is missing: $XC_TARGET_SPEC_FILE" >&2
      CLAIM_FAILURES=$((CLAIM_FAILURES + 1))
      return 0
    fi
  fi
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
  if XC_PUBLISH_EXECUTE="$invocation_publish_execute" XC_PERF_REPORT="${XC_PERF_REPORT:-${log%.log}.performance.json}" "${command[@]}" 2>&1 | tee "$log"; then
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
