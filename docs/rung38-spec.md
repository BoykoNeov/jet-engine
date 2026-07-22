# Rung 38 — Two-spool matching: the triangular cascade (no simultaneous solve)

Rungs 31–37 are all **single-spool**: one compressor, one turbine, one shaft. Every real
turbojet above modest thrust splits the compression into a low-pressure (LP) spool and a
high-pressure (HP) spool, each with its **own** turbine and its **own** shaft — the two
shafts are mechanically independent (they can and do run at different, only aerodynamically
related, speeds). This rung builds the first two-spool model: a plain **turbojet** LPC+HPC
(no bypass — a fan/bypass split is a different engine and a separate seam), matched the way
rung 31 matched the single-spool engine — pure isentropic component maps plus choked-throat
geometry, no compressor efficiency maps yet (that pairing, rung 32 for a single spool, is a
deferred seam here too).

---

## Station layout

```
0 -> 2 -> 25 -> 3 -> 4 -> 45 -> 5 -> 9
     LPC  HPC  burn  HPT  LPT  nozzle
```

`2` = LPC inlet (= overall compressor inlet), `25` = LPC exit / HPC inlet, `3` = HPC exit
(= overall compressor exit, the burner inlet), `4` = burner exit / HPT inlet, `45` = HPT
exit / LPT inlet, `5` = LPT exit / nozzle inlet, `9` = nozzle exit. No station between `45`
and the nozzle other than `5` — no reheat/afterburner, no bleed, no interstage loss modeled
(inherited scope limits).

---

## What is genuinely new: a THIRD choked throat

