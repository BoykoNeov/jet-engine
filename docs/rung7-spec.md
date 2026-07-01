# Rung 7 — Thermal NOx: the Zeldovich mechanism & kinetically-limited NO

Rung 6 computed the **major-species** composition from chemical equilibrium and froze it
through the turbine and nozzle. Rung 7 adds the first **pollutant** and the first **kinetics**:
**thermal nitric oxide**, formed by the extended Zeldovich mechanism. Its defining feature is
that, unlike the major species, **NO does *not* reach equilibrium** in a combustor — it is
rate-limited, kinetically frozen far below its equilibrium value at realistic residence times.
NO is a **trace** species (ppm), so it is a **decoupled diagnostic layer** on top of the
rung-6 cycle: the cycle stays **bit-for-bit rung 6**.

> **Read `docs/rung6-spec.md` first**, and `docs/plans/rung7-anchor-nox.md` (numbers-before-
> code: sourced `NO`/`N` thermochemistry, the Hanson–Salimian rate constants, the `K`-check
> table, the equilibrium-NO and kinetic-vs-`τ` tables, the T-sensitivity, the deferred seams).
> This file states only what *changes*; the property machinery, the equilibrium solve, the
> `a6`/`a7` substrate, and the "derive before you code" / conservation-assert contract carry over.

---

## What rung 7 adds (and what it deliberately does not)

**Adds:**

- **Two species — `NO`, `N`** — beside the rung-6 ten. NASA-7 `a1…a5` from GRI-Mech 3.0;
  `a6`/`a7` **derived** from sourced ΔH̄f°/S° (JANAF), like rungs 5–6. Adding them to the data
  dicts is **inert** to rungs 1–6 (nothing references them; `_equil_solve` uses its fixed
  8-species basis).
- **The extended Zeldovich mechanism** (3 reactions) with **Hanson & Salimian (1984)** rate
  constants, and the **thermo-kinetic `K`-check** binding them to the existing `_g_molar`
  (`a6`+`a7`) substrate — `k1f·k2f/(k1r·k2r) = exp(−ΔG°_{N₂+O₂⇌2NO}/RuT)` (rung-7's rung-6-style
  joint certification). Every Zeldovich reaction is `Δν=0` ⇒ `Kc=Kp`, no pressure factor.
- **A superimposed equilibrium-NO layer** — `½N₂+½O₂⇌NO` (`Δν=0`), `x_NO,e = Kp_NO·√(x_N₂ x_O₂)`
  read off the **frozen rung-6 pool**. NO is **not** added to `_equil_solve` (load-bearing:
  keeps reduce-to-rung-6 automatic); `N` is never needed (reverse-rate kinetics).
