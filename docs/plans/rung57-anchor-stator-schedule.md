# Rung 57 anchor — the stator schedule on the TRANSIENT plant

The probes that fixed the instrument and the sign **before** the predictions, the predictions
as written, and their scoring. Same discipline as rungs 37/44/46/49/56: probe first (the
project has been wrong about a sign often enough that guessing is not a method), pre-register
only once the instrument is trusted, then score honestly.

Everything below was run from `M:\claud_projects\temp\rung57` against the repo as of
`807de42`, on the CPG gas, `FLIGHT = (250 K, 50 kPa, M0 0.85)`, `π_LPC/π_HPC/Tt4 = 3/6/1500`,
the rung-49 `REAL` losses, the rung-53 shapes
`LP = (a .20, b .05, σ .1, l .7)`, `HP = (a .08, b .15, σ .1, l 1.0)`, both with
`φ_surge = 0.55`, and the rung-45 accel ramp `Tt4 1000 → 1400`.

---

## The probes (instrument-fixing — NOT evidence for the predictions)

### A — does a constant `vsv` even run on the transient plant?

The first time any transient closure sees `v ≠ 0`; rung 53's own `phi_max` docstring says so
("the stator is a steady rung"). `phi_max` *shrinks* with `v` and `ψ` is depressed everywhere,
so the rung-40/43 bracket was the expected first failure. It survives to `v = 0.45`:

| `v_LP` | min `φ_L` | `M_φ,L` | `M_i,L` | min `φ_H` | `M_i,H` |
|---|---|---|---|---|---|
| 0.00 | 0.73547 | +0.18547 | +0.45850 | 0.86120 | +0.65701 |
| 0.10 | 0.70228 | +0.18096 | +0.49425 | 0.86059 | +0.65619 |
| 0.20 | 0.67088 | +0.17539 | +0.52761 | 0.86009 | +0.65552 |
| 0.45 | 0.59988 | +0.15900 | +0.60118 | 0.85928 | +0.65442 |

Rung 53's currency split, replayed dynamically: `M_φ` **shrinks** on closing, `M_i` **grows**.
A constant setting is therefore *rung 53 on the transient plant* — a confirmation, and the
reason this rung had to be about something else.

### B/C/D — CONTAMINATED, and the contamination was the finding

Probes B, C and D marched the scheduled leg from the **bare** equilibrium while the march
itself used the scheduled maps: the run started off its own running line, so a start transient
was folded into every "credit". The CONST legs (permanent map, `equilibrium` sees it) and
probe A were always consistent. **Their erosion / r-sweep / residual numbers are discarded.**

What survives from them, and is used below: probe C's **steady** leg (rung 53's own matcher,
`VariableStatorMatcher.stator_margin`) —

| `Tt4` | `M_i(0)` | `M_i(0.20)` | net | erosion |
|---|---|---|---|---|
| 1000 | +0.52472 | +0.59435 | +0.06963 | 0.6518 |
| 1200 | +0.65514 | +0.72262 | +0.06748 | 0.6626 |
| 1400 | +0.76869 | +0.83442 | +0.06573 | 0.6713 |
| 1500 | +0.81818 | +0.88319 | +0.06500 | 0.6750 |

and probe D's **ds-convergence**, which is a property of the march and survives the
contamination: the erosion is identical to four decimals at `ds` = 0.02 / 0.01 / 0.005.

### E — WRONG, and caught

