# Rung 39 — Two-spool + component maps: the cascade acquires a DIRECTION

Rung 38 built the first two-shaft matcher and closed with an explicit prediction about its
own successor:

> "A real compressor's flow capacity is a function of *both* corrected speed and pressure
> ratio … once that dependence is present, the HP spool's mass flow constrains what the LP
> spool can deliver at the interface, and the two spools' operating points DO need a joint
> (usually iterative, `N_L`/`N_H`) solve … 'two-spool + component maps' is the deferred seam
> that would very likely reintroduce the coupling — the rung-31→32 correction, replayed on a
> second shaft."

Rung 39 builds that seam. The prediction is **wrong**, and the way it is wrong is the rung.

---

## The structural novelty: TWO shaft speeds

Rung 32 attached a shaft speed `N` to the single-spool running line (a compressor map is
indexed by corrected speed, so the map is what made `N` computable at all). Rung 38 has two
mechanically independent shafts and computes **no** speed for either — it never needed one,
because isentropic-efficiency knobs are speed-blind.

Putting a `ComponentMap` on each compressor makes `n_L` and `n_H` both real, independent
coordinates, and creates an object no predecessor has: the **speed ratio** `N_L/N_H`. That
ratio is the natural two-spool diagnostic — a twin-spool engine's defining behaviour is that
its two shafts do not throttle together — and it is the structural novelty that carries this
rung, exactly as "`N` enters" carried rung 32.

Each compressor gets its own map (its own island `a,b,c` and its own speed-line loading
`sigma,l`); each turbine gets the same map's near-flat `a_t`. Design references are captured
per-face: `mdot_corr` at station 2 for the LPC, at station **25** for the HPC.

---

## Finding A (the correction) — the map opens EXACTLY ONE arrow

### The algebra: `pi_LPC` cancels out of the HP compressor's corrected flow

