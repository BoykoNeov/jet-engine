# Rung 26 — Freeze-out: an ANCHORED chemical time that resolves WHERE recombination quenches

Rung 25 resolved the finite-rate nozzle flow between rung-14's bounds with a **single normalized
Damköhler number `Da`** — a cartoon knob (like `α`, `φ_p`, `τ_q`) that slides the whole expansion
uniformly from frozen to the irreversible-fast ceiling. Its own spec named the seam it left open:

> *"A CONSTANT `Da` interpolates the bracket but CANNOT show FREEZE-OUT — the point where `τ_chem(T)`
> overtakes `τ_flow` as the gas cools and recombination stops. That is the pedagogical heart of nozzle
> non-equilibrium and the honest next seam; a `T`-dependent `τ_chem` would capture it but reintroduces
> an unanchored Arrhenius constant (the exact trap `docs/mixing-scale-negative.md` recorded)."*

**Rung 26 builds it — and the build REFUTES that seam's own framing on BOTH counts.** The chemical time
is **anchored**, to the very mechanism the project already cites; and the freeze is **not Arrhenius** —
it is **density-driven, against an opposing temperature effect.**

**THE HEADLINE IS A MOVING FREEZE POINT, NOT A NEW BOUND.** Rung 26 adds no state above rung 25's
three (F/I/R) — the finite-rate flow still lands strictly inside `[V9_F, V9_I]`. What it adds is
*where*: rung-25's uniform `Da` is replaced by a **local** `Da(T,p) = τ_res/τ_chem(T,p)` computed from
an anchored recombination clock, so the relaxation **shuts off partway down the nozzle** — and the
shut-off point **MOVES with `Tt4`**: lean it never switches on (frozen from entry), hot it crosses
mid-expansion and later still as `Tt4` climbs. **That motion is the physics a constant `Da`
structurally cannot express** — exactly the rung.

> **Read `docs/rung25-spec.md` first** (the `dh=v·dp` spine, `_finite_rate_expand`, the three-state
> F/I/R picture, `_equilibrate_hp`/`_irreversible_fast_expand`) and
> `docs/plans/rung26-anchor-freeze-out.md` (the anchored GRI-Mech rates + the verified feasibility
> gate). This file states only what *changes*. **No new chemistry, no new species, no new bound** —
> only a `T,p`-dependent relaxation *rate* on rung-25's exact integrator. A **pure diagnostic beside
> the cycle**: the production nozzle stays frozen, so the cycle is **bit-for-bit rung 6.**

---

## The anchor — GRI-Mech 3.0, the mechanism already at `gas.py:94`

The recombination clock uses the dominant three-body radical sink, **verbatim from GRI-Mech 3.0**
(the mechanism `gas.py:94` already cites for the five dissociation species' NASA polynomials — so
**zero new unanchored constants**, categorically unlike `mixing-scale-negative.md`'s penetration
exponent `p` which had no anchor at all):

```
  H + OH + M  ⇌  H2O + M        k = A·T^n,   A = 2.200e22 cm⁶/mol²/s,   n = −2.000,   Ea = 0.00
```

The other radical sinks in the pool are the same shape: `2H+M` (A=1.0e18, n=−1), `2O+M` (A=1.2e17,
n=−1), `O+H+M` (A=5.0e17, n=−1) — **every one has `Ea = 0.00` and `n < 0`.** There is **no thermal
barrier to freeze out against.** `H+OH+M` is taken as the representative clock (fastest, largest `A`);
the freeze *existence* and *motion* are insensitive to the pick (all four scale the same way in `T,p`).

## Derive before you code — the governing relations

**1. The recombination time (the clock).** For the termolecular sink, the OH-consumption time is

```
  τ_chem(T,p; comp) = 1 / ( k(T) · [OH] · [M] ),        k(T) = A·T^n  (Ea = 0 ⇒ NO exponential)
  [OH] = x_OH · c_tot,   [M] = c_tot,   c_tot = p/(Ru·T)      (mol/cm³ after the unit convert)
  ⟹  τ_chem ∝ 1 / ( A·T^n · x_OH · c_tot² ),   c_tot² ∝ (p/T)²
```

