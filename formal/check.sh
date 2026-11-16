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
    start=$(date +%s)
    rc=0
    timeout "$limit" $tamarin --derivcheck-timeout=0 --prove="$lemma" "$model" \
      +RTS -M8G -RTS >"$out" 2>&1 || rc=$?
    # Tamarin 1.12.0 enables GHC's threaded RTS (-N) by default (-with-rtsopts=-N).
    # On an observed nondeterministic `tamarin-prover: <<loop>>` abort (upstream
    # root cause unconfirmed), retry once with single-threaded -N1 within the
    # remaining LEMMA_TIMEOUT budget (#569).
    if [ "$rc" -ne 124 ] && grep -q "tamarin-prover: <<loop>>" "$out" && \
        { [ "$rc" -ne 0 ] || ! grep -Eq "^  $lemma \((all-traces|exists-trace)\): verified \(" "$out"; }; then
      elapsed=$(($(date +%s) - start))
      if [ "$elapsed" -lt 0 ]; then
        elapsed=0
      fi
      rem=$((limit - elapsed))
      if [ "$rem" -lt 1 ]; then
        echo "$model:  $lemma: tamarin hit <<loop>>, but ${limit}s budget is exhausted; not retrying"
        rc=124
      else
        echo "$model:  $lemma: tamarin hit <<loop>>; retrying with +RTS -N1 -M8G -RTS (${rem}s remaining)"
        rc=0
        timeout "$rem" $tamarin --derivcheck-timeout=0 --prove="$lemma" "$model" \
          +RTS -N1 -M8G -RTS >"$out" 2>&1 || rc=$?
      fi
    fi
    if [ "$rc" -eq 0 ]; then
      line=$(grep -E "^  $lemma \((all-traces|exists-trace)\): " "$out" || true)
    elif [ "$rc" -eq 124 ]; then
      line="  $lemma: tamarin timed out after ${limit}s"
    else
      line=$(grep -E "^  $lemma \((all-traces|exists-trace)\): " "$out" || true)
      if [ -z "$line" ]; then
        line="  $lemma: tamarin failed (exit $rc)"
      else
        line="$line (exit $rc)"
      fi
    fi
    if grep -q "wellformedness check failed" "$out"; then
      line="$line (wellformedness check failed)"
    fi
    echo "$model:$line"
    case $line in
      *"wellformedness check failed"*) status=1; tail -n 60 "$out" ;;
      *)
        if [ "$rc" -eq 0 ]; then
          case $line in
            *": verified ("*) proved=$((proved + 1)) ;;
            *) status=1; tail -n 40 "$out" ;;
          esac
        else
          status=1
          tail -n 40 "$out"
        fi
        ;;
    esac
    rm -f "$out"
  done
done
if [ "$proved" -eq 0 ]; then
  echo "no lemma was proved"
  status=1
fi
exit $status
