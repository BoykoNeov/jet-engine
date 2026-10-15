# Rung 87 — the CONTROLLED valve beside the stator schedule: what should it watch?

**Code:** the device in `rust/src/limited_bleed.rs` (`ValveReference`, `BleedLimiter::on_incidence`,
`target`, `r64_solve_b_at`); the reading in `rust/src/valve_reference.rs` · **Gates:**
`rust/tests/rung87.rs` · **Panel:** `print_valve_reference_table` (born in Rust, held to its own capture
`rust/oracle/rust_owned/`, a change detector only) · **Anchor (givens, derivations, every pre-registered
bar, committed `9f45f4e` before any was measured):** `docs/plans/rung87-anchor-valve-reference.md` — §
refs below are to it · **Sandbox:** the Controls view's bleed valve, "opens on its own"
(`docs/plans/sandbox-plan.md` § 16).

**Shipped as KNOBS** (the sandbox direction): every result is shown; predictions were registered first
and are scored here, a miss being a finding.

Rung 86 closed by naming this seam: *"A **φ-referenced bleed limiter beside a stator schedule and a
fuel leg**"*. Rungs 64–80 put rung 64's valve beside fuel legs and beside a stator LIMITER (68, 69),
never beside the stator SCHEDULE.

> **THE HEADLINE — a controlled valve beside a wall-moving lever must watch the wall, or it pays to
> undo the lever.** The LP stator schedule lowers the flow coefficient on purpose while it raises the
> margin to the metal. A valve holding the FLOW COEFFICIENT reads that as danger: it bleeds to buy the
> flow coefficient back, runs out of opening sooner (its ceiling falls 0.042–0.053), and — matched to
> the same incidence protection — still spends 1.27–3.50× the air and more thrust, because a constant
> flow coefficient over-protects early in the ramp (stators most closed) to be safe late (stators
> reopened). The same valve holding the INCIDENCE lets the stators carry part of its load (it bleeds
> less with them than without, where the flow-coefficient valve bleeds more). **And the reference must
> match the fuel side's:** an incidence valve beside a flow-coefficient FUEL floor lets the flow
> coefficient fall to `φ_lim/(1+v·φ_lim)`, the floor wakes, and the run starves — in every scored cell,
> 6 of them before the first point.
>
> **What is construction and what is not (a side review, accepted).** The incidence valve opens, point
> by point, exactly as far as its protection needs, so "the `φ` valve costs more at equal protection"
> (H1/H2) is nearly implied by the device (anchor D3: *a prediction, not a theorem*). Its content is the
> SIZE — 1.27–3.50× on the scored cells, largest at low set points where the valve only works early with
> the stators most closed — and that the one channel that could reverse it (bleed slows the spool, which
> moves `v`) never did: the smallest ratio is 1.27. That channel shows instead in the H4 MISS: a valve
> that opens late saturates BELOW the full-open march's ceiling. H5 is the result least implied by the
> construction.

## 1. What it adds

**The device (anchor § 1).** Rung 64's law with ONE thing changed — the wall it watches: hold
`T_c − 1/φ + v ≥ m_lim`, `m_lim = T_c − 1/φ_lim`, i.e. `φ ≥ φ_lim/(1 + v·φ_lim)`. `T_c` cancels; `v`
is a function of the shaft state alone, so inside one outer solve the target is a constant and rung 64's
monotone bracket stands. The reference lives ON the `BleedLimiter` (default `φ`) so every sibling
constructor that keeps the floor keeps its reference; a lagged incidence valve and an incidence valve
reaching `r64_solve_b` (the spelling rungs 65–80 call) are REFUSED, because those readers take
`phi_lim` as a flow coefficient. The `φ` valve's three cells call `r64_solve_b_at(lim, lim.phi_lim, …)`
and read no `v` — every rung-64…80 path is unchanged to the operation.

