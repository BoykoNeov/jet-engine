# Rung 32 — Component-map matching: the map re-labels the choke-pinned work

Rung 31 built the off-design running line and closed with a slogan: the choked turbine NGV +
choked nozzle strip the compressor of freedom, so `π_c`, `ṁ` and thrust ride one fixed running
line — *"a pumping characteristic **without a compressor map**; the choked downstream hardware
**is** the map."* It got there by holding the component efficiencies `η_c, η_t` at their design
values along the whole line (its own stated concession — "the map curvature is rung 32").

Rung 32 puts a real **component map** on the compressor and turbine and asks what that
concession cost. The answer is a **cross-rung correction** — the rung-29 / rung-28 move:

> **Rung 31's "without a compressor map" over-claimed.** The choked hardware sets the compressor
> **work schedule** `τ_c(Tt4)` map-free (that part of rung 31 survives untouched). But converting
> that work into a **pressure ratio `π_c`, a mass flow `ṁ`, and a shaft speed `N` needs the real
> map.** Hold `η_c` constant and you misplace the running line: with a peaked (peak-η-near-design)
> compressor map, throttling walks you off the efficiency island, `η_c` droops, and `π_c`/`ṁ`
> fall **below** rung 31's constant-η line — a first-order shift, growing with throttle. The
> **turbine**, by contrast, barely moves on its map — but for a **structural** reason, not because
> turbine maps happen to be flat: on a single spool the turbine's corrected speed `N/√Tt4` stays
> within ~1% of design as you throttle (N and √Tt4 fall together), so the turbine sits at a nearly
> fixed map point *even if its map is steep*. Rung 31's "hold `η_t` constant" was nearly exact —
> and now we know why. **The compressor is where the map bites.**

Like every rung 7→31 this is a **diagnostic beside the cycle**: a separate entry point
(`MapMatcher`), the default `build_turbojet(…).run(…)` design path untouched (bit-for-bit rung 6),
and it **reduces to rung 31 bit-for-bit when the map is flat**.

---

## What is genuinely new: the shaft speed `N` enters

Rung 31 traced the entire running line — the `(ṁ, π_c)` locus — **without ever computing `N`**.
It never needed to: the two choke constraints pinned the turbine and the shaft handed back the
compressor by pure thermodynamics. A **map is different**: a compressor map is a surface
parametrized by **corrected speed** `N/√Tt2` and **corrected mass flow** `ṁ√Tt2/pt2`, so to read
`η_c` off it you must know where on it you are — and that requires `N`. Rung 32's structural
novelty is that **`N` is the coordinate the map forces into the problem.**

The running line does **not** move onto the map arbitrarily; the choke still pins `(ṁ, π_c)`. The
map's **speed lines** only *label* that pinned point with an `N`, and its **efficiency contours**
supply the `η` there — which then feeds weakly back into `π_c`.

---

## The two maps (representative analytic closures — shapes disclosed)

Component maps are empirical hardware data. Consistent with the project's no-data-file discipline
and with the mixing-closure precedent (rungs 12–24: parametric closures with **disclosed** shapes,
**load-bearing claims shape-robust, magnitudes disclaimed**), rung 32 uses **physically-grounded
analytic maps** and tests every load-bearing claim across **three shapes**. The *method* is
anchored (Cohen–Rogers–Saravanamuttoo / Mattingly off-design matching, same textbook family as the
rung-2/31 design anchor); the map *shape* is a modeling choice, and no claim that rides on it is
promoted past "disclaimed."

### Compressor speed lines (from Euler work + a loading law) — these give `N`

Euler turbomachinery work: `Δh_c = ψ·U²`, with `ψ` the stage **loading (work) coefficient** and
`U ∝ N` the blade speed (fixed geometry). With `Δh_c = c̄p·Tt2·(τ_c−1)` and corrected speed
`n ≡ (N/√Tt2)/(N/√Tt2)_design`, normalizing by the design point gives the **speed-line family**

```
(τ_c − 1)/(τ_c − 1)_d  =  ψ(φ)·n²,        φ ≡ (ṁ√Tt2/pt2)/(…)_d / n   (flow coefficient ∝ Ca/U)
ψ(φ)  =  1 − σ·(φ − 1)²                    # loading law; ψ(1)=1 at design; σ = disclosed shape
```

The choke pins `(τ_c, ṁ)` at each operating point (rung 31), so **inverting this one equation for
`n`** places the pinned point on its speed line — that is `N`. With `σ = 0` (flat loading) it
collapses to `n = √[(τ_c−1)/(τ_c−1)_d]`, a function of the choke-pinned work alone; the map's
content is the `σ ≠ 0` curvature that makes `N` genuinely map-dependent.

### Compressor efficiency island — this is what bites `π_c`

