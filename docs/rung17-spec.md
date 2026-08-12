# Rung 17 — The exhaust-NO clamp through the combustor-mixing-fidelity ladder (a rung-14 corollary from the rich side)

Rung 14 opened the **cycle-side nozzle seam** and, as its corollary, showed the rung-7 **dropped NO
clamp** (`cNO ≤ cNOe`) finally **fires** in the nozzle: on the cooling path equilibrium NO collapses,
so a realistic frozen exhaust NO is wildly super-equilibrium at the exit (`max_a ≈ 250` on the φ_p=1.0
mixed-out ICAO-band number). Rung 16 resolved the combustor NO **per pocket** through the β-PDF quench.
Rung 17 puts the two together and asks the rung-14 clamp question **at the rich RQL primary, through
three levels of combustor-mixing fidelity** — and gets the *opposite* headline from rung 14, for a
reason that is itself the lesson.

**The physics.** Carry the exhaust NO through the **same** rung-14 nozzle collapse to `T9`, but source
it from three progressively-faithful combustor-mixing models, and read the dropped-clamp margin
`a = [NO]/[NO]_e(T9)` for each:

| model | source | at the RICH φ_p=1.5 primary |
|---|---|---|
| **MIXED-OUT** | rung-8 `x_no_mix` (the standard shortcut) | `a ≈ 0.02` — **DORMANT** |
| **BULK QUENCH** | rung-11 `x_no_quenched` (mean-field re-making) | `a ≈ 3.4` — **FIRES** |
| **PER-POCKET** | rung-16 `ei_no_pocket_quench` (β-PDF segregation) | `a ≈ 13.6` — **FIRES harder** |

At a **rich** primary the mixed-out exhaust NO is deceptively **low** (φ≈0.4 mixed out makes ≈0 NO), so
a naive clamp check reads **dormant** — the crude shortcut **HIDES** the super-equilibrium NO. The
fuller models **reveal** it: the dilution re-making (rung 11) and the near-stoich β-PDF pockets (rung
16) put NO back, and it freezes super-equilibrium through the nozzle. **The ladder
`a_mixed < 1 < a_bulk < a_pocket` is the rung** — a *structural* ordering (the quench only adds NO; the
per-pocket excess is additive) whose IN-BAND firing (`a_bulk, a_pocket > 1` across the RQL J-band, not
universal — see § scope) is the lesson: three *independent* physics composing correctly.

> **The honest scope, up front (this rung is deliberately modest — read this before the numbers).**
> - **The identity is a stated fact, not a test.** `a_pocket/a_bulk = ⟨EI⟩_pocket/EI_bulk =` rung-16's
>   station-4 gap **by construction**: the nozzle denominator `x_no_e(T9)` is common to both and cancels,
>   and `x_no ∝ EI` at fixed overall far. No computation could make it false, so **no gate asserts it**
>   — the nozzle is a **no-op** on the pocket/bulk ratio. Rung 17 is a **synthesis** of rungs 11/16/14,
>   not new chemistry.
> - **Split the ordering from the firing — they are different strengths.** The **ORDERING**
>   `a_mixed ≤ a_bulk ≤ a_pocket` is **STRUCTURAL** (not a 2-point spot-check): the clamp-free quench
>   only *adds* NO to the mixed-out pool (`x_no_quenched ≥ x_no_mix` in the dormant regime) and the
>   per-pocket excess is additive (`x_no_pocket = x_no_bulk + κ·⟨EI⟩_pocket`, `⟨EI⟩ ≥ 0`). And
>   `a_mixed < 1` is robust (a rich primary makes ≈0 NO). **These are the certified claim.** The
>   **FIRING** (`a_bulk > 1`, `a_pocket > 1`) is the **un-pinned threshold**: it holds across the RQL
>   J-band but is **NOT universal** — as the quench gets *fast* (`J→∞`) `x_no_quenched → x_no_mix` (the
>   rung-10 `τ_q→0` reduce: fast quench = ideal quench = mixed-out), so even `a_bulk → a_mixed < 1`
>   (dormant). probe4 shows the slide: `a_bulk = 5.0 → 3.35 → 2.0` for `J = 100 → 225 → 625` (>1 in-band,
>   heading for <1). Every firing magnitude **and the gap** ride on un-pinned scales (`C_e`, `τ_res`,
>   `H`, `J`; the gap moves ~23% over `C_e = 0.15 → 0.20`). **The headline — mixing-out HIDES super-eq
>   NO — rides on the IN-BAND firing, and that in-band firing is the lesson, not a universal claim.**
> - **Contrast rung 14, don't collide with it.** Rung 14 fires *on* the φ_p=1.0 mixed-out number
>   (`a≈250`) — the **zoned-vs-unzoned** axis. Rung 17 is the **mixing-fidelity** axis at the **rich**
>   φ_p=1.5 primary, where that same mixed-out number is dormant. Not a contradiction: the same
>   dropped-clamp lesson from the rich side, where the mixed-out shortcut is **unconservative**.
> - **The clamp is DORMANT at station 4.** `max_a_quench < 1` over the pockets — the combustor NO is
>   sub-equilibrium; it only goes super-equilibrium **in the nozzle** (the rung-14 collapse). A pocket
>   going super-equilibrium *at the burner* (hotter `Tt4` / longer dwell) is a further, still-deferred
>   seam.

