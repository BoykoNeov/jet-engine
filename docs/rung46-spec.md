# Rung 46 — The TIT topping governor: enforcing the redline rebates the LATE spool, MISSES the binding one

**Scope.** `TwoSpoolFuelTransient.integrate_fuel(…, Tt4_max=…)` + `topping_relief` /
`_topping_fuel` — a **TIT topping governor** clipping the metered fuel to hold `Tt4 ≤ Tt4_max`,
the **first fuel-side FEEDBACK** in the ladder. Rung 43/45 established that a fuel accel makes
`Tt4` overshoot (rung 35 on two shafts); every one of them ended on the same concession —
"`Tt4` overshoot vs a redline is not claimed — no TIT limit is modelled." Rung 46 models it.

> **The governor works — it pins `Tt4` at the redline (the effect added). Its surge-side
> consequence INVERTS the naive "the two accel limits are coupled, so enforcing one relieves
> the other." Enforcing the TIT redline rebates surge margin on the LATE, non-binding **HP**
> spool (`relief_hp > 0`) but **ZERO** on the EARLY, binding **LP** spool (`relief_lp = 0` to
> machine zero) — a two-shaft SURGE-RELIEF SPLIT no single shaft can show. The mechanism: the
> surge debit is paid on the EARLY-ramp fuel (the LP surge minimum falls at `Tt4 ≈ 1374`,
> DURING the ramp, below any valid redline, then self-recovers), while the governor only trims
> LATE fuel (once `Tt4 > Tt4_max`, near the ramp end). It cannot refund a surge cost incurred
> UPSTREAM of its window. Rung 35's two limits are coupled in CAUSE but SEQUENCED in time; the
> governor acts on the trailing (TIT) limit and structurally misses the leading (surge) one. So
> rung 45's "fuel ENLARGES the surge approach" gets its punchline: the enlargement is deposited
> early, and a TIT topping governor is structurally too late to claw it back. The one lever —
> in the FAST-ramp limit (`r ≤ 0.3`) the LP surge minimum migrates ABOVE the redline and
> `relief_lp` goes positive — so the governor becomes a modest LP-surge lever precisely where
> surge is most dangerous.**

Unlike rungs 7–30, 36, 41, 44, 45 (pure diagnostics that only *read* the running point), rung 46
**FEEDS BACK**: the applied fuel depends on the current spool state through `_topping_fuel`. This
is the control-side counterpart of rung 45 and the culmination of the fuel-control ladder
(35 → 43 → 45 → 46). Arming the governor with `Tt4_max=None` (or any redline above the bare peak)
leaves rung 43/45 **bit-for-bit** — the clip branch is never consulted — and the default
`build_turbojet(…).run(…)` design run is **bit-for-bit rung 6**.

---

## The control law — a min-select on fuel, the standard accel-schedule TIT limiter

At each RK sub-evaluation of `integrate_fuel(…, Tt4_max)`, the scheduled fuel is applied unless it
would drive `Tt4` above the redline, in which case it is CLIPPED to the instantaneous value that
pins `Tt4 == Tt4_max` at the current flow (`_topping_fuel`, a bracketed Illinois solve — `Tt4`
rises monotonically with fuel at fixed spool speeds). That is a **min-select**:
`mdot_fuel_applied = min(schedule, topping_setpoint(ν_L, ν_H, Tt4_max))`. Dormant while the
schedule is below topping (bit-for-bit rung 43); active only in the overshoot window, where fuel
rides the topping value and `Tt4` pins at the redline.

The set-point solve runs the SAME quasi-steady closure (`_instant_fuel`/`_close_fuel`) the plant
already uses, so no new physics enters — only a new **control** input. There is **no new time
constant**: the governor is instantaneous (an idealised topping limiter), deliberately, so the
finding rides on the min-select geometry, not on a fitted actuator lag.

---

## Sign-space only — inherited from rungs 41/44/45, plus one imposed redline

`Tt4_max` is **IMPOSED** exactly as `phi_surge` is (rung 41), so rung 46 makes **no** claim about
the absolute redline an engine should carry. It delivers **signs**: that the governor holds the
redline, that enforcing it splits the surge relief (HP rebated, LP not), that the split is
shape-robust, and that the LP relief switches on only in the fast-ramp limit. The **rung-36
discipline is enforced exactly: report the crossing, gate the flip** — here the "flip" is the
sign of the surge-relief differential, not a magnitude.

