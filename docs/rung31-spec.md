# Rung 31 — Off-design matching: the operating point becomes an OUTPUT

Every rung so far has been a **design-point** calculation: you *specify* `π_c` and `Tt4`
and the cycle follows. That is a lie about how an engine actually runs. Real hardware is
**fixed** — the compressor, the turbine nozzle-guide-vane (NGV) throat, the exhaust-nozzle
throat are all cut once. When the flight condition or the throttle changes, the engine
cannot re-choose `π_c`; it runs wherever the fixed geometry *forces* it. Rung 31 asks: given
fixed hardware, where does the engine run?

This is the first **structural** rung. Rungs 7–30 were diagnostics that only *read* a cycle
whose inputs were given. Here the operating point `(π_c, ṁ, f, …)` is the **output of a
matching solve** — the compressor pressure ratio is no longer a knob you turn but a number
the choked hardware hands you.

**The verdict: with the turbine NGV and the exhaust nozzle both choked, the compressor has
no freedom left.** `τ_c − 1` is slaved to `Tt4/(τ_r·T0)` and `π_c`, `ṁ` ride a single fixed
**running line**. Reducing to the design point when operated there is automatic. The one
place the textbook "`τ_t` is exactly constant" is not quite true — the variable-`cp` drift —
is the rung.

---

## What is actually new: two choke constraints pin the turbine, invert the compressor

A **choked** station passes a fixed *corrected mass flow per unit area*:

```
ṁ·√Tt / (A·pt) = MFP*(gas, Tt, far)          # the M=1 mass-flow parameter
```

`MFP*` depends only on the gas and the stagnation temperature (the isentropic ratios
`T*/Tt, p*/pt, V*` are level-independent — `pt` cancels), so it is a property of the flow,
not of the pressure. We compute it **exactly** from rung 30's sonic-throat solver — the same
`_sonic_throat` that found the nozzle's `M=1` state — reused now for **both** choked throats:

```
MFP*(Tt, far) = ρ*·V*·√Tt / pt = (p*/(R_t·T*))·V*·√Tt / pt      # from _sonic_throat(Tt,pt,far)
```

The engine has two choked throats and therefore two constraints:

```
(NGV,  A4 fixed)   ṁ = A4·pt4·MFP*(Tt4, f) / √Tt4
(nozzle, A8 fixed) ṁ = A8·pt9·MFP*(Tt9, f) / √Tt9
```

`A4` and `A8` are **fixed geometry**, captured once from the design run (this is exactly the
throat area rung 30 deliberately did *not* carry — "rung 31's to add"). Off-design they do
not move.

### The turbine operating point is pinned by geometry (the classic result)

Equate the two mass flows (`pt4` cancels, `Tt9 = τ_t·Tt4`, `pt9 = π_n·π_t·pt4`):

```
π_t / √τ_t  =  A4·MFP*(Tt4, f) / ( A8·π_n·MFP*(τ_t·Tt4, f) )        (★)
```

The right side is **almost constant** — pure geometry `A4/A8`, the nozzle loss `π_n`, and a
ratio of two sonic mass-flow parameters. Combined with the turbine's own thermodynamics
(the isentropic-efficiency map `π_t → τ_t`), (★) is **one equation in `π_t`**, solved by
bisection. Its solution is **independent of the compressor and of the flight speed**: the
turbine "does not know" the operating condition changed. In a calorically-perfect gas the
right side is a genuine constant and `τ_t, π_t` are *exactly* fixed — the textbook result
that a choked turbine feeding a choked nozzle runs at constant temperature and pressure
ratio.

### The structural inversion: `π_c` falls out of the shaft balance

At the **design** point the shaft balance *sets the turbine work* (you gave `π_c`, the
compressor work followed, the turbine repaid it, `τ_t` came out). **Off-design that runs
backwards.** `τ_t` is now pinned by (★), so the shaft balance instead determines the
**compressor**:

```
h_c(Tt3) − h_c(Tt2)  =  η_m·(1+f)·( h_t(Tt4, f) − h_t(Tt5, f) )     # Tt5 = τ_t·Tt4, known
   ⇒  Tt3 = T_from_h_c( h_c(Tt2) + η_m(1+f)(h_t(Tt4)−h_t(Tt5)) )
```

`Tt3` is now known, so `π_c` is recovered by **inverting** the compressor efficiency map
(the exact inverse of the shipped `Compressor.apply`):

```
h_c(Tt3s) = h_c(Tt2) + η_c·( h_c(Tt3) − h_c(Tt2) )                  # ideal substate
π_c       = pr_c(Tt3s) / pr_c(Tt2)                                 # OUTPUT, not input
```

`π_c` is now a *number the hardware hands you*, not a knob. Because `τ_t` is fixed and
`Tt2 = T0·τ_r`, this collapses (in CPG) to the textbook slaving law

