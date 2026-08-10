---
name: rung79-gap-margin
description: "Rung 79 § 9's constant-gap seam, checked — the § 5 march STANDS STILL; a limiter armed at the initial operating point has no transient, and the rig placement a constrained Jacobian requires is what voids the march"
metadata: 
  node_type: memory
  type: project
  originSessionId: f6cdade6-e7eb-4c87-93df-91219b539161
  modified: 2026-08-10T10:08:50.861Z
---

Checked rung 79 § 9's "constant gap" seam (2026-08-10). Doc: `docs/rung79-gap-margin.md`,
gates in `tests/test_rung79.py`. A CORRECTION to a shipped rung, not rung 80.

**The finding, and it was not the assigned question.** `coord_march`'s march never leaves its
initial state — `nu_lp`/`nu_hp` spread **exactly 0.0** over 341 steps; only the COMMAND ramps.
The rig arms three φ floors on one wall (`φ_lim = 0.80`) and the **stator** lifts the free start
(`φ_lp = 0.7731`) exactly onto it, so rung 49's fuel leg reads a state already ON its floor and
its cap IS the flowing fuel. **A limiter armed with zero initial margin has no transient.**
Rung 78's march stands still identically. Positive control: at `φ_lim ≤ 0.75` the same rig
accelerates 180 K.

**Why the rig is like that, and why it must not be "fixed".** `PHI_JAC = 0.80` sitting ON the
wall is what §§ 1–4's **constrained linearisation** requires — a Jacobian must be taken with
every loop on its constraint. The same placement buys §§ 1–4 and silently voids § 5. The
transferable trap: **a rig tuned for a frozen-state reader can be structurally incapable of the
trajectory reader in the same section**, and nothing in either section notices.

**Lessons worth more than the physics:**

* **Pick the DISCRIMINATOR before the sweep.** Sweeping `margin` first would have confirmed
  `AccelSchedule.cap`'s own source line. Printing `κ_ss` over the traversed band cost seconds and
  killed the seam's premise: `κ` cannot drift because the plant never moves.
* **A counter is only as good as the NOUN it counts.** The shipped `n_distinct > 10` guard was
  read as refuting "one state logged 1366 times" — it counts distinct **floats** (a solve whose
  START POINT moves), never distinct **states**. The FOURTH vacuity trap, inside the guard § 8.1
  was proudest of. See [[rung79-state-coordinate]] and [[rung78-residual-gauge]].
* **Register the vacuity condition of the PLANT separately from the INSTRUMENT's.** § 8.1's
  "registering the vacuity condition beats registering the result" is true and insufficient: a
  guard on the reading cannot see a plant that never produced one.
* **The EVALUATION POINT is the measurement.** `gap+1 = a_cap/mf`, and
  `d ln(gap+1)/d ln(1+margin) = 1/(1−c)` — rung 77's stiffness ([[rung77-stiffness-ledger]]),
  because `a_cap` is a fixed point. Read at the fixed point the two instruments agree to
  **5e−6**; read at the plant's fuel (12.6% away in `w`) they miss by 1.6e−3. **That gradient IS
  the non-vacuity control** — an instrument agreeing at both points would be measuring nothing.
* Deliverable shape: `docs/` correction + spec edits + **gates**, never a re-rig. Gate the
  standstill so an edit that unpins it must RESCORE § 5; the fingerprint arms are bit-exact
  ([[golden-gate-slice4]]).
* CLAUDE.md's line was paid for by compression, not a budget bump — 2 B headroom left
  ([[claude-md-is-a-reference]]).
