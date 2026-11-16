#!/bin/sh
# SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
# SPDX-License-Identifier: GPL-3.0-or-later

# Regression tests for formal/check.sh and formal/mutants.py using stub
# tamarin-prover scripts, covering the single-threaded `+RTS -N1 -M8G -RTS`
# retry on observed nondeterministic `tamarin-prover: <<loop>>` aborts
# (upstream root cause unconfirmed), retry failure handling, and shared
# `LEMMA_TIMEOUT` budgeting (#569).
set -eu
cd "$(dirname "$0")"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# Single-lemma fixture for fast, targeted check.sh tests.
cat >"$tmp/single.spthy" <<'EOF'
theory Single
begin
lemma check_prop:
  all-traces "T"
end
EOF

# 1. check.sh retries with +RTS -N1 -M8G -RTS when tamarin-prover hits <<loop>>.
cat >"$tmp/tamarin-loop" <<'EOF'
#!/bin/sh
set -eu
printf '%s\n' "$*" >>"$CALLS_LOG"
lemma=""
has_n1=0
for arg in "$@"; do
  case $arg in
    --prove=*) lemma=${arg#--prove=} ;;
    -N1) has_n1=1 ;;
  esac
done
if [ "$has_n1" -eq 0 ]; then
  echo "[Theory Test] Theory closed"
  echo "tamarin-prover: <<loop>>" >&2
  exit 1
fi
echo "  $lemma (all-traces): verified (4 steps)"
EOF
chmod +x "$tmp/tamarin-loop"

: >"$tmp/calls-check.log"
out_check=$(CALLS_LOG="$tmp/calls-check.log" FORMAL_MODELS=hayalet-halka-gunluk.spthy \
  ./check.sh "$tmp/tamarin-loop" 2>&1)
echo "$out_check" | grep -q "retrying with +RTS -N1 -M8G -RTS"
grep -q -- "+RTS -M8G -RTS$" "$tmp/calls-check.log"
grep -q -- "+RTS -N1 -M8G -RTS$" "$tmp/calls-check.log"

# 2. check.sh fails without retrying when tamarin-prover fails for another reason,
#    and reports the actual exit code rather than a timeout.
cat >"$tmp/tamarin-real-fail" <<'EOF'
#!/bin/sh
set -eu
printf '%s\n' "$*" >>"$CALLS_LOG"
echo "tamarin-prover: parse error" >&2
exit 2
EOF
chmod +x "$tmp/tamarin-real-fail"

: >"$tmp/calls-check-fail.log"
if out_fail=$(CALLS_LOG="$tmp/calls-check-fail.log" FORMAL_MODELS=hayalet-halka-gunluk.spthy \
    ./check.sh "$tmp/tamarin-real-fail" 2>&1); then
  echo "expected check.sh to fail on a genuine tamarin error" >&2
  exit 1
fi
echo "$out_fail" | grep -q "tamarin failed (exit 2)"
if echo "$out_fail" | grep -q "timed out"; then
  echo "unexpected timeout message on exit 2 error" >&2
  exit 1
fi
if grep -q -- "-N1" "$tmp/calls-check-fail.log"; then
  echo "unexpected -N1 retry on a non-<<loop>> error" >&2
  exit 1
fi

# 3. check.sh: <<loop>> on attempt 1, then falsified (exit 0) on -N1 retry
#    must fail, print the falsified line, and not count the lemma as proved.
cat >"$tmp/tamarin-loop-falsified-0" <<'EOF'
#!/bin/sh
set -eu
printf '%s\n' "$*" >>"$CALLS_LOG"
lemma=""
has_n1=0
for arg in "$@"; do
  case $arg in
    --prove=*) lemma=${arg#--prove=} ;;
    -N1) has_n1=1 ;;
  esac
done
if [ "$has_n1" -eq 0 ]; then
  echo "tamarin-prover: <<loop>>" >&2
  exit 1
fi
echo "  $lemma (all-traces): falsified - found trace (2 steps)"
exit 0
EOF
chmod +x "$tmp/tamarin-loop-falsified-0"