*Physical justification:* a three-body recombination rate is `k[X][Y][M]`, so its relaxation time
goes as one over `k` and over the **square** of concentration — a **density²** law. `x_OH` is the
**current, self-consistently marched** radical fraction (not a bracket — the integrator carries the
real relaxing composition, so the clock sees the true local radical pool). A single representative
reaction is a **single-timescale surrogate** for the whole recombination network — an
order-of-magnitude clock, disclosed.

**2. The local Damköhler number (the switch).** Distribute a total residence time
`τ_res = L/(0.6·V9_frozen)` uniformly over the geometric pressure schedule `p(s)=pt9·(p9/pt9)^s`
(uniform-in-`s` residence — a declared simplification; `s` is linear in `ln p`). `τ_res` is pinned to
the **frozen/cycle `V9`**, NOT to the freeze-out output — a residence time defined by the velocity we
are solving for would be a fixed-point coupling. The per-step relaxation exponent is then
`dt_k/τ_chem = (τ_res·ds)/τ_chem(T_k,p_k) = Da_local(s_k)·ds`, with

```
  Da_local(s) = τ_res / τ_chem(T(s), p(s); comp(s))
  relax_k = 1 − exp( −Da_local(s_k)·ds )        (exact linear relaxation over the step, as rung 25)
```

*This is rung-25's `relax = 1 − exp(−Da·ds)` with the scalar `Da` promoted to the local
`Da_local(s)`* — the ONLY structural change to the integrator. When `Da_local` is held **constant at
the literal `Da`** it is rung 25 **bit-for-bit** (the reduce, below — note this needs the constant
injected *at the `Da_local` value*, not via the `τ_res/τ_chem` division).

**3. The freeze criterion (the finding).** As the gas expands, `c_tot² ∝ (p/T)²` **craters** (density
collapse) while `k(T)=A·T^n` with `n=−2…−1` **accelerates** on cooling. The density term dominates, so
`τ_chem` **grows**, `Da_local` **falls through 1**, and `relax→0`: the composition **freezes** at
whatever it reached. The crossing `Da_local(s_freeze)=1` is the **freeze point** — an **OUTPUT** of the
anchored chemistry and the geometry, not a knob.

## The feasibility gate — does a TURBOJET nozzle (≈8×, not a rocket's 100–1000×) freeze?

The load-bearing question, because every classical freeze-out anchor (Bray, rocket nozzles) is a
100–1000× expansion. **Run** (`docs/plans/rung26-anchor-freeze-out.md`, project's own
`_equilibrium_composition` + `build_turbojet`), bracketed frozen (radical-rich, `Da` upper bound) vs
equilibrium (radical-poor, lower bound):

| `Tt4` | expansion | `Da_local` frozen bd.: entry → exit | Da=1 crossing |
|---|---|---|---|
| 1500 | 6.3× | 0.31 → 0.043 | **never > 1 — frozen from entry** |
| 1800 | 7.4× | 1.45 → 0.17 | s ≈ 0.2 |
| 2200 | 8.6× | 4.50 → 0.45 | s ≈ 0.7 |

**It freezes, robustly, hot.** And two structural bonuses fall out:

- **Lean it is frozen from entry** (`Da_local < 1` throughout) — an **INDEPENDENT derivation of the
  production frozen nozzle** from anchored rate constants, reproducing rung 14/25's **dormant-lean /
  earns-its-keep-hot** arc *without* a bracket.
- **The freeze point MOVES with `Tt4`** (never → s≈0.2 → s≈0.7): hotter ⇒ freezes later ⇒ more
  recombination ⇒ same sign as rung 25's growing `(I−F)` gap. **This is the rung.**

**⚠ These crossings are the PROBE's, an upper bound — the shipped integrator freezes EARLIER.** The
table pins `x_OH` at the frozen bound (radical-rich ⇒ `Da_local` **upper** bound). The production
integrator marches `x_OH` self-consistently, and as radicals recombine `x_OH` drops ⇒ `τ_chem` rises
⇒ `Da_local` falls ⇒ the relaxation **self-quenches**, so the real freeze point sits *upstream* of the
probe's `s`. The bracket still saves the sign (if even the upper bound freezes hot, the real flow
freezes at least as much — conservative), and the *motion* should survive (a hotter entry carries more
radicals **and** a higher entry `Da_local`, so it still freezes later). But the **magnitude of the
`Tt4` spread is unverified** on the real integrator — and the open risk is that `Tt4=1800` freezes so
early it compresses toward the `Tt4=1500` frozen-from-entry case and squeezes the spread that is the
whole rung. **Build the integrator and run gate 5 FIRST, as a test that can fail;** do not expect
`s_freeze` to match this table.

