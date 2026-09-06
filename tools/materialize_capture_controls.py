#!/usr/bin/env python3
"""Owner-authorized, source-hash-guarded claim launcher integration.
AI-generated assistance. This migration is removed after launcher qualification.
"""
from pathlib import Path
import hashlib

root = Path(__file__).resolve().parents[1]
path = root / "scripts/claim_common.sh"
raw = path.read_bytes()
assert hashlib.sha1(b"blob " + str(len(raw)).encode() + b"\0" + raw).hexdigest() == "9178610f9fc0c2999127386bdde11bb5098b2ed6"
s = raw.decode()

def once(text, old, new):
    if text.count(old) != 1:
        raise RuntimeError(f"ambiguous launcher edit: {old[:100]!r}")
    return text.replace(old, new, 1)

s = once(s, 'CAPTURE_DISTANCE=${CAPTURE_DISTANCE:-false}\n', '''CAPTURE_DISTANCE=${CAPTURE_DISTANCE:-false}
CAPTURE_DEVIATION_DECOMPOSITION=${CAPTURE_DEVIATION_DECOMPOSITION:-false}
CAPTURE_PRIME_POWER_RESPONSE=${CAPTURE_PRIME_POWER_RESPONSE:-false}
CAPTURE_U_FLOW_RESPONSE=${CAPTURE_U_FLOW_RESPONSE:-false}
CAPTURE_SECTOR_GAP_CERTIFICATE=${CAPTURE_SECTOR_GAP_CERTIFICATE:-false}
''')
s = once(s, '    --capture-distance)\n', '''    --capture-deviation-decomposition)
      CAPTURE_DEVIATION_DECOMPOSITION=true
      shift
      ;;
    --capture-prime-power-response)
      CAPTURE_PRIME_POWER_RESPONSE=true
      shift
      ;;
    --capture-u-flow-response)
      CAPTURE_U_FLOW_RESPONSE=true
      shift
      ;;
    --capture-sector-gap-certificate)
      CAPTURE_SECTOR_GAP_CERTIFICATE=true
      shift
      ;;
    --capture-distance)
''')
s = once(s, '      echo "  TARGET DISTANCE:', '''      echo "  EXTRA MEASUREMENTS: --capture-deviation-decomposition (requires distance),"
      echo "                      --capture-prime-power-response, --capture-u-flow-response"
      echo "  EXPLICIT PROOF: --capture-sector-gap-certificate requires gap/maximum/ultra"
      echo "                 and the corrected Toolkit baseline. Ultra alone never enables it."
      echo "  TARGET DISTANCE:''')
anchor = 'if [[ -z "${BIN+x}" ]]; then\n'
insert = '''# Validate extra controls before building or launching expensive work.
for CAPTURE_CONTROL in CAPTURE_DEVIATION_DECOMPOSITION CAPTURE_PRIME_POWER_RESPONSE CAPTURE_U_FLOW_RESPONSE CAPTURE_SECTOR_GAP_CERTIFICATE; do
  case "${!CAPTURE_CONTROL}" in
    true|false) ;;
    *) echo "$CAPTURE_CONTROL must be true or false" >&2; exit 2 ;;
  esac
done
if [[ "$CAPTURE_DEVIATION_DECOMPOSITION" == "true" && "$CAPTURE_DISTANCE" != "true" ]]; then
  echo "--capture-deviation-decomposition requires --capture-distance" >&2
  exit 2
fi
if [[ "$CAPTURE_PRIME_POWER_RESPONSE" == "true" || "$CAPTURE_U_FLOW_RESPONSE" == "true" ]]; then
  if [[ "$PARITY_POLICY" != "even-sector" ]]; then
    echo "Prime-power and u-flow response controls require even-sector parity" >&2
    exit 2
  fi
fi
if [[ "$CAPTURE_SECTOR_GAP_CERTIFICATE" == "true" ]]; then
  case "$RESEARCH_CAPTURE_LEVEL" in
    gap|maximum|ultra) ;;
    *) echo "--capture-sector-gap-certificate requires gap, maximum, or ultra capture" >&2; exit 2 ;;
  esac
  if [[ ! "$RESEARCH_SECTOR_EIGENPAIRS" =~ ^[0-9]+$ ]] || ((RESEARCH_SECTOR_EIGENPAIRS < 2)); then
    echo "Sector-gap certification requires at least two sector eigenpairs" >&2
    exit 2
  fi
fi

'''
s = once(s, anchor, insert + anchor)
s = once(s, '  if [[ "$ROOT_VALIDATION_LEVEL" == "certified" ]]; then', '  if [[ "$ROOT_VALIDATION_LEVEL" == "certified" || "$CAPTURE_SECTOR_GAP_CERTIFICATE" == "true" ]]; then')
anchor = 'BENCHMARK_ARGS=()\n'
s = once(s, anchor, '''EXTRA_CAPTURE_ARGS=()
[[ "$CAPTURE_DEVIATION_DECOMPOSITION" != "true" ]] || EXTRA_CAPTURE_ARGS+=(--capture-deviation-decomposition)
[[ "$CAPTURE_PRIME_POWER_RESPONSE" != "true" ]] || EXTRA_CAPTURE_ARGS+=(--capture-prime-power-response)
[[ "$CAPTURE_U_FLOW_RESPONSE" != "true" ]] || EXTRA_CAPTURE_ARGS+=(--capture-u-flow-response)
[[ "$CAPTURE_SECTOR_GAP_CERTIFICATE" != "true" ]] || EXTRA_CAPTURE_ARGS+=(--capture-sector-gap-certificate)

''' + anchor)
s = once(s, 'run_research_claim() {\n', '''run_research_claim() {
  if [[ "${1:-}" != "run" ]] && ((${#EXTRA_CAPTURE_ARGS[@]} > 0)); then
    echo "Explicit extra capture controls apply only to the run subcommand" >&2
    return 2
  fi
''')
s = once(s, '"${ADVANCED_ROOT_ARGS[@]}" "${DISTANCE_ARGS[@]}"\n', '"${ADVANCED_ROOT_ARGS[@]}" "${DISTANCE_ARGS[@]}" "${EXTRA_CAPTURE_ARGS[@]}"\n')
path.write_text(s)
print("materialized explicit launcher controls; ultra defaults unchanged")
