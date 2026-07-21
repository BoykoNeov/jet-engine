# Rung 33 — The subsonic-nozzle matching branch (the decoupling breaks)

Rung 31 matched the engine against fixed hardware with **two choked throats** — the turbine
NGV and the rung-30 convergent nozzle. That gave the clean result `(★) π_t/√τ_t =
A4·MFP4/(A8·π_n·MFP9)`: **pure geometry** pins the turbine, `τ_t` and `π_t` are constant (on a
CPG gas, to machine zero), and "the turbine does not know the operating condition changed." Rung
31 also flagged its own scope edge: throttle back far enough (or climb) and `pt9/p0` falls below
the critical ratio, the **nozzle unchokes**, `(★)` is void, and the matcher reported
`nozzle_choked = False` rather than quoting an invalid choked-branch number — the subsonic-nozzle
matching mode "deferred (Mattingly's dual mode)."

Rung 33 builds that second mode. It is not a new physical effect — it is the **same off-design
match with one throat unchoked** — but it inverts rung 31's headline result, and the inversion is
the rung.

---

## What is actually new: only the NGV stays choked

Below the unchoke boundary the exhaust nozzle is **subsonic**: it expands fully to ambient
(`p9 = p0`, `M9 < 1`), exactly like the default nozzle. Its corrected throughput is therefore no
longer a fixed sonic `MFP*` but a **subsonic** mass-flow parameter that depends on the actual
pressure ratio:

```
ṁ·√Tt9/(A8·pt9) = MFP(M9),      M9 = M9(pt9/p0)      # subsonic, fully expanded
```

The single choked constraint that remains is the NGV:

```
ṁ = A4·pt4·MFP*(Tt4, f)/√Tt4                          # NGV still choked
```

Equate the two mass flows (`ṁ` through the NGV = `ṁ` through the nozzle):

```
π_t/√τ_t  =  A4·MFP*(Tt4, f) / ( A8·π_n·MFP(M9) )     (★★)
```

`(★★)` looks like `(★)` with the sonic `MFP9` replaced by the **subsonic** `MFP(M9)`. But
`MFP(M9)` depends on `pt9/p0 = π_n·π_t·pt4/p0`, and `pt4 = π_b·π_c·π_d·pt0` moves with **`π_c`**,
which the shaft balance ties back to `τ_t`. So the right side is **no longer pure geometry** —
`(★★)` couples the turbine to the compressor. The clean decoupling of rung 31 is gone.

### The solve

`π_t` is no longer pinned; it becomes the **equilibrating unknown** that makes the NGV-choked
supply meet the subsonic-nozzle demand. A one-dimensional root-find:

```
resid(π_t) = ṁ_NGV(π_t) − ṁ_nozzle,subsonic(π_t) = 0
```

For each trial `π_t`: the turbine map gives `τ_t, Tt5`; the shaft balance sets the compressor
rise `Tt3`; inverting the compressor efficiency map gives `π_c` (hence `pt4`, `pt9`); `ṁ_NGV`
comes from the NGV choke constant and `ṁ_noz = A8·ρ9·V9` from the fully-expanded nozzle exit.
The `(f, pt4)` fixed point nests inside exactly as on the choked branch. `resid` is
monotone-decreasing in `π_t` (more turbine expansion → more compressor work → higher `pt9` → the
nozzle passes more), so it brackets cleanly and bisects.

---

## The finding: the decoupling breaks — and it SURVIVES the CPG gas (the inversion of rung 31)

On the choked branch the coupling to the operating condition ran through the **variable-`cp`**
physics: `τ_t` drifted `~2.8%` on the reacting gas but was **machine-constant on a CPG gas** (rung
31 gate 2 / gate 5 — the drift was a second-order `γ_t(T)`-curve effect that *died* when `γ` was
frozen). Rung 33 is the opposite. The subsonic coupling runs through **`π_c`** — a structural,
geometric channel — so it is **first-order** and it **survives the CPG gas**:

| | choked branch (rung 31) | subsonic branch (rung 33) |
|---|---|---|
| `τ_t` on **CPG** gas | **constant** to machine zero | **VARIES** (`~1.2%` across the window) |
| what couples it | `γ_t(T)` curve (var-`cp`), 2nd order | `π_c` via `pt9/p0`, structural, 1st order |

That single contrast — **CPG `τ_t` constant on the choked branch, varying on the subsonic
branch** — is the rung. Rung 31's "the turbine does not know the operating condition changed" is
true *only while both throats are choked*; unchoke one throat and the turbine re-couples to the
compressor immediately, on any gas.

### The framing trap (what the coupling is NOT)

It is tempting to say the ambient pressure `p0` "re-enters" the turbine match. **It does not.** The
cycle is homogeneous of degree 1 in pressure: scale `p0 → λ·p0` at fixed `(M0, T0, Tt4)` and every
*ratio* — `π_c, τ_t, π_t, M9` — is unchanged (only the extensive `ṁ` and thrust scale). The
coupling is to the **pressure ratio `π_c` through `pt9/p0`**, not to the pressure level. Verified
to machine zero (gate 6): the subsonic operating point is `p0`-invariant.

---

## The envelope: bounded ABOVE by unchoke, BELOW by thrust-neutral idle

The subsonic branch is a genuine operating regime with **two** boundaries:

- **Upper — nozzle unchoke.** Above `pt9/p0 ≈` critical the nozzle chokes and the choked branch
  (rung 31) takes over. At the design flight this is `Tt4 ≈ 600`.
- **Lower — thrust-neutral idle.** As `Tt4` falls the turbine does less work, `π_c → 1`, and the
  jet slows until `(1+f)·V9 = V0`: **net thrust reaches zero**. Below it the engine produces net
  drag (it would windmill, not thrust) — a physical sub-idle limit. The matcher reports SUB-IDLE
  there rather than quoting a negative-thrust point (and rather than tripping the shared `_score`
  efficiency cascade, which degenerates at zero net thrust). At the design flight this is
  `Tt4 ≈ 440`.

The window widens at low ram (near-static): lower `pt` unchokes the nozzle at a higher `Tt4`, so
the whole idle-descent / ground-idle regime lives on this branch — which is exactly where a real
engine spends its subsonic-nozzle time.

---

## Reduce-to-prior contract (the spine)

**The choked path is left LITERALLY unchanged.** `OffDesignMatcher.match` computes the choked
branch exactly as rung 31 did, checks whether the rebuilt nozzle is choked, and only **then**
dispatches to `_match_subsonic` if it is not. So:

- Every choked operating point (design and all off-design points above unchoke) is **bit-for-bit
  rung 31** — the rung-31 and rung-32 test suites, which exercise the same `match()`, still pass
  unchanged (14/14). This is the reduce gate: rung 33 adds a branch, it does not perturb the old one.
- At the unchoke **boundary** the two branches meet continuously: the last choked point has
  `M9 = 1`, the first subsonic point has `M9` just below 1, and `π_c, τ_t` are continuous across
  the switch (gate 2).
- The default `build_turbojet(…).run(…)` design path is untouched, so the production cycle stays
  bit-for-bit rung 6 and the rungs-7+ invariant holds (gate 7).

---

## Verification gates (`tests/test_rung33.py`)

1. **REDUCE / CHOKED BIT-FOR-BIT.** Matching at the design point returns `π_c = 10`,
   `branch = "choked"`; choked off-design points stay choked (dispatch only fires below unchoke).
   The unchanged rung-31/32 suites are the bit-for-bit witness.
2. **DISPATCH + BOUNDARY CONTINUITY.** Choked above unchoke, subsonic below; `M9` passes through 1
   continuously and `π_c, τ_t` do not jump across the boundary.
3. **THE RUNG (CPG `τ_t` VARIES).** On a CPG gas the subsonic `τ_t` varies measurably with throttle
   (and monotonically, rising toward 1), while the **choked** branch on the same CPG gas holds
   `τ_t` constant to machine zero — the inversion of rung 31.