## The kill test (the spine) — density drives it DESPITE an opposing temperature effect

**A mechanism certification on the STANDALONE clock, not the marched path.** The kill test runs
`_tau_chem_recomb` with `x_OH` **pinned** at the frozen-entry value — the clean isolation of `k(T)`
against `c_tot²` only holds when the radical fraction is held constant (the production integrator
marches `x_OH`, so it does not isolate them). It certifies the *mechanism*; the production flow uses
the self-consistent `x_OH`. On the frozen bound (`x_OH` constant, so `k(T)` and `c_tot²` isolate
cleanly), `Tt4=2200` (`T` 1992→1214 K, `p` 427→50 kPa):

| factor | over the expansion | effect on `τ_chem` |
|---|---|---|
| `k(T)=A·T^n`, n=−2 | ×2.69 (accelerates as `T` falls) | ÷2.69 — **OPPOSES** freezing |
| `c_tot²` | ×27.1 | ×27.1 — **DRIVES** freezing |
| NET | ×10.1 | `Da_local` 4.50 → 0.45 |

- **Kill T** (`k` pinned at entry ⇒ density alone): `Da_local → 0.166` — **STILL FREEZES.**
- **Kill p** (`[M]` pinned at entry ⇒ `T` alone): `Da_local → 12.1` — **NO FREEZE; it RISES.**

**The correct framing: density drives freeze-out DESPITE an opposing temperature effect** — *not*
"density-driven, not Arrhenius" (which merely reads back the `p`-dependence put into `τ_chem`).
Arrhenius intuition predicts `Da` *falls* on cooling; the anchored rate makes it *rise*. Non-circular,
and the **opposite sign**. This is what refutes rung 25's "unanchored-Arrhenius trap" framing: there is
no Arrhenius term (`Ea=0`), and the mechanism runs the other way.

## What rung 26 adds (and what it deliberately does not)

**Adds** (all in `turbojet/gas.py`, all *decoupled* from the cycle, all BESIDE rung 25's methods):

- Module constants `_K_HOHM_A=2.2e22`, `_N_HOHM=-2.0` (GRI-Mech 3.0 verbatim) + the CHEMKIN→SI unit
  convert, and `_tau_chem_recomb(comp, T, p, *, kill_T=None, kill_M=None)` — the anchored clock, with
  the kill-test pins as keyword hooks.
- `FreezeOut(L, nstep, rate_scale)` — the config: the **one geometric knob** `L` (residence length,
  ≈0.5 m), the march resolution `nstep`, and a dimensionless `rate_scale` (default 1.0) that scales
  `τ_chem` to drive the *limit* gates and sweeps (`rate_scale→0` ⇒ `τ_chem→∞` ⇒ `Da_local→0` ⇒ frozen;
  `→∞` ⇒ irreversible-fast). **`rate_scale` does NOT give the bit-for-bit reduce** — it scales `τ_chem`
  but `Da_local` still varies with `T,p`; the reduce needs a *constant `Da_local`* (below).
- `_freeze_out_expand(comp_entry, far, Tt9, pt9, p9, V9_frozen, da_local_fn, nstep)` — rung-25's
  `_finite_rate_expand` **loop duplicated verbatim**, with the scalar `Da` promoted to a per-step
  `Da_local = da_local_fn(comp, T, p)`. In production `da_local_fn = lambda comp,T,p: τ_res /
  _tau_chem_recomb(comp,T,p)` with `τ_res = L/(0.6·V9_frozen)`. The `Da_local`-as-callable seam is what
  makes gate 2 bit-for-bit: the reduce test passes `lambda comp,T,p: Da` (the literal constant, no
  division), so the exponent `−Da_local·ds` is the *same float* as rung-25's `−Da·ds`. Returns
  `(T9, V9, comp9, dS, s_freeze, Da_entry, Da_exit)`. Everything else — the exact exponential
  relaxation, the implicit-trapezoid `dh=v·dp` energy bisection, the 2nd-law and atom-conservation
  asserts — is **byte-identical** to rung 25. *Duplication is deliberate* (gate 1 keeps rung 25
  literally untouched); the drift risk is accepted and gate 2 is the tripwire that catches it.
- `FreezeOutNozzleState` + `Gas.freeze_out_nozzle(far, Tt4, pt4, Tt9, pt9, p9, freeze_out)` — the
  public diagnostic. Reuses rung-25's (F)/(I)/(R) references (`_expand_nozzle`,
  `_irreversible_fast_expand`) and reports the freeze-out flow's `V9_freeze`, the freeze point
  `s_freeze`, `Da_local` at entry/exit, and the frozen-in exit `CO/(CO+CO2)`. **Only reads** the
  handed-in state; touches no cycle path.

