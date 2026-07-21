# Rung 37 — The two internal clocks: volume-filling CONFIRMS, heat-soak CORRECTS

Rungs 34–36 made the **shaft** the only dynamic element. Every component below the rotor was
**quasi-steady**: the gas path re-establishes its choked throats and its flame in milliseconds,
"instantly" relative to the spool. Rung 34 filed the omission as a single bundled concession:

> **Quasi-steady components.** The shaft is the ONLY dynamic element — no combustor volume-filling
> (`dp/dt`), no heat soak into the metal, no tip-clearance transients. These are the next dynamic
> seams; they add faster clocks *below* `τ_spool`, they do not change the `r` framing.

That sentence makes **two** claims about **two** different physical clocks, and rung 37 tests them.
The result is a **contrast** — the two clocks fall on opposite sides of `τ_spool`, so one claim holds
and the other breaks:

> **Combustor volume-filling is genuinely a fast clock** (`τ_fill ≈ 5–30 ms ≪ τ_spool`): it
> **CONFIRMS** the concession — the peak surge excursion is unmoved (lands on rung-35's `E0` to
> machine zero) — while exposing the one thing rung 34's rigid coupling hid: the **first rung where
> compressor mass flow ≠ NGV mass flow** (the plenum stores the difference).
>
> **Heat-soak is NOT a fast clock** (`τ_soak = m·c/hA ≈ 1–20 s ~ τ_spool`): it **CORRECTS** the
> concession. A second STATE — the metal temperature `Tm` — carries **thermal memory**, so the
> transient is `E(r, θ₀)`, a function of the thermal history and **not of `r` alone**. The **primary**
> cost is the **acceleration-time lag** (the CRS/Walsh-Fletcher headline). For **surge**, the modeled
> combustor gas-path sink is *protective* — `cold < hot-reslam < adiabatic`, so heat soak is **never
> worse** than the no-soak baseline; a hot reslam is merely **less protected** than a cold first-accel,
> not a hazard above it. **NOTE (the honest scope):** this is the **opposite sign** to the operational
> reslam/"bodie" surge hazard, which is heat soakage moving the working line *toward* surge — an effect
> that lives in a channel this rung does **not** model (compressor-side soak / tip-clearance), so this
> rung's combustor channel does not reproduce it.

Like every rung 7→36 this is a **diagnostic beside the cycle**: a separate entry point
(`CombustorTransient`, subclassing `SpoolTransient`), both effects **default OFF** and reduce to
rung 34/35 **bit-for-bit** (each OFF switch is an **exact dispatch**, not a stiff limit), and the
default `build_turbojet(…).run(…)` design path is untouched (bit-for-bit rung 6).

---

## Effect 1 — combustor volume-filling: the plenum (a `dp/dt` state)

### The model (the intercomponent-volume method, Fawke & Saravanamuttoo 1971)

The combustor is a control volume `V` between the compressor exit and the NGV. Mass accumulates in
it, so the burner-exit pressure `pt4` becomes a **state**:

```
m_cv = pt4·V/(R·Tt4) ,        dm_cv/dt = ṁ_c + ṁ_fuel − ṁ_NGV
```

With `Tt4` quasi-steady (mass storage only — thermal storage in the gas is a further, smaller seam),

```
dpt4/dt = (R·Tt4/V)·(ṁ_c + ṁ_fuel − ṁ_NGV) .
```

### What is genuinely new: the compressor / NGV mass-flow DECOUPLING

Rung 34 closed the flow by tying compressor mass flow **rigidly** to the NGV: `pt4 = π_b·π_c·pt2`
with `ṁ_c = ṁ_NGV` enforced by the NGV-continuity root-find. **The plenum breaks that.** `pt4` is
now the state; the compressor delivers whatever its speed line gives at the imposed back-pressure,
and the NGV drains whatever the choke passes — their **difference charges the volume**:

- **Compressor side (run from BACK-PRESSURE — the structural novelty):** at corrected speed `n` the
  required pressure ratio is imposed by the plenum, `π_c = pt4/(π_b·pt2)`. Invert the forward speed
  line `π_c(n, m)` for the corrected flow `m` (monotone-decreasing in `m` on the operable surge side)
  ⇒ `ṁ_c = m·ṁ_corr,d·pt2/√Tt2`, `f = ṁ_fuel/ṁ_c`, `Tt4 = burner(Tt3, f)` (the rung-35 forward burner).
  This is a **third** use of the map — not forward (rung 34) nor inverted-for-`n` (rung 32), but
  **inverted-for-`m` at a given `π_c`**.