The HPT NGV choke (rung 38's `A4`) fixes the corrected flow at station 4. Refer it to the HPC
face at station 25:

```
mdot*sqrt(Tt25)/pt25  =  [mdot*sqrt(Tt4)/pt4] * (pt4/pt25) * sqrt(Tt25/Tt4)
```

and `pt4 = pi_b*pi_HPC*pi_LPC*pt2` while `pt25 = pi_LPC*pt2`, so

```
pt4/pt25 = pi_b * pi_HPC                                                   (pi_LPC CANCELS)

=>  mdot_corr,25 = A4 * pi_b * pi_HPC * MFP*(Tt4,f) * sqrt(Tt25/Tt4) / (1+f)          (†)
```

**`pi_LPC` is absent from (†).** Physically: the LP compressor raises the pressure and the
mass flow *proportionally*, so the HP core sees the same **corrected** flow no matter what the
LP spool delivers. There is no `pi_LPC`-dependent loss modeled anywhere between stations 25 and
4, so nothing reintroduces it.

`Tt25` and `Tt3` come from rung 38's **energy** cascade (Step 3/Step 4), which reads no
compressor efficiency at all. So `tau_HPC = Tt3/Tt25` is map-free too, and the HP compressor's
entire map coordinate pair `(m_H, n_H)` — hence `eta_HPC`, hence `pi_HPC` — is a **closed,
self-contained fixed point in `pi_HPC` alone**. It cannot see `eta_LPC`.

The LP face does **not** cancel:

```
mdot_corr,2 = A4 * pi_b * pi_HPC * pi_LPC * MFP*(Tt4,f) * sqrt(Tt2/Tt4) / (1+f)      (‡)
```

`pi_HPC` appears explicitly. So `pi_HPC -> m_L -> phi_L -> eta_LPC -> pi_LPC`: a genuine new
arrow, **HP -> LP**.

### The finding

> **The map does not dissolve rung 38's cascade into a 2×2 solve — it gives the cascade a
> DIRECTION.** Rung 38 had *no* arrow between the two compressor pressure ratios (both leaves
> closed, symmetric). Rung 39 opens exactly **one**: `eta_HPC` reaches `pi_LPC`, while
> `eta_LPC` leaves `pi_HPC` **bit-for-bit unchanged**. The solve stays strictly triangular —
> it merely acquires an order in which the two spools' pressure ratios must be solved
> (**HP first, LP onto it**), which is the *opposite* of the order rung 38's energy cascade
> runs in (`Tt25` before `Tt3`).

This is the **rung-28 shape**: rung 38's *verdict* ("the two compressor pressure ratios are
never bound by a joint 2×2 solve") **SURVIVES** the map; rung 38's own *stated reason for
expecting it to fail* is **REFUTED**. The measured asymmetry, on 4 disclosed shape pairs × 3
throttles (CPG and reacting):

| channel | mechanism | magnitude |
|---|---|---|
| `eta_HPC -> pi_LPC` | (‡): `pi_HPC` is in the LP face's corrected flow | **1.5e-4 … 6.7e-4**, sign **negative** everywhere |
| `eta_LPC -> pi_HPC` | (†): `pi_LPC` cancelled | **EXACTLY ZERO** (`==`) |

### The one channel that DOES re-open it — and why it is weak

With a representative **turbine** map (`a_t != 0`) the closed leaf opens, through a long chain:

```
eta_LPC -> phi_L -> n_L -> N_L -> nu_LPT -> eta_LPT -> Tt5 -> (energy) -> Tt25 -> pi_HPC
```

Measured at **8e-7 … 2.3e-6** — i.e. **two to three orders weaker** than the HP->LP arrow
(ratios 119× to 548× across the shapes). This is not new physics: it is **rung 32's own
sub-finding** ("the turbine is pinned in corrected speed, so `|Δeta_t| ≪ |Δeta_c|`"),
transplanted onto the LP spool. The honest layered statement:

> Compressor maps + design-held turbine efficiencies ⇒ the leaf is **EXACT**. A representative
> turbine map opens it only at ~1e-6, for rung 32's structural reason. **The exact ratio is
> disclaimed** (it rides on `a_t`); only the sign (turbine maps DO open it) and the
> order-of-magnitude weakness are load-bearing.

### Why this is not a tautology

"Add a feedback loop and feedback exists" would be vacuous. The gated claim is the
**asymmetry**, not the existence of coupling: one specified arrow opens with a measured sign
and magnitude, and the other stays shut **bit-for-bit** for a provable algebraic reason.
An implementation that merely iterated all four efficiencies jointly would produce ~1e-15
noise in the closed leaf and could not make this claim at all — which is why the solve is
built triangular (below), so the closed leaf is a **code-level guarantee**, exactly as rung 38's
`_cascade` made its symmetric version one.

---

## Finding B (the structural novelty) — the SPEED SLIP is map content

With two speeds, `N_L/N_H` off design (the "slip") becomes measurable. Two results:

### B1 — `slip == 1` on a CPG gas with flat maps is a STRUCTURAL IDENTITY

With both NGVs choked, `tau_HPT` and `tau_LPT` are pinned by geometry, so `Tt45/Tt4` and
`Tt5/Tt45` are constants. Both shaft balances then read

```
dh_LPC = eta_m*(1+f)*[h_t(Tt45) - h_t(Tt5)]  =  eta_m*(1+f)*cp_t*Tt4*(tau_HPT - tau_HPT*tau_LPT)
dh_HPC = eta_m*(1+f)*[h_t(Tt4)  - h_t(Tt45)] =  eta_m*(1+f)*cp_t*Tt4*(1 - tau_HPT)
```

on a CPG gas — **both proportional to `(1+f)*Tt4` times a pure geometric constant.** With a
flat map `n = sqrt[(tau_c-1)/(tau_c-1)_d]`, and `(tau_LPC-1) = dh_LPC/(cp_c*Tt2)`,
`(tau_HPC-1) = dh_HPC/(cp_c*Tt25)`, so

```
N_L^2 ∝ n_L^2 * Tt2  ∝ dh_LPC   and   N_H^2 ∝ n_H^2 * Tt25 ∝ dh_HPC
=>  (N_L/N_H)^2 = dh_LPC/dh_HPC / [same at design]   —   (1+f) AND Tt4 BOTH CANCEL
```

So the slip is **exactly 1 at every throttle, independent of `f` and `Tt4`** — verified to 9
digits. The two shafts throttle in lockstep, and nothing in the geometry can break it.

### B2 — what breaks it: the map dominates, the gas curve contributes

`slip` is broken by exactly two channels, and the **rung-31-gate-5 mirror** separates them:

| gas / map | `Tt4`=1500 | 1300 | 1100 | 900 |
|---|---|---|---|---|
| CPG, **flat** | 1.000000000 | 1.000000000 | 1.000000000 | 1.000000000 |
| thermally-perfect, flat | 1.000000000 | 0.995767632 | 0.990994820 | 0.985184874 |
| reacting, flat | 1.000000000 | 0.995188254 | 0.989857143 | 0.983508436 |
| CPG, **shaped** | 1.000000000 | 0.980781 | 0.964264 | 0.949717 |

- the **`cp(T)` gas curve** breaks the CPG identity on its own (~1.5% at `Tt4`=900) — the same
  species as rung 31's `tau_t` drift and rung 30's "0.03% is the physics, not error";
- the **map** breaks it by ~5.0% on the same CPG gas — **~3.4× the gas channel, the dominant
  of the two.**

This **inverts rung 32's decomposition — on the CPG gas, and only there.** Rung 32's lesson was
"the *work* is choke-pinned and map-free; the map only *re-labels* it into `pi_c`/`mdot`/`N`."
On a **CPG** gas the slip-deviation does not exist at all without the map — it is identically
zero on the flat map (B1), so there the map is the **sole** channel and genuinely *creates* the
object rather than re-labelling something already there.

**That statement must NOT be made unconditionally**, and the table above is why: on the
**reacting** gas with the **same flat maps** the slip is already 0.9835 at `Tt4`=900 — a 1.5%
deviation with **no map anywhere**. On the real gas the map is therefore the **dominant**
channel (~3.4×), **not the sole** one. The honest scoping:

> **CPG:** map = sole channel (the inversion of rung 32 is clean).
> **Real gas:** map = dominant channel; the `cp(T)` curve alone already breaks the identity.

### B3 — the direction (empirical, shape-robust only)

`N_L/N_H` falls **monotonically** with throttle across all 4 disclosed shape pairs (5.1%–7.5%
at `Tt4`=900): **the LP spool falls away from the HP spool as the engine is throttled back.**
That is the textbook twin-spool behaviour (at idle a twin-spool runs a high `N_H` and a much
lower `N_L`), and it is the reason twin-spool engines are said to self-protect against the
low-power surge margin rung 36 exhibited on a single spool.

**The idle comparison is DIRECTIONAL ONLY.** The modeled window stops at `Tt4`≈700 (nozzle
unchoke), which is nowhere near idle, and real twin-spool slip at idle is far larger than the
5.1%–7.5% measured here. The textbook observation anchors the **sign**; it does **not** anchor
this rung's magnitude, which is disclaimed like every other representative-map number. Nor is
the surge-protection consequence claimed — rung 36's surge machinery is single-spool, and
whether the slip actually protects the LP spool is left to a two-spool surge rung.

**This direction is NOT structural** — unlike B1 there is no cancellation guaranteeing its
sign; it rides on the relative droop of the two maps. It is claimed as **sign-robust across
the representative shapes** (rungs 12–24/32 methodology) with the **magnitude disclaimed**.

---

## The solve: triangular by construction

The structure is load-bearing — it is what makes Finding A's closed leaf exact rather than
1e-15 noise:

```
outer (f, pt4) fixed point                        [rung 38's, unchanged — the shared scalar]
  |
  +-- OUTER turbine-efficiency loop               [INERT when a_t == 0: converges on pass 1]
        |
        +-- Steps 1-2  (*-HP), (*-LP)  -> tau_HPT, Tt45, tau_LPT, Tt5       [geometry]
        +-- ENERGY     LP balance -> Tt25 ;  HP balance -> Tt3              [map-free]
        +-- HP eta loop:  secant on eta_HPC, using (†)  -> eta_HPC, pi_HPC, m_H, n_H
        |                 *** reads NO LP quantity — (†) has no pi_LPC ***
        +-- LP eta loop:  secant on eta_LPC, using (‡)  -> eta_LPC, pi_LPC, m_L, n_L
                          *** reads pi_HPC from the step above — the ONE arrow ***
```

`(†)` and `(‡)` are written in the code in exactly the closed forms above, so the cancellation
is **manifest in the source**, not a numerical coincidence. The efficiency secants are rung
32's (same positive-feedback structure, same tolerance and step cap).

---

## Reduce-to-prior contract (the spine)

- **FLAT maps ⇒ rung 38 `TwoSpoolMatcher.match`.** With `a=b=c=sigma=l=a_t=0` every
  `eta_*_at` returns its base, so both efficiency secants and the turbine loop converge on
  their first pass at the design efficiencies, and the remaining arithmetic is rung 38's
  `_cascade` with two independent sub-expressions reordered. This was targeted, not promised
  (the rung-32 framing allowed a ≤1e-9 fallback for reordered arithmetic) — and it **lands
  bit-for-bit**: gate 1 asserts `==` on `pi_lpc`, `pi_hpc`, `tau_hpt`, `tau_lpt`, `mdot_air`
  and `thrust` on the **reacting** gas across the throttle sweep.
- **`lp_disabled=True` ⇒ rung 31 `OffDesignMatcher`, and with a shaped map ⇒ rung 32
  `MapMatcher`** — both by **exact dispatch** (no LP hardware is ever constructed), extending
  rung 38's wiring. This completes the ladder: `flat+disabled -> 31`, `shaped+disabled -> 32`,
  `flat two-spool -> 38`, `shaped two-spool -> 39`.
- **Cycle untouched ⇒ rung 6 bit-for-bit.** A separate entry point (`TwoSpoolMapMatcher`); the
  default `build_turbojet(...).run(...)` design path is not perturbed.

---

## Verification gates (`tests/test_rung39.py`)

1. **REDUCE — flat maps ⇒ rung 38.** `TwoSpoolMapMatcher` with `ComponentMap.flat()` on both
   spools reproduces `TwoSpoolMatcher.match` (`pi_lpc`, `pi_hpc`, `tau_hpt`, `tau_lpt`,
   `mdot_air`, `thrust`) at design and across a throttle sweep, on the reacting gas.
2. **REDUCE (the ladder) — `lp_disabled` dispatch.** Flat + `lp_disabled` == `OffDesignMatcher`
   (rung 31) and shaped + `lp_disabled` == `MapMatcher` (rung 32), bit-for-bit, by construction.
3. **NON-TAUTOLOGICAL — an independent bare-math CPG two-spool MAP cascade.** No `Gas`,
   `Component`, `ComponentMap` or `TwoSpoolMapMatcher` calls: its own closed-form CPG
   thermodynamics, its own choke bisections, its own speed-line inversions and efficiency
   fixed points, reproducing the shipped `(pi_LPC, pi_HPC, eta_LPC, eta_HPC, n_L, n_H)` to
   machine zero across a throttle sweep. Two code paths, one operating point (the rung-31-gate-2
   / rung-33-gate-4 / rung-38-gate-2 anchor, extended to carry the maps).
4. **FINDING A — the ASYMMETRY.** At a fixed `(Tt2, pt2, Tt4, f)` (rung 38 gate-3's isolation
   protocol, so the outer `f` loop's own disclosed cross-talk cannot confound it), with
   compressor maps only (`a_t=0`): perturbing `eta_LPC` leaves `pi_HPC` **bit-for-bit
   unchanged** (`==`), while perturbing `eta_HPC` **moves** `pi_LPC` (sign **negative**),
   across ≥3 shape pairs × ≥2 throttles, on CPG **and** the reacting gas. Contrast asserted in
   the same test: `eta_HPT`/`eta_LPT` move **both** ratios — so the gate cannot be misread as
   "the spools don't talk."
5. **FINDING A — the weak back-arrow.** With `a_t != 0` the closed leaf **does** open, and its
   magnitude is **≥50× smaller** than the HP->LP arrow at every point (a deliberately loose
   bound: the measured range is 119×–548× and the ratio is **disclaimed**).
6. **FINDING B1 — the structural identity.** On a CPG gas with flat maps, `N_L/N_H == 1` to
   ~1e-9 at **every** throttle, and (the `(1+f)`-cancellation) also under a deliberately
   perturbed `f` — the identity is `f`- and `Tt4`-independent.
7. **FINDING B2 — the mirror + the dominance.** On the **same** flat maps the identity breaks
   on the `thermally_perfect` and reacting gases (the rung-31-gate-5 mirror, asserted
   side-by-side with the CPG constancy), and on the same CPG gas the **map** channel is
   **larger** than the gas channel.
8. **FINDING B3 — direction.** `N_L/N_H` is monotone-decreasing with throttle across ≥3
   disclosed shape pairs. **Magnitude disclaimed** — only the sign is gated.
9. **SCOPE GUARD.** Nozzle unchoke raises rung 38's documented "OUT OF SCOPE" error rather
   than mis-solving (inherited, re-asserted through the map path).
10. **CYCLE UNTOUCHED.** The default design run is bit-for-bit rung 6.

---

## Concessions

- **Representative maps, not hardware maps.** Inherited wholesale from rung 32: the shapes are
  disclosed, every load-bearing claim is verified shape-robust, and every **magnitude** (the
  HP->LP arrow strength, the back-arrow ratio, the slip depth) is **disclaimed**. A real
  hardware/CFD map remains the standing seam.
- **The slip DIRECTION is empirical.** B1 is structural; B3 is not (see B3).
- **The back-arrow ratio is disclaimed** — it rides on `a_t`; only sign + order of magnitude.
- **Both NGVs assumed choked; fully-choked branch only.** Inherited from rung 38 unchanged —
  nozzle unchoke on the LP spool is still the deferred rung-33-shaped follow-on, now with the
  map complication on top.
- **Steady only.** The two-shaft **transient** (two inertias — the two-spool analogue of rung
  34, and now with two *speeds* to be states) needs this matcher first; still deferred, and
  rung 39 is what makes it well-posed (rung 38 could not supply `N_L`, `N_H` at all).
- **One `eta_m` for both shafts; no bypass/bleed/interstage loss/reheat; isentropic knobs
  only.** All inherited from rung 38.
- **No surge line on either spool.** Rung 36's surge machinery is single-spool; a two-spool
  surge margin (and whether the slip *protects* the LP spool at low power, as the textbook
  twin-spool rationale claims) is a natural follow-on this rung deliberately does not make.

---

## Anchor

`docs/plans/rung39-anchor-two-spool-maps.md`. The **method** is rung 32's representative-map
closure applied to rung 38's cascade; the **non-tautological gate** (an independent bare-math
CPG two-spool map cascade) is the rigorous anchor, exactly as for rungs 31/33/38. The slip
direction (B3) is anchored qualitatively to the standard twin-spool observation that the LP
shaft speed falls off far more than the HP shaft speed as a twin-spool engine is throttled
toward idle (CRS Ch. 9); the **magnitude** is this model's, disclaimed.