**Does NOT add / deliberately out of scope:**

- **`finite_rate_nozzle` (rung 25) is literally untouched** — a *new* method beside it, per the
  one-diagnostic-per-rung pattern.
- **No new bound.** The freeze-out flow lands inside `[V9_F, V9_I]`; rung 25's three-state F/I/R
  picture is unchanged. Rung 26 resolves *where within* the bracket the flow freezes, not a fourth
  state.
- **No claim on the freeze LOCATION or the frozen-in composition.** The bracket is wide
  (`s ∈ [0.1, 0.7]` at `Tt4=2200`); location rides on `τ_res` (`L`), on the representative-reaction
  pick, and on the frozen↔equil composition bracket. Certify **existence** and **motion**, disclaim the
  number — the rung-16/23/24 precedent (they assert no emissions global-min *location* either).
- **No shifting turbine, no finite-rate exhaust NO** — inherited from rung 25's deferred list.

## The honest trade — one geometric knob, in exchange for anchored chemistry

Rung 25's `Da` was a **pure cartoon** (a single normalized Damköhler, no physical time). Rung 26 splits
it into `Da_local = τ_res/τ_chem(T,p)` — the **chemistry anchored to GRI-Mech 3.0**, one **geometric**
constant `τ_res` (`L≈0.5 m`) left. That is a **real reduction in unanchored content, not zero.** The
`τ_res` magnitude sets the freeze *location* (disclaimed) but **not** the freeze *existence* or its
*motion with `Tt4`* — both survive any O(1) `L` (the gate crosses for all reasonable `L`), which is why
those two are the certified claims and the location is not.

## The reduce contract — CONVERGENT/EXACT to rung 25, stated loudly

- **`freeze_out=None` ⇒ rung 25 untouched** (trivially exact — `freeze_out_nozzle` is a separate
  method; `finite_rate_nozzle` and the whole rung-1..25 suite are bit-for-bit unchanged).
- **Constant `τ_chem` ⇒ rung-25 `_finite_rate_expand` BIT-FOR-BIT.** With `_tau_chem_recomb` stubbed to
  a constant (so `Da_local ≡ Da` everywhere), `_freeze_out_expand` reproduces
  `_finite_rate_expand(Da)` to the ULP — the promotion of `Da` to `Da_local(s)` is the *only* change,
  and it collapses back identically. **This is the load-bearing reduce.**