- **NGV side:** `ṁ_NGV = A4·pt4·MFP*(Tt4, f)/√Tt4` (the choke, `∝ pt4`).

`ṁ_c ≠ ṁ_NGV` off equilibrium — the **first rung** where the two mass flows differ. The nondimensional
plenum clock is `r_v ≡ τ_fill/τ_spool`; `dpt4/ds` is scaled so the linearized drain rate is `1/r_v`
(the coefficient fixed at the design station-4 state, so `τ_fill` rides slightly off-design as a
real fixed `V` would). `r_v` is a **single disclaimed clock ratio**, the `I`/`L`/`τ_res` concession
in a new guise.

### The finding — CONFIRMATION (peak) + the decoupling (structural)

At `r → 0` (a fuel step at a frozen spool) the plenum fills to its full quasi-steady `pt4` **before
`ν` can move**, so the operating point lands on the **exact** rung-34/35 constant-`N` point:

```
peak excursion (finite plenum)  =  E0 (rung 35)  to machine zero,  INDEPENDENT of r_v.
```

Measured (fast gas, `Tt4` 1100→1400, three surge shapes, `r_v ∈ {0.03, 0.1}`): `peak − E0 =
−0.0000%` in every cell. **So volume-filling confirms `E0`** — the `r → 0` peak is unmoved, full stop.
The **load-bearing content is the decoupling itself**: `max|ṁ_c + ṁ_fuel − ṁ_NGV|/ṁ_NGV ≈ 22%`
during the fill — a genuinely new degree of freedom the rigid coupling could not represent. This is the
anti-tautology hook: not "a fast clock relaxes fast" (vacuous), but "the peak survives a finite
combustor **and** the compressor unlocks from the NGV."

---

## Effect 2 — heat-soak: the metal temperature `Tm` (a slow second STATE)

### The model (CRS Ch. 9 / Walsh & Fletcher — heat soakage)

The combustor + turbine metal has thermal mass. During a transient it exchanges heat with the gas,
so it becomes a second state sitting **between burner-exit and turbine-inlet**:

```
Tt4,turb = Tt4,burner − G·(Tt4,burner − Tm) ,   G = hA/(ṁ4·cp)      (gas → cold metal heat sink)
m·c·dTm/dt = hA·(Tt4,burner − Tm)   ⇒   dTm/ds = (Tt4,burner − Tm)/r_m ,   r_m ≡ τ_soak/τ_spool
```

`G` is the heat-extraction gain (an NTU-like fraction, held constant — disclaimed magnitude); `r_m`
is the metal clock ratio. Both **default 0 ⇒ OFF**. The NGV and the turbine see the depressed
`Tt4,turb`; the burner balance and `dTm/ds` use `Tt4,burner`. The compressor closure is rung-35's
NGV-continuity root-find with `Tt4,turb` in the choke and the turbine.

**Equilibrium ⇒ `Tm = Tt4,burner` ⇒ `Q ≡ 0`.** So heat-soak **never moves the running line** — it is a
**purely transient** effect (a clean structural reduce: the rung-35 equilibrium is untouched).

### The finding — CORRECTION: `E(r) → E(r, θ₀)`, history-dependence, and the accel-lag

Heat-soak makes the transient depend on the **initial metal state** `θ₀`, not on `r` alone:

- **Surge is PROTECTED, robustly.** A cold metal depresses `Tt4,turb` → the NGV (a colder sonic
  throat) passes **more** corrected flow → higher `φ` → **away** from surge, so the peak excursion
  **falls below** the adiabatic (rung-34/35) value. Ordering **`cold < hot-reslam < adiabatic`** holds
  across every `G ∈ {0.05, 0.1, 0.2} × r_m ∈ {1, 3, 10} ×` 3 shapes. rung-34/35's adiabatic combustor
  is therefore the **conservative worst case** — a CONFIRMATION of the bound with the mechanism named.
  (The rung-36 counter-channel — a slower spool parks longer at low `ν` where the margin is thin —
  does **not** flip the peak, which occurs early near `ν₀`; the peak is nearly `r_m`-independent.)

  | shape (G=0.1, r_m=3) | adiabatic (rung 35) | cold first-accel | hot reslam |
  |----------------------|---------------------|------------------|------------|
  | surge_flow           | 10.156%             | 9.224%           | 9.660%     |
  | surge_pressure       | 13.062%             | 11.827%          | 12.436%    |
  | surge_tilted         | 11.389%             | 10.333%          | 10.835%    |

