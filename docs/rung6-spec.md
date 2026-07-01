# Rung 6 — High-Temperature Dissociation & the Chemical-Equilibrium Solve

Rung 5 put enthalpies on an absolute (formation) scale but still burned to **fixed,
complete** products (`CH₂ + 1.5 O₂ → CO₂ + H₂O`). Rung 6 lets the products **dissociate**
at high temperature — `CO₂⇌CO+½O₂`, `H₂O⇌OH+½H₂`, `½O₂⇌O`, … — with the composition set
by **chemical equilibrium** (`Kp(T) = exp(−ΔG°/RuT)`) rather than assumed. This needs the
absolute-**entropy** constant `a7` (`S/R = a1 lnT + … + a7`) on top of Fork B's absolute
enthalpy `a6`. The burner's complete-combustion `_products_composition(f)` is replaced by a
`T`,`p`-coupled equilibrium solve seeded from it.

> **Read `docs/rung5-fork-b.md` first**, and `docs/plans/rung6-anchor-equilibrium.md`
> (numbers-before-code: sourced thermochemistry, the `Kp` table, the CEA methane anchor,
> the datum-convention discovery, the AFT drop and station-4 delta). This file states only
> what *changes*; the property machinery, dual gas, efficiencies, and the "derive before
> you code" / conservation-assert contract all carry over.

---

## What rung 6 adds (and what it deliberately does not)

**Adds:**

- **Five dissociation species** — `CO, H₂, OH, O, H` beside `{CO₂, H₂O, N₂, O₂, Ar}` (the
  radicals OH/O/H included on purpose: `CO`+`H₂`-only undershoots the flame-temperature
  band). NASA-7 `a1…a5` from GRI-Mech 3.0 (rung 3's source).
- **The absolute-entropy constant `a7`, derived (not transcribed)**, mirroring rung-5's
  `a6`: rung 3's `_antideriv_phi` is exactly the `S/R` polynomial part with *no constant*,
  so `Ru·a7` is the additive absolute-entropy term. `a7 = S°/Ru − antideriv_phi(A_low,
  298.15)` from each species' tabulated `S°(298.15)`. Cross-checked against GRI-Mech's own
  tabulated `a7` (≤ 0.3 % for the major species; the formation/entropy self-checks are
  exact by construction).
- **The equilibrium constant with its standard-state factor.** For `Σ νᵢ Xᵢ = 0`:
  ```
  Kp(T) = Πᵢ (xᵢ)^νᵢ · (p/p°)^Δν = exp(−ΔG°(T)/RuT),   p° = 1 bar,   Δν = Σ νᵢ
  ```
  The **`(p/p°)^Δν` factor is load-bearing** — a dissociation reaction changes mole count,
  so high combustor pressure suppresses dissociation. `ΔG°(T) = Σ νᵢ ḡᵢ°(T)`,
  `ḡ° = h̄° − T s̄°` on absolute `h̄` (`a6`) and `s̄` (`a7`).
- **The equilibrium composition solver** — 3 element balances (C, H, O) + 5 reaction `Kp`
  relations for the 8 reacting mole numbers, by **damped Newton in `ln(nᵢ)` space** (keeps
  `nᵢ>0`), seeded from complete combustion. N₂/Ar inert.
- **The equilibrium burner** (the one station that changes): a **root-find on `f`** over the
  absolute-enthalpy balance, with the equilibrium composition solved at each trial `f`. See
  § Station 4.
- **A `Gas.reacting_equilibrium(...)` factory** beside `Gas.reacting()` (Fork A) and
  `Gas.reacting_forkb()` (Fork B), so every frozen / Fork-A / Fork-B path is untouched.

**Deliberately deferred (seams kept):**

- **Thermal NOx (`½N₂+½O₂⇌NO`, `½N₂⇌N`)** — kinetically limited in reality (not equilibrium),
  its own pollutant-formation topic, ~10-20 K more flame-temp sink. Explicit seam.
- **Equilibrium *vs frozen* flow downstream.** Rung 6 solves equilibrium **at the burner
  only** and **freezes** that composition through turbine + nozzle. Justified: station-4
  dissociation is negligible (below), so re-equilibrating the expansion would move nothing
  here — but the frozen-vs-equilibrium nozzle contrast is genuinely rich and is its own
  rung. Seam kept.
- Off-design / component maps, the choked convergent nozzle, the afterburner — still out.

---

## The load-bearing result: the cycle barely moves, the flame-temp diagnostic drops

At the cycle's station 4, dissociation is suppressed **twice**: **lean** combustion (excess
O₂ pushes `CO₂⇌CO+½O₂` back) **and high combustor pressure** (mole-increasing reactions
shift back under compression). So the cycle sits where dissociation is negligible:

