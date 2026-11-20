\* SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
\* SPDX-License-Identifier: GPL-3.0-or-later

--------------------------- MODULE GhostRingTiming ---------------------------
(*
 * Ghost ring (docs/hesap.md, decision 0013), part A5 of the formal model
 * (#251): bounded timing, clock-skew, settling, and freshness evaluation.
 *
 * The Tamarin models (A1-A4c) abstract time to ordering events (Old,
 * Settled) and assume honest records enter the log on time. This TLA+
 * specification checks the concrete numeric inequalities under bounded
 * witness inclusion (T_INCLUSION), root freshness (T_ROOT_FRESH), and
 * bounded clock skew (+/- DELTA) with monotonically non-decreasing witness
 * root timestamps.
 *
 * To keep exhaustive TLC exploration fast, Init picks one of three disjoint
 * timing scenarios ("proposal", "removal", "rotate"):
 *   1. "proposal": witness-time settling (T_SETTLE) for recovery_add,
 *      strict-mode proposals, rk_remove, change_phrase, and resolve_fork;
 *      timely-reject guarantee; witness-time late-reject determinism (no
 *      split between early and late readers).
 *   2. "removal": post-removal secrecy/freshness bound (P3): a contact
 *      holding a pre-removal root cannot pass its freshness check after
 *      T_INCLUSION + T_ROOT_FRESH + 2 * DELTA.
 *   3. "rotate": A3/A3b rotate settling (D_ROTATE) and rotating-device
 *      follow margin (D_ROTATE_DEVICE): a remove checked against a
 *      pre-rotate root (D_w = 0) enters before the rotate settles, and no
 *      contact still sends to an unsettled pre-rotate root once the
 *      rotating device follows the rotate.
 *)

EXTENDS Integers

CONSTANTS
    T_INCLUSION,       \* Max real time from witness submission to root inclusion (real: 1h)
    T_ROOT_FRESH,      \* Max root age accepted by a reader/writer (real: 1h)
    DELTA,             \* Max clock skew between real time and witness/local clocks (real: <= 5m)
    T_SETTLE,          \* Witness-time proposal wait (real: 72h or 7d)
    D_ROTATE,          \* Witness-time rotate settling delay for contacts (real: 4h)
    D_ROTATE_DEVICE,   \* Witness-time delay before a ring device follows a rotate (real: 6h)
    MAX_TIME           \* Finite real-time horizon for exhaustive model checking

ASSUME
    /\ T_INCLUSION \in Nat /\ T_INCLUSION > 0
    /\ T_ROOT_FRESH \in Nat /\ T_ROOT_FRESH > 0
    /\ DELTA \in Nat /\ DELTA > 0
    /\ T_SETTLE \in Nat /\ T_SETTLE > T_INCLUSION + 2 * DELTA
    /\ D_ROTATE \in Nat /\ D_ROTATE > T_INCLUSION + T_ROOT_FRESH + 2 * DELTA
    /\ D_ROTATE_DEVICE \in Nat /\ D_ROTATE_DEVICE >= D_ROTATE + T_ROOT_FRESH + 2 * DELTA
    /\ MAX_TIME \in Nat /\ MAX_TIME >= D_ROTATE_DEVICE + 2 * DELTA
    /\ MAX_TIME >= T_SETTLE + 2 * DELTA

NONE == -1
RealTimes == 0..MAX_TIME
WitnessTimes == 0..(MAX_TIME + DELTA)
OptReal == {NONE} \cup RealTimes
OptWitness == {NONE} \cup WitnessTimes
Decisions == {"none", "accepted", "rejected"}
Scenarios == {"proposal", "removal", "rotate"}

Max(a, b) == IF a >= b THEN a ELSE b

VARIABLES
    scenario,             \* Active disjoint scenario
    now,                  \* Ground-truth real time
    wTime,                \* Latest monotonically non-decreasing witness root timestamp
    \* Scenario 1 ("proposal"):
    propEntry,            \* Witness timestamp when proposal entered a root
    rejSubmitted,         \* Real time when a member submitted a targeted reject
    rejEntry,             \* Witness timestamp when the reject entered a root
    readerA,              \* First reader's decision
    readerB,              \* Second reader's decision
    acceptTimeA,          \* Witness timestamp when readerA accepted
    acceptTimeB,          \* Witness timestamp when readerB accepted
    \* Scenario 2 ("removal"):
    remSubmitted,         \* Real time when remove(x) was submitted to witnesses
    remEntryReal,         \* Real time when remove(x) entered a witnessed root
    remEntryW,            \* Witness timestamp when remove(x) entered a root
    preRootW,             \* Witness timestamp of a pre-removal root held by a contact
    staleActionTime,      \* Latest real time a contact passed freshness with preRootW
    \* Scenario 3 ("rotate"):
    rotEntryW,            \* Witness timestamp when rotate entered a root
    preRotRootW,          \* Witness timestamp of a root issued before rotate entered (D_w = 0)
    writeRemReal,         \* Real time an honest writer submitted remove after checking preRotRootW
    remRotEntryW,         \* Witness timestamp when that remove entered a root
    unsettledRootW,       \* Witness timestamp of a root where rotate is still unsettled
    devFollowReal,        \* Real time when the rotating device followed the rotate
    staleSendAfterFollow  \* TRUE if a contact passed freshness on an unsettled root at/after devFollowReal

vars == <<
    scenario, now, wTime,
    propEntry, rejSubmitted, rejEntry, readerA, readerB, acceptTimeA, acceptTimeB,
    remSubmitted, remEntryReal, remEntryW, preRootW, staleActionTime,
    rotEntryW, preRotRootW, writeRemReal, remRotEntryW, unsettledRootW,
    devFollowReal, staleSendAfterFollow
>>

proposalVars == <<propEntry, rejSubmitted, rejEntry, readerA, readerB, acceptTimeA, acceptTimeB>>
removalVars == <<remSubmitted, remEntryReal, remEntryW, preRootW, staleActionTime>>
rotateVars == <<rotEntryW, preRotRootW, writeRemReal, remRotEntryW, unsettledRootW, devFollowReal, staleSendAfterFollow>>

TypeOK ==
    /\ scenario \in Scenarios
    /\ now \in RealTimes
    /\ wTime \in WitnessTimes
    /\ wTime >= now - DELTA
    /\ wTime <= now + DELTA
    /\ propEntry \in OptWitness
    /\ rejSubmitted \in OptReal
    /\ rejEntry \in OptWitness
    /\ readerA \in Decisions
    /\ readerB \in Decisions
    /\ acceptTimeA \in OptWitness
    /\ acceptTimeB \in OptWitness
    /\ remSubmitted \in OptReal
    /\ remEntryReal \in OptReal
    /\ remEntryW \in OptWitness
    /\ preRootW \in OptWitness
    /\ staleActionTime \in OptReal
    /\ rotEntryW \in OptWitness
    /\ preRotRootW \in OptWitness
    /\ writeRemReal \in OptReal
    /\ remRotEntryW \in OptWitness
    /\ unsettledRootW \in OptWitness
    /\ devFollowReal \in OptReal
    /\ staleSendAfterFollow \in BOOLEAN

Init ==
    /\ scenario \in Scenarios
    /\ now = 0
    /\ wTime \in 0..DELTA
    /\ propEntry = NONE
    /\ rejSubmitted = NONE
    /\ rejEntry = NONE
    /\ readerA = "none"
    /\ readerB = "none"
    /\ acceptTimeA = NONE
    /\ acceptTimeB = NONE
    /\ remSubmitted = NONE
    /\ remEntryReal = NONE
    /\ remEntryW = NONE
    /\ preRootW = NONE
    /\ staleActionTime = NONE
    /\ rotEntryW = NONE
    /\ preRotRootW = NONE
    /\ writeRemReal = NONE
    /\ remRotEntryW = NONE
    /\ unsettledRootW = NONE
    /\ devFollowReal = NONE
    /\ staleSendAfterFollow = FALSE

\* Real time advances by 1 tick. Witness inclusion enforcement prevents real
\* time from moving past submission + T_INCLUSION while a submitted record is
\* still missing from witnessed roots.
Tick ==
    /\ now < MAX_TIME
    /\ (scenario = "proposal" /\ rejSubmitted /= NONE /\ rejEntry = NONE) =>
           now < rejSubmitted + T_INCLUSION
    /\ (scenario = "removal" /\ remSubmitted /= NONE /\ remEntryReal = NONE) =>
           now < remSubmitted + T_INCLUSION
    /\ (scenario = "rotate" /\ writeRemReal /= NONE /\ remRotEntryW = NONE) =>
           now < writeRemReal + T_INCLUSION
    /\ now' = now + 1
    /\ wTime' \in Max(wTime, (now + 1) - DELTA)..((now + 1) + DELTA)
    /\ UNCHANGED <<scenario, proposalVars, removalVars, rotateVars>>

\* --- Scenario 1: Proposal settling and targeted reject determinism -----

PropEnter ==
    /\ scenario = "proposal"
    /\ propEntry = NONE
    /\ \E w \in wTime..(now + DELTA) :
           /\ wTime' = w
           /\ propEntry' = w
    /\ UNCHANGED <<scenario, now, rejSubmitted, rejEntry, readerA, readerB,
                   acceptTimeA, acceptTimeB, removalVars, rotateVars>>

SubmitReject ==
    /\ scenario = "proposal"
    /\ propEntry /= NONE
    /\ rejSubmitted = NONE
    /\ rejSubmitted' = now
    /\ UNCHANGED <<scenario, now, wTime, propEntry, rejEntry, readerA, readerB,
                   acceptTimeA, acceptTimeB, removalVars, rotateVars>>

RejectEnter ==
    /\ scenario = "proposal"
    /\ rejSubmitted /= NONE
    /\ rejEntry = NONE
    /\ now <= rejSubmitted + T_INCLUSION
    /\ \E w \in wTime..(now + DELTA) :
           /\ wTime' = w
           /\ rejEntry' = w
    /\ UNCHANGED <<scenario, now, propEntry, rejSubmitted, readerA, readerB,
                   acceptTimeA, acceptTimeB, removalVars, rotateVars>>

ReaderAAccept ==
    /\ scenario = "proposal"
    /\ propEntry /= NONE
    /\ readerA = "none"
    /\ wTime >= propEntry + T_SETTLE
    /\ ~(rejEntry /= NONE /\ rejEntry < propEntry + T_SETTLE)
    /\ readerA' = "accepted"
    /\ acceptTimeA' = wTime
    /\ UNCHANGED <<scenario, now, wTime, propEntry, rejSubmitted, rejEntry,
                   readerB, acceptTimeB, removalVars, rotateVars>>

ReaderAReject ==
    /\ scenario = "proposal"
    /\ propEntry /= NONE
    /\ readerA = "none"
    /\ rejEntry /= NONE
    /\ rejEntry < propEntry + T_SETTLE
    /\ readerA' = "rejected"
    /\ UNCHANGED <<scenario, now, wTime, propEntry, rejSubmitted, rejEntry,
                   readerB, acceptTimeA, acceptTimeB, removalVars, rotateVars>>

ReaderBAccept ==
    /\ scenario = "proposal"
    /\ propEntry /= NONE
    /\ readerB = "none"
    /\ wTime >= propEntry + T_SETTLE
    /\ ~(rejEntry /= NONE /\ rejEntry < propEntry + T_SETTLE)
    /\ readerB' = "accepted"
    /\ acceptTimeB' = wTime
    /\ UNCHANGED <<scenario, now, wTime, propEntry, rejSubmitted, rejEntry,
                   readerA, acceptTimeA, removalVars, rotateVars>>

ReaderBReject ==
    /\ scenario = "proposal"
    /\ propEntry /= NONE
    /\ readerB = "none"
    /\ rejEntry /= NONE
    /\ rejEntry < propEntry + T_SETTLE
    /\ readerB' = "rejected"
    /\ UNCHANGED <<scenario, now, wTime, propEntry, rejSubmitted, rejEntry,
                   readerA, acceptTimeA, acceptTimeB, removalVars, rotateVars>>

\* --- Scenario 2: Post-removal freshness window (P3) --------------------

SubmitRemoval ==
    /\ scenario = "removal"
    /\ remSubmitted = NONE
    /\ remSubmitted' = now
    /\ UNCHANGED <<scenario, now, wTime, proposalVars,
                   remEntryReal, remEntryW, preRootW, staleActionTime, rotateVars>>

IssuePreRoot ==
    /\ scenario = "removal"
    /\ remEntryReal = NONE
    /\ \E w \in wTime..(now + DELTA) :
           /\ wTime' = w
           /\ preRootW' = w
    /\ UNCHANGED <<scenario, now, proposalVars,
                   remSubmitted, remEntryReal, remEntryW, staleActionTime, rotateVars>>

RemovalEnter ==
    /\ scenario = "removal"
    /\ remSubmitted /= NONE
    /\ remEntryReal = NONE
    /\ now <= remSubmitted + T_INCLUSION
    /\ \E w \in wTime..(now + DELTA) :
           /\ wTime' = w
           /\ remEntryReal' = now
           /\ remEntryW' = w
    /\ UNCHANGED <<scenario, now, proposalVars,
                   remSubmitted, preRootW, staleActionTime, rotateVars>>

ContactUsePreRoot ==
    /\ scenario = "removal"
    /\ preRootW /= NONE
    /\ \E c \in (now - DELTA)..(now + DELTA) :
           c - preRootW <= T_ROOT_FRESH
    /\ staleActionTime' = now
    /\ UNCHANGED <<scenario, now, wTime, proposalVars,
                   remSubmitted, remEntryReal, remEntryW, preRootW, rotateVars>>

\* --- Scenario 3: Rotate settling (D_ROTATE) and device follow margin ---

IssuePreRotRoot ==
    /\ scenario = "rotate"
    /\ rotEntryW = NONE
    /\ \E w \in wTime..(now + DELTA) :
           /\ wTime' = w
           /\ preRotRootW' = w
    /\ UNCHANGED <<scenario, now, proposalVars, removalVars,
                   rotEntryW, writeRemReal, remRotEntryW, unsettledRootW,
                   devFollowReal, staleSendAfterFollow>>

RotateEnter ==
    /\ scenario = "rotate"
    /\ rotEntryW = NONE
    /\ \E w \in wTime..(now + DELTA) :
           /\ wTime' = w
           /\ rotEntryW' = w
    /\ UNCHANGED <<scenario, now, proposalVars, removalVars,
                   preRotRootW, writeRemReal, remRotEntryW, unsettledRootW,
                   devFollowReal, staleSendAfterFollow>>

\* An honest writer checks preRotRootW (D_w = 0: issued before rotate entered)
\* as fresh on its local clock and submits an AK remove.
WriterSubmitRemove ==
    /\ scenario = "rotate"
    /\ preRotRootW /= NONE
    /\ rotEntryW /= NONE
    /\ writeRemReal = NONE
    /\ \E c \in (now - DELTA)..(now + DELTA) :
           c - preRotRootW <= T_ROOT_FRESH
    /\ writeRemReal' = now
    /\ UNCHANGED <<scenario, now, wTime, proposalVars, removalVars,
                   rotEntryW, preRotRootW, remRotEntryW, unsettledRootW,
                   devFollowReal, staleSendAfterFollow>>

WriterRemoveEnter ==
    /\ scenario = "rotate"
    /\ writeRemReal /= NONE
    /\ remRotEntryW = NONE
    /\ now <= writeRemReal + T_INCLUSION
    /\ \E w \in wTime..(now + DELTA) :
           /\ wTime' = w
           /\ remRotEntryW' = w
    /\ UNCHANGED <<scenario, now, proposalVars, removalVars,
                   rotEntryW, preRotRootW, writeRemReal, unsettledRootW,
                   devFollowReal, staleSendAfterFollow>>

\* Witnesses issue a root while the rotate is still unsettled (< rotEntryW + D_ROTATE).
IssueUnsettledRoot ==
    /\ scenario = "rotate"
    /\ rotEntryW /= NONE
    /\ \E w \in wTime..(now + DELTA) :
           /\ w < rotEntryW + D_ROTATE
           /\ wTime' = w
           /\ unsettledRootW' = w
    /\ UNCHANGED <<scenario, now, proposalVars, removalVars,
                   rotEntryW, preRotRootW, writeRemReal, remRotEntryW,
                   devFollowReal, staleSendAfterFollow>>

\* The rotating device follows its own rotate only once wTime >= rotEntryW + D_ROTATE_DEVICE.
DeviceFollowRotate ==
    /\ scenario = "rotate"
    /\ rotEntryW /= NONE
    /\ devFollowReal = NONE
    /\ wTime >= rotEntryW + D_ROTATE_DEVICE
    /\ devFollowReal' = now
    /\ UNCHANGED <<scenario, now, wTime, proposalVars, removalVars,
                   rotEntryW, preRotRootW, writeRemReal, remRotEntryW,
                   unsettledRootW, staleSendAfterFollow>>

\* After the rotating device follows the rotate, check if any contact with a
\* skewed local clock can still treat an unsettled root (preRotRootW or
\* unsettledRootW) as fresh and send to the pre-rotate head.
ContactStaleSend ==
    /\ scenario = "rotate"
    /\ devFollowReal /= NONE
    /\ ~staleSendAfterFollow
    /\ \E rootW \in {preRotRootW, unsettledRootW} :
           /\ rootW /= NONE
           /\ \E c \in (now - DELTA)..(now + DELTA) :
                  c - rootW <= T_ROOT_FRESH
    /\ staleSendAfterFollow' = TRUE
    /\ UNCHANGED <<scenario, now, wTime, proposalVars, removalVars,
                   rotEntryW, preRotRootW, writeRemReal, remRotEntryW,
                   unsettledRootW, devFollowReal>>

Next ==
    \/ Tick
    \/ PropEnter
    \/ SubmitReject
    \/ RejectEnter
    \/ ReaderAAccept
    \/ ReaderAReject
    \/ ReaderBAccept
    \/ ReaderBReject
    \/ SubmitRemoval
    \/ IssuePreRoot
    \/ RemovalEnter
    \/ ContactUsePreRoot
    \/ IssuePreRotRoot
    \/ RotateEnter
    \/ WriterSubmitRemove
    \/ WriterRemoveEnter
    \/ IssueUnsettledRoot
    \/ DeviceFollowRotate
    \/ ContactStaleSend

Spec == Init /\ [][Next]_vars

\* --- Safety Invariants -------------------------------------------------

\* No reader accepts a proposal before T_SETTLE of witness time has elapsed.
SafetyNoAcceptBeforeSettle ==
    scenario = "proposal" =>
        /\ (readerA = "accepted" => propEntry /= NONE /\ acceptTimeA >= propEntry + T_SETTLE)
        /\ (readerB = "accepted" => propEntry /= NONE /\ acceptTimeB >= propEntry + T_SETTLE)

\* Any targeted reject entering before propEntry + T_SETTLE prevents acceptance.
SafetyRejectBeforeSettleVoids ==
    (scenario = "proposal" /\ propEntry /= NONE /\ rejEntry /= NONE /\ rejEntry < propEntry + T_SETTLE) =>
        /\ readerA /= "accepted"
        /\ readerB /= "accepted"

\* Early and late readers never split on a proposal (late rejects are ignored by witness time).
SafetyLateRejectDoesNotSplit ==
    scenario = "proposal" =>
        /\ ~(readerA = "accepted" /\ readerB = "rejected")
        /\ ~(readerA = "rejected" /\ readerB = "accepted")

\* A reject submitted strictly more than T_INCLUSION + DELTA before
\* propEntry + T_SETTLE is guaranteed to enter before the settling deadline.
SafetyTimelyRejectGuaranteed ==
    (scenario = "proposal" /\ propEntry /= NONE /\ rejSubmitted /= NONE /\ rejEntry /= NONE
        /\ rejSubmitted + T_INCLUSION + DELTA < propEntry + T_SETTLE) =>
        rejEntry < propEntry + T_SETTLE

\* Post-removal freshness bound (P3): no contact uses a pre-removal root past
\* remSubmitted + T_INCLUSION + T_ROOT_FRESH + 2 * DELTA.
SafetyPostRemovalWindowBounded ==
    (scenario = "removal" /\ remSubmitted /= NONE /\ staleActionTime /= NONE) =>
        staleActionTime <= remSubmitted + T_INCLUSION + T_ROOT_FRESH + 2 * DELTA

\* A3 rotate settling: an AK remove checked against a pre-rotate root (D_w = 0)
\* enters a witnessed root strictly before the rotate settles.
SafetyRemoveEntersBeforeRotateSettles ==
    (scenario = "rotate" /\ rotEntryW /= NONE /\ remRotEntryW /= NONE) =>
        remRotEntryW < rotEntryW + D_ROTATE

\* A3b device follow margin: once the rotating device follows its rotate, no
\* contact still treats an unsettled-rotate root as fresh.
SafetyNoStaleSendAfterDeviceFollowsRotate ==
    scenario = "rotate" => ~staleSendAfterFollow

=============================================================================
