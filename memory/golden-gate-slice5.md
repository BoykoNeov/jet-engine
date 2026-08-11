---
name: golden-gate-slice5
description: "Fingerprint slice 5 (rung 80) — a finite difference inherits its drift from the quantity DIFFERENCED, so a relative band misprices it in both directions at once; and a per-arm 'measured X' comment names the keys ON that leg, not the arm's worst"
metadata: 
  node_type: memory
  type: project
  originSessionId: e512adb7-2864-4199-8658-ae080a228275
  modified: 2026-08-11T11:28:23.303Z
---

Fingerprint slice 5 shipped 2026-08-11 (commit `49e292c`): ONE arm, `r80`, four readers
(`split_arrest` leading, `split_liveness`, `split_gains` on the clip AND demand walls,
`split_saturation`), 1,092 values, golden 39→40 kernels, 21,297→22,389. It pays the only debt
this project had written down — `docs/rung80-spec.md` § 10 asked for the arm AND named the reader
it should lead with. No settings of its own (inherits rung 79's, which inherit 78's, which inherit
77's), so it stays differenceable against slices 3 and 4. See [[golden-gate-slice4]],
[[golden-gate-slice3]], [[golden-fingerprint-gate]], [[rung80-split-wall]].

**THE FINDING, and it is a tolerance lesson not a rung one.** 37 of 1,092 values differed
CPython-vs-PyPy and every one is a `.c0`/`.c1` finite-difference cross-gain. They sort by
**MAGNITUDE, not by name**: the drift is inherited from the size of the quantity being
*differenced*, not from the gain's own size, so it sits FLAT at ~1e-11 across a family spanning
fifteen decades (1.3e-11 → 1.5e4). A relative band therefore misprices the family in both
directions at once — sizing it for the small gains needs 1e-3, which hangs a thousandth-scale
band on the large ones that currently reproduce to 1e-16. Thirteen decades of blindness bought to
fix a mispricing. So the arm is compared on the ABSOLUTE leg (1e-9), with `TOL = 1e-15` as
r72/r73's declared backstop that no key rides.

**Re-earn an inherited constant on the new arm's own numbers.** 1e-9 was already in `ABS_TOL` five
times, and taking it on family resemblance is how a band stops being measured. Both halves of the
two-sided rule were re-derived here: ≥11.8× above this arm's measured 8.5e-11, AND 3.06 decades
below the LIVE scale of the same cross-gain family — so a near-dead gain waking to its siblings'
1e-6 still fails.

**The superlative that needed checking (advisor's catch), and what checking it taught.** The claim
"first arm priced on an absolute leg for a reason other than a structural zero" rests on what the
OTHER five arms' legs are for — a claim about code I had not read. It VERIFIED: all 34 of r75's
absolute-leg riders are `det0` structural zeros (golden ~1e-11, relative errors 1.6–3.6), and the
same holds for r70/r72/r73/r74/r76. Sharpened rather than dropped: prior legs certify *still zero*;
this one is SIZED BY A LIVE value — and it carries one near-dead cell too, which is the old reason,
now disclosed.

**The reading trap found on the way, worth knowing before sizing any future band:** a per-arm
`# measured X on <keys>` comment means *the worst among keys that RIDE the absolute leg*, NOT the
arm's worst absolute drift. On r75 those differ by 8.7× — riders peak at 1.759e-10 (the recorded
1.8e-10), while the arm's global worst absolute error is **1.566e-9**, above its own 1e-9 band and
passing only because the relative leg covers it (8.4e-13 on a value of magnitude 1871). The
convention is coherent and r80's entry follows it; the trap is differencing a number written under
that convention against a globally-measured one.

**The anti-vacuity rule here is deliberately WEAK, and the disclosure matters more than the rule.**
Rung 80's `demand` shared-wall arm has an empty interior-cell list ON PURPOSE (that is rung 74's
finding), so the new `cells#n` guard only demands that ONE list be non-empty. What actually makes
this arm impossible to pass while measuring nothing is the four DISCRETE readings — `four_live`=33,
`first_arrest`=0.7732, `ever_two_authorities`=False, `first_sat`=0.855 — compared exactly, none of
which a dead plant produces. Say that where the weak rule is, or a future reader reads the weakness
as a hole (rung 77's dead closure returning a perfect `1.000e+00` is the failure they will suspect).

**How to apply:** (a) before pricing a band, ask what KIND of quantity carries the drift — a
difference, a determinant, a residual — because that decides the CURRENCY before any number does;
(b) never difference a carried-forward gate timing against a fresh one — the 12:08-at-1314 vs
~10:05-at-1313 move is ≤60.6 s attributable and the rest is box variance, the same error `c89387d`
already had to correct once in this module (see [[never-run-the-gate-for-timing]]); (c) the arm
costs 60.6 s on PyPy, heaviest in the module by 3×, ~8.7 min under CPython — the whole-suite 6.2×
PyPy factor UNDERSTATES tight numeric marches, where the JIT wins hardest.