```
η_c  =  η_c,d − a·(φ − 1)² − b·(n − 1)² − c·(φ − 1)(n − 1)     # concentric-ellipse contours
```

Peak efficiency at the design point (`φ = n = 1`, the standard peak-at-design calibration), drooping
away. `(a, b, c) ≥ 0` are disclosed shape parameters. `η_c` is read at the operating point and enters
`π_c = [1 + η_c(τ_c − 1)]^(γc/(γc−1))` (the exact inverse of the shipped `Compressor.apply`), which is
the **only** place the compressor map bites the running line.

### Turbine map (choked ⇒ fixed corrected flow; index by corrected speed only)

The turbine NGV is choked, so its corrected mass flow is fixed (that *is* the choke). Its map is
therefore indexed by **corrected speed alone**, `n_t ≡ (N/√Tt4)/(…)_d = (N/N_d)·√(Tt4_d/Tt4)` on
the single shaft:

```
η_t  =  η_t,d − a_t·(n_t − 1)²         # turbine efficiency maps are notoriously FLAT (a_t small)
```

`η_t` enters the turbine choke solve (`_solve_turbine`) and hence `τ_t`, `τ_c`. Because `a_t` is
small (real turbine maps are flat near design), this feedback is tiny — the sub-finding.

**Flat map** `= {a=b=c=0, σ=0, a_t=0}` ⇒ `η_c ≡ η_c,d`, `η_t ≡ η_t,d` (rung 31 exactly), with `n` a
passive diagnostic. This is the reduce.

---

## The solve (extends rung 31; `η` becomes self-consistent with the map)

Rung 31's inner joint fixed point on `(f, pt4)` is unchanged and runs with **fixed** `η_c, η_t`. Around
it rung 32 wraps an **outer solve for the map-consistent efficiencies**: given `(η_c, η_t)`, the inner
loop produces the operating point `(π_c, τ_c, ṁ, n, φ, n_t)`; the map returns targets
`η_c^map(φ,n)`, `η_t^map(n_t)`; the outer solve drives `η = η^map(η)` to a fixed point. The `η_c`
feedback is **positive** (lower `η_c` → lower `π_c` → lower `φ,n` → lower `η_c`), so a plain
substitution can oscillate; rung 32 uses a **secant** iteration on `η_c` (with `η_t` — nearly
constant — substituted alongside), which is stable and quadratic. A non-convergence assert guards
the deep-throttle edge where the positive feedback is strongest.

`N` is then attached from `n` (corrected) and `N/N_d = n·√(Tt2/Tt2_d)` (physical, single shaft). No
absolute rpm is claimed — that needs blade geometry (disclaimed).

---

## Reduce-to-prior contract (the spine)

- **Flat map ⇒ rung 31 bit-for-bit.** With `{a=b=c=0, σ=0, a_t=0}` every `η` equals its design
  value, the outer solve is inert, and `MapMatcher.match` reproduces `OffDesignMatcher.match`'s
  `π_c`, `ṁ`, `τ_t`, stations and thrust to solver tolerance (observed machine-zero at design;
  ≤1e-9 across the sweep). `N` is an extra diagnostic output that does not perturb the reduce.
- **Cycle untouched ⇒ rung 6 bit-for-bit.** The default `build_turbojet(…).run(…)` design path is
  not touched; building a `MapMatcher` (which runs a convergent design internally) does not perturb
  the default run. The rungs-7+ invariant holds.

---

## The finding (shape-robust) and its disclaimed magnitudes

**Load-bearing (verified across three map shapes):**

1. **The work `τ_c` is map-free.** `τ_c(Tt4)` from the map matcher equals rung 31's to ~1e-4 across
   the throttle sweep — the choke pins the compressor work regardless of the map. Rung 31's work
   schedule survives intact.
2. **`π_c` and `ṁ` are NOT map-free — they droop.** With a peaked compressor map, `π_c` and `ṁ`
   fall **below** rung 31's constant-η line off-design, **same (negative) sign for every shape**,
   the gap growing monotonically with throttle. This is the first-order correction of rung 31's
   "without a map" slogan.
3. **The turbine barely moves on its map — structurally.** The turbine corrected speed `nu_t =
   (N/√Tt4)/(…)_d` stays within ~1% of design across a 2:1 throttle (single spool: N and √Tt4 fall
   together), so `|Δη_t|` is ~2e-5 **even for a deliberately steep turbine map** (`a_t` cranked
   25×), against `|Δη_c|` ~1e-2. Rung 31's "hold `η_t` constant" was nearly exact, and the reason
   is structural (the turbine is pinned in corrected speed), not that turbine maps happen to be
   flat. **The compressor is where the map bites.**
