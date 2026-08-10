# Rung 80 anchor — THE SPLIT WALL

**Written BEFORE any measurement.** Every prediction below is timestamped by the commit that
adds this file; nothing here is edited after a sweep runs, only annotated with the outcome in
`docs/rung80-spec.md`. This is the repo's rung-49/64/78 pre-registration convention.

---

## 0. THE SEAM, AND THE KNOB

`docs/rung74-arrest-interval.md` § 8:

> **STILL OPEN, and now sharper: a four-loop live-wall march in `demand`.** § 5 shows it is
> unreachable on the **shared** wall by construction. The one untried route is to **split the
> wall** — arm the airflow levers ABOVE the fuel leg so the queue's order reverses and they
> become authoritative. That is a real knob (`sm_air` beside `sm`), zero new constants.

**THE KNOB.** `_shared_rig` builds all four loops' floors from ONE margin `sm`, through the SAME
`from_margin(cmap, ., sm)` factory (rung 68 § 10's zero-new-constant rule). Rung 80 adds a second
margin `sm_air`, used for the **airflow** legs only:

    phi_lim = (1 + sm    ) * cmap.phi_surge      the FUEL leg's floor   (rung 49)
    phi_air = (1 + sm_air) * cmap.phi_surge      the VALVE's and STATOR's floor (rungs 64/68)

`sm_air = None` ⇒ the inherited single-wall path, untouched, so the reduce is by **dispatch** and
not by recomputing an equal float. No new constant: `phi_surge` is the map's own, and `sm_air`
is a swept knob exactly as `sm` has been since rung 49.

## 1. WHY THE SPLIT SHOULD DO ANYTHING — THE MECHANISM BEING TESTED

`docs/rung74-arrest-interval.md` § 3 measured the non-definitional half:

> **THE MIN-SELECT'S WINNER DECIDES WHETHER THE OTHER LOOPS ON ITS CONSTRAINT EVER SEE A
> VIOLATION AT ALL.** The airflow levers are not parallel defenders of the wall — they act only
> inside the authoritative leg's **tracking error**, and their entire authority is its failure.

On a **shared** wall the fuel leg holds `φ ≥ φ_lim` (in `demand`, over-protecting at every wall
measured), so the airflow levers see no violation: 0/341 motion. A **split** wall gives them a
violation of size `φ_air − φ_lim` that the fuel leg is not merely failing to prevent but is
**not asked to** prevent. That is the whole of the construction.

## 2. THE WINDOW, DERIVED FROM TWO ALREADY-MEASURED NUMBERS

Both endpoints are read off `docs/rung74-arrest-interval.md`, not chosen here:

| edge | value | source | why it bounds the window |
|---|---|---|---|
| free droop (`demand`) | **0.7464354455** | § 5 | below it the fuel leg is **dormant** — three loops, not four |
| free operating point | **0.7731162133** | § 2, § 4 | at or above it a floor **lifts `φ(0)` onto the wall** and the plant **ARRESTS** |

so a four-loop `demand` cell requires

    0.7464354455  <  phi_lim  <  phi_air  <=  0.7731162133

a ~3.5 % window in `φ`. **The anchor cell is `phi_lim = 0.75`, `phi_air = 0.77`**
(`sm = 0.3636363636…`, `sm_air = 0.4`), both strictly inside it, on the rung-79 test rig
(`FLOOR = 0.55`, `B = 0.10`, `V_MAX = 0.20`, `LO/HI/TT4_MAX = 1000/1400/1200`).

## 3. THE PREDICTIONS, IN THE ORDER THEY WILL BE SCORED

**P1 — THE PRIMARY, AND IT IS A PREDICTION OF FAILURE.** The split will **not** produce four live
loops. The airflow levers have 3×–40× spare authority at these walls (§ 2 of the arrest doc) and
only `φ_air − φ_lim ≈ 0.02` of gap to close, so they will close it, `φ` will rise to `φ_air`, and
**the fuel leg's own floor will stop being violated** — the leg goes dormant and `_riding4`
returns 0 points, `n_live ≤ 3` a **sixth** time, by a mechanism no prior rung states:

> **A SET OF FLOORS ON ONE VARIABLE IS A TOTAL ORDER, AND ONLY THE TOP ONE IS EVER LIVE.**
> Splitting the wall does not put loops in parallel; it re-labels which loop is the leader.

Scored on the **fuel leg's own cut** (`required_fuel` at the anchor cell), not on lever motion —
the arrest doc § 3.2's warning that the motion counter and the liveness label are different nouns
applies to this rung's own counters.

**P2 — THE ALTERNATIVE, AND IT IS PREDICTED IMPOSSIBLE ON THIS HARDWARE.** Both walls stay live
only if the levers **saturate** before closing the gap. The valve's saturation edge is bracketed
at `φ ∈ [0.8500, 0.8550]` (arrest doc § 4); the arrest edge is `0.7731162133`. Saturation sits
**above** the arrest edge, so the two requirements are incompatible and the four-loop cell is shut
by a **derived impossibility**, not by a failed search. **The escape is a larger `b_max`**, which
the arrest doc § 8 already flags as unmeasured, and which this rung will scope but not chase.

**P3 — THE ARREST EDGE CHANGES OWNER.** With both walls below `0.77312`, `φ(0)` is unchanged, so
the arrest bracket should relocate from `sm` to `sm_air`: sweeping `sm_air` with `sm` **fixed**
should march at `φ_air = 0.7731` and arrest at `0.7732`, the **same** two numbers rung 74's
bracket has, on the **new** knob. This is the rung's gate shape — a threshold with a derived edge,
so it cannot be satisfied by tuning.

**P4 — THE DISCRIMINATOR, AND THE RANK CLAIM IS DELIBERATELY *NOT* MADE.** That the rank is
unmoved is **definitional** — `∂(c − φ)/∂x = −∂φ/∂x` for any level `c`, so a gradient is blind to
its own level and a rank test on it would pass forever and guard nothing (this repo's recorded
failure mode: arrest doc § 3.1, rung 78 § 5.5's four vacuity traps). The **measurable** question
is rung 69's: `det J` was blind to the **coordinate** split and `c1` was the discriminator, so —

> does a **LEVEL** split move `c1` / the cyclic product, or only the operating point?

Predicted: **no structural move** — `c1` shifts only through the relocated operating point, and
the three-φ-loop cyclic identity holds at its inherited value. **If the gain reader finds no
interior point at all (P1 holding), this is reported as VACUOUS and not as a confirmation.**

## 4. THE CONTROLS, FIXED IN ADVANCE

1. **THE POSITIVE CONTROL FOR THE READER.** Every measurement is run in `clip` as well as
   `demand`. `clip` has four-loop cells at the shared wall (arrest doc § 5: valve 275, stator 281
   at `0.760`), so **zero lever motion in `clip` means the reader is broken, not the plant quiet.**
2. **THE DROPPED-KWARG CONTROL.** The plant asserts `phi_air > phi_lim` by name whenever
   `sm_air` is set. `_shared_rig` is overridden six times up this ladder, each with an explicit
   argument list; a knob that failed to reach the rig would make the reduce test pass *because it
   was ignored* and the liveness reader honestly report "no change", which reads as a finding.
   The split is therefore applied **on the machine `super()` returns**, in this rung's own
   override — the ladder's own idiom for carrying a knob — and asserted, never threaded.
3. **THE ANCHOR CELL IS QUOTED WITH ITS SETTINGS.** Rung 63's recorded failure — a number taken
   at another rung's settings and re-used — is why every table in the spec names `phi_lim`,
   `phi_air`, the coordinate and the clocks in its own header.

## 5. WHAT WOULD REFUTE EACH PREDICTION

| # | refuted by |
|---|---|
| P1 | `_riding4` returning a non-empty set at the anchor cell with `required_fuel > 0` throughout |
| P2 | any `φ_air ≤ 0.77312` at which the valve reaches `b_max` (would open the four-loop cell) |
| P3 | the arrest bracket on `sm_air` landing anywhere other than `[0.7731, 0.7732]` |
| P4 | a cyclic product that moves by more than the operating-point shift accounts for |

## 5a. SCORED — APPENDED AFTER THE SWEEPS, WITH NOTHING ABOVE EDITED

Everything in §§ 0–5 is as committed before any measurement. The outcomes:

| # | verdict | where |
|---|---|---|
| **P1** | **REFUTED** — the fuel leg stays live (215/341 cuts) and `n_riding4` reaches 33–39. The four-loop cell OPENS | `docs/rung80-spec.md` § 2 |
| **P2** | **REFUTED twice** — the valve rides at 15 % of authority in the window, and saturated-and-marching cells exist at 0.855–0.88 | § 4 |
| **P3** | **REFUTED** — neither split arm arrests at any wall; the arrest needs the floors to COINCIDE | § 3 |
| **P4** | **ASKED AND ANSWERED, with its rank half still refused** — `c1`, `zeros` and the cyclic product reported, never gated; the zeros carry a positive control | § 5 |

**Both controls did their job.** The `clip` arm moved both levers on every row (245/251 at the
shared baseline, matching `docs/rung74-arrest-interval.md` § 5 exactly), and the dropped-kwarg
assert never had to fire because the walls were read back off the built limiters in every row.

**The finding is not any of P1–P3.** It is that the seam named the wrong noun: the split delivers
the FOUR-LOOP CELL and not a FOURTH LIVE LOOP, because authority is decided on the ACTUATOR and a
level on the CONSTRAINT cannot reach it. Full statement in `docs/rung80-spec.md` § 6.

## 6. THE REDUCE CONTRACT

`sm_air = None` ⇒ **exact dispatch** to rung 79: the override returns `super()`'s machine
untouched, so every rung-79 reader is bit-for-bit. Gated on the shipped rung-79 arms, and
additionally at `sm_air = sm` — which must equal the `None` path to the last bit, since it
rebuilds the same floors from the same factory.