---

## THE FINDINGS

### (1) THE HEADLINE — the surge-relief SPLIT (`relief_hp > 0`, `relief_lp = 0`)

`topping_relief`, accel `Tt4` 1000 → 1400, `r = 0.5`, `rho = 1.0`, redline `Tt4_max = 1480`
(above the 1400 endpoint, below the ~1645 bare peak — a redline every steady point clears):

| shape | `relief_lp` | `relief_hp` | LP-min `Tt4` | HP-min `Tt4` | held |
|---|---|---|---|---|---|
| flow/press | **+0.00000** | +0.00279 | 1373.7 | 1558.5 | True |
| press/flow | **+0.00000** | +0.00357 | 1403.9 | 1583.3 | True |
| tilted | **+0.00000** | +0.00308 | 1404.9 | 1564.6 | True |
| hp-only | **+0.00000** | +0.00267 | 1372.9 | 1556.7 | True |

`relief_* = min_phi_*(topped) − min_phi_*(bare)`; `> 0` means the topped raw min `phi` sits ABOVE
(safer than) the bare one. **The LP relief is machine-zero at every shape; the HP relief is
positive at every shape.** A single shaft cannot exhibit a differential — this SPLIT is the
two-shaft claim. It holds even on **hp-only** (the LP map FLAT, no rung-40 complex mode), so the
split is the pure **window** mechanism, not a mode artifact.

**Why the LP is machine-zero and not merely small:** the trajectory is bit-identical bare-vs-topped
*up to and through* the LP's deepest surge point — the clip has not yet fired there — so
`min_phi_lp` is literally the same float. The exact zero is the tell of the mechanism, not a
rounded small number.

**Where the weight rests.** The durable, `rho`- and shape-robust half of the split is
`relief_lp = 0` and its mechanism: the two spools' surge minima sit at **different `Tt4`**
(≈1374 vs ≈1558), so one clip window (which opens at `Tt4 > Tt4_max`) structurally catches the HP
minimum and misses the LP one — a two-shaft fact no single shaft can present. The `relief_hp > 0`
partner is smaller and **`rho`-dependent** (it tracks rung 43's `rho`-loud overshoot: Table E shows
it vanishing to `0.00000` at `rho = 0.2`, so the HP *rebate* needs a sufficient overshoot,
`rho ≳ 0.5`). The headline is therefore **not** "the HP is universally rebated" but "the two surge
minima live at different `Tt4`, so a single TIT clip window cannot rebate both — and the one it
misses is the binding one."

### (2) THE MECHANISM — the surge debit is paid EARLY, the governor trims LATE

The bare accel trajectory (`r = 0.5`, `rho = 1.0`, flow/press), sampled:

| `s` | `Tt4` | `phi_lp` | `phi_hp` |
|---|---|---|---|
| 0.20 | 1318 | 0.7410 | 0.8804 |
| **0.24** | **1374** | **0.7399** ← LP min | 0.8738 |
| 0.26 | 1400 | 0.7400 | 0.8712 |
| **0.40** | 1558 | 0.7520 | **0.8627** ← HP min |
| **0.50** | **1645** ← Tt4 peak | 0.7705 | 0.8654 |

The **LP surge minimum leads** (`s = 0.24`, `Tt4 ≈ 1374`, DURING the ramp, below the 1400
endpoint), then **self-recovers** (`phi_lp` climbs back to 0.77) as the spool catches the airflow —
all before `Tt4` reaches its 1645 peak. The **HP surge minimum lags** (`s = 0.40`, `Tt4 ≈ 1558`),
and the **TIT peak lags furthest** (`s = 0.50`). The topping governor engages only where
`Tt4 > Tt4_max ≥ 1400` — i.e. **after** the LP has already passed its worst point and **at/around**
the HP's. It refunds the fuel that would have driven the (late) HP and TIT excursions; it cannot
refund the (early) LP surge cost, which was deposited upstream of its window and has already
self-recovered.

This is rung 45's "fuel ENLARGES the surge approach" completed: the enlargement is **deposited on
the early-ramp fuel**, and a TIT topping governor — which by construction acts only once `Tt4` is
already high — is **structurally too late** to claw it back. Rung 35's two accel limits are
**coupled in cause** (both born of fuel outrunning the lagging spool) but **sequenced in time**;
the governor exploits a temporal coincidence it does not have.

