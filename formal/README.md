# Formal model (Tamarin)

This folder holds the Tamarin model of the account design
([hesap.md](../docs/hesap.md),
[decision 0013](../docs/kararlar/0013-hayalet-halka-hesap.md)) (#251).
Before any code is written, the design's security claims are proven or
refuted here as lemmas.

The model grows piece by piece. Each piece is a separate PR:

| Piece | Contents | Status |
| --- | --- | --- |
| A1 | Membership chain: `create`, `add`, `remove`, `rotate` | In this folder |
| A2 | Multi-valued transparency log, witnesses, enforced inclusion (P2) | In this folder |
| A3 | Settling of a rotate and the writer's freshness (the fix for the gap found in A2); ring secret, offers, freshness when sending and when receiving (P3) | In this folder |
| A4 | Recovery and the recovery part of P7 (A4a); strict mode (A4b), `resolve_fork` (A4c), writing the design rules into `hesap.md` (A4d) | A4a and A4b in this folder, the rest next |
| A5 | TLA+ evaluation of the waiting times | Next |

## Running it

It needs Tamarin 1.12.0 and Maude 3.5.1. CI downloads both at a fixed
version and checks their SHA-256 (`.github/workflows/formal.yml`).

```sh
formal/check.sh          # proves each lemma separately
formal/mutants.py        # checks that the attack is found in broken models
```

Both commands end with a non-zero exit code if a lemma is not proven or an
attack is not found. The time limit per lemma is set with `LEMMA_TIMEOUT`
(seconds, default 600).

## A1: membership chain (`hayalet-halka-zincir.spthy`)

### What the model contains

- Device keys: `IK` and the two halves of the management key `AK`
  (Ed25519, ML-DSA-65).
- Records: `create`, `add`, `remove`, `rotate`. Every record carries the
  ring identity, the digest of the previous head, the operation, the
  payload and the signer.
- Every device and every contact keeps its own view of the chain. Views
  move forward by checking the records that come from the network with the
  same rules.
- The attacker is the whole network (Dolev-Yao). A malicious mailbox
  server is therefore not modelled separately: it reads, drops, replays
  and forges every message.
- Every device key can leak at any time. A removed or stolen device is a
  device whose keys have leaked. The two halves of `AK` leak separately;
  this also models the break of a single algorithm.

### Abstractions and why

In some places the model is simpler than the design, in others stronger.
The places where it is stronger say what the proof does not show for the
real system; later pieces close them.

| Abstraction | Why | When it widens |
| --- | --- | --- |
| The ring has two device slots | Multiset membership did not let the Tamarin search end; two slots cover a membership change and the replay of an old record | A4: resolving a fork may need three slots |
| Each device is in one ring | A device creates or joins only one ring. So the "in the same ring" part of P1 holds by itself in this model; that the record carries the ring identity is not tested here | A3: a two-ring scenario, together with the contact's head |
| Ring identity `h('ring', first device)` | In the design `H("ring", create)`; both are bound to the first record and never change, which makes no difference in this piece | A3: together with contact pinning |
| The hybrid `AK` signature is two separate signatures, each signing the same body with its own label (`'ed'`, `'ml'`) | Because the body names the signer together with its two public keys, each half is bound to the other's key (hesap.md, "Keys") | Does not change |
| `IK` is a single key | In the design `IK` is also Ed25519 + ML-DSA-65; the split into two halves for `rotate` is not tested in this piece | A3 |
| A `remove` record names the removed device by the digest of its keys | The real record names the device by `IK`; the digest keeps Tamarin from needlessly analysing the attacker's knowledge | Does not change |
| Records carry no ring secret (`r_e`) | Confidentiality (P3) is the subject of A3 | A3 |
| **Stronger:** the contact pins the chain from the first record (`create`) over a verified channel | In the design the contact pins the head current at first pairing; the verification words verify only the ring identity, that is the first record. A head pinned by a contact that pairs after a membership change is not covered by this model | A3 |
| **Stronger:** the contact sees records in the clear, knows the members' public keys and computes the new member set itself | In the design the contact sees only the head (`C_e`, operation type, digest of the previous head), the `AK` signatures and the Merkle proofs; it cannot check exactly which device a `membership` head signed by a member adds. The contact proofs here are therefore stronger than what a real contact can check | A3: the contact is modelled with a head that sees the Merkle root as an opaque digest |
| Some lemmas are bounded by a number of steps (`Step`) | So the search ends; the bound is written in the lemma's text | As needed |

### Lemmas

| Lemma | Kind | What it says |
| --- | --- | --- |
| `sanity_contact_follows_add` | exists-trace | A contact can accept an `add` record (the model works) |
| `sanity_contact_follows_add_remove` | exists-trace | A contact can accept an `add` and then the `remove` of the same member |
| `sanity_contact_follows_remove_by_slot_2` | exists-trace | The member in the second slot can remove the first (the mirrored rules work) |
| `compromised_member_can_add` | exists-trace | If both `AK` halves of a member leak, the attacker can add a device. Without this trace the lemma below could hold for the wrong reason |
| `membership_change_needs_member_ak` | all-traces | **P1:** every accepted `add` and `remove` was signed, on the same previous head, by the named member. Exceptions: both `AK` halves of that member leaked earlier, or the signer is a key the attacker generated. One half leaking is not enough; `IK` is never enough |
| `signer_must_be_member` | all-traces, bounded | If no `AK` has leaked, a contact does not accept, as the `add` right after the first record, one signed with a key the attacker generated. It shows the last exception of P1 only for this single step; the form after honest steps and the unbounded form do not exist yet |
| `rotate_needs_member_ik` | all-traces | Every accepted `rotate` was signed on the same head with the named member's `IK` (if `IK` has not leaked). `rotate` cannot change the member set; this holds structurally in the acceptance rules |

Bringing back a removed device by replaying an old `add` record is ruled
out by P1 when no key has leaked: the record does not name the new head,
so it is not accepted. The `add_does_not_bind_previous_head` mutant shows
that the attack is found if the record does not name the head.

### Mutant checks (`mutants.py`)

A proof says something only if the model fails without the rule it rests
on. Each mutant breaks a single check in a copy of the model and expects
Tamarin to find an attack trace. The number of places each edit applies to
is written down; if the number does not match, the script stops.

| Mutant | Broken check | Expected attack |
| --- | --- | --- |
| `accept_checks_ed_half_only` | The contact checks only the Ed25519 half of the `AK` signature | A forged `add` is accepted once only that half leaks |
| `accept_checks_ml_half_only` | The contact checks only the ML-DSA-65 half | A forged `add` is accepted once only that half leaks |
| `accept_signer_not_member` | The contact takes the signer's keys from the record, not from its own view | An `add` is accepted with the attacker's key, with no key leaked |
| `add_does_not_bind_previous_head` | The `add` record does not name the head it extends (in all four rules) | A removed device comes back by replaying an old `add` |
| `remove_signature_not_checked` | The contact does not check the `remove` signature | A forged `remove` is accepted with no key leaked |
| `rotate_signature_not_checked` | The contact does not check the `rotate` signature | A forged `rotate` is accepted with no key leaked |

### Not proven yet

- **The attacker's key cannot become a member, unbounded:** the lemma "the
  signer is a device, or both `AK` halves of a device have leaked" did not
  end in Tamarin by induction; it needs a helper invariant. For now its
  bounded form is `signer_must_be_member`.
- **A removed device whose keys leak:** that, if a device's keys leak
  after it is removed, the contact does not accept it as the signer of any
  record unless it is added again. This is expected, because the contact's view
  no longer contains the removed device, but the lemma that shows it (even
  in bounded form) did not end in Tamarin. It needs an invariant showing
  that an owner cannot have more than one view. This is the core of the
  design's "old authority" threat (H3). A2 proves the part of it at the
  head the device was removed from with the log
  (`removed_device_locked_out_of_old_head`); later heads are still open
  (below, A2).
- Forks, the log, freshness, recovery and strict mode: A2-A4.

## A2: transparency log (`hayalet-halka-gunluk.spthy`)

A1's chain, keys and attacker stay as they are. What changes is where the
records are read from.

### What the model contains

- Records are read from the log, not from the network. The log key is
  bound to the head being extended: `L = h('log', head)`. More than one
  value can stand under a key, and none is ever deleted.
- Anyone can write to the log (in reality with an ARC token). The log
  keeps only values signed by a member of that head; because readers
  ignore invalid values anyway, writing another value changes nothing.
- Honest devices write their own records to the log in the same step in
  which they write them. This is the witnesses' enforced inclusion: the
  server cannot censor an honest record. The writer writes even if there
  is another value under the key; the fork then stops the readers.
- A reader (device or contact) applies the fork rule. An `AK` record is
  accepted if no other valid `AK` record was written under the same key
  before; it wins over a `rotate` (fork rule 1). A `rotate` is accepted if
  no other valid value of any kind was written under the same key before.
  A reader never leaves a branch it has followed (fork rule 4).
- The attacker is still the whole network, and every key can leak at any
  time.

### Abstractions and why

| Abstraction | Why | When it widens |
| --- | --- | --- |
| No time: an honest record is in the log the moment it is written | The real bound is 1 hour of inclusion + 1 hour of root freshness (P3: 2 hours). Only the order is tested here | A3 (freshness), A5 (durations) |
| The log is a single global set | It stands in for the witness assumption. With n witnesses, k signatures per root and f witnesses colluding with the server: no censorship needs `f < k`, a single view needs `f < 2k − n`. An honest majority alone is not enough (n = 5, k = 3, f = 2: the two bad witnesses sign both roots, and one honest witness signs each). If these conditions do not hold, there is no P2 and P3 assurance | Does not change; the threshold is to be corrected in the design (below) |
| **Stronger:** a reader sees every value written before it reads | Perfect freshness. In reality a contact sees a root at most 1 hour old | A3 |
| Values are not encrypted | The server not being able to read the content is P5, not P2 | A3 |
| `create` is not written to the log | The contact pins the first record over a verified channel as in A1 (the key `L_0` is not used here) | A3 |
| **Stricter:** every value a member signed counts as a fork, even if its operation makes no sense | The design ignores invalid values. This only makes the reader stop more often; it does not make the security lemmas easier | A4 |
| **Stricter:** `rotate`s that fork with each other do not merge by the smallest digest; the reader stops | Merging a harmless fork (fork rule 2) affects usability, not security; merging is modelled in A4 together with `resolve_fork` | A4 |

### Lemmas

| Lemma | Kind | What it says |
| --- | --- | --- |
| `sanity_contact_follows_add_via_log` | exists-trace | A contact can accept an `add` from the log |
| `sanity_leaked_device_rotates_before_removal` | exists-trace | The `rotate` of a device whose `IK` leaked can be accepted at that head before the removal is written to the log |
| `removed_device_survives_on_rotate_branch` | exists-trace | **The gap the model found** (below): a removed device still gets records accepted after the removal by a contact that follows its own `rotate` branch |
| `accepted_records_are_logged` | all-traces | **P2, visibility:** every record any reader accepts is in the log as a valid value before it is accepted. It holds structurally because the acceptance rules read from the log; the `accept_from_network_not_log` mutant shows why this matters |
| `ak_read_was_logged`, `ak_reads_agree` | all-traces, helper | An `AK` read reads a value in the log; two `AK` reads under the same key read the same value |
| `no_split_view_membership` | all-traces | **P2, a single view of membership changes:** two readers do not accept two different `add`s or `remove`s of the same head, whichever key leaks. A split between a `rotate` and an `AK` record is outside this lemma (the gap below) |
| `removed_device_locked_out_of_old_head` | all-traces | **H3, at the head it was removed from:** once a member has removed `x` from a head, no reader accepts a record signed by `x` at that head, even if all of `x`'s keys have leaked, the server helps, and `x` wrote its own record to that head first (a race). The only record that can be accepted is the removal itself |

### Mutant checks

The mutants do not give the broken model the model's own lemmas: assuming
a `[reuse]` helper that the breakage makes false would hide the attack.

| Mutant | Broken check | Expected attack |
| --- | --- | --- |
| `honest_remove_not_logged` | The witnesses do not enforce inclusion; the server censors the honest `remove` | The removed device's `rotate` is accepted at the old head |
| `reader_accepts_rotate_despite_fork` | For a `rotate`, the reader looks only at other `rotate`s; the `AK` record has no priority (fork rule 1) | The same attack, even with the `remove` in the log |
| `reader_ignores_ak_fork` | The reader does not look for a second `AK` record under the same key | Two contacts accept two different `add`s of the same head (split view) |
| `accept_from_network_not_log` | The contact takes the `add` record from the network, not from the log | A record that is not in the log is accepted |

### The gap the model found: a contact left on the `rotate` branch

Fork rule 1 says "a `rotate` written at the same moment by a device about to
be removed cannot block the removal". But by rule 4 an observer ignores
the forks below the head it has pinned. Together they give this trace
(`removed_device_survives_on_rotate_branch`):

1. `d` adds `n`; contact `c` accepts this `add`.
2. `n`'s `IK` leaks. The attacker writes a `rotate` in `n`'s name; `c`
   accepts it and moves to the new head.
3. `d` removes `n` from the old head. For everyone who reads that head from
   then on, the `remove` wins, but `c` never looks at that head again.
4. The attacker keeps extending `c`'s branch in `n`'s name. If `n`'s `AK`
   has leaked too, it can even remove `d`.

So a removed device stays the ring's only authority for the contacts it
moved to its own branch before the removal. This is exactly the subject of
H3. The ring (`d`) sees the foreign `rotate` under its own key and warns,
but a rule is needed on the contact's side. In this model (A2) the claim
"a removed device is locked out on later heads as well" is **false**, not
only unproven. The fix is in A3 (below).

### Feedback to the design (to be written into `hesap.md` in A4)

- The conflict between fork rules 1 and 4 (the gap above) and the fixing
  rule in A3 (settling time and the writer's freshness).
- Binding an `AK` record to the witnessed root it was checked against, and
  treating a record that enters a root late as invalid (A3, "The
  assumption the model rests on").
- The witness threshold: `hesap.md` says "k > n/2". For a single view, `k`
  must be chosen together with the number `f` of witnesses that may
  collude so that `2k − n > f` (e.g. with n = 5 and k = 4, at most f = 2
  bad witnesses are tolerated; with k = 3, only f = 0).

## A3: settling of a rotate (`hayalet-halka-tazelik.spthy`)

A2's model stays as it is; two rules are added that close the gap A2
found. There is still no clock: "the time has passed" is an event (`Old`).

### Rules added

- **Settling:** a reader (device or contact) follows a `rotate` only after
  the record has stood in the log for the settling time `D`.
- **The writer's freshness:** an honest device writes a record only from a
  head under whose key no value has settled yet. The real rule: the
  writer's fresh root must not show a value under that head's key older
  than `D_w`. A newer value (e.g. a `rotate` that has not settled yet)
  does **not** block writing: the `AK` record is written and wins by fork
  rule 1. If this were read as "there must be no value under the key", a
  device whose `IK` leaked could postpone its own removal for ever by
  writing a `rotate` every `D` (with one ARC token each).

Together they ensure this: a `remove` written while a `rotate` is still
settling is in the log before the `rotate` settles and beats it by fork
rule 1 (`sanity_remove_beats_unsettled_rotate`). Once the `rotate` has
settled, an honest device can no longer write from that old head; it first
follows the `rotate` and writes the removal from the new head.

### Working out the durations (to be settled in A5)

The durations are measured not from the moment of writing, which the
attacker chooses, but from the moment the `rotate` **first enters** a
witnessed root (`T_entry`). The writer's root is at most 1 hour old and
does not show the `rotate` as older than `D_w`; so the writer's record is
written before `T_entry + D_w + 1 hour` and enters a root at most 1 hour
later, that is before `T_entry + D_w + 2 hours`. If the reader measures the
age with its own root's timestamp, `D > D_w + 2 hours + witness clock
skew` is enough; if it measures with its local clock, 1 more hour is added
for the age of the root. With `D_w = 0` (the writer's root does not show
the value at all) and a clock skew of a few minutes, `D = 4 hours` is
safe.

### The assumption the model rests on: a record enters the log on time

The model puts an honest record into the log in the step in which it is
written. In A2 this was only about order; here it carries weight, because
the writer's freshness check must still hold when the record **enters a
root**. In reality the witnesses count the 1-hour bound from the moment
they **receive** the record. If an attacker on the network drops the
upload of an honest `remove` and writes it to the log after the `rotate`
has settled, the readers that followed the `rotate` stay on it and those
who come later follow the `remove`: a split view
(`no_split_view_membership` does not catch this, because it compares only
two membership changes). The design must prevent this; the proposal to
write into `hesap.md` in A4:

- An `AK` record carries the witnessed root it was checked against (its
  digest or timestamp). Readers and witnesses treat a record that enters a
  root later than `1 hour + clock skew` after that root as invalid.
- If the record does not enter a root in time, the writer checks freshness
  again and signs the record again.

### Effect on honest rotation

The rotating device counts its own `rotate` as pending for `D` and keeps
using its old `IK`. If an `AK` record is written to the same head in this
time, the `rotate` drops (rule 1) and the device rotates again from the
new head. For the weekly rotation this has no effect on the user. For a
device whose `IK` is suspected to have leaked, rotation takes up to 4
hours; in that case the right step is removal.

### Lemmas

A2's all-traces lemmas hold in this model too and are not proven again:
the model only adds a rule with no consequences (`Log_Age`), new events
and restrictions. Removing the `Log_Age` steps from a trace of this model
leaves a trace of A2, and none of A2's lemmas or restrictions mentions
`Old`, `WriteFresh` or `Settled`. This reasoning relies on the two files
staying the same: if a rule changes in A2, the same change must be made in
A3 (generating A3 from A2 with a script would later make sure of this).
(Proving the all-traces lemmas again here did not finish in 10 minutes,
because the new restrictions make the search larger.) A2's two
exists-trace lemmas are proven again here, because the restrictions could
remove a trace. The new ones:

| Lemma | Kind | What it says |
| --- | --- | --- |
| `sanity_contact_follows_settled_rotate` | exists-trace | A settled `rotate` can still be followed; the rule does not block rotation |
| `sanity_remove_beats_unsettled_rotate` | exists-trace | While the `rotate` of a device whose `IK` leaked is still settling, a `remove` written from the same head goes through and the contact follows it; a pending `rotate` cannot delay a removal |
| `no_rotate_branch_past_removal` | all-traces | **H3, the gap from A2 closed:** if a member removes `x` from a head, no reader follows `x`'s `rotate` at that head, whichever was written first. It covers only the head the removal was made at; this is enough to block the second step of the attack trace in A2, and the general form is open (below) |

### Mutant checks

| Mutant | Broken check | Expected attack |
| --- | --- | --- |
| `rotate_followed_before_settled` | The reader follows a `rotate` as soon as it enters the log (A2's rule) | The `rotate` branch attack from A2 |
| `writer_not_fresh` | An honest device writes from an old head under which a `rotate` has settled | The contact follows `n`'s `rotate` at the head where a member removed `n` |

### Not proven yet

- **The unbounded form:** the lemma "after the removal, a reader accepts
  only those records of `x` that entered the log before the removal (or
  `x` was added again)" did not end by induction in 450 seconds. The known
  attack (the `rotate` branch) is closed; the general form needs a helper
  invariant.

## A3b: what a removed device cannot do, P3 (`hayalet-halka-gizlilik.spthy`)

A3's chain, log, settling and writer's freshness stay as they are. Added:

- Every record carries the ring KEM key of the epoch it starts
  (`pk(r_e)`); the writer gives `r_e` to the members of the new head (in
  reality with HPKE to the members' `KK` keys).
- A device whose state leaks gives the attacker the `r_e` of every head it
  is a member of. A removed or stolen device is such a device.
- The contact encrypts an offer to its head's KEM key. A device in the
  ring sends the contact an offer on behalf of the ring, signed with its
  `IK`.
- Before sending and before receiving an offer, the contact does a
  freshness check: under the key of the head it uses there is neither an
  `AK` record nor a settled value. A `rotate` that has not settled yet
  does not block sending; the old head is still current then. This is
  looser than the rule in `hesap.md` (there every valid value under the
  key stops sending); the looser rule will be written into `hesap.md` in
  A4, because with the strict rule contacts could not send for `D` after
  every weekly `rotate`.
- The rotating device follows its own `rotate`, like the other readers,
  only once it has settled (A3, "Effect on honest rotation"). Until then it
  writes `AK` records from the old head; there they win over the `rotate`
  (rule 1).

Because the record body changed (the KEM key was added), this is a new
model: the lemmas of A1-A3 are not carried over here, and P1 and P2 are
not claimed here.

### Abstractions

| Abstraction | Why | When it widens |
| --- | --- | --- |
| `r_e` is given to the members with a secure fact | HPKE and key confirmation (MAC) are tested not here but in the protocol notes and the test vectors | Stage C |
| Offer confidentiality is expressed as the confidentiality of the ring key | Encrypting the offer to the KEM key is standard public-key encryption; adding this step as a lemma did not let the search end. Proven: the key the contact encrypts to is known only if a device that is a member of that head leaks, or if only the attacker wrote the head | Does not change |
| Perfect freshness: the contact sees every value written before it sends | The 2-hour bound in P3 (1 hour of inclusion + 1 hour of root freshness) comes in A5 | A5 |
| The offer signature binds the ring, the sender's head and the offer identity; it does not bind the receiving contact or the receiver's head | Resending an offer to another contact is not ruled out here; the echo of the receiver's head in `hesap.md` "Sending" and the binding to the contact will be settled in the protocol notes (stage C) | Stage C |
| The contact's freshness check and the sending of the offer are two steps | A single step did not let the search end; the check is done again in the sending step, so the lemmas hold for the moment of sending | Does not change |
| No handshake between the contact and the ring (Noise + ML-KEM) and no contact key `k_AB` | The handshake is tested in the existing protocol; renewing `k_AB` will be handled together with the contact book | A4 or a separate piece |

### Lemmas

| Lemma | Kind | What it says |
| --- | --- | --- |
| `ring_secret_secret`, `kem_origin` | all-traces, helper | An `r_e` is known only through a leaked member of that head; a head's KEM key is either a member's `pk(r_e)` or was written to the log from the network |
| `at_rotate_head_settled` | all-traces, helper (induction) | Before anyone is at the head a `rotate` starts, a value has settled under the key of the previous head (this is enough for the freshness restrictions) |
| `sanity_contact_sends`, `sanity_contact_receives`, `sanity_contact_sends_after_removal`, `sanity_remove_from_rotate_head` | exists-trace | A contact can send, can receive an offer from a member, and can send to the new head after following a removal; a removal can be written from the head of a `rotate` (the lemma below is not vacuous) |
| `sent_head_key_secret` | all-traces | **P3, confidentiality:** the key the contact encrypts an offer to is known only if a member of that head leaked its state, or if no member wrote the head. The second case is a head written to the log from the network that the contact follows; for the contact to follow it, a member's `IK` (`rotate`) or both halves of its `AK` must leak. This is not proven separately (below) |
| `no_offer_to_head_after_removal` | all-traces | **P3, freshness when sending:** once a member has removed `x` from a head, no contact sends an offer to that head |
| `no_offer_from_removed_device` | all-traces | **P3, freshness when receiving:** once a member has removed `x` from a head, no contact accepts an offer from `x` at that head, even if all of `x`'s keys have leaked |
| `no_fresh_check_before_rotate_after_removal` | all-traces | **P3, the head before an unsettled `rotate`:** if the removal was written from the head a `rotate` started, no contact's freshness check passes any more at that `rotate`'s previous head, neither for sending nor for receiving |
| `no_rotate_branch_past_removal`, `no_rotate_branch_of_remover` | all-traces | A3's lemma proven again in this model: no reader follows the `rotate` signed by `x` at the head `x` was removed from; nor does it follow the remover's own `rotate` |
| `received_offer_authentic` | all-traces | An offer the contact accepts as coming from ring member `s` is `s`'s, if `s`'s `IK` has not leaked and `s` is not the attacker's key. The attacker's key being a member is the subject of P1 (A1: both halves of `AK` leaking) |

The three freshness lemmas follow quickly (4-12 steps) from the contact's
check (a restriction) and from the abstraction that honest records enter
the log in the step they are written. What they state is real, but it is
not the whole of P3: which heads are covered is above, those not covered
are below.

### The gap the model found: the previous head when a removal is written from a `rotate`'s head

In the first version of PR #282 the rotating device moved to its own
`rotate`'s head at once. The independent review found this trace: `d`
creates the ring and adds `x`; the contact is at head `{d, x}`. `d` writes
its weekly `rotate` and moves to the new head; the `rotate` has not settled
yet. `d` then removes `x` from the new head. The contact is still at the
old head, and there is only an unsettled `rotate` there: the check passes.
The contact encrypts to a head of which `x` is a member, and `x` (with no
key leaked, just by running the software) can send offers at the old head.
The window lasts until the `rotate` settles, about 4 hours; it exceeds the
2 hours of P3.

The fix is the rule A3 already states: the rotating device follows its own
`rotate` once it has settled; until then it writes the removal from the
old head, and the removal wins over the `rotate`. After the `rotate` has
settled, the removal is written from the new head, but then there is a
settled value at the old head and contacts do not send from there.
`no_fresh_check_before_rotate_after_removal` proves this; the
`rotator_follows_own_rotate_at_once` mutant finds the trace again.

### Mutant checks

| Mutant | Broken check | Expected attack |
| --- | --- | --- |
| `contact_sends_without_freshness` | The contact does not look at the log before sending | The contact sends to the head `x` was removed from (receiving files with an old head, a freeze attack) |
| `receive_without_freshness` | The contact does not look at the log before receiving an offer | The contact accepts an offer from `x` at the head `x` was removed from |
| `rotator_follows_own_rotate_at_once` | The rotating device moves to its own `rotate`'s head at once | The removal is written from the new head, and the contact passes the check at the old head (the gap above) |
| `send_offer_no_recheck` | The contact checks freshness only when it looks up the head's key, not when sending | The check passes, then the removal is written, then the offer goes to a head of which `x` is a member |
| `receive_signature_not_checked` | The contact does not check the offer's `IK` signature | With no key leaked, a forged offer counts as coming from a member |

Four of the A3b mutants (all but `receive_signature_not_checked`) also drop
rules the attack does not use, so the search ends (`mutants.py`, `drop`).
Dropping rules only removes traces; an attack found in the smaller model
exists in the mutant too.

### Not proven yet

- **Later heads:** P3's freshness lemmas cover the head the removal was
  made at, the previous head of a `rotate` if the removal was written from
  that `rotate`'s head, and the `rotate` branch in A3. The unbounded lemma
  "no head a contact sends to after the removal contains `x`" rests on
  A3's unbounded lock-out lemma and does not exist yet.
- **A device whose `AK` leaked before the removal:** such a device can,
  before the removal, write its own branch (e.g. a record that removes the
  other member) and move contacts there. This is the "late fork, pair
  again" limit the design accepts (`hesap.md`, "Rules to settle in the
  formal model", item 1) and is handled in A4.
- **Heads two or more steps back:** e.g. `rotate`, `rotate`, then the
  removal. The reasoning above holds as it is (every previous head has an
  `AK` record or a settled value), and these cases were traced by hand in
  the second review, but the lemma did not finish in 4 GB of memory. A
  general helper "whoever is at the child head was at the parent head
  before" is needed.
- **A third member's `rotate`:** the model uses two device slots; the
  rotating device is either `x` or the remover. The reasoning carries over
  to a third member as it is, but it is not expressed in the model.
- **A3's model with the old rule:** in `hayalet-halka-tazelik.spthy` the
  rotating device still moves to its own `rotate`'s head at once; A3's
  lemmas were proven with that rule. Because the new rule changes the
  device's behaviour (it keeps writing from the old head), A3's results
  will be proven again with the new rule in A4.
- **Heads the attacker wrote:** for the second case of
  `sent_head_key_secret` (a head no member wrote), the strong lemma "for
  the contact to follow this head, a member's key must leak" did not
  finish in 8 GB of memory; it needs a helper invariant.
- **Confidentiality of the offer itself:** what is proven is the
  confidentiality of the key the contact encrypts to; the confidentiality
  of the offer comes from standard public-key encryption and is not a
  separate lemma.

### Feedback to the design (to be written into `hesap.md` in A4)

- **Freshness rule:** `hesap.md` stops sending on every valid value under
  the key. The model uses the looser rule: an `AK` record or a settled
  value stops it, an unsettled `rotate` does not. Together with the next
  item the looser rule is safe, and it keeps contacts from being unable to
  send for `D` after every weekly `rotate`.
- **The rotating device:** follows its own `rotate` only once it has
  settled; until then it writes `AK` records from the old head. No device
  of the ring follows a `rotate` before it has settled.
- **"Settled" must mean the same for everyone:** in the model settling is
  a single event; in reality every device and contact decides it for
  itself. If a contact starts counting from the moment it first sees the
  `rotate` (e.g. a contact that was offline for 10 hours), it encrypts for
  `D` to the older head, of which `x` is a member, while the removing
  device has long since removed `x` from the new head. The rule: everyone
  measures settling from the witnessed inclusion time, with the same `D`,
  against the present time; a device follows a `rotate` only after the
  moment by which every contact must count it as settled (margin: clock
  skew and root age). It will be settled numerically in A5.
- **Offer signature:** it should bind the receiving contact (or the
  context shared with the contact) and the receiver's head; `hesap.md`
  should state plainly what is bound.

## A4a: recovery (`hayalet-halka-kurtarma.spthy`)

A3's chain, log, settling and writer's freshness, with A3b's `rotate` rule
(the rotating device follows its own `rotate` once it has settled). Added:

- The ring's first record carries the public half of the recovery key
  (`RK`). The secret half is the 24-word phrase; the owner holds it, and it
  is on no device. `Reveal_Phrase` is the theft of the phrase.
- **Membership period:** every view carries the head of the last
  membership record (create, add, remove, reject, recovery) as `m`.
  `rotate` does not change `m`.
- `recovery_add`: an `RK`-signed proposal that adds a device into an empty
  slot; it is written not against a head but against the membership
  period `m`, and enters the log under its own key `h('rlog', m)`. So a
  pending recovery carries over to the following `rotate`s ("a recovery
  record does not open a fork ... carried over to the following epochs")
  and `rotate` does not stop it. The owner writes it with the phrase and a
  new device (`Owner_Recover`); someone who stole the phrase writes it
  from the network (`Log_Write_RK`).
- **Wait:** readers (devices and contacts) follow a recovery only once it
  has settled. In reality this is 72 hours or 7 days; in the model it is
  A3's "settled" event.
- **Priority:** if an `AK` record was written earlier from a head of
  period `m`, the recovery is not followed (the fork rule, an extension of
  rule 1).
- **Two recoveries are a real fork:** if another recovery against `m`
  entered the log first, the reader follows neither.
- `recovery_reject`: an `AK` record a member writes from its current head.
  Unlike the other records it does not depend on the writer's freshness: a
  reject always counts (`hesap.md`, fork rule 5). The rule's only
  precondition is the member's view, so the member can write it at any
  moment of the wait. With the priority rule it stops the recovery against
  `m`.
- **Contacts** follow a recovery only after comparing the words again with
  the owner, for that recovery record (`!Words(r, record)`).

Because the record body and the view changed, this is a new model; A3's
executability traces and `no_rotate_branch_past_removal` were proven again
here.

### Gaps the review found in the model's first version

In the first version of PR #283 the recovery was written under a head's
key. The independent review found two traces:

- **A late recovery that cannot be rejected:** `d` writes its weekly
  `rotate`, the `rotate` settles but `d` has not followed it yet. Someone
  who stole the phrase writes a recovery under the same key. The writer's
  freshness forbade `d` from writing (rejecting) under that key any more;
  the value before the recovery also blocked `d`'s own `rotate`. `d`'s
  only way was to follow the thief's recovery. This contradicted the rule
  "the device branch stays valid" in `hesap.md`.
- **Two recoveries split the readers:** when the owner and the thief wrote
  two separate recoveries under the same key, one device followed one and
  one contact the other.

The fix is to bind the recovery to the membership period and to count two
recoveries as a fork. `sanity_reject_after_settled_rotate` shows that the
first trace can now be rejected, `no_split_recovery` that the second is
closed.

The second review showed that the same gap came back by another route: if
a single member's own two `rotate`s forked and settled (or someone who
stole its `IK` wrote two values to the head's key), the member could
neither follow a `rotate` nor write from that head (the writer's
freshness). Its only move was to follow the thief's recovery. The fix: a
reject does not depend on the writer's freshness.

### Abstractions

| Abstraction | Why | When it widens |
| --- | --- | --- |
| A recovery adds only into an empty slot | Two device slots; if all devices are lost, one slot in the ring is empty | Does not change |
| The recovered device writes no new records in this piece, and readers do not follow a reject and move to a new head | Not needed for the recovery part of P7; so the service continuing after a reject (liveness) is not expressed here | A4c |
| A rejected `RK` does not stay invalid in later periods | Here a reject stops the recovery only against the period the reject was written in | A4c (with `change_phrase`) |
| The owner writes the recovery against the period of any head it learned from the log | In reality it moves from the monthly recovery record to the newest head; because the attacker can do every such write too (`Log_Write_RK`), it does not affect the proofs | Stage C |
| The wait is A3's settling event | Durations in A5 | A5 |
| **The contact's identity check is a trusted channel** (`!Words`) | The words prove only the channel to the device that shows them; the thief's device can show words too. The real assumption is that the contact recognises the other person out of band (face to face, by voice): an assumption about people, open to social engineering. The contacts' half of P7 rests entirely on it; under censorship it is the only protection | Does not change; it is explained to the user |

### Lemmas

| Lemma | Kind | What it says |
| --- | --- | --- |
| A3's four executability traces, `no_rotate_branch_past_removal` | exists-trace / all-traces | Hold with the new rules too |
| `sanity_owner_recovers` | exists-trace | The owner recovers, a contact compares the words and follows |
| `sanity_recovery_despite_rotate` | exists-trace | A phone stolen while locked (`IK` leaked, `AK` not leaked) writes a `rotate`, the contact follows this settled `rotate`, and still follows the recovery in the same period |
| `sanity_reject_after_settled_rotate` | exists-trace | A member can reject the pending recovery even after its own `rotate` has settled and been followed |
| `ak_record_stops_recovery` | all-traces | **P7, devices:** once an honest member has written an `AK` record (`recovery_reject`, `add`, `remove`) from a head of period `m`, nobody follows a recovery against `m`. The reader rule restates `Rec_Priority`; the real content is that the reject can be written at any time (the rule's only precondition is the view, see above) |
| `recovery_was_pending` | all-traces | **P7, devices:** every recovery that is followed was itself in the log for the wait beforehand, where every member could see it (a restatement of the reader's wait rule) |
| `read_rec_was_logged` | all-traces, helper | Every recovery that is followed was written to the log first (for `no_split_recovery`) |
| `no_split_recovery` | all-traces | Two readers do not follow different ones of two recoveries against the same period (a consequence of `Rec_Single`) |
| `contact_not_moved_by_phrase_alone` | all-traces | **P7, contacts:** the phrase alone does not move a contact; a contact follows a recovery only to the device the owner added against the same period, after checking the identity for that record. This lemma restates the identity-check assumption above |

### Mutant checks

| Mutant | Broken check | Expected attack |
| --- | --- | --- |
| `recovery_without_wait` | The device follows the recovery without waiting | The recovery is followed without waiting in the log at all |
| `recovery_beats_ak_record` | The reader does not apply the priority rule | The recovery is followed after a member's reject |
| `contact_skips_words` | The contact does not check the identity again | The contact is moved to the device of the one who stole the phrase |
| `recoveries_not_a_fork` | Two recoveries do not count as a fork | A device and a contact follow different recoveries |

All four drop unused rules so the search ends (`drop`).

Because most lemmas restate reader rules, these mutants test that the
rules are wired into the model, not that the design is correct. The
design's real foundations are that a real reader can apply the priority
rule (below, feedback to the design) and that a member can reject at any
time; the second is read from the form of the rule and was not expressed
as a lemma.

### Not proven yet

- **Can the attacker's own key be a member:**
  `rk_remove_needs_staying_device` assumes that the staying member is an
  honest device. The first record and a recovery admit only honest
  devices; an `add` proposal is written only with a member's signature, so the
  attacker's own key can get in only with a leaked `AK`. This step was
  argued here, not proven: its lemma (if an honest device wrote the `add`
  proposal, the new member is an honest device or the proposer's `AK`
  leaked) did not finish in 10 minutes. A4c.
- **If the phone and the phrase are stolen together,** P7 gives no
  assurance (`hesap.md`, P7); the model does not claim it.
- **Endless rejection by a thief who holds `AK`:** someone who steals an
  unlocked device can reject every recovery with the priority rule. The
  owner's way out is `resolve_fork`: A4c.
- **Censorship:** if the witnesses censor the reject record together with
  the server, the wait is only a delay; P7 is then provided by the
  contacts' identity check. The model's log is single and uncensored (A2
  assumption).
- **A late reject silently splits readers:** if reader A has followed a
  settled recovery and an offline member then writes a reject from the
  same period, A stays on the recovery branch and the member on the reject
  branch. In the model readers never follow a reject and this fork is not
  detected (`hesap.md` rules 5 and 6): A4c.
- **Someone who stole only the phrase can stop the owner's recovery for
  good:** writing a second recovery against the same period stops both
  with `Rec_Single`. `resolve_fork` does not solve this either, because it
  is signed with `RK`; what remains is pairing again with the contacts.
  This is a denial of service and does not break P7: A4c and A4d.
- **After the recovery:** the recovered device has no view; it does not
  reject, does not remove the lost device and writes no records. There are
  no lemmas for what happens after a recovery.
- Forks after a recovery, `resolve_fork` and a rejected `RK`: A4c.

### Feedback to the design (to be written into `hesap.md` in A4d)

- A recovery is a proposal written not against a head but against the
  membership period; `rotate` does not stop it, it carries over. A member
  can always reject a pending recovery from its current head.
- Two recoveries against the same period are a real fork. Someone who
  stole only the phrase can use this to stop the owner's recovery (denial
  of service); the design should say so.
- A reject does not depend on the writer's freshness: a member of a forked
  or filled head must also be able to reject.
- For priority, a real reader must look for an `AK` record under **every**
  head of period `m` (including `rotate` branches it does not follow and
  forked heads). `hesap.md` says only "under the same key" today. The
  model counts every value with a valid `AK` signature whatever its `op`;
  this is cautious for P7.
- The model writes the recovery under a separate key `h('rlog', m)`;
  `hesap.md:447`, however, describes the recovery under `L_{e+1}`, naming
  the newest head as its parent. One must be chosen in A4d.
- The contact's re-verification is bound to the digest of the recovery
  record (the re-find mailbox is already named with this digest).
- The contacts' half of P7 rests not on the words but on the contact
  recognising the other person out of band; `hesap.md` and the user texts
  should say this plainly.

## A4b: strict mode (`hayalet-halka-sikimod.spthy`)

Built on A4a's idea of membership periods and proposals. In strict mode a
single signature cannot change membership at once (`hesap.md`,
"Multi-layer approval", K3b). Every change is a **proposal** written
against the membership period `m` that readers follow only once it has
settled (in reality 72 hours of witness time):

- **`AK` proposal** (a pending `add` or `remove`): one member's `AK`
  signature.
- **Recovery** (`recovery_add`): an `RK` signature, into an empty slot.
- **`rk_remove(x)`:** an `RK` signature **and the staying member's `AK`
  signature**. This is the way out if a device `x` stolen unlocked vetoes
  its own removal: `x`'s reject does not count against it. Because it also
  needs the staying member's `AK`, the phrase alone cannot create it, and
  `RK` can never remove a ring's only device.

A **reject** (`reject_pending`, `recovery_reject`) is targeted: it is an
`AK` record that names the proposal by its digest, and it counts only if
it entered the log before the proposal settled. A counted reject voids only
that proposal; it does not start a new period and stops nothing else.

**One change per period:** a reader follows a proposal only if every other
proposal that entered the log before it against the same period was voided
by a counted reject: the first writer wins. A live earlier proposal stops
the later ones until it is rejected, or settles and is followed itself;
the ones written later do not count against the earlier proposal.

With these rules the model proposes an answer to two questions in the
"Rules to settle in the formal model" section of `hesap.md`: the ordering
of the pending record (a proposal against the period, which does not race
with the head's key, one change per period) and the exact definition of
the `RK` path when a stolen device vetoes (`rk_remove`).

### Gaps the review found in the first version

In the first version (cbd7333) a reject was untargeted (every `AK` record
written from the period stopped all of that period's proposals),
`reject_pending` and `recovery_reject` were a single record, and
`rk_remove` carried only the `RK` signature. The independent review found:

- **Critical:** according to `hesap.md`, `recovery_reject` voids `RK`. With
  the two rejects as one record, the stolen device's veto also voided `RK`
  and closed the `RK` path. Now `reject_pending` is a separate, targeted
  record (below, feedback to the design).
- **Splitting readers (found in Tamarin):** there was no order between an
  `AK` proposal and an `RK` proposal. One contact could follow the thief's
  `remove(d)` proposal and another the owner's `rk_remove(e)`. Now there is
  one change per period (`no_split_change`).
- **A single reject froze the period:** an untargeted reject also killed
  all later proposals of that period. If the thief proposed first, the
  owner's reject also stopped the owner's own `rk_remove`, and the ring
  froze with the thief inside. Now a reject is targeted and time-bound
  (`sanity_thief_proposes_first`).
- **The owner's phone could be removed with the phrase alone:** because the
  target's reject did not count, if the other device was offline for 72
  hours, the phrase thief could remove the phone the owner carries. Now
  `rk_remove` also needs the staying member's `AK` signature
  (`rk_remove_needs_staying_device`).

The second review (b7a25cf) gave three important findings, all fixed:

- `Change_Single` looked at every proposal written before the moment of
  reading. So, after `rk_remove(e)` had settled, the thief could leave late
  readers stuck for ever with a proposal written to the old period. Now
  only the proposals that settled before the followed one count. `Old_Once`
  makes sure every record has a single settling moment, and
  `Settle_In_Order` makes proposals settle in log order. Checking the fix
  showed that, without `Settle_In_Order`, a live proposal written earlier
  could stay unsettled for ever and so be skipped.
- Two reachability lemmas did not require the reject to be on time.
- The `strict_immediate_accept` mutant did not test any lemma of the
  model. Now it carries `ReadChange` and breaks `no_change_without_wait`.

### Abstractions

| Abstraction | Why | When it widens |
| --- | --- | --- |
| The ring is strict from the first record (the policy field is in the first record); no switching on and off with a `policy` record | Switching off needs the number of signatures strict mode asks for, so it is a proposal; it falls under the same rule | A4c or stage C |
| The first record carries two devices | Strict mode is recommended for two or more devices | Stage C |
| No two-signature `add` and `remove` | With two slots there can be no two-signature `add`. A two-signature `remove` (two members' signatures, at once) is the honest way, but removing the device in the second slot needs its signature too; that is not possible in the stolen-device scenario. Its interaction with proposals was not modelled | A4c |
| No `rotate` | Proposals are written against the period and `rotate` does not change the period; A4a showed that `rotate` blocks neither recovery nor reject | Does not change |
| A reject does not start a new period; readers do not follow a reject | A targeted reject voids only its own proposal; the period's members do not change | Does not change |
| **A rejected `RK` does not become invalid** | According to `hesap.md`, `recovery_reject` voids `RK`; here `RK` is permanent. Voiding must not happen through the reject of `x`, the target of `rk_remove(x)` (below) | A4c (with `change_phrase`) |
| The added device gets no view once the proposal is followed | Not needed for this piece's properties (as in A4a) | A4c |
| Every record has a single settling moment (`Old_Once`); proposals against a period settle in the order they entered the log (`Settle_In_Order`) | All the rules look only at the first settling moment. In reality the wait is the same for everyone, so the settling order is the log order | Does not change |
| No extra factor for recovery in strict mode (YubiKey, Shamir share) | An extra factor only makes stealing the phrase harder | Stage C |
| The log does not check that the `AK` signing `rk_remove` belongs to a member; the reader does. A reject can be written with any value taken from the network, not with the writer's own view | Both only add traces (in the safe direction for security lemmas). Modelling the log's member check and reading from the log for a reject took Tamarin's search out of memory | A4c |

### Lemmas

| Lemma | Kind | What it says |
| --- | --- | --- |
| `sanity_strict_remove` | exists-trace | A member proposes removing the other, the proposal waits, a contact follows |
| `sanity_stolen_device_vetoes` | exists-trace | The owner proposes removing the stolen `e`; the thief vetoes with `e`'s `AK` before the proposal settles (counted) |
| `sanity_rk_remove_despite_thief_reject` | exists-trace | The owner writes `rk_remove(e)` with the phrase and their own device, the thief rejects it too before it settles; the contact still follows. Without `x`'s exemption there is no trace (tried locally). Together with the previous lemma it is the stolen-device scenario; proving both steps in one trace did not end in the search |
| `sanity_late_proposal_does_not_block` | exists-trace | After `rk_remove(e)` settles, the thief writes `remove(d)` to the old period; a contact that reads afterwards still follows `rk_remove(e)`. With the old `Change_Single` there is no trace (tried locally) |
| `sanity_thief_proposes_first` | exists-trace | The reverse order: the thief proposes `remove(d)` first, `d` rejects in time, the owner still removes the stolen device with `rk_remove(e)`; the rejected proposal written first settles first (`Settle_In_Order`) and does not block the later `rk_remove` |
| `sanity_recovery_after_strict_remove` | exists-trace | Recovery is reachable in this model too: after a strict removal the owner recovers into the empty slot, a contact follows |
| `no_change_without_wait` | all-traces | **K3b:** a reader never changes membership at once; the change it follows was itself in the log for the wait beforehand. A phone stolen unlocked cannot move anyone at once, nor against a counted reject (`reject_stops_change`); if nobody rejects, it moves them at the end of the wait (below, the offline member). It restates `Settled_Old` |
| `reject_stops_change` | all-traces | **K3b:** a reject an honest member writes before the proposal settles stops it (for `rk_remove(x)`, a member other than `x`). It restates `Not_Rejected` |
| `read_change_was_logged` | all-traces, helper | Every proposal that is followed was written to the log first |
| `read_change_settled` | all-traces, helper | Every proposal that is followed settled first (`Old`) |
| `earlier_proposal_must_be_rejected` | all-traces | The first writer wins: a reader follows a proposal only if every proposal written before it against the same period was voided by a counted reject; a live earlier proposal (the thief's) is never skipped |
| `no_split_change` | all-traces | Two readers do not follow different changes in a period, whatever the stolen device or the phrase thief writes. Written for `#i < #j`: the other order is the same lemma with the names swapped, and the same moment is a single rule that carries a single `ReadChange` |
| `rk_remove_needs_staying_device` | all-traces | **P7, `rk_remove`:** if the staying member `b` is an honest device, a reader follows `rk_remove(x)` only if the owner wrote it or `b`'s `AK` leaked too (phone and phrase together, outside P7) |
| `contact_not_moved_by_phrase_alone` | all-traces | A4a's P7 lemma holds with the new rules too |

`no_split_change` and `rk_remove_needs_staying_device` do not restate rules;
they are the combined consequence of the reader rules. The others express
reader rules as in A4a.

### Mutant checks

| Mutant | Broken check | Expected attack |
| --- | --- | --- |
| `strict_immediate_accept` | The reader follows a single-signature `remove` at once, as in the default mode | The change is followed without waiting (the negation of `no_change_without_wait`) |
| `proposal_without_wait` | The reader follows a proposal before it settles | The proposal is followed without waiting |
| `veto_ignored` | The reader does not count a reject written in time | The rejected proposal is followed |
| `rk_remove_by_phrase_alone` | `rk_remove` needs only `RK` | A device is removed with the phrase alone |
| `changes_not_a_fork` | The reader does not look at the period's other proposals | Two readers follow different changes |
| `exemption_only_in_veto` | `Change_Single` counts `x`'s own reject of `rk_remove(x)`, `Not_Rejected` does not | Two readers follow different changes: the two restrictions must define "counted reject" the same way |

### Not proven yet

- **If the phone and the phrase are stolen together,** strict mode gives no
  assurance either: the thief can write `rk_remove` with the `AK` of the
  device in hand and the phrase, and remove the owner's device. `hesap.md`
  leaves this outside P7; the way out is pairing again.
- **Denial of service:** a device stolen unlocked can reject every `AK`
  proposal and keep writing new proposals; the owner has to reject each one
  within the wait, or else a live proposal written before the owner's
  `rk_remove` stops it (or wins itself). The phrase thief can do the same
  with a second recovery proposal. It does not break security, it makes
  the owner wait: A4c and A5.
- **Offline member:** to reject a proposal, the member has to be online
  within the wait. While the owner's device is offline for 72 hours, the
  thief's `remove(d)` proposal goes through; this is how strict mode is
  designed (with the assumption, as in A4a, that devices see the log).
- **Censorship:** if the witnesses censor the owner's reject together with
  the server, contacts follow the thief's proposal after 72 hours. Contacts
  do not compare words for `AK` proposals. The model's log is single and
  uncensored (A2 assumption); asking for the words for proposals that
  remove a device will be discussed in A4d.
- **A late reject and what follows:** a reader that has followed a
  proposal does not count a reject that comes later (a reject is
  time-bound). Rings with two or more members, a `policy` change and the
  ring after a proposal is followed: A4c and stage C.

### Feedback to the design (to be written into `hesap.md` in A4d)

- In strict mode every change is a proposal against the membership period,
  has its own log key and does not race with the head's key; one change
  per period is followed and the first writer wins: every proposal that
  **entered the log before** the followed proposal (that is, since the wait is
  the same for everyone, that settled before it) must have been voided by a
  counted reject (`earlier_proposal_must_be_rejected`); the ones that come
  later do not count against it and lose to it. Otherwise someone who can
  still write to an old period (a thief holding the removed device's `AK`)
  can make late-reading contacts never follow the change
  (`sanity_late_proposal_does_not_block`).
- A departure from `hesap.md` rule 5: by rule 5 a recovery record does not
  open a fork and the device record has priority. In strict mode a
  recovery is a proposal too and joins the same queue: a live `AK`
  proposal written before it in the same period stops it, and an `AK`
  proposal written after it loses to it. Rule 5 should be written this way
  for strict mode.
- A reject is targeted (it names the proposal by its digest), counts only
  if it entered the log before the proposal settled, and voids only that
  proposal. `reject_pending` and `recovery_reject` should be kept separate,
  or, if they are one record, the rule that voids `RK` should depend on the
  target: a reject by `x`, the target of `rk_remove(x)`, must never void
  `RK`.
- The exact definition of "if the stolen device writes `reject_pending`,
  the path goes through `RK`": `rk_remove(x)` is a pending proposal that
  carries `RK` and the staying member's `AK` signature and that members
  other than `x` can reject. The phrase alone is not enough.
- To add to K3b: a contact does not follow an `rk_remove(x)` that a member
  other than `x` rejected in time; it follows neither of two live proposals
  of the period.
- A real reader measures that a reject entered the log "before settling"
  with witness time (A5).
