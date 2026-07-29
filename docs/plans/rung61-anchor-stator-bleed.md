# Rung 61 anchor — STATOR + BLEED together

The two halves of rungs 36/41's standing concession, now both built (rung 53's stator,
rung 42's valve), placed on ONE steady two-spool machine.

**Everything in § Predictions was written down BEFORE a single number was measured.**
§ Measured is filled in afterwards and § Score reconciles the two. That order is the point.

---

## The seam, as it was actually posed

Six specs (53, 55, 56, 57, 58, 59, 60) carry the same sentence:

> *"**stator + bleed together** — rung 53's saturation says the bleed takes over where the
> stator's authority ends."*

That is a **TAKEOVER / SEQUENCING** hypothesis: the stator works until it saturates, then the
valve carries on. It is scored as written (P1). A second, different reading — that the valve
does not extend the stator but **pays its bill** — is scored beside it (P2–P5). Running only
the second would dodge the seam as posed.

## What is NOT available as a finding — rung 60's tautology, in advance

`M_i = T_c − (1/φ_op − v) = 1/φ_s0 − 1/φ_op + v` has an **explicit `+v`** and nothing else in
it moves with the stator. So ANY pair of levers that leaves `φ_op` where it found it hands
back `ΔM_i = v` exactly. Rung 60 already published that value (`docs/rung60-spec.md`, "leg
floors φ ⇒ … = v"), reached by **pinning** `φ`. Bleed-restoration is a **third route to the
same identity** — and the correct reading of that is that rung 60's tautology needs no floor
at all, only restoration. It is a derivation step here, gated for exactness, and **it is not
this rung's headline.**

---

## Predictions (written before measuring)

### P1 — the seam as posed: TAKEOVER. Predicted **REFUTED**, with a direction.

Rung 53's authority ceiling is a **map-validity / loading-peak** object (`solve_n`'s
speed-line bracket at `v_edge`, the incidence peak at `v_peak`), not an energy shortage.
Bleed does not act on either law — it moves `φ_op`. So:

- **P1a.** Opening the valve **raises** `v_edge` on the LP spool (the bracket is reached
  because closing the stator unloads the speed line; bleed pushes `φ_op` back up, away from
  it). Predicted **rise, ≥ 5 %** over `b = 0 → 0.10`.
- **P1b.** Therefore the sequencing picture is **backwards**: the valve does not begin where
  the stator stops. It operates in the SAME range and **extends** the stator's own range.
  "Takeover" is scored REFUTED if P1a's sign holds.
- **P1c.** `v_peak` (the incidence peak, where it is interior) moves **less** than `v_edge`
  — the peak is set by the loading law, the edge by the solve.

### P2 — THE HEADLINE: compensability is SPOOL-DEPENDENT, and the HP branch dies at `π*`.

Define `b*(v)` = the bleed that restores `φ_op` to its bare (`v = 0, b = 0`) value.

- **P2a.** On the **LP** spool `b*(v)` exists and is modest: rung 42 measured
  `Δφ_L ≈ +0.078` at `b = 0.10` and rung 53 measured `Δφ_L = −0.119` at `v = 0.20`, so
  `b*(0.20) ≈ 0.15`. Predicted **`b*(0.20) ∈ [0.12, 0.18]`**.
- **P2b.** On the **HP** spool it is **unattainable near `π*`**. Rung 42's `dφ_H/db` passes
  through zero at `π* = γc^(γc/(γc−1)) = 3.2467` and REVERSES below it, so `b*_HP → ∞` as
  `π_HPC → π*` and is **negative** (i.e. requires a valve that sucks) below. Predicted
  `b*_HP/b*_LP > 10` at design, **diverging** as `Tt4` falls toward the `π*` crossing.
- **P2c.** This is **rung 42's OWN crossing read in a new currency — NOT a fourth independent
  appearance of `π*`.** It is billed as a prediction the composition makes and the plant
  confirms, nothing more.

### P3 — the tautology, gated as an identity and DEMOTED.

Along the iso-`φ_op` locus, to machine precision and independent of shape / gas / throttle:

```
    ΔM_i  = + v                             (exact)
    ΔM_φ  = + v φ_s0² / (1 + v φ_s0)        (exact)
```

