# Rung 74 anchor — THE DEMAND COORDINATE (rung 73 § 11's sharpest seam)

Scored in `docs/rung74-spec.md` § 9. The rule this project runs under: **a prediction that is
edited after the measurement is not a prediction.**

**AND THIS ANCHOR DISCLOSES ITS OWN ORDER, as rungs 71, 72 and 73 did.** Two things were
measured before this document existed, both in `M:\claud_projects\temp\rung74\`
(`probe0_cap_in_slack.py`, `probe1_killcheck.py`, outputs saved beside them):

1. **A FEASIBILITY CHECK ON THE CAP (§ 0.2), and it was not optional.** The demand
   coordinate's target is the leg's **cap**, and *both* shipped caps are floored at the
   schedule — `_surge_fuel` returns `mf_sched` **itself** when `φ` is clear of the floor
   ("DORMANT — the leg is not consulted"), and `required_gov` short-circuits to `0` before
   `_topping_fuel` is ever called. **The cap above the schedule has never been computed in
   this project.** If it had been unreachable, the demand target would have had to fall back
   to the floored cap, the dormant-leg cut this rung reports would have been an artifact of
   that fallback, and this would have been a *different* rung. It is reachable — 341 of 341
   points, both arms, both caps (§ 0.2).
2. **THE OPEN-LOOP SIZE OF THE FORCING (§ 0.3).** This is the measurement that decides whether
   the framing is right at all, and running it late would have meant writing an anchor around
   an effect that might have been numerical dust. **It is reported as measurement and is NOT
   scored as prediction.** What § 2B scores is everything it does not settle: the closed-loop
   bill, the sign pattern in the gains, the spectrum invariance *measured on two independently
   marched plants*, the floor's address, the latch arm, the reference cells and the reduce.

§ 2 is split as rungs 72's and 73's were. **§ 2A is DERIVED** — worked out on paper from the
inherited laws before any demand plant existed — and is listed as derivation, **not scored as
prediction**, except where § 9 finds a derivation measured false (rung 72's D5 precedent).

---

## 0. WHAT THE SEAM IS, AND THE TWO CHECKS THAT PRECEDED THIS DOCUMENT

### 0.1 The seam, and the one line that separates the two plants

Rung 73 § 11: *Every leg in this family lags its **clip**; a real fuel control lags the
**demand** (`w = mf_sched − g`). On a ramp those are different plants — they differ by
`ṁf_sched·τ`. **It is the sharpest seam here**, and it is the last place `n_live = 4` could
still hide.*

Both fuel-side legs since rung 47 carry the clip as the state:

    dg/ds = ( required(ν,q,v,mf_sched) − g ) / τ ,   g ≥ 0 ,   mf = mf_sched − max(gf, gr)

with `required = max(0, mf_sched − cap)` — the **cut**, floored at zero. A fuel control does
not hold a cut; each leg holds the **fuel it would allow** and the lowest wins:

    dw/ds = ( cap(ν,q,v) − w ) / τ ,   NO state floor ,   mf = min(mf_sched, w_f, w_r)

**THE ALGEBRA IS ONE LINE, AND IT IS WHY THIS IS A COORDINATE AND NOT A LEVER.** Substituting
`w = mf_sched − g` and `cap = mf_sched − req`:

    dg/ds = ( req − g ) / τ  +  ṁf_sched                    [the demand plant, in clip coords]

The added term is **state-independent** (the schedule is a function of `s` alone), so it is a
pure **forcing**: it cannot appear in any Jacobian. That is § 2A's whole derivation and it is
*not* scored — a `_jac4` that was handed eigenvalue invariance by construction would be the
sixth instance of the shipped-instrument-agrees-with-itself pattern this family has documented
five times (rung 67 gate 9, rung 71 § 1.4, rung 72 §§ 4 and 8, rung 73's `_reference` no-op
that returned a *perfect confirmation* having measured nothing). **The invariance is therefore
measured on two INDEPENDENTLY MARCHED plants, `w` a genuine state**, and never inferred from
the identity above.

### 0.2 THE CAP ABOVE THE SCHEDULE — measured first, because it picks the rung

Probe 0, along rung 73's own anchor trajectory (`inc` both arms, 341 points, every point in
probe 1; every 20th printed in probe 0), extending each shipped set-point solve **upward**:

| | at `s = 0` | at `s = 0.1` | from `s = 0.2` | failures |
|---|---|---|---|---|
| `cap_Tt4 / mf_sched` | **1.3032** | 1.0190 | binding (< 1) | **0 of 341** |
| `cap_φ / mf_sched` | **1.0000** | binding | binding | **0 of 341** |

**Both are reachable everywhere, so the plant is the clean one and no fallback is needed.**
The two caps say different things and both matter:

* the **governor's** cap sits a third above the schedule at the start — real headroom, which
  the clip coordinate's `max(0, ·)` **erases**. A demand-lagged governor chasing `1.303×` does
  not trail into authority; one chasing a cap *floored at the schedule* would. **That is the
  artifact this probe was run to avoid reporting as a finding.**
* the **`φ` leg's** cap is at the schedule at `s = 0` and below it from `s = 0.1` — so on this
  anchor the `φ` leg is *binding almost from the start*, and flooring changes nothing for it.
  The dormancy question is live for the governor and nearly moot for the surge leg.

**The upward search is a new solve in a regime the family has never exercised**, and it is
disclosed as this rung's one implementation cost (§ 6). It introduces no constant: it walks the
inherited bracket in the other direction (`_surge_fuel` shrinks by `0.9`; this grows by `1/0.9`)
and asserts, loudly and by name, if it ever fails.

### 0.3 THE FORCING IS NOT DUST — open loop, and the derivation lands within 0.5%

Probe 1, the demand lag integrated along rung 73's trajectory against both targets
(`ṁf_sched = 0.02792`, `τ_gov = 0.05`, so `ṁf_sched·τ = 1.396e−3`):

| `s` | `g_r` (rung 73) | `g_r` (demand) | difference | `ṁf_sched·τ_gov` |
|---|---|---|---|---|
| 0.400 | 6.023e−3 | 7.424e−3 | **1.401e−3** | 1.396e−3 |
| 0.500 | 8.592e−3 | 9.978e−3 | **1.386e−3** | 1.396e−3 |
| 0.700 | 9.526e−3 | 9.543e−3 | 1.7e−5 | **0** (ramp over) |

and against the **fuel leg's own clip**, `req_f ≈ 1.05e−3` at `s = 0.1`: the forcing is
**larger than the quantity it is added to**. This is open loop — the plant is *not* re-marched,
so it is a feasibility probe and never a result — but it settles that the closed-loop rung
exists.

It also shows the effect **outlives the ramp**: the forcing vanishes at `s = r`, but the leg
unwinds on `τ_rel`, so 238 (`φ` arm) / 204 (incidence arm) points carry a positive demand clip
where rung 73 has the leg dormant. Rungs 51/52's release edge, reached by a coordinate.

### 0.4 THE ATTACK/RELEASE TEST INVERTS, AND IT WOULD HAVE PASSED EVERY GATE

Attack in clip coordinates is `required > g`. Substituting: `required > g ⟺ cap < w`. A port
that writes `lag.tau(cap, w)` and keeps rung 52's shipped body (`tau_att if first > second`)
selects **`tau_rel` on attack** — with `tau_rel = 3·tau_att` in `_shared_march`, a 3× clock
error in the direction that *slows protection*. It would have read as a finding (*the demand
coordinate is less protective*) and passed every gate one would think to write. The correct
port **swaps the arguments**, and § 2B.7 gates the returned constant directly on a known-attack
point.

---

## 1. THE PLANT, AND THE THREE DECLARED KNOBS

`_lag_coord` joins `_share_law` (rung 72) and `_ref_law` (rung 73) as a **third declared knob**:

| `_lag_coord` | state | target | floor | is |
|---|---|---|---|---|
| `clip` | `g` | `required` | on the **STATE** (`g ≥ 0`) | **rung 73 / 72, bit-for-bit** |
| `demand` | `w` | `cap` | on the **COMPOSITION** (`mf ≤ mf_sched`) | **THE PLANT** |
| `demand-latched` | `w` | `min(mf_sched, cap)` | on the **STATE** (`w ≤ mf_sched`) | § 3's instrument |

**The latched arm is the isolation instrument** — rung 72's SUM law and rung 73's reading C in
their fourth shape. Without it this rung changes two laws at once (the coordinate *and* the
floor's address) and no cell is attributable. Latched is *exactly* the clip plant plus the
forcing, so it separates them: `latched − clip` is the **coordinate**, `demand − latched` is
the **floor's address**.

`_ref_law` survives the coordinate unchanged, and that is derivation, not choice: rung 73's
hook `req_applied = g_own + req_sched − max(gf,gr)` maps, term for term, to

    w_target = w_own + cap − mf_app                       [the same law, in demand coordinates]

with the same float-identical branch when the leg holds (`mf_app == w_own`). So both reference
cells exist here and rung 73's pole at the origin must survive — **an eigenvalue cannot notice
a coordinate**, which is the sharpest way to say what this rung is.

`demand × sum` is **REFUSED**, on rung 73's own words: `min(mf_sched, w_f, w_r)` has no `sum`
reading that keeps the schedule as an input, so it would swap two declared laws at once.

---

## 2A. DERIVED — worked out before any demand plant existed. NOT SCORED.

* **D1** The demand plant in clip coordinates is `dg/ds = (req − g)/τ + ṁf_sched`; the term is
  state-independent, so **`J` is the same matrix** and `eig`, `zeros`, `det J` and every cyclic
  product are invariant. Measured on two independently marched plants (§ 2B.1–2), never
  inferred.
* **D2** In *demand* coordinates the Jacobian entries are `D J D⁻¹` with `D = −I` on the two
  fuel-side states: **fuel↔non-fuel off-diagonals flip sign; fuel↔fuel and non-fuel↔non-fuel
  entries do not.** Cyclic products cross the block an even number of times, so they are
  invariant — which is why the *spectrum* cannot be the discriminator and the *entries* are.
* **D3** Min-select is flat in the masked **demand** exactly as it was in the masked **clip**
  (`min()` is flat in its non-minimal argument), so the masked column is still zero, `M` is
  still block-triangular, and **`n_live` is still ≤ 3**. The seam closes by refutation, the
  third running.
* **D4** The steady tracking error of a first-order lag on a ramp is `ṁf_sched·τ`, so the
  demand plant's applied clip exceeds the clip plant's by that amount **while the schedule
  moves, and by nothing after it stops** — modulo the release clock's unwinding.
* **D5** `demand-latched` on a **FLAT** schedule is the clip plant **exactly** (`ṁf_sched ≡ 0`
  makes the forcing vanish and the two floors coincide), which is a reduce by *identity* rather
  than by dispatch — rung 71's form, two rungs on.

## 2B. PREDICTED — scored in § 9.

* **P1** Two independently marched plants, gains measured through the shipped closures: the
  **spectrum agrees to ≤ 1e−9 relative** in every cell, on both arms and all three clock arms.
* **P2** The **entries do not**: the sign pattern of D2 is measured, and at least one
  off-diagonal pair differs by exactly a sign at ≥ 1e−6 absolute — i.e. the plants are *not*
  the same matrix, only similar. (A cell where every entry agrees would mean the port did not
  change coordinates — rung 73's `_reference` no-op, one rung on.)
* **P3** `zeros`, `det J` and all four `pair_·` products agree **cell for cell** with rung
  73's (`3 / 2 / 2 / 1` under `applied`, `2 / 1 / 1 / 0` under `sched`).
* **P4** Closed loop, the bill: `max Tt4` **falls** and `min φ_lp` **rises** under `demand`
  against `clip` on every arm — the forcing is an extra cut, and an extra cut is protective on
  both currencies. Magnitude **not** pre-registered (§ 0.3 is open loop).
* **P5** The **hand-over moves EARLIER** under `demand`: both legs carry `ṁf_sched·τ` more cut,
  but the governor's clip is the larger and grows faster, so it overtakes sooner. (Rung 73's
  applied reference moved it *later*; this moves it the other way, and for an unrelated reason.)
* **P6** `demand − latched` is **zero to machine precision wherever both legs are riding**, and
  non-zero only where a leg is dormant — the floor's address is a *boundary* property.
* **P7** `latched` vs `clip` on a **FLAT** schedule off the running line: **bit-for-bit
  identical** (D5), with the run gated non-vacuous (both legs must ride).
* **P8** The lag returns `tau_att` on a known-attack point in demand coordinates (§ 0.4).
* **P9** `_lag_coord = 'clip'` reproduces rung 73 **bit-for-bit** on all six of its arms.

**REFUSED IN ADVANCE:** any repair that clamps the unfloored cap at a ceiling (a new constant);
marching `demand × sum`; and reporting the spectrum invariance from a single plant's `_jac4`.

---

## 3. AMENDMENT — P7, DISCLOSED WITH ITS TIMING

**P7 was recognised as unattainable *as stated* before the flat-schedule march ran**, and it is
recorded here rather than quietly rescored, because *when* an author stopped believing a
prediction is the only thing that separates a prediction from a straw man.

The recognition was a derivation, not a measurement: `demand-latched` computes `cap − w` where
the clip plant computes `−(req − g)`. Those are the same real number and **not** the same
sequence of binary floating-point operations, so "bit-for-bit" was never reachable — the
reachable claim is *equal to rounding*. Nothing was re-run to find this out and no measurement
existed at the time.

It is **left in § 2B unedited** and scored REFUTED-as-stated in the spec's § 9 against the
measured `1.59e−12`, on the same reasoning rung 73 § 1.3 applied to its own refuted tolerance:
a prediction that is corrected after the author sees it is worth less than a prediction that is
scored honestly. **This paragraph is the disclosure; the score is the result.**

**The other amendment is NOT to a prediction but to a reader**, and it is recorded in the spec
(§ 2) rather than here because it was found *after* the measurement: the first `demand_law`
compared the clip plant under the APPLIED reference against the demand plants under `sched`,
changing two laws at once — this anchor's own § 1 refusal, broken by the reader written to test
it. Every § 2 number was re-measured at one reference. It moved the magnitudes (the `φ_lim =
0.80` arms by 32 K and 71 K) and no verdict.
