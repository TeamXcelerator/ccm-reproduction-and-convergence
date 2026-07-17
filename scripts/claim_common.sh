#!/usr/bin/env bash
# Shared zero-configuration launcher for independently runnable claim scripts.

CLAIM_REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$CLAIM_REPO_ROOT"

BIN=${BIN:-./target/release/ccm-reproduction}
if [[ ! -x "$BIN" ]]; then
  echo "ccm-reproduction is not built; building the release binary..."
  cargo build --release --features hp --bin ccm-reproduction
fi
