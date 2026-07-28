---
name: rung53-variable-stator
description: "SHIPPED rung 53 = the variable stator; the first FLOOR-moving lever, and the headline is a general law — a margin is a DISTANCE, so when the lever moves the boundary the distance is coordinate-dependent; BOUNDS the phi-currency of rungs 36-52"
metadata: 
  node_type: memory
  type: project
  originSessionId: a142962b-e6ae-45e2-8420-5ee5d18bd505
  modified: 2026-07-28T03:24:46.303Z
---

**SHIPPED rung 53 (2026-07-28) = the VARIABLE STATOR** — rung 42's own named other half
("bleed moves `phi_op`; it does NOT move the stall floor — that is the variable-stator half of
the seam"). Chosen over rung 52's named seam (the two-lag cascade) deliberately: rungs 46–52
were seven consecutive fuel-side-limiter rungs and the instrument had become the subject, so the
pick was the lever that adds a new *mechanism* instead of a new *instrument*. See
[[rung52-asymmetric-lag]], [[rung42-interstage-bleed]].

**HEADLINE — a general law, not a device study** (the advisor's redirect, and it was right):
*a surge margin is a DISTANCE to a boundary; when a lever moves the boundary that distance is
COORDINATE-DEPENDENT — two reference-free margins vanishing on the SAME boundary can disagree on
whether the lever helped, and only the coordinate whose boundary is FIXED measures a margin.*
Closing the stators SHRINKS the `φ`-margin (−0.327) while GROWING the incidence margin (+0.370
= the closed form `1/(2+l)`). `T_c` is blade metal, so incidence is the fixed-boundary
coordinate; `M_φ` becomes a mixed measure the moment the floor moves.

**Two things that made it a rung rather than an observation:**
1. **The split PROVABLY requires a moving floor.** `sign(dM_φ/dx) = sign(φ' + v'φ_surge²)`,
   `sign(dM_i/dx) = sign(φ' + v'φ_op²)` — at `v'=0` they differ only by the strictly positive
   Jacobian `1/φ_op²`, so a floor-fixed lever *cannot* split them. That **BOUNDS** rungs 36–52's
   `φ`-currency (licensed by derivation now, not assumption) rather than refuting it. In general
   they split iff `−φ_op'/v' ∈ (φ_surge², φ_op²)` — an interval **whose width is the open margin**.
2. **Zero new constants.** Both channels derived: `ψ` gains `−v(1+l)φ` from Euler pre-swirl with
   `t₂ = l/(1+l)` read off the map's OWN rung-34 slope; the floor law `φ_s0/(1+vφ_s0)` from
   `T_c = 1/φ_surge` read off rungs 36/41's OWN imposed floor.

**Method lessons worth keeping:**
- **The advisor's blocking check nearly killed the headline and instead hardened it.** It asked
  whether the throttle alone could split the currencies at `v=0` (since `M_i` is a monotone
  reparameterisation of `M_φ`). Working it analytically produced the Jacobian proof — which is
  now the rung's strongest claim. *Answer the "could this mechanism be wrong" question with
  algebra before running anything.*
- **Refuse vote-counting across currencies.** A third currency (`SM_N`) also sided with
  incidence, and the tempting line was "2 of 3 say it helps." The advisor caught that this is
  exactly what the rung's own law forbids: `SM_N`'s boundary *also* moves, so its agreement is
  **unexplained by the law** and is reported as measured, not as evidence.
- **Two confidence levels in one section, stated explicitly.** The currency result is coordinate
  algebra (rides on no magnitude); the constant-incidence schedule's numbers are model-bound
  (holding design incidence costs `N_L` +26 % because the lumped one-stage map has no stator-row
  flow capacity and no stage stack to rematch — the channels that carry the real benefit).
- **A mid-build hypothesis raised and refuted:** an "authority turning point" at `D_v = 1+l`.
  The incidence benefit only saturates asymptotically; `solve_n`'s bracket is hit first. The
  bracket failure that suggested it was my own ladder bug. Recorded in the anchor, not promoted.
- **The reduce came out STRONGER than planned**: because the stator adds no closure (P1 — it
  enters through `solve_n` alone, so it is a SPEED lever, `dm/dv` a machine zero at design
  because the η-island is stationary there), `match` is **not overridden at all** and the maps
  are the same objects — an IDENTITY of code path, not rung 42's dispatch.
- Unpredicted: the stator is **thrust-neutral** (specific thrust flat to <0.13 %, peaked at
  `v=0`) and its cost currency is **shaft speed** (+19 %) — the opposite of the bleed valve. And
  the LP needs **6.7×** the HP's stator authority, so the schedule's size inherits rung 41's
  exposure split.

**Next seam named:** the **stator-row FLOW CAPACITY** channel (refused — needs a new
area-per-setting constant); then a stator schedule `v(n)` on the *transient* plant (the first
lever that could move the wall *during* an accel), and stator + bleed together.