```
τ_c − 1  =  η_m(1+f)·(cp_t/cp_c)·(Tt4/(T0·τ_r))·(1 − τ_t)   ∝  Tt4 / (τ_r·T0) .
```

### Closing the loop: `f`, `ṁ`, thrust

`f` depends on `Tt3` and the burner pressure `pt4 = π_b·π_c·π_d·pt0`, which depends on `π_c`,
which depends on `f` through the hot enthalpies — a weak coupling closed by a **fixed-point
on `f`** (the same `Burner._solve_equilibrium` the design cycle uses; it converges in a
handful of passes because `f` is a small correction). The absolute mass flow then comes from
the turbine choke constant

```
ṁ = A4·pt4·MFP*(Tt4, f)/√Tt4 ,
```

and `ṁ/ṁ_R` is the **mass-flow lapse**. The nozzle is choked by construction (the (★) match
guarantees it passes exactly this `ṁ`), so its exit is the rung-30 sonic state and the
engine's existing specific-thrust formula — momentum plus the underexpanded pressure term —
scores it unchanged.

---

## Reduce-to-prior contract (the spine)

**Running the matching solver at the design flight condition and `Tt4` reproduces the design
cycle** — `π_c = 10`, every station, `ṁ`, thrust — to solver tolerance (≤1e-9; observed
~1e-12). This is **reduce-by-construction**, the rung-29 move in matching form:

- `A4, A8` are captured from *that* design run, so the two choke constraints are satisfied by
  the design `(τ_t, π_t, ṁ)` identically — (★) has its root exactly at `π_t = π_t_design`.
- The compressor inversion uses the *exact algebraic inverse* of `Compressor.apply`, so
  recovering `Tt3_design` returns `π_c = 10`.
- The `f` fixed-point starts already converged (`Burner._solve_equilibrium` at the design
  `Tt3, pt4` is the design `f`).

Unlike rung 29's `==`-exact delegation, the operating point here is genuinely the output of a
root-find, so the reduce is **to tolerance, not bit-for-bit** — the honest contract for a
solve whose answer is an implicit function of the inputs. The **default `build_turbojet(…)
.run(…)` design path is untouched**, so the production cycle stays bit-for-bit rung 6 and the
rungs-7–30 invariant holds (gate 3).

---

## The choked envelope (state the scope, don't quote outside it)

The whole method rests on **both** throats being choked. At the design point the nozzle is
choked hard (`pt9/p0 = 6.29 ≫` critical `1.85`, rung 30) and the NGV more so. But throttling
back (lower `Tt4`) or climbing (lower `p0`/higher `M0` at fixed geometry) lowers `pt9/p0`, and
below the critical ratio the **nozzle unchokes** — its exit returns to `p9 = p0`, (★) no
longer pins the flow, and a second (subsonic-nozzle) matching mode takes over (Mattingly's
dual mode). Rung 31 models the **choked-nozzle branch** and **reports the `Tt4` at which the
nozzle unchokes** as the branch boundary; it does not claim the subsonic-nozzle matching
mode (deferred, like the rung-32 maps). The NGV is assumed choked throughout the reported
envelope (standard — the NGV is the last throat to unchoke); this is asserted, not modeled as
an NGV passage.

---

## The finding: the choked hardware strips the compressor of freedom — and `τ_t` still drifts

Two results, one expected and one not:

1. **`π_c` is slaved.** Along the running line the compressor pressure ratio and mass flow
   are fixed functions of `Tt4/(τ_r·T0)`; the engine cannot run off its own line. Throttle
   down and `π_c`, `ṁ`, thrust all fall together — the pumping characteristic, produced
   **without a compressor map** (the choked downstream hardware *is* the map).

2. **`τ_t` is not exactly constant — and the driver is the `γ_t(T)` curve.** The textbook
   says a choked-turbine/choked-nozzle pair holds `τ_t, π_t` rigidly constant. That is a
   **calorically-perfect** statement: it needs the `MFP*` ratio in (★) to be a pure constant.
   On the real gas the two sonic parameters `MFP*(Tt4)` and `MFP*(Tt9)` sit at different
   temperatures, so `τ_t` drifts **+2.8%** over a 2:1 throttle (1500→800 K, both choked; only
   **0.16%** on the `M0` axis, where `Tt4` and so the composition barely move).

   **Kill-test — which real-gas effect drives it?** A three-gas ladder (drift over the same
   choked 1500→800 range), because the CPG contrast alone holds *both* `γ(T)` and composition
   fixed:

   | gas | `τ_t` drift | what it isolates |
   |-----|-----------|------------------|
   | CPG (fixed `γ`) | **0.000%** | the pure-constant `MFP*` ratio |
   | `thermally_perfect` (var `cp(T)`, **frozen** comp) | **+2.30%** | the `γ_t(T)`-curve effect alone |
   | `reacting_equilibrium` (var `cp` + comp) | **+2.84%** | the full real gas |

   The `γ_t(T)` curve carries **81%** of the drift; the composition shift (`f` falling with
   throttle) adds the minority ~19%. The clean reason it is a `γ(T)`-*curve* effect: within a
   single operating point **both throats carry the same frozen station-4 composition**, so
   `R_t` cancels in the `MFP4/MFP9` ratio — what is left is purely the different `γ_t(T)`
   sampled at `Tt4` vs `Tt9`. Same species of finding as rung 30's "0.03% is the physics, not
   error" — the honest refinement the constant-`cp` textbook cannot see.

