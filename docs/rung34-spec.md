# Rung 34 — The spool transient: `N` becomes a STATE, not an output

Rungs 31–33 solved **steady** off-design operating points. Every one of them closes the flow with
the **shaft power balance** `η_m·P_turbine = P_compressor` — that algebraic constraint is what pins
the compressor work `τ_c` (rung 31) and, once rung 32 put a map on it, lets the speed lines read
off a shaft speed `N`. The running line is the locus of those balanced points.

Rung 34 asks the question rung 32 made askable the moment `N` appeared: **what happens when the
shaft is NOT balanced?** A real spool has rotational inertia; a fuel change unbalances the power and
the rotor accelerates. So the shaft balance stops being an algebraic constraint and becomes a
**differential equation** — and `N`, which rungs 31–33 *computed*, becomes the **state variable that
carries the engine's memory** between instants.

This is the first **dynamic** rung. Everything before it was a fixed point; this one integrates.

---

## The model: quasi-steady components, one dynamic element (the shaft)

The separation of time scales is the whole physical content of the standard first transient model:

- **The gas path is acoustically fast.** Pressure waves cross the engine, the choked throats
  re-establish their sonic mass flow, and the burner re-lights its flame in milliseconds. Relative
  to the rotor these are instantaneous, so at each instant the **flow is quasi-steady**: it satisfies
  exactly the same compatibility relations as the steady matcher — *except the shaft balance*.
- **The shaft is mechanically slow.** The rotor's angular momentum changes only as fast as the net
  torque can change it, over tenths of a second to seconds. So the **shaft speed `N` is the one
  dynamic state**; the net power imbalance drives it.

Newton for the spool (rotational form), with polar moment of inertia `I` and angular speed `ω`:

```
I·ω·(dω/dt)  =  η_m·P_turbine(N, Tt4)  −  P_compressor(N, Tt4)                     (SHAFT ODE)
```

The left side is `d/dt(½ I ω²)` — net power goes into (or out of) the rotor's kinetic energy. At
steady state the right side is zero and we are back on rung 32's running line. Off it, the sign of
the imbalance moves `N` toward the running line (shown stable below).

**`I` is a genuinely new physical constant** — the first since rung 26's `L` and rung 29's `τ_res`.
Like them it sets a **time scale and nothing else** (derivation below), and like them its magnitude
is **disclaimed**.

---

## What is genuinely new: the compressor map runs FORWARD

Rungs 31–32 used the compressor **backward**: the choke + shaft balance pinned `(τ_c, ṁ)`, and the
speed-line equation was *inverted* (`solve_n`) to label that pinned point with an `N`. The transient
cannot do that — `N` is now an input (the state), not something to be read off at the end. So the map
is used **forward**: given the corrected speed `n(N, Tt2)` and a trial corrected flow `m`, the Euler
speed line gives the work directly

```
τ_c  =  1 + (τ_c,d − 1)·ψ(φ)·n² ,     ψ(φ) = 1 − σ(φ−1)² ,     φ = m/n            (FORWARD MAP)
```

which is the **exact inverse of rung 32's `solve_n`** (asserted to machine zero — gate 6). The
efficiency island gives `η_c(φ, n)`, and the compressor pressure ratio follows from the *same*
enthalpy/`pr` inverse the shipped `Compressor.apply` uses (gas-correct, reduces to the CPG closed
form on a CPG gas):

```
Tt3 = Tt2·τ_c ,   Tt3s = T_from_h_c(h_c(Tt2) + η_c·[h_c(Tt3) − h_c(Tt2)]) ,
π_c = pr_c(Tt3s)/pr_c(Tt2)
```

### The closure at `(N, Tt4)` — NO shaft balance

