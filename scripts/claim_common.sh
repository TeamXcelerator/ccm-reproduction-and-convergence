#!/usr/bin/env bash
# Shared zero-configuration launcher for independently runnable claim scripts.

CLAIM_REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$CLAIM_REPO_ROOT"

if [[ -z "${BIN+x}" ]]; then
  CLAIM_TARGET_DIR="$CLAIM_REPO_ROOT/target"
  BIN="$CLAIM_TARGET_DIR/release/ccm-reproduction"
  # Cargo's incremental freshness check is quick and prevents an executable
  # built from an older toolkit lockfile, without HP, or in an externally
  # overridden CARGO_TARGET_DIR from being mistaken for the current binary.
  cargo build --quiet --release --features hp --bin ccm-reproduction \
    --target-dir "$CLAIM_TARGET_DIR"
elif [[ ! -x "$BIN" ]]; then
  echo "Configured reproduction binary is not executable: $BIN" >&2
  exit 1
fi