Rung 31 pinned the single turbine by two choked throats: the turbine NGV (station 4, area
`A4`) and the nozzle (station 9, area `A8` via rung 30's `_sonic_throat`). A two-spool engine
adds a throat that does not exist in the single-spool model at all: the **LP turbine NGV** —
the inter-turbine duct at station `45`, area `A45` — between the HP turbine's exit and the LP
turbine's inlet. Turbine nozzle guide vanes run at large pressure ratios and stay choked over
a far wider range than an external nozzle does (whose exit ratio collapses toward 1 as you
throttle back), so — exactly as rung 31 **assumed** the single NGV choked without deriving it
station-by-station — this rung assumes **both** HPT NGV (`A4`) and LPT NGV (`A45`) stay choked
throughout the modeled envelope. The **nozzle** (`A8`) still gets the real treatment: rung
30/33's choked-or-subsonic branch. See Concessions for what "assumed" is carrying and the
scope line this draws.

---

## The derivation: the rung-31 `(★)` trick, applied twice, chained

Write every shaft balance in **specific** (per-unit-mass) work — mass flow is common to both
compressors and both turbines (no bypass, no bleed), so it cancels out of both shaft balances
exactly as it did in rung 31's single one. Two mechanically independent shafts, two balances:

```
LP shaft:  eta_m*(1+f)*[h_t(Tt45) - h_t(Tt5)]   =  h_c(Tt25) - h_c(Tt2)        (LP BALANCE)
HP shaft:  eta_m*(1+f)*[h_t(Tt4)  - h_t(Tt45)]  =  h_c(Tt3)  - h_c(Tt25)       (HP BALANCE)
```

Five unknowns sit in the way of a design-point run: `pi_LPC, pi_HPC, pi_HPT, pi_LPT, f` (`pt4`
follows algebraically from the first two once the loop below closes). Naively this looks like
it needs a joint solve over two spools of two unknowns each. It does not, because of what
"both NGVs choked" buys:

**Step 1 — `tau_HPT` is pinned by geometry, independent of BOTH shafts.** Exactly rung 31's
`(★)` (`pi_t/sqrt(tau_t) = A_in*MFP_in / (A_out*MFP_out)`), applied between `A4` and `A45`
instead of `A4` and `A8`:

```
pi_HPT / sqrt(tau_HPT)  =  A4 * MFP(Tt4, f) / (A45 * MFP(Tt45, f))                    (★-HP)
```

`MFP` is the pressure-level-independent sonic mass-flow parameter (`choked_mfp`, rung 31/30).
Bisect on `pi_HPT` exactly as `OffDesignMatcher._solve_turbine` already does — the same
monotone bracket, same tolerance. This determines `Tt45 = Tt4 * tau_HPT` from `Tt4` and the
two areas ALONE: no compressor, no shaft, no `pi_LPC`/`pi_HPC` anywhere in the equation.

**Step 2 — `tau_LPT` is pinned by geometry too, same trick, one throat down.** Between `A45`
and `A8` (with the nozzle's `pi_n` loss folded in, exactly as rung 31's original `(★)` did):

```
pi_LPT / sqrt(tau_LPT)  =  A45 * MFP(Tt45, f) / (A8 * pi_n * MFP(Tt5, f))             (★-LP)
```

— valid **only while the nozzle stays choked** (see Scope below). This determines
`Tt5 = Tt45 * tau_LPT` from `Tt45` (already known from Step 1) and the two areas alone —
again no compressor, no shaft.

**Step 3 — the LP shaft balance: energy fixes `Tt25`, THEN efficiency converts it to
`pi_LPC`.** The LP balance's right-hand side is `h_c(Tt25) - h_c(Tt2)`; `Tt2` is the flight
condition (always known) and the left-hand side (`Tt45`, `Tt5`, both from Steps 1–2) is now a
pure number. This solves for the **actual** exit temperature `Tt25` by energy conservation
alone — `eta_LPC` has not entered yet. Only the LAST substep, converting that already-fixed
`ΔT` into a pressure ratio (the isentropic-efficiency inverse, the exact rung-31 compressor
inverse), reads `eta_LPC`: `pi_LPC = pr_c(Tt2 + eta_LPC·(Tt25−Tt2)) / pr_c(Tt2)`.

**Step 4 — the HP shaft balance triangularizes ONTO Step 3's `Tt25`.** The HP balance's
right-hand side is `h_c(Tt3) - h_c(Tt25)` — and `Tt25` is now known, from Step 3 (energy only,
not `pi_LPC` or `eta_LPC`). Solve for `Tt3` by energy, THEN invert with `eta_HPC` for `pi_HPC`.

So the two-spool match is a **triangular cascade**, not a simultaneous 2×2 solve:
`Tt4 -> (★-HP) -> Tt45 -> (★-LP) -> Tt5 -> LP balance -> Tt25 -> pi_LPC -> HP balance -> pi_HPC`.
Five unknowns, zero simultaneous pairs.

### The precise claim — and where it is NOT "the spools are independent"

It would be tempting (and WRONG) to read the cascade as "the LP spool solves independent of
the HP spool." It does not: `Tt45` — an HP-turbine quantity, shaped by `eta_HPT` — is the
energy source for BOTH the LP turbine (Step 2) and, later, the HP compressor (Step 4); a
worse `eta_HPT` leaves LESS enthalpy for the LP turbine to extract, which changes `Tt5`,
hence `Tt25`, hence `pi_LPC` too. **`eta_HPT` and `eta_LPT` (and the three areas) move BOTH
compressor ratios** — they sit upstream of the whole energy cascade. Verified directly (gate
3): perturbing `eta_HPT` or `eta_LPT` moves both `pi_LPC` and `pi_HPC`.

What IS airtight, and is the actual finding, is narrower and sits one level down:

> **Each compressor's OWN isentropic efficiency is a terminal leaf.** `eta_LPC` enters
> *only* the last algebraic step of Step 3 — converting the already energy-fixed `Tt25` into
> a pressure ratio — and never feeds back into anything upstream; `Tt25` itself, and
> everything Step 4 computes from it, is completely blind to `eta_LPC`. Symmetrically,
> `eta_HPC` enters only Step 4's last step and cannot reach `pi_LPC` — Step 3 finishes in
> full before Step 4 even begins, and Step 4 never revisits Step 3. **The two compressor
> PRESSURE ratios are therefore never bound by a joint (2×2) nonlinear system**, because
> compressor efficiency only ever maps an already-fixed `ΔT` to a pressure ratio — it never
> feeds back into the energy balance that fixed `ΔT` in the first place. A real
> compressor-MAP-based matcher does NOT have this property (a map's flow capacity is a
> function of the pressure ratio too, so efficiency-like quantities re-enter the energy/mass
> balance and force a genuine simultaneous solve — see "the finding is a no-map model
> artifact" below).

### The one place the spools still talk beyond the cascade: the scalar `f`

The cascade above is exact **at a fixed `f`**. `f` itself is the shared scalar rung 31 already
carries in its own `(f, pt4)` fixed point (weak, and only nonzero for the equilibrium gas,
whose dissociation reads `pt4`): `pt4 = pi_b * pi_HPC * pi_LPC * pt2` needs *both* compressor
ratios, and the burner's `f`-solve reads `(Tt3, pt4, Tt4)`. So across outer-loop iterations
(not within one cascade evaluation) the two spools do feed back on each other through `f` —
resolved by the SAME outer fixed-point loop rung 31 already runs.

---

## The finding is a NO-MAP MODEL ARTIFACT, not a physical law

This triangularization is not a surprising fact about real twin-spool engines — it is a
direct consequence of the SAME simplification rung 31 already made (isentropic efficiency +
choked-throat geometry, no compressor flow-capacity map) applied to a second spool. A real
compressor's flow capacity is a function of *both* corrected speed and pressure ratio (that
is exactly what rung 32's map added on top of rung 31); once that dependence is present, the
HP spool's mass flow constrains what the LP spool can deliver at the interface, and the two
spools' operating points DO need a joint (usually iterative, `N_L`/`N_H`) solve — the standard
textbook twin-spool matching procedure iterates for exactly this reason. **This rung's honest
scope is therefore the same rung-31-before-rung-32 shape**: the triangular cascade is real
*for this idealization*, and "two-spool + component maps" is the deferred seam that would
very likely reintroduce the coupling — the rung-31→32 correction, replayed on a second shaft.
That parallel, not a claim about physical twin-spool engines in general, is the honest home
for the finding.

---

## Scope: the fully-choked branch only — nozzle-unchoke is OUT OF SCOPE (a future rung)

Rung 33 needed a whole separate matching mode because when the SINGLE nozzle unchokes, its
throat stops passing a pressure-independent sonic mass flow and `pi_t` becomes the equilibrating
unknown. The same failure mode threatens `(★-LP)` here: if the nozzle unchokes, `tau_LPT` stops
being geometric and becomes the unknown that matches the LPT-NGV-choked supply to the
subsonic-nozzle demand — precisely rung 33's inversion, **relocated one throat upstream** (onto
the LP spool instead of directly onto the single spool). This is a genuinely different solve,
not a free reuse of `_match_subsonic` (which is built for one turbine, not two chained
choke-pins). Building it is a natural rung-33-shaped follow-on to this one, deliberately **not**
attempted here. `TwoSpoolMatcher.match` therefore asserts the nozzle stayed choked and raises a
clear, scope-flagging error otherwise — "flag, don't lie," the rung-31 ethos, rather than
returning numbers off a solve that no longer applies.

---

## Reduce-to-prior contract (the spine)

**Exact dispatch, not a knob-to-zero (the rung-37 pattern).** Setting `pi_LPC=1` and giving the
LPT zero specific work does **not**, by itself, reproduce rung 31: the LPT NGV throat `A45`
would still be *present*, and its geometric choke-pin `(★-LP)` would generically NOT land on
exactly `tau_LPT=1` just because the LPC does no work — the degenerate knob and the geometry
constraint are not automatically consistent. So the reduce cannot be a limit; it has to be a
literal absence of the LP hardware: `TwoSpoolMatcher(design_engine=<a plain single-spool
`Engine` from `build_turbojet`>, ..., lp_disabled=True)` never constructs an LPC, an LPT, or an
`A45` at all — it builds a bare `OffDesignMatcher` around the supplied single-spool design and
forwards every `.match()` call to it, verbatim. The two-spool-specific code path (the cascade
above) is never even entered. This is the same "exact dispatch when the extra state is never
built" contract rung 37 used for its plenum/heat-soak clocks.

- **`lp_disabled=True` ⇒ `OffDesignMatcher.match` bit-for-bit.** Same object, same call — a
  degenerate `TwoSpoolMatcher` IS a rung-31 `OffDesignMatcher` by construction, not by limit.
- **Cycle untouched ⇒ rung 6 bit-for-bit.** A new, separate entry point
  (`build_two_spool_turbojet` / `TwoSpoolEngine` / `TwoSpoolMatcher`); the default
  `build_turbojet(...).run(...)` design path is not perturbed, `Engine.run`'s single-shaft-balance
  logic is not touched.

---

## Verification gates (`tests/test_rung38.py`)

1. **REDUCE — `lp_disabled=True` is `OffDesignMatcher` by construction.** A `TwoSpoolMatcher`
   built with `lp_disabled=True` around a plain single-spool design delegates every `.match()`
   call to an internally-held `OffDesignMatcher` — same numbers, because it is the same object,
   not a converged limit.
2. **NON-TAUTOLOGICAL — an independent CPG closed-form solve of the cascade.** On a
   self-consistent CPG gas, a hand-written closed-form solve of `(★-HP)`, `(★-LP)`, and the two
   triangular shaft balances (no `_sonic_throat`, no `Compressor`/`Turbine.apply`, no
   `TwoSpoolMatcher`) reproduces the shipped solver's `(pi_LPC, pi_HPC, tau_HPT, tau_LPT)` to
   machine zero — two code paths, one operating point (the rung-31-gate-2 / rung-33-gate-4
   anchor, replayed for the two-spool cascade). `tau_HPT`/`tau_LPT` are also directly asserted
   `Tt4`-independent on CPG, and — the mirror, rung-31 gate 5 doubled — DO drift on the reacting
   gas over the same choked window, isolating the CPG constancy as the gas-model effect.
3. **THE FINDING — no 2×2 solve; compressor efficiency is a terminal leaf.** At a fixed
   `(Tt2, Tt4, f)`, perturbing `eta_HPC` leaves the solved `pi_LPC` **bit-for-bit unchanged**
   (Step 3 never reads it), and perturbing `eta_LPC` leaves the solved `pi_HPC` **bit-for-bit
   unchanged** (Step 4 never reads it) — each compressor efficiency is a dead end for the
   OTHER spool's pressure ratio. Contrast, in the SAME test: perturbing `eta_HPT` or `eta_LPT`
   (turbine/energy-path parameters) moves **BOTH** `pi_LPC` and `pi_HPC` (they shape the
   shared `Tt45`/`Tt5` energy cascade) — the gate asserts this too, so "no 2×2 solve between
   the compressor ratios" is not misread as "the two spools don't talk."
4. **SCOPE GUARD — nozzle unchoke raises, does not silently mis-solve.** Throttling to a point
   where the nozzle would unchoke raises the documented scope error rather than returning a
   number off the (no-longer-valid) `(★-LP)` relation.
5. **PHYSICALITY / DIRECTION.** `pi_LPC>1`, `pi_HPC>1`, `0<tau_HPT<1`, `0<tau_LPT<1`,
   `pt4 > pt25 > pt2` — the rung-31 sanity contract, doubled.
6. **CYCLE UNTOUCHED.** The default design run is bit-for-bit rung 6; constructing a
   `TwoSpoolEngine`/`TwoSpoolMatcher` does not perturb it.

---

## Concessions

- **Both NGVs assumed choked, not derived station-by-station.** Same status as rung 31's
  single-NGV assumption — an engineering fact about turbine nozzle pressure ratios, not a
  from-first-principles proof that `A4` and `A45` never unchoke. The nozzle, which DOES see the
  full ambient-pressure swing, gets the real rung-30/33 treatment; the inter-turbine duct does
  not.
- **Nozzle-unchoke is OUT OF SCOPE, not solved.** A rung-33-shaped follow-on (the LP spool's
  own subsonic branch) is the natural next seam; this rung deliberately does not attempt it.
- **No compressor/turbine MAPS.** Isentropic efficiency knobs only (rung 31 parity); "two-spool
  + maps" — the natural rung-32 analogue — is the deferred seam the finding's honesty section
  names explicitly, and is expected to reintroduce genuine spool coupling.
- **One `eta_m` for both shafts.** A single mechanical-efficiency knob is reused for both the LP
  and HP shaft balances (no new loss parameter); a more detailed model could split them.
- **No bypass, no bleed, no interstage duct loss, no reheat.** Plain series turbojet only — a
  fan/bypass split is a different engine (out of scope, flagged in the working contract).
- **Turbojet-only, single design point per spool split.** The design-point `pi_LPC`/`pi_HPC`
  split is a free design choice this rung does not optimize; it is specified, like rung 1–6's
  `pi_c`.
- **Steady only — no two-shaft transient.** The dynamic analogue (two shaft-inertia ODEs, the
  natural two-spool version of rung 34) needs this steady matcher to exist first (its
  equilibrium reduce gate is "== this rung's match"), and is deferred.
- **Structural, on a separate entry point.** Like rungs 31–35, this solves a NEW operating point
  (`π_LPC`, `π_HPC` are OUTPUTS) rather than reading the existing design run; it does so through
  `build_two_spool_turbojet`/`TwoSpoolEngine`/`TwoSpoolMatcher`, so the production
  `build_turbojet(...).run(...)` design path is untouched.

---

## Anchor

`docs/plans/rung38-anchor-two-spool-matching.md`. The **method** is the same mass-flow-
compatibility mechanism rung 31 anchors to Mattingly Ch. 8 / Cohen-Rogers-Saravanamuttoo's
choked-nozzle-guide-vane matching, applied a second time to the inter-turbine duct — a standard
gas-dynamics fact (a choked throat passes a pressure-independent corrected mass flow) chained
twice, not a claim attributed to a specific twin-spool textbook procedure (those are written
around compressor MAPS and therefore iterate; see "The finding is a no-map model artifact"
above for why this rung's triangular result does not contradict that literature). The
**non-tautological gate** (an independent CPG closed-form solve of the whole cascade) is the
rigorous anchor, exactly as it was for rungs 31/33.