The key structural simplification: **the compressor operating point is closed by the NGV choke
alone, on either branch.** The NGV is the last throat to unchoke (rung 33's concession), so it is
choked throughout the reported envelope; its sonic mass flow is a pure function of `(Tt4, f)` and
`pt4`. Since `pt4 = π_b·π_c·pt2` and `π_c` depends only on `(n, m)` — *not* on the turbine — mass
continuity through the NGV is one equation in the one unknown `m`:

```
ṁ_air(m)·(1 + f)  =  A4·pt4(m)·MFP*(Tt4, f)/√Tt4                                   (NGV CLOSURE)
```

Root-find `m`. That fixes `(π_c, τ_c, η_c, ṁ_air, f, pt4, Tt3)` **without ever balancing the shaft**.
The turbine expansion is then whatever the downstream hardware demands:

- **Choked nozzle:** the turbine NGV *and* the nozzle are both choked, so `π_t` is pinned by rung
  31's pure-geometry `(★) π_t/√τ_t = A4·MFP4/(A8·π_n·MFP9)` — **independent of `N`**. Turbine specific
  work `h_t(Tt4) − h_t(Tt5)` is therefore fixed at fixed `Tt4` (bar the weak `f` drift).
- **Subsonic nozzle:** `(★)` is void (rung 33); `π_t` is the value that makes the fully-expanded
  nozzle pass exactly the NGV mass flow, `A8·ρ9·V9 = ṁ4` — a 1-D root-find on `π_t`. The instant
  evaluator **dispatches choked-vs-subsonic exactly as rung 33's `match()` does** (reusing the same
  `_sonic_throat`/`Nozzle` machinery), so the transient spans both matching branches seamlessly.

Both give `Tt5`, hence `P_t = η_m·(1+f)·[h_t(Tt4) − h_t(Tt5)]` and `P_c = h_c(Tt3) − h_c(Tt2)` (per
unit air mass), hence the imbalance that drives the SHAFT ODE.

**Why this is stable.** On the choked branch `P_t` (specific) is pinned by `Tt4` while `P_c` rises
monotonically with `N` (more speed → more compressor work). So the bracket `η_m·P_t − P_c` is
**decreasing in `N`**: above the equilibrium `N` it is negative (decelerate), below it positive
(accelerate). The running line is a **stable attractor** — the transient's equilibrium manifold.

---

## The finding: it is NOT "shape is `I`-independent" — that is a tautology

The tempting headline — *"the trajectory shape is `I`-independent; `I` only sets the clock"* — is
**true but vacuous**, and stating it as the finding would fail this project's own standard (the
rung-29 gate-2 / rung-33 gate-4 anti-tautology bar). In a one-state model `N(t)` is the only dynamic
variable; everything else is an algebraic function of `(N, Tt4)`. Nondimensionalize time by
`s = t/τ_spool`:

```
ν ≡ N/N_d ,     dν/ds  =  Φ(ν, Tt4) ≡ p_net(ν, Tt4)/ν ,
τ_spool ≡ I·ω_d² / P_ref  (P_ref = design shaft power)                            (NONDIM SHAFT)
```

`Φ` contains no `I`. So "the `s`-trajectory is `I`-independent and `t = τ_spool·s`" is just
dimensional analysis — the same content as "a step response rescales with its time constant." Not a
finding.

**`I` becomes load-bearing only when a SECOND clock competes with it.** Give the fuel its own
time scale: ramp `Tt4` from `Tt4,1` to `Tt4,2` over a finite fuel time `τ_fuel`. Now the response
depends on the **ratio**

```
r  =  τ_fuel / τ_spool
```

and the peak **excursion of the operating point above the running line** — the compressor-map
distance toward surge, `E = max_t [ π_c(t)/π_c,runningline(ν(t)) − 1 ]` — is a genuine function
`E(r)`:

- **`r → 0` (fuel step, spool frozen):** `ν` cannot move while the fuel changes, so the operating
  point jumps at **constant `N`** to the new `Tt4`. `E(0)` is the constant-speed displacement of the
  running line — an **algebraic map property**, the largest excursion possible.
- **`r → ∞` (slow ramp):** `ν` tracks the running line quasi-statically; the point never leaves it,
  `E → 0`.
- **crossover at `r ~ 1`:** when the fuel and the spool share a time scale.