- **Station-4 cycle delta: +0.149 %** on `f` at the design point (Tt4 = 1800 K, pt4 ≈ 13 atm,
  lean) — the same order as rung 4→5's "barely moves." Bounded `< 0.5 %`.
- **The unconstrained adiabatic-flame-temperature diagnostic drops 116 K**: rung-5's
  deliberately-high stoichiometric 2375 K falls to **2259 K**, into the real kerosene-air
  ~2250 K band. *This is the payoff the rung-5 AFT gate teed up.*

Both are **measured** in the anchor doc, not asserted. The lesson is the contrast: the
dramatic dissociation effect is a near-stoichiometric, ~1-atm phenomenon; the engine runs
lean, high-pressure, and metallurgically capped — exactly where it doesn't bite.

---

## The datum convention (§2b of the anchor doc — one energy datum, correct `Kp`)

Two enthalpy scales differ by a per-species `h_sensible(298.15)` constant: **A**
(`a6`-at-298.15, formation, "elements = 0 at 298.15") and **B** (`0K-sensible + formation`,
production Fork B). The rule rung 6 follows:

- **`Kp` / equilibrium composition** uses scale **A** — *required*, not a choice: `Σ νᵢ h̄ᵢ(T)`
  must equal the true `ΔH°_rxn(T)`, and scale B adds `Σ νᵢ h_sens,i(298.15) ≠ 0` whenever
  `Δν ≠ 0` (every dissociation reaction), mis-weighting `Kp` ~20 % at 2000 K. But `Kp` is a
  **datum-free physical constant**; its *output* is scale-free mole numbers.
- **The cycle-burner energy balance** uses scale **B**, single-scale end to end, so it
  **reduces to production Fork B exactly** when dissociation is off.
- **The AFT diagnostic** (test-only) uses scale **A**, matching the CEA methane anchor.

This is **not mixing** — the only object crossing from the `Kp` solve into the energy
balance is the (datum-free) composition. It continues rung 5's own split (`test_forkb.py`
already computes AFT on scale A while production runs on scale B). The **anti-seam gate**
makes it a measurement: in the cold-`Tt4` limit, rung-6 `f` == rung-5 Fork-B `f` to ~1e-6.

---

## Station equations — only the burner changes

Every other station is **bit-for-bit rung 5**: the composition is **frozen** at the
station-4 equilibrium mixture, so the turbine Δh, the nozzle `V9 = √(2(h(Tt9)−h(T9)))`, and
every substate are the same enthalpy/`pr` machinery on a fixed mixture (a `_TPGSection`
built from the frozen composition). **`a7` enters ONLY the `Kp`/`ΔG°` solve — never the
downstream `pr = exp(φ/R)`** (the additive entropy constant cancels in every `pr` ratio,
the exact parallel to "only the burner sees `a6`").

### 4 — Burner: equilibrium composition + root-find on `f`
```
pt4 = π_b · pt3
f   : root-find    h_air,B(Tt3) + f·h_fuel  =  Σᵢ nᵢ(f)·h̄ᵢ,B(Tt4)  +  (1−η_b)·f·LHV
      where  nᵢ(f) = equilibrium composition at (Tt4, pt4, f)   [scale-A Kp solve]
```
- The rung-4/5 algebraic fixed point `f = (h4−h3)/(η_b·hPR−h4)` is *derived from* complete
  combustion; with dissociation `hPR` is not the true release, so it is replaced by a
  **root-find (bisection) on the absolute balance**, composition re-solved at each trial `f`.
  Seed a tight bracket around the (cheap) Fork-B `f` — the equilibrium `f` is a `+0.149 %`
  correction to it. The `(1−η_b)·f·LHV` loss term is exactly Fork B's.