### (3) THE LEVER / the caveat — `relief_lp` switches on in the FAST-ramp limit

`relief_lp` is machine-zero only at **moderate** ramp rates. As the ramp gets faster the LP surge
minimum migrates to **higher `Tt4`** (the airflow lags harder, so `phi_lp` bottoms out later
relative to the fuel-driven `Tt4` rise); once it rises above the redline the governor begins to
rebate it (flow/press, `Tt4_max = 1440`, `rho = 1.0`):

| `r` | `relief_lp` | `relief_hp` | LP-min `Tt4` |
|---|---|---|---|
| 1.00 | +0.00000 | +0.00000 | 1219 |
| 0.50 | +0.00000 | +0.00606 | 1374 |
| 0.30 | **+0.00264** | +0.02319 | 1564 |
| 0.15 | **+0.01922** | +0.03869 | 1745 |

So the governor becomes a (modest) LP-surge lever **precisely in the fast-accel regime where
surge is most dangerous** — and even there the LP rebate stays well below the HP's. The gated
claim is therefore "zero at moderate ramp rates, positive in the fast-ramp limit," not an
unconditional zero.

### (4) DECEL — accel-only governor, bit-for-bit rung 45

On a decel (`Tt4` 1400 → 1000) the metered fuel FALLS and `Tt4` undershoots; it never exceeds any
redline above the endpoint, so the clip **never fires** and the topped march is **bit-for-bit** the
bare (rung 45) march at every shape (`Tt4_peak = 1400`, no overshoot). The topping governor is an
**acceleration**-schedule limiter; its whole action lives on the accel.

---

## Reduce-to-prior contract (the spine)

- **Dormant ⇒ rung 45/43 bit-for-bit.** `Tt4_max=None`, or any redline `≥` the bare peak, leaves
  the clip branch un-consulted: `integrate_fuel` returns the identical trajectory float-for-float,
  `phi_excursion_fuel`/`transient_surge_margin_fuel` are identical armed-vs-bare, and the rung
  38–45 suites pass unchanged.
