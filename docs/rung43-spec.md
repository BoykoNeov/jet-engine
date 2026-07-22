# Rung 43 — Two-shaft fuel metering: the two spools sit at DIFFERENT points in ONE overshoot loop

**Scope.** `TwoSpoolFuelTransient` (subclasses rung 40's `TwoSpoolTransient`) —
rung 35's control on rung 40's plant. Fuel (`ṁ_fuel`) is metered; `Tt4` becomes an
**OUTPUT** floating against the airflow two lagging spools can currently pump.

Rung 35 closed rung 34's `Tt4(t)`-by-fiat concession on **one** shaft. Rungs 39/40 built
the two-shaft plant but kept rung 34's `Tt4` command; rung 40 listed "**fuel metering** on
two shafts (rung 35's control, not carried over)" as an open seam. Rung 43 carries it over.

---

## Why this is not rung 35 replayed

Rung 35's finding — a fuel step at a frozen spool starves the airflow, so `Tt4`
**overshoots** (a TIT excursion) and that over-temperature amplifies the surge excursion —
**re-measures unchanged here** and is labelled **INHERITED** throughout. It is not this
rung's finding.

The two-shaft content is a question rung 35 **structurally could not ask**. The overshoot
loop is:

```
    f    = ṁ_fuel / ṁ_air          <-- the LP FACE sets the airflow  (LP lag spikes f)
    Tt4  = burner(Tt3, f)                                             (Tt4 floats up)
    ṁ4   = A4 · pt4 · MFP*(Tt4) / √Tt4   <-- the HP-FED NGV CHOKE meters it back
```

`ṁ_air` is the **LP-face** corrected flow (`m_L · mcorr_lp_d · pt2/√Tt2`), but the `Tt4` it
produces is metered through the **HP-fed** NGV throat (`pt4 = π_b·π_HPC·π_LPC·pt2`). The two
spools therefore sit at **different points in the one loop**. With one shaft there is one
clock and no question; with two there is a clock **ratio** `ρ = τ_L/τ_H` (rung 40's one
surviving parameter) and the question becomes: **which spool's lag governs the overshoot?**

**The answer is neither**, and *why* is the rung.

---

## The closure

`_close_fuel(ν_L, ν_H, ṁ_fuel, Tt2, pt2)` is rung 40's `_close` with the burner run
**FORWARD** (rung 35's `_tt4_from_f`, the exact inverse of the shipped `f`-solve): still
**ONE root in `m_L`**, still **NO shaft balance** (both power residuals stay OUTPUTS, which
is what makes them the two ODE right-hand sides). The only change is that `f` and `Tt4`
become outputs of the trial flow instead of `f` being solved against a commanded `Tt4`.

### A bracket that rung 40 did not need

Rung 40's `_close` brackets `m_L` on `[·, min(2.5, φ_max·n_L)]`. That wall is safe **only
because `Tt4` is pinned**. With `Tt4` floating, far past the root the mixture goes lean, the
HP map leaves its physical branch (`π_HPC` → 0.01 at `φ_L ≈ 2.0`) and the sonic-throat solve
fails — a wall-to-wall bracket can straddle nonsense. `_close_fuel` therefore scans **up from
the rich wall and takes the FIRST sign change** (`g` rises monotonically through the physical
root). This is a small structural consequence of the control change, not a bug fix in rung 40:
**rung 40's `_close` is left LITERALLY unchanged** (the rung-33/39/40 discipline), so the
rung-40 suite still witnesses it bit-for-bit.

---

## THE FINDING — both spools are in the loop, and both relieve it

**Channel isolation** (rung 41's `surge_margin_channels` move, applied to the transient):
march the fuel ramp with one spool's speed **held** at its initial value. `Tt4_peak` [K],
shape `flow/press`, `Tt4` 1250 → 1450:

| ρ | r | both free | LP frozen | HP frozen | ΔLP | ΔHP |
|---|---|---|---|---|---|---|
| 0.5 | 0.25 | 1591.2 | 1643.3 | 1627.7 | **+52.1** | +36.5 |
| 1.0 | 0.25 | 1611.9 | 1643.3 | 1651.4 | +31.3 | **+39.4** |
| 2.0 | 0.25 | 1625.9 | 1643.3 | 1667.3 | +17.3 | **+41.4** |

Same on the other two shape pairs (`press/flow` +53.3/+38.2 → +17.8/+42.6; `tilted`
+53.1/+38.4 → +17.7/+43.3 over the same `ρ`), with the crossover in the same place.

1. **Freezing EITHER spool makes the overshoot WORSE — every sampled point.** Both spools'
   motion relieves it. Neither is a bystander.
2. **The share of the relief TRADES with `ρ`**: as the LP spool slows, the LP channel's
   relief shrinks (+52 → +31 → +17) and the HP channel's grows (+37 → +39 → +41), the two
   crossing between `ρ` = 0.5 and 1 on every shape. **Direction only** — `ΔLP` and `ΔHP` do
   not sum to the total and are **not** calibrated weights.

*That* is why no single spool's clock can govern the overshoot: the responsibility for
quenching it is **shared and `ρ`-dependent**.

### The positive, and its CEILING

At fixed `r`, the overshoot **rises monotonically with `ρ`** — a heavier LP spool worsens
the TIT excursion, because the LP-face airflow lag is what spikes `f`.
`X = Tt4_peak − Tt4_target` over `ρ` = 0.25 → 8, **6/6 monotone rising** (3 shape pairs × 2
ramp durations):

| shape | r | X(ρ = 0.25, 0.5, 1, 2, 4, 8) |
|---|---|---|
| flow/press | 0.25 | 117.3 141.2 161.9 175.9 184.1 188.6 |
| flow/press | 1.00 | 52.3 60.1 74.8 93.3 109.2 119.9 |
| press/flow | 0.25 | 121.0 145.2 166.3 180.7 189.1 193.7 |
| press/flow | 1.00 | 52.7 61.0 76.5 95.9 112.6 123.8 |
| tilted | 0.25 | 119.8 144.0 165.1 179.4 187.8 192.3 |
| tilted | 1.00 | 52.7 60.7 75.8 94.7 111.0 121.9 |

It is **BOUNDED**, and the bound is structural: `ρ` multiplies **only** the LP ODE
(`dν_L/ds = Φ_L/ρ`), so **`ρ → ∞` IS the LP-frozen system**. The LP-frozen march is
therefore `ρ`-independent **bit-for-bit**, and `X(ρ)` converges upward onto it:

| shape | r | X(ρ = 8, 32, 128) | LP-frozen ceiling | `ρ`-free |
|---|---|---|---|---|
| flow/press | 0.25 | 188.59 → 192.08 → 192.98 | **193.28** | `==` |
| flow/press | 1.00 | 119.85 → 129.39 → 132.02 | **132.92** | `==` |
| press/flow | 0.25 | 193.67 → 197.28 → 198.20 | **198.51** | `==` |
| press/flow | 1.00 | 123.83 → 133.93 → 136.72 | **137.68** | `==` |
| tilted | 0.25 | 192.32 → 195.89 → 196.81 | **197.12** | `==` |
| tilted | 1.00 | 121.94 → 131.75 → 134.45 | **135.38** | `==` |

(`==` verified over `ρ` = 0.25, 1, 7, 50 — the ceiling is **bit-identical**, e.g.
193.27628904725066 at every `ρ`, not merely close.)

So the monotone rise is not open-ended: **the worst TIT excursion a heavy LP spool can
produce is the LP-frozen march**, computable without marching the LP spool at all.

---

## THE NEGATIVE, stated plainly — there is NO effective clock ratio

The obvious way to make this one-dimensional again is an **effective ramp ratio**
`r_eff = r/ρ^q` (`q=0` ⇒ the HP clock, `q=1` ⇒ the LP clock / "the slow spool
rate-limits"). **It does not exist**, and the reason it *appeared* to exist is a trap worth
recording.

**The currencies are CIRCULAR.** The fitted exponent reads back whichever spool sits in the
denominator of the excursion's reference:

| currency | denominator | best `q` (flow/press · press/flow · tilted) |
|---|---|---|
| `E_temp_H` | ν_H running line | **0.05 · 0.05 · 0.05** |
| `X = Tt4_peak − Tt4_target` | none (spool-neutral) | **0.35 · 0.45 · 0.35** |
| `E_temp_L` | ν_L running line | **0.65 · 0.45 · 0.65** |

The HP-referenced currency reads **0.05 on every shape** while the neutral and LP-referenced
ones sit far above it. So `E_temp`'s `q ≈ 0` was **never** evidence that "the HP clock
governs"; it was the reference reading itself back. Only `X` is spool-neutral, which is why
every magnitude above is quoted in `X` — **the data selected the instrument, not the answer
it gave.**

**Stated at its true strength.** The ordering is `q*(E_temp_H) < q*(X) ≤ q*(E_temp_L)` —
strict on two shapes and a **TIE on `press/flow`** (0.45 = 0.45). An earlier draft of this
spec claimed a strict three-way monotone across all three shapes; **the shipped measurement
refuted it** and it is corrected here. What survives — and is gated — is the HP-referenced
currency reading low against a spool-neutral one, which is all the circularity argument needs.

And even on `X` there is **no collapse**. The best exponent does cut the spread ~4.9×
against the `q=0` endpoint, but it bottoms out at a **~14–15% residual**: points that a
real effective clock would place on one curve still differ by about a seventh. (No
project-wide numerical threshold is invoked for what counts as a collapse — the claim
rests on the residual being large in absolute terms *and* on the exponent that achieves
it being currency-dependent, which a genuine one-parameter reduction could not be.)
The only exponent statement made is that the
optimum on `X` is **INTERIOR** (`0 < q* < 1`) — matching neither single-spool clock, and
fitting **3.4×–4.9× better** than either (residuals on the shipped `collapse_exponent`
metric):

| shape | res @ `q=0` (HP clock) | `q*` | res @ `q*` | res @ `q=1` (LP clock) |
|---|---|---|---|---|
| flow/press | 0.689 | 0.35 | **0.142** | 0.512 |
| press/flow | 0.705 | 0.45 | **0.149** | 0.513 |
| tilted | 0.696 | 0.35 | **0.147** | 0.516 |

It is explicitly **not** a refutation of `q=1`: on `X`, `q=0` fits **substantially worse**
than `q=1` on all three shapes (0.689 vs 0.512, 0.705 vs 0.513, 0.696 vs 0.516). If either
single-spool clock were to be preferred it would be the **LP** one — "the slow spool
rate-limits" — the *opposite* of what the HP-referenced currency's `q ≈ 0` appeared to say.
Neither is close to the interior optimum, and **none of the three collapses**.

**Deliberately NOT claimed** (each was written, probed, and withdrawn):

* ~~"the overshoot rides on the geometric-mean composite clock `√det ∝ ρ^(−1/2)`"~~ —
  **DROPPED.** `√det·√ρ` = const to machine precision is a true **rung-40** Jacobian
  identity, but it is **not connected to the overshoot**. `q*(X)=0.35` is almost exactly the
  **midpoint** of the two circular currencies (0.05, 0.65) — an averaging-artifact
  signature, not evidence for 1/2. No effective clock is claimed and `√det` is not this
  rung's anchor.
* ~~"`q=1` (the slow spool rate-limits) is refuted in every currency"~~ — **FALSE.** On `X`,
  `q=0` (0.689) is *substantially worse* than `q=1` (0.512). Best-`q` ranges 0.05 → 0.35 →
  0.65 across the three currencies, so
  **nothing about the exponent is currency-independent.**
* ~~"the overshoot is irreducibly two-dimensional"~~ — **overclaim.** What is shown is that
  no **power-law** `r/ρ^q` collapses it; that no one-parameter family does is not shown.
  The honest statement: **rung 35's single-clock `r` framing does not extend to two shafts
  via any effective clock ratio.**
* ~~"fuel metering breaks rung 39's `(†)` cancellation and re-couples LP into the HP core"~~
  — **a category error, killed before any code.** `(†)` is a *steady* η-fixed-point
  artifact and does not arise in the transient closure at all (rung 40 established this);
  and control-invariance (gate 1) shows the fuel-metered and `Tt4`-metered steady points are
  **the same point**. Same manifold, different knob.

---

## Verification gates

1. **REDUCE — CONTROL-INVARIANCE (the non-tautological gate).** Feeding
   `ṁ_fuel = f_eq·ṁ_air,eq` of a rung-40 `Tt4`-control point to `equilibrium_fuel`
   reproduces that point — `ν_L`, `ν_H`, `Tt4`, `π_LPC`, `π_HPC` to machine zero (measured
   0, 4e-15, 2e-15) — through the **forward-burner closure**, a genuinely different code
   path. This is also the gate that kills the `(†)` framing empirically.
2. **REDUCE — `lp_disabled` EXACT DISPATCH** ⇒ rung 35's `SpoolTransient` fuel path
   **bit-for-bit** (`==`). No two-shaft state is built.
3. **REDUCE — `Tt4`-CONTROL UNTOUCHED** ⇒ rung 40's `equilibrium`/`integrate` bit-for-bit
   (`==`); rung 40's `_close` is literally unchanged, so the rung 31–42 suites pass
   unchanged.
4. **REDUCE — SETTLE.** A fuel ramp marched long lands **on** the target equilibrium
   (`Tt4` → 1450.0, `ν` → the matched pair).
5. **FINDING — THE MECHANISM.** Freezing either spool **worsens** the overshoot at every
   sampled `(ρ, r)` — sign/existence only, with the asserted CONTRAST that the *share*
   trades with `ρ` (LP channel weakens, HP channel strengthens as `ρ` rises). **No
   calibrated split is gated.**
6. **FINDING — THE CEILING.** The LP-frozen march is `ρ`-independent **bit-for-bit**
   (`ρ` multiplies only the LP ODE), and `X(ρ)` rises monotonically toward it.
7. **FINDING — `ρ`-MONOTONICITY.** `X` rises monotonically with `ρ` across ≥3 shape pairs ×
   2 ramp durations. **Sign only**; magnitudes disclaimed.
8. **INHERITED — TIT-limited before surge** (rung 35), re-measured on two shafts
   (`E_temp0` ≈ 0.35 vs `E_surge0` = 0.024–0.079). Labelled inherited, not a finding.
9. **THE WITHDRAWN CLAIMS, asserted as such** (rung 40's gate-7 move): the test asserts the
   best-fit `q` **DIFFERS** across the three currencies, so "the overshoot collapses on an
   effective clock" cannot silently creep back.
10. **CYCLE UNTOUCHED.** Default `build_turbojet(…).run(…)` is bit-for-bit rung 6.

### Why rung 43 carries no independent bare-math gate

Rungs 38/39/40 each ship a bare-math CPG cascade because their reduce never enters the new
code. Rung 43's **does**: control-invariance (gate 1) lands every steady fuel point
**exactly on rung 40's steady manifold**, which rung 40's own independent bare-math cascade
ties down — so the fuel closure is anchored **transitively**, and `lp_disabled` anchors the
degenerate path onto rung 35. The genuinely new content is the **transient** freeze/
monotonicity **signs**, which are shape-robust directions, not magnitudes a bare-math
replica would constrain. (Rung 42 set this precedent; the reason is on the record here for
the same reason it was there.)

### One change to a SHIPPED rung — a defect, found here and fixed here

Building this rung's reacting-gas scope gate surfaced a pre-existing bug in **rung 40's**
`equilibrium`, unrelated to fuel metering. Its convergence test is an **absolute**
`_EQ_TOL = 1e-12`, but the residual's noise floor is **gas-dependent**: ~1e-14 on CPG (fine)
against ~1e-10 on the **reacting** gas, where the equilibrium sub-solve inside `_close`
leaves that much scatter in `Φ`. So the Newton converged physically in ~5 iterations and
then spun on noise it could never get under:

```
reacting, Tt4 = 1400:  |Phi| = 6.4e-2 -> 2.5e-11 by iteration 4,
                       then FLAT at 1.08e-10 for 75 more -> AssertionError
```

It therefore **raised at `Tt4` = 1300 and 1400 while 1500/1450/1200 squeaked under** —
non-monotone in `Tt4`, which is the tell that it is a solver artifact and not physics. Rung
40's own suite happened to sample only 1500 and 1200 and never hit the hole.

The fix is a **best-so-far acceptance AFTER the loop** (accept the lowest-residual iterate if
it beat 1e-8). It is **bit-for-bit safe by construction**, not by testing: the primary
tolerance-return is untouched and runs first, so every input that already converged returns
at the *identical iteration* with *identical* `(ν_L, ν_H)`, and the new branch is reachable
**only** by inputs that previously raised. A stagnation exit *inside* the loop — the obvious
alternative — would not have this property, since it could fire mid-descent on a
currently-passing case. Rungs 40/41/42 pass unchanged as the witness.

This is the project's "stop and explain surprises" contract applied across a rung boundary:
the surprise belonged to rung 40, so it was diagnosed and fixed there rather than routed
around in rung 43.

---

## Concessions

* **Every magnitude is disclaimed** — the overshoot numbers ride on `ρ` (a disclaimed clock
  group, doubled, inherited from rung 40), on the two representative maps, on the fuel step,
  and on the `Tt4` band. Load-bearing: the freeze **signs**, the `ρ`-**monotonicity**, the
  **ceiling identity**, and the reduces.
* **No exponent, no effective clock, no `√det`** — see the negatives above.
* **The freeze is sign/existence.** `ΔLP` + `ΔHP` sums to nothing; the channel split is a
  direction, not a decomposition.
* **Reacting-gas fuel control deferred** (rung 35's concession, carried **verbatim**):
  `_tt4_from_f` is built for the non-equilibrium gas and **asserts** against an equilibrium
  one. The finding is gas-independent; the **reacting reduce is the `Tt4`-control path**
  (bit-for-bit rung 40).
* **No surge line on either spool** — so **no surge-survival claim**. Rung 41's transient
  two-spool surge line is still deferred, and rung 42's bleed still does not read the
  transient.
* **`Tt4` overshoot vs a redline** is not claimed — no TIT limit is modelled (rung 35's
  concession).
* **Fully-choked branch, both NGVs choked, one `η_m`, no bypass, isentropic knobs, no
  bleed** — all inherited from rungs 38–42.

---

## Anchor

`docs/plans/rung43-anchor-two-shaft-fuel.md` — the measured tables above.