: >"$tmp/calls-check-lf0.log"
if out_lf0=$(CALLS_LOG="$tmp/calls-check-lf0.log" FORMAL_MODELS="$tmp/single.spthy" \
    ./check.sh "$tmp/tamarin-loop-falsified-0" 2>&1); then
  echo "expected check.sh to fail when -N1 retry falsifies the lemma (exit 0)" >&2
  exit 1
fi
echo "$out_lf0" | grep -q "retrying with +RTS -N1 -M8G -RTS"
echo "$out_lf0" | grep -q "check_prop (all-traces): falsified - found trace (2 steps)"
echo "$out_lf0" | grep -q "no lemma was proved"
if echo "$out_lf0" | grep -q "timed out"; then
  echo "unexpected timeout message on falsified retry (exit 0)" >&2
  exit 1
fi

# 4. check.sh: <<loop>> on attempt 1, then falsified (exit 1) on -N1 retry
#    must fail, report the falsified line (not timeout), and not count as proved.
cat >"$tmp/tamarin-loop-falsified-1" <<'EOF'
#!/bin/sh
set -eu
printf '%s\n' "$*" >>"$CALLS_LOG"
lemma=""
has_n1=0
for arg in "$@"; do
  case $arg in
    --prove=*) lemma=${arg#--prove=} ;;
    -N1) has_n1=1 ;;
  esac
done
if [ "$has_n1" -eq 0 ]; then
  echo "tamarin-prover: <<loop>>" >&2
  exit 1
fi
echo "  $lemma (all-traces): falsified - found trace (3 steps)"
exit 1
EOF
chmod +x "$tmp/tamarin-loop-falsified-1"

: >"$tmp/calls-check-lf1.log"
if out_lf1=$(CALLS_LOG="$tmp/calls-check-lf1.log" FORMAL_MODELS="$tmp/single.spthy" \
    ./check.sh "$tmp/tamarin-loop-falsified-1" 2>&1); then
  echo "expected check.sh to fail when -N1 retry falsifies the lemma (exit 1)" >&2
  exit 1
fi
echo "$out_lf1" | grep -q "check_prop (all-traces): falsified - found trace (3 steps)"
echo "$out_lf1" | grep -q "no lemma was proved"
if echo "$out_lf1" | grep -q "timed out"; then
  echo "unexpected timeout message on falsified retry (exit 1)" >&2
  exit 1
fi

# 5. check.sh: <<loop>> on attempt 1, then non-timeout crash (exit 3) on -N1
#    retry must report the exit code, not a timeout.
cat >"$tmp/tamarin-loop-crash" <<'EOF'
#!/bin/sh
set -eu
has_n1=0
for arg in "$@"; do
  if [ "$arg" = "-N1" ]; then
    has_n1=1
  fi
done
if [ "$has_n1" -eq 0 ]; then
  echo "tamarin-prover: <<loop>>" >&2
  exit 1
fi
echo "tamarin-prover: internal failure" >&2
exit 3
EOF
chmod +x "$tmp/tamarin-loop-crash"

if out_crash=$(FORMAL_MODELS="$tmp/single.spthy" ./check.sh "$tmp/tamarin-loop-crash" 2>&1); then
  echo "expected check.sh to fail when -N1 retry crashes" >&2
  exit 1
fi
echo "$out_crash" | grep -q "tamarin failed (exit 3)"
if echo "$out_crash" | grep -q "timed out"; then
  echo "unexpected timeout message on retry crash (exit 3)" >&2
  exit 1
fi

# 6. check.sh: <<loop>> on attempt 1, then non-zero exit despite a verified
#    line on -N1 retry must NOT be counted as proved.
cat >"$tmp/tamarin-loop-verified-exit1" <<'EOF'
#!/bin/sh
set -eu
lemma=""
has_n1=0
for arg in "$@"; do
  case $arg in
    --prove=*) lemma=${arg#--prove=} ;;
    -N1) has_n1=1 ;;
  esac
done
if [ "$has_n1" -eq 0 ]; then
  echo "tamarin-prover: <<loop>>" >&2
  exit 1
