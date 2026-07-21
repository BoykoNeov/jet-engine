# Rung-32 anchor — component-map matching (the map labels the choke-pinned line)

Two-part anchor, both re-derived and verified against the running model (temp probes
`rung32_proto.py`, `rung32_smoke.py`, `rung32_turb.py`, `rung32_nshape.py`). Part A is the
**method** (Cohen–Rogers–Saravanamuttoo / Mattingly off-design component matching) and the
rigorous **reduce-to-rung-31** gate. Part B is the reacting-gas running line that carries the
finding.

## Part A — the method, and the reduce (the rigorous, non-tautological gate)

The textbook off-design procedure superimposes the turbine + nozzle **flow compatibility** on the
**compressor map** to locate the equilibrium running line, then reads `η` and `N` off the map
along it (Cohen–Rogers–Saravanamuttoo *Gas Turbine Theory*, off-design chapter; Mattingly
*Elements of Propulsion* Ch. 8 — same textbook family as the rung-2 design anchor and the rung-31
referencing anchor). The *equations* are the anchor; no single worked map example is pinned (the
map shape is a disclosed representative closure — rungs 12–24 methodology).

**The rung-31 collapse.** For a **choked-turbine / choked-nozzle single spool** the flow
compatibility already pins the running line WITHOUT iterating a compressor map (that IS rung 31).
So the map's remaining job is exactly to **label** the pinned `(ṁ, π_c)` point with `(η_c, η_t,
N)`:
- the compressor **efficiency island** supplies `η_c`, which enters `π_c = [1 + η_c(τ_c−1)]^(γc/(γc−1))`;
- the compressor **speed lines** (Euler work `Δh_c = ψ U²` + loading law `ψ(φ)`) supply `N` by
  inverting `(τ_c−1)/(τ_c−1)_d = ψ(m/n)·n²` at the pinned point;
- the turbine map (choked ⇒ fixed corrected flow) supplies `η_t` at the turbine corrected speed.

### Gate 1 — the FLAT map reproduces rung 31 BIT-FOR-BIT (two code paths, one operating point)

Flat map `{a=b=c=0, σ=0, a_t=0}` ⇒ every `η` held at design ⇒ `MapMatcher.match` == rung-31
`OffDesignMatcher.match`. Reacting gas (`Gas.reacting_equilibrium()`), design `π_c=10, Tt4=1500,
M0=0.85`, real losses, `nozzle_convergent=True`, throttle sweep:

| `Tt4` | `π_c` (map) | `π_c` (rung 31) | rel | `τ_t` rel | `N/N_d` |
|-------|-------------|-----------------|-----|-----------|---------|
| 1500 | 10.00000000 | 10.00000000 | 0.0e0 | 0.0e0 | 1.0000 |
| 1200 | (rung-31 value) | (rung-31 value) | ≤1e-9 | ≤1e-9 | ~0.80 |
| 900  | 4.64074354 | 4.64074354 | 0.0e0 | 0.0e0 | 0.7732 |

`π_c`, `ṁ`, `τ_t`, all stations and thrust match to **machine zero** at design and ≤1e-9 across the
sweep; `N` is an extra diagnostic that does not perturb the reduce. The map-off matching and rung
31 are two different code paths onto the same operating point — the non-tautological check.

## Part B — the reacting running line (the finding)

Same reacting design reference (the rung-30 choked-convergent point, specific thrust 745.7). With a
**peaked** compressor map (peak-η at design), throttle sweep, `M0 = 0.85`:

### The finding — `π_c`/`ṁ` droop; the work `τ_c` is map-free

| `Tt4` | `π_c` (r31) | `π_c` (map) | `Δπ_c` | `Δṁ` | `η_c` | `τ_c` rel to r31 |
|-------|------------|------------|--------|------|-------|------------------|
| 1500 | 10.000 | 10.000 | 0.00% | 0.00% | 0.8800 | 0 (design) |
| 1300 | 7.856 | ~7.826 | −0.4% | −0.4% | ~0.878 | ~1e-9 |
| 1100 | 6.094 | ~6.014 | −1.3% | −1.3% | ~0.872 | ~1e-7 |
|  900 | 4.641 | ~4.539 | −2.2% | −2.2% | ~0.864 | ~4e-7 |

(Representative flow-dominated shape; thermally-perfect numbers, gas-independent physics.) `π_c` and
`ṁ` fall **below** rung 31's constant-η line, **same sign for all three map shapes**, gap growing
with throttle. The work `τ_c` matches rung 31 to ~1e-4 (choke-pinned) — the map moves `π_c`/`ṁ`, not
the work.

**Shape robustness (the load-bearing claim).** At `Tt4 = 900`, `Δπ_c` across the three shapes:
flow-dominated ≈ −2.2%, pressure-dominated ≈ −1.7%, tilted ≈ −2.3% — same sign, magnitude
disclaimed.

### Sub-finding — the turbine is pinned in corrected speed (structural, single spool)

The turbine corrected speed `nu_t = (N/√Tt4)/(…)_d` stays within ~1% of design across a 2:1+
throttle (N and √Tt4 fall together on a single spool):

| `Tt4` | `nu_t` | `N/N_d` | `|Δη_t|` (steep map `a_t=0.5`) | `|Δη_c|` |
|-------|--------|---------|-------------------------------|----------|
| 1500 | 1.00000 | 1.0000 | 0.0 | 0.0 |
| 1100 | 1.00323 | 0.859 | 5.2e-6 | ~1.1e-2 |
|  900 | 1.00612 | 0.779 | 1.9e-5 | ~1.6e-2 |
|  700 | 1.00679 | 0.688 | 2.3e-5 | ~2.0e-2 |

Even a **25×-steeper** turbine map moves `η_t` by only ~2e-5 — the turbine barely samples its map
because its corrected speed is pinned. Rung 31's "hold `η_t` constant" was nearly exact for this
structural reason.

### `N` is a real (shape-dependent) output, but its schedule is robust

`N/N_d` falls monotonically with `Tt4` (1.000 → 0.688). Across the speed-line loading curvature
`σ ∈ {0, 0.3, 0.6, 1.0}` the spread at fixed `Tt4` grows 0.40% (1300) → 3.43% (700) — so `N` is
**genuinely** σ-dependent (not the tautological `√(τ_c−1)` of the σ=0 case), yet the leading
throttle schedule is robust (~few %). Absolute rpm is disclaimed (needs blade geometry).

## Cross-links

- **Method / physics anchor:** Cohen–Rogers–Saravanamuttoo *Gas Turbine Theory* (off-design
  component matching); Mattingly *Elements of Propulsion* Ch. 8. Same textbook family as
  `docs/plans/rung2-anchor-mattingly.md` and `docs/plans/rung31-anchor-offdesign.md`.
- **Hardware / running-line anchor:** rung 31 (`docs/rung31-spec.md`,
  `docs/plans/rung31-anchor-offdesign.md`) — the choke-pinned running line this map labels; the
  flat-map reduce is rung 31 bit-for-bit.
- **Map-shape discipline:** the representative-closure methodology of rungs 12–24 (parametric
  shapes disclosed, load-bearing claims shape-robust, magnitudes disclaimed).