4. **NON-TAUTOLOGICAL ANCHOR.** An **independent CPG closed-form solve of `(★★)`** reproduces the
   shipped solver's operating point `(π_t, π_c, τ_t, M9)` to machine zero. This is the rigorous gate,
   and it matters here specifically: gate 1 (reduce-to-design) is a *choked* point that returns
   **before** the subsonic dispatch, so the subsonic solve — the actual new code — has no
   reduce-to-prior of its own. The independent path is pure calorically-perfect algebra
   (`τ_t = 1 − η_t(1 − π_t^((γ−1)/γ))` → shaft → `π_c = [1+η_c(τ_c−1)]^(γc/(γc−1))` → `M9` from the
   isentropic `pt9/p0` → the `MFP(M9)`/`MFP*` closed forms → root-find `π_t` on the mass balance)
   with **no `_sonic_throat` and no `Nozzle.apply`**. Two genuinely separate code paths, one
   operating point — it ties the deep-subsonic *values* to the textbook (a 1% `π_c` drift in the
   shipped solve is caught here, where gates 1/2 miss it). The isentropic / `MFP(M9)` identities the
   shipped point satisfies are kept only as a secondary consistency check, not the anchor.
5. **ENVELOPE.** The subsonic branch is monotone (`π_c, M9`, specific thrust fall with `Tt4`),
   bounded above by unchoke and below by thrust-neutral idle (SUB-IDLE raised, not force-fit).
6. **HOMOGENEITY (the framing).** Scaling `p0` leaves the subsonic ratios `π_c, τ_t, M9` invariant:
   the coupling is to `π_c` via `pt9/p0`, not to the ambient pressure.
7. **CYCLE UNTOUCHED / MAP OUT OF SCOPE.** The default design run is bit-for-bit rung 6; the rung-32
   `MapMatcher` (which overrides `match`) does **not** inherit the subsonic branch.

---

## Concessions

- **Component efficiencies held at design.** As on the choked branch, `η_c, η_t, η_b, π_b, π_n`
  are kept at their design values along the subsonic running line. The map curvature is rung 32;
  **subsonic + component map is out of scope** — `MapMatcher` overrides `match` and stays on its
  choked-only path (it flags `nozzle_choked = False` below unchoke without re-solving). Threading
  the map through the subsonic solve is a further seam.
- **NGV assumed choked throughout.** The NGV is the last throat to unchoke; on the reported
  subsonic envelope it is still choked (asserted, not modeled as a vane passage) — the same
  concession rung 31 makes.
- **Thrust-neutral idle is the modeled lower bound.** Below it the engine does not self-sustain
  useful thrust; spool-down transient / windmilling dynamics are a separate (time-dependent) seam.
- **Diagnostic beside the cycle.** The production run stays on the specified-`π_c` design path;
  off-design (choked or subsonic) is a separate entry point.

---

## Anchor

`docs/plans/rung33-anchor-subsonic.md`. Two-part, mirroring rung 31:
- **The method** — Mattingly *Elements of Propulsion* Ch. 8's **dual matching mode** (choked vs
  subsonic nozzle); the subsonic branch replaces the sonic `MFP*` with the compressible-flow
  `MFP(M9)` at the told back-pressure `p9 = p0`. Same textbook family as the rung-2/30/31 anchors.
- **The CPG independent-solve gate** (gate 4, the rigorous anchor): an **independent** closed-form
  solve of `(★★)` on a self-consistent calorically-perfect gas — pure algebra, no `_sonic_throat`,
  no `Nozzle.apply` — reproduces the shipped solver's `(π_t, π_c, τ_t, M9)` to machine precision.
  Two separate code paths onto one operating point: the non-tautological check that the
  `_sonic_throat`/`Nozzle` matching *is* the textbook subsonic ratio method on the gas the textbook
  assumes (and the only anchor tying the deep-subsonic values to anything, since gate 1 never
  executes the subsonic path).
