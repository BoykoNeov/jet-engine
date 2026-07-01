# Rung-7 external anchors — thermal NOx, the extended Zeldovich mechanism, kinetics

Verified constants + worked checks that anchor **thermal NO formation** (rung 7): the two
new species (`NO`, `N`) and their thermochemistry, the Hanson–Salimian rate constants, the
**thermo-kinetic `K`-check** that ties the transcribed rates to the already-verified
`a6`/`a7` substrate, the **superimposed equilibrium-NO** layer, and the **extended-Zeldovich
kinetic integrator** with its residence-time knob. Every number was re-derived
**before any production code** (project discipline), in `M:\claud_projects\temp\rung7\`.

> **One honest scope limit up front (§6).** The *thermochemistry* and *rate constants* are
> hard-certified — the `a6`/`a7` self-checks are exact, and the **`K`-check** (§3) binds the
> rate constants to the thermo *relatively*. But the **absolute kinetic magnitude** (the NO
> *rate* itself) has **no local textbook digit** to match, unlike rungs 2–6. It rests on
> order-of-magnitude literature agreement + the `K`-check's relative consistency, and is fenced
> from below by a two-sided magnitude gate (§5a) — not a 0.02 %-to-the-book anchor. Stated
> plainly, not papered over.

> **Read `docs/rung6-spec.md` first.** Rung 7 is built *on* rung 6: it reads the frozen
> station-4 equilibrium pool (`O, O₂, OH, H, N₂`) and computes `Kp` from the same
> `_g_molar` (`a6`+`a7`) machinery — only adding `NO`'s thermochemistry. It does **not**
> touch the C/H/O `_equil_solve`; NO is a **superimposed trace layer**, which is what makes
> reduce-to-rung-6 automatic.

**The lesson inverts rung 6.** Rung 6: the major species *do* reach equilibrium (the cycle
barely moves; the drama is the adiabatic-flame-temperature drop). Rung 7: **NO does *not*
reach equilibrium** — at realistic residence times it is kinetically frozen at a few percent
of its equilibrium value — and it is **exponentially temperature-sensitive** (`~exp(−38370/T)`).
That is *why* thermal NOx, not just blade metallurgy, governs how hard you can run the flame:
raising the peak temperature explodes NO even though the mixed-out, capped, lean turbine
inlet itself makes almost none.

---

## 1. Species set + sourced thermochemistry

Rung 6 carried `{N₂, O₂, Ar, CO₂, H₂O, CO, H₂, OH, O, H}`. Rung 7 adds **`NO`** and **`N`**.
NASA-7 `a1…a5` (the `cp/R` shape) transcribed from **GRI-Mech 3.0** `thermo30.dat` (the same
source as every prior species). Sourced standard properties at 298.15 K (CODATA/JANAF, as
tabulated in Turns *An Introduction to Combustion*, App. A):

| species | ΔH̄f° (J/mol) | S° (J/mol·K) | note |
|---|---|---|---|
| NO | +90 291 | 210.758 | JANAF; ΔHf° has a real literature spread (§ self-check) |
| N  | +472 680 | 153.30  | CODATA |

### The `a6`/`a7` self-check (derived, mirroring rungs 5–6)
`a6` from ΔH̄f°, `a7` from S° — **derived, not transcribed**, exactly as rung 6 did, so no
fresh column risk. The formation/entropy self-check `H̄(298.15)=ΔH̄f°`, `S̄(298.15)=S°` is
**exact by construction** (~1e-16) for both species. Cross-checking the *derived* `a6`/`a7`
against GRI-Mech's own *tabulated* constants certifies the sourced thermochemistry and the
transcribed polynomial **jointly**:

| species | a6 dev vs GRI | a7 dev vs GRI |
|---|---|---|
| N  | 0.001 % | −0.004 % |
| NO | **−1.19 %** | 0.059 % |

`N` is dead-on. `NO`'s **`a7` (entropy) is 0.06 %** — the polynomial and `S°` are confirmed —
but its **`a6` is −1.19 %**: GRI-Mech's tabulated `a6` implies ΔH̄f°(NO) ≈ 91.27 kJ/mol,
versus our JANAF **90.291**. This is a genuine **literature spread in ΔH̄f°(NO)**, not a
transcription error — the exact twin of rung 6's OH case (1.2 %). **The `K`-check (§3) proves
JANAF is the right pick**: raising ΔH̄f° to GRI's 91.27 would *worsen* the `K`-check by ~11 %.

---

## 2. The extended Zeldovich mechanism + the rate constants

Thermal NO forms by the **extended Zeldovich mechanism** (three reactions; the third is the
"extended" one):

```
1:  O + N₂  ⇌  NO + N        (rate-limiting: E_a ≈ 319 kJ/mol ⇒ k1f ~ exp(−38370/T))
2:  N + O₂  ⇌  NO + O
3:  N + OH  ⇌  NO + H
```

Rate constants — **Hanson & Salimian (1984)**, as tabulated in Turns (units cm³, mol, s;
`T` in K; converted to SI m³ by ×1e−6 in code):

| k | forward | reverse |
|---|---|---|
| 1 | `1.8e14 · exp(−38370/T)` | `3.8e13 · exp(−425/T)` |
| 2 | `1.8e10 · T · exp(−4680/T)` | `3.8e9 · T · exp(−20820/T)` |
| 3 | `7.1e13 · exp(−450/T)` | `1.7e14 · exp(−24560/T)` |

Every Zeldovich reaction is **mole-conserving (`Δν = 0`)** — no `(p/p°)` factor anywhere, so
`Kc = Kp`. That is what makes the `K`-check clean and it is also the physical headline of §5.

---

## 3. The load-bearing gate — the thermo-kinetic `K`-check

Reactions 1 + 2 sum to the overall `N₂ + O₂ ⇌ 2 NO`. Detailed balance then demands

```
k1f·k2f / (k1r·k2r)  =  Kc(N₂+O₂⇌2NO)  =  exp(−ΔG°/RuT),   ΔG° = 2ḡ°(NO) − ḡ°(N₂) − ḡ°(O₂)
```

The right side is computed from the **existing `_g_molar`** (rung-6 `a6`+`a7`) once `NO`'s
thermochemistry is added — `N` cancels in the product, so this needs only `NO`. Agreement
certifies the transcribed **rate constants** *and* `NO`'s **formation thermochemistry**
together — a slip in either forward/reverse pair, or in ΔH̄f°(NO)/S°(NO), breaks it:

| T (K) | `k1f·k2f/(k1r·k2r)` | `exp(−ΔG°/RuT)` | ratio |
|---|---|---|---|
| 1800 | 1.2302e−04 | 1.1886e−04 | 1.035 |
| 2000 | 4.1313e−04 | 3.9819e−04 | 1.038 |
| 2200 | 1.1131e−03 | 1.0703e−03 | 1.040 |
| 2500 | 3.6565e−03 | 3.5028e−03 | 1.044 |

**Agreement to ~3.5–4.4 %** across the whole flame band (reaction 2 alone: ~2.4 %). A gross
transcription error would be **orders of magnitude** off; the residual few-percent is the
known small inconsistency between empirical rate fits and JANAF thermo. **Gate tolerance:
ratio in [0.95, 1.10].** This is rung 7's rung-6-style joint certification.

---

## 4. The superimposed equilibrium-NO layer (NO is trace; keep it OUT of `_equil_solve`)

NO is trace (ppm), so it neither perturbs the C/H/O equilibrium nor the cycle energy balance.
It is a **superimposed partial equilibrium** on the frozen rung-6 pool:

```
½N₂ + ½O₂ ⇌ NO   (Δν = 0)   ⇒   x_NO,e = Kp_NO · √(x_N₂ · x_O₂),   Kp_NO = exp(−ΔG°/RuT)
```

using the frozen mixture's own `x_N₂`, `x_O₂`. **`N` is not added to `_equil_solve`** — this is
load-bearing: leaving the C/H/O solve untouched makes reduce-to-rung-6 automatic. And `N` is
never needed anywhere in the kinetics either (§5, reverse-rate form). Validation that ignoring
`N` in the layer is safe:

| T (K) | x_N,e | x_NO,e | ratio N/NO |
|---|---|---|---|
| 2300 | 3.3e−08 | 3083 ppm | 1.1e−05 |
| 2500 | 2.5e−07 | 6158 ppm | 4.0e−05 |

Equilibrium NO (stoich `(CH₂)ₙ`, 1 atm), for the magnitude anchor (known band ~few-thousand
ppm for stoichiometric flames):

| T (K) | 2000 | 2200 | 2300 | 2400 |
|---|---|---|---|---|
| x_NO,e (ppm) | 795 | 2052 | 3083 | 4444 |

---

## 5. The kinetic integrator + the two numbers rung 7 turns on

**One-equation extended-Zeldovich model** (Heywood/Turns), QSS on `N`, **reverse-rate form**
for R2/R3 so equilibrium `[N]` is never needed (uses the rung-6 pool's `O`, `H` directly):

```
d[NO]/dt = 2 R1 (1 − α²) / (1 + α·R1/(R2+R3)),   α = [NO]/[NO]_e
R1 = k1f[O][N₂]    R2 = k2r[NO]_e[O]    R3 = k3r[NO]_e[H]     (mol/m³·s)
```

α=0 → rate = `2 R1` (initial rate); α→1 → rate → 0 (**saturates at [NO]_e**). Integrated by
RK4 over a **combustor residence time `τ`** (a new *design input*, stated like "specified exit
pressure" was — no combustor geometry is modelled), the pool frozen at the burner `(T, p)`.

### (a) Kinetic NO is frozen far below equilibrium — and τ→∞ recovers equilibrium
Stoich `(CH₂)ₙ`, T = 2300 K, p = 1 atm; equilibrium NO = 3083 ppm:

| τ (ms) | 0.5 | 1 | **3** | 10 | 100 | 1000 | →∞ |
|---|---|---|---|---|---|---|---|
| kinetic NO (ppm) | 17.3 | 34.5 | **102.8** | 336 | 2281 | 3083 | 3083 |
| kin / eq | 0.006 | 0.011 | **0.033** | 0.109 | 0.740 | 1.000 | **1.000** |

At a realistic **τ ≈ 3 ms**, kinetic NO is **3.3 % of equilibrium**. The **τ→∞ asymptote is
an internal consistency gate**: the integrator recovers the independently-computed equilibrium
NO to solver tolerance (measured 1.0000) — but note it clamps to the *thermodynamic* `[NO]_e`,
so it does **not** constrain the kinetic *rate*. **The absolute kinetic magnitude is pinned
only by a two-sided band** (the `K`-check tests ratios; the asymptote cancels the rate; the
freezing gates only fail if the rate is too *fast*): in the first **1 ms** (≪ `τ_NO`, so growth
is ~linear) the kinetics deposit **34.5 ppm** at 2300 K stoich → **gate `[10, 100] ppm`**, which
catches a too-*slow* error (units slip, dropped factor) that would otherwise pass silently. This
is order-of-magnitude literature, **not** a book digit (§6, § What stays un-anchored).

The **characteristic time** `τ_NO = [NO]_e/(2R1)` is the classic proof of freezing — it is far
longer than any combustor residence:

| T (K) | 2100 | 2300 | 2500 |
|---|---|---|---|
| τ_NO | **906 ms** | 89 ms | 13 ms |

### (b) The exponential temperature sensitivity — the payoff
Initial NO rate, stoich, 1 atm, relative to 2000 K (driven by `k1f ~ exp(−38370/T)` *and* the
equilibrium `[O]` pool, itself steeply T-dependent):

| T (K) | 1800 | 2000 | 2200 | 2400 |
|---|---|---|---|---|
| x_O,e (ppm) | 3.3 | 31.0 | 197.6 | 913.0 |
| initial rate (rel.) | 0.016 | 1.0 | **30.0** | **494** |

**~30× per 200 K** — a ~500× swing from 2000→2400 K. This is *the* number: NOx is set by the
**peak flame temperature**, exponentially.

### (c) Equilibrium NO is pressure-independent (`Δν = 0`) — inverting rung 6
Stoich `(CH₂)ₙ`, T = 2300 K:

| p (atm) | 1 | 5 | 13 |
|---|---|---|---|
| x_NO,e (ppm) | 3083 | 2419 | 2089 |

The NO reaction carries **no explicit `(p/p°)` factor** — the residual drift is *only* through
the O₂ pool, which is itself set by rung-6's pressure-suppressed dissociation. So high
combustor pressure, which *stomped* rung-6 dissociation flat, does **not** directly save you
from NOx. A sharp contrast worth drawing.

### (d) The station-4 cycle number — the mixed-out inlet makes almost none
The actual supersonic anchor (Tt4 = 1800 K, pt4 ≈ 13.1 atm, lean far = 0.0328), NO computed on
the frozen station-4 pool:

| quantity | value |
|---|---|
| equilibrium NO | 3063 ppm |
| kinetic NO, τ = 3 ms | **0.43 ppm** (0.01 % of equilibrium), EI_NO = 0.014 g/kg |

At the metallurgically **capped, lean, mixed-out** turbine inlet, thermal NO is negligible —
too cool (the 1800-K initial rate is ~1/60 of the 2000-K one) *and* kinetically frozen. Real
engine NOx is a **hot primary-zone** phenomenon; this mixed-out, single-`Tt4` cycle model does
not resolve combustor zoning (deferred seam, § below). That is the honest cycle-level result,
and it is fully consistent with the arc: **the cycle barely moves; the drama is in the
diagnostic.**

---

## 6. Tolerances (from the measured gaps, not guessed)
- **`a6`/`a7` self-check** `H̄(298.15)=ΔH̄f°`, `S̄(298.15)=S°`: exact by construction (~1e-9 rel).
- **`a6`/`a7` vs GRI-Mech tabulated:** `N` ≤ 0.004 %; `NO` `a7` 0.06 %, `a6` −1.19 %
  (ΔHf°(NO) literature spread, the known exception — like rung-6 OH).
- **`K`-check:** `k1f·k2f/(k1r·k2r)` vs `exp(−ΔG°/RuT)` → **ratio in [0.95, 1.10]** across
  1800–2500 K (measured 1.035–1.044).
- **τ→∞ asymptote:** kinetic NO → equilibrium NO to **~1e-3** relative (measured 1.0000).
- **Equilibrium-NO magnitude:** stoich `(CH₂)ₙ` at 2300 K in **2500 < NO < 3500 ppm** (band,
  not a book digit — depends on the lean-stoich O₂ pool).
- **Kinetic freezing:** at τ = 3 ms, 2300 K, kinetic NO **< 10 %** of equilibrium (measured 3.3 %);
  τ_NO(2100 K) **> 100 ms** ≫ residence.
- **Absolute kinetic magnitude (two-sided, the too-slow guard):** NO at τ = 1 ms, 2300 K stoich
  in **`[10, 100] ppm`** (measured 34.5). This is the *only* gate that fails if the rate is too
  *slow* — order-of-magnitude literature, **not** a to-the-digit book anchor.
- **T-sensitivity:** initial rate at 2200 K **> 20×** the 2000-K rate (measured 30×).
- **Reduce-to-rung-6 (tight):** NO/N are a *superimposed* layer and are never added to
  `_equil_solve`, so every rung 1–6 suite stays green **untouched**; the cycle (far, thrust) is
  bit-for-bit rung 6.

## What stays UN-anchored (state it plainly)
- **Equilibrium-`[O]` assumption.** The radical pool is the rung-6 *equilibrium* `O`. Real
  flame fronts carry **super-equilibrium `O`** that drives NO faster; and **prompt NO**
  (`CH + N₂ → …`, Fenimore) is a separate, non-thermal path. Using equilibrium `O` is Turns'
  standard first model — the explicit next seam.
- **Combustor zoning / primary-zone equivalence ratio.** The diagnostic reads the **mixed-out
  station-4** pool (or a stated flame temperature), not a resolved rich-primary → dilution
  combustor. That is *why* the station-4 number is so low; realistic engine EI_NO needs zoning.
- **`τ` as a standalone knob** (the first time-dimension input) — stated, not derived from
  geometry; default ~3 ms, swept.
- **ΔHf°(NO)** carries a real ~1 kJ/mol literature spread; we take JANAF 90.291 (the `K`-check
  confirms it over GRI's 91.27).
- **The absolute kinetic *rate* has no local textbook digit** (unlike every rung 2–6 anchor).
  What *is* certified: the rate constants + NO thermochemistry, *relatively*, via the `K`-check
  (§3), and the `a6`/`a7` self-checks (exact). What is **not** book-pinned: the absolute
  d[NO]/dt magnitude — it rests on order-of-magnitude literature agreement (initial rate
  ~34.5 ppm/ms at 2300 K stoich; `τ_NO` ~tens–hundreds of ms) plus the §5a two-sided gate. A
  future rung could pin it to a specific Turns/Heywood worked example *read from the text*
  (not a recalled digit); until then this is stated, not claimed as anchored.