**The reading (`valve_reference.rs`).** One run in rung 87's currencies — the LP incidence margin
`M_i = T_c − 1/φ_LP + v` as the GRID minimum (a pinned plateau has nothing for a parabola to refine),
the bleed RE-SOLVED at every point (`b_at_point`; rung 64 § 5.22 (ii)), the thrust integral against the
SAME stator face's shut run — and `matched_phi`, Illinois over the `φ` valve's set point until its run
minimum `M_i` equals a given one.

**G1, the sibling trap, fifth instance — live and SILENT.** Rung 62's builder (rung 86's rig and, until
now, the sandbox's only one) DROPS a floor: `at_lever(floored(..))` returns `lever.lim = None`,
`armed_bleed() = false`, no panic. Every rung-87 machine is built by rung 64's, `sibling` asserts what it
built, and `tests/rung87.rs` records the trap.

## 2. Reduces

With the stators off the incidence valve IS the `φ` valve, bit for bit, every point and every re-solved
opening (`φ_lim/(1 + 0·φ_lim)` is `φ_lim` exactly — the target is spelled that way for it), on a riding
and a saturating cell. A `φ`-referenced valve takes every rung-64 path unchanged: no oracle, no
`rung64`–`rung86` gate and no CLI golden moved. The default design run stays bit-for-bit rung 6.

## 3. Scoring (§ 3 — bars frozen at `9f45f4e`)

Shaped and tilted maps × `r` ∈ {0.25, 0.5, 1.0} × `sm` ∈ {0.30 … 0.58}, both stator faces, `ds` 0.005
and 0.0025.

| # | verdict | measured |
|---|---|---|
| D1 (no credit) | arithmetic | each valve pins its own coordinate to ≤ 3.3e−15 while riding; the `φ` valve's incidence is `m_lim + v` |
| D2 (no credit) | arithmetic | the dormancy ledger: a set point at or below the shut run's own minimum never opens the valve; the run IS the shut run, bit for bit |
| D4 (no credit) | arithmetic | an incidence fuel floor beside the `φ` valve at the same `sm`, stators on, withholds exactly 0 while the valve is not saturated |
| H1 | **HIT**, 19 of 19 riding cells, both steps — *mostly by construction* | at equal incidence protection the `φ` valve bleeds 1.27–3.50× more; ratios move ≤ 1.3 % between steps |
| H2 | **HIT**, 19 of 19, both steps | and loses more thrust integral (table) |
| H1/H2's 20th cell | not scorable | shaped `r` 0.25, `sm` 0.46: the incidence valve SATURATED there (H4's miss), so there is no protection to match |
| H3 | **HIT**, 11 of 11 | beside the stators the incidence valve bleeds LESS than without them; the `φ` valve, MORE (G3) |
| H4 (the iff) | **MISS**, 29 of 30 cells (60 readings) | shaped `r` 0.25, `sm` 0.46, stators on: D2 put `m_lim` 0.5729 under the full-open 0.5818 (outside the band) and predicted R; the closed loop saturates and holds 0.5724, at both steps |
| H5 | **HIT**, 20 of 20 | incidence valve beside a `φ` fuel floor at the same `sm`, stators on: fuel cut in every cell and the thrust integral far below the matched pair's; 14 runs stop early (8–277 points of 291–441), 6 never start (the floor refuses the first point) |

**H5's matched pair, disclosed after measuring.** The anchor called the incidence valve beside an
incidence fuel floor at the SAME `sm` "dormant by D4"; D4 is about a floor STRICTLY below. At equal set
points the two hold one pin and share it in a split the solver decides (rung 66's degeneracy; G4 saw it
on the `φ` pair): the floor withheld 0 to 1.5e−2 there, and the sandbox's browser build and native model
— identical up to last-place rounding — landed 1e−4 apart in shaft speed on that case. H5's verdict
(starved vs ran to the end) does not depend on the split; its thrust numbers at equal set points are
quoted only as a sign.

