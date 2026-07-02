# Rung 12 — Spatial unmixedness: the two-stream variance layer that recovers the Holdeman optimum

Rung 11 derived the quench *rate* from the jet momentum-flux ratio `J`, but on a **single
well-mixed core** (mean-field), so its `J`-sweep is **monotone**: a stronger jet only ever re-makes
*less* NO. Rung 11 named the missing piece out loud — the dilution-jet **optimum** is a
**spatial-variance** effect (an over-penetrating jet leaves an un-mixed hot near-stoich core), and
a mean field has no variance by construction. Rung 12 adds exactly that variance, with the smallest
closure that exhibits it — **two streams** — and the NO-vs-`J` curve finally **turns back up**, with
the emissions minimum landing **AT** the classic **Holdeman dilution-jet optimum** `C=(S/H)·√J≈2.5`.

**Honest scope up front:** the optimum's *location* is an **input** (we place the unmixedness kink
at Holdeman's empirical `C_opt`), not a *prediction* of 2.5 — a calibrated model at this tower's
altitude, like `C_e`/`τ_q`. The **certified** content is the both-flanks **turn-up** and the min's
**`(H/S)²` shift** with the spacing.

> **Read `docs/rung11-spec.md` first**, and `docs/plans/rung12-anchor-unmixedness.md` (numbers-
> before-code: the two-stream closure, the **two design choices** that make it work — an *absolute*
> core dwell (survives `J→∞`) and a *kinked* unmixedness (pins the min at `C_opt`) — the
> machine-checked worked example (the turn-up, the min AT `C_opt`, the `(H/S)²` shift, the exact
> reduce)). This file states only what *changes*; the Zeldovich rates, the clamp-free schedule-aware
> integrator, the τ_q-independent trajectory, the jet-mixing `τ_q`/schedule, the `a6`/`a7`/`Kp`/
> `_equil_solve` substrate, and the "derive before you code" / conservation-assert contract all
> carry over **unchanged**.

---

## What rung 12 adds (and what it deliberately does not)

**Adds:**

