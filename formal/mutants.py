#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
# SPDX-License-Identifier: GPL-3.0-or-later

"""Mutation tests for the Tamarin models in formal/.

A proof only means something if the model would fail without the rule it
relies on. Each mutant below breaks one check in a copy of the model and
adds an attack lemma; Tamarin must find the attack trace in the mutant.
The same scenario is a proved lemma in the real model (check.sh). The
model's own lemmas are dropped from the mutant: a [reuse] helper that the
mutation makes false would otherwise be assumed and hide the attack. Every
lemma must therefore come after all rules and restrictions; a [sources]
lemma would be dropped too, which is sound but may keep a mutant search
from ending.

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
LOG = os.path.join(HERE, "hayalet-halka-gunluk.spthy")
FRESH = os.path.join(HERE, "hayalet-halka-tazelik.spthy")
SECRET = os.path.join(HERE, "hayalet-halka-gizlilik.spthy")
RECOVERY = os.path.join(HERE, "hayalet-halka-kurtarma.spthy")
TIMEOUT = int(os.environ.get("LEMMA_TIMEOUT", "600"))

BOUND_ONE_STEP = """
    & (All t #x #y. Step(t) @ x & Step(t) @ y ==> #x = #y)
    & not (Ex #x. Step('MT') @ x) & not (Ex #x. Step('AT') @ x)
    & not (Ex #x. Step('MR') @ x) & not (Ex #x. Step('AR') @ x)
    & not (Ex #x. Step('MA') @ x)"
"""

# Part 2: once a member removed n from h1, a reader still accepts a rotate
# signed by n on h1. n's IK leaks; the honest remove is in the log.
LOCKOUT_ATTACK = """
lemma attack:
  exists-trace
  "Ex c r h0 h1 n d nr #p #i #t #j.
      Pin(c, r, h0) @ p
    & Accept(c, r, h0, 'add', n, d) @ i
    & h1 = h(<'rec', r, h0, 'add', n, d>)
    & Signed(d, r, h1, 'remove', h(n)) @ t
    & Accept(c, r, h1, 'rotate', nr, n) @ j
    & #t < #j
    & (All u #x #y. Step(u) @ x & Step(u) @ y ==> #x = #y)
    & not (Ex #x. Step('MT') @ x) & not (Ex #x. Step('AR') @ x)"
"""

# Part 3: the rotate-branch attack of part 2. A contact follows a rotate by
# n, a member then removes n from that head, and n extends the contact's
# branch after the removal.
ROTATE_BRANCH_ATTACK = """
lemma attack [heuristic=c]:
  exists-trace
  "Ex c r h0 h1 h2 n d nr nr2 #p #i #k #t #j.
      Pin(c, r, h0) @ p
    & Accept(c, r, h0, 'add', n, d) @ i
    & h1 = h(<'rec', r, h0, 'add', n, d>)
    & Accept(c, r, h1, 'rotate', nr, n) @ k
    & h2 = h(<'rec', r, h1, 'rotate', nr, n>)
    & Signed(d, r, h1, 'remove', h(n)) @ t
    & Accept(c, r, h2, 'rotate', nr2, n) @ j
    & #k < #t & #t < #j
    & (All u #x #y. Step(u) @ x & Step(u) @ y & not (u = 'AT') ==> #x = #y)
    & (All #x. Step('AT') @ x ==> #x = #k | #x = #j)
    & not (Ex #x. Step('MT') @ x) & not (Ex #x. Step('AR') @ x)
    & not (Ex e #x. RevA1(e) @ x) & not (Ex e #x. RevA2(e) @ x)
    & (All e #x. RevIK(e) @ x ==> e = n)
    & (All c2 r2 h #x. Pin(c2, r2, h) @ x ==> #x = #p)
    & (All L v #x #y. Old(L, v) @ x & Old(L, v) @ y ==> #x = #y)"
"""

# Part 3, smaller: a contact follows n's rotate on the head from which a
# member removes n (the negation of no_rotate_branch_past_removal).
ROTATE_AND_REMOVE_ATTACK = """
lemma attack [heuristic=c]:
  exists-trace
  "Ex c r h0 h1 n d nr #p #i #k #t.
      Pin(c, r, h0) @ p
    & Accept(c, r, h0, 'add', n, d) @ i
    & h1 = h(<'rec', r, h0, 'add', n, d>)
    & Accept(c, r, h1, 'rotate', nr, n) @ k
    & Signed(d, r, h1, 'remove', h(n)) @ t
    & (All u #x #y. Step(u) @ x & Step(u) @ y ==> #x = #y)
    & not (Ex #x. Step('MT') @ x) & not (Ex #x. Step('AR') @ x)
    & not (Ex e #x. RevA1(e) @ x) & not (Ex e #x. RevA2(e) @ x)
    & (All e #x. RevIK(e) @ x ==> e = n)
    & (All c2 r2 h #x. Pin(c2, r2, h) @ x ==> #x = #p)
    & (All L v #x #y. Old(L, v) @ x & Old(L, v) @ y ==> #x = #y)"
"""

# Rules a mutant's attack does not need, dropped so that the search ends.
# Dropping rules only removes traces, so an attack found in the smaller
# model is an attack on the mutant too. A rule ends at the first blank
# line or the next rule, so rules must not contain blank lines.
SECRET_UNUSED = [
    "Log_Write_AK_1", "Log_Write_AK_2", "Log_Write_IK_1", "Log_Write_IK_2",
    "Reveal_IK", "Reveal_A1", "Reveal_A2", "Reveal_State", "Reveal_Ring_1",
    "Reveal_Ring_2", "Accept_Rotate_1", "Accept_Rotate_2", "Accept_Remove_1",
    "Accept_Remove_2", "Log_Age", "Member_Add_2", "Member_Remove_2",
]

RECOVERY_UNUSED = [
    "Device_Join_Request", "Member_Add_1", "Member_Add_2", "Member_Remove_1",
    "Member_Remove_2", "Member_Rotate_1", "Member_Rotate_2", "Accept_Add_1",
    "Accept_Add_2", "Accept_Remove_1", "Accept_Remove_2", "Accept_Rotate_1",
    "Accept_Rotate_2", "Log_Write_AK_1", "Log_Write_AK_2", "Log_Write_IK_1",
    "Log_Write_IK_2", "Reveal_IK", "Reveal_A1", "Reveal_A2", "Owner_Recover_2",
    "Log_Write_RK_2", "Device_Accept_Recover_2", "Contact_Accept_Recover_2",
    "Member_Reject_2",
]


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
    {
        "name": "honest_remove_not_logged",
        "why": "witnesses do not enforce inclusion: the server censors an honest remove",
        "model": LOG,
        "edits": [
            ("remove LogValid",
             r"(rule Member_Remove_[12]:(?:(?!\nrule ).)*?)LogValid\(h\(<'log', h\(hb\)>\), body, 'ak'\), ",
             r"\1", 2),
            ("remove LogEntry",
             r"(rule Member_Remove_[12]:(?:(?!\nrule ).)*?),\n      !LogEntry\([^\n]*\) \]",
             r"\1 ]", 2),
        ],
        "lemma": LOCKOUT_ATTACK,
    },
    {
        "name": "reader_accepts_rotate_despite_fork",
        "why": "a reader lets a rotate win over an AK record under the same head (fork rule 1)",
        "model": LOG,
        "edits": [
            ("Log_Read_IK looks at rotates only",
             r"(restriction Log_Read_IK:\n  \"All L v v2) k( #i #t\.[^\"]*)LogValid\(L, v2, k\)",
             r"\1\2LogValid(L, v2, 'ik')", 1),
        ],
        "lemma": LOCKOUT_ATTACK,
    },
    {
        "name": "reader_ignores_ak_fork",
        "why": "a reader does not look for a second AK record under the same head",
        "model": LOG,
        "edits": [
            ("restriction Log_Read_AK",
             r"restriction Log_Read_AK:\n  \"[^\"]*\"\n", "", 1),
        ],
        "lemma": """
lemma attack:
  exists-trace
  "Ex c1 c2 r h0 n1 n2 s #p #q #i #j.
      Pin(c1, r, h0) @ p & Pin(c2, r, h0) @ q
    & Accept(c1, r, h0, 'add', n1, s) @ i
    & Accept(c2, r, h0, 'add', n2, s) @ j
    & not (n1 = n2)
    & (All t #x #y. Step(t) @ x & Step(t) @ y & not (t = 'AA') ==> #x = #y)
    & not (Ex #x. Step('MT') @ x) & not (Ex #x. Step('AT') @ x)
    & not (Ex #x. Step('MR') @ x) & not (Ex #x. Step('AR') @ x)"
""",
    },
    {
        "name": "accept_from_network_not_log",
        "why": "a contact takes an add record from the network instead of the log",
        "model": LOG,
        "edits": [
            ("rule Accept_Add_1:",
             r"(rule Accept_Add_1:(?:(?!\nrule ).)*?)!LogEntry\(h\(<'log', hh>\), <body, sig1, sig2>\)",
             r"\1In(<body, sig1, sig2>)", 1),
        ],
        "lemma": """
lemma attack:
  exists-trace
  "Ex c r h0 n s #p #i.
      Pin(c, r, h0) @ p
    & Accept(c, r, h0, 'add', n, s) @ i
    & not (Ex k #t. LogValid(h(<'log', h0>), <'rec', r, h0, 'add', n, s>, k) @ t)""" + BOUND_ONE_STEP,
    },
    {
        "name": "rotate_followed_before_settled",
        "why": "a reader follows a rotate as soon as it is logged (part 2's rule)",
        "model": FRESH,
        "edits": [
            ("restriction Rotate_Settled",
             r"restriction Rotate_Settled:\n  \"[^\"]*\"\n", "", 1),
        ],
        "lemma": ROTATE_BRANCH_ATTACK,
    },
    {
        "name": "writer_not_fresh",
        "why": "an honest device writes from a head under which a rotate has settled",
        "model": FRESH,
        "edits": [
            ("restriction Writer_Fresh",
             r"restriction Writer_Fresh:\n  \"[^\"]*\"\n", "", 1),
        ],
        "lemma": ROTATE_AND_REMOVE_ATTACK,
    },
    {
        "name": "contact_sends_without_freshness",
        "why": "a contact sends without checking the log under its head",
        "model": SECRET,
        "edits": [
            ("restriction Contact_Fresh_Send",
             r"restriction Contact_Fresh_Send:\n(?:  [^\n]*\n)+?(?=\n)", "", 1),
        ],
        "drop": SECRET_UNUSED + ["Member_Rotate_1", "Member_Rotate_2", "Device_Offer_1",
                                 "Device_Offer_2", "Contact_Receive_1", "Contact_Receive_2"],
        "lemma": """
lemma attack:
  exists-trace
  "Ex c r hh f a b d x #t #i.
      Signed(d, r, hh, 'remove', h(x)) @ t
    & SentOffer(c, r, hh, f, a, b) @ i
    & #t < #i
    & (All u #x #y. Step(u) @ x & Step(u) @ y ==> #x = #y)
    & not (Ex #x. Step('MT') @ x) & not (Ex #x. Step('AT') @ x)
    & not (Ex #x. Step('AR') @ x)
    & not (Ex y z #x. ExtWrite(y, z) @ x)"
""",
    },
    {
        "name": "receive_without_freshness",
        "why": "a contact takes an offer without checking the log under the sender's head",
        "model": SECRET,
        "edits": [
            ("restriction Contact_Fresh_Receive",
             r"restriction Contact_Fresh_Receive:\n(?:  [^\n]*\n)+?(?=\n)", "", 1),
        ],
        "drop": SECRET_UNUSED + ["Member_Rotate_1", "Member_Rotate_2", "Contact_Send",
                                 "Contact_Send_Offer", "Device_Offer_1", "Contact_Receive_1"],
        "lemma": """
lemma attack:
  exists-trace
  "Ex c r hh g d x #t #i.
      Signed(d, r, hh, 'remove', h(x)) @ t
    & Received(c, r, hh, g, x) @ i
    & #t < #i
    & (All u #x #y. Step(u) @ x & Step(u) @ y ==> #x = #y)
    & not (Ex #x. Step('MT') @ x) & not (Ex #x. Step('AT') @ x)
    & not (Ex #x. Step('AR') @ x)
    & not (Ex y z #x. ExtWrite(y, z) @ x)"
""",
    },
    {
        "name": "rotator_follows_own_rotate_at_once",
        "why": "a device moves to the head of its own rotate before it settles",
        "model": SECRET,
        "edits": [
            ("Member_Rotate_1 view",
             r"\[ View\(d, h\(rb\), h\(hb\), d, y\), Out\(<body, sign\(<'ik'",
             "[ View(d, h(rb), h(body), d, y), Out(<body, sign(<'ik'", 1),
            ("Member_Rotate_2 view",
             r"\[ View\(d, h\(rb\), h\(hb\), y, d\), Out\(<body, sign\(<'ik'",
             "[ View(d, h(rb), h(body), y, d), Out(<body, sign(<'ik'", 1),
        ],
        "drop": SECRET_UNUSED + ["Member_Rotate_2", "Device_Offer_1", "Device_Offer_2",
                                 "Contact_Receive_1", "Contact_Receive_2"],
        "lemma": """
lemma attack [heuristic=c]:
  exists-trace
  "Ex r h1 nr s k d x #t #i.
      Signed(d, r, h(<'rec', r, h1, 'rotate', nr, s, k>), 'remove', h(x)) @ t
    & Fresh(h(<'log', h1>)) @ i
    & #t < #i
    & (All u #x #y. Step(u) @ x & Step(u) @ y ==> #x = #y)
    & not (Ex #x. Step('AT') @ x) & not (Ex #x. Step('AR') @ x)
    & not (Ex y z #x. ExtWrite(y, z) @ x)"
""",
    },
    {
        "name": "send_offer_no_recheck",
        "why": "a contact checks freshness only when it looks up the head's key, not when it sends",
        "model": SECRET,
        "edits": [
            ("Contact_Send_Offer recheck",
             r"--\[ Fresh\(h\(<'log', hh>\)\), FreshSend\(h\(<'log', hh>\)\), SentOffer",
             "--[ SentOffer", 1),
        ],
        "drop": SECRET_UNUSED + ["Member_Rotate_1", "Member_Rotate_2", "Device_Offer_1",
                                 "Device_Offer_2", "Contact_Receive_1", "Contact_Receive_2"],
        "lemma": """
lemma attack [heuristic=c]:
  exists-trace
  "Ex c r hh f a b d x #t #i.
      Signed(d, r, hh, 'remove', h(x)) @ t
    & SentOffer(c, r, hh, f, a, b) @ i
    & #t < #i
    & (All u #x #y. Step(u) @ x & Step(u) @ y ==> #x = #y)"
""",
    },
    {
        "name": "receive_signature_not_checked",
        "why": "a contact takes an offer as the ring's without checking its IK signature",
        "model": SECRET,
        "edits": [
            ("rule Contact_Receive_1:",
             r"(rule Contact_Receive_1:(?:(?!\nrule ).)*?)Eq\(verify\(sig, <'offer', r, hh, g>, sik\), true\),\n      ",
             r"\1", 1),
        ],
        "lemma": """
lemma attack:
  exists-trace
  "Ex c r hh g s #p #i.
      Pin(c, r, hh) @ p
    & Received(c, r, hh, g, s) @ i
    & not (Ex a b e f #o. OfferedBy(a, b, e, f) @ o)
    & not (Ex e #x. RevIK(e) @ x)
    & not (Ex u #x. Step(u) @ x)
    & not (Ex y z #x. ExtWrite(y, z) @ x)"
""",
    },
    {
        "name": "recovery_without_wait",
        "why": "a device follows a recovery before it has settled",
        "model": RECOVERY,
        "edits": [
            ("Device_Accept_Recover_1 wait",
             r"(rule Device_Accept_Recover_1:(?:(?!\nrule ).)*?)Settled\(h\(<'rlog', m>\), <body, sig>\), ",
             r"\1", 1),
        ],
        "drop": RECOVERY_UNUSED,
        "lemma": """
lemma attack:
  exists-trace
  "Ex o r m n #k.
      Accept(o, r, m, 'recover', n, 'rk') @ k
    & not (Ex v #x. Old(h(<'rlog', m>), v) @ x & #x < #k)
    & (All u #x #y. Step(u) @ x & Step(u) @ y ==> #x = #y)"
""",
    },
    {
        "name": "recovery_beats_ak_record",
        "why": "a reader follows a recovery although a member wrote an AK record under the same key",
        "model": RECOVERY,
        "edits": [
            ("restriction Rec_Priority",
             r"restriction Rec_Priority:\n(?:  [^\n]*\n)+?(?=\n)", "", 1),
        ],
        "drop": RECOVERY_UNUSED,
        "lemma": """
lemma attack [heuristic=c]:
  exists-trace
  "Ex d r m o n #t #k.
      HonestAK(d, r, m) @ t
    & Accept(o, r, m, 'recover', n, 'rk') @ k
    & #t < #k
    & (All u #x #y. Step(u) @ x & Step(u) @ y ==> #x = #y)"
""",
    },
    {
        "name": "contact_skips_words",
        "why": "a contact follows a recovery without comparing the words again",
        "model": RECOVERY,
        "edits": [
            ("Contact_Accept_Recover_1 words",
             r"(rule Contact_Accept_Recover_1:(?:(?!\nrule ).)*?)!Words\(r, h\(body\)\), ",
             r"\1", 1),
        ],
        "drop": RECOVERY_UNUSED,
        "lemma": """
lemma attack:
  exists-trace
  "Ex c r n #k.
      ContactRecovered(c, r, n) @ k
    & not (Ex #o. OwnerRecovered(r, n) @ o)
    & (All u #x #y. Step(u) @ x & Step(u) @ y ==> #x = #y)"
""",
    },
    {
        "name": "recoveries_not_a_fork",
        "why": "readers follow one of two recoveries against the same period without stopping",
        "model": RECOVERY,
        "edits": [
            ("restriction Rec_Single",
             r"restriction Rec_Single:\n(?:  [^\n]*\n)+?(?=\n)", "", 1),
        ],
        "drop": RECOVERY_UNUSED,
        "lemma": """
lemma attack:
  exists-trace
  "Ex r m v v2 #i #j.
      ReadRec(r, m, v) @ i
    & ReadRec(r, m, v2) @ j
    & not (v = v2)
    & (All u #x #y. Step(u) @ x & Step(u) @ y ==> #x = #y)"
""",
    },
]


def mutate(text, mutant):
    for label, pattern, replacement, expected in mutant["edits"]:
        text, count = re.subn(pattern, replacement, text, flags=re.S)
        if count != expected:
            sys.exit(f"{mutant['name']}: edit '{label}' applied {count} times, "
                     f"expected {expected}; update mutants.py")
    for rule in mutant.get("drop", []):
        text, count = re.subn(r"\nrule %s:\n.*?(?=\n\n|\nrule )" % re.escape(rule), "", text, flags=re.S)
        if count != 1:
            sys.exit(f"{mutant['name']}: rule {rule} found {count} times; update mutants.py")
    first = re.search(r"^lemma ", text, re.M)
    cut = first.start() - 1 if first else -1
    if cut < 0 or "\nrestriction " in text[cut:]:
        sys.exit(f"{mutant['name']}: expected all lemmas after the rules and restrictions")
    return text[:cut] + "\n" + mutant["lemma"] + "\nend\n"


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
