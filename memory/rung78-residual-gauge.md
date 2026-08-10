---
name: rung78-residual-gauge
description: "Rung 78 — a residual's SLOPE is a GAUGE but its root's UNIQUENESS is not; three vacuity traps in one section, caught only by counting"
metadata: 
  node_type: memory
  type: project
  originSessionId: 43aa3272-cde6-4e56-a40e-e9c9fccb57ff
  modified: 2026-08-10T04:46:57.737Z
---

Rung 78 (THE RESIDUAL GAUGE) closes [[rung77-stiffness-ledger]] § 9's "a cap whose `c` approaches
1" seam **by refuting its premise**. Anchor the accel leg's cap at its own root and scale the
departure — `cap_k(w) = w0 + k·(cap(w) − w0)` — and `cap_k(w0) = w0` identically, so the set point
is invariant *by algebra* while `G_k′ = 1 − k·c` is a free dial reaching zero and going negative.

**HEADLINE: a residual's SLOPE is a GAUGE; its root's UNIQUENESS is not.** `G_w` spans `+1.5 … −2.0`
matching `1 − k·c` to 1.18e−08, and `dw*/dq` does not move (7.44e−08) — `G_q` carries the same
vanishing factor, so rung 77 § 3's `c → 1` route is a **removable** singularity. But the gauge
destroys uniqueness: a second root collides with the true one exactly at `k·c = 1` (root counts
{1,2,4}, band [0.9,1.2]), and inside it a solver converges **cleanly onto the wrong root**.
**What `c → 1` costs is well-posedness, not sensitivity.** Rung 76 survives because `solve` →
`sensed` MOVES the root (2.8e−02 … 1.4e−01): a rewriting that moves the root is a device, one
that preserves it is a coordinate.

**The methodological lesson, and it is the reusable part: § 5 was written wrong three times and
every failure looked exactly like a pass.**

1. **Wrong coordinate** — marched in `clip`, where the ladder dispatches out before `_cap_fuel`
   is reached. Four gauges, four bit-identical trajectories, branch never executed.
2. **The instrument was itself vacuous** — the fix was a hit counter written `self._x += 1`,
   which creates an *instance* attribute and leaves the class attribute the reader queries at
   zero forever.
3. **Masking** — with both fixed, `hits = 1366` but `binds = 0`: the cap was computed 1366 times
   a march and discarded every time. "The branch ran" and "its value reached the plant" are
   different claims.

**Why:** a bit-identical result is the signature of both a perfect invariance and a measurement
that never happened, and prose inspection cannot tell them apart. Only **counting** did.

**How to apply:** when a section's result is "nothing moved", require a counter proving (a) the
new code path executed and (b) its output was *consumed* — then gate both. § 5 is scored **NOT
ESTABLISHED** rather than passed, and the test asserts `binds == 0` so that if a future edit makes
the leg bind, the gate fails and forces the spec to be rewritten as a result.

Two more things worth carrying: the advisor blocked a `band = 0.30` exclusion I had shipped —
the anchor registered `1e−3`, so a round number covering the measurement is a tuned pass wearing
a pre-registered threshold's clothes; the exclusion is now *measured* (excluded iff the residual
is multi-rooted there), which makes §§ 1 and 3 interlock. And `ok == True` is not a correctness
guard for a root finder — the guard is `|w − w0|/w0`, because silent wrong-root convergence is
worse than divergence. Three of six pre-registered predictions were refuted and all three
refutations are the rung's content. See [[rung76-fuel-dependent-cap]].
