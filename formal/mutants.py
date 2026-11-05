#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
# SPDX-License-Identifier: GPL-3.0-or-later

"""Mutation tests for the Tamarin models in formal/.

A proof only means something if the model would fail without the rule it
relies on. Each mutant below breaks one check in a copy of the model and
adds an attack lemma; Tamarin must find the attack trace in the mutant.
The same scenario is a proved lemma in the real model (check.sh).

Usage: formal/mutants.py [tamarin-prover command]
"""

import os
import re
import shlex
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
CHAIN = os.path.join(HERE, "hayalet-halka-zincir.spthy")
TIMEOUT = int(os.environ.get("LEMMA_TIMEOUT", "600"))

BOUND_ONE_STEP = """
    & (All t #x #y. Step(t) @ x & Step(t) @ y ==> #x = #y)
    & not (Ex #x. Step('MT') @ x) & not (Ex #x. Step('AT') @ x)
    & not (Ex #x. Step('MR') @ x) & not (Ex #x. Step('AR') @ x)
    & not (Ex #x. Step('MA') @ x)"
"""

MUTANTS = [
    {
        "name": "accept_checks_ed_half_only",
        "why": "a contact checks only the Ed25519 half of the AK signature",
        "model": CHAIN,
        "edits": [
            ("rule Accept_Add_1:",
             r"(rule Accept_Add_1:(?:(?!\nrule ).)*?)      Eq\(verify\(sig2, <'ml', body>, sa2\), true\),\n",
             r"\1", 1),
        ],
        "lemma": """
lemma attack:
  exists-trace
  "Ex c r h0 n s #p #i #k.
      Pin(c, r, h0) @ p
    & Accept(c, r, h0, 'add', n, s) @ i
    & RevA1(s) @ k
    & not (Ex #x. RevA2(s) @ x)
    & not (Ex #j. Signed(s, r, h0, 'add', n) @ j)""" + BOUND_ONE_STEP,
    },
    {
        "name": "add_does_not_bind_previous_head",
        "why": "an add record does not name the head it extends; an old add replays",
        "model": CHAIN,
        "edits": [
            ("honest add", r"body = <'rec', h\(rb\), h\(hb\), 'add', n, d>",
             "body = <'rec', h(rb), 'any', 'add', n, d>", 2),
            ("accepted add", r"body = <'rec', r, hh, 'add', n, s>",
             "body = <'rec', r, 'any', 'add', n, s>", 2),
        ],
        "lemma": """
lemma attack:
  exists-trace
  "Ex c r h0 h1 h2 n s #p #i #j #k.
      Pin(c, r, h0) @ p
    & Accept(c, r, h0, 'add', n, s) @ i
    & h1 = h(<'rec', r, 'any', 'add', n, s>)
    & Accept(c, r, h1, 'remove', h(n), s) @ j
    & Accept(c, r, h2, 'add', n, s) @ k
    & not (Ex #x. Signed(s, r, h2, 'add', n) @ x)
    & not (Ex d #x. RevA1(d) @ x) & not (Ex d #x. RevA2(d) @ x)
    & not (Ex #x. Step('MT') @ x) & not (Ex #x. Step('AT') @ x)
    & (All #x #y. Step('MA') @ x & Step('MA') @ y ==> #x = #y)
    & (All #x #y. Step('MR') @ x & Step('MR') @ y ==> #x = #y)
    & (All #x #y. Step('AR') @ x & Step('AR') @ y ==> #x = #y)"
""",
    },
    {
        "name": "rotate_signature_not_checked",
        "why": "a contact accepts a rotate without checking its IK signature",
        "model": CHAIN,
        "edits": [
            ("rule Accept_Rotate_1:",
             r"(rule Accept_Rotate_1:(?:(?!\nrule ).)*?)  --\[ (Step\('AT'\), )Eq\(verify\(sig, <'ik', body>, sik\), true\),\n      ",
             r"\1  --[ \2", 1),
        ],
        "lemma": """
lemma attack:
  exists-trace
  "Ex c r h0 x s #p #i.
      Pin(c, r, h0) @ p
    & Accept(c, r, h0, 'rotate', x, s) @ i
    & not (Ex d #k. RevIK(d) @ k)
    & not (Ex #j. Signed(s, r, h0, 'rotate', x) @ j)
    & (All t #x #y. Step(t) @ x & Step(t) @ y ==> #x = #y)
    & not (Ex #x. Step('MT') @ x) & not (Ex #x. Step('MA') @ x)
    & not (Ex #x. Step('AA') @ x) & not (Ex #x. Step('AR') @ x)
    & not (Ex #x. Step('MR') @ x)"
""",
    },
    {
        "name": "accept_checks_ml_half_only",
        "why": "a contact checks only the ML-DSA half of the AK signature",
        "model": CHAIN,
        "edits": [
            ("rule Accept_Add_1:",
             r"(rule Accept_Add_1:(?:(?!\nrule ).)*?)Eq\(verify\(sig1, <'ed', body>, sa1\), true\),\n      ",
             r"\1", 1),
        ],
        "lemma": """
lemma attack:
  exists-trace
  "Ex c r h0 n s #p #i #k.
      Pin(c, r, h0) @ p
    & Accept(c, r, h0, 'add', n, s) @ i
    & RevA2(s) @ k
    & not (Ex #x. RevA1(s) @ x)
    & not (Ex #j. Signed(s, r, h0, 'add', n) @ j)""" + BOUND_ONE_STEP,
    },
    {
        "name": "accept_signer_not_member",
        "why": "a contact takes the signer's keys from the record, not from its view",
        "model": CHAIN,
        "edits": [
            ("premise", r"(rule Accept_Add_1:(?:(?!\nrule ).)*?)\[ View\(o, r, hh, s, 'empty'\)",
             r"\1[ View(o, r, hh, m, 'empty')", 1),
            ("conclusion", r"(rule Accept_Add_1:(?:(?!\nrule ).)*?)\[ View\(o, r, h\(body\), s, n\) \]",
             r"\1[ View(o, r, h(body), m, n) ]", 1),
        ],
        "lemma": """
lemma attack:
  exists-trace
  "Ex c r h0 n s #p #i.
      Pin(c, r, h0) @ p
    & Accept(c, r, h0, 'add', n, s) @ i
    & not (Ex #x. DeviceNew(s) @ x)
    & not (Ex d #x. RevA1(d) @ x) & not (Ex d #x. RevA2(d) @ x)""" + BOUND_ONE_STEP,
    },
    {
        "name": "remove_signature_not_checked",
        "why": "a contact accepts a remove without checking its AK signature",
        "model": CHAIN,
        "edits": [
            ("rule Accept_Remove_1:",
             r"(rule Accept_Remove_1:(?:(?!\nrule ).)*?)Eq\(verify\(sig1, <'ed', body>, sa1\), true\),\n      Eq\(verify\(sig2, <'ml', body>, sa2\), true\),\n      ",
             r"\1", 1),
        ],
        "lemma": """
lemma attack:
  exists-trace
  "Ex c r h0 h1 n s #p #i #j.
      Pin(c, r, h0) @ p
    & Accept(c, r, h0, 'add', n, s) @ i
    & h1 = h(<'rec', r, h0, 'add', n, s>)
    & Accept(c, r, h1, 'remove', h(n), s) @ j
    & not (Ex #x. Signed(s, r, h1, 'remove', h(n)) @ x)
    & not (Ex d #x. RevA1(d) @ x) & not (Ex d #x. RevA2(d) @ x)
    & (All t #x #y. Step(t) @ x & Step(t) @ y ==> #x = #y)
    & not (Ex #x. Step('MT') @ x) & not (Ex #x. Step('AT') @ x)
    & not (Ex #x. Step('MR') @ x)"
""",
    },
]