- **History-dependence: a hot reslam is LESS protected than a cold accel.** A re-acceleration from
  **hot** metal (as after a chop-and-slam) has little heat sink, so its excursion sits **just below the
  adiabatic** value — above the cold first-accel but still below the no-soak baseline. `E` genuinely
  depends on `θ₀`; a single-state `E(r)` structurally cannot express it. **This is NOT the operational
  bodie surge hazard** — that hazard is the *opposite sign* (heat soakage reducing surge margin) and
  lives in an **unmodeled compressor-side channel** (tip-clearance / compressor heat soak). This rung's
  combustor gas-path sink only ever *helps* surge; a hot reslam is the least-helped case, not a
  above-baseline hazard. The overlap with the real bodie is the **history-dependence**, not the sign.

- **The PRIMARY effect is the ACCEL-TIME LAG.** The cold metal steals turbine work, so a cold
  acceleration is **~2.5× slower** (`t_accel ≈ 5.55` vs adiabatic `2.15` at `G=0.1, r_m=3`) and grows
  without bound as `τ_soak` grows (`r_m = 10`: the cold accel does not reach 99 % within `s = 10`).
  This is the thrust-response lag every transient-performance text leads with. A hot reslam even gets a
  brief early **boost** (`Tm > Tt4,burner` at first → the metal *releases* heat), so it is ≈ adiabatic-fast.

So heat-soak **refutes** "they do not change the `r` framing": `τ_soak ~ τ_spool` is not a fast clock,
and the second state adds a whole axis (`θ₀`) plus a large lag.

---

## Reduce-to-prior contract (the spine)

Each OFF switch is an **exact dispatch**, not a numerical limit (the advisor's requirement — a stiff
`V → 0` would be a limit):

- **`r_v = 0` (no plenum) ⇒ rung 35 bit-for-bit.** With no volume the plenum state is not created;
  `integrate`/`equilibrium` **dispatch to `super().integrate_fuel`/`equilibrium_fuel`** (the rung-33
  "leave the path literally unchanged" move) — the rung 31–36 suites pass unchanged.
- **`G = 0` (`hA = 0`, no heat sink) ⇒ rung 35 bit-for-bit.** `Q ≡ 0` ⇒ `Tt4,turb ≡ Tt4,burner` ⇒ no
  metal state; same dispatch.
- **NON-TAUTOLOGICAL reduce (plenum).** The plenum **equilibrium** (`dν/ds = 0` AND `dpt4/ds = 0`)
  reproduces rung 35's `equilibrium_fuel` — through the **back-pressure-inverted** compressor closure,
  a genuinely different code path than rung 35's NGV-continuity root-find (two closures, one point).
- **NON-TAUTOLOGICAL reduce (heat-soak).** The heat-soak equilibrium reproduces rung 35 because
  `Tm → Tt4,burner ⇒ Q = 0` — verifying heat-soak is transient-only (the running line is unmoved).
- **Cycle untouched ⇒ rung 6 bit-for-bit.** Separate entry point; the design run is not perturbed.

---

## Verification gates (`tests/test_rung37.py`)

1. **REDUCE — both OFF ⇒ rung 35.** `r_v = 0` and `G = 0`: `equilibrium`/`integrate` equal the
   rung-35 `equilibrium_fuel`/`integrate_fuel` outputs bit-for-bit (dispatch, not re-solve). The rung
   31–36 suites are the bit-for-bit witness.
2. **PLENUM equilibrium == rung 35 (non-tautological).** With `r_v > 0` the plenum equilibrium
   reproduces `equilibrium_fuel` (`ν, π_c, τ_t, ṁ_air`, and `ṁ_c = ṁ_NGV` at the fixed point) — via
   the back-pressure closure, machine-zero at design, tight on a sweep.