`E(r)` is monotone-decreasing from the constant-`N` displacement to zero, with the knee at `r ≈ 1`.
**That** is the rung: it is *why real engines schedule their fuel ramps* — you slow the fuel
(raise `r`) to keep the acceleration excursion clear of the surge line — and it is the honest home
for `I`, because "fast" and "slow" are meaningful only relative to `τ_spool`. The step's peak
excursion is presented as what it is — an algebraic property of the map (constant-`N` vs
running-line displacement) — and the *dynamical* weight sits on the ratio.

### Direction (the surge-margin wording — inherited rung-32 concession)

An **acceleration** (fuel step up) throws the point **above** the running line (higher `π_c`, lower
corrected flow at fixed `N`) — **toward lower surge margin**. A **deceleration** (fuel step down)
throws it **below** (toward flameout / lean blowout). Rung 32 draws **no surge line**, so the claim
is the **signed displacement toward/away from surge**, quantified — never "the transient reaches
surge." The magnitude of `E` rides on the map shape and is verified **shape-robust in sign**,
**disclaimed in magnitude** (rung-32 methodology).

### Spool-down and windmilling (the rung-33 handshake)

Cut the fuel toward its floor: `η_m·P_t < P_c`, `dν/ds < 0`, and `N` **decays** — a spool-down
transient. As `ν` and `Tt4` fall together `pt9/p0` drops, the nozzle **unchokes**, and the trajectory
crosses onto rung 33's **subsonic branch**; it continues down toward rung 33's **thrust-neutral
idle**. Whether a **windmilling equilibrium** — a positive-`N` steady point at (near-)zero fuel where
ram-driven turbine work just balances compressor drag — exists **within** the self-sustaining window
is left to the numerics: it is discovered, not predicted, and may report SUB-IDLE (rung 33's lower
bound) rather than a fixed point.

---

## Reduce-to-prior contract (the spine)

