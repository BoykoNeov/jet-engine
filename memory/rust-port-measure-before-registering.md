---
name: rust-port-measure-before-registering
description: "Pre-register AFTER the measurements the decisions depend on, not before — and a slack tolerance bar in the source is a place to look for a wrong exactness claim, not a bar to transcribe"
metadata: 
  node_type: memory
  type: project
  originSessionId: a910b66c-05e8-4ffd-9c82-b2f101e29a98
  modified: 2026-08-12T14:00:35.120Z
---

Phase 3 slice E of the Rust port (rungs 14/17 — the nozzle strand, 2026-08-12) closed phase 3 at
100% bit-equality again (511/511 vs PyPy), so once more the bit-count was not where the value was.

**The pre-registration ritual changed shape, and this is the durable part.** Slices A–D wrote the
pre-registration first and measured after. Slice E ran four probes on the Python BEFORE writing a
word of the plan section, and **all four moved a gate that would otherwise have been transcribed
from the source unchecked.** Registering a decision you have not measured is registering a guess
with extra ceremony. The order that works: orient → probe → pre-register → dump → port → gate.

**A SLACK BAR IN THE SOURCE IS A LEAD, NOT A NUMBER TO COPY.** The source gated a reduce it called
"EXACT" at a tolerance six orders looser than the quantity it measures. Probing it found the claim
is true in algebra and false in arithmetic — and, more usefully, that the residual SPLITS: making
the solver's stopping rule a knob showed the rule contributes a factor of a few and the FLOOR is
that two code paths add the same numbers in different orders. That split is only visible if you
parameterise something the source hard-codes. Third claim of exactness this port has corrected
(see [[rust-port-inside-outside-exactness]] for the other two, which had a different mechanism —
summation shape rather than two routes).

**THE VACUITY RULE MUST BE RE-ASKED AFTER THE DESIGN IS FIXED.** The pre-registration confidently
said the new closure-based test was "strictly stronger" than the Python's monkey-patch version. It
was not: the same refactor that removed the global mutation also removed the branch the test was
checking, so the transcription compared a function to itself. **The rule ("what could this test
still FAIL for?") was applied to the source's design and never re-applied to mine.** The plan
records the correction inline rather than silently fixing it. See [[rust-port-ported-test-vacuity]].

**A DEFERRAL IS A CLAIM ABOUT A DEPENDENCY, AND IT CAN BE WRONG.** An earlier slice parked three
gates behind "the nozzle strand and the PDF family". One of them needed a three-line accessor with
no nozzle code in it and had been portable for two slices. The "don't ship a test whose subject is
absent" rule is right; its failure mode is the inverse — a portable gate parked behind an unrelated
dependency that nothing re-checks. Same family as [[rung80-split-wall]]'s wrong-noun seam.

Two things that held: a guard the source calls unreachable turned out **reachable**, so it is gated
from both sides at both design points — the best case of a family this port keeps meeting (an
earlier slice's guard was genuinely dormant and needed a second operating point; another's could
not fire at all and had to be labelled a tripwire). And sweeping past the source's own gate paid
for the **sixth** consecutive slice — this time locating a band edge the
source states and never measures, and finding that the quantity the rung's headline is defined on
goes dormant while the ladder it describes does not.

**POST-SHIP REVIEW (same day) — A BIT-EQUALITY GATE IS BLIND TO AN ASSUMPTION BOTH SIDES SHARE.**
The slice shipped before its closing review because the advisor was overloaded twice; retried
later, it found the gap. To make the sweep affordable the oracle hoists one expensive call out of
a loop, arguing it cannot depend on the swept knobs. **That argument was never measured, and it is
the one kind of error 100 % bit-equality cannot catch** — the Rust gate hoists it too, so a wrong
hoist bakes the same stale value into BOTH references and the gate passes with both wrong. Ask, of
any shortcut: *would the reference and the implementation be wrong TOGETHER?* If yes, no amount of
agreement between them is evidence.

**And the obvious check for such a hoist is VACUOUS** — re-calling the hoisted function inside the
loop compares a pure function to itself, because the reason it was hoistable is that it does not
take the loop's arguments. The check that bites is against a **DIFFERENT ROUTE** that does take
them (here: the full closure, which builds its own copy internally), plus an assertion that the
compared states are DISTINCT so the check cannot go quietly vacuous. That is
[[rust-port-ported-test-vacuity]]'s case #8 arriving in the FIX rather than in the test.

**A shipped REASON can be wrong while the shipped CHOICE is right, and checking it PAYS.** A guard
census ran on one branch only, documented as avoiding a second guard on the other. Measured: the
second guard is unreachable — the solver's own bracket keeps every evaluation inside the valid
range — so both branches fire the same guard at the same place. The choice was harmless, the
reason false, and correcting it turned a one-branch assumption into a two-branch measurement. Same
family as [[rung84-staircase-law]] and [[rung28-coupled-no-march]]: confirm the verdict, correct
the reason. **Retry a post-completion review that failed to run; it is not optional ceremony.**

Related: [[rust-port-decided]], [[rust-port-location-keys-refute]], [[rust-port-shape-keys]],
[[golden-fingerprint-gate]].