3. **PLENUM finding — peak = `E0`, and the DECOUPLING.** The `r → 0` plenum peak excursion equals
   rung-35's algebraic `E0` to tolerance, **independent of `r_v`** (≥2 values), across ≥3 shapes; and
   the transient exhibits `max|ṁ_c − ṁ_NGV| > 0` (the mass-flow split is real, not machine-zero).
4. **HEAT-SOAK equilibrium == rung 35 (transient-only).** With `G > 0` the equilibrium reproduces
   rung 35 (because `Q = 0` at the fixed point) — heat-soak does not move the running line.
5. **HEAT-SOAK finding — `cold < hot-reslam < adiabatic`, SHAPE- and KNOB-ROBUST.** The peak surge
   excursion of a cold first-accel is below a hot reslam is below the adiabatic (rung-35) value, same
   ordering across ≥3 shapes × ≥2 `G` × ≥2 `r_m` (magnitudes disclaimed).
6. **HEAT-SOAK — the accel-time LAG.** A cold acceleration reaches its target `ν` **later** than the
   adiabatic one (thrust-response lag), monotone in `G`; the hot reslam is ≈ adiabatic-fast.
7. **CYCLE UNTOUCHED.** The default design run is bit-for-bit rung 6; constructing a
   `CombustorTransient` does not perturb it.

---

## Concessions

- **Two disclaimed clock ratios `r_v`, `r_m` (and `G`).** Only the nondimensional trajectories and the
  ratios are claimed; wall-clock seconds and the volume/`hA`/metal-mass magnitudes are disclaimed — the
  `I`/`L`/`τ_res` concession, twice. Load-bearing claims (peak = `E0`; `cold < hot < adiabatic`; the
  accel-lag sign) are verified robust to them.
- **Effects modeled SEPARATELY (each with the other off).** The contrast is the point; the combined
  3-state model (`ν, pt4, Tm` together) is a further seam — the interaction is not claimed.
- **Plenum: mass storage only.** `Tt4` is quasi-steady in the volume (the standard first-cut); a full
  energy-storage plenum (`d(m_cv·u)/dt`) is a further seam and does not change the peak = `E0` result.
- **Heat-soak: constant `G`, one lumped metal node.** A flow-varying `hA` (Reynolds) and a distributed
  liner/turbine metal are further seams; the sign is verified robust to `G`. `Tm = Tt4,burner` at
  steady (adiabatic wall at equilibrium) is what makes it transient-only.
- **No surge line crossing.** Inherited rung 32/36: the excursion is a signed displacement on a
  representative map; here heat-soak *reduces* it (protective) — the crossing is not claimed.
- **Fuel control; quasi-steady elsewhere; single spool; isentropic knobs; NGV choke.** Inherited from
  rungs 31–35. Tip-clearance transients and two-spool dynamics remain further dynamic seams.
- **Diagnostic beside the cycle.** The production run is untouched; both effects are read-only extras
  on a separate entry point.

---

## Anchor

`docs/plans/rung37-anchor-combustor-dynamics.md`. The **volume-filling** method is the
**intercomponent-volume** model (Fawke & Saravanamuttoo, *Digital computer simulation of the dynamic
response of a twin-spool turbofan*, 1971; CRS *Gas Turbine Theory* Ch. 9): a control volume between
compressor and turbine whose pressure is a state driven by the compressor/turbine mass-flow imbalance.
The **heat-soak** method is the standard transient-performance heat-soakage model (CRS Ch. 9; Walsh &
Fletcher, *Gas Turbine Performance*, transient chapter): a lumped metal thermal capacitance that lags
the gas, slows the acceleration, and makes the transient **thermal-history-dependent**. (The texts'
**hot reslam / bodie** surge hazard — a re-accel from a hot engine being more surge-critical — is driven
by the **compressor-side** soak / tip-clearance channel, *not* modeled here; this rung's combustor
gas-path sink is the opposite, protective sign — see § Effect 2.) The **reduce** (both OFF ⇒ rung 35, and the two equilibria == rung 35 via the
independent closures) is the rigorous, non-tautological anchor — two code paths onto one operating
point, exactly as rungs 31–36 did for their solves.