Both currencies rise: the compensation **banks** rung 53's split rather than trading it.
Predicted residuals **< 1e-12**. Labelled rung 60's identity by a third route.

### P4 — TWO loci, and the price is coordinate-dependent.

"Restore the point" (`φ_op`) and "restore the reported margin" (`M_φ`) are different
instructions, because the stator moved the floor between them.

- **P4a.** `b*_{M_φ}(v) < b*_{φ}(v)` strictly (the floor DROPPED, so part of the reported
  debit is already gone).
- **P4b.** Their gap **grows with `v`**, and equals the floor motion `v φ_s0²/(1+v φ_s0)`
  converted through `dφ_L/db`.
- **P4c.** So the ladder extends: rung 54 found a constraint's SEVERITY is coordinate-
  dependent, rung 56 a lever's COST. Predicted here: **the PRICE of undoing one lever with
  another** is too.

### P5 — the trade: an OVERSPEED debit converted into a THRUST debit.

Rung 53's stator is thrust-neutral and paid in shaft speed (`N_L` up > 15 %); rung 42's valve
is paid in thrust/TSFC (−10 % / +6.3 % at `b = 0.10`).

- **P5a.** At the compensated point the `N_L` rise is **less than half** the bare stator's at
  the same `v` (bleed restores `φ_op`, hence most of the speed).
- **P5b.** Thrust falls by roughly rung 42's `b*`-interpolated amount (**−12 to −18 %** at
  `b* ≈ 0.15`), and TSFC rises.
- **P5c.** So the pair is not a free lunch in any currency; it **relocates** the bill.

### P6 — a CORRECTION candidate: per-spool cleanliness is NOT preserved.

Rung 53 P5 says the stator is *"a cleaner per-spool DoF than rung 42's bleed valve"* —
`vsv_lp` leaves `φ_HP`/`n_HP` **bit-identical**, while bleed reaches the HP through the shared
`Tt25` energy channel even on a flat map.