fi
echo "  $lemma (all-traces): verified (4 steps)"
exit 1
EOF
chmod +x "$tmp/tamarin-loop-verified-exit1"

if out_ve1=$(FORMAL_MODELS="$tmp/single.spthy" ./check.sh "$tmp/tamarin-loop-verified-exit1" 2>&1); then
  echo "expected check.sh to fail when -N1 retry exits non-zero" >&2
  exit 1
fi
echo "$out_ve1" | grep -q "no lemma was proved"

# 7a. check.sh subtracts elapsed seconds from LEMMA_TIMEOUT for the -N1 retry.
real_date=$(command -v date)
mkdir -p "$tmp/spy-rem"
cat >"$tmp/spy-rem/date" <<'EOF'
#!/bin/sh
set -eu
if [ "${1:-}" = "+%s" ]; then
  n=$(cat "$DATE_COUNT" 2>/dev/null || echo 0)
  n=$((n + 1))
  printf '%s\n' "$n" >"$DATE_COUNT"
  if [ "$n" -eq 1 ]; then
    echo 1000
  else
    echo 1420
  fi
  exit 0
fi
exec "$REAL_DATE" "$@"
EOF
cat >"$tmp/spy-rem/timeout" <<'EOF'
#!/bin/sh
set -eu
printf '%s\n' "$1" >>"$TIMEOUTS_LOG"
shift
exec "$@"
EOF
chmod +x "$tmp/spy-rem/date" "$tmp/spy-rem/timeout"

: >"$tmp/calls-rem.log"
: >"$tmp/timeouts-rem.log"
rm -f "$tmp/date-rem.count"
out_rem=$(PATH="$tmp/spy-rem:$PATH" REAL_DATE="$real_date" \
  DATE_COUNT="$tmp/date-rem.count" TIMEOUTS_LOG="$tmp/timeouts-rem.log" \
  CALLS_LOG="$tmp/calls-rem.log" LEMMA_TIMEOUT=600 \
  FORMAL_MODELS="$tmp/single.spthy" ./check.sh "$tmp/tamarin-loop" 2>&1)
echo "$out_rem" | grep -q "180s remaining"
[ "$(tr '\n' ',' <"$tmp/timeouts-rem.log")" = "600,180," ] || {
  echo "expected timeout budgets 600 then 180, got: $(cat "$tmp/timeouts-rem.log")" >&2
  exit 1
}

# 7b. check.sh skips the -N1 retry and reports timeout when remaining budget <= 0.
mkdir -p "$tmp/spy-zero"
cat >"$tmp/spy-zero/date" <<'EOF'
#!/bin/sh
set -eu
if [ "${1:-}" = "+%s" ]; then
  n=$(cat "$DATE_COUNT" 2>/dev/null || echo 0)
  n=$((n + 1))
  printf '%s\n' "$n" >"$DATE_COUNT"
  if [ "$n" -eq 1 ]; then
    echo 1000
  else
    echo 1600
  fi
  exit 0
fi
exec "$REAL_DATE" "$@"
EOF
cp "$tmp/spy-rem/timeout" "$tmp/spy-zero/timeout"
chmod +x "$tmp/spy-zero/date" "$tmp/spy-zero/timeout"

: >"$tmp/calls-zero.log"
: >"$tmp/timeouts-zero.log"
rm -f "$tmp/date-zero.count"
if out_zero=$(PATH="$tmp/spy-zero:$PATH" REAL_DATE="$real_date" \
    DATE_COUNT="$tmp/date-zero.count" TIMEOUTS_LOG="$tmp/timeouts-zero.log" \
    CALLS_LOG="$tmp/calls-zero.log" LEMMA_TIMEOUT=600 \
    FORMAL_MODELS="$tmp/single.spthy" ./check.sh "$tmp/tamarin-loop" 2>&1); then
  echo "expected check.sh to fail when budget is exhausted before retry" >&2
  exit 1
fi
echo "$out_zero" | grep -q "tamarin timed out after 600s"
if grep -q -- "-N1" "$tmp/calls-zero.log"; then
  echo "unexpected -N1 retry when LEMMA_TIMEOUT budget was already exhausted" >&2
  exit 1
