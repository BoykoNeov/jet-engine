---
name: rung27-no-freeze-out
description: "SHIPPED rung 27 = NO freeze-out; the frozen-NO assumption is DERIVED (Da_NO≪1 from entry everywhere), kill test INVERTS rung 26 (both terms agree)"
metadata: 
  node_type: memory
  type: project
  originSessionId: 1885c151-c894-4a03-bec2-a5a7b27ad619
---

SHIPPED rung 27 (jet-engine project) = **NO freeze-out**: `NOFreezeOut(L,…)` /
`Gas.no_freeze_out_nozzle(…)`, a pure diagnostic beside the cycle (bit-for-bit rung 6).

Applies rung-26's anchored-clock / local-`Da` machinery to **exhaust NO** via a
`_tau_no_destroy` clock built from **rung 7's OWN Zeldovich reverse rates** (`NO+O`, `NO+H`
— zero new constants, already K-checked). Question: is the frozen-NO assumption every NO
number has carried since rung 7 (and the rung-14/17 clamp corollary reads OFF) actually
EARNED?

**Finding — it IS earned.** `Da_NO ≪ 1` from ENTRY at every `Tt4` (3–9 orders clear;
frozen-from-entry *everywhere*, unlike rung-26's major pool which is frozen only lean), on an
**upper bound** (radical-rich frozen station-4 pool = fastest possible relaxation). Robust to
the NO level (the super-eq clock `τ_NO = 1/(2 c_tot(k2r x_O + k3r x_H))` is `[NO]_e`- and
`a`-independent).

**The kill test INVERTS rung 26's.** This clock is Arrhenius (`θ≈20820/24560 K` ⇒ `k` craters
on cooling) AND bimolecular (`c¹`), so its two factors AGREE — both DRIVE freezing. Rung 26's
recombination clock (`Ea=0`, termolecular `c²`) had them OPPOSE (density won DESPITE `k`
rising). Same nozzle, two anchored clocks, opposite mechanism structure: NO freezes BECAUSE of
temperature, the majors freeze DESPITE it.

**A CONFIRMATION rung** (quieter than rungs 24/26's inversion/negative) that retires the
clamp corollary's last premise. **No moving freeze point** — rung-26's headline has no
analogue (`s_freeze_NO≡0`); the honest trend is the Da_NO-vs-Da_recomb **separation narrowing**
with `Tt4` (3.7e7→2.2e3, no crossing claimed).

Reduce: `Da_NO≡0` ⇒ the rung-14/17 clamp `max_a` bit-for-bit. `tests/test_rung27.py` green on
gates 1–10. Spec `docs/rung27-spec.md`; anchor `docs/plans/rung27-anchor-no-freeze-out.md`
(anchor doc committed b59a319 before the build, per numbers-before-code). Probe lives in
`M:\claud_projects\temp\rung27-anchor\` (framing-B v2 = the shipped clock).

**Open (kept additive):** the rung-26-coupled march (NO riding the *relaxing* pool — can only
push deeper into frozen; secondary refinement). See [[rung26-freeze-out]], [[rung25-finite-rate-nozzle]].
