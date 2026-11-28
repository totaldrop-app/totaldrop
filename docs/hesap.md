# Account system: the Ghost ring (design)

- **Status:** Accepted, 4 November 2026
  ([decision 0013](kararlar/0013-hayalet-halka-hesap.md), #243).
  Supersedes [decision 0004](kararlar/0004-mullvad-tarzi-hesap.md) (the
  16-digit number).
- **Code:** none. This document binds the behaviour. The byte-level
  formats are added to the [protocol notes](protocol-v0.1-notes.md) before
  any code is written.
- **Review:** before any code is written, the design is tested with a
  formal model (Tamarin and TLA+) and an independent human cryptography
  review (see "Order of verification and implementation"). The Tamarin
  models (A1–A4c) and TLA+ timing evaluation (A5) are in
  [formal/](../formal/README.md) (#251), and the rules they settled have
  been written into this document.

In TotalDrop the account is optional. Sending and receiving with a code
keeps working without an account. An account gives two things:

- **My devices:** your devices send to each other without entering a code.
- **Known contacts:** you send to people you have paired with once, without
  entering a code.

## Principles

1. **No account on the server.** The server keeps no account table, device
   list or contact list. The account is a signed membership chain that your
   devices keep among themselves.
2. **The server is untrusted.** Every security claim must hold even while
   the server is malicious ([threat model](tehdit-modeli.md)).
3. **No new cryptographic primitives.** Only standard parts are used:
   Ed25519, ML-DSA-65, X-Wing (X25519 + ML-KEM-768), HPKE, HKDF-SHA-256,
   CPace, Noise (the existing handshake), Privacy Pass ARC, Shamir. Their
   combination is still a custom group membership protocol; the risk is
   managed accordingly.
4. **Fail closed.** If a contradiction is seen, sending stops and the user
   is warned; neither side is trusted silently.
5. **No sensitive operation can be done by compromising a single secret or
   a single device.**

## What the user sees

### No identity to type

No account number, user name or password is typed on any device.

- The first device sets up the ring by itself.
- When a new device is added, the new device shows a QR. A device in the
  ring scans the QR and starts the addition.
- The same verification words appear on both screens. The user picks the
  right words from a few options.
- The local network is used only to **find** the device. Adding is always
  done with a QR or a code (CPace).

### One list

Every device in the ring sees the same device list and the same contact
list. A contact added on the phone is ready on the PC too. The lists are
synchronised inside the ring, end-to-end encrypted.

### The user gives the names

Behind the scenes, identities are random keys. On screen, the name the user
gave appears: "Mum", "Work PC". This approach is called "petnames" (Miller
and Stiegler): a secure, local name is used instead of a global, unique
one.

- Names are synchronised only inside the ring, encrypted. They do not go to
  the other side or to the server.
- The name the other side suggests is shown as a separate, faded label. It
  is not filled into the name field automatically; otherwise an attacker
  could call themselves "Mum".
- If the same or a similar name is given, a warning appears. The
  similarity check also covers homoglyphs and directional characters.
- An **8-word (88-bit)** fingerprint derived from the contact's ring
  identity (see "Membership chain") is shown on request. The ring identity
  does not change for the ring's whole lifetime. Colour is only
  decoration and carries no security claim.

### Trust tiers

| Tier | How it is added | What happens on sending |
| --- | --- | --- |
| **My devices** | QR or code + verification words, once | Received without asking (can be turned off in settings) |
| **Known contacts** | A three-word code + verification words, once | Sent without a code. The receiver is asked "Mum is sending 3 files, accept?". "Always accept" can be chosen per contact |
| **Nearby** | With a code every time (today's system) | Approval is asked every time |

- The system is built on "a known person", not "a known device". Which of
  their devices a person sends from does not matter. When a person adds a
  device to their ring or removes one, the other side has nothing to do.
- After a successful transfer with a code, a "Save this contact" button
  appears. The button only appears if both sides have an account.
- A blocked contact's records are deleted and the secret key between the
  two sides is discarded. No notification goes to the blocked side; their
  offers stay unanswered. The blocked contact can keep following the
  ring's later heads in the log from the head they hold (see "Remaining
  risks").

### Multi-layer approval

| Operation | Default | Strict mode (setting) |
| --- | --- | --- |
| Adding a device | The management key on an existing device (biometrics or PIN) + verification words. A notification goes to the other devices at once; it can be undone with one tap within 24 hours (undoing is a `remove` record) | `add` carries the `AK` signatures of two distinct devices (takes effect at once). If a second signature does not come (or the ring has a single device), the record enters the period proposal queue (`H("log prop", m)`) as **pending** with one signature and takes effect after 72 hours of witness time; any current device can cancel it before settling with a targeted `reject_pending` |
| Removing a device | The management key | The same rule as `add`: two distinct signatures at once, or one signature pending in the period queue for 72 hours of witness time. Because contacts do not see the number of devices, the rule is the same for every ring size. If a stolen device vetoes its own removal with `reject_pending`, the owner writes `rk_remove(IK)` signed by both `RK` and a staying member's `AK` (waits 72 hours; the target's own `reject_pending` does not count, while any other member's timely `reject_pending` voids it) |
| Turning strict mode on or off | One device's `AK` (`policy`) | Turning strict mode off needs two distinct devices' `AK` signatures (no single-signature downgrade path; if only one device survives, add a second device via a 72-hour pending `add` or `recovery_add` first and co-sign with two `AK`s, or pair again) |
| Adding by recovery | 24 words (`RK`, logged under `H("log recovery proposal", m)`) + waiting + out-of-band re-verification with contacts | Besides the 24 words, a YubiKey or the Shamir shares of 2 of 3 contacts; the `recovery_add` joins the unified period queue `H("log prop", m)` |
| Changing the recovery phrase | One device's `AK` + waiting (`change_phrase`); once settled, the old `RK` is invalid from that head onward | Two distinct devices' `AK` signatures + waiting (`change_phrase`) |
| "Always accept" for a contact | The management key | The same |

Strict mode is not just a device setting: it sits as a **policy field** in
the chain and in the head sent to contacts. So contacts also check that an
immediate single-signature `add` or `remove` is invalid in strict mode.

There is no second factor on the server side (SMS, TOTP), and there will
not be, because there is no account on the server. If the ring has 2 or
more devices and a recovery phrase, turning on strict mode is recommended.

## Keys

| Key | Owner | Use |
| --- | --- | --- |
| `IK_i`: Ed25519 + ML-DSA-65 | Every device | Signs `rotate` records and the single-use endpoint key in the handshake |
| `AK_i`: Ed25519 + ML-DSA-65, bound to user verification | Every device, in the OS key store | Management key: records that change membership, the contact book, rejecting a recovery, changing the phrase |
| `KK_i`: X-Wing | Every device, renewed periodically | Receiving the new ring secret |
| `RK`: Ed25519 + ML-DSA-65 and `RKK`: X-Wing | Derived from 24 words (BIP-39, 256 bits), stored on no device; their public keys are in the chain | `RK` only for delayed device addition and fork resolution; `RKK` to open the monthly recovery record in the log |
| `r_e`: 32 bytes | Every device in epoch e | Mailbox names, the ring KEM key |
| `k_AB`: 32 bytes | Two rings (you and a contact) | Contact mailboxes; renewed on every device removal |

Every derivation is done with `HKDF-SHA-256`. Labels are length-prefixed
and unique (domain separation). Values such as `day` are encoded with a
fixed width. Every record has an algorithm version field.

In a hybrid signature, **both** halves must verify. The two halves sign
domain-separated messages, and each is bound to the other's public key.
That the verification path cannot skip a half is tested with separate
tests.

The key store reality: ML-DSA keys cannot be kept non-exportable in the
Android Keystore or the Linux Secret Service. Ed25519 is in hardware only
on recent Android versions. So every ML-DSA half is stored as a blob
wrapped with a hardware key; the wrapping key of `AK` is also bound to
user verification. Because `IK` signs on every transfer, it cannot be
bound to biometrics. That is why management operations are done with a
separate `AK`.

## Membership chain

A record carries these fields: version, epoch `e`, a digest of the
previous record's canonical encoding, the operation, the policy field
(strict mode), the payload, one or two signers, and the signatures.

The **ring identity** is the canonical digest of the `create` record:
`H("ring", create)`. It does not change for the ring's whole lifetime; the
HPKE `info` fields, offers, handshake signatures and the fingerprint are
bound to this value.

The operations are:

| Operation | Signer | Note |
| --- | --- | --- |
| `create` | The first device's `AK` | Contains the `RK` and `RKK` public keys and initial policy |
| `add(IK, AK, KK)` | An existing device's `AK` (in strict mode two distinct devices at once, or one signature pending in `H("log prop", m)`) | |
| `remove(IK)` | An existing device's `AK` (in strict mode two distinct devices at once, or one signature pending in `H("log prop", m)`) | In the default mode it is an emergency operation, done at once |
| `rk_remove(IK)` | Active `RK` + a staying device's `AK` | Strict-mode escape when a stolen device vetoes its own removal; waits 72 hours in `H("log prop", m)`. The targeted device's own `reject_pending` does not count; any other member's timely `reject_pending` voids it. `RK` alone cannot create it |
| `reject_pending(digest)` | An existing device's `AK` | Targeted cancellation of one pending proposal in strict mode; counts only if it enters a witnessed root before that proposal settles. Does not start a new period and does not revoke `RK` |
| `policy(strict mode)` | `1 AK` to enable strict mode; `2 distinct AK`s in strict mode to turn it off | Turns strict mode on or off |
| `rotate` | An existing device's `IK` | Only renews the secret; the member set and membership period `m` do not change. Contacts follow after `D = 4h` of witness time; ring devices follow after `D_device = 6h` |
| `recovery_add(IK, AK, KK)` | Active `RK` | Adds into a free slot against membership period `m` (`H("log recovery proposal", m)` in default mode; `H("log prop", m)` in strict mode), waits by witness time, and needs out-of-band contact re-verification |
| `recovery_reject` | An existing device's `AK` | Distinct from `reject_pending`. Cancels a pending `recovery_add` (independent of writer freshness) and, on an undisputed head, revokes `RK`; on a disputed branch of a real fork it does not revoke `RK` |
| `change_phrase(RK, RKK)` | `1 AK` (default) or `2 distinct AK`s (strict) | Rotates to a new phrase after its waiting period settles; the old `RK` is rejected at the new head and its descendants |
| `resolve_fork` | Active `RK` | Chooses one validly logged branch under a forked head, waits by witness time, overrides rule 4 on the losing branch, and requires out-of-band contact re-verification bound to its digest |

The contact list and names are not in this chain; they are kept in a
separate **contact book** inside the ring (see "The ring towards
contacts"); contact changes do not start an epoch.

- **Every chain record starts a new epoch** and generates a new
  `r_{e+1}`. This secret is encrypted with HPKE separately to the `KK` key
  of every remaining device. The HPKE `info` field binds: the ring
  identity, the epoch and the receiver's `IK`.
- **The size is fixed.** A record always carries 15 HPKE ciphertexts; the
  empty ones are random padding. The server cannot infer the number of
  devices from the record size.
- **The record's signature covers every ciphertext.** Every receiver
  verifies a key confirmation (MAC) under `r_{e+1}`. A device that
  receives a broken payload does not drop silently; it reports an error.
- **The chain is not visible to the server.** It travels end-to-end
  encrypted in the ring mailbox.
- **Timing:**
  - Ring heads have a lifetime of at most 7 days.
  - The weekly `rotate` is done by **a single device** per epoch: the first
    device online in an order derived from `r_e`, with a random offset.
    The other devices take over only 24 hours before the head expires. So
    the rotation pattern does not reveal the number of devices.
  - Extending a head is always done with a new epoch. An old head is not
    extended by re-signing it.
  - Membership changes are not hidden: contacts see **in which epoch** the
    member set changed (not how many devices there are). Hiding the change
    would stop contacts from checking the rules for adding and removing;
    security was put ahead of this privacy.

### Fork rule

More than one valid continuation of the same epoch may be seen. For
example, two offline devices may have rotated at the same time, or a stolen
device may have written its own continuation. The log keeps every valid
continuation (see "Anonymous transparency log"). Everyone who reads
applies these rules:

1. **Priority and `rotate` settling:** an `AK`-signed membership record
   (`remove`, `add`, `policy`, `change_phrase`, `recovery_reject`) takes
   priority over an `IK`-signed `rotate` in the same epoch. To prevent a
   removed device from stranding contacts on an early `rotate` branch (the
   gap found in formal model A2):
   - Contacts follow a `rotate` only once it has stood in the log for
     **`D = 4 hours`** of witness time from the witnessed root in which it
     first appeared (`T_entry`, where `D > 1h inclusion + 1h root age + 2 * Delta`).
   - Before writing an `AK` record from the pre-rotate head, an honest ring
     device checks a fresh witnessed root (at most 1 hour old) showing
     **no value under the head's key yet (`D_w = 0`)**. A concurrent
     `rotate` not yet present in that root does not block the write, and
     the resulting `AK` record enters a witnessed root strictly before
     `D = 4 hours` and beats the unsettled `rotate` by rule 1.
   - Once a `rotate` is already visible in the writer's fresh witnessed
     root (past `D_w = 0`), the ring device (including the rotating device
     itself) **stops writing `AK` records from the pre-rotate head** — so
     no late `AK` record can enter after `D = 4 hours` and split contacts —
     waits until **`D_device = 6 hours`** of witness time
     (`D_device >= D + 1h root age + 2 * Delta`, ensuring no contact can
     still pass freshness on an unsettled root), and then follows the
     `rotate` and writes `AK` records from the post-rotate head. (Contacts'
     send/receive freshness check is looser: an unsettled `rotate` does not
     block sending or receiving; see "Freshness".)
   - Every `AK` record binds the `<= 1h`-fresh witnessed root it was
     checked against; witnesses and readers reject any `AK` record that
     enters a witnessed root more than **`2 hours + 2 * Delta`**
     (`1h root age + 1h inclusion + 2 * Delta`, i.e. 2 hours 10 minutes for
     `Delta <= 5 minutes`) after its bound root's witness timestamp (if
     delayed past that window, the writer re-checks freshness and
     re-signs). Because `2 hours + 2 * Delta < D = 4 hours`, an honest
     on-time writer is never rejected while a delayed pre-signed `AK`
     upload can never enter after the `rotate` settles.
2. **Harmless fork:** concurrent `rotate` records with the same member set
   merge deterministically: the one with the smallest digest wins. Because
   the signer chooses `r`, it can grind which record wins; this is harmless,
   because the membership is the same and the key confirmation shows that
   no device was dropped.
3. **Real fork:** in every other case (e.g. two `remove`s removing each
   other) devices and contacts fail closed. Sending stops and a warning is
   shown.
4. An observer ignores forks below the head it has pinned (except where
   rule 7 overrides this on a losing branch).
5. **Disputed branches, recovery period keys, and the strict-mode queue:**
   a **disputed branch** is one of the branches of an unresolved real fork
   formed by two device-signed records. A reject by devices that are
   members only on a disputed branch does not count.
   - **Default mode (`recovery_add`):** a recovery proposal is written not
     under `L_{e+1}` but against the **membership period `m`** (the head of
     the last membership record; `rotate` does not change `m`) under its own
     log key `H("log recovery proposal", m)`. A pending recovery therefore
     carries over subsequent `rotate`s without opening a fork, and a locked
     stolen phone's `rotate` neither cancels nor delays it. Any valid `AK`
     record written from **any** head of period `m` (readers check every
     head of period `m`, including unfollowed `rotate` branches and forked
     heads) beats the recovery by rule 1. A member writes `recovery_reject`
     from its current head without depending on writer freshness, so even a
     member on a forked or filled head can reject. **Two `recovery_add`
     records against the same period `m` are a real fork** and readers
     follow neither.
   - **Strict mode (unified period queue):** every single-signature change
     (`add`, `remove`, `recovery_add`, `rk_remove(x)`) is a proposal logged
     under one shared period key **`H("log prop", m)`** and waits 72 hours
     of witness time (7 days for single-device `recovery_add`). **One change
     per period, first writer wins:** a reader follows a proposal `v` only
     if it has settled, has no counted targeted reject (`reject_pending(H(v))`
     for `AK`/`rk_remove` proposals, or `recovery_reject(H(v))` for
     `recovery_add`), and every other proposal against `m` that entered a
     witnessed root **before `v`** was voided by a counted reject. Proposals
     logged after `v` lose to `v` and cannot freeze late readers. A targeted
     reject counts **if and only if** it enters a witnessed root strictly
     before `v`'s witness-time settling deadline
     (`rejEntry < propEntry + T_SETTLE`, i.e. 72 hours or 7 days); late
     rejects are ignored uniformly by all readers so early and late readers
     never split. `reject_pending`
     voids only its named proposal, does not start a new period, and is kept
     separate from `recovery_reject` so a stolen device's veto never revokes
     `RK`. For `rk_remove(x)`, `x`'s own `reject_pending` is exempt while
     any other current member's timely `reject_pending` voids it.
6. **A late fork is a fork too.** However late a value was written, it is
   not ignored; otherwise the first writer (e.g. a device stolen unlocked,
   while its owner's device is offline) would win for good. Rule 4 only
   protects the ring's own devices and contacts whose head has moved past
   the fork; a contact that pinned an old head sees a late fork and fails
   closed (`late_fork_never_hijacks_contact`). In a ring without a recovery
   phrase, no safer automatic rule exists: affected contacts pair again out
   of band. This is a loss of service, not a takeover (see "Remaining
   risks").
7. **Fork resolution (`resolve_fork`):** only a `resolve_fork` record signed
   by the head's **active** `RK` resolves a real fork (an old `RK` replaced
   by a settled `change_phrase` or revoked by an undisputed
   `recovery_reject` is rejected). The record must name a branch that was
   **validly logged** under the forked head's key with a valid member `AK`
   signature (`resolved_branch_has_valid_lineage`); a stolen `RK` alone
   cannot fabricate an unlogged branch. Two competing `resolve_fork` records
   for the same forked head and active `RK` are themselves a fork and
   readers follow neither (`no_split_fork_resolution`). `resolve_fork` is
   subject to the witness-time waiting period and **overrides rule 4**:
   contacts that already pinned their head on the losing branch also move to
   the chosen branch. Contacts treat `resolve_fork` like a recovery: there
   is no automatic acceptance, and out-of-band human re-verification bound
   to the `resolve_fork` digest is required
   (`resolve_fork_requires_contact_reverification`). So someone who steals
   the phrase — even together with an old or compromised device `AK` —
   cannot move contacts silently. While the fork is unresolved, both
   branches are disputed and a `recovery_reject` on a disputed branch does
   not invalidate `RK`.
8. If the fork cannot be resolved (the thief holds both a device and the
   phrase, or stalls resolution with a competing `RK` record), contacts pair
   with you again. The worst outcome is a loss of service, not a silent
   takeover.

## Mailboxes

The server only knows short-lived mailboxes. The mailboxes are Durable
Objects; each is opened under a 16-byte name that looks random, and has
size and message limits. Writing to a mailbox needs an ARC token. There is
no other table on the server.

These **account mailboxes** are a different kind from the rendezvous
mailboxes of a transfer with a code. The rendezvous mailbox rules in the
[protocol notes](protocol-v0.1-notes.md) (deleted within an hour, 32
messages per side, 16 KiB body) do not change; the account mailboxes'
lifetime and limits are below, and are written into the protocol notes as
a separate item before the code.

| Mailbox | Name | Lifetime |
| --- | --- | --- |
| Ring | `HKDF(r_e, "mailbox ring" ‖ day)` | 48 hours |
| Contact, per direction | `HKDF(k_AB, "mailbox to A" ‖ day)` | 48 hours |
| Catch-up | From a device-specific secret value in the chain, per epoch | 30 days |

- **Day and month:** the UTC date, from the wall clock. Devices listen to
  yesterday's, today's and tomorrow's mailbox; this covers a ±1 day clock
  difference between devices. A larger difference is shown to the user as
  a "clock is wrong" warning.
- **Waiting periods** (recovery, strict mode) are measured not by the
  local clock but by the timestamps in the log roots that the witnesses
  co-sign. Because the witnesses are independent, a fake time from the
  network (NTP, NITZ) does not shorten the wait. The device also keeps the
  largest time it has seen, and rejects time going backwards.
- A device offline for more than two days catches up from the catch-up
  mailbox and through direct chain synchronisation with the other
  devices. A contact offline for a long time uses the "ask for the current
  head" flow.
- **Recovery record:** every month, a device in the ring writes **only the
  latest head**, encrypted with HPKE to `RKK`, into the transparency log.
  The key is `H("log recovery", RKK public key, month)`. This is not a
  mailbox: the log deletes no value, so the real head cannot be pushed out
  by writing garbage; the reader discards values with invalid signatures.
  The contact book, the `k_AB`s, the names and the ring secret (`r_e`) are
  not in the record. So someone who steals the phrase cannot, without
  waiting, learn your contacts, their names or the contact mailboxes. A
  removed device can also compute the key; it cannot open the content, and
  the head there gives no more than what it can already follow from the
  log (see "Remaining risks").
- Each mailbox is polled with a separate connection and at random times.
  Connections from the same NAT IP can still be linked (see "Remaining
  risks").
- Chain records (~23 KB) exceed the rendezvous mailbox body limit (16 KiB).
  Account mailboxes use chunked sending.

## The ring towards contacts

- **Ring head:** the devices are put into a Merkle tree with 16 leaves.
  Each leaf has the form `H("leaf", m, nonce_i, IK_i, AK_i)`; `m` is the
  membership version and goes up only when the member set changes. The
  leaves are re-salted on every membership version, and their order is
  random. Empty leaves are random padding. The tree's root is `C_e`.
- **The head sent to a contact:** `{ring identity, e, C_e, operation type,
  policy, ring KEM public key, previous_head_digest, expiry}`. The
  operation type can be `rotate`, `membership` or `recovery`. The ring KEM
  public key is derived from `r_e`; offers are encrypted to it.
- **How a contact checks the head:**
  - `rotate`: the root does not change (`C_e = C_{e−1}`); the contact
    compares this directly. An `IK` signature and a Merkle proof that the
    signer is in this root are enough. So membership cannot be changed with
    `IK`.
  - `membership`: with a Merkle proof against `C_{e−1}`, as many `AK`
    signatures as the previous head's policy field asks for. In strict
    mode, a single-signature `add` or `remove` is valid only after waiting
    72 hours by witness time, with no timely targeted `reject_pending` and
    no unvoided earlier proposal against period `m` (fork rule 5;
    including `rk_remove`).
  - `recovery`: an `RK` signature; the waiting and re-verification rules
    apply.
  - The proof is 4 steps and takes 128 bytes.
- **Pinning:** at the first pairing, the contact pins the ring head. This
  trust on first use (TOFU) is protected by the verification words. After
  that, the contact accepts only continuations that link to the previous
  head and are in the transparency log. Nothing is sent to an expired
  head.
- **Following the head:** starting from the head it pinned, the contact
  fetches the values under `L_{k+1}` in order, checks each one, and moves
  on until a proof of absence comes. This is at least 52 steps a year; the
  contact catches up on every connection.
- **Number of devices:** the head does not show the number of devices
  (1–16). The contact sees in which epoch the membership changed, and over
  time can count the different devices it sees in handshakes. These are
  accepted limits.
- **Contact book:** a contact's security fields (`k_AB`, the pinned head)
  and the name given are append-only records inside the ring, signed with a
  device's `AK`. A change in the security fields is bound to the pairing
  transcript. Ordering uses a hybrid logical clock (HLC) with bounded clock
  skew. The book is synchronised in the ring mailbox, encrypted under
  `r_e`.
- **Renewing contact keys:** when a device is removed, a new `k_AB` is
  generated for every contact. The new value is sent not with the old
  `k_AB` but with HPKE to the other ring's current X-Wing key, and with an
  `AK` signature; so the removed device that knows the old key cannot learn
  the new value.
- **Finding again:** how contacts find the ring after a recovery is in the
  "Recovery" section.
- **After a removal:** when a device is removed, every contact it added or
  changed is marked "verify again".

## Anonymous transparency log (in the first release)

Suppose the server and someone with an old authority (a removed device or a
stolen phrase) work together. If the server is the only channel between a
contact and the ring, the two can show an isolated contact a history that
is consistent in itself but fake. This is the known limit of fork
consistency (SUNDR). Getting past it needs either a single-view log or
out-of-band verification. TotalDrop uses both in the first release.

- **Structure:** the log is a verifiable map. **More than one value** can
  sit under a key, and none is ever deleted.
  - Key: `L_{e+1} = H("log", head_digest_e)`. Only those who know the e-th
    head (the ring and its contacts) can compute the key. The key of the
    `create` record is `L_0 = H("log first", ring identity)`.
  - Value: the (e+1)-th head, encrypted with a key derived from
    `head_digest_e`. The server and others who read the log cannot see the
    content, and cannot link rings to each other.
  - The reader decrypts **every** value under a key and checks its
    signature. Invalid values are ignored. More than one valid value means
    a fork, and the fork rule applies.
- **Witnesses:** independent witnesses co-sign the log's roots with a
  timestamp. So the server cannot show two contacts two different logs.
- **Enforcing inclusion:** if the witnesses only sign the root, the server
  can, by never writing an honest record (censorship), present a removed
  device's own `rotate` record as the only valid value. To prevent this:
  - The app carries a fixed list of witnesses (`n` witnesses). Contacts
    accept a root only with the signatures of at least `k` witnesses from
    this list, chosen for at most `f` colluding witnesses so that
    **`2k - n > f`** (single view; an honest majority alone is not enough,
    e.g. `n = 5, k = 4` tolerates `f <= 2` while `k = 3` tolerates only
    `f = 0`) and **`f < k`** (no censorship).
  - The ring also sends every record, together with `L` and the encrypted
    value itself, to **all** the witnesses on the list (with an ARC token).
    The witness forwards the record to the log itself too, and gives the
    ring a signed receipt.
  - A witness does not co-sign a root that still lacks an `(L, value)`
    pair it received more than 1 hour ago. Because the witness works with
    the full record and not a digest, the server can add every reported
    record; stopping the log with a made-up digest is not possible.
    Meaningless records stay under a random `L`, affect nobody, and are
    limited by ARC.
  - Every accepted root says that every witness included the record; so
    censorship needs `k` witnesses to work together. This is the assumption
    P2 and P3 already rest on.
  - Time limit: a record enters a root within 1 hour at the latest, and the
    root a contact accepts is at most 1 hour old. Allowing bounded clock
    skew `Delta` between participants and witnesses (verified tight in
    `formal/GhostRingTiming.tla`), the effect of a removal reaches every
    contact within **`2 hours + 2 * Delta`** at the latest (2 hours 10
    minutes for `Delta <= 5 minutes`).
  - The witness sees an `L` that looks random and an encrypted value, and
    cannot read the content. But it sees the connection's IP and time (see
    "Remaining risks").
- **Freshness:** before every send, the contact asks for a root that the
  witnesses co-signed at most 1 hour ago, and checks that in that root
  there is **no valid `AK` record and no settled value** under `L_{e+1}`
  (an unsettled `rotate` does not block sending, avoiding a 4-hour pause
  after every weekly `rotate`; this stays safe because no ring device
  follows a `rotate` until `D_device = 6 hours`). If a valid `AK` record or
  settled value is present, or no fresh root comes, it does not send. This
  stops, at the moment of sending, a removed device from receiving files
  with an old head (the freeze attack).
- **Freshness when receiving too:** a ring that receives an offer does the
  same check for the sending ring before showing the offer: a fresh
  witnessed root showing no valid `AK` record and no settled value under
  the key after the sender's head. So a removed device cannot send files in
  "Mum"'s name with an old head, even with "always accept" on.
- **The ring's own check:** if the ring sees a valid value it does not know
  under its own keys, it understands the situation at once and warns the
  user. It also warns if its own record with a witness receipt does not
  enter a witnessed root within 2 hours; that means censorship with the
  witnesses also working together.
- **Writing garbage:** someone who knows the key (a contact or a removed
  device) can write invalid values under the key. They are ignored, and
  every write needs an ARC token. There is **no** limit on values per key:
  with a limit, either the garbage would push the real record out, or
  enforcing inclusion would stop the log. The reader fetches the values
  page by page and discards the invalid ones cheaply (decryption fails).
  The remaining effect is the reader's bandwidth and the ARC cost of the
  garbage.
- **ARC keys:** the ARC issuer keys are published in the same log. So the
  server cannot tag clients by giving each one a different key (key
  consistency).
- **Candidates (to be verified):**
  - For the verifiable map, Meta's open-source `akd` library (Parakeet,
    NDSS 2023). Its fitness for several values per key will be checked.
  - For the witnesses, Sigsum or the transparency.dev witness network.

## Recovery

- **It is optional.** In a single-device ring it is recommended with a
  prominent warning.
- **Phrase:** 24 words (BIP-39) derive `RK` and `RKK`.
- **Splitting:** optionally, the phrase can be split into t-of-n shares
  with Shamir. For example 2 of 3 known contacts, or one contact + a
  YubiKey (WebAuthn PRF). The shares are checked against `RK`'s public key;
  a broken share is caught. Synced passkeys are not used, because they tie
  the user to an Apple or Google account.
- **The way (if every device is lost):**
  1. The new device derives `RKK` from the phrase, opens the latest
     recovery records in the log and learns the latest head. It does not
     learn the contacts.
  2. It writes an `RK`-signed `recovery_add` against the current membership
     period `m` (the head of the last membership record; `rotate` does not
     change `m`): under `H("log recovery proposal", m)` in default mode, or
     in the unified period queue `H("log prop", m)` in strict mode.
  3. The waiting starts (measured strictly in witness time). The ring's
     remaining devices (if any) and the contacts see the record in the log.
- **Waiting:** 72 hours if the ring has 2 or more devices, 7 days with a
  single device; measured by witness time.
  - During the wait, any device can write `recovery_reject` from its
    current head (independent of writer freshness). In default mode, an
    undisputed `recovery_reject` stops the recovery against `m` and revokes
    `RK` (a new phrase is given with `change_phrase`, with its own waiting
    period). In strict mode, `recovery_reject` is targeted by proposal
    digest and kept separate from `reject_pending`; it counts only if it
    enters a witnessed root before the proposal settles (so submitting
    strictly more than `1 hour + Delta` before the deadline guarantees
    voiding).
  - In default mode, any valid `AK` record written from any head of period
    `m` takes priority and invalidates the recovery (fork rule 5); an
    `IK`-signed `rotate` does not stop or delay the recovery, because it
    does not change `m`. In default mode, two competing `recovery_add`
    records against the same period `m` form a real fork and readers follow
    neither. In strict mode, `recovery_add` shares the unified
    first-writer-wins queue `H("log prop", m)` with `AK` proposals and
    `rk_remove` (fork rule 5): a later `recovery_add` loses to an earlier
    settled proposal and cannot freeze late readers, while if a phrase
    thief writes an earlier `recovery_add`, an honest member voids it with
    a timely targeted `recovery_reject` within 72 hours (and if no honest
    device remains, contacts still refuse out-of-band re-verification and
    pair again).
- **Against a malicious server, the wait is not a veto window:** if the
  server, together with the witnesses, censors the reject record, the
  wait is only a delay. In that case P7 is provided by the contacts'
  re-verification.
- **Contacts:** during the wait, contacts see a "this contact is recovering
  their account" warning. Once the wait is over, it is the contacts who
  find the recovered ring again:
  - Every device follows all the contact heads it has pinned in the log at
    least once a day; it sees the recovery head without waiting for a send.
  - A contact that sees the recovery head leaves an offer, encrypted to the
    head's KEM key, in a **find-again mailbox** named
    `HKDF(recovery head digest, "mailbox verify again" ‖ day)`. The new
    device listens to this mailbox for 30 days.
  - Others who know the head (such as a blocked contact) can also leave an
    offer in this mailbox. So the recovered ring treats every offer like a
    new pairing: the verification words are **compared again** (bound to
    the recovery record digest), a new `k_AB` is set up, and the contact is
    given a name again. When the time runs out, there is no automatic
    acceptance. Because the words only prove that the channel reaches the
    device displaying them (a thief's device can display words too), the
    contacts' half of P7 rests on the contact **recognising the other person
    out of band** (face to face or by voice); user-facing prompts state
    this plainly.
- **Remaining risk:** if the phone and the phrase are stolen together, this
  layer does not protect.

## Sending

1. The sender leaves an **offer** in the contact's mailbox.
   - The offer is encrypted with HPKE to the ring's X-Wing key (derived
     from `r_e`).
   - The device that sends the offer signs it with its `IK` and adds a
     Merkle proof against the sending ring's current head. The signature
     explicitly binds: the identities of both rings (the shared contact
     context), the sender's head, the HPKE output, and the receiver's head
     as the sender knows it (head echo).
   - The offer holds only the rendezvous information, not the file key.
2. An online device in the receiving ring opens the offer. It starts the
   existing Noise + ML-KEM handshake
   ([protocol notes](protocol-v0.1-notes.md), "Section 6: connections and
   the handshake").
3. Both sides prove that they are current members of their own ring:
   - the single-use endpoint key is signed with `IK`;
   - the signature covers the Noise handshake hash, the role and the
     identities of both rings;
   - a Merkle proof is added to the signature.
   These bindings prevent unknown key-share and replay attacks. The Noise
   pre-shared key (psk) is derived from the two rings' shared secret
   instead of the code. The exact definition will be written into the
   protocol notes.
4. The session keys come from ephemeral keys. Even if the ring secret or a
   device key leaks later, a recorded transfer cannot be decrypted.

## Against abuse

- **ARC tokens** (Anonymous Rate-limited Credentials, an IETF Privacy Pass
  draft): from a credential obtained once, a fixed number of tokens that
  cannot be linked to each other are generated. So a quota per credential
  can be set without a client identity.
  - The issuer and the verifier are the same Worker.
  - A Rust implementation is needed on the client side too.
- **Obtaining a credential:** needs a proof of work of a few seconds, and
  there is a limit per IP.
- **Resistance to fake clients (Sybil)** rests on the proof of work and the
  IP limit. These are weak against botnets and can strain users behind
  operator NAT (CGNAT). Play Integrity is not used: it does not exist on
  Linux and breaks privacy.

## Notifications (Android)

- By default, UnifiedPush and a WebSocket connection while the app is open
  are used.
- **Optional FCM wake-up:** only a content-free "wake up" signal goes to
  Google. No mailbox name or identity goes.
- The wake-up token is derived separately for each mailbox and blinded.
  No table linking mailboxes and devices forms on the server.

## Mathematics

| Topic | Value |
| --- | --- |
| Recovery phrase | 2048²⁴ = 2²⁶⁴, 256 bits after the checksum; ~2¹²⁸ under Grover |
| Mailbox name collision | N²/2¹²⁹ ≈ 5·10⁻¹⁹ for N ≈ 1.9·10¹⁰ names/year |
| Guessing a mailbox name | ~2⁻¹⁰² per try with ~5·10⁷ live names |
| Contact pairing | Three-word code: 2⁻³³ per online try (CPace, no offline tries). Verification words: 2⁻⁴⁴, but only if the user compares them |
| Fingerprint | 8 words, 88 bits; imitating a single target ~2⁸⁸ operations, any one of N contacts ~2⁸⁸/N |
| Shamir 2-of-3 | If each share holder can be reached with probability 0.9, the recovery probability is 0.972. 0.991 for 3-of-5. t−1 shares give no information at all about the secret |
| Waiting (assuming honest delivery) | Probability of going unnoticed P = Π(1 − p_i)^T. Two devices, p = 0.9, T = 3 days: 10⁻⁶. One device, p = 0.5, T = 7 days: 0.0078. The assurance against a malicious server comes not from this formula but from the log and the contacts' re-verification |
| Epoch change size | Always 15 × (1120 + 48) ≈ 17.5 KB of HPKE (padded); a record ~23 KB |
| Yearly traffic | At least 52 epochs × ~23 KB ≈ 1.2 MB, per ring |
| Why not MLS | At n ≤ 16, TreeKEM's O(log n) advantage (4 against 15) is meaningless; MLS's complexity is large |

## Security properties

Every property is written into the [threat model](tehdit-modeli.md) and
tied to a test or to a lemma in the formal model.

| No | Property |
| --- | --- |
| P1 | **Membership integrity:** only an existing device, or (delayed) `RK`, adds records to the chain |
| P2 | **Rollback and fork detection:** every valid continuation of an epoch appears in the log; more than one is a fork. The witnesses prevent two different views from being shown |
| P3 | **Post-removal secrecy and authentication:** at most `2 hours + 2 * Delta` (2 hours 10 minutes for `Delta <= 5 minutes`) after the removal record is reported to the witnesses, the removed device cannot read the content sent and cannot send files in the ring's name, even working with the server. This is provided by enforcing inclusion and the freshness check when sending and receiving. If the witnesses work with the server, this assurance does not hold |
| P4 | **Forward secrecy:** transfer keys are ephemeral |
| P5 | **No account table on the server:** accounts, device counts and contact relations cannot be extracted from the stored data without the keys. The IP and timing of live connections are out of this scope |
| P6 | **Least information towards contacts:** the ring head does not show the number of devices. In which epoch the membership changed is visible |
| P7 | **No silent takeover:** if only the phrase is stolen, a takeover is noticed on the devices or by the contacts. If the phone and the phrase are stolen together, this assurance does not hold |
| P8 | **Post-quantum secrecy:** key distribution and transfer are done with X-Wing or ML-KEM |

## Remaining risks (accepted)

- **IP and timing:** Cloudflare sees the IP at connection time. Devices and
  mailboxes connecting from the same NAT can be linked. OHTTP (through a
  separate organisation's relay) and Tor, which reduce this, come in a
  later stage. The texts on the site and in the stores do not say "the
  server knows nothing"; they say "the server keeps no account".
- **The moment of obtaining an ARC credential:** the issuer sees the IP.
  Behind a home NAT, the anonymity set is a single household.
- **What contacts see:** a contact sees in which epoch the membership
  changed, and over time can count the devices it sees in handshakes.
  Anyone who knows a head (a blocked contact or a removed device too) can
  follow the later heads in the log: they see the times of membership
  changes, the operation type and the expiry dates, not the content or the
  number of devices. Cutting this needs per-contact head delivery or a
  chain renewed on blocking; left for a later design stage.
- **Writing garbage into the log:** someone who knows an epoch's log key (a
  contact or a removed device) can slow readers down by writing many
  invalid values under that key. They cannot lock the epoch; ARC makes it
  expensive.
- **The phrase together with an old removed device:** this is in the same
  class as "the phone and the phrase stolen together". It can open a fork
  and choose its own branch with `RK`; contacts are not moved unless they
  re-verify out of band.
- **Competing `recovery_add` or `resolve_fork` denial of service by a phrase
  thief:** someone who stole only the recovery phrase cannot silently take
  over the ring (P7), but can log a competing `recovery_add` against the
  same membership period (stalling both records as a real fork in default
  mode, or occupying strict mode's first-writer-wins queue `H("log prop", m)`
  until an honest member voids it in time) or a second `resolve_fork`
  against the same forked head (stalling resolution under the
  single-resolution rule). When no honest device can clear the queue or
  resolve the stall, contacts pair again out of band.
- **Strict-mode veto denial of service:** a device stolen unlocked in strict
  mode can veto single-signature `AK` proposals and submit earlier proposals
  in the period queue that an honest member must reject within 72 hours
  before `rk_remove` can win. This delays removal (loss of liveness) while
  preserving security as long as an honest member comes online within the
  wait.
- **Traffic for following contacts:** every device follows the contact
  heads in the log once a day. The log server and the witnesses can link
  these queries by IP; the queries are paged and at random times, and
  OHTTP comes in a later stage.
- **A device stolen unlocked:** before it is removed, it can add a device
  with `AK` or try to remove the other devices. The other devices get a
  notification; mutual removal is a real fork and is resolved with `RK`.
- **Witnesses:** if enough witnesses work with the server, P2 and P3 do not
  hold and records can be censored. The witnesses are chosen from
  different organisations. Enforcing inclusion is not in the standard
  witness protocol; TotalDrop needs to add it to the witness software.
- **Stolen together:** if the phone and the phrase are stolen together,
  there is no protection.
- **Stopping with a late fork:** a removed device can, by writing a
  conflicting record under an old head's key, leave the contacts that still
  pin that head failing closed. The ring's own devices and up-to-date
  contacts are not affected. `resolve_fork` with `RK`, or pairing with the
  contact again, solves it. For a ring without a recovery phrase, this
  means pairing again.
- **What the witnesses see:** because every record goes directly to all the
  witnesses, a witness can, over time, link a ring's connection IP and its
  weekly pattern; it cannot see the content or the contacts. OHTTP reduces
  this and comes in a later stage.
- **Live takeover:** the live takeover of an unlocked device in active use
  is out of scope.
- **Library maturity:** the post-quantum Rust libraries are pre-1.0 and
  have not been independently audited. The hybrid construction keeps the
  classical assurance. The algorithms can be replaced behind the version
  field. The X-Wing test vectors are pinned to a single draft version and
  put into the repository.

## Acceptance criteria (for the code and the release)

- **K1. Imitating a malicious server.** The test server can do anything.
  Even so:
  - it cannot add a device to the list;
  - it cannot bring back a removed device;
  - it cannot show two contacts two different lists, or if it does, it is
    noticed.
- **K2. After removal.** At most `2 hours + 2 * Delta` (2 hours 10 minutes
  for `Delta <= 5 minutes`) after the removal record is reported to the
  witnesses, the removed device cannot open the files sent and cannot send
  files in the ring's name, even if the server helps it and censors the
  honest record (while the witnesses are honest).
- **K3. Recovery.** Someone who knows the recovery phrase cannot add a
  device without the devices and the contacts noticing, and cannot learn
  the contact list without waiting. The test exercises the local clock, the
  waiting period, rejects and re-verification.
- **K3b. Strict mode.** A contact rejects a single-signature `add` or
  `remove` proposal before 72 hours of witness time have passed, or after a
  timely targeted `reject_pending` has entered a witnessed root before the
  proposal settled; rejects an `rk_remove(x)` if any current member other
  than `x` submitted a timely `reject_pending` (while ignoring `x`'s own
  reject); enforces first-writer-wins single change per period (ignoring
  late rejects and later competing proposals once an earlier live proposal
  settles); and requires two distinct members' `AK` signatures to disable
  strict mode (`policy`) or rotate the phrase (`change_phrase`) in strict
  mode.
- **K3c. Witnesses.** A made-up record report cannot stop the log; a record
  that the server dropped enters the root that contacts accept within 1
  hour at the latest, or the root is not signed.
- **K4. Leaked database.** Even if the whole server database leaks, that
  two mailbox names belong to the same account cannot be found without the
  keys.
- **K5. Formal model.** The protocol is modelled in Tamarin (`formal/`,
  pieces A1–A4c) proving P1, P2, P3, and P7 with broken-model mutation tests
  (including the "bad server + old authority" pattern; P4, P5, P6, and P8
  rest on the underlying primitives and stage-C protocol notes), and the
  waiting periods and clock-skew bounds are exhaustively model-checked in
  TLA+ (`formal/GhostRingTiming.tla`, piece A5).
- **K6. Fuzz.** There are fuzz targets for the chain, head, offer and log
  proof parsers, and for the fork merge rule.

## Rules to settle in the formal model

In four review rounds, every patch opened a new corner case in the
interaction of the fork and recovery rules. All four open questions have
now been settled by the Tamarin models (A1–A4c) and the TLA+ timing model
(A5) in [formal/](../formal/README.md) (#251):

- **Late fork in a ring without a recovery phrase (settled — A4c, fork
  rule 6):** no safer automatic rule exists. Ignoring late values would
  allow an unlocked stolen device that writes first while the owner is
  offline to hijack the ring permanently. Late forks fail closed
  (`late_fork_never_hijacks_contact`), and contacts still pinned on the old
  head pair again out of band (or follow `resolve_fork` when the ring has a
  recovery phrase).
- **Strict-mode escape when a stolen device vetoes its removal (settled —
  A4b, `hayalet-halka-sikimod.spthy`):** the owner writes `rk_remove(x)`, a
  pending proposal in `H("log prop", m)` signed by both the active `RK` and
  a staying member's `AK` that waits 72 hours of witness time. `x`'s own
  `reject_pending` does not count against it (and never revokes `RK`,
  because `reject_pending` is separate from `recovery_reject`), while any
  other current member's timely `reject_pending` voids it. Requiring the
  staying member's `AK` ensures the phrase alone can never remove a device
  (`rk_remove_needs_staying_device`).
- **Ordering of pending single-signature records (settled — A4a/A4b/A5,
  fork rule 5):** proposals are logged against the membership period `m`
  (unchanged by `rotate`) rather than racing with `L_{e+1}` — under
  `H("log recovery proposal", m)` in default mode and in one unified queue
  `H("log prop", m)` in strict mode. In strict mode, **one change per
  period, first writer wins**: a reader follows a proposal only once it has
  waited 72 hours of witness time, has no counted targeted reject
  (`rejEntry < propEntry + 72h` in witness time), and every earlier
  proposal against `m` was voided by a counted reject (`no_split_change`,
  `earlier_proposal_must_be_rejected`). Late rejects entering at or after
  settling are ignored uniformly by all readers (`GhostRingTiming.tla`,
  `SafetyLateRejectDoesNotSplit`).
- **Priority (rule 1), recovery proposals, and `resolve_fork` together
  provide P7 (settled — A4a/A4b/A4c):** in default mode, any `AK` record
  from any head of period `m` (including a freshness-independent
  `recovery_reject`) beats `recovery_add` (`ak_record_stops_recovery`),
  while contacts require out-of-band human re-verification bound to the
  record digest (`contact_not_moved_by_phrase_alone`). For forks,
  `resolve_fork` requires the active `RK` (old phrases rotated out by
  `change_phrase` or revoked by an undisputed `recovery_reject` are
  rejected: `old_phrase_cannot_resolve_after_change_phrase`,
  `revoked_rk_cannot_resolve`), a validly logged member branch
  (`resolved_branch_has_valid_lineage`), single resolution per forked head
  (`no_split_fork_resolution`), settling, and out-of-band contact
  re-verification (`resolve_fork_requires_contact_reverification`). The
  remaining phrase-only threat is denial of service via competing
  `recovery_add`/`resolve_fork` records, not silent takeover.

## Order of verification and implementation

1. **This document and [decision 0013](kararlar/0013-hayalet-halka-hesap.md)**
   (no code).
2. **Tamarin model** and an independent human cryptography review. The
   review also covers the choice of libraries.
3. **Protocol notes:** byte formats and test vectors.
4. **Core:** `totaldrop-crypto` and `totaldrop-core`. With known-answer
   vectors, property tests and fuzz targets.
5. **Server:** the mailbox, log and ARC Workers. Tested by imitating a
   malicious server.
6. **Apps:** Linux (KDE, Hyprland, Omarchy) and Android. Texts in 35
   languages.
7. **Privacy policy:** updated in 35 languages before the feature is
   released.

## Sources

- **MLS Virtual Clients** (a related design, hides the number of devices):
  https://datatracker.ietf.org/doc/draft-ietf-mls-virtual-clients/
- **ARC:** https://datatracker.ietf.org/doc/draft-yun-privacypass-arc/
- **Parakeet** (WhatsApp key transparency): https://eprint.iacr.org/2023/081
- **ELEKTRA** (multi-device key transparency): https://julialen.github.io/elektra_ccs23.pdf
- **Signal "bad device"** (the lesson of weak device registration): https://eprint.iacr.org/2021/626
- **Analysis of WhatsApp multi-device groups:** https://eprint.iacr.org/2025/794
- **Earlier examples:**
  - Keybase per-user key: renewed on device revocation, encrypted
    separately to each device.
  - The Apple iCloud Keychain circle.
  - Social recovery in smart contract wallets (guardians + a time lock).
