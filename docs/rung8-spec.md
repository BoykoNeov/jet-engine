# Rung 8 — Combustor zoning: the primary-zone NOx effect

Rung 7 installed thermal NO as a trace kinetic diagnostic, then evaluated it on the **mixed-out,
metallurgically-capped, lean** station-4 pool (Tt4 = 1500 K, φ ≈ 0.41) and got EI_NO ≈ 8e-6 g/kg
— essentially zero — ending on *"Real NOx is a hot primary-zone effect."* Rung 8 makes that
concrete without touching the cycle: it runs the **same** extended-Zeldovich integrator on a
**two-zone** (near-stoichiometric **primary** → **dilution**) combustor. NO is set in the hot
primary and **frozen** through the dilution that cools the gas back to Tt4. EI_NO climbs to
**~13–18 g/kg**, into the measured ICAO band. It is the rung-6 *AFT-into-the-real-band* moment,
one level up: the capped mixed-out temperature never made the NO — a hot zone you averaged away
did.

> **Read `docs/rung7-spec.md` first**, and `docs/plans/rung8-anchor-zoning.md` (numbers-before-
> code: sourced primary-zone φ, the ICAO EI_NO band, the machine-checked two-zone worked
> example, the reduce-to-rung-7 gate). This file states only what *changes*; the Zeldovich
> integrator, the `a6`/`a7`/`Kp`/`_equil_solve` substrate, and the "derive before you code" /
> conservation-assert contract carry over.

---

## What rung 8 adds (and what it deliberately does not)

**Adds:**

- **A two-zone combustor diagnostic** — `Gas.zoned_nox(far, Tt3, Tt4, p, phi_primary, τ)` — that
  reuses the rung-7 `_thermal_no` on a **hot primary-zone** pool instead of the mixed-out one.
  Nothing in the thermochemistry or kinetics changes; only the `(T, p, far)` handed to the
  integrator does.
- **An adiabatic primary-flame-temperature solve from Tt3** — the primary burns fuel with only
  its share of the air, **preheated to the actual compressor-exit Tt3**, on the existing scale-A
  `_h_molar_A` + `_equilibrium_composition`. (Promotes the test-only `_aft` helper into a small
  reusable primary-AFT function — starting from Tt3, not 298 K.)
- **A dilution / mix-out step** — add the remaining air at Tt3, **re-equilibrate the major
  species** (release the stored dissociation energy → `T_mix` returns to ≈ Tt4), **freeze the NO
  moles**. Yields the diluted NO mole fraction and the split-independent `T_mix`.
- **`main.py` panel + `NOTES.md` section + `tests/test_rung8.py`.**

**Deliberately does NOT:**

- **Touch the cycle.** NO is still trace and decoupled; `zoned_nox` is a pure diagnostic. Every
  cycle station is **bit-for-bit rung 6** (and the whole rung 1–7 suite stays green, untouched).
- **Model a rich primary.** Held **φ_primary ≤ 1** (lean-complete-combustion stoichiometry only).
  A rich-burn → quick-quench → lean-burn (RQL) combustor is the next seam (§ deferred).
- **Add super-equilibrium O / prompt NO** (rung-7's deferred seam) — the primary pool is rung-6
  *equilibrium* O.
- **Add finite mixing** — mix-out is single-step and instantaneous; NO is frozen at the primary
  value (no secondary-zone Zeldovich).

---

## The load-bearing result: it completes rung 7's inversion

Rung 7: NO is frozen far below equilibrium and exponentially T-sensitive — but computed at the
**capped mixed-out** Tt4 it is negligible. Rung 8: resolve the **near-stoichiometric primary**
(AFT ≈ 2300–2450 K) and the same integrator gives **~13–18 g/kg** (φ_p = 0.9–1.0), into the
measured band — a **~6-order-of-magnitude** lift purely from *where* the chemistry is evaluated.
The T-sensitivity shows through directly: φ_p 0.7 → 1.0 (AFT +383 K) swings EI_NO **36×**. This
is *why* real combustors fight peak temperature (lean-premixed, staging, RQL) — the mixed-out
turbine inlet you designed to the blade limit is not where the NOx was made.

---

## The datum / substrate reuse (no new convention)

No new energy datum, no new species, no new reactions. The primary-AFT solve uses the **same
scale-A `_h_molar_A`** the rung-6 AFT diagnostic used; the Zeldovich layer is the **same**
rung-7 `_thermal_no`. The only datum-free objects crossing between steps are **mole numbers**
(primary equilibrium composition, frozen NO moles) — exactly as composition crossed scales in
rungs 6–7. NO never enters the scale-B cycle balance, so rung 5's one-datum invariant is
untouched.

---

## The equations — a two-zone diagnostic layer, no station changes

Every cycle station is **bit-for-bit rung 6**. Given overall `far`, combustor inlet `Tt3`,
turbine-inlet cap `Tt4`, pressure `p`, primary equivalence ratio `φ_p ≤ 1`, residence `τ`:

```
air split:     far_p = φ_p · far_stoich        (primary fuel/air; all fuel in primary)
               α = far / far_p                  (fraction of air in the primary, ≤ 1)
primary AFT:   solve T_p :  Σ nᵢ(far_p,T_p)·h̄ᵢ_A(T_p) = h̄_air_A(Tt3) + n_fuel·h̄f_fuel
               (equilibrium products; air PREHEATED to Tt3, scale A)
primary NO:    NOxState = _thermal_no(comp(far_p,T_p,p), T_p, p, τ, far_p)   ← rung-7, unchanged
mix-out:       solve T_mix : Σ nᵢ(far,T_mix)·h̄ᵢ_A(T_mix)
                            = α·Σ nᵢ(far_p,T_p)·h̄ᵢ_A(T_p) + (1−α)·h̄_air_A(Tt3)
               (re-equilibrate majors; enthalpy conserved)
freeze NO:     n_NO,total = α · x_NO(τ)·n_tot(primary);   x_NO,mix = n_NO,total / n_tot(mix)
outputs:       T_p, EI_NO (set in primary), x_NO(τ), x_NO,mix, T_mix
```

- **Standing asserts (rung-8 deltas):** (1) `φ_p ≤ 1` and `α ≤ 1` (lean-stoich scope guard);
  (2) `T_mix ≈ Tt4` within the η_b gap (the central conservation gate — majors must
  re-equilibrate); (3) carry over rung 7's `K`-check (now at the hotter primary T), the
  `0 ≤ x_NO ≤ x_NO,e` clamp, and the trace guard.

---

## Verification gates (priority order)

1. **Reduce-to-rung-7 (load-bearing).** At `α → 1` (φ_p = φ_overall, all air in the primary) the
   two-zone diagnostic collapses to rung 7's single mixed-out pool — in **two** parts (the naive
   "primary AFT → Tt4, exact to ~1e-6" is *wrong*: two effects push the primary AFT a few K above
   Tt4). **Exact:** `zoned_nox` EI_NO equals `thermal_nox(far, T_p, p)` — the rung-7 integrator at
   the *same* primary AFT `T_p` — to machine precision (certifies far_p, α, the mole-freeze
   scaling). **Physical:** `T_p` lands just above Tt4 (a ~8 K scale-A/scale-B **datum** offset that
   *survives* η_b = 1, plus a ~9 K η_b piece), and the zoned EI is within an **O(1) factor**
   (~1.3–1.7) of the mixed-out `thermal_nox(far, Tt4, p)` — so it is reduce-to-rung-7, not
   reduce-to-itself. The whole rung 1–7 suite stays green, untouched; the cycle is bit-for-bit
   rung 6. (See docs/plans/rung8-anchor-zoning § 3 for the two-effect decomposition.)
