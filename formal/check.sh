#!/bin/sh
# SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
# SPDX-License-Identifier: GPL-3.0-or-later

# Proves every lemma of every model in formal/ and fails unless each one is
# "verified" (all-traces) or has a trace (exists-trace). Each lemma runs on
# its own, with its own time limit, so one slow lemma names itself.
# Usage: formal/check.sh [tamarin-prover command]; default: tamarin-prover.
set -eu
tamarin=${1:-tamarin-prover}
# A relative path to the prover still works after the cd below.
case $tamarin in
  */*) tamarin=$(cd "$(dirname "$tamarin")" && pwd)/$(basename "$tamarin") ;;
esac
cd "$(dirname "$0")"
limit=${LEMMA_TIMEOUT:-600}
export LANG=C.UTF-8
status=0
proved=0
for model in *.spthy; do
  [ -f "$model" ] || { echo "no models in formal/"; exit 1; }
  for lemma in $(sed -n 's/^lemma \([A-Za-z0-9_]*\).*/\1/p' "$model"); do
    out=$(mktemp)
    if timeout "$limit" $tamarin --derivcheck-timeout=0 --prove="$lemma" "$model" \
        +RTS -M8G -RTS >"$out" 2>&1; then
      line=$(grep -E "^  $lemma \((all-traces|exists-trace)\): " "$out" || true)
    else
      line="  $lemma: tamarin failed or timed out after ${limit}s"
    fi
    echo "$model:$line"
    if grep -q "wellformedness check failed" "$out"; then
      line="$line (wellformedness check failed)"
    fi
    case $line in
      *"wellformedness check failed"*) status=1; tail -n 60 "$out" ;;
      *": verified ("*) proved=$((proved + 1)) ;;
      *) status=1; tail -n 40 "$out" ;;
    esac
    rm -f "$out"
  done
done
if [ "$proved" -eq 0 ]; then
  echo "no lemma was proved"
  status=1
fi
exit $status