The transient's **equilibrium** (`dν/ds = 0`, i.e. the `N` where `η_m·P_t = P_c`) must reproduce the
steady matcher — via a **genuinely different closure** (forward-map + NGV-continuity + power-balance,
vs the steady matcher's choke + shaft-balance), so the check is non-circular:

- **Flat map ⇒ rung 31 bit-for-bit.** With `ComponentMap.flat()` the forward speed line is
  `τ_c = 1 + (τ_c,d − 1)·n²` (map-free) and `η_c ≡ η_c,d`; the equilibrium-`N` solve reproduces
  `OffDesignMatcher.match` (rung 31) — `π_c, ṁ, τ_t`, stations, thrust — to solver tolerance. This is
  the **tightest** anchor: the simplest steady solver, the most tautology-free contrast. The
  equilibrium solver uses **only the forward closure** — it never calls `MapMatcher.match` internally
  (that would make the reduce circular).
- **Shaped map ⇒ rung 32 bit-for-bit.** With any `ComponentMap` shape the equilibrium reproduces
  `MapMatcher.match` (`π_c, ṁ, τ_c, N`).
- **Cycle untouched ⇒ rung 6 bit-for-bit.** Separate entry point; the default `build_turbojet(…).run`
  design path is not perturbed. The rungs-7+ invariant holds.

Because the equilibrium of the ODE *is* the steady running line, the dynamic rung sits cleanly on top
of the static ones: rung 34 adds the *time axis*, it does not move any steady number.

---

## Verification gates (`tests/test_rung34.py`)

1. **REDUCE — equilibrium == steady matcher.** The equilibrium-`N` solve with the **flat** map
   reproduces `OffDesignMatcher.match` (rung 31), and with a **shaped** map reproduces
   `MapMatcher.match` (rung 32), across a throttle sweep (machine-zero at design; ≤1e-8 on the sweep).
   Uses the forward closure only (never calls the steady matchers internally).
2. **STABILITY / ATTRACTOR.** `Φ(ν, Tt4)` is decreasing through its zero at every sampled `Tt4`
   (`∂Φ/∂ν < 0`): perturb `N` off equilibrium and the power imbalance has the restoring sign. An
   integrated step from an off-equilibrium `N` relaxes back onto the running line.
3. **THE FINDING — `E(r)` monotone, knee at `r ≈ 1`.** For a finite fuel ramp, the peak
   above-running-line excursion `E(τ_fuel/τ_spool)` **decreases monotonically** from the constant-`N`
   displacement (`r → 0`) toward ≈0 (`r → ∞`). The `r → 0` limit equals the **algebraic** constant-`N`
   map displacement (computed with no integration) to tolerance — certifying that the step excursion
   is a map property, and that the *dynamical* content is the ratio.
4. **DIRECTION, SHAPE-ROBUST.** Acceleration drives the point **above** the running line
   (`Δπ_c > 0` at fixed `N`), deceleration **below**, **same sign across ≥3 `ComponentMap` shapes**;
   magnitude disclaimed. Worded "toward/away from surge," no surge line asserted.
5. **`I` IS ONLY THE CLOCK (the anti-tautology witness).** Two integrations of the SAME nondimensional
   ramp at different `I` give the SAME `ν(s)` trajectory (to integrator tolerance) and physical times
   in the ratio of the `I`s — the explicit demonstration that the step-excursion framing would be
   vacuous and the ratio framing is not.
6. **FORWARD/BACKWARD MAP INVERSE.** `solve_n(m, τ_c_forward(n, m)) == n` to machine zero across a
   grid — the forward speed line is the exact inverse of rung 32's `solve_n`.
7. **SPOOL-DOWN CROSSES INTO RUNG 33 / SUB-IDLE.** A fuel-cut spool-down decreases `N` monotonically,
   the branch label flips `choked → subsonic` as `pt9/p0` falls through critical, and the trajectory
   terminates at rung 33's thrust-neutral idle (reported SUB-IDLE, not force-fit). The subsonic
   instant is dispatched by the same rung-33 logic (no reinvention).
8. **CYCLE UNTOUCHED.** The default design run is bit-for-bit rung 6; constructing a `SpoolTransient`
   does not perturb it.

---

## Concessions

- **`I` (and `ω_d`) — one disclaimed time group.** The physical time scale `τ_spool = I·ω_d²/P_ref`
  rides on the moment of inertia and the (rung-32-disclaimed) absolute design speed `ω_d`. Only the
  **nondimensional** trajectory `ν(s)` and the **ratio** `r = τ_fuel/τ_spool` are claimed; wall-clock
  seconds are illustrative. This is the `L`/`τ_res` concession in a new guise (one knob sets the clock).
- **Quasi-steady components.** The shaft is the ONLY dynamic element — no combustor volume-filling
  (`dp/dt`), no heat soak into the metal, no tip-clearance transients. These are the next dynamic
  seams; they add faster clocks *below* `τ_spool`, they do not change the `r` framing.
- **Control = `Tt4(t)`.** The fuel schedule is expressed as a burner-exit-temperature schedule
  (matches the shipped `Burner`; `f` is the output). A true `ṁ_fuel(t)` schedule with `Tt4` as an
  output — the real fuel-metering picture — is a further seam.
- **No surge line / surge margin.** Inherited from rung 32: the excursion is a signed displacement
  toward surge on a representative map, never a surge-line crossing. Shape-robust sign, disclaimed
  magnitude.
- **Isentropic knobs; NGV assumed choked; single spool.** Inherited from rungs 31–33.
- **Diagnostic beside the cycle.** The production run stays on the specified-`π_c` design path; the
  transient is a separate entry point. Switching the production engine onto a marched `N(t)` is a
  re-foundation, not this rung.

---

## Anchor

`docs/plans/rung34-anchor-spool-transient.md`. The **method** is the standard gas-turbine transient
model — quasi-steady components with a dynamic shaft inertia (Cohen–Rogers–Saravanamuttoo *Gas
Turbine Theory* Ch. 9 "Prediction of performance — off-design and transient"; Walsh & Fletcher *Gas
Turbine Performance* transient chapter): the acceleration/deceleration excursion between the surge and
flameout lines, and the scheduled fuel ramp that keeps it clear. The **reduce gate** (equilibrium ==
rung 31/32 via the independent forward closure) is the rigorous, non-tautological anchor — two code
paths onto one operating point, exactly as rungs 31–33 did for their static solves.