fi
[ "$(tr '\n' ',' <"$tmp/timeouts-zero.log")" = "600," ] || {
  echo "expected only one timeout call (600), got: $(cat "$tmp/timeouts-zero.log")" >&2
  exit 1
}

# 7c. check.sh enforces shared wall-clock LEMMA_TIMEOUT with real GNU timeout.
cat >"$tmp/tamarin-slow-loop" <<'EOF'
#!/bin/sh
set -eu
lemma=""
has_n1=0
for arg in "$@"; do
  case $arg in
    --prove=*) lemma=${arg#--prove=} ;;
    -N1) has_n1=1 ;;
  esac
done
if [ "$has_n1" -eq 0 ]; then
  sleep 1.1
  echo "tamarin-prover: <<loop>>" >&2
  exit 1
fi
sleep 1.5
echo "  $lemma (all-traces): verified (4 steps)"
EOF
chmod +x "$tmp/tamarin-slow-loop"

if out_slow=$(LEMMA_TIMEOUT=2 FORMAL_MODELS="$tmp/single.spthy" \
    ./check.sh "$tmp/tamarin-slow-loop" 2>&1); then
  echo "expected check.sh to time out when attempt 1 + retry exceed LEMMA_TIMEOUT" >&2
  exit 1
fi
echo "$out_slow" | grep -q "tamarin timed out after 2s"

# 8. mutants.py retries with +RTS -N1 -M8G -RTS when tamarin-prover hits <<loop>>.
cat >"$tmp/tamarin-mutant-loop" <<'EOF'
#!/bin/sh
set -eu
printf '%s\n' "$*" >>"$CALLS_LOG"
has_n1=0
for arg in "$@"; do
  if [ "$arg" = "-N1" ]; then
    has_n1=1
  fi
done
if [ "$has_n1" -eq 0 ]; then
  echo "[Theory Mutant] Theory closed"
  echo "tamarin-prover: <<loop>>" >&2
  exit 1
fi
echo "  attack (exists-trace): verified (3 steps)"
EOF
chmod +x "$tmp/tamarin-mutant-loop"

: >"$tmp/calls-mutants.log"
out_mut=$(CALLS_LOG="$tmp/calls-mutants.log" FORMAL_MODELS=hayalet-halka-tazelik.spthy \
  python3 ./mutants.py "$tmp/tamarin-mutant-loop" 2>&1)
echo "$out_mut" | grep -q "retrying with +RTS -N1 -M8G -RTS"
echo "$out_mut" | grep -q "attack found"
grep -q -- "+RTS -M8G -RTS$" "$tmp/calls-mutants.log"
grep -q -- "+RTS -N1 -M8G -RTS$" "$tmp/calls-mutants.log"

# 9. mutants.py fails without retrying when no attack is found and <<loop>> did not occur.
cat >"$tmp/tamarin-mutant-miss" <<'EOF'
#!/bin/sh
set -eu
printf '%s\n' "$*" >>"$CALLS_LOG"
echo "  attack (exists-trace): falsified - no trace found"
EOF
chmod +x "$tmp/tamarin-mutant-miss"

: >"$tmp/calls-mutants-miss.log"
if out_miss=$(CALLS_LOG="$tmp/calls-mutants-miss.log" FORMAL_MODELS=hayalet-halka-tazelik.spthy \
    python3 ./mutants.py "$tmp/tamarin-mutant-miss" 2>&1); then
  echo "expected mutants.py to fail when no attack trace is found" >&2
  exit 1
fi
echo "$out_miss" | grep -q "NO ATTACK FOUND"
if grep -q -- "-N1" "$tmp/calls-mutants-miss.log"; then
  echo "unexpected -N1 retry when <<loop>> did not occur" >&2
  exit 1
fi

