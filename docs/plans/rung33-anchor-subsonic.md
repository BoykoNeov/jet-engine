# Rung-33 anchor — the subsonic-nozzle matching branch

Two-part anchor, mirroring rung 31. Part A pins the subsonic matching solver against the
textbook **compressible-flow mass-flow parameter** to machine precision on a CPG gas (the
rigorous, non-tautological gate). Part B is the reacting-gas running line across the
nozzle-unchoke boundary that carries the finding.

## Part A — the CPG algebraic MFP (the standard method), gate 4

Below the nozzle-unchoke boundary only the NGV stays choked. The exhaust nozzle expands fully to
`p9 = p0` at **subsonic** `M9`, so it passes the compressible-flow mass-flow parameter (Mattingly
*Elements of Propulsion*, standard 1-D isentropic gas dynamics):

```
MFP(M9)   = √(γ/R)·M9·(1 + ε M9²)^(−(γ+1)/(2(γ−1))) ,   ε = (γ−1)/2
pt9/p0    = (1 + ε M9²)^(γ/(γ−1))                     # isentropic, told back-pressure p9 = p0
MFP*      = √(γ/R)·(2/(γ+1))^((γ+1)/(2(γ−1)))          # the NGV sonic parameter, still choked
```

The two mass flows (NGV = nozzle) give the subsonic match constraint `(★★)`:

```
π_t/√τ_t  =  A4·MFP*(Tt4) / ( A8·π_n·MFP(M9) ) ,     M9 = M9(pt9/p0)
```

### Gate 4 — an INDEPENDENT CPG closed-form solve of `(★★)` reproduces the shipped solver

Self-consistent CPG dual gas (`R_t = (γ_t−1)/γ_t·cp_t`, `γ_c=1.4, cp_c=1004, γ_t=1.3,
cp_t=1239`), design `π_c=10, Tt4=1500, M0=0.85`, real losses. A second solver, written entirely in
closed-form calorically-perfect algebra — **no `_sonic_throat`, no `Nozzle.apply`** — root-finds
`π_t` on the same `(★★)` mass balance:

```
τ_t   = 1 − η_t(1 − π_t^((γ_t−1)/γ_t))                         # CPG isentropic turbine
Tt3   = Tt2 + η_m(1+f)·cp_t(Tt4−Tt5)/cp_c ,  f one-shot        # shaft balance (nested f)
π_c   = [1 + η_c(Tt3s/Tt2 − 1)]^(γ_c/(γ_c−1)) ,  Tt3s = Tt2+η_c(Tt3−Tt2)
M9    = √( 2/(γ_t−1)·[(pt9/p0)^((γ_t−1)/γ_t) − 1] ) ,  pt9 = π_n π_t π_b π_c pt2
mdot_NGV = A4·pt4·MFP*(Tt4)/√Tt4  =  A8·pt9·MFP(M9)/√Tt9 = mdot_noz   # root
```

At each matched subsonic point (`Tt4 = 580, 540, 500, 460`) the independent solve reproduces the
shipped solver's `π_t, π_c, τ_t, M9` to **machine zero** (`Δπ_t = 0`, `Δπ_c ≈ 1e-15`). Two
genuinely separate code paths — the shipped `_sonic_throat`/`Nozzle`/`(★★)`-bisection and the
closed-form algebra — onto one operating point.

**Why this is the load-bearing gate here.** Gate 1 (reduce-to-design) is a *choked* point: it
returns before the subsonic dispatch and never runs `_match_subsonic`. So the subsonic solve has
no reduce-to-prior anchor and only a loose boundary-continuity check. This independent solve is the
only thing that ties the deep-subsonic operating-point *values* to the textbook — verified by
injecting a 1% `π_c` error into the shipped `_subsonic_operating`: this gate fails (`reldiff = 1e-2`)
where gates 1 and 2 pass. On the gas the textbook assumes, the subsonic matching solver *is* the
textbook dual-mode ratio method.

## Part B — the reacting-gas running line across the unchoke boundary (the finding)

Design REFERENCE = the choked-**convergent** design point (rung 30): `Gas.reacting_equilibrium()`,
`π_c=10, Tt4=1500, M0=0.85`, real losses
(`pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96, eta_t=0.90, eta_m=0.99, pi_n=0.98`),
`nozzle_convergent=True`. Same fixed throats as rung 31 (`A4, A8` captured from that run).

### Running line — throttle sweep across the boundary (M0 = 0.85)

Reacting gas (`Gas.reacting_equilibrium()`). The nozzle unchokes near `Tt4 ≈ 625` (just above the
first row); `Tt4 = 700, 650` are still choked (bit-for-bit rung 31, not repeated here).