> **Read `docs/rung14-spec.md` (the nozzle collapse + the dropped clamp) and `docs/rung16-spec.md` (the
> per-pocket β-PDF quench) first**, and `docs/plans/rung17-anchor-superequilibrium-exhaust.md`
> (numbers-before-code: the Heywood NO-freezing anchor, the ladder, the scale sweep). This file states
> only what *composes*. No new chemistry, no new integrator, no new config — rung 17 reuses `zoned_nox`
> (rungs 8/11/16) and `nozzle_flow` (rung 14) **verbatim**, both untouched, so the cycle stays
> **bit-for-bit rung 6**.

---

## What rung 17 adds (and what it deliberately does not)

**Adds** (all in `turbojet/gas.py`, all *decoupled* from the cycle):

- `ExhaustNOxClampState` dataclass — the ladder: `T9`, the common denominator `x_no_e_exit`, the
  `no_collapse_ratio` (rung 14), the three exhaust-NO mole fractions
  (`x_no_mixed_out`/`x_no_bulk_quench`/`x_no_pocket`) and their margins
  (`a_mixed_out`/`a_bulk_quench`/`a_pocket`), the transparency pair
  (`ei_no_quenched`/`ei_no_pocket_quench`) and their ratio `gap_pocket_over_bulk` (≡ `a_pocket/a_bulk`,
  the rung-16 station-4 gap), and `max_a_quench` (the station-4 clamp dormancy). Two predicates:
  `hides_super_eq` (mixed-out dormant **and** bulk fires — the headline) and `ladder_monotone`.
- `Gas.exhaust_no_clamp(far, Tt3, Tt4, p, Tt9, pt9, p9, phi_primary, mixing, pocket_quench, tau)` — the
  public diagnostic. It calls `zoned_nox` three ways (rung 8/11/16) for the numerators, `nozzle_flow`
  **once** (rung 14) for the common denominator `x_no_e(T9)`, and forms the three margins. It **only
  reads** the state handed in and touches **no** cycle path.

**Does NOT add / deliberately out of scope:**

- **No new chemistry, integrator, or config.** Every number is a rung-8/11/16 or rung-14 output read
  verbatim; the only new code is the arithmetic that arranges them into the ladder.
- **No claim on the firing magnitude.** `a_bulk`, `a_pocket`, and the gap are scale-dependent (§ scope).
  The certified content is the *direction*.
- **No burner-side super-equilibrium.** The clamp is dormant at station 4 (`max_a_quench < 1`); the NO
  only goes super-equilibrium in the nozzle. A pocket going super-eq *at the burner* (hotter `Tt4`,
  longer dwell — the rung-14 exhaust-NO corollary in reverse) stays deferred.
- **No super-equilibrium O / prompt NO.** As in rungs 7–16 the frozen exhaust NO is an equilibrium-O
  **lower bound**, so every `a` here is a lower bound too.

---

## The one thing that makes it work (stated loudly — it IS the rung)