# 10. mutants.py: <<loop>> on attempt 1, then falsified on -N1 retry must fail
#     and print NO ATTACK FOUND.
cat >"$tmp/tamarin-mutant-loop-falsified" <<'EOF'
#!/bin/sh
set -eu
printf '%s\n' "$*" >>"$CALLS_LOG"
has_n1=0
for arg in "$@"; do
  if [ "$arg" = "-N1" ]; then
    has_n1=1
  fi
done
if [ "$has_n1" -eq 0 ]; then
  echo "tamarin-prover: <<loop>>" >&2
  exit 1
fi
echo "  attack (exists-trace): falsified - no trace found"
EOF
chmod +x "$tmp/tamarin-mutant-loop-falsified"

: >"$tmp/calls-mutants-lf.log"
if out_mlf=$(CALLS_LOG="$tmp/calls-mutants-lf.log" FORMAL_MODELS=hayalet-halka-tazelik.spthy \
    python3 ./mutants.py "$tmp/tamarin-mutant-loop-falsified" 2>&1); then
  echo "expected mutants.py to fail when -N1 retry falsifies the attack" >&2
  exit 1
fi
echo "$out_mlf" | grep -q "retrying with +RTS -N1 -M8G -RTS"
echo "$out_mlf" | grep -q "NO ATTACK FOUND"
echo "$out_mlf" | grep -q "attack (exists-trace): falsified"

# 11a. mutants.py enforces one shared LEMMA_TIMEOUT deadline across attempt 1 and -N1 retry.
cat >"$tmp/tamarin-mutant-slow-loop" <<'EOF'
#!/bin/sh
set -eu
has_n1=0
for arg in "$@"; do
  if [ "$arg" = "-N1" ]; then
    has_n1=1
  fi
done
if [ ! -f "$SLOW_DONE" ]; then
  if [ "$has_n1" -eq 0 ]; then
    sleep 1.1
    echo "tamarin-prover: <<loop>>" >&2
    exit 1
  fi
  : >"$SLOW_DONE"
  sleep 1.5
fi
echo "  attack (exists-trace): verified (3 steps)"
EOF
chmod +x "$tmp/tamarin-mutant-slow-loop"

rm -f "$tmp/mutant-slow.done"
if out_mslow=$(LEMMA_TIMEOUT=2 SLOW_DONE="$tmp/mutant-slow.done" \
    FORMAL_MODELS=hayalet-halka-tazelik.spthy \
    python3 ./mutants.py "$tmp/tamarin-mutant-slow-loop" 2>&1); then
  echo "expected mutants.py to time out when attempt 1 + retry exceed LEMMA_TIMEOUT" >&2
  exit 1
fi
echo "$out_mslow" | grep -q "NO ATTACK FOUND"
echo "$out_mslow" | grep -q "timed out"

# 11b. mutants.py skips the -N1 retry and treats the mutant as timed out when
#      remaining <= 0 after attempt 1.
: >"$tmp/calls-mutants-zero.log"
out_mzero=$(CALLS_LOG="$tmp/calls-mutants-zero.log" \
  FORMAL_MODELS=hayalet-halka-tazelik.spthy \
  python3 - "$tmp/tamarin-mutant-loop" <<'PY' 2>&1
import subprocess
import sys
import time
import mutants

now = [1000.0]
orig_run = subprocess.run

def advancing_run(*args, **kwargs):
    res = orig_run(*args, **kwargs)
    now[0] += 601.0
    return res

time.monotonic = lambda: now[0]
subprocess.run = advancing_run
mutants.TIMEOUT = 600
sys.argv = ["mutants.py", sys.argv[1]]
try:
    mutants.main()
except SystemExit as exc:
    if exc.code == 0:
        sys.stderr.write("expected non-zero exit when budget exhausted\n")
        sys.exit(2)
    sys.exit(0)
sys.stderr.write("expected SystemExit\n")
sys.exit(2)
PY
)
echo "$out_mzero" | grep -q "NO ATTACK FOUND"
echo "$out_mzero" | grep -q "timed out"
if grep -q -- "-N1" "$tmp/calls-mutants-zero.log"; then
  echo "unexpected -N1 retry in mutants.py when remaining budget <= 0" >&2
  exit 1
fi

echo "formal runner tests ok"
