# Rung 79 — THE STATE COORDINATE

Rung 78 § 9's fourth seam. Anchor + scoring: `docs/plans/rung79-anchor-state-coordinate.md`,
§ 8 below. Gates: `tests/test_rung79.py`. Code: `StateCoordinateTransient` in `turbojet/engine.py`.

Rung 78 re-wrote a leg's **LAW** and found the root preserved and its **UNIQUENESS DESTROYED**.
This rung re-writes a leg's **STATE COORDINATE** — rung 60's incidence `M_i` in place of rung 49's
`φ` — and asks the same question of the other side.

> **HEADLINE: A COORDINATE IS A GAUGE ON THE SET POINT AND UNREACHABLE ON THE PLANT — because the
> branch that makes a leg AUTHORITATIVE is the branch that SUBSTITUTES THE ORIGINAL COORDINATE
> BACK IN.** The two sets `{the knob is live}` and `{the leg reaches applied fuel}` are **DISJOINT
> BY CONSTRUCTION**, and the disjointness is an identity of `_cap_free`'s branch condition, not a
> measurement (§ 5.3: 3 / 1363 / **0** of 1366).
>
> **BOUNDS rung 78** — uniqueness survives *this* gauge because the multiplier is strictly
> positive; rung 78's loss belonged to its **affine law-side family**, whose `1 − k·c` passes
> through zero. **It does NOT correct rung 78**, whose statement was true of the family it was
> made about.

---

## 0. WHAT MOVED, AND WHAT DID NOT

The **SEVENTH declared knob**, beside `_share_law` (72), `_ref_law` (73), `_lag_coord` (74),
`_windup_law` (75), `_cap_law` (76) and `_gauge_k` (78):

    _phi_ref in {"phi", "incidence"}          "phi" == rung 78, by exact dispatch

**ZERO new constants**, and the THIRD knob in the family to add none. `T_c = tan_beta1_crit() =
1/φ_surge` is rung 53's; `m_lim` is rung 49's own `φ_lim` read through rung 60's shipped
`from_phi`. Both cancel (§ 1).

### 0.1 `incidence × _gauge_k ≠ 1` is REFUSED

The two knobs re-write **different legs'** residuals, so composing them is neither rung 78 nor
rung 79 and nothing measures it. Asserted by name — rung 78 § 0.3's refusal of `sensed × gauge`,
one knob over.

### 0.2 The settings are rung 78's, taken verbatim

`phi_lim = 0.80`, `margin = 0.10`, `taus = (0.05,)*4`, `r = 0.5`, `s_settle = 1.2`, `ds = 0.005`,
`v_max = 0.20`, `inc = False`, coordinate `demand`. Rung 63's lesson: a number quoted from another
rung's settings is not a comparison. `1/φ_lim² = 1.5625` exactly.

---

## 1. THE DERIVATION — AND IT IS AN IDENTITY, SO IT CANNOT FAIL