- **P6.** Predicted: the **compensated pair is NOT clean**. `Δφ_HP ≠ 0` at the compensated
  point, because the only lever that can buy the LP debit back is the dirty one. **A pair
  inherits the DIRTIER lever's arrow** — so rung 53's cleanliness is a property of the lever
  in isolation and does not survive composition. Predicted `Δφ_HP` of order `+0.008` (rung
  42's design-point figure scaled to `b*`).

---

## Measured

All numbers: CPG gas, `FLIGHT = (250 K, 50 kPa, M0 0.85)`, design `π_LPC/π_HPC/Tt4 =
3/6/1500`, `φ_surge = 0.55`, LP map `flow/press` unless a shape is named.

### The two-axis reduce (before anything else)

`(v=0,b=0)` ⇒ rung 39, `(v=0.15,b=0)` ⇒ rung 53, `(v=0,b=0.08)` ⇒ rung 42 — **all
BIT-FOR-BIT** on 15 matched fields at `Tt4` = 1500 / 1200 / 950. Both silent-failure traps
checked explicitly: `map_lp.vsv == 0.20` under the new MRO, and `at_setting` returns a
`StatorBleedMatcher` carrying `bleed`.

### P2 — compensability across the throttle, both spools (`v = 0.20`)

| `Tt4` | `π_HPC` | `b*_LP` | `b*_HP` |
|---|---|---|---|
| 1500 | 6.0000 | 0.18636 | **none** — valve authority exhausted |
| 1300 | 5.1833 | 0.15118 | **none** |
| 1100 | 4.3892 | 0.12254 | **none** |
| 900 | 3.6303 | 0.10017 | **none** |
| 800 | 3.2683 | 0.09127 | **none** |

`π* = 3.24674`. The HP branch does **not** merely diverge toward `π*` — it is unreachable at
**every** throttle in the choked band. What the stator spends against what the whole valve
range (`b: 0 → 0.45`) returns, in `φ_H`:

| `Tt4` | `π_HPC` | stator spends | whole valve returns | shortfall |
|---|---|---|---|---|
| 1500 | 6.0000 | −0.125684 | +0.026294 | **−0.099390** |
| 1300 | 5.1833 | −0.117298 | +0.020348 | **−0.096950** |
| 1100 | 4.3892 | −0.109970 | +0.013369 | **−0.096601** |
| 900 | 3.6303 | −0.104650 | +0.005944 | **−0.098706** |

Short by **4–17×**, and the shortfall is **throttle-invariant to 3 %**.

### P3 — the two identities (LP, `target="phi"`)

Residuals `≤ 1.2e-11` at every `(Tt4, v)` in {1500,1300,1100} × {0.10,0.20,0.30} **and** on
the flat-η island: `ΔM_i = v` and `ΔM_φ = v φ_s0²/(1+v φ_s0)` (= 2.867299e-02 / 5.450450e-02
/ 7.789700e-02 at `v` = 0.10/0.20/0.30 — **identical at every throttle**).

### P5 — the bill does NOT relocate (the finding this rung is built on)

| `Tt4` | `v` | `b*` | `Δn` stator → comp | retained | `ΔF` stator → comp |
|---|---|---|---|---|---|
| 1500 | 0.10 | 0.09020 | +6.42 % → +4.71 % | 73 % | −0.12 % → −8.91 % |
| 1500 | 0.20 | 0.18636 | +13.06 % → +11.06 % | 85 % | −0.45 % → −18.68 % |
| 1500 | 0.30 | 0.29206 | +19.87 % → **+20.25 %** | **102 %** | −0.96 % → −30.00 % |
| 1300 | 0.20 | 0.15118 | +11.55 % → +9.30 % | 81 % | −0.58 % → −15.91 % |
| 1100 | 0.20 | 0.12254 | +10.19 % → +7.90 % | 78 % | −0.64 % → −14.19 % |
| 1100 | 0.30 | 0.18984 | +15.52 % → +13.36 % | 86 % | −1.05 % → −22.43 % |

### The MECHANISM — the compensated loading, in closed form

`n` solves `τ_c = 1 + (τ_d−1) ψ(φ) n²`. Decomposed at `Tt4` = 1500, `v` = 0.20:

| cell | `b` | `φ` | `n` | `ψ` | `τ_c` |
|---|---|---|---|---|---|
| bare | 0 | 1.00000 | 1.00000 | 1.00000 | 1.40971 |
| stator | 0 | 0.88106 | 1.13062 | 0.78228 | 1.40971 |
| compensated | 0.18636 | 1.00000 | 1.11060 | **0.66000** | 1.33353 |

`ψ_comp = 0.66000` is **exactly** `1 − v(1+l) = 1 − 0.20·1.7`; at `v = 0.30` it is exactly
`0.49000`. Verified against the closed form `ψ_comp = base(φ_bare) − v(1+l)φ_bare` on **all
five shapes × 2 throttles × 3 settings: residuals ≤ 1.2e-11.**

So the stator's `φ`-drop was carrying a **rebate**: `base(φ)` rises as `φ` falls
(+0.0833 here), partly paying for the swirl the stator removed. Restoring `φ` **forfeits the
rebate**, so the compensated point is *more* unloaded (0.660 < 0.782) than the stator-only
point. `Δn_comp` factorises into a **loading term** (+23.09 %) fighting a **demand term**
(−9.77 %, bleed's lower `τ_c`); at `v = 0.30` the loading term (+42.86 %) beats the demand
term (−15.83 %) and the compensated machine **overspeeds the bare stator**.

### P1 — the seam as posed, and the superposition table

`authority_ceiling` at four valve positions, LP (`peak_interior=False` throughout — rung 53's
concession holds on these shapes, so `v_edge` is the admitted **artifact** edge):

| `Tt4` | `b`=0 | 0.05 | 0.10 | 0.15 |
|---|---|---|---|---|
| 1500 `v_edge` | 1.2400 | 1.2000 | 1.2000 | 1.1600 |
| 1500 span `M_i` | 0.20856 | 0.20268 | 0.19787 | 0.19079 |
| 1000 `v_edge` | 2.5200 | 2.4400 | 2.3600 | 2.2800 |
| 1000 span `M_i` | 0.38133 | 0.36195 | 0.34206 | 0.32168 |

Opening the valve **shrinks** the stator's remaining authority — monotonically, at every
throttle. `M_i(v=0)` **rises** with `b` (0.818 → 0.927 at `Tt4` = 1500) by more than the peak
rises: the valve **pre-spends** what the stator was going to buy.

The artifact-free version — the four-cell superposition table in `M_i`, five shapes × 2
throttles × 3 `(v,b)` pairs (30 rows): **interaction / (sum of credits) ∈ [−2.28 %, +0.45 %]**,
sign **shape-dependent** (positive on `flow/press` and `steep`, negative on `press/flow`,
`tilted`, `flat-eta`). The same table in the **costs** gives an interaction that is
**positive in every one of the 6 measured rows** (`Δn` +0.21 % … +1.68 %, `ΔF` +0.08 % …
+0.60 %).

### THE PRICE COLLAPSES ON `(1+l)` — `b*/[v(1+l)]` across the five shapes

`l` spans 0.7 → 1.2 (a 71 % range) across the disclosed shapes. At fixed `(v, Tt4)`:

| `Tt4` | `v` | flow/press | press/flow | tilted | steep | flat-η | spread |
|---|---|---|---|---|---|---|---|
| 1500 | 0.10 | 0.5306 | 0.5351 | 0.5325 | 0.5363 | 0.5296 | **1.25 %** |
| 1500 | 0.20 | 0.5481 | 0.5598 | 0.5531 | 0.5638 | 0.5460 | **3.21 %** |
| 1500 | 0.30 | 0.5727 | 0.5970 | 0.5831 | 0.6085 | 0.5694 | 6.68 % |
| 1300 | 0.20 | 0.4446 | 0.4433 | 0.4430 | 0.4448 | 0.4493 | **1.42 %** |
| 1200 | 0.20 | 0.4002 | 0.3949 | 0.3965 | 0.3953 | 0.4080 | **3.27 %** |
| 1200 | 0.30 | 0.4142 | 0.4141 | 0.4128 | 0.4166 | 0.4216 | **2.11 %** |

The price's **entire shape-dependence is the map's own loading slope** `(1+l)`. The
coefficient itself (0.38–0.61) rides on `v` and the throttle and is disclaimed.

### The COST interaction, on all five shapes — and a new machine-zero

`i_n` (speed) is **positive in all 30 rows**, +0.19 % … +1.81 %. `i_F` (thrust) is positive
on the four shaped maps (+0.02 % … +0.70 %) and **EXACTLY `0.0`** — a literal float zero, all
six rows — on the **flat-η island**, where `i_n` stays nonzero (+1.9e-03 … +1.5e-02).

So the pair's **thrust** interaction is entirely **η-mediated and switchable off**; its
**speed** interaction is not. The same split rung 53's P5 found for the stator's inter-spool
arrow, now for the composite's cost.

### The compensable ceiling — NOT claimable (the check that killed it)



Walked `v` up at the design throttle until `b*` no longer exists:

| shape | `l` | `1/(1+l)` | last `v` that compensates | first that fails | ratio |
|---|---|---|---|---|---|
| flow/press | 0.70 | 0.5882 | 0.41 | 0.43 | 0.71 |
| press/flow | 1.00 | 0.5000 | 0.35 | 0.37 | 0.72 |
| tilted | 0.85 | 0.5405 | 0.37 | 0.39 | 0.70 |
| steep | 1.20 | 0.4545 | 0.31 | 0.33 | 0.70 |
| flat-eta | 0.70 | 0.5882 | 0.41 | 0.43 | 0.71 |

This *looked* like a derived ceiling scaling as `1/(1+l)`. **It is not, and the check that
killed it was run before publishing.** `_B_CAP = 0.45` is a constant introduced by this rung
(rung 42's own bound is `b < 0.5`), so the walk was re-run at three caps:

| cap | flow/press | press/flow | tilted | steep | flat-η |
|---|---|---|---|---|---|
| 0.35 | 0.561 | 0.580 | 0.574 | 0.550 | 0.595 |
| 0.42 | 0.663 | 0.660 | 0.648 | 0.682 | 0.663 |
| 0.45 | 0.697 | 0.700 | 0.685 | 0.682 | 0.697 |

(ratio of the last compensable `v` to `1/(1+l)`.) **The ratio moves with the cap**
(0.55 → 0.70), so the ceiling's *level* is where the walk was truncated, not a property of
the plant — **the ceiling is NOT claimed.** What survives the check is the price scaling
above (`b* ∝ v(1+l)`, cap-free), from which any cap induces a ceiling `∝ 1/(1+l)` trivially.
The death reason is `b ≥ cap` at every throttle from 1500 down to 900 on two shapes — rung
42's **envelope** guard never binds first, so this is a valve-authority statement throughout.

### P4 — two loci

| `Tt4` | `v` | `b*_φ` | `b*_{M_φ}` | gap | floor motion |
|---|---|---|---|---|---|
| 1500 | 0.05 | 0.04451 | 0.02381 | +0.02070 | 0.01472 |
| 1500 | 0.10 | 0.09020 | 0.04952 | +0.04068 | 0.02867 |
| 1500 | 0.20 | 0.18636 | 0.10686 | +0.07950 | 0.05450 |
| 1500 | 0.30 | 0.29206 | 0.17273 | +0.11933 | 0.07790 |
| 1200 | 0.05 | 0.03265 | 0.01192 | +0.02073 | 0.01472 |
| 1200 | 0.10 | 0.06612 | 0.02509 | +0.04104 | 0.02867 |
| 1200 | 0.20 | 0.13606 | 0.05516 | +0.08090 | 0.05450 |
| 1200 | 0.30 | 0.21123 | 0.09035 | +0.12088 | 0.07790 |

`b*_φ > b*_{M_φ}` strictly; the gap grows with `v`; and the **gap is throttle-invariant to
1.5 %** while each price separately moves by 27–37 %.

### P6 — the other spool

`Δφ_HP`: stator alone **+0.000000e+00** (rung 53's exact zero reproduced); compensated
**+1.553991e-02** at `v` = 0.20. On the **flat-η island** — rung 53's own zeroing control —
the compensated arrow is **+1.571717e-02**, i.e. it **survives**.

---

## Score

| | prediction | outcome |
|---|---|---|
| **P1a** | `v_edge` **rises** ≥ 5 % with `b` | **WRONG SIGN.** It *falls*, monotonically, at every throttle. |
| **P1b** | "takeover" REFUTED | **CONFIRMED** — and harder than predicted: the valve does not begin where the stator stops, it brings that stop **nearer**. The two are substitutes on one budget (credits additive to ≤ 2.3 %), not a sequence. |
| **P1c** | `v_peak` moves less than `v_edge` | **VACUOUS on these shapes** — the peak is not interior, so `v_peak == v_edge`. Not scored. |
| **P2a** | `b*(0.20) ∈ [0.12, 0.18]` | **MISS on the level** (0.18636 at design), right at the top of the band and outside it. Reachability confirmed. |
| **P2b** | `b*_HP/b*_LP > 10`, **diverging** toward `π*` | **MECHANISM RIGHT, SHAPE WRONG.** Not a divergence: unreachable at *every* throttle, by a **throttle-invariant** shortfall of ≈0.097. The right statement is that the two levers **do not span the same space**, not that one ratio blows up. |
| **P2c** | do not bill `π*` as a fourth appearance | **HELD** — and it turned out `π*` is not the operative boundary at all. |
| **P3** | both identities exact, `< 1e-12` | **CONFIRMED** (≤ 1.2e-11 with the bisection tolerance `1e-11`; tightening `_B_TOL` tightens it). Demoted as planned. |
| **P4a/b** | `b*_{M_φ} < b*_φ`, gap grows in `v` | **CONFIRMED**, plus an **unpredicted** invariance: the gap is throttle-invariant to 1.5 %. |
| **P5a** | overspeed retained **< 50 %** | **REFUTED.** Retained **73–102 %** — and at `v` = 0.30 the compensated point overspeeds the bare stator. This refutation became the rung. |
| **P5b** | thrust −12 to −18 % at `b*` | **CLOSE, slightly outside** (−18.68 % at `b*` = 0.186). |
| **P6** | pair not clean, `Δφ_HP ~ +0.008` | **CONFIRMED**, level ~2× predicted (+0.0155 at `b*` = 0.186, consistent with rung 42's +0.008 at `b` = 0.10) — **and sharpened**: the arrow survives the flat-η control, so it is not switchable off. |

**Two predictions carried the rung by being wrong.** P5a's refutation forced the mechanism
probe that produced the exact `ψ_comp` closed form, and P1a's wrong sign turned "does the
valve take over?" into "the valve competes for the same budget."
