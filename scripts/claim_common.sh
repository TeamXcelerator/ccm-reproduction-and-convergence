#!/usr/bin/env bash
# Shared zero-configuration launcher for independently runnable claim scripts.

CLAIM_REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$CLAIM_REPO_ROOT"

# Claim scripts accept capture controls directly. Environment variables remain
# available for unattended server jobs, but command-line flags take precedence.
RESEARCH_CAPTURE_LEVEL=${RESEARCH_CAPTURE_LEVEL:-research}
RESEARCH_SECTOR_EIGENPAIRS=${RESEARCH_SECTOR_EIGENPAIRS:-8}
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
    --research-sector-eigenpairs)
      if (($# < 2)); then
        echo "--research-sector-eigenpairs requires a positive integer" >&2
        exit 2
      fi
      RESEARCH_SECTOR_EIGENPAIRS=$2
      shift 2
      ;;
    --help|-h)
      echo "Usage: bash ${BASH_SOURCE[1]} [--research-capture LEVEL] [--research-sector-eigenpairs COUNT]"
      echo "  LEVEL: claim, research (default), gap, or maximum"
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
  cargo build --quiet --release --features hp --locked --bin ccm-reproduction \
    --target-dir "$CLAIM_TARGET_DIR"
elif [[ ! -x "$BIN" ]]; then
  echo "Configured reproduction binary is not executable: $BIN" >&2
  exit 1
fi

# Capture cost is explicit. The balanced "research" default retains the
# complete finite root window and all artifacts naturally produced by that
# calculation, but does not launch the much more expensive parity-sector
# eigenvector analysis. Arithmetic and convergence criteria are identical in
# every mode.
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

run_research_claim() {
  "$BIN" "$@" "${RESEARCH_CAPTURE_ARGS[@]}"
}