- **`lp_disabled` ASSERTS — the split is inherently two-shaft.** A surge-relief split *between*
  spools has no single-shaft image (rung 45's contract, one rung on); `integrate_fuel(Tt4_max=…)`
  asserts on the degenerate engine. (The `Tt4_max=None` dispatch to rung 35 is untouched and still
  reduces bit-for-bit.)
- **Reduces to rung 44 on the `Tt4`-command path by construction** — with `Tt4` commanded there is
  no overshoot, so a redline above the commanded endpoint is never approached and the governor is
  inert. The TIT limit is **born on the fuel path** (rung 45's structure, mirrored).
- **Decel ⇒ rung 45 bit-for-bit** (finding 4): the clip never fires on a deceleration.
- **Cycle untouched ⇒ rung 6 bit-for-bit.** The governor lives only on the two-spool transient;
  the default design run is unchanged.

### Why rung 46 carries no independent bare-math gate (rung 45's precedent)

Rung 46 marches rung 43's `integrate_fuel`, whose every steady point control-invariance (rung 43
gate 1) lands exactly on rung 40's steady manifold — itself tied down by rung 40's independent
bare-math cascade. So the plant is anchored **transitively**. The genuinely new content is (a) the
topping-governor FEEDBACK, whose correctness is gated by "`Tt4` is HELD at the redline" plus the
dormant-reduce, and (b) the surge-relief-split **signs** — shape-robust directions, not magnitudes
a bare-math replica would constrain (rungs 42/43/45 set this precedent).

---

## Verification gates (`tests/test_rung46.py`)

1. **REDUCE — DORMANT ⇒ rung 45/43 bit-for-bit.** `Tt4_max` above the bare peak ⇒ `integrate_fuel`
   / `phi_excursion_fuel` / `transient_surge_margin_fuel` **`==`** the bare call; `Tt4_max=None` is
   the current path; the rung-45 read-only reduces (armed-vs-bare, cycle bit-for-bit rung 6) still
   hold.
2. **REDUCE — `lp_disabled` ASSERTS.** `integrate_fuel(Tt4_max=…)` (and `topping_relief`) assert on
   the degenerate engine; the `Tt4_max=None` degenerate dispatch to rung 35 is unchanged.
3. **THE GOVERNOR HOLDS.** For a redline in the gap (`Tt4_hi < Tt4_max < ` bare peak), the topped
   `Tt4_peak ≤ Tt4_max + 1e-6` at every shape and across `rho`. (The `1e-6` K margin is **not**
   arbitrary: the topping set-point solve is an Illinois root with `tol = 1e-9` on *fuel*, so the
   held-`Tt4` residual is `(dTt4/dmdot_fuel)·1e-9 ≪ 1e-6` K — the gate tolerance sits comfortably
   above the solve's noise floor, the discipline the solver-tolerance audit put on record.)
4. **THE HEADLINE — the SPLIT.** At every shape (incl. hp-only): `relief_lp ≈ 0` (below a machine
   tolerance) AND `relief_hp > 0` (strictly). Sign/existence of the differential; magnitudes
   disclaimed. The hp-only case witnesses that the split is the window mechanism, not the rung-40
   complex mode.
5. **THE MECHANISM.** On the bare accel the LP surge minimum occurs at a **lower** `Tt4` than the HP
   surge minimum AND than the `Tt4` peak (LP-min `Tt4` `<` redline `<` HP-min `Tt4`), so the clip
   window excludes the LP minimum and includes the HP one — the reason the split has the sign it
   does.
6. **THE LEVER.** `relief_lp` is machine-zero at moderate `r` but **strictly positive** in the
   fast-ramp limit (`r ≤ 0.3`) at a redline the moderate ramp cleared — the caveat that keeps the
   zero conditional.
7. **DECEL — bit-for-bit rung 45.** A topped decel march (redline above the endpoint) equals the
   bare decel march float-for-float (clip never fires).
8. **CYCLE UNTOUCHED ⇒ rung 6.** Default `build_turbojet(…).run(…)` bit-for-bit after exercising
   the governor.

---

## Concessions

- **`Tt4_max` imposed — no redline-level claim.** The absolute redline, the rebate magnitudes and
  the fast-ramp switch-on point are all disclaimed; load-bearing are the governor holding the
  redline, the surge-relief SPLIT sign, its shape-robustness, and the reduces.
- **Idealised instantaneous governor — no actuator lag.** The topping is a pure min-select with no
  new time constant, deliberately, so the finding rides on the window geometry. A lagged governor
  (a real fuel valve / a `τ_gov`) is left open; a slow-enough governor would smear the clip window
  and could reach earlier into the LP surge point — a natural next seam, not modelled here.
- **The split is sign/existence.** `relief_lp = 0` is the machine-zero of "the clip never touched
  the LP minimum," and `relief_hp > 0` is small (≤ 0.008 in the sampled band); neither magnitude is
  gated, only the differential's sign.
- **`rho` is a DISCLAIMED clock group, doubled** (inherited from rung 40). The bare surge object is
  `rho`-flat (rung 45); the governor's HP rebate carries a weak `rho` trend (the clip amount tracks
  the `rho`-loud overshoot) but it is not calibrated.
- **Reacting-gas fuel control deferred** (rung 35/43/45's concession, carried verbatim):
  `_tt4_from_f` asserts against an equilibrium gas, so the governor runs on the non-equilibrium
  gases; the finding is gas-independent and the reacting reduce is the `Tt4`-control path (rung 44,
  bit-for-bit rung 6 at the cycle).
- **Fully-choked branch, both NGVs choked, no bypass, one `eta_m`, isentropic knobs, no bleed** —
  inherited from rungs 38–45 unchanged. No **TIT-redline-with-bleed** and no **variable-stator**
  interaction is modelled (the open list's "fuel + bleed together").

---

## Anchor

`docs/plans/rung46-anchor-topping-governor.md`. The **method** is again Cohen–Rogers–Saravanamuttoo
*Gas Turbine Theory* Ch. 9 — the **acceleration fuel schedule limited by a maximum turbine
temperature** (the topping / TIT-limiting governor of the fuel control), now on rung 43's floating
`Tt4`. Rung 46's own contribution is the **surge-relief SPLIT** (enforcing the TIT redline rebates
the late non-binding HP spool but machine-zero the early binding LP one), its **window mechanism**
(the surge debit is paid on early-ramp fuel, upstream of the governor's late window), its
**shape-robustness** (incl. the mode-free hp-only), and the **fast-ramp lever** (the LP rebate
switches on once the LP surge minimum migrates above the redline).
