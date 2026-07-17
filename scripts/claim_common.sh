#!/usr/bin/env bash
# Shared zero-configuration launcher for independently runnable claim scripts.

CLAIM_REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$CLAIM_REPO_ROOT"

BIN=${BIN:-./target/release/ccm-reproduction}
if [[ "$BIN" == "./target/release/ccm-reproduction" ]]; then
  # Cargo's incremental freshness check is quick and prevents an executable
  # built from an older toolkit lockfile or without the HP feature from being
  # mistaken for the current reproduction binary.
  cargo build --quiet --release --features hp --bin ccm-reproduction
elif [[ ! -x "$BIN" ]]; then
  echo "Configured reproduction binary is not executable: $BIN" >&2
  exit 1
fi