**Equal-protection table** (`ds` 0.005 unless marked; `b_int` = time integral of the opening; `ΔF_int`
against the same stator face's shut run; "(S)" = that cell saturates; ¹ = within D2's 0.005 band, shown
but not scored):

| maps | `r` | `sm` | inc. valve `b_int`, stators on | same, stators off | matched `φ` valve's `sm*` | its `b_int` | ratio (`ds` 0.005 / 0.0025) | `ΔF_int` inc. / `φ*` |
|---|---|---|---|---|---|---|---|---|
| shaped | 0.25 | 0.34 | 7.922e-04 | 5.983e-03 | 0.2609 | 2.752e-03 | 3.474 / 3.476 | -0.067 / -0.225 |
| shaped | 0.25 | 0.38 | 6.445e-03 | 1.462e-02 | 0.3038 | 1.180e-02 | 1.830 / 1.830 | -0.545 / -0.980 |
| shaped | 0.25 | 0.42 | 1.482e-02 | 2.541e-02 (S) | 0.3464 | 2.378e-02 | 1.605 / 1.615 | -1.271 / -2.553 |
| shaped | 0.5 | 0.38 ¹ | 3.222e-04 | 9.295e-03 | 0.2876 | 2.111e-03 | 6.552 / 6.774 | -0.026 / -0.165 |
| shaped | 0.5 | 0.42 | 9.485e-03 | 2.421e-02 | 0.3447 | 2.160e-02 | 2.277 / 2.277 | -0.784 / -2.378 |
| shaped | 0.5 | 0.46 | 2.390e-02 | 4.053e-02 | 0.3896 | 4.102e-02 | 1.716 / 1.722 | -2.147 / -5.400 |
| shaped | 0.5 | 0.50 | 3.957e-02 | 5.265e-02 (S) | 0.4344 | 5.142e-02 | 1.300 / 1.301 | -4.489 / -7.598 |
| shaped | 1 | 0.42 | 2.210e-03 | 1.957e-02 | 0.3223 | 7.638e-03 | 3.456 / 3.501 | -0.169 / -0.571 |
| shaped | 1 | 0.46 | 1.824e-02 | 4.115e-02 | 0.3753 | 3.540e-02 | 1.941 / 1.941 | -1.616 / -5.021 |
| shaped | 1 | 0.50 | 3.929e-02 | 6.553e-02 (S) | 0.4265 | 6.435e-02 | 1.638 / 1.641 | -4.735 / -9.524 |
| shaped | 1 | 0.54 | 6.334e-02 | 8.231e-02 (S) | 0.4798 | 8.021e-02 | 1.266 / 1.268 | -8.102 / -11.674 |
| tilted | 0.25 | 0.34 | 8.877e-04 | 6.383e-03 | 0.2574 | 2.761e-03 | 3.111 / 3.116 | -0.078 / -0.236 |
| tilted | 0.25 | 0.38 | 6.567e-03 | 1.504e-02 | 0.2993 | 1.155e-02 | 1.758 / 1.778 | -0.580 / -0.999 |
| tilted | 0.25 | 0.42 | 1.491e-02 | 2.538e-02 (S) | 0.3424 | 2.359e-02 | 1.582 / 1.583 | -1.330 / -2.307 |
| tilted | 0.5 | 0.42 | 8.058e-03 | 2.308e-02 | 0.3391 | 1.907e-02 | 2.367 / 2.367 | -0.687 / -1.785 |
| tilted | 0.5 | 0.46 | 2.228e-02 | 3.990e-02 | 0.3848 | 3.963e-02 | 1.779 / 1.786 | -1.936 / -5.002 |
| tilted | 0.5 | 0.50 | 3.849e-02 | 5.190e-02 (S) | 0.4289 | 5.057e-02 | 1.314 / 1.316 | -4.173 / -7.247 |
| tilted | 1 | 0.42 ¹ | 4.058e-04 | 1.622e-02 | 0.3126 | 2.633e-03 | 6.488 / 6.486 | -0.032 / -0.201 |
| tilted | 1 | 0.46 | 1.429e-02 | 3.794e-02 | 0.3683 | 2.996e-02 | 2.097 / 2.097 | -1.143 / -4.051 |
| tilted | 1 | 0.50 | 3.546e-02 | 6.269e-02 | 0.4191 | 6.056e-02 | 1.708 / 1.712 | -4.064 / -8.782 |
| tilted | 1 | 0.54 | 5.977e-02 | 8.045e-02 (S) | 0.4728 | 7.770e-02 | 1.300 / 1.300 | -7.535 / -11.473 |