| `Tt4` | branch | `π_c` (OUTPUT) | `τ_t` | `M9` | `ṁ/ṁ_R` | sp. thrust | `pt9/p0` |
|-------|--------|------|-------|------|---------|-----------|---------|
| 620 | subsonic | 3.054 | 0.810408 | 0.9988 | 0.496 | 141.8 | 1.884 |
| 600 | subsonic | 2.957 | 0.809824 | 0.9709 | 0.489 | 125.3 | 1.825 |
| 560 | subsonic | 2.757 | 0.809735 | 0.9133 | 0.472 |  92.3 | 1.713 |
| 520 | subsonic | 2.553 | 0.811338 | 0.8539 | 0.455 |  59.5 | 1.608 |
| 480 | subsonic | 2.347 | 0.814923 | 0.7935 | 0.436 |  27.3 | 1.513 |
| 440 | **SUB-IDLE** | — | — | — | — | ≤ 0 | — |

The auto-dispatch: at each `Tt4` the matcher solves the choked branch, checks the rebuilt nozzle,
and switches to the subsonic branch when it is not choked. `M9` passes through 1 continuously at
the boundary; the branch label flips `choked → subsonic`; below thrust-neutral idle (`Tt4 ≈ 460`)
the matcher reports SUB-IDLE (net thrust ≤ 0) rather than a drag point.

**Why the reacting `τ_t` is muddied — and why the finding is a CPG statement.** On the reacting
gas `τ_t` is **non-monotone** here (`0.8104 → 0.8097` dip near the boundary, then rising to
`0.8149`), spread only **0.64%**. Two effects compete: the **composition drift** pulls `τ_t` *down*
as `Tt4` falls (the same sign as rung 31's choked reacting drift), while the **structural `π_c`
coupling** pushes it *up*. Composition dominates near the boundary; the structural coupling wins
deeper. The CPG gas removes the composition channel entirely, exposing the pure structural coupling
as a clean **monotone ~1.2%** rise (Part A / gate 3) — which is why the rung is stated on CPG.

### The finding: the decoupling breaks — first-order, survives CPG

On the choked branch `τ_t` was **machine-constant on CPG** (rung 31 gate 2) and drifted only on the
reacting gas (a 2nd-order `γ_t(T)` effect). On the subsonic branch the coupling runs through **`π_c`**
(structural), so `τ_t` **VARIES on the CPG gas** — measured spread **≈ 1.2%** across the subsonic
window (`Tt4 = 580 → 460`), rising monotonically toward 1 as the turbine expands less. This is the
inversion of rung 31: the effect that *died* on CPG for the choked branch is *first-order and alive*
on the subsonic branch.

The framing that is **wrong**: it is NOT a coupling to the ambient pressure `p0`. The cycle is
homogeneous degree 1 in pressure — scaling `p0` at fixed `(M0, T0, Tt4)` leaves every ratio
(`π_c, τ_t, M9`) invariant (verified to machine zero, gate 6). The coupling is to the **pressure
ratio `π_c`** through `pt9/p0`.

### The envelope (both boundaries)

- **Upper — nozzle unchoke:** `pt9/p0 <` critical (`≈ 1.85`); at `M0=0.85` the nozzle unchokes near
  `Tt4 ≈ 600`. At low ram (near-static `M0 ≈ 0.10`) the lower `pt` unchokes it much higher (CPG:
  `Tt4 ≈ 820`), widening the subsonic window — the idle-descent / ground-idle regime.
- **Lower — thrust-neutral idle:** as `Tt4` falls, `π_c → 1` and `(1+f)V9 → V0`, so net thrust → 0
  (`Tt4 ≈ 440` at `M0=0.85`). Below it the engine produces net drag (windmilling) and does not
  self-sustain useful thrust — reported as SUB-IDLE.

## Cross-links

- **Method anchor:** Mattingly *Elements of Propulsion* Ch. 8 — the **dual matching mode** (choked
  vs subsonic nozzle). Same textbook family as the rung-2/30/31 design/off-design anchors. The
  subsonic branch replaces the sonic `MFP*` with the compressible-flow `MFP(M9)` at the told
  back-pressure `p9 = p0`.
- **Hardware anchor:** the fixed convergent nozzle is rung 30's; the fixed throats `A4, A8` are
  rung 31's design capture. The subsonic exit reuses the shipped `Nozzle` subcritical branch
  (`p9 = p0`, `M9 < 1`), which is bit-for-bit the default nozzle at that condition.