- **`rate_scale→0` ⇒ (F) frozen**, transitively (`Da_local→0` ⇒ `relax→0` ⇒ composition pinned ⇒
  rung-25's `Da→0` frozen limit). Convergent, 2nd-order in `1/nstep` (inherited).
- **`rate_scale→∞` ⇒ (I) irreversible-fast**, transitively (the integrator's large-`Da` asymptote,
  rung-25 gate 3).

## Verification gates (priority order)

1. **REDUCE — `freeze_out=None` untouched** (LOAD-BEARING): `finite_rate_nozzle`, `nozzle_flow`, and
   the whole rung-1..25 suite are bit-for-bit unchanged.
2. **REDUCE — constant `Da_local` ⇒ rung-25 `_finite_rate_expand` bit-for-bit** (LOAD-BEARING): drive
   `_freeze_out_expand` with `da_local_fn = lambda comp,T,p: Da` (the literal constant injected at the
   `Da_local` level — NOT via `τ_res/τ_chem` division, which is a different float path) and it
   reproduces `_finite_rate_expand(Da)` **to the ULP** at matched `Da`/`nstep`. An *identity* reduce
   (unlike rung-25's own convergent integrator reduce): the `Da`→`Da_local(s)` promotion is the only
   change and it collapses back exactly. This gate is also the **drift tripwire** for the duplicated
   loop.
3. **LIMITS — `rate_scale→0` → (F)** (convergent 2nd-order) and **`rate_scale→∞` → (I)** (rung-25
   gate 3), both transitively through rung 25.
4. **THE FREEZE EXISTS — and is DORMANT LEAN, EARNS ITS KEEP HOT**: `Da_local < 1` throughout at
   `Tt4=1500` ⇒ freeze-out flow ≈ frozen (frozen from entry); `Da_local` crosses 1 at `Tt4 ≥ 1800`
   ⇒ the flow relaxes partway then freezes. Reproduces rung 14/25's arc from anchored chemistry.
   **Certify in COMPOSITION space, not `V9`:** the whole `[V9_F, V9_I]` bracket is sub-percent hot
   (rung-25 result), so `V9_freeze − V9_frozen` is a tiny wiggle — the ordering
   `V9_frozen ≤ V9_freeze ≤ V9_irrev_fast` holds but its tolerance must account for the small margin.
   The load-bearing observables are `s_freeze` and the **frozen-in exit `CO/(CO+CO2)`** (what a
   downstream calc would actually read), not `V9`.
5. **THE FREEZE POINT MOVES with `Tt4`** (THE RUNG): `s_freeze` strictly increases with `Tt4`.
   **Established on the real self-quenching integrator, NOT the probe table** — build the integrator
   and run this gate FIRST (§ feasibility gate ⚠); the probe's `s` values are an upper bound the
   marched flow undershoots. The certified claim is the *monotone motion*, not the `s` values. The
   physics a constant `Da` cannot express.
6. **THE KILL TEST — density drives the freeze against an opposing `T` effect** (non-circular):
   kill-`T` (`k` pinned) still freezes (`Da_local < 1`); kill-`p` (`[M]` pinned) `Da_local` **rises**
   above 1 (no freeze). Opposite sign to Arrhenius intuition.
7. **2nd LAW** — `dS ≥ 0` for the freeze-out flow (inherited from `_finite_rate_expand`).
8. **ATOM CONSERVATION** — the vector relaxation conserves C, H, O exactly (`|Δ| < 1e-12`, inherited).
9. **CYCLE UNTOUCHED** — a `freeze_out_nozzle` call does not perturb station 4 or the cycle `V9` (pure
   diagnostic; NO/dissociation never enter the cycle solve).
10. **GUARDS** — requires the equilibrium (rung-6) gas; rejects `p9 > pt9`; the exit-`T` bisection and
    the `_T_EXIT_FLOOR` guards are inherited from rung 25.

## NOT claimed (the rung-16/23/24 precedent)

- **The freeze LOCATION** (`s_freeze` to any precision) and **the frozen-in composition** — bracket is
  wide; both ride on `τ_res`, the representative-reaction pick, and the composition bracket.
- **The freeze-out `V9` magnitude** — rides on the same unanchored `L`. What is certified is that it
  sits inside `[V9_F, V9_I]`, and that the freeze *exists* hot / *is absent* lean / *moves* with `Tt4`.

## Done when

`Gas.freeze_out_nozzle` returns the freeze-out flow (`V9_freeze ∈ [V9_F, V9_I]`, `s_freeze`,
`Da_local` entry/exit, frozen-in CO) beside rung-25's F/I/R references; the freeze is **absent lean,
present hot, and its point moves with `Tt4`**; the kill test fires; `main.py` prints the rung-26 panel
(the `Da_local(s)` profile crossing 1, the freeze point walking downstream with `Tt4`, and the kill
test); `tests/test_rung26.py` is green on gates 1–10; the whole prior suite is untouched.

## The rung-27+ seam (keep it additive)

- **`T`-dependent freeze-out of exhaust NO** — the clamp corollary (rungs 14/17) still freezes an
  equilibrium-O lower bound through the nozzle; the same anchored `Da_local(T,p)` chain would resolve
  *where* NO itself freezes (Zeldovich is slow — it may freeze earlier than the recombination clock).
- **A resolved `τ_res` from the nozzle geometry** — replace the O(1) `L` with an area-schedule
  residence time, retiring the last geometric knob and *pinning* the freeze location.
- **A shifting turbine** — inherited from rung 25; a less-super-equilibrium entry shrinks `(R−I)` and
  moves the freeze point.
