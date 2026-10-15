# Rung 91 anchor — TURBINE COOLING AIR: HP-compressor air that rejoins at the turbines (pre-registered 2026-10-15 — § 3)

**Seam:** rung 90 § *The next seam*: *"Turbine cooling air — HP-compressor air that skips the HP
turbine's nozzle and rejoins downstream: a bleed that changes the turbines' flow and work, not only
the compressors'."* Rung 90's concession: *"cooling air that re-enters at the turbine is not
modelled (it changes the turbine flow and work — a different sink)."*

**The user's choices (2026-10-15):** TWO streams — vane cooling (rejoins before the HP turbine rotor,
so it works there) and blade cooling (rejoins after it, so it works only in the LP turbine); the
whole seam in one rung, as rung 90 — the steady two-shaft engine, the throttle-slam engine, the
protection valve beside it, and a sandbox knob; shaft-power offtake left to its own rung.

Shipped as KNOBS (the sandbox direction): every prediction below is scored HIT/MISS, and a miss is
a finding.

**Status:** § 0 measured on a scratch crate (`W:\temp\claude\jet-rung91-probe`, not in the repo),
on EXISTING code only — no cooling code existed when it ran. § 1–§ 3 written BEFORE any cooling
number was computed by cooling code. What § 0 saw is a GIVEN and scores no credit.

---

## 0. GIVENS — measured on the uncooled plant before anything was written

