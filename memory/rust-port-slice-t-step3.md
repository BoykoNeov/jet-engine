---
name: rust-port-slice-t-step3
description: "Slice T step 3 (rung 48's 16 gates) — a defect can be invisible because every READER of the quantity is invariant under it, not because the bars are loose; and a registered prediction needs an injection aimed at ITS mechanism"
metadata: 
  node_type: memory
  type: project
  originSessionId: 2007a92b-bcce-4a60-abac-505522330831
  modified: 2026-08-19T19:36:01.733Z
---

Rung 48's sixteen gates ported one-to-one, **0 source lines added or changed**, 16/16 first run in
0.49 s. Five injections, four caught. The process lessons:

**A SURVIVING DEFECT IS NOT ALWAYS A LOOSE BAR — CHECK THE PREDICATES, NOT THE MARGINS.**
[[rust-port-slice-t-step2]]'s blindness was quantitative (the tightest bar was 2.19×, so anything
smaller hid under it). This one is structural: the survivor doubles an integrated quantity, and
every reading of that quantity in the WHOLE project — three call sites, enumerated by grep across
both languages — is either `> 0` or a pairwise `<`. Both predicates are invariant under
multiplication by a positive constant, so **no scaling error in it is observable anywhere**. There
is no bar to tighten; only a value oracle can hold it. When an injection survives, ask what the
readers actually assert before assuming the tolerance is at fault.

**A PREDICTION DERIVED FROM A MEASURED MECHANISM IS STILL A PREDICTION.** A shipped measurement said
the dormant fast path returns its argument float-identically, so I asserted the artifact it explains
would vanish in Rust. It did not — the run refuted it in one line, and the artifact reproduces
exactly as the source describes it. Cheap to check, and the reasoning felt like a deduction.

**AN INJECTION THAT FAILS A GATE HAS NOT NECESSARILY TESTED THE CLAIM THE GATE CARRIES — READ THE
PANIC, NOT THE PASS/FAIL.** A pre-registered prediction said six exact-zero assertions pass or fail
as one block. Four injections split their carrier gates every time, which looked like a refutation;
the messages showed every failure fired on a PRECONDITION, never on a zero. Testing the claim needed
an injection aimed at its mechanism (a 1-ulp perturbation of the shared float), and then all six
failed together — the prediction HELD. A subset-of-gates count is not evidence about a claim until
you know which assertion fired.

**AND THAT TARGETED INJECTION'S FIRST TWO SPELLINGS MEASURED NOTHING**, caught by the
[[rust-port-slice-s-step3]] precondition rather than by their green tick: perturbing the value
UPWARD is filtered out exactly by a downstream `retain(c < mf)`. The "exact structural zero" the
port had recorded is **one-sided** — unobservable above, bit-sensitive below. Run the probe before
believing either a pass or a fail.

**A HELPER THAT LOOKS SHAREABLE ACROSS TWO NEIGHBOURING SUITES USUALLY IS NOT.** The two files'
map sets differ by one entry and their settle times by 2×; reusing the neighbour's helper would have
failed nothing and widened the grid silently. [[rust-port-slice-n-step4]]'s lesson, one slice on.
