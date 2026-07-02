# Rung 14 — Equilibrium-vs-frozen nozzle flow: the rung-6 cycle-side seam (and where rung-10's dropped clamp earns its keep)

Rungs 7–13 were a **NOx diagnostic ladder** riding on the *frozen* rung-6 pool; the cycle never
moved. Rung 14 turns back to the **cycle itself** and re-opens a seam rung 6 named and deferred
(`docs/plans/rung6-anchor-equilibrium.md` § "What stays un-anchored": *"Equilibrium at the burner
only; frozen composition downstream … the 'equilibrium vs frozen nozzle expansion' difference is
negligible here — but that contrast is genuinely rich and is its own rung; seam kept."*). This is
that rung.

**The physics.** The production nozzle **freezes** the station-4 equilibrium mixture through the
whole expansion (rungs 6–13, `_EquilibriumSection`). A real nozzle flow lies between two idealised
limits:

- **FROZEN** — chemistry infinitely *slow*: composition held at the station-4 mixture (what
  production does). A **lower** bound on nozzle performance.
- **EQUILIBRIUM / SHIFTING** — chemistry infinitely *fast*: composition = equilibrium(T, p) at
  **every** point of the expansion. As the exhaust cools, the dissociated radicals CO/H₂/OH/O/H
  **recombine** to CO₂/H₂O, releasing chemical energy that partly converts to kinetic energy →
  a **higher** exit velocity. An **upper** bound.

The real nozzle (finite Damköhler number) sits between them. Rung 14 computes **both bounds** as a
pure diagnostic beside the cycle — the production nozzle stays frozen, so **the cycle is bit-for-bit
rung 6.**

**Two complementary lessons — the honest arc mirrors the rung-10 dropped clamp.**

1. **THRUST (major species).** The frozen↔equilibrium gap is **negligible at the cool lean design
   point** (Tt4=1500 K, φ≈0.4: dissociation ≈ 5×10⁻⁶, ΔV9 ≈ 0.006%) and **earns its keep hot** (a
   Tt4 sweep: ~1% of the carbon dissociates by 2200 K and recombination buys ≈0.46% more exhaust
   velocity). *Dormant here, real there* — exactly like the clamp.
2. **NO / THE CLAMP.** On the **same cooling path** equilibrium NO **collapses** (Kp_NO falls
   steeply with T), so any realistic frozen exhaust NO is **super-equilibrium** at the exit and rung
   7's **dropped** clamp `cNO≤cNOe` finally **fires** (max_a ≫ 1). Rung 10 dropped that clamp *on
   principle* and proved it **dormant** on the combustor quench (max_a = 0.677 < 1), flagging that it
   *would* bite "in the near-stoich exhaust cooling in the still-open nozzle seam." This rung is that
   seam: the clamp earns its keep here.

> **Read `docs/rung6-spec.md` and `docs/plans/rung6-anchor-equilibrium.md` first** (the dissociation
> equilibrium, `a6`/`a7`/`Kp`, the two enthalpy scales), and `docs/rung10-spec.md` § "the clamp
> trap." This file states only what *changes*. The `_equilibrium_composition` solver, the scale-B
> absolute enthalpy `_h_molar_B`, the absolute entropy `_s_molar` (`a7`), the `_equilibrium_no_fraction`
> / `_kp_no` NO thermochemistry, and the "derive before you code" / conservation-assert contract all
> carry over **unchanged**. No new chemistry is added — only reversible-adiabatic bookkeeping over the
> existing equilibrium machinery.

---

## What rung 14 adds (and what it deliberately does not)

**Adds** (all in `turbojet/gas.py`, all *decoupled* from the cycle):

- `_mix_entropy_molar(comp, T, p)` — the **absolute mixture entropy per mol air**,
  `S = Σ nᵢ[s0ᵢ(T) − Ru·ln(xᵢ·p/p0)]`, on the rung-6 absolute-entropy constant `a7` (`_s_molar`) plus
  the standard partial-pressure/mixing term. This is the ONE new thermodynamic quantity.
- `_mix_mass_per_air(comp)` — mixture mass per mol air, **invariant** under recombination (atoms
  conserved), so the frozen and equilibrium exits share the `V9 = √(2ΔH/m)` denominator.
- `_expand_nozzle(comp_entry, far, Tt9, pt9, p9, shifting)` — **one** reversible-adiabatic expansion
  from the entry total state to the back-pressure, used **both** ways. Frozen (`shifting=False`) holds
  the composition; equilibrium (`shifting=True`) re-equilibrates at each `T`. It bisects the exit `T9`
  on `S_mix(comp(T9), T9, p9) = S_mix(comp_entry, Tt9, pt9)`, then `V9 = √(2(H_entry −
  H_abs(comp9,T9))/m)` on **absolute** (scale-B) enthalpy so the recombination (formation) energy
  appears.
- `_nozzle_clamp_diag(comp_entry, Tt9, T9, x_no_frozen)` — the clamp corollary: the equilibrium-NO
  **collapse ratio** `x_NO_e(Tt9)/x_NO_e(T9)` (frozen-NO-independent) and, given a frozen exhaust NO,
  `max_a = x_NO_frozen/x_NO_e(T9)` (> 1 ⇒ the clamp fires).
- `NozzleFlowState` dataclass + `Gas.nozzle_flow(far, Tt4, pt4, Tt9, pt9, p9, x_no_frozen=None)` — the
  public diagnostic. It **only reads** the state handed in and touches **no** cycle path.

**Does NOT add / deliberately out of scope:**

- **No change to the production nozzle.** The cycle's headline thrust is still the **frozen** number;
  the equilibrium value is a *bracket bound*, reported beside it.
- **No finite-rate nozzle chemistry** (the real Damköhler-number flow between the bounds). Rung 14
  gives the *bracket*; a transported/finite-rate nozzle is a further seam.
- **No shifting turbine.** The bracket isolates the **nozzle**: both expansions start from the same
  frozen station-4 gas at the nozzle-entry total state (Tt9=Tt5, pt9=π_n·pt5). A full equilibrium
  turbine would reopen the shaft balance and break bit-for-bit rung 6 — deferred.

---

## The load-bearing modeling call: a common *physical* entry (not a re-equilibrated one)

Both expansions start from the **same physical gas** — the frozen station-4 mixture `comp_entry` at
the nozzle-entry total state `(Tt9, pt9)` — so the stagnation enthalpy `H_entry` **and** entropy
`S_entry` are **identical**. The recombination benefit then shows up honestly as a higher **V9**, not
as a shifted entry temperature.

The tempting alternative — re-equilibrate the entry to `eq(Tt9,pt9)` for the shifting case — is
**wrong for a fair bracket**: it *lowers* `H_entry` (equilibrium is more recombined) so the two
expansions no longer share a stagnation enthalpy, and the recombination benefit hides in a different
entry state instead of appearing as `V9_equil > V9_frozen`. Re-equilibrating buys strict isentropy at
the entry at the cost of comparing two *different* gases. The physical-gas entry costs only a
negligible sliver of entry irreversibility (the honest **"infinite-rate switched on at the throat"**
model) and gives a clean bracket. The frozen-in station-4 dissociation is the whole story and it
already lives in the physical entry gas.

## The entropy — why frozen reduces to the production nozzle *exactly*

For a **fixed** composition the mixing part `Σ nᵢ(−Ru·ln xᵢ)` of `_mix_entropy_molar` is invariant
between entry and exit (the mole fractions don't change), so the isentropic constraint collapses to
`Σ nᵢ[s0ᵢ(T9) − s0ᵢ(Tt9)] = n_tot·Ru·ln(p9/pt9)` — the pr-ratio relation the production nozzle already
solves. So the **frozen** branch of `_expand_nozzle` reproduces the production `V9`/`T9` to machine
precision (the load-bearing reduce). For a **shifting** mixture the mole fractions *do* change, so the
mixing term is live and the drop is genuinely a different (equilibrium) path.

---

## Verification gates (priority order)

1. **reduce (LOAD-BEARING, exact)** — the **frozen** expansion reproduces the production nozzle
   `V9`/`T9` to machine precision (`< 1e-6`; measured ≈ 5×10⁻¹² m/s); and **freezing the composition
   inside the equilibrium branch is bit-for-bit the frozen result** (the ONLY difference between the
   brackets is the composition shift). The whole rung-1..13 suite stays green untouched.
2. **reduce (dissociation → 0)** — at a cool combustor (Tt4=1300 K) the entry `CO/(CO+CO2) < 1e-5` and
   the bracket collapses (`ΔV9/V9 < 1e-4`).
3. **direction** — a shifting expansion is **faster** (`V9_equil ≥ V9_frozen`) and **hotter at exit**
   (`T9_equil > T9_frozen` — recombination reheats), and the exit is **more recombined** than the
   frozen entry (`CO_exit < CO_entry`).
4. **magnitude / monotone** — `ΔV9/V9` grows with combustor temperature: dormant at the design point
   (< 1e-4, ≈ 0.006%), earns its keep hot (> 3e-3, ≈ 0.46% at 2200 K).
5. **isentropic self-check** — both expansions conserve mixture entropy (`S_exit = S_entry`, tight).
6. **the CLAMP earns its keep** — the equilibrium-NO collapse ratio ≫ 1, and fed the realistic rung-8
   zoned (ICAO-band) exhaust NO the exit `max_a ≫ 1` (and decisively past rung 10's dormant 0.677).
7. **cycle untouched** — a `nozzle_flow` call must not perturb station 4 or the cycle `V9` (pure
   diagnostic; NO/dissociation never enter the cycle solve).
8. **guards** — requires the equilibrium (rung-6) gas; rejects a back-pressure above the total
   pressure; the exit-`T` bisection guards against pinning at the cold `_equilibrium_composition`
   convergence floor.

## Conservation asserts (rung-14 deltas)

- `_expand_nozzle` post-loop **bracket guard**: the exit `T9` is not pinned at `_T_EXIT_FLOOR`
  (500 K — the equilibrium Newton diverges below it; every real exit sits > 700 K), the twin of
  `_primary_aft`'s bracket-edge guard.
- `nozzle_flow` **requires the equilibrium gas** and **`p9 ≤ pt9`** (the same back-pressure guard the
  production nozzle carries).
- The equilibrium composition solve carries its own C/H/O atom-balance + Newton-convergence asserts
  (rung 6), now exercised at **nozzle-exit** temperatures the cycle never reaches.

## Done when

`Gas.nozzle_flow` returns the `[V9_frozen, V9_equilibrium]` thrust bracket (dormant at the design
point, ≈0.46% hot) and the clamp corollary (max_a ≫ 1 with the zoned exhaust NO); `main.py` prints
the rung-14 panel; `tests/test_rung14.py` is green; the whole prior suite is untouched.

## The rung-15+ seam (keep it additive)

- **The PDF through the finite quench** (the *former* rung-14 seam, now **rung 15**): rung 13 resolved
  the mixture-fraction β-PDF on the *ideal* bell; carrying it through the rung-12 `_quench_no`
  trajectory unites the composition-variance *location* with the dwell *climb* and turns rung 13's
  ≈0 optimum floor into a finite bulk NO.
- **Finite-rate nozzle chemistry** — the real flow *between* the frozen and equilibrium bounds (a
  nozzle Damköhler number / a transported-composition expansion).
- **A shifting turbine** — extend the equilibrium expansion upstream of the nozzle (reopens the shaft
  balance).
- **Super-equilibrium O / prompt NO** in the exhaust — the clamp corollary is still an equilibrium-O
  lower bound on the frozen NO.
