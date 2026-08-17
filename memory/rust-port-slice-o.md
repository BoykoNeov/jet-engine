---
name: rust-port-slice-o
description: "Slice O (rung 61, the diamond) — a deferral written into the code the next slice would RUN came due exactly as written, and the defect it named lives in an EDGE rather than a node"
metadata: 
  node_type: memory
  type: project
  originSessionId: 6ecc4aa5-64d2-4910-9c3f-678688d41b4b
  modified: 2026-08-17T16:04:42.495Z
---

Slice O ported rung 61 (`StatorBleedMatcher`, the port's one multiple-inheritance diamond) and
**completed phase 5**. `stator_bleed.rs` + `slice_o_oracle.rs` + `rung61.rs`: **8 256 distinct
keys bit-exact** on the fast arm, **204 on the reacting arm first run**, 23 gates from Python's 18.
The diamond itself was a non-event — § 5.3's pre-flight had already reduced it to two table
pointers ([[rust-port-ladder-architecture]]).

**THE LESSON — A DEFERRAL IS WORTH THE GREP THAT FINDS IT, SO WRITE IT INTO THE CODE THE NEXT
SLICE WILL RUN.** Slice L had left `lp_eta_loop_bleed`'s `solve_n` panicking on a measured
zero-firing rule and written the expiry into `bleed.rs`'s own module note — naming the mechanism
(rung 61's `at_setting` keeps the valve OPEN through rung 54's ceiling walk), the rung, and the
instruction (*"slice O must MEASURE that site, not inherit this paragraph"*). Every clause was
right. But **the oracle panicked first**; the backtrace named the frame, and only then did the
note turn out to have called it. What reaches you is a failure, not a paragraph — which is an
argument for putting the deferral where the next slice's compiler and tests will hit it.

**AND THE DEFECT WAS IN AN EDGE, NOT A NODE.** No rung-42 gate could see it (rung 42's readers
never walk until refusal) and no rung-54 gate could (rung 54's walk never ran on an open valve).
It exists only in the COMPOSITION — the first such defect in the port. What retired it is a
count, `counters::lp_bleed_aborts() > 0`, because what it replaced was a zero-firing count.

**A HAND-TYPED CONSTANT IN A DUMP SCRIPT REPORTED ITSELF AS A PORT DEFECT.** The smoke gate had
11 of 12 values bit-exact and `thrust` off by 1 ULP; chasing it reached `v0`, a *phase-2*
quantity. Cause: the probe built the gas with `R_c = .4/1.4*cp` where the suite computes
`(gamma_c - 1.0)/gamma_c * cp`, and **`1.4 - 1.0` is not the literal `0.4`**. The port's existing
rule (*never retype a decimal, dump the bits*) does not cover this — the defect is UPSTREAM of the
dump. **Build an oracle's INPUTS with the source's own expression, not an equivalent-looking one.**
Third instance of a measuring pass finding a defect in the instrument ([[rust-port-slice-m]]).

**A RESIDUAL THAT DOES NOT CLOSE IS A POPULATION YOU HAVE NOT FOUND.** The read/key reconciliation
pin claimed one duplicated key and failed at 444 vs 384; the missing 60 were a second key emitted
twice, and 60 was exactly `2 gases x 5 shapes x 2 spools x 3 settings`. An approximate bar would
have absorbed all 60 silently. [[rust-port-slice-l-step4]]'s lesson, second instance.

**Also:** the plan's own written prediction that slice O would add a `Descendant::Bleed` variant
was **wrong and corrected before any code** — the carrier already existed one level down, the
same fact that cost slice N two edits ([[rust-port-slice-n-step3]]), for once in the port's
favour. Two injected detectors both fired, and one fired on a physics gate not aimed at it — an
independent witness, the converse of [[rust-port-slice-n-step5]]'s gates that could not see the
defect they named. And `grep '\._[A-Z_]* *='` over the suite, run at PRE-FLIGHT as slice N's entry
prescribes, found the one `m._B_CAP = cap` reach-inside immediately instead of at the last step.