**The ladder direction encodes two real rung results composing correctly at a regime where the crude
model lies.** (1) The rich primary makes little NO mixed-out (rung 9's bell collapse), but the dilution
sweeps *through* stoichiometric and **re-makes** it (rung 10/11) — so `a_bulk > a_mixed`. (2) NO peaks
**at** stoichiometric, so a segregated β-PDF (near-stoich pockets) has a mean **above** the well-mixed
value (rung 13/16's sign of segregation) — so `a_pocket > a_bulk`. Both lifts survive the nozzle
because the collapse denominator is common. The teaching payoff: **the mixed-out shortcut is
unconservative precisely at the rich RQL primary** real low-NOx combustors use — you must resolve the
mixing (quench + pockets) to see the exhaust NO that actually leaves the engine, and rung 14 proves
that NO is frozen super-equilibrium at the exit.

> ### SHARPENING (2026-08-12, from the Rust port's slice E) — where the firing stops, and what
> does NOT stop with it
>
> This rung is careful that the firing is IN-BAND and not universal: as the quench gets fast
> (`J→∞`) the bulk re-making vanishes, `x_no_quenched → x_no_mix`, and `a_bulk → a_mixed < 1`.
> That is stated everywhere and **measured nowhere** — the suite tests one `J`. Bisecting on the
> sign of `a_bulk − 1` at the shipped design point:
>
> | `C_e` | `a_bulk` at the RQL `J` = 225 | `a_bulk` crosses 1 at |
> |---|---|---|
> | 0.15 | 4.356 | **`J` ≈ 3978–4000** |
> | 0.20 | 3.272 | **`J` ≈ 2457–2470** |
>
> So the firing survives about **11× past the shipped band**, and the edge itself moves 1.6× on an
> entrainment scale nothing pins — which is the "rides on un-pinned mixing scales" caveat made
> numerical. Deliberately left as a coarse bracket: the crossing is a smooth root, so a resolved
> digit would just be a `C_e`/`τ_res`/`H` reading in disguise.
>
> **The correction is what happens PAST the crossing.** The natural reading of the caveat — that
> `J→∞` sends the whole ladder dormant — is wrong:
>
> | `J` | `a_mixed` | `a_bulk` | `a_pocket` | gap | `hides_super_eq` | `ladder_monotone` |
> |---|---|---|---|---|---|---|
> | 225 (RQL) | 0.01582 | 3.272 | **11.06** | 3.38 | true | true |
> | 4 000 | 0.01582 | 0.789 | **12.82** | 16.26 | **false** | true |
> | 16 000 | 0.01582 | 0.402 | **14.34** | 35.65 | **false** | true |
>
> `a_pocket` **RISES** while `a_bulk` falls through 1. The mechanism is rung 16's own, and its
> docstring says so already: `ei_no_pocket_quench` = term 1, the mean-field bulk riding
> `τ_mean ∝ 1/√J` and collapsing, **plus** term 2, the β-PDF integral at
> `τ_core = τ_res·(1+b_u·u)` — which `PocketQuenchPDF.core_dwell` describes as an *absolute*
> residence whose "NO penalty survives `J→∞`", with `u` growing off-optimum. Measured term 2 =
> 0.646 → 0.997 → 1.155 g/kg over those three `J`.
>
> **So the two predicates say different things and this rung's prose blurs them.**
> `hides_super_eq` is defined on `a_bulk` and is therefore the in-band claim; `ladder_monotone` is
> the claim about the ladder, and it survives everywhere measured. Checked against the obvious
> alternative explanation — the segregation `g` is PINNED at `g_max` = 0.3 from `J` ≈ 225 upward
> and unpinned at `J` = 25, so the clip is exercised on both sides and the rise is not the width
> moving. Gated as `rust/tests/rung17.rs::the_ladder_does_not_go_dormant_with_the_bulk` and
> `…::the_firing_band_edge_is_located_and_moves_with_the_scale`;
> `docs/plans/todo-rust-port.md` § 4.10 finding 2.

---

## The equations — a composition, no station changes

Every cycle station is **bit-for-bit rung 6**. `exhaust_no_clamp` reads three `zoned_nox` results and
one `nozzle_flow`:

```
COMMON DENOMINATOR (rung 14):  x_no_e(T9) = Kp_NO(T9)·√(x_N2·x_O2)   at the frozen nozzle-exit T9
NUMERATORS (rung 8/11/16):
   x_mixed  = zoned_nox(…).x_no_mix                                  [rung 8, no quench]
   x_bulk   = zoned_nox(…, mixing).x_no_quenched                     [rung 11, mean-field quench]
   x_pocket = κ · zoned_nox(…, mixing, pocket_quench).ei_no_pocket_quench,  κ = x_bulk/ei_no_quenched
                                                                     [rung 16, β-PDF mean; κ from x∝EI]
LADDER:  a_i = x_i / x_no_e(T9)     ⇒     a_mixed < 1 < a_bulk < a_pocket   (the certified DIRECTION)
IDENTITY (stated, not gated):  a_pocket/a_bulk = x_pocket/x_bulk = ei_no_pocket_quench/ei_no_quenched
                                = rung-16's station-4 gap        (x_no_e(T9) and κ both cancel)
```

- `κ = x_no/EI` is a pure function of the overall far (same `n_tot`, `n_fuel` per mol air), so it is
  **common** to the bulk and every pocket — which is exactly why the nozzle cancels in the ratio.
- **Requires** the equilibrium (rung-6) gas and **both** a `mixing` and a `pocket_quench` (the bulk and
  per-pocket rungs need the jet). Back-pressure guard `p9 ≤ pt9` is inherited from `nozzle_flow`.

---

## Verification gates (priority order)

1. **THE LADDER (load-bearing).** At the rich RQL design point (φ_p=1.5): the **ordering**
   `a_mixed_out < a_bulk_quench < a_pocket` (structural — the quench adds NO, the excess is additive)
   with `a_mixed_out < 1` (robustly dormant), and the **in-band firing** `a_bulk_quench > 1` **and**
   `a_pocket > 1`. `hides_super_eq` and `ladder_monotone` are both True. Three independent physics
   composing. (The firing is *in-band*, not universal — gate 4 and § scope name the fast-quench edge.)
2. **The rung-14 contrast (the other side of the same lesson).** The **mixed-out** clamp fires at
   φ_p=1.0 (rung 14's `a ≫ 1`) but is **dormant** at φ_p=1.5 (`a < 1`) — the rich primary hides it.
   (Runs the *same* `x_no_mix`-through-the-nozzle construction rung 14 uses, at the two φ_p.)
3. **The identity is exact (reported, not a physics gate).** `a_pocket/a_bulk == gap_pocket_over_bulk`
   to machine precision — a consistency check on the arithmetic (the nozzle no-op), *documented as
   algebra*: it cannot fail by construction, so it is a **witness**, not a discriminating test.
4. **Scale-sensitivity — the ORDERING holds, the MAGNITUDE does not.** Sweep `C_e`: the **ordering**
   `a_mixed<1<a_bulk<a_pocket` holds at every scale (structural), while the **magnitudes and the gap
   move** (`gap` ~23% over `C_e=0.15→0.20`; `a_bulk` `4.46→3.35`). The gate asserts **both**: ordering
   invariant, magnitude variant — the honest scope made a test. **The firing is verified in-band, NOT
   claimed universal**: a fast enough quench (`J→∞`) drives `a_bulk→a_mixed<1` (the rung-10 `τ_q→0`
   reduce — probe4: `a_bulk 5.0→3.35→2.0` over `J=100→625`), the deliberately-named edge of the claim.
5. **Reduce-to-components (exact).** The numbers `exhaust_no_clamp` uses are bit-identical to the
   underlying diagnostics: `x_no_bulk == zoned_nox(…, mixing).x_no_quenched`, `a_bulk ==
   nozzle_flow(…, x_no_frozen=x_no_bulk).max_a`, and `ei_no_pocket_quench ==` the rung-16 value — it
   *composes*, it does not recompute.
6. **Cycle untouched.** An `exhaust_no_clamp` call leaves the cycle `far`/stations bit-identical —
   rung 6. The whole rung 1–16 suite stays green.
7. **Clamp dormancy at station 4.** `max_a_quench < 1` — the combustor NO is sub-equilibrium; the
   super-equilibrium is a **nozzle** phenomenon (the collapse), not a burner one.
8. **Guards.** Requires the equilibrium gas; requires **both** `mixing` and `pocket_quench`; the
   back-pressure guard `p9 ≤ pt9` (inherited).

## Conservation asserts (rung-17 deltas)

- No new asserts of its own beyond the two guards (equilibrium gas; `mixing`+`pocket_quench` present).
  Every underlying assert still fires: the rung-7 **K-check** + **trace guard** at every trajectory `T`
  in the bulk and pocket quenches, the rung-13 **mean-preservation** on the β-PDF quadrature, the
  rung-14 **bracket guard** on the nozzle-exit bisection and its `p9 ≤ pt9` back-pressure guard.

## Done when

`Gas.exhaust_no_clamp` returns the `ExhaustNOxClampState` ladder (mixed-out dormant → bulk fires →
pocket fires harder, with the identity witnessed and the scope stated); `main.py` prints the rung-17
panel (the ladder, the rung-14 φ_p contrast, the per-pocket exhaust-NO *distribution* as
visualization, and the scale-sweep showing the ordering is structural while magnitudes move — the
firing in-band, not universal); `tests/test_rung17.py` is
green; the whole prior suite is untouched.

## The rung-18+ seam (keep it additive)

- **Burner-side super-equilibrium** — the regime where a pocket goes super-equilibrium **at the
  combustor** (hotter `Tt4` / longer dwell), so `max_a_quench > 1` and the dropped clamp fires *before*
  the nozzle. Rung 17's clamp is dormant at station 4; this seam is the rung-14 exhaust-NO corollary
  turned inward.
- **A transported PDF / dwell spectrum** — predict `g(C)` and `τ_core(C)` (and hence the un-pinned
  firing magnitude) from a mixing equation instead of modeling them, which is what would let rung 17
  claim a *magnitude*, not just a direction.
- **Finite-rate nozzle chemistry** — the real flow *between* rung 14's frozen and equilibrium bounds;
  the exhaust NO would then partially relax toward `x_no_e(T9)` rather than freezing, softening `a`.
- **Super-equilibrium O / prompt (Fenimore) NO** — every `a` here is an equilibrium-O lower bound.
