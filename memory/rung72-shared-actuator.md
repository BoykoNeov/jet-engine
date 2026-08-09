---
name: rung72-shared-actuator
description: Rung 72 — two loops on ONE actuator; min-select masks a leg so ONE plant IS rungs 68/69/70/71 by AUTHORITY; closed a seam by refuting its premise; three predictions died of one oversight
metadata: 
  node_type: memory
  type: project
  originSessionId: 63348b1b-2010-4972-adee-1420b3254bc5
  modified: 2026-08-09T13:08:29.195Z
---

Rung 72 (shipped 2026-08-09) armed rung 52's φ fuel leg BESIDE rung 47's `Tt4` governor — two
limiters on ONE actuator, the `n`=4 seam rungs 70/71 both named and both asserted against.

**The finding.** Min-select makes authority EXCLUSIVE, so `max(gf,gr)` is *flat* in the masked
clip: its column is `(−1,0,0,0)`, the block is triangular, and the six-state plant IS rung 68,
69, 70 or 71 at every instant — polynomial for polynomial, to 7.1e−17 — plus a free pole at the
masked leg's own clock. `zeros = n_live − m_live`, counting loops that hold AUTHORITY. The rank
changes at the hand-over **with no state, gain or clock moving**. Rung 66's mirror: one VARIABLE
⇒ `pair = 1` (redundant); one ACTUATOR ⇒ `pair_FR = 0` (exclusive).

**The first seam closed by REFUTING its premise.** `(4,m)` is unreachable via a shared actuator
— `n_live` is always 3 — so rung 71's "only unoccupied shape" loses one of its two routes.
Rung 69 § 11's (a fourth LP lever) stays open; don't conflate the two shapes.

**Three predictions died of ONE oversight, and the oversight was the content.** P1, P5 and the
"derived, not scored" D5 all assumed the GOVERNOR holds authority throughout. It doesn't — the
fuel leg holds it over the early part of every joint window, which is exactly why there are four
cells and not two. *Authority is not a static property of a build, and I reasoned as if it were.*
See [[rung63-fuel-bleed]] on checking a quoted number was taken at this rung's settings.

**Two instruments nearly shipped broken, both silently.**
1. `_charpoly4`'s Faddeev–LeVerrier recursion used `A` where it needs `M_{k−1}`. It returned a
   WRONG polynomial with entirely plausible downstream numbers — stable-looking roots, a
   determinant of 5.9e+05, a root residual of 1e−09 (the root finder faithfully solving the
   wrong polynomial). Nothing downstream could tell. Fix: gate a new numerical instrument
   against an INDEPENDENT trace/cofactor determinant and a triangular matrix, and rebuild the
   bug in the test so the self-test has actually failed once.
2. The MAX-vs-SUM discriminator agreed with itself at matched clocks: at `τ_f = τ_g` the SUM law
   has `(1,−1,0,0)` as an exact eigenvector with eigenvalue `−1/τ`, so the free-pole test passes
   under BOTH laws. It separates them only at UNMATCHED clocks (1e−15 vs 3.6e−1). Same family as
   [[rung66-two-lag-cascade]] and [[rung70-generic-split]].

**Compare POLYNOMIALS, not roots, when a root is repeated.** The rung-68 cell (rank one) showed a
4.6e−7 "disagreement" that was √(machine precision) resolving a double zero root — the two
readers' base points agreed to 0.0 exactly. Coefficient by coefficient the same cell reads
3.6e−17. Diagnose an outlier before tabulating it. Related: [[golden-fingerprint-gate]].

**"Derived, not scored" is not "exempt from correction."** D5 sat in the anchor's derivation
section and was measured false; it is scored in spec § 9 anyway, and the anchor is NOT edited
(rung 70's precedent).

**The gated quantity is the mask leak (`== 0.0` exactly), not the free pole.** `_jac4` puts
`−1/τ_i` on the diagonal by construction, so the pole follows algebraically from the measured
zeros — gating it would be the instrument agreeing with itself ([[rung67-cascade-a]] gate 9's
retraction, third shape). Sharpest new trap: a fuel leg that lost its `_b_state`/`_v_state`
boundary returns `F_q = F_v = 0` and its row looks exactly like a MASKED one — the rung would
confirm its own headline through a bug. Both legs are asserted against both blind versions.

**Next seam, named as sharpest:** an APPLIED-fuel-referenced leg. The whole triangular structure
rests on both inherited legs computing at the SCHEDULED fuel (rungs 47/52's own discipline);
reference one to the applied fuel and `F_r ≠ 0`, the block form is gone, and `n_live` might
reach 4 after all.