- **A kinetic NO integrator** — the one-equation extended-Zeldovich model (QSS on `N`,
  reverse-rate `R2/R3` on the pool's `O`,`H`), RK4-integrated over a **residence-time knob `τ`**
  (the first time-dimension input). Returns kinetic `x_NO(τ)`, equilibrium `x_NO,e`, the initial
  rate, the characteristic time `τ_NO`, and the emission index `EI_NO`.
- **A station-4 NOx diagnostic** surfaced **parallel to the rung-6 AFT** (a test-only helper +
  a `main.py` panel) — it does **not** feed the cycle.

**Deliberately deferred (seams kept):**

- **Super-equilibrium `O` / prompt NO.** The pool is rung-6 *equilibrium* `O`; real flame fronts
  have super-equilibrium `O` (faster NO) and a separate prompt-NO (`CH+N₂`) path. Explicit next seam.
- **Combustor zoning.** The diagnostic reads the mixed-out station-4 (or a stated flame `T`), not
  a rich-primary → dilution combustor — which is *why* the station-4 NO is so low. Realistic
  engine EI_NO needs zoning; kept as a seam.
- **NO feedback on the cycle.** NO is trace (< 0.5 %); its energy/mole effect on `f`/`Tt`/thrust
  is neglected, so the cycle is exactly rung 6. (Stated, and checked: reduce-to-rung-6.)
- Off-design / component maps, the choked convergent nozzle, the afterburner — still out.

---

## The load-bearing result: the lesson inverts rung 6

**Rung 6:** the major species *do* reach equilibrium; the cycle barely moves; the drama is the
adiabatic-flame-temperature drop. **Rung 7:** NO *does not* reach equilibrium — at τ ≈ 3 ms it
sits at **~3 % of equilibrium** (2300 K), because the characteristic NO time (`τ_NO ≈ 90 ms` at
2300 K, **906 ms** at 2100 K) is far longer than the residence time. And the initial rate is
**exponentially** temperature-sensitive (`~exp(−38370/T)`): **~30× per 200 K**, a ~500× swing
from 2000→2400 K. So:

- The **cycle is bit-for-bit rung 6** — NO is trace, decoupled.
- At the **capped, lean, mixed-out** station 4 (Tt4 = 1800 K) thermal NO is **negligible**
  (~0.4 ppm at τ = 3 ms) — too cool *and* frozen.
- The **payoff** is the T-sensitivity: it is the **peak flame temperature**, not the mixed-out
  Tt4, that governs NOx — which is *why* thermal NOx (not just blade metal) caps how hard the
  flame runs, and why real combustors fight temperature (lean-premixed, staging).

And a clean secondary inversion: rung-6 dissociation was **pressure-suppressed** (`(p/p°)^Δν`);
equilibrium NO is **pressure-independent** (`Δν=0`) — high combustor pressure does not directly
save you from NOx.

---

## The datum / substrate reuse (no new convention)
`Kp_NO` and the `K`-check use the **same scale-A `_g_molar`** (`a6`+`a7`) the rung-6 `Kp` solve
uses — NO just needs its own `a6`/`a7`. There is **no new energy datum**: NO never enters the
scale-B cycle energy balance (it is trace and decoupled), so rung 5's one-datum invariant is
untouched. The only rung-6 objects the NOx layer reads are the **frozen mole numbers**
(`O, O₂, OH, H, N₂`) — datum-free — exactly as the composition crossed scales in rung 6.

---

## The equations — a diagnostic layer, no station changes

Every cycle station is **bit-for-bit rung 6**. The NOx layer, evaluated on the frozen pool
`comp` at `(T, p)` (the burner, or a stated flame `T`):

```
equilibrium:   x_NO,e = Kp_NO(T) · √(x_N₂ · x_O₂),      Kp_NO = exp(−(ḡ_NO − ½ḡ_N₂ − ½ḡ_O₂)/RuT)
kinetics:      d[NO]/dt = 2 R1 (1 − α²)/(1 + α·R1/(R2+R3)),   α = [NO]/[NO]_e
               R1 = k1f[O][N₂],  R2 = k2r[NO]_e[O],  R3 = k3r[NO]_e[H]   (SI: mol/m³·s)
               [X] = x_X · p/(RuT);   integrate 0→τ by RK4 from [NO]=0
outputs:       x_NO(τ),  x_NO,e,  initial rate 2R1,  τ_NO = [NO]_e/(2R1),  EI_NO (g NO/kg fuel)
```

- **Standing asserts:** (1) `K`-check ratio in `[0.95, 1.10]` (a module-level self-check on the
  rate constants vs thermo); (2) `0 ≤ x_NO(τ) ≤ x_NO,e` at all times (the integrator never
  overshoots equilibrium); (3) NO is trace (`x_NO,e < 0.02`) — the justification for decoupling.

---

## Verification gates (priority order)
1. **Reduce-to-lower-rung (load-bearing).** NO/N are a *superimposed* layer, never added to
   `_equil_solve`; every rung 1–6 suite stays green **untouched**, and the cycle (far, thrust,
   every station) is **bit-for-bit rung 6**.
2. **The `K`-check (the rung-6-style joint certification).** `k1f·k2f/(k1r·k2r)` vs
   `exp(−ΔG°/RuT)` in `[0.95, 1.10]` across 1800–2500 K — certifies the rate constants *and*
   NO's thermochemistry together.
3. **The τ→∞ asymptote.** Kinetic NO → the independently-computed equilibrium NO to ~1e-3
   (integrator/equilibrium mutual consistency).
4. **Formation + entropy self-checks.** `H̄(298.15)=ΔH̄f°`, `S̄(298.15)=S°` for NO/N (~1e-9);
   derived `a6`/`a7` vs GRI (N ≤ 0.004 %; NO `a7` 0.06 %, `a6` the known ΔHf° spread).
5. **Magnitude + kinetic freezing.** Equilibrium NO at 2300 K stoich in `2500 < NO < 3500 ppm`;
   at τ = 3 ms kinetic NO < 10 % of equilibrium; `τ_NO(2100 K) > 100 ms` ≫ residence; and a
   **two-sided absolute-magnitude gate** — NO at τ = 1 ms, 2300 K stoich in `[10, 100] ppm` —
   which is the *only* gate that catches a too-*slow* kinetic error (the others test ratios or
   clamp to thermodynamic `[NO]_e`). Order-of-magnitude literature, **not** a book digit: the
   absolute kinetic rate has no local textbook anchor (stated, see rung7-anchor § What stays
   un-anchored).
6. **T-sensitivity.** Initial rate at 2200 K > 20× the 2000-K rate (measured 30×), monotone.
7. **Pressure independence.** Equilibrium NO carries no `(p/p°)` factor (`Kp_NO` p-independent).

## Conservation asserts (rung-7 deltas)
Carry over rung 6's, plus: the **`K`-check** self-consistency; **`0 ≤ x_NO ≤ x_NO,e`** on the
integrator; the **trace** guard `x_NO,e < 0.02` (decoupling justification).

## Done when
Reduce-to-lower-rung reproduces rungs 1–6 to the digit (existing suites untouched, green) and
the cycle is bit-for-bit rung 6; the `K`-check, τ→∞ asymptote, self-checks, magnitude/freezing,
T-sensitivity, and pressure-independence gates hold. `main.py` gains a rung-7 NOx panel (the
T-sweep: equilibrium NO, kinetic NO@τ, kin/eq %, τ_NO vs residence, the T-sensitivity, the
station-4 mixed-out number, the pressure-independence contrast); `NOTES.md` gains a rung-7
section (the lesson inversion, why the cycle doesn't move, why peak flame T governs NOx);
`CLAUDE.md` scope + deferred-seams updated (thermal NOx / Zeldovich kinetics done; super-
equilibrium O / prompt NO, combustor zoning, and equilibrium-vs-frozen nozzle flow = the next
seams).

## The rung-8+ seam (keep it additive)
Rung 7 installs thermal NO as a trace kinetic diagnostic on the frozen equilibrium pool. Next
seams, all riding this substrate: (a) **super-equilibrium O / prompt NO** — a richer radical
pool and the Fenimore path; (b) **combustor zoning** — a rich-primary → dilution model so
station-4 EI_NO becomes engine-realistic; (c) **equilibrium vs frozen nozzle expansion** (the
rung-6 seam, still open). Only *where* and *on what pool* the chemistry runs changes.