- **A spatial-unmixedness config** (`Unmixedness`) — the dilution-jet spacing `S` (which, with the
  paired `JetMixing`'s duct height `H` and momentum `J`, forms the Holdeman group `C=(S/H)√J`), the
  uniformity optimum `C_opt` (≈2.5), the under-mixed-core dwell-at-optimum `τ_res` and its
  off-optimum growth `b_u`, and the core-fraction sensitivity/cap `k_u`, `w_max`. Physical defaults;
  it **rides on** a `JetMixing` (it needs the jet's `J` and `H`), so `unmixedness` is passed *with*
  `mixing`.
- **A two-stream (bulk + core) quench where the CORE carries the off-optimum penalty (two ways)** —
  the flow splits into a mean-field **bulk** (fraction `1−w`, quenched at the rung-11 jet time
  `τ_mean(J)` — the monotone-falling reference, *not* a function of `C_opt`) and an under-mixed
  **core** (fraction `w`, quenched at an **absolute** dwell `τ_core(C)=τ_res·(1+b_u·u)` that misses
  the fast jet). Off-optimum the core worsens **two ways**: more gas segregates (`w↑`) AND it lingers
  longer (`τ_core↑`). Both streams traverse the **same** τ-independent trajectory; rung 12 adds **no
  new chemistry and no new integrator**, only a second `_quench_no` call at `τ_core` and a mass-weight.
- **The unmixedness** `u(C)=|ln(C/C_opt)|` — an **L1 (kinked)** distance from the optimum, `0` at
  `C_opt`, symmetric in `ln C`, driving both `w=min(w_max, k_u·u)` and the core dwell.
- **The payoff** — a **non-monotone `J`-sweep** whose minimum lands **AT `C_opt`**:
  `EI_no_unmixed = (1−w)·EI(τ_mean) + w·EI(τ_core)` **FALLS then RISES**, with the minimum pinned at
  the Holdeman uniformity optimum (`J_min = J_opt`), shifting **exactly as `(H/S)²`** with the spacing.
- **`main.py` panel + `NOTES.md` section + `tests/test_rung12.py`.**

**Deliberately does NOT:**

- **Touch the cycle.** NO is still trace and decoupled; the variance layer is opt-in via
  `unmixedness`. Every cycle station is **bit-for-bit rung 6** (the whole rung 1–11 suite stays
  green). The reduce is a **short-circuit**: `unmixedness=None` runs the exact rung-11 mean-field
  path (and its `mixing=None`/`tau_q=None` reduces below it).
- **Resolve the mixing field.** This is a **two-stream** closure — the *minimal* variance model, not
  a PDF-transport / CFD resolution of the cross-plane. It captures that *some* gas dwells far longer
  than the mean and that the segregation is governed by the Holdeman group; it does **not** claim the
  spatial structure. The **turn-up**, the **optimum AT `C_opt`**, and the **`(H/S)²` shift** are the
  content.
- **Claim `EI_total` is a function of `C` alone.** It is not — the bulk rides `J` directly. It is the
  optimum **location** that pins at `C_opt` (via the kink). `S`, `τ_res`, `k_u`, `b_u`, `w_max` are
  un-anchored / order-of-magnitude (like `C_e`, `α`, `φ_p`, `τ_q`); `C_opt≈2.5` is Holdeman's value.
- **Add super-equilibrium O / prompt (Fenimore) NO** — still deferred (rung-7 seam); the two-stream
  spike stays an equilibrium-O lower bound. Held **φ_p ≤ 2.0** (soot bound, rung 9).

---

## The two design choices that make it work (stated loudly — they ARE the rung)

**(1) The core dwell is ABSOLUTE, so the turn-up survives `J → ∞`.** The naïve move — model
unmixedness as a *mean-preserving spread* of `τ_q`, justified by convexity (Jensen) — is
**backwards**: with the clamp **dormant** (max_a ≪ 1 here), NO accumulates as **rate × dwell**, so
`EI ∝ τ_q` (the rung-10 anchor: `EI/τ_q` *falls* 1.107→1.083, mildly **concave**). A spread adds
≤ 0; a *multiplicative* core `τ_core ∝ τ_mean ∝ 1/√J → 0` vanishes as `J → ∞`, leaving the curve
monotone — the rung-11 ceiling. An **absolute** `τ_core = τ_res·(1+b_u·u)` (the dilution-zone
residence, J-independent) keeps the core penalty finite — and *growing* off-optimum — so the total
turns up.

**(2) The unmixedness is KINKED, so the min lands AT `C_opt`.** With a *smooth* (parabolic) `w`,
`w'(C_opt)=0`, so at `C_opt` the total derivative is just the bulk's `dB/dlnC < 0` — still falling,
and the min drifts to a **stronger** jet than the uniformity optimum. The **kink** (`u=|ln(C/C_opt)|`)
gives `w` a non-zero slope at `C_opt`, so the turn-up starts **there** the moment the unmixedness
beats the penetration benefit: `k_u·[EI(τ_core)−EI(τ_mean)] > EI(τ_mean)` at `C_opt` — exactly the
condition that an emissions optimum **exists** at the uniformity point. The default `k_u=2.5` clears
it, so the EI-min **pins at `C_opt` for all `S`** ⇒ `J_min = J_opt`, shifting as `(H/S)²`. And
`w → 0` at `C_opt` ⇒ the optimum point sits **exactly** on the rung-11 curve (a clean invariant).

---

## The equations — a second stream + a mass-weight, no station changes

Every cycle station is **bit-for-bit rung 6**. `zoned_nox` is the rung-8..11 flow; rung 12 only adds
the second (core) quench and the mass-weight when an `Unmixedness` is passed:

```
MEAN FIELD  (unmixedness=None):  the exact rung-11 jet-mixing quench                → rung 11
TWO-STREAM  (unmixedness=Unmixedness(S,…), REQUIRES mixing=JetMixing(J,…)):
   C   = (S/H)·√J                                   Holdeman group (H, J from the jet)
   u   = |ln(C/C_opt)|                              unmixedness (KINKED L1; 0 at C_opt)
   w   = min(w_max, k_u·u)                          segregated core fraction (0 at C_opt)
   τ_core = τ_res·(1 + b_u·u)                        core dwell (ABSOLUTE; grows off-optimum)
   EI_bulk = EI(τ_mean = mixing.tau_q)              the rung-11 mean field (∝ 1/√J; the reference)
   EI_core = EI(τ_core)                             the lingering core (survives J→∞)
   EI_no_unmixed = (1 − w)·EI_bulk + w·EI_core      → FALLS then RISES; minimum AT C_opt (J_min=J_opt)
```

- **`unmixedness` REQUIRES `mixing`** — it needs the jet's `J` and `H` (assert). `unmixedness=None`
  keeps the exact rung-11 path; `k_u=0` is bit-for-bit the mean-field bulk at every `J`.
- **Standing asserts (rung-12 deltas):** the rung-7 **K-check** + **trace guard** still bind at every
  trajectory `T` (reused via `_quench_no`, both streams); `S>0`, `C_opt>0`, `τ_res>0`, `0<w_max≤1`,
  `k_u≥0`, `b_u≥0`; `unmixedness ⇒ mixing`; `n_NO ≥ 0` (negatives only — **no** equilibrium cap,
  carried from rung 10/11); the clamp-dormancy `max_a < 1` now spans **both** streams.

---

## Verification gates (priority order)

1. **Reduce-to-rung-11 (load-bearing, exact by construction).** `unmixedness=None` short-circuits to
   the rung-11 path *before* any two-stream code — every existing call is **bit-for-bit rung 11**
   (hence 10/9/6; the whole rung 1–11 suite stays green). Second level: `Unmixedness(k_u=0)` ⇒ `w≡0`
   ⇒ the total equals the mean-field bulk at every `J`, bit-for-bit; and `C=C_opt` ⇒ total = bulk.
2. **The turn-up (THE lesson).** `EI_no_unmixed` is **non-monotone** in `J`: it **falls then rises**,
   an interior minimum. Rung 11's monotone fall is broken; the mean-field bulk is still falling at
   the far edge, so the **variance** is what turns it up. A monotone curve fails it.
3. **The optimum is AT the Holdeman group `C_opt`.** `J_min == J_opt = (C_opt·H/S)²` — shrink the
   spacing `S` and the EI-min moves to higher `J` **exactly as `(H/S)²`** (the kink pins it at `C_opt`
   for every `S`). This is THE Holdeman claim.
4. **At `C_opt` the two-stream total == the mean-field bulk** (`w=0` there) — the clean seam/invariant.
5. **The core penalty survives `J → ∞` and grows off-optimum.** `EI_core` at a strong jet far exceeds
   the fast bulk (an absolute, growing dwell — not the vanishing jet time), and is minimised at
   `C_opt`. The load-bearing physics that keeps the turn-up alive at strong jets.
6. **`w(C)` is the unmixedness** — 0 at `C_opt`, rising (kinked) on both flanks, symmetric in `ln C`.
7. **Cycle untouched.** Re-running the cycle after an `unmixedness` `zoned_nox` call leaves station 4
   bit-for-bit (pure diagnostic).
8. **Clamp dormancy persists + K-check binds** along the trajectory, over **both** streams
   (`max_a < 1`; even the longest-lingering core).

## Conservation asserts (rung-12 deltas)
Carry over rung 6/7/8/9/10/11's, plus: `unmixedness ⇒ mixing`; the `Unmixedness` positivity/range
guards (`S,C_opt,τ_res>0`, `0<w_max≤1`, `k_u≥0`, `b_u≥0`); `w∈[0,w_max]` with `w=0` at `C_opt`; the
clamp-free integrator still guards **negatives only** (no equilibrium cap), and the `max_a<1`
dormancy gate now spans the bulk **and** the core.

## Done when
Reduce-to-rung-11 holds exactly (short-circuit; rungs 1–11 green, untouched; cycle bit-for-bit rung
6); the `J`-sweep **turns back up** with the minimum **AT `C_opt`** (the recovered Holdeman optimum),
`J_min = J_opt` **shifting as `(H/S)²`** with the spacing; the core penalty survives strong jets and
grows off-optimum; the K-check/clamp-dormancy gates hold over both streams. `main.py` gains a rung-12
unmixedness panel (the turn-up `J`-sweep: mean-field bulk vs two-stream total; the min at `C_opt`;
the `(H/S)²` shift); `NOTES.md` gains a rung-12 section (the variance the mean field missed; the two
design choices; why the min pins at `C_opt`); `CLAUDE.md` scope + rung table + deferred seams updated
(spatial unmixedness / Holdeman optimum **done** — with super-equilibrium O / prompt NO and the
frozen-vs-equilibrium nozzle still carved out).

## The rung-13+ seam (keep it additive)
Rung 12 adds variance with the *minimal* closure — **two** streams. Next seams, all still additive on
this substrate: (a) **a resolved mixing PDF** — more than two streams / a mixture-fraction
distribution (a β-PDF or PDF-transport closure), so the segregation is *predicted* from the jet
field rather than parameterised by `w(C)`/`τ_core(C)`; (b) **super-equilibrium O / prompt (Fenimore)
NO** — the richer radical pool in the mixing shear layer that *is* this under-mixed core, above the
equilibrium-O lower bound; (c) the still-open **equilibrium-vs-frozen nozzle expansion** (rung-6
seam). Only *how finely* the mixing field is resolved, *on what radical pool*, and *how the nozzle
freezes* changes.