**This section is declared UNSCORED, in advance, in the anchor.** It is four lines of algebra. It
is *run* because an unconfirmed identity is an **unrun code path**, which is a different thing
worth knowing — but a measurement disagreeing here would be a **BUG, not a finding**, and a scored
block full of confirmations is the shape of the failure this project has recorded four times
(rung 73's perfect confirmation having measured nothing; rung 77's `1.000e+00`; rung 75's blind
instrument; rung 78 § 5.1's three traps).

    Gi(w) = m_lim − M_i(w)
          = [T_c − 1/φ_lim + v] − [T_c − 1/φ(w) + v]
          = 1/φ(w) − 1/φ_lim                                  ← T_c AND v CANCEL IDENTICALLY
          = Gs(w) · h(w) ,        h(w) := 1/(φ(w)·φ_lim) > 0   STRICTLY

The blade metal and the stator setting are both **absent** from the incidence residual. Rung 60
chose `M_i` *because* the stator does not move its wall; in the residual the stator does not
appear at all.

| | claim | measured |
|---|---|---|
| **D1** | root SET preserved **pointwise** (`sign Gi ≡ sign Gs`), so equal counts on any window | counts **equal** at 10/10, `n_roots = [1]`, located roots agree to `0.000e+00` (§ 4) |
| **D2** | slope at the root scales by exactly `1/φ_lim²` — a **derived** factor, no fitted content | `1.5625` to **`4.33e−09`** over 10 points |
| **D3** | `dw*/dq` **invariant** (both halves carry the same `h(w*)`) | **`0.000e+00`** — not small, *exactly* zero |
| **D4** | monotonicity survives, so the **shipped** bracket applies in both coordinates | 2 732 `_cap_free` calls, **zero** failures; no damped Newton anywhere in this rung |

### 1.1 The structural point, and it is why this BOUNDS rung 78

Rung 78's gauge had a **free dial** `k`, and its slope factor `1 − k·c` **passes through zero** —
which is exactly how it destroyed uniqueness. This gauge has **NO DIAL AT ALL**, and the reason is
the cancellation above: **there is nothing left to sweep.** "Cannot be driven singular" is not
*we swept and never reached zero*; it is *there is nothing to sweep*, and the multiplier is
positive because **a coordinate must be invertible**. That is the missing hypothesis under which
rung 78's uniqueness survives.

D4 is the same statement in the solver: rung 78 had to write a damped Newton **because** its
family could invert `G`'s sign. **The root-finder requirement is itself the diagnostic separating
a law-side family from a coordinate.**

---

## 2–4. THE READERS

`coord_scan` (§§ 1–3), `coord_census` (§ 4), `coord_march` (§ 5), `coord_forced` (§ 5.2).

**§ 4's instrument has an inherited POSITIVE CONTROL, and that is what makes `[1]` a measurement.**
`_root_count` is rung 78 § 3's own walk, and there it **found** a second root sweeping in and
colliding at `k·c = 1`. An instrument that has detected the thing it is now reporting absent is
not a blind one. `n_roots = [1]` is also the first time any rung has counted the **φ leg's** roots.

---

## 5. THE PLANT — AND THE KNOB CANNOT REACH IT

`hits = 1366`, `binds = 1366` (the φ leg wins the inner `min(accel, φ)` at **every** call),
`flips = 0`, trajectory `worst = 0.000e+00`, `sched_moved = 0.000e+00`.

**And almost none of that means what it looks like.**

> **⚠ CORRECTED AFTER SHIP — § 9's gap seam, checked: `docs/rung79-gap-margin.md`.**
> **This march never leaves its initial state.** `nu_lp` and `nu_hp` have **exactly 0.0**
> relative spread over all 341 steps; only `s` and `mf_sched` (1.478×) move. The rig arms three
> φ floors at one wall and the **stator** lifts the free initial point (`φ_lp = 0.7731`) onto it
> **exactly**, so rung 49's leg binds at `s = 0` with no authority left — *a limiter armed with
> zero initial margin has no transient*. The "1366 calls across the accel" below are **1366 calls
> at ONE operating point**, and every § 5 number phrased as being *across the accel* must be read
> that way. **§ 5.1–5.3 survive intact** (they are identities of `_cap_free`'s branch condition,
> which a static plant satisfies as surely as a moving one); § 5.5 gains a **fourth** entry.
> The rig is **not** changed — `PHI_JAC = 0.80` on the wall is what §§ 1–4's constrained
> linearisation requires (that doc § 2.3). Rung 78's march stands still identically.

### 5.1 `d_set = 0.0` IS EVIDENCE ABOUT `_surge_fuel`, NOT ABOUT COORDINATES

`_cap_free` short-circuits to `shipped()` whenever `G(mf_sched) > 0`, i.e. whenever the leg
**BINDS**. Measured: **fallback at 1363 of 1366** calls per coordinate; the coordinated residual
was bracketed **3** times.

And `_surge_fuel` is **not coordinate-free**. It brackets its **own hardcoded** `G(w) =
surge.phi_lim − _instant_fuel(…)[k]` (`engine.py` ~4760) — that is `Gs`, **written in the very
coordinate the knob replaces**. So on a binding leg this knob is not *inert*, it is
**UNREACHABLE**: the plumbing routes past it and substitutes the original coordinate back in.

The plant's `d_set = 0.0` at 1366 of 1366 therefore says nothing about coordinate invariance.
**The invariance claim rests entirely on § 5.2.**

### 5.2 THE SHORT-CIRCUIT DOES NOT CREATE THE INVARIANCE; IT SHARPENS IT

`coord_forced` bypasses the short-circuit and brackets the coordinated residual in **both**
regimes. Binding at 10 of 10 points, and:

| | measured |
|---|---|
| `d_forced` max / median | **`6.14e−15` / `7.64e−16`** — float noise, 3–27 ulp |
| same float | 1 of 10 |
| **forced vs SHIPPED solve** | **`0.000e+00`** — the bypass reproduces `_surge_fuel` exactly |

That last row is what licenses the bypass: it is measuring the **same** set point, not a nearby
one. So the coordinate is root-preserving to ~1e−15, and the short-circuit rounds that to exactly
zero. **An exact invariance and an unmeasured one are indistinguishable in the plant, and they
differ by 1e−15 here.**

### 5.3 THE COMPLEMENTARITY — AND IT IS AN IDENTITY, NOT A MEASUREMENT

`_cap_free` short-circuits **iff** `G(mf_sched) > 0` **iff** the cap lies **below** the schedule —
which is exactly when the cap survives `_applied_demand`'s `min(mf_sched, wf, wr)` and reaches
applied fuel. It brackets the coordinated residual **iff** the cap lies at or **above** the
schedule — which is exactly when `_applied_demand` throws the cap away.

    {the knob is live}  ∩  {the leg reaches applied fuel}  =  ∅          BY CONSTRUCTION

Counted rather than asserted: **`n_live = 3`, `n_reach = 1363`, `n_both = 0`**, and `3 + 1363 =
1366` — the two sets **partition** the calls.

**So the knob is unreachable in both regimes, for two different reasons, and there is no third
regime.** This is a *third* masking mechanism in the ladder, distinct from rung 72's `min`-mask
and rung 76's law-vs-plant split — and unlike those it is a consequence of **where a solver
short-circuits**, which no rung had looked at.

### 5.4 `binds` IS NOT THE LAST SELECTOR, AND P3'S WORDING ASSUMED IT WAS

`binds` counts the `min(accel, φ)` **inside** `_cap_fuel`. `_applied_demand` is a **second**
selector further down. A cap can win the inner min at all 1366 calls and still be masked below —
so `binds = 1366/1366` is, by itself, consistent with total masking. That is rung 78 § 5.1's third
trap (*the branch ran* and *its value reached the plant* are different claims) landing **one
selector lower than the counter built to catch it**. § 5.3's `n_reach` is the honest number.

### 5.5 THREE INSTRUMENT FAILURES, AND WHAT CAUGHT THEM

1. **The probe flag on the instance.** `_with_probe` wrote `self._coord_probe = True`; `_cap_march`
   builds a **new** machine through `_shared_rig`/`at_lever`, which read the class default
   `False`. The log came back **empty** while `hits`/`binds` (class-level writes) reported
   **1366/1366 and looked like a flawless pass**. The carried-knob trap of rungs 61–78, landing on
   the *instrument* rather than the plant.
2. **The vacuity guard returning `None` in its most vacuous case.** `vacuous` was conditioned on
   `max(dels) > 0`, so `d_max == 0` — the *strongest* possible vacuity — was reported as
   *unmeasured*.
3. **A single fallback total.** It could not distinguish *3 slack states in each of 2 coordinates*
   from *6 in `phi` alone*; only the second would mean the knob was never exercised. Splitting the
   counter by coordinate settled it (`br_phi = 3`, `br_inc = 3`).

4. **AND A FOURTH, WHICH SURVIVED THE SHIP** (`docs/rung79-gap-margin.md` § 4.1). The
   `n_distinct` guard was read as refuting *"one state logged 1366 times"*. It is exactly that:
   the 129 distinct `p_phi` values are float-level products of a bracketed solve whose **start
   point** `mf_sched` sweeps 1.478× while the plant state is constant to 1e−15. The counter
   distinguishes distinct **floats**; the claim needed distinct **states**. **A counter is only
   as good as the noun it counts** — "count, never eyeball" does not save you from counting the
   wrong thing, and this one sat inside the guard § 8.1 is proudest of.

The first three were caught by **counting**, never by inspection — rung 78 § 5.1's lesson, and it
did not prevent the same class of failure recurring, only shortened it. The fourth was not caught
at all until the seam it left behind was checked.

---

## 6. CONCESSIONS (in addition to every one rungs 62–78 list, all inherited)

* **The knob is unreachable on this plant** (§ 5.3), so every trajectory claim here is about a
  coordinate that the plumbing declines to consult. Nothing about the *dynamics* of an
  incidence-referenced fuel leg is established.
* **`br_inc = 3` of 1366.** The knob was exercised on the plant three times. Three is not zero —
  which is the only reason § 5 is reported at all — but it is not a trajectory either.
* **§§ 1–4 are an identity.** They confirm code paths run, not that anything was discovered.
* ~~**The gap is essentially constant and unexplained.**~~ **CHECKED — `docs/rung79-gap-margin.md`.**
  `gap ∈ [0.126136034007005, 0.126136034007016]`, and the explanation is § 5's correction box:
  the ratio is fixed to 13 digits because the **plant is fixed** to 15. With the plant pinned,
  `p_phi ≡ mf`, so `gap + 1 = a_cap/mf` — a **frozen-state ratio**, not an accel-wide invariant.
  Swept: `gap(margin = 0) = 0` **exactly** (a standing plant is AT steady state, and rung 48's
  `margin = 0` schedule IS the steady-state fuel), and
  `d ln(gap+1)/d ln(1+margin) = 1/(1−c)` — **rung 77's stiffness**, agreeing with the shipped
  `_c_at` to **5.3e−06** when read at the fixed point.
* **P2 is vacuous by 13 orders of magnitude**, so this rung says nothing about whether a
  discontinuous selector can amplify a coordinate's float noise. The question is real and
  **untouched**.
* **One operating point, one leg, one coordinate pair.** `phi_lim = 0.80`, LP spool, φ ↔ `M_i`.
  This meant *one setting*; it is true in the far stronger sense of **one STATE** — see § 5's
  correction box. Everything § 5 says about *trajectories* is a statement about a plant that did
  not move.

## 7. THE REDUCE

`_phi_ref = "phi"` dispatches `_cap_fuel` to `super()` on an exact comparison, so not one float in
this family moves and the identity is structural rather than numerical. Gated in **both**
directions on rung 73's discipline: at `incidence` the residual's **SLOPE RATIO** must equal
`1/φ_lim²` (else the knob is dead — a coordinate that changes nothing is not a coordinate) **and**
the **SET POINT** must not move (else it is not a coordinate at all, it is a device). Either half
alone passes something broken.

## 8. THE ANCHOR, SCORED

| | prediction | verdict |
|---|---|---|
| **P1** | set point moves by solver noise only: `0 < δ < 1e−9` at a majority of cells | **SPLIT.** On the plant `δ = 0.0` **exactly** at 1366/1366 — the lower bound REFUTED, and not because the solver is slope-insensitive but because the coordinate was **never consulted** (§ 5.1). Forced past the short-circuit: **`6.14e−15` / `7.64e−16`**, inside the registered band. The prediction was right about the mechanism and wrong about where it would be visible |
| **P2** | the min-select never flips | **HELD, AND VACUOUSLY, ON ~~BOTH~~ THREE GROUNDS** — `flips = 0`, but `d_max = 0.0` (nothing moved), `gap ≈ 0.1261` (nothing could have crossed), **and the plant never left its initial state at all** (§ 5's box; the third ground was found after ship). 13 orders of margin |
| **P2n** | the non-vacuity guard, registered in advance | **FIRED AS REGISTERED, ON BOTH GROUNDS.** See § 8.1 |
| **P3** | `binds > 0`, and `binds == hits` | **HELD** — 1366/1366, and rung 78's masked leg is indeed this rung's authoritative one. **But the CLAIM needed narrowing**: `binds` is not the last selector (§ 5.4); the honest number is `n_reach = 1363` |
| **P4** | trajectory worst move `< 1e−9` | **HELD, VACUOUSLY** — `0.000e+00`, for P1's reason |
| **P5** | root count **1** and **equal** in both coordinates at every cell | **HELD** — `[1]`, equal at 10/10, on an instrument with rung 78's second root as inherited positive control |
| **P6** | `_cap_free` converges in both coordinates; no damped Newton | **HELD** — 2 732 calls, zero failures |
| **—** | *(unregistered)* | **NEW: § 5.3's COMPLEMENTARITY.** `n_live = 3`, `n_reach = 1363`, `n_both = 0`. An identity of the branch condition, and the rung's actual content |

### 8.1 P2n IS THE ANCHOR'S ONE CLEAN SUCCESS, AND IT IS THE TRANSFERABLE RESULT

Every other row is a confirmation, a vacuous hold, or a split. **P2n was written before the
measurement, predicted the most likely disappointing outcome in those words, and caught it — on
both grounds at once.** Rung 78 § 5.1 hit three vacuity traps *after the fact* and had to rewrite
its section twice; this rung hit three more (§ 5.5) and caught all three immediately, because the
guard existed before the number did.

**Registering the vacuity condition is worth more than registering the result.** That is the
finding most likely to outlive this rung's physics.

**AND IT IS NOT ENOUGH, WHICH IS THE SHARPER HALF.** P2n asked *"is the gap wide enough to make
`flips = 0` uninformative?"* and answered yes. It never asked *"did the plant move?"* — and it
had not (§ 5's box). A registered vacuity guard protects the question you thought to register;
the fourth trap (§ 5.5) sat **underneath** this one, in the same section, and shipped. The
transferable form is therefore narrower than the sentence above: **register the vacuity condition
of the INSTRUMENT and of the PLANT separately**, because a guard on the reading cannot see a
plant that never produced one.

## 9. NEXT SEAMS

* **A PLANT WHERE THE KNOB IS REACHABLE.** § 5.3 says `{live} ∩ {reaches}` is empty *for a leg
  solved through `_cap_free`*. A leg whose set point is computed **without** a binding
  short-circuit would break the complementarity — that is a plumbing change, and it is the only
  route to a real trajectory measurement here.
* ~~**THE CONSTANT GAP.**~~ **CLOSED — `docs/rung79-gap-margin.md`, and it CORRECTS this rung.**
  The `(1 + margin)` half **held**; the other two clauses are **refuted**. `κ` does not drift —
  `n_H` is clamped at the schedule table's first abscissa at 341/341 steps because **the plant
  never moves** (§ 5's box) — and there is **no** relation between the two legs: the residual is
  rung 76's `solve` cap law being a **fixed point**, whose gain is rung 77's `1/(1−c)`. Rung 49's
  floor enters only by standing the plant still, which is what makes `p_phi ≡ mf`.
* ~~**A φ-FLOOR MARCH WITH NON-ZERO INITIAL MARGIN.**~~ **CLOSED by REFUTING it —
  `docs/rung74-arrest-interval.md`, and all three clauses were wrong.** The window `(0.7731,
  0.7884)` is the **ANTI-window**: `0.7731162133` is the **free operating point**, so a wall
  above it is *lifted onto* by an airflow floor and the initial margin is exactly **zero** — the
  condition the seam was trying to escape. `0.7884` has no provenance in this rig (a stator-only
  lift there uses 19.5% of `v_max`) and is recorded as unsourced. And it is **not a new rig**: in
  `clip` the cell `(clip, 0.76)` — rungs 75/76's own wall — already has all four loops live with
  +1.3% initial margin. In `demand` it is unreachable at **every** wall, because the arrest is an
  **interval** `(0.7731162133, ≈0.852)` above the free point and the leg **holds** below it.
* ~~**RUNGS 72–77's § 5 SECTIONS.**~~ **CLOSED — `docs/rungs72-77-march-audit.md`.** None of
  them stands still, and the seam had to be **retargeted** first: "§ 5" is this rung's
  numbering, and the audit's unit is the *march call site*. The arrest is the cell `(demand,
  0.80)` — **rung 74 § 2.2 owns it**, and this rung's § 5 marched inside it without citing it.
* **CAN A DISCONTINUOUS SELECTOR AMPLIFY FLOAT NOISE?** P2's real question, untouched because the
  gap is 13 orders too wide. It needs a plant where two legs cross.
* **THE SAME COORDINATE ON A LEG THAT IS NOT `min`-COMPOSED.** Rung 76 named the COMPOSITION as
  the obstruction; this rung adds that the *solver's short-circuit* is a second one.
* Everything rungs 72–78 § 8–11 leave, unchanged by this rung.
