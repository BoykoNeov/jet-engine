---
name: rust-port-slice-af-step4
description: "Slice AF step 4 — two doc-comment claims written as mechanism that nothing had measured, one of them false; and a shipped refusal message four formatting divergences wide that no step could see until a reader read it"
metadata:
  node_type: memory
  type: project
---

Slice AF step 4 (rung 74, `DemandCoordinateTransient`). Plan § 5.30.4. `_coord_march`,
`_demand_gains_at` and all six readers — `demand_law`, `demand_gains`, `latch_discriminator`,
`windup_law`, `flat_schedule_identity`, `forcing_openloop`. See [[rust-port-status]],
[[rust-port-slice-af-step3]], [[instrument-fed-by-what-it-certifies]].

**THE LESSON: *is that measured?* has to be asked of the sentences that say a thing MATTERS, not
only of the ones that say it cannot be seen.** [[rust-port-slice-af-step3]] recorded a booked
BLINDNESS that turned out to be false. This step shipped the mirror image twice in one file: two
doc comments asserting that a difference was LIVE — *carrying the sibling's spelling here would
admit a point straddling the kink*, and *the omission is the reason this test has to set the law on
the reader's own machine*. The first is false (the conjunct reads a class attribute that is
`"max"` on every path into the body, so adding it back changes nothing); the second invents a
causal link to a test that never goes through the function. **A sentence saying *this matters*
reads like diligence, so nothing prompts a second look at it** — a claimed blind spot at least
invites the question.

**How to apply it:** what caught both was writing the mutation sweep, because a predicted SURVIVE
needs a proof and neither claim could supply one. So: type a mutation for every sentence in a doc
comment that asserts a difference is observable, not just for the code. If you cannot name the
mutation, you cannot name the mechanism either.

## A shipped refusal message that was four formatting divergences wide

`windup_law` catches an `AssertionError` and records `str(exc)[:240]`, which made the TEXT of the
joint-IC refusal a COMPARED VALUE for the first time in the crate. Step 3's port did not match
Python, in four places inside those 240 characters: Rust's `{:.3e}` writes `2.898e-3` where
CPython writes `2.898e-03` (the exponent is always signed and padded to two digits), and Rust's
`{:?}` on a `&str` writes double quotes where Python's `!r` writes single ones. **43 sites in 14
`src` files format a float that way inside a message**, and one file already recorded the
divergence as known-and-unmatched — which was true only while nothing read one. Rung 74's is
repaired through two helpers; the other 42 are BOOKED, not swept.

**The general form: a divergence that is inert because nothing reads it is not fixed, it is
DEFERRED — so the moment a reader arrives, re-audit the class rather than the one site.**

## The measurement that opened the step, and why it went first

Every one of the six readers folds over a FILTERED subset and Python's `max(…, default=None)`
returns `None` on an empty one. **An `Option` that is `None` on both sides agrees perfectly and
measures nothing** — [[rust-port-slice-t-step1]]'s exact-zero class in `Option` clothing. So all
six were driven in Python and every subset counted BEFORE a line of Rust: 3 569 leaf keys, of
which exactly **four** are `None`, all of them the same key on the two arms where the plant never
accelerates, with fourteen `Some` siblings to discriminate against. No fold in the step is taken
over an empty set on any shipped grid.

## The branch every shipped caller misses, named before it could read as coverage

`_demand_gains_at` picks its state pair with a KEY-PRESENCE test, and the only shipped caller
hands it CLIP points — so the PROJECTION arm is live and the direct arm is reached by nothing.
Removing the direct arm entirely SURVIVES the whole 3 731-key diff. **A port that always projected
would have passed every measurement this step makes**, which is why the arm is written and its
deadness is a booked measurement rather than a silence.

## And a defect in the sweep's own scorer, found by re-reading it rather than by a failure

`drive_and_diff` returned the same failure tuple for *the mutant compiled and its values moved*
and for *the mutant did not compile*, and the scorer wrote `KILL` for both. **A mutation that does
not build then CONFIRMS a predicted KILL without ever having been tested** — step 1's *a mutation
that does not apply looks like a finding*, in the quieter direction where it looks like a
confirmation. The log records each build's exit code, so the two are separable after the fact and
every KILL was re-audited against it.