4. **`N` is a real new output.** Its leading throttle schedule (`N/N_d` falls with `Tt4`) is robust
   across speed-line shapes (~few-% spread over `σ ∈ [0,1]`); its absolute value is map-dependent.

**Disclaimed (shape-dependent — quoted only as illustrative):**
- The **magnitude** of the `π_c`/`ṁ` droop (rides on `(a,b,c)`).
- The absolute `N(Tt4)` schedule (rides on `σ`; only the *ratios* and the *falling* trend are robust).
- **No surge line, no surge-margin claim.** A representative efficiency island is not a surge
  boundary; the classic CRS surge-margin-along-the-running-line payoff is deliberately **not** made
  (it needs a real map's surge line — data this rung does not carry).

---

## Verification gates (`tests/test_rung32.py`)

1. **REDUCE TO RUNG 31 (the spine).** The flat map (`ComponentMap.flat()`) makes `MapMatcher.match`
   reproduce `OffDesignMatcher.match` — `π_c`, `ṁ`, `τ_t`, stations, thrust — across a throttle
   sweep (machine-zero at design; ≤1e-9 on the sweep). `N` present but inert.
2. **CYCLE UNTOUCHED.** The default design run is bit-for-bit rung 6; constructing a `MapMatcher`
   does not perturb it (the rungs-7+ invariant).
3. **THE FINDING — `π_c`/`ṁ` droop, SHAPE-ROBUST.** For each of ≥3 map shapes, off-design `π_c` and
   `ṁ` fall below the flat-map (rung-31) values, same sign, gap growing with throttle.
4. **WORK IS MAP-FREE.** `τ_c` from the map matcher equals rung 31's to ~1e-4 across the sweep — the
   choke-pinned work is unmoved by the map (isolates *what* the map moves: `π_c`/`ṁ`, not `τ_c`).
5. **TURBINE PINNED IN CORRECTED SPEED (sub-finding).** `nu_t` stays within ~1% of design across the
   sweep, so `|Δη_t| ≪ |Δη_c|` (orders) **even with a steep turbine map** (`a_t` cranked up) — the
   flatness is structural (single-spool `N/√Tt4` ≈ const), not a map assumption.
6. **`N` ATTACHES AND IS MONOTONE.** `N/N_d` and corrected speed are produced, equal 1 at design,
   and fall monotonically with `Tt4`; the leading schedule is robust across speed-line `σ` (bounded
   spread) while its magnitude is disclaimed.
7. **DIRECTION / CONVERGENCE.** Hotter `Tt4` ⇒ higher `π_c`, `ṁ`, `N`; the outer secant converges
   across the choked envelope (a non-convergence assert guards the deep-throttle feedback edge).

---

## Concessions

- **Representative maps, not hardware maps.** Analytic loading + efficiency-island + flat-turbine
  closures with disclosed shapes; every load-bearing claim is verified shape-robust and every
  magnitude is disclaimed (rungs 12–24 methodology).
- **No surge line / surge margin.** See the finding — deliberately not claimed.
- **No absolute shaft speed.** `N` is reported only as `N/N_d` and corrected ratios; absolute rpm
  needs blade geometry (deferred).
- **Isentropic knobs only; choked-nozzle branch only.** Inherited from rung 31 — the map matcher
  builds on rung 31's choke machinery, so its envelope (nozzle unchokes at deep throttle) and its
  isentropic-`η` scope are unchanged. Past unchoke the matcher flags the mode change (rung 31's
  boundary), it does not model the subsonic-nozzle branch.
- **`η_b, π_b, π_n` held at design.** Only `η_c, η_t` are put on maps (the two the rung-31
  concession named); combustor/nozzle-loss maps are a further seam.
- **Diagnostic beside the cycle.** The production run stays on the specified-`π_c` design path; the
  map matcher is a separate entry point. Switching the production engine to run off-design on a map
  is a re-foundation, not this rung.

---

## Anchor

`docs/plans/rung32-anchor-component-maps.md`. The **method** is the Cohen–Rogers–Saravanamuttoo /
Mattingly off-design component-matching procedure (superimpose the turbine+nozzle flow
compatibility on the compressor map to find the equilibrium running line; read `η`, `N` off the
map along it) — same textbook family as the rung-2 design anchor and the rung-31 referencing
anchor. Rung 32's specific result is that for a **choked-turbine / choked-nozzle single spool** the
compatibility *collapses* (rung 31 already pinned the line without iterating a compressor map), so
the map's remaining job is exactly to **label the pinned line with `(η, N)`** — and the `η` label
moves `π_c`/`ṁ` at first order (the correction of rung 31), while the turbine label barely moves
and `N` rides mostly on the choke. The reduce-to-rung-31 flat-map gate is the rigorous,
non-tautological check (two code paths — map-off vs rung 31 — onto the same operating point).