---

## Verification gates (`tests/test_rung31.py`)

1. **REDUCE TO DESIGN (the spine).** The matching solver at the design flight + `Tt4`
   reproduces the design-run `π_c`, stations, `ṁ`, and thrust to ≤1e-9.
2. **THE SOLVER IS RIGHT (non-tautological).** On a **CPG** gas the TPG matching solve
   reproduces Mattingly's **closed-form referencing**: `τ_t, π_t` *exactly* constant across
   the throttle sweep; `τ_c − 1 ∝ Tt4/(τ_r·T0)`; `π_c = [1 + η_c(τ_c−1)]^(γc/(γc−1))`; and the
   mass-flow scaling `ṁ ∝ pt4/√Tt4`. Two different code paths (sonic-throat matching vs the
   algebraic ratio equations) onto the same operating point. Without this, gate 1 only
   exercises the design point itself.
3. **CYCLE UNTOUCHED.** Building the engine with the default path gives bit-for-bit the
   rung-6 stations, `V9`, thrust (the rungs-7+ invariant).
4. **THE VERDICT / RUNNING LINE.** Along a throttle sweep `π_c` and `ṁ` are strictly
   monotone in `Tt4` and match `Tt4/(τ_r·T0)` slaving; the reported nozzle-unchoke `Tt4`
   bounds the branch; outside it the solver flags the mode change rather than lying.
5. **THE DRIFT (the finding).** On the reacting gas `τ_t` is *not* constant along the sweep
   (drift measurably non-zero) while on a self-consistent CPG gas it is constant to 1e-9 —
   isolating the drift as the variable-`cp` physics.
6. **DIRECTION.** Throttle up (higher `Tt4`) ⇒ higher `π_c`, higher `ṁ`, higher thrust;
   asserted inside the solver on every call (contract #4).

---

## Concessions

- **Component efficiencies held at design.** `η_c, η_t, η_b, π_b, π_n` are kept at their
  design values along the running line. A real compressor's `η_c` moves along its map; that
  curvature is **rung 32** (Cohen–Rogers–Saravanamuttoo component maps). Rung 31 supplies the
  running line the map would sit on; holding the `η`'s constant is exactly the "no map"
  idealization it isolates.
- **NGV assumed choked; no NGV passage modeled.** The turbine choke is imposed by pinning its
  corrected-flow group and asserting `p* > `(downstream), not by resolving a vane passage.
- **Choked-nozzle branch only.** The subsonic-nozzle matching mode past unchoke is reported
  as a boundary, not modeled (see envelope).
- **`π_d(M0)` via `ram_recovery`.** Off-design at varying `M0` the inlet recovery follows the
  MIL-E-5008B correlation already in the repo; at the design `M0 = 0.85` (subsonic) it is 1,
  so the reduce is untouched.
- **Diagnostic beside the cycle.** The production run stays on the specified-`π_c` design
  path; off-design is a separate entry point. Switching the production engine to run
  off-design is a re-foundation, not this rung.

---

## Anchor

`docs/plans/rung31-anchor-offdesign.md`. Two-part:
- **The standard closed-form referencing method** — the choked turbine + choked nozzle
  result (constant `τ_t, π_t`, the `Tt4/(τ_r T0)` slaving, the mass-flow scaling), as in the
  performance-analysis chapters of Mattingly *Elements of Propulsion*, Hill & Peterson, and
  Cohen–Rogers–Saravanamuttoo (same textbook family as the rung-2 design anchor). The
  *equations* are the anchor — not a specific worked example (none was pinned) — and rung 31
  derives and reproduces them independently.
- **The CPG self-consistency gate** (gate 2, the rigorous anchor): the matching solver, run on
  a self-consistent calorically-perfect gas, reproduces those closed-form referencing
  equations to machine precision (`τ_t, π_t` const, slaving factor const, `π_c` to 1e-14) —
  the non-tautological check that the sonic-throat matching *is* the textbook ratio method on
  the gas the textbook assumes.