The matched `φ` valve's set point sits BELOW the incidence one's name in every cell (`sm*` < `sm`): it
needs less flow coefficient at its binding point, which is LATE (the matched run's `M_i` minimum sits at
`s` 0.25–0.95, where the stators have reopened), and still bleeds more because it holds that flow
coefficient through the early stretch where `v` is large.

## 4. The findings, in order of how much the construction already implied them

1. **Mismatched references starve the engine** (H5). An incidence valve and a flow-coefficient fuel
   floor disagree about the danger whenever the stators are closed; the valve stands down, the floor
   cuts the fuel. Not implied by either device alone.
2. **A late-opening valve's ceiling sits below the full-open bound** (H4's MISS). Rung 64 defined the
   ceiling as the full-open march's minimum; a closed loop that opens only when needed has given up the
   bleed's earlier effect on the spool, so on one cell it saturates 0.009 below that bound in `M_i`.
3. **The stators pay part of the incidence valve's bill and add to the `φ` valve's** (H3 + G3) — rung
   86 C3's fuel-floor finding, one lever over.
4. **At equal protection the `φ` valve costs 1.27–3.50× the air** (H1/H2) — mostly by construction; its
   content is the size and that the spool-speed channel never reversed it.

## What it does to its neighbours

* **Rung 64 — its ceiling BOUNDED.** "The ceiling is `min φ` over the fully-open march" is a bound, not
  the closed loop's ceiling: a valve that opens late can saturate below it (H4). And beside a
  wall-moving lever the ceiling is coordinate-dependent: the stators LOWER it in `φ` and RAISE it in
  incidence (G2).
* **Rung 60 — reproduced one lever over.** A valve pins its coordinate; re-referencing moves the pin
  (D1). Inherited, no credit.
* **Rung 66 — its degeneracy met on the page.** A valve and a fuel floor on the same coordinate at the
  same set point share one pin by rounding (H5's disclosure, the sandbox's 1e−4).
* **Rung 86 — its seam CLOSED, its C3 EXTENDED** from the fuel floors to the valve; its knob finding
  (the `φ` floor fights the stators) now has a second victim — the `φ` valve.

## Concessions

* ONE placement (`n_lo` = 0.65), the LP currency only, rung 36's imposed `φ_surge`, invented maps, the
  valve on the LP only — inherited from rungs 57–64.
* The incidence valve is INSTANTANEOUS; a lagged one (rung 65's bandwidth) is refused, not built.
* D2's 0.005 band was chosen knowing G3's one closed-loop violation of a full-open ceiling (disclosed
  in the anchor); H4's miss is outside it.
* The fuel leg was scored only as the two stall floors at the valve's own `sm` (H5) and, in G4, the `φ`
  pair; the acceleration schedule and the turbine limiter beside the valve run in the sandbox (replayed
  bit for bit, crash-mapped) but carry no rung-87 claim.

## The next seam

A **lagged incidence valve** (rung 65's bandwidth on a MOVING target — does the lag's cost depend on
which wall it tracks?), and the **station-3 customer bleed** — a different sink into the same two
sensed inputs (rung 86's, still open).
