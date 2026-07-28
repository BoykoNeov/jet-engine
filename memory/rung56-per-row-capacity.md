---
name: rung56-per-row-capacity
description: "SHIPPED rung 56 = per-row capacity; the seam asked for a C per row and the design ladder supplied all but one; binding row MIGRATES; machine's two binding rows differ by END and SPOOL; corrects rung 54"
metadata: 
  node_type: memory
  type: project
  originSessionId: 13218776-b14c-4c30-9012-efe363614892
  modified: 2026-07-28T08:17:54.453Z
---

**SHIPPED rung 56 (2026-07-28) = PER-ROW CAPACITY** — rung 54's throat channel resolved onto
rung 55's stage stack. Discharges rung 55's named seam. `docs/rung56-spec.md`.

**The seam asked for the wrong thing.** Rung 55 wrote *"it needs a `C` per row"* — read as `K`
new constants. It needs ONE: at design `φ_k`=1 ⇒ `Vx_k`=`U_k`, so on a constant mean radius every
row has the same design throat VELOCITY while `Tt_k` climbs the ladder, and the total-referenced
Mach `ν = M/√(1+(γ−1)/2·M²)` scales as `ν_k = ν_1/√θ_k,d`. **Profile DERIVED, level DISCLOSED** —
rung 54's own pattern. Lesson worth carrying: *when a seam names a cost in constants, check
whether the existing construction already fixes the shape.*

**The derived profile FIGHTS the seam**, which is what made it a real prediction rather than an
inspection: the rear rows come out designed with MORE capacity exactly where the off-design march
loads them hardest. Result — the binding row **MIGRATES** (front near design, rear at part power,
one-way, all five shapes, both splits). The seam's "binds at the back" is HIT at part power and
**REFUTED near design**, for a derived reason.

**Headline (stated precisely after an advisor catch):** the machine's two binding rows are
different rows — different END *and* different SPOOL (machine-wide minima: incidence LP-front
`M_i`=0.349, capacity HP-rear `M_c`=0.164). Per spool the ends agree; the cross-spool half is a
separate comparison. The first draft over-claimed "two worst rows at diagonal corners" as if the
ends carried the spool distinction — see [[refuse-vote-counting-across-currencies]] for the same
class of error.

**Corrects rung 54** (rung-28 shape): its *"the HP never approaches its throat at any throttle"*
is true at the FACE and nearly false at the rear row — the face RELAXES with throttle while the
rear row TIGHTENS, `C*`=0.913 at `Tt4`=800. Banner added to `docs/rung54-spec.md`.

**The non-tautology gate is a RESOLUTION gap, not a feedback one** (advisor's framing): the
channel enters no solver, so unlike rung 55's `work_gap` there is no feedback leg. The content is
face-margin vs binding-row margin at the same solved state — exactly `1.0` at `K`=1.

**Two predictions scored honestly against:** P3's "increments at least halve" MISSED on the HP
(~0.53); P6's "throat debit tracks rung 55's `dN` ratio within 25 %" was **REFUTED** — and the
refutation is the rung's fourth law: the speed ratio is `v`-invariant while the throat ratio
collapses, so **a LEVER'S COST is coordinate-dependent too** (after rung 53's margin, rung 54's
constraint severity, rung 55's lever benefit). Also recorded in the anchor: a P4 clause I wrote
that contradicted P1's own content, and a gate that over-reached its prediction's band.

**Next seam: PER-ROW BLADING** — rungs 55/56's stages share ONE map; needs a `ψ_k`/`T_c,k` law,
and unlike the capacity profile the ladder does NOT supply it.

Related: [[rung55-stage-stack]], [[rung54-stator-throat]], [[rung53-variable-stator]].