def mutate(text, mutant):
    for label, pattern, replacement, expected in mutant["edits"]:
        text, count = re.subn(pattern, replacement, text, flags=re.S)
        if count != expected:
            sys.exit(f"{mutant['name']}: edit '{label}' applied {count} times, "
                     f"expected {expected}; update mutants.py")
    end = text.rindex("\nend")
    return text[:end] + "\n" + mutant["lemma"] + "\nend\n"


def main():
    tamarin = shlex.split(sys.argv[1]) if len(sys.argv) > 1 else ["tamarin-prover"]
    env = dict(os.environ, LANG="C.UTF-8")
    failed = 0
    for mutant in MUTANTS:
        with open(mutant["model"], encoding="utf-8") as f:
            text = mutate(f.read(), mutant)
        with tempfile.NamedTemporaryFile("w", suffix=".spthy", delete=False, encoding="utf-8") as f:
            f.write(text)
            path = f.name
        try:
            run = subprocess.run(
                tamarin + ["--derivcheck-timeout=0", "--prove=attack", path, "+RTS", "-M8G", "-RTS"],
                capture_output=True, text=True, env=env, timeout=TIMEOUT)
            out = run.stdout + run.stderr
        except subprocess.TimeoutExpired:
            out = "timed out"
        finally:
            os.unlink(path)
        found = re.search(r"^  attack \(exists-trace\): verified", out, re.M)
        print(f"{mutant['name']}: {'attack found' if found else 'NO ATTACK FOUND'} ({mutant['why']})")
        if not found:
            failed = 1
            print(out[-3000:])
    sys.exit(failed)


if __name__ == "__main__":
    main()