Rig A (rungs 42/90's): CPG gas (γ 1.4 / 1.3, `cp` 1004 / 1239, so `κ = cp_c/cp_t` = 0.8103),
`π_LPC` 3 × `π_HPC` 6, `Tt4` 1500 K design, rung 42's losses (`η_HPT` 0.92), 250 K / 50 kPa /
Mach 0.85, flat maps and the `flow/press` shapes, `Tt4` 1500 → 650 K. The running line is rung 90's
G1.

**G1 — the HP turbine sits at ONE point on the whole line.** Both throats choked on the CPG gas
(rung 31's `(★)`): `τ_HPT` = 0.8374, `π_HPT` = 0.4305 at every `Tt4`, flat and shaped. The model
below needs its local slopes: `σ = d ln τ_HPT / d ln π_HPT = η g π^g / τ` = **0.2087** and
`ρ = τσ/(1−τ)` = **1.0749** (`g = (γ_t−1)/γ_t`). Station temperatures (flat):

| `Tt4` | 1500 | 1300 | 1100 | 1000 | 900 | 800 | 700 | 650 |
|---|---|---|---|---|---|---|---|---|
| `Tt3` | 709.8 | 651.6 | 593.9 | 565.3 | 536.8 | 508.4 | 480.2 | 466.1 |
| `Tt45` | 1256.1 | 1088.6 | 921.1 | 837.4 | 753.7 | 669.9 | 586.2 | 544.3 |
| `κ Tt3/Tt4` | 0.383 | 0.406 | 0.438 | 0.458 | 0.483 | 0.515 | 0.556 | 0.581 |
| `κ Tt3/Tt45` | 0.458 | 0.485 | 0.523 | 0.547 | 0.577 | 0.615 | 0.664 | 0.694 |

**G2 — rung 90's measured customer response is the baseline** (`d ln φ / dc`, one-sided `c` 0.02,
rung 90's shipped code): flat 1500 K LP −0.429 / HP +0.218, 1000 K −0.081 / +0.388, 650 K
+0.277 / +0.602; its § 1 model within 0.0095 everywhere (rung 90 § 3). A cooling stream is a
customer bleed of the same share PLUS the air put back; § 1's model reduces to rung 90's when the
put-back terms are struck (`κθ = 0`, `p = 0`), so the new content is exactly the put-back.

## 1. THE DEVICE — where the cooling air enters (derived before coding)

Per unit HPC delivery, with the customer share `c` (rung 90) and the cooling shares `ε₁` (vane)
and `ε₂` (blade), `ε = ε₁ + ε₂`, all shares of the HPC delivery and ADDITIVE:

```
HPC delivers 1 · burner air (1 − c − ε) · HPT nozzle throat (A4) passes the burner gas G = (1−c−ε)(1+f)
mixer 1 (after the A4 throat, before the HPT rotor): + ε₁ at Tt3  →  G + ε₁ at Tt41
HPT rotor works on G + ε₁ from Tt41 to Tt44
mixer 2 (after the rotor, before the LPT nozzle throat A45): + ε₂ at Tt3  →  G + ε at Tt45
LPT and nozzle pass G + ε
```

**The mixers** are adiabatic and keep total pressure (Mattingly's coolant mixers): `G h_t(Tt4) +
ε₁ h_c(Tt3) = (G+ε₁) h_t(Tt41)`, likewise for mixer 2, on the SAME enthalpy zero the burner's own
balance uses (CPG: `h = cp T` from 0 K on both sections). The fuel/air ratio is diluted at each
(`f41 = f·air/(air + ε₁)`, …) and every hot-gas call downstream carries the diluted value. **A mixer
takes two streams**, so it is not a `apply(state, gas)` component — the same declared divergence as
the Turbine and the Nozzle — and it carries its own mass and energy assertion.

Where `ε` enters — and the one place it breaks every earlier two-spool rung's structure:

1. **The HPT pin — NEW, the structural novelty.** Rung 31's `(★)` between the A4 and A45 throats
   held the HP turbine at one point (G1). The A45 throat now passes the coolant too, at the mixed
   temperature: `π_HPT √(Tt4/Tt45) = (A4/A45)·(G+ε)/G`. **The HP turbine moves.** The LPT pin is
   untouched in form (same flow through A45 and A8), fed with the mixed `Tt45` — so on the CPG gas
   `τ_LPT`, `π_LPT` stay EXACTLY where they were.
2. **The steady cascade loses its triangle.** Rungs 38–90 solved turbines from `(Tt4, f)` alone,
   then `Tt25`, then `Tt3`. Both mixers need `Tt3` before the HPT pin can be solved, and `Tt3`
   comes from the HP balance, which needs the HPT: a loop `Tt3 → mixers → π_HPT → Tt45 → Tt25 →
   Tt3`. It is closed by a fixed point on `Tt3` inside the cascade (construction claim, § 3). **The
   transient closure does not loop**: there `Tt3` comes from the compressor maps at the trial
   state, before any turbine is touched.
3. **The HP shaft balance:** `h(Tt3) − h(Tt25) = η_m (G+ε₁)(h_t(Tt41) − h_t(Tt44))`.
4. **The LP shaft balance:** `(1−b)[h(Tt25) − h(Tt2)] = η_m (1−b)(G+ε)(h_t(Tt45) − h_t(Tt5))`.
5. **The face referrals** (†) and (‡) pick up `1/(1−c−ε)` (rung 90's `1/(1−c)`).
6. **The forward rebuild** books both extractions at station 3 and both returns at their mixers;
   new stations `41` and `44` in the table.
7. **Thrust:** the cooling air was captured and LEAVES THROUGH THE NOZZLE (it is not drag, as the
   customer air is): per inlet air `F/ṁ₂ = (1−b)[(G+ε)V9 + pressure term − (1−c)V0] − (b +
   (1−b)c)·V0`, written so `ε = 0` is rung 90's line.

Untouched: the A4 throat's relation (the vane coolant enters downstream of it); the compressor
maps. The HPT efficiency map's corrected speed is referred to the ROTOR inlet `Tt41` (= `Tt4` at
`ε₁ = 0`). And on the TRANSIENT plant the same terms enter rung 90's four cells: the closure and the
fuel closure through the burner share `(1−c−ε)` only; the power loop and the thrust tail through
the new turbine pins and balances (3, 4, 7).

**Where it lives.** On the design engine (`TwoSpoolEngine`), beside the customer bleed, as a pair
of shares — copied onto rung 38's core, so every sibling a reader builds keeps it. The design run
IGNORES it: the hardware is captured with cooling OFF, and the knob is cooling MODULATED on that
fixed hardware (concession, § Concessions of the spec). Every lower solver — rung 90's own bodies
included, and everything that already refuses a customer bleed — **refuses** active cooling in
words. **The perfect gas only:** a mixer on a reacting or equilibrium gas would re-react the
diluted mixture; those gases are refused in words.

**Bounds**, in words and checked at the ROOT (rung 90's lesson 2): each share in `[0, 0.25)`, and
`c + ε < 0.5` at the converged point.

**Reduce (the spine).** Cooling inactive (`None`, both shares 0) dispatches to the earlier code
VERBATIM: rung 90's bodies when the customer bleed is active, else rung 42's / rung 62's (→ rung
39 / 57). Bit for bit, gated, with a counter per path; no oracle and no CLI golden may move; the
default design run stays rung 6.

### The linear model (the prediction instrument)

Flat maps, CPG, fixed `Tt4`, `b = c = 0`, both throats choked. With `μᵢ = εᵢ/(1+f)`, `θ = Tt3/Tt4`,
`θ' = Tt3/Tt45` (uncooled), `σ`, `ρ` from G1, and rung 90's `a`, `t_L`:

```
HPT pin   : p ≡ d ln π_HPT = [μ + ½ μ₁(κθ − 1) + ½ μ₂(κθ' − 1)] / (1 − σ/2)        (d ln τ_HPT = σ p)
HP work   : h = −ε + μ₁ κθ − ρ p          (per HPC air: the burner gas lost, vane enthalpy returned,
                                           and the expansion the back-pressured A45 takes away)
LP work   : ℓ = −ε + μ₁ κθ + σ p + μ₂ κθ' (the coolant enthalpy, and the HP's untaken drop)
compressors (rung 90's algebra, its −c replaced by h and ℓ, its referral c by ε):
    d ln φ_L = a_L ℓ + a_H (h − t_L ℓ) + ε − ½ ℓ
    d ln φ_H = (a_H − ½)(h − t_L ℓ) + ½ t_L ℓ + ε
```

(With `h = ℓ = −c`, `ε → c` it is rung 90's model exactly; G2 validates that half.)

**What it says, before any cooling code.** Returning the air does NOT soften the customer bleed's
signature — it **reverses the HP arrow at high power**. The coolant rejoins ahead of the next
choked throat, so the A45 throat must pass more gas: its back-pressure rises (`p ≈ +0.75`), the HP
turbine expands less, and the HP spool loses MORE work than if the air had been thrown overboard
(`h` ≈ −1.43 vane, −1.85 blade, against rung 90's −1). The LP turbine gets the HP's untaken drop
and the coolant's enthalpy back (`ℓ` ≈ −0.47 / −0.39), but the HP's pressure-ratio collapse
starves the LP face (the `a_H` term) and **the LP is pushed toward surge at every power**, the
customer bleed's low-power relief gone. Blade cooling is worse than vane cooling on both spools at
every point: it does no HP work at all, yet back-pressures the HP turbine as much.

Model values on the line (flat), per unit share:

| `Tt4` | 1500 | 1400 | 1300 | 1200 | 1100 | 1000 | 900 | 850 | 800 | 750 | 700 | 650 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| vane `p` | +0.751 | +0.759 | +0.767 | +0.777 | +0.788 | +0.801 | +0.817 | +0.826 | +0.836 | +0.848 | +0.861 | +0.876 |
| vane LP | −1.028 | −0.955 | −0.876 | −0.790 | −0.698 | −0.598 | −0.489 | −0.432 | −0.372 | −0.309 | −0.244 | −0.177 |
| vane HP | −0.239 | −0.185 | −0.125 | −0.059 | +0.015 | +0.096 | +0.187 | +0.237 | +0.289 | +0.344 | +0.402 | +0.464 |
| blade `p` | +0.792 | +0.800 | +0.810 | +0.821 | +0.834 | +0.850 | +0.868 | +0.879 | +0.891 | +0.905 | +0.920 | +0.938 |
| blade LP | −1.609 | −1.536 | −1.458 | −1.373 | −1.281 | −1.182 | −1.075 | −1.018 | −0.959 | −0.898 | −0.835 | −0.769 |
| blade HP | −0.624 | −0.562 | −0.492 | −0.415 | −0.330 | −0.234 | −0.125 | −0.066 | −0.003 | +0.065 | +0.137 | +0.215 |
| rung 90 LP | −0.438 | −0.379 | −0.315 | −0.246 | −0.171 | −0.089 | +0.001 | +0.049 | +0.099 | +0.153 | +0.209 | +0.268 |
| rung 90 HP | +0.215 | +0.240 | +0.269 | +0.301 | +0.339 | +0.382 | +0.432 | +0.460 | +0.490 | +0.522 | +0.556 | +0.594 |

HP crossings (linear interpolation): **vane ≈ 1120 K**, **blade ≈ 798 K**. No LP crossing in the
band for either stream (rung 90's was 910 K).

## 2. THE READINGS

**Steady (rig A).** `d ln φ / dε` per stream, Richardson-extrapolated to `ε → 0` from one-sided
differences at 0.01 and 0.02 (`2 D(0.01) − D(0.02)`: the model is first-order and the slopes here
are up to 2× rung 90's, so its one-sided 0.02 would carry curvature the model does not); `d ln
π_HPT / dε` the same way; `τ_LPT`, `π_LPT` against the uncooled line; thrust per inlet air and TSFC
at `ε` 0.05 per stream against the customer bleed at `c` 0.05 (reported, not scored); `Tt41` and
`Tt45` (reported).

**Transient (rig B = rung 90's, i.e. rung 64's).** CPG, shaped/tilted × `r` ∈ {0.25, 0.5, 1}, `ds`
0.005, `Tt4` 1000 → 1400 K, settle 1.2, rung 64's `φ` valve (`b_max` 0.10, no stators, no fuel
leg), set points `sm` ∈ {0.30, 0.34, 0.38, 0.42}; vane `ε₁` 0.05 alone, blade `ε₂` 0.05 alone, and
the customer bleed `c` 0.05 (rung 90's, re-run here for the equal-length comparison). Readings as
rung 90's: valve-shut `min φ_LP`, `min φ_HP`; the valve's bleed integral over the stretch both runs
reach; the ceiling (valve fully open).

## 3. PREDICTIONS (frozen here; scored in the spec)

**Construction (gated, no credit):** the reduce (inactive ⇒ prior code bit for bit, steady and
transient, every dispatch path counted); the steady cascade's `Tt3` fixed point converges and its
answer satisfies both mixers' energy balances and both shaft balances to round-off; transient
equilibrium ≡ steady match to 1e−9 (rung 62's plant gate) for vane alone, blade alone, both, each
with the customer bleed off / share / amount, valve shut and open; `τ_LPT`, `π_LPT` unchanged by
cooling on the CPG gas to round-off; the mass identities (nozzle flow = burner gas + ε); lower
solvers refuse active cooling; siblings keep it; non-CPG gases refused.

**The bars are derived.** The model drops only the fuel-ratio response `df` to `Tt3` (rung 90's
residual: ≤ 0.0095 at `|h|, |ℓ| = 1`). Here `|h|` reaches 2, so that residual scales to ≤ ~0.02; the
bars keep rung 90's 0.03 (HP) and 0.05 (LP). A crossing's bar is the magnitude bar over the
model's local slope in `Tt4`.

**Steady — flat maps:**
- **A1 (the HP turbine moves).** `d ln π_HPT/dε` within 0.03 of § 1's `p` at every `Tt4`, both
  streams (rung 31's choked pin held it at exactly 0).
- **A2 (HP, magnitude).** `d ln φ_H/dε` within 0.03 of § 1's table at every `Tt4`, both streams.
- **A3 (LP, magnitude).** `d ln φ_L/dε` within 0.05 of § 1's table at every `Tt4`, both streams.
- **A4 (HP, the arrow reverses at high power).** Each stream's HP response changes sign ONCE in the
  band, NEGATIVE above (rung 90's was positive everywhere): vane at **1120 ± 40 K**, blade at
  **798 ± 25 K** (slopes 7.3e−4 / 1.35e−3 per K).
- **A5 (LP, no relief).** `d ln φ_L/dε < 0` at every `Tt4`, both streams (rung 90's crossed at
  910 K).
- **A6 (the ordering).** Blade below vane on BOTH spools at every `Tt4`.

**Steady — shaped (`flow/press`, sign only):**
- **A7.** At 1500 K both spools negative for both streams; at 650 K LP negative and HP positive for
  both streams.

**Transient — the valve (rig B; from the steady model over the ramp, `Tt4` 1000–1400 K, where § 1
gives LP −0.60 to −0.95 (vane), −1.18 to −1.54 (blade), against rung 90's −0.09 to −0.38):**
- **B1.** Valve shut, each stream LOWERS `min φ_LP` in all 6 cells.
- **B2.** Valve shut, blade cooling LOWERS `min φ_HP` in all 6 cells (its HP response is negative
  over the whole ramp). Vane cooling's HP is NOT scored: § 1 puts its crossing inside the ramp.
- **B3.** The valve's bleed integral RISES with each stream at every set point where it rides in
  both runs.
- **B4.** The ceiling FALLS with each stream in all 6 cells.
- **B5 (ordering).** Blade costs the valve MORE than vane, and vane MORE than the customer bleed of
  the same share, at every set point where all three ride.

**The likeliest miss, named in advance:** A2/A4 for the vane stream rest on the HPT-pin slope `p`
and `ρ` — the model linearises the HP turbine about one point, and `ρ ≈ 1.07` amplifies any error in
`p`. If `p` lands within its bar but the vane HP misses, the second-order HP-work term is the cause.
A1 separates the two.
