# Rung-6 external anchors — dissociation, chemical equilibrium, the `Kp` solve

Verified constants + worked checks that anchor **high-temperature dissociation** (rung 6):
the absolute-entropy constant `a7` (mirroring rung-5's `a6`), the equilibrium constant
`Kp(T)` with its **standard-state pressure factor**, the equilibrium composition solver,
and the two numbers the rung turns on — the **equilibrium adiabatic flame temperature
drop** and the **negligible station-4 cycle delta**. Every number was re-derived and
**certified before any production code** (project discipline), in `M:\claud_projects\temp\rung6\`.

> **Read `docs/rung5-fork-b.md` first.** Rung 6 is built *on* Fork B: `Kp` needs
> `ΔG° = ΔH° − TΔS°` on absolute enthalpy (`a6`, installed in rung 5) **and** absolute
> entropy (`a7`, added here). The one-convention rule (rung-5 seam) is load-bearing:
> the equilibrium solve consumes production's `0K-sensible + formation` enthalpy scale.

The mechanism (advisor's framing, confirmed by the numbers): at the cycle's station 4,
dissociation is suppressed **twice** — **lean** combustion (excess O₂ pushes CO₂⇌CO+½O₂
back by Le Chatelier) **and high combustor pressure** (mole-increasing reactions shift
back under compression). So the cycle sits exactly where dissociation is negligible; the
dramatic ~116 K drop is a **near-stoichiometric, ~1-atm** phenomenon that shows up only
in the **adiabatic-flame-temperature diagnostic**. That contrast *is* the lesson.

---

## 1. Species set + sourced thermochemistry

Rung 5 carried `{N₂, O₂, Ar, CO₂, H₂O}`. Rung 6 adds the five C/H/O dissociation
products **CO, H₂, OH, O, H** (the radicals OH/O/H included on purpose — CO+H₂-only
undershoots the flame-temperature band). Eight C/H/O-bearing species over three elements
(C, H, O) ⇒ **five independent reactions** (below). N₂/Ar stay inert. **NO/N (Zeldovich)
deferred** — kinetically limited in reality, its own pollutant-formation topic, and it
drags N into the equilibrium for ~10-20 K more sink; explicit seam.

NASA-7 `a1…a5` (the `cp/R` shape) transcribed from **GRI-Mech 3.0** `thermo30.dat` (the
same source rung 3's O₂/H₂O/CO₂/N₂/Ar match verbatim). Sourced standard properties at
298.15 K (CODATA/JANAF):

| species | ΔH̄f° (J/mol) | S° (J/mol·K) | note |
|---|---|---|---|
| CO  | −110 527 | 197.660 | CODATA |
| H₂  | 0 | 130.680 | element |
| OH  | +38 987 | 183.708 | JANAF (ΔHf° has ~1.5 kJ/mol literature spread; see §6) |
| O   | +249 180 | 161.058 | CODATA |
| H   | +217 998 | 114.716 | CODATA |
| (CO₂, H₂O, N₂, O₂, Ar carried from rung 5) | | | |

### Deriving `a7` (mirrors rung-5's `a6`), and the joint cross-check
Full NASA-7 entropy is `S/Ru = a1 lnT + a2 T + a3 T²/2 + a4 T³/3 + a5 T⁴/4 + a7`. Rung 5's
`_antideriv_phi` is exactly that polynomial part **with no constant** (it drops out of
every `pr` ratio), so `Ru·a7` is precisely the additive absolute-entropy term — the exact
twin of how `Ru·a6` is the additive formation term. Rather than transcribe two more NASA
columns (fresh transcription risk), **derive** both constants from the sourced ΔH̄f°/S°:

```
a6 = ΔH̄f°/Ru − antideriv_h(A_low, 298.15)          (rung 5)
a7 = S°/Ru   − antideriv_s(A_low, 298.15)           (rung 6)
```

**The single load-bearing cross-check.** GRI-Mech 3.0 *tabulates* its own `a6`/`a7`; we
never used them (we derive), so comparing derived-from-(ΔHf°,S°) against GRI-Mech's
tabulated constants certifies the **sourced thermochemistry and the transcribed
polynomial jointly** — a slip in either shows up as a mismatch:

| species | a6 dev vs GRI | a7 dev vs GRI |
|---|---|---|
| O₂, CO₂, H₂O, CO, H₂, O, H, N₂, Ar | ≤ 0.02 % | ≤ 0.3 % |
| OH | 1.2 % | 3.5 %* |

*OH's gap is **literature spread in OH's ΔHf°/S°**, not a transcription error (GRI-Mech's
OH assumes ΔHf° ≈ 39.35 kJ/mol vs our 38.99; the a7 "%" is large only because OH's
`a7 ≈ −0.10` is near zero — the absolute Δs is 0.03 J/mol·K, negligible). OH is a minor
species; the equilibrium anchor (§4) is the binding certification. The
formation/entropy self-check `H̄(298.15)=ΔH̄f°`, `S̄(298.15)=S°` is exact by construction
for all ten species; elements land at `h=0` at 298.15 K.

---

## 2. The equilibrium constant — and the standard-state factor (the trap)

For a reaction `Σ νᵢ Xᵢ = 0` (products positive), the equilibrium condition `Σ νᵢ μᵢ = 0`
with `μᵢ = ḡᵢ°(T) + RuT ln(pᵢ/p°)` gives

```
Kp(T) = Πᵢ (xᵢ)^νᵢ · (p/p°)^Δν  =  exp(−ΔG°(T)/RuT),   ΔG°(T) = Σ νᵢ ḡᵢ°(T),  Δν = Σ νᵢ
```

**The `(p/p°)^Δν` factor is the trap.** A dissociation reaction changes mole count
(CO₂⇌CO+½O₂ has Δν = +½), so `Kp` is **not** a bare mole-fraction ratio. With the
standard state **`p° = 1 bar = 100 000 Pa`** (stated, and used consistently), this factor
is exactly what makes high combustor pressure suppress dissociation. Get `p°`, `Δν`, or
the partial-pressure convention wrong and the composition looks plausible but is wrong.

### The five basis reactions and their `log₁₀ Kp(T)`
(computed from the derived `ḡ°(T) = h̄ − T s̄`, so JANAF-consistent by the §1 cross-check):

| T (K) | CO₂→CO+½O₂ | H₂O→H₂+½O₂ | H₂O→OH+½H₂ | ½O₂→O | ½H₂→H |
|---|---|---|---|---|---|
| 1800 | −3.687 | −4.269 | −4.618 | −3.915 | −3.446 |
| 2000 | −2.879 | −3.540 | −3.782 | −3.175 | −2.789 |
| 2200 | −2.221 | −2.943 | −3.098 | −2.569 | −2.249 |
| 2500 | −1.434 | −2.226 | −2.278 | −1.840 | −1.599 |
| 3000 | −0.480 | −1.349 | −1.277 | −0.947 | −0.801 |

(CO₂⇌CO+½O₂ at 2500 K ⇒ `Kp = 0.0368`, the classic textbook value.)

---

## 2b. The enthalpy-datum convention (a load-bearing numbers-first discovery)

Rung 5 left a seam note: "rung 6's `Kp` solve must consume production's `0K-sensible +
formation` scale, or migrate both paths — must not mix." Working the numbers **corrected
that steer**, and the correction is itself an anchor result. Two enthalpy scales are in
play, differing by a per-species `h_sensible(298.15)` constant:

- **Scale A — `a6`-at-298.15** (formation, "elements = 0 at 298.15 K"): the textbook
  combustion scale, `h_i(T) = ΔH̄f_i + [h_sens_i(T) − h_sens_i(298.15)]`.
- **Scale B — `0K-sensible + formation`**: what production Fork B (rungs 4/5) uses,
  `h_i(T) = h_sens_i(T) + ΔH̄f_i`.

**`Kp` requires scale A — and this is not a datum *choice*, it's correctness.** For `Kp`
to be physical, `Σ νᵢ h̄ᵢ(T)` must equal the true `ΔH°_rxn(T)`. On scale B it comes out
`ΔH°_rxn(T) + Σ νᵢ h_sens,i(298.15)`, and that extra term is **nonzero exactly when
Δν ≠ 0** — i.e. for every dissociation reaction. Measured: scale B mis-weights
CO₂⇌CO+½O₂ by `exp(−Σν·h_sens(298.15)/RuT) ≈ 0.8×` at 2000 K (~20 %). The methane anchor
(§4) is the empirical proof scale A is right: scale-A equilibrium AFT = 2231.7 K vs CEA
2226 K; scale B lands ~19 K low.

**But `Kp` is a datum-free physical constant** (the log-Kp table above) — computing it
correctly *needs* formation `h̄` + absolute `s̄`, but that is chemistry, not the model's
energy datum. So the model keeps **exactly one energy datum: scale B**, and the design is
not "mixing":

| computation | scale | why |
|---|---|---|
| `Kp` / equilibrium **composition** | A (internal to the solve) | correct reaction ΔG°; **output is scale-free mole numbers** |
| cycle-**burner energy balance** | B (single-scale, end to end) | reduces to production Fork B **exactly** when dissociation off |
| **AFT diagnostic** (test-only) | A | matches the CEA physical anchor |

The only object crossing from the `Kp` solve into the energy balance is the composition
(mole numbers) — genuinely datum-free — so no `cp·298 ≈ 0.3 MJ/kg` seam can form inside a
balance. **This continues rung 5's own split** (`test_forkb.py` already computes AFT on
scale A while production runs on scale B); rung 6 only adds the correct `Kp` constants.

The decisive **anti-seam gate** (turns the argument into a measurement): in the cold-`Tt4`
limit the rung-6 cycle `f` must equal the rung-5 Fork-B `f` to **solver tolerance (~1e-6)**
— a constant ~1 % offset there would betray scale-A enthalpy leaking into the balance.

---

## 3. The equilibrium composition solver

Unknowns: the eight reacting mole numbers `nᵢ` (per mol air). Equations: **3 element
balances** (C, H, O) + **5 reaction relations**. Solved by **damped Newton in `ln(nᵢ)`
space** (keeps `nᵢ > 0`, avoids the classic "went-negative" blow-up), seeded from the
complete-combustion composition. The reaction relation in log form:

```
Σ νᵢ [ ln(nᵢ) − ln(n_tot) ] + Δν·ln(p/p°) − ln Kp = 0        (n_tot includes inert N₂/Ar)
```

Robustness: capped Newton step, analytic Jacobian; converges in a handful of steps at
every (T, p) reached by the flame-temperature and burner solves below.

---

## 4. The independent physics anchor — methane-air stoichiometric AFT

The binding certification of the whole chain (`a6+a7`, `Kp`, the `(p/p°)^Δν` factor, the
solver): reproduce a **published equilibrium flame temperature**. Our C/H/O species set
covers methane by atom count (CH₄ + 2 O₂ → …; C=1, H=4 per mol fuel), so we can hit the
well-known **CEA/Turns methane-air value directly**. Constant-pressure, 1 atm, reactants
at 298.15 K:

| quantity | ours | reference |
|---|---|---|
| CH₄-air stoich equilibrium AFT | **2231.7 K** | ~2226 K (Turns *Intro to Combustion*, Φ=1; CEA/Cantera-GRI30) |
| no-dissociation AFT (same case) | 2327.5 K | — (drop = 96 K) |

**+0.26 % (≈ 6 K) high** — precisely what deferring NO/N (a few K sink) plus the OH
spread predict. Product mole fractions (excl. inert N₂/Ar) are textbook for stoich
methane-air at ~2230 K: `CO₂ 0.295, H₂O 0.632, CO 0.030, H₂ 0.012, OH 0.011,
O 0.0008, H 0.0014, O₂ 0.018` (residual O₂ *at stoich* is the signature of dissociation).

### The `(p/p°)^Δν` factor, demonstrated (stoich, T=2300 K fixed)

| p | CO/(CO+CO₂) | OH mole no. | excess O₂ |
|---|---|---|---|
| 1 atm | 0.1143 | 3.9e−3 | 0.0086 |
| 5 atm | 0.0686 | 2.4e−3 | 0.0053 |
| 13 atm | 0.0503 | 1.7e−3 | 0.0039 |

Dissociation falls with pressure — the plumbing is live and correctly signed.

---

## 5. The two numbers rung 6 turns on

### (a) Equilibrium AFT for our `(CH₂)ₙ` fuel — the diagnostic drops into band
Constant-pressure, 1 atm, reactants 298.15 K; no-dissociation is rung 5's value:

| f | no-dissociation (rung 5) | equilibrium (rung 6) | drop (K) |
|---|---|---|---|
| 0.0200 | 1059.3 | 1059.3 | 0.0 |
| 0.0300 | 1379.1 | 1379.0 | 0.1 |
| 0.0500 | 1942.2 | 1934.6 | 7.6 |
| 0.0676 (≈stoich) | 2374.9 | **2259.3** | **115.7** |

Rung 5's deliberately-high 2375 K falls to **2259 K** — into the real kerosene-air
~2250 K band. Lean points barely move (dissociation is a hot, near-stoich effect). **This
is the payoff the rung-5 AFT gate teed up.**

### (b) Station-4 cycle delta — measured, and negligible
The fuel/air ratio `f` to reach `Tt4`, **production-faithful design** (scale-A composition,
scale-B energy balance §2b): rung-5 Fork B (complete combustion) vs rung-6 equilibrium,
`Tt3≈850 K, η_b=0.98`:

| Tt4 (K) | p | f Fork B | f equilibrium | Δf |
|---|---|---|---|---|
| 1800 | 13 atm (station-4) | 0.030482 | 0.030528 | **+0.149 %** |
| 1800 | 1 atm | 0.030482 | 0.030579 | +0.318 % |
| 2200 | 1 atm | 0.044411 | 0.045927 | +3.413 % |

At the **actual station-4 condition** (Tt4 metallurgically capped at 1800 K, pt4 ≈ 13 atm,
lean f ≈ 0.0305) the cycle needs **+0.149 % more fuel** — the same order as rung 4→5's
"barely moves." Equilibrium needs *slightly more* fuel because dissociated products retain
chemical enthalpy not released as sensible heat. The 1-atm and 2200-K rows *measure* both
suppression mechanisms (pressure, temperature) so the negligibility is shown, not asserted.
The base (Fork-B) column is reproduced **exactly** in the cold-`Tt4` limit (§2b anti-seam
gate); the +0.149 % is a bounded delta on top of an exactly-reducing base.

---

## 6. Tolerances (from the measured gaps, not guessed)
- **`a6`/`a7` self-check** `H̄(298.15)=ΔH̄f°`, `S̄(298.15)=S°`: exact by construction (~1e-9 rel).
- **`a6`/`a7` vs GRI-Mech tabulated:** ≤ 0.02 % (a6) / ≤ 0.3 % (a7) for the nine major
  species; OH the known exception (§1).
- **Methane-air AFT vs CEA/Turns:** 2231.7 K vs ~2226 K → assert in a physical band
  `2210 < T < 2245 K` (not a to-the-digit book match: NO/N deferred + OH spread).
- **Equilibrium AFT drop:** stoich `(CH₂)ₙ` AFT `2250 < T < 2275 K` and strictly *below*
  the rung-5 no-dissociation value; monotone in f.
- **Station-4 delta:** `|Δf| < 0.5 %` at the design condition (measured 0.149 %). Assert
  the *bound*, not the digit — production `Tt3` comes from the compressor and `η_b=0.98`,
  so the exact number drifts.
- **Anti-seam (cold-`Tt4` limit):** rung-6 cycle `f` == rung-5 Fork-B `f` to **~1e-6**
  (proves no scale-A enthalpy leaked into the scale-B energy balance, §2b).
- **Reduce-to-lower-rung (tight):** `Gas.reacting_equilibrium()` is a *separate* factory;
  every rung 1–5 suite stays green untouched, including `test_forkb.py`'s deliberately-high
  no-dissociation AFT gate.

## What stays UN-anchored (state it plainly)
- **NO/N (Zeldovich) deferred** — no thermal-NOx; would sink the flame temp a few more K
  and is its own pollutant-chemistry topic. Explicit rung-7+ seam.
- **Equilibrium at the burner only; frozen composition downstream** (turbine/nozzle keep
  the station-4 mixture). Justified: station-4 dissociation is negligible (§5b), so the
  "equilibrium vs frozen nozzle expansion" difference is negligible here — but that
  contrast is genuinely rich and is its own rung; seam kept.
- **OH ΔHf°** carries ~1.5 kJ/mol literature spread; we take JANAF 38.99 kJ/mol.
- **AFT has no local book digit** for `(CH₂)ₙ`; the methane-air point is the transferable
  anchor (same solver, same species, atom-count-exact).