- Converged `f` → **freeze** the station-4 equilibrium mixture (build a `_TPGSection` via
  `_mixture()` so `R_t` tracks the mole-count shift) for the whole downstream cycle; write
  `far = f`, `mdot·(1+f)`.
- **Standing asserts:** (1) atom conservation (C/H/O) on the converged composition; (2) the
  frozen mixture is keyed on `(far, Tt4, pt4)` — reusing a `Gas` with the same `far` but a
  different burn condition trips a loud assert (the "pure function of `far` for a fixed burn
  config" invariant, no hidden state).

---

## Verification gates (priority order)
1. **Reduce-to-lower-rung (load-bearing).** `Gas.reacting_equilibrium()` is a separate
   factory; every rung 1–5 suite stays green **untouched**, including `test_forkb.py`'s
   deliberately-high no-dissociation AFT gate.
2. **Anti-seam / reduce-to-rung-5 (cold-`Tt4` limit).** Rung-6 cycle `f` == rung-5 Fork-B
   `f` to ~1e-6 as `Tt4` drops (dissociation → 0). Proves the datum split has no seam.
3. **The `Kp`/equilibrium physics anchor.** Methane-air stoichiometric equilibrium AFT in
   the band `2210 < T < 2245 K` (measured 2231.7 K; CEA/Turns ~2226 K), with the pressure
   sweep showing dissociation falling as `p` rises (the `(p/p°)^Δν` factor live).
4. **Formation + entropy self-checks.** `H̄(298.15)=ΔH̄f°`, `S̄(298.15)=S°` per species
   (~1e-9); `a7` derivation matches GRI-Mech tabulated (major species ≤ 0.3 %).
5. **Equilibrium-AFT drop.** Stoichiometric `(CH₂)ₙ` AFT in `2250 < T < 2275 K`, strictly
   *below* the rung-5 no-dissociation value, monotone in `f`. (Test-only, scale A.)
6. **Station-4 delta bounded.** `|Δf| < 0.5 %` at the design point; C/H/O atom conservation
   on every burner run.

## Conservation asserts (rung-6 deltas)
Carry over rung 5's, plus: **atom conservation** (C/H/O) on the converged equilibrium
composition; **one-burn-config guard** on the frozen-mixture cache; the **anti-seam**
cold-limit equality as a standing test.

## Done when
Reduce-to-lower-rung reproduces rungs 1–5 to the digit (existing suites untouched, green);
the anti-seam cold-limit `f` matches Fork B to ~1e-6; the methane AFT anchor, the self-checks,
and the equilibrium-AFT drop hold; the station-4 delta is bounded. `main.py` gains a
Fork-B-vs-equilibrium panel (near-identical — +0.02 % on `f` at the 1500 K panel vs +0.15 %
at the 1800 K anchor, the steeply Tt4-dependent delta — the AFT drop, a dissociation-vs-pressure
line); `NOTES.md` gains a rung-6 section (why the cycle barely moves, why the flame-temp
diagnostic finally drops, the one-energy-datum + correct-`Kp` story). `CLAUDE.md` scope +
deferred-seams updated (dissociation/equilibrium + `a7` done; thermal-NOx and
equilibrium-vs-frozen nozzle flow = the next seams).

## The rung-7+ seam (keep it additive)
Rung 6 installs equilibrium at the burner on frozen-downstream composition. The next seams:
(a) **thermal NOx** — add `NO`/`N` and the `½N₂+½O₂⇌NO` / `½N₂⇌N` reactions (equilibrium
*and* a Zeldovich-kinetics variant, since NO is rate-limited); (b) **equilibrium vs frozen
nozzle expansion** — re-solve equilibrium as the gas cools through the turbine/nozzle
(recombination releases heat), the classic performance-relevant contrast. Both ride the
`a6`+`a7`+`Kp` substrate rung 6 built; only *where* the equilibrium solve runs changes.
