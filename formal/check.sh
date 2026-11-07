#!/bin/sh
# SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
# SPDX-License-Identifier: GPL-3.0-or-later

# Proves every lemma of every model in formal/ and fails unless each one is
# "verified" (all-traces) or has a trace (exists-trace). Each lemma runs on
# its own, with its own time limit, so one slow lemma names itself.
# Usage: formal/check.sh [tamarin-prover command]; default: tamarin-prover.
# FORMAL_MODELS=hayalet-halka-sikimod.spthy formal/check.sh runs one model.
set -eu
tamarin=${1:-tamarin-prover}
# The command may carry arguments; a relative path to its program still
# works after the cd below.
prog=${tamarin%% *}
case $prog in
  */*) tamarin="$(cd "$(dirname "$prog")" && pwd)/$(basename "$prog")${tamarin#"$prog"}" ;;
esac
cd "$(dirname "$0")"
limit=${LEMMA_TIMEOUT:-600}
export LANG=C.UTF-8
status=0
proved=0
# FORMAL_MODELS (space separated file names) limits the run, as CI does
# with one job per model.
for model in ${FORMAL_MODELS:-*.spthy}; do
  [ -f "$model" ] || { echo "no models in formal/"; exit 1; }
  lemmas=$(sed -n 's/^lemma \([A-Za-z0-9_]*\).*/\1/p' "$model")
  [ -n "$lemmas" ] || { echo "$model: no lemmas"; status=1; }
  for lemma in $lemmas; do
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
