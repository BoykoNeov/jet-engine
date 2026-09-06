---
name: rust-port-slice-ag-step2
description: "Slice AG step 2 — a gate grid inherited across the boundary where its defect class comes into existence, and a single-line sweep that cannot score an N-site value"
metadata: 
  node_type: memory
  type: project
  originSessionId: 4579ae15-6c80-41e6-9e5b-c275bde7ce6b
  modified: 2026-09-06T19:55:13.920Z
---

Slice AG step 2 (rung 75's `_rhs_laws`, `_with_windup`, `_windup_march` → `rust/src/anti_windup.rs`,
353 → 601 lines; `rust/tests/slice_ag_laws.rs`, 11 gates). Three findings, one of them about my own
instrument.

**The lesson: a test grid is inherited from the previous slice, and a defect class can come into
existence exactly at the boundary it is carried across.** The gate whose whole subject is *which
clock does each row divide by* ran on a grid where all five clocks were `0.05` — so any permutation
of the five divisors passed every assertion in it. A mutation sweep proved it by surviving
`F ÷ tau_gov`. The grid came from slice AF, and it was harmless there **because rung 74's laws
return targets and solved values: no clock appears in their algebra at all.** Rung 75 is the first
rung in the ladder whose laws divide, so it is the first rung where the defect is even expressible,
and the file that inherited the grid was the first file that could be blinded by it. It was also a
precondition for the NEXT step, not a tidy-up of this one: step 3's `_jac4` writes `−1/tau` on the
diagonal, and this rung's pre-registered trap (`tau_t` added to `taus`) is detectable only if
`−1/tau_t` and `−1/tau_f` are different numbers. **How to apply:** when seeding a gate file from the
previous slice's, ask what the new rung's algebra does that the old one's did not, and whether the
copied constants can still discriminate it. Distinct-by-default costs nothing; equal-by-inheritance
costs a whole class of discriminator, silently.

**A mutation sweep scores a LINE; a value written from one source at N sites cannot be scored by a
one-line injection.** Rung 75 copies `_ic_cap` from `self` at three sites (`at_lever`,
`_shared_rig`, `_windup_march`), and the march reaches its sibling through the second — so dropping
the march's own line is a foregone `SURVIVED` that says nothing about gate power, in the port or in
Python. Removing all three at once kills in both binaries. My draft had called the single line
load-bearing; the sweep refuted the attribution and the N-site injection restored the claim in its
true form (one live set, three faithful copies). Ask *can anything see this VALUE*, not *this line*.

**And the port reproduced the rung's headline by FAILING.** Three march gates failed inside rung
74's own shipped refusal — `demand × applied` has no interior equilibrium — because the `none` arm
written as a control is the half that cannot run one rung down. Read as a measurement, the 2×2 is
the rung: the tracking term is what gives the masked leg an equilibrium to converge to. Step 1's
lesson, recurring inside the same slice: read the failure before fixing the gate.

Related: [[rust-port-slice-ag-step1]], [[rust-port-slice-ag-preflight]],
[[instrument-fed-by-what-it-certifies]], [[rust-port-copy-vs-rederivation]],
[[rust-port-oracle-cannot-see-a-missing-gate]], [[rung75-antiwindup-device]].