`SchedTransient` overrode `_close_fuel` only, so `equilibrium`/`fuel_for_Tt4` (which run
through rung 40's `_close`) saw the bare maps. It reported the HP stator **debiting its own
spool** by −0.0205. With both closures armed (probe G) the sign reverses to **+0.0625**.
Recorded because the ladder's rule is that a caught error is published, not deleted.

### G — the consistent instrument

Both closures armed ⇒ the start IS the scheduled running line. At `r = 0.5`:

| armed | `ΔM_i,LP` | `ΔM_i,HP` | `nu0_L` | `nu0_H` |
|---|---|---|---|---|
| — (bare) | 0 | 0 | 0.75574 | 0.78979 |
| LP `v` = 0.20 | +0.05272 | +0.00062 | 0.81662 | 0.78979 |
| HP `v` = 0.20 | +0.02929 | +0.06252 | 0.75608 | 0.86324 |
| both 0.20/0.10 | +0.07051 | +0.03813 | 0.81691 | 0.83445 |

Both stators credit both spools; the credits superpose to 1.7 % (LP 0.0527+0.0166 = 0.0693 vs
0.0705 measured). **The superposition has no derivation behind it and is not claimed** — one
line, no gate.

`nu0_L` 0.75574 → 0.81662 is rung 53's "paid in SHAFT SPEED", now as an initial condition:
a state-fed schedule has already acted before `s = 0`. That observation set P3/P4.

### I — a PRE-EXISTING defect, logged not fixed here

The rung-40 `_close` bracket's high wall `min(2.5, φ_max,LP·n_L)` drives the **HP** face far
outside its own map: at `m_L` = 2.11, `φ_H` = 4.12, `ψ_H` = −3.09, `τ_hpc` = −2.18 and
**`Tt3` = −648.75 K**. `gas.pr_c(-648.75)` raises a float to a fractional power on a negative
base and Python returns a **complex**, which reaches `glo < 0.0 < ghi` as a `TypeError` — while
every caller in the ladder catches `AssertionError` only. Stator-independent (it reproduces at
`v = 0` on a flat-η map). Out of rung 57's scope; see § the open-tasks entry.

---

## The predictions, as written (before the shipped class existed)

**P1 — NO CLOCK.** The erosion fraction `1 − ΔM_i/v` at a constant `v` = 0.20 lies inside a
**2-point band** across `r ∈ [0.1, 2.0]` (a 20× range), and rung 53's design-point closed form
`1 − 1/(2+l)` predicts it to within **10 %**.

**P2 — the NON-TAUTOLOGY.** Over that same `r` range the margin the lever is credited against
must swing by **more than 30 %**. (Without this P1 is "a small number is small".)

**P3 — NOT an initial-condition device.** START-ONLY delivers **less than 0.35** of the FULL
credit at every `r`, and its share **falls** with `r`.

**P4 — the state-fed schedule SELF-CANCELS.** FULL < RAMP-ONLY at every `r` (the ratio strictly
below 1), and the ratio **falls** as `r` rises.

**P5 — rung 53's TWO EXACT ZEROS BREAK.** At a fixed transient state, `v_LP` moves `φ_HP` by
more than `1e-3` (rung 53: exactly `+0.000e+00`), and the break **survives a flat-η island** —
i.e. it is *not* the η-mediated channel rung 53 identified.

**P6 — the REDUCE.** Both schedules `None` ⇒ the march is bit-for-bit rungs 43–52 on every
recorded key; and a schedule that returns 0.0 everywhere is bit-for-bit too (the swap machinery
is inert at `v == 0`, so the reduce is not merely a skipped branch).

---

## Scoring

Against the shipped `ScheduledStatorTransient` (the probe numbers above are the instrument;
these are the result, and they are the numbers `docs/rung57-spec.md` publishes).

### P1 — claim **HIT**, band **MISSED on one shape**

| shape | erosion band over `r ∈ [0.1, 2]` | width | closed form `1 − 1/(2+l)` | max err |
|---|---|---|---|---|
| flow/press (`l` = 0.70) | 0.6438 – 0.6543 | **1.05 pts** | 0.6296 | **3.9 %** |
| tilted (`l` = 0.85) | 0.6346 – 0.6602 | **2.56 pts** | 0.6491 | **2.2 %** |

The pre-registered band was **2 points**: hit on the primary shape, **missed on `tilted`**
(2.56). The 10 % closed-form clause is hit on both, with room to spare. Scored as rung 56's P3
was — *band missed, claim held* — and the gate is written to the measured bands, not the
predicted one, with the miss stated in its docstring.

### P2 — **HIT**

Bare-margin swing **52.0 %** (flow/press) and **61.8 %** (tilted) across the same sweep, i.e.
**> 10×** the erosion spread in both cases. The dynamics dominate the margin and are inert to
the lever — which is what makes P1 content rather than "a small number is small".

### P3 — **HIT**, and harder than written

`share_start` = 0.271 / 0.120 / 0.030 / −0.030 / −0.068 at `r` = 0.1 / 0.25 / 0.5 / 1 / 2.
Under the predicted 0.35 everywhere, monotonically falling, and it **changes sign** — the
higher starting speed is a *debit* on a slow ramp. The prediction only asked for < 0.35 and
falling.

**This refutes the hypothesis the probe-G observation suggested** (that a state-fed schedule is
essentially an initial-condition device). It has acted before `s` = 0, but that head start is
not where the credit comes from.

### P4 — **HIT**

`FULL/RAMP-ONLY` = 0.896 / 0.803 / 0.769 / 0.756 / 0.754 — strictly below 1 at every ramp rate
and monotonically deepening, with `nu0_armed` > `nu0_bare` (0.8166 vs 0.7557) at every row as
the stated mechanism.

### P5 — **HIT, both halves**

At the fixed state (`nu_L` 0.775960, `nu_H` 0.806924, `mf` 0.015917):

| island | lever | `Δφ_LP` | `Δφ_HP` | `Δn_HP` | `ΔTt25` |
|---|---|---|---|---|---|
| shaped-η | `v_LP` | −6.647e-02 | −9.608e-03 | +1.531e-02 | −13.05 K |
| shaped-η | `v_HP` | −1.278e-01 | −1.653e-01 | −6.458e-03 | +5.72 K |
| flat-η | `v_LP` | −6.609e-02 | −9.289e-03 | +1.573e-02 | −13.37 K |
| flat-η | `v_HP` | −1.179e-01 | −1.617e-01 | −6.010e-03 | +5.31 K |

Both of rung 53's `==`-reported zeros break (both `> 1e-3`), and the flat-η island — rung 53's
own zeroing control — reproduces the shaped arrow to **within 5 %**, so the break is **not**
η-mediated. `ΔTt25` names the channel.

Both islands are toggled at the **same** state, supplied through `arrow_toggle(state=…)`. That
is required, not convenient: the two islands have different running lines, so letting each find
its own minimum would compare two toggles at two different states.

### P6 — **HIT**

`no schedule`, `zero schedule`, and `zero schedule on BOTH spools` are each **bit-for-bit**
(`max|diff|` = 0.000e+00 over 171 points × 9 keys) against `TwoSpoolFuelTransient`, and
`_arm` at `v` == 0 returns the design map **object** (`is`).

---

## By-product — a pre-existing defect, fixed

Probe I's complex leak (above) is real and stator-independent. The fix shipped with this rung
is one assertion in each of the two transient closures' `g`, converting a non-real residual
into the ladder's documented off-map `AssertionError`.

**The first attempt was wrong and is recorded.** Guarding on `Tt25 > 0 and Tt3 > 0` *inside*
`ev` broke a previously-working path: with a **shaped** η island, `eta_c_at` collapses at
`φ_H` ≈ 4.26 and cancels the negative enthalpy back to a positive one, so `pr_c` stays real and
the bracket's high wall — though it sits at `Tt3` = −721 K and is physically meaningless — is a
usable endpoint that `_close` relies on. Rejecting it broke every shaped-map `equilibrium`. The
shipped guard tests the *residual*, not the temperatures, so it fires only where the arithmetic
has actually left the reals.

## Where the probes live

`M:\claud_projects\temp\rung57\` — `probeA_constant_vsv.py`, `probeB_schedule.py`,
`probeC_decide.py`, `probeD_blocking.py`, `probeE_split.py` (the wrong one),
`probeF_hp_const.py`, `probeG_consistent.py`, `probeH_split.py`, `probeI_leak.py`,
`probeJ_arrow.py`, `score.py`. Regenerable; nothing in the repo depends on them.
