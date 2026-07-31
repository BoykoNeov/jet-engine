---
name: rung67-cascade-a
description: "Rung 67 (cascade A) — one scalar P sets both the ringing window and the damping, so the mode is admissible but unobservable; three near-misses that would each have killed or faked the rung"
metadata: 
  node_type: memory
  type: project
  originSessionId: 31fefe7b-d550-42e7-acb1-0bf2465cea7c
  modified: 2026-07-31T12:40:33.264Z
---

Rung 67 shipped 2026-07-31: rung 47's lagged `Tt4` governor beside rung 65's lagged φ valve —
two loops on **two** variables, the inverse of [[rung66-two-lag-cascade]]. `P = R_q·C_g < 0`
(measured −2.0e−2) kills the degeneracy, so the pair buys AUTHORITY (erosion 0.93×/1.26× vs
rung 66's 38×) and opens a ringing window `ρ + 1/ρ < 2 + 4|P|` in the clock ratio.

**The headline inverted mid-rung, and that is the lesson.** The seam promised "an oscillatory
mode is admissible" and that is true — but `ζ = 1/√(1+|P|)` and `T = 2πτ/√|P|` contain **no
time constant**, so the same scalar that opens the window damps it: `ζ = 0.990`, `T = 44τ`,
dead in `e^−44` per period **at every clock pair**. Admissible ≠ observable, and no bandwidth
choice can change it. I nearly wrote the seam's promise as the finding.

**Three near-misses, each of which would have killed or faked the rung:**

1. **The § 0 pre-check earned its keep twice.** Rung 50's assert calls the governor's window
   "post-ramp by construction" — true only at rung 46/47's own redline. Scheduled fuel drives
   *instantaneous* Tt4 to ~1900 K, so a lower `Tt4_max` engages early and the windows overlap
   at all 15 corners. Had they not, this was a negative doc.
2. **A base-point error that looked exactly like the null result.** Measuring `C_g` at `g = 0`
   (the unclipped schedule) put the valve command hard on `b_max`, so both sides of the central
   difference returned the stop and `C_g` read **exactly 0** — i.e. "the loops are independent."
   It is saturation. The base point must be the APPLIED fuel. **A zero cross-gain is a
   saturated actuator or a missing `_b_state`, never a decoupled loop.**
3. **An instrument defect that reads as slow convergence.** Copying rung 66's `_violation`
   break (`s > s_hi`) drops the whole final cell when float accumulation lands past `r`.
   Immaterial on an early-ramp currency (φ), worth `ds×490` on one whose integrand PEAKS at the
   limit (Tt4): 2.8 % monotone drift over 8× `ds` with increments refusing to halve. Fixed by
   interpolating the straddling cell; the credit ratio was stable either way, so no published
   number moved.

**Prediction-setting lesson, two rungs running:** both "designed to fail" predictions HIT
again. They are not aggressive enough. What *did* produce the surprise was registering **both
branches** of one prediction — P3 came back SPLIT (rung 66's withheld-fuel spread collapses
84 % → 0.014 %, so it WAS the zero eigenvalue; its violation-integral spread survives
40.7 % → 45.5 %, so that half was ordinary transient sensitivity). Use two-branch registration
more; it is the device that cannot be narrated after the fact.

See [[always-commit-and-push]], [[claude-md-is-a-reference]] (the byte budget was bumped for
this row, with the reason named in the guard).
