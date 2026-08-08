#!/usr/bin/env bash
# Shared zero-configuration launcher for independently runnable claim scripts.

CLAIM_REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$CLAIM_REPO_ROOT"

# Claim scripts accept capture controls directly. Environment variables remain
# available for unattended server jobs, but command-line flags take precedence.
RESEARCH_CAPTURE_LEVEL=${RESEARCH_CAPTURE_LEVEL:-research}
RESEARCH_SECTOR_EIGENPAIRS=${RESEARCH_SECTOR_EIGENPAIRS:-8}
ROOT_ACQUISITION_MODE=${ROOT_ACQUISITION_MODE:-seeded}
PARITY_POLICY=${PARITY_POLICY:-even-sector}
ROOT_VALIDATION_LEVEL=${ROOT_VALIDATION_LEVEL:-off}
ROOT_ENCLOSURE_DIGITS=${ROOT_ENCLOSURE_DIGITS:-}
INCLUDE_NEGATIVE_ROOTS=${INCLUDE_NEGATIVE_ROOTS:-false}
ALLOW_ROOT_OVERSUBSCRIPTION=${ALLOW_ROOT_OVERSUBSCRIPTION:-false}
while (($# > 0)); do
  case "$1" in
    --research-capture)
      if (($# < 2)); then
        echo "--research-capture requires claim, research, gap, or maximum" >&2
        exit 2
      fi
      RESEARCH_CAPTURE_LEVEL=$2
      shift 2
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
    --help|-h)
      echo "Usage: bash ${BASH_SOURCE[1]} [--research-capture LEVEL] [--research-sector-eigenpairs COUNT] [--root-acquisition MODE] [--parity-policy POLICY] [--root-validation LEVEL] [--root-enclosure-digits DIGITS] [--include-negative-roots] [--allow-root-oversubscription] [--verify-cache]"
      echo "  LEVEL: claim, research (default), gap, or maximum"
      echo "  ROOT ACQUISITION: seeded (default for every claim script) or independent"
      echo "  PARITY POLICY: even-sector (default), natural, or adaptive-even"
      echo "  ROOT VALIDATION: off (default) or certified"
      echo "  ROOT ENCLOSURE: defaults to the claim's display digits; override only when needed"
      echo "  ADVANCED ROOTS: signed and finite-shortfall controls require independent HP discovery"
      echo "  CACHE VALIDATION: --verify-cache recomputes and compares claim artifacts; disabled by default"
      exit 0
      ;;
    *)
      echo "Unknown claim-script argument: $1" >&2
      echo "Use --help for supported capture controls." >&2
      exit 2
      ;;
  esac
done

if [[ -z "${BIN+x}" ]]; then
  CLAIM_TARGET_DIR="$CLAIM_REPO_ROOT/target"
  BIN="$CLAIM_TARGET_DIR/release/ccm-reproduction"
  # Cargo's incremental freshness check is quick and prevents an executable
  # built from an older toolkit lockfile, without HP, or in an externally
  # overridden CARGO_TARGET_DIR from being mistaken for the current binary.
  CLAIM_FEATURES=hp
  if [[ "$ROOT_VALIDATION_LEVEL" == "certified" ]]; then
    CLAIM_FEATURES=hp,root-certification
  fi
  cargo build --quiet --release --features "$CLAIM_FEATURES" --locked --bin ccm-reproduction \
    --target-dir "$CLAIM_TARGET_DIR"
elif [[ ! -x "$BIN" ]]; then
  echo "Configured reproduction binary is not executable: $BIN" >&2
  exit 1
fi

# Capture cost is explicit. The balanced "research" default retains the
# claim's explicit ordinal root window and all artifacts naturally produced by
# that calculation, but does not launch the much more expensive parity-sector
# eigenvector analysis. Every paper claim defaults to seeded acquisition, and
# capture level cannot change the selected policy. Arithmetic and convergence
# criteria are identical in every mode.
case "$RESEARCH_CAPTURE_LEVEL" in
  claim|research|gap)
    RESEARCH_CAPTURE_ARGS=(--research-capture "$RESEARCH_CAPTURE_LEVEL")
    ;;
  maximum)
    RESEARCH_CAPTURE_ARGS=(
      --research-capture maximum
      --research-sector-eigenpairs "$RESEARCH_SECTOR_EIGENPAIRS"
    )
    ;;
  *)
    echo "RESEARCH_CAPTURE_LEVEL must be claim, research, gap, or maximum" >&2
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

run_research_claim() {
  if [[ "${1:-}" == "run" ]]; then
    "$BIN" "$@" "${RESEARCH_CAPTURE_ARGS[@]}" "${ROOT_ACQUISITION_ARGS[@]}" "${PARITY_POLICY_ARGS[@]}" "${ROOT_VALIDATION_ARGS[@]}" "${ADVANCED_ROOT_ARGS[@]}"
  elif [[ "${1:-}" == "check-evenness" ]]; then
    "$BIN" "$@" "${RESEARCH_CAPTURE_ARGS[@]}" "${ROOT_ACQUISITION_ARGS[@]}"
  else
    "$BIN" "$@" "${RESEARCH_CAPTURE_ARGS[@]}"
  fi
}