2. **EI_NO lands in the ICAO band.** Primary φ_p = 0.9–1.0 gives EI_NO in **single-digit-to-tens
   g/kg** (order-of-magnitude landing zone, not a book digit — the absolute Zeldovich rate is
   un-pinned, per rung 7). The mixed-out value is ~6 orders lower.
3. **Mix-out temperature is split-independent and returns to Tt4.** `T_mix` is (near-)identical
   across all φ_p in a sweep and equals Tt4 within the η_b gap — the conservation gate. A
   frozen-majors mix-out (energy trapped) would miss Tt4 and *fail* this gate (the discriminating
   check that the re-equilibration is real).
4. **NO-mole conservation through dilution.** EI_NO (per kg fuel) is unchanged by dilution
   (moles conserved) even as the NO mole fraction falls; a clean concentration-vs-emission-index
   separation.
5. **T-sensitivity of the primary.** EI_NO rises monotonically and strongly with φ_p (AFT):
   ≥ 10× over φ_p 0.7 → 1.0 (measured 36×).
6. **`K`-check + trace guard hold at the primary T** (~2300–2450 K, inside the rung-7 band).

## Conservation asserts (rung-8 deltas)
Carry over rung 6/7's, plus: the **φ_p ≤ 1 / α ≤ 1** scope guard; the **`T_mix ≈ Tt4`** mix-out
closure (re-equilibration gate); NO-mole conservation across dilution.

## Done when
Reduce-to-rung-7 holds (exact in the same-`T_p` framing; physical to an O(1) factor vs the
mixed-out Tt4 number — the primary AFT sits ~8 K + ~9 K above Tt4, datum + η_b; rungs 1–7 suites
untouched, green; cycle bit-for-bit rung 6); EI_NO
lands in the ICAO band at φ_p ≈ 1; the split-independent-`T_mix`-→-Tt4 gate, NO-mole
conservation, T-sensitivity, and `K`-check/trace gates hold. `main.py` gains a rung-8 zoning
panel (φ_p sweep: primary AFT, EI_NO into the band vs the mixed-out ~zero, T_mix returning to
Tt4, the dilution NO-fraction drop); `NOTES.md` gains a rung-8 section (the completed inversion,
why zoning is *the* NOx lever); `CLAUDE.md` scope + deferred-seams updated (combustor zoning
done; rich primary / RQL, super-equilibrium O / prompt NO, equilibrium-vs-frozen nozzle = the
next seams).

## The rung-9+ seam (keep it additive)
Rung 8 splits the combustor into a lean-stoich primary + dilution and freezes NO through mix-out.
Next seams, all still additive on this substrate: (a) **rich primary / RQL** — rich CO/H₂
stoichiometry so the primary can run φ > 1 and the model becomes a real low-NOx combustor;
(b) **super-equilibrium O / prompt NO** — a richer radical pool + the Fenimore path in the
primary; (c) **finite-rate mix-out** — a secondary-zone Zeldovich in the cooling gas instead of
a frozen NO; (d) the still-open **equilibrium-vs-frozen nozzle expansion** (rung-6 seam). Only
*where* and *on what pool* the chemistry runs changes.
