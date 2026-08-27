---
name: rust-port-slice-z-step5
description: "Slice Z step 5 — the bar written to repair one vacuity exposed a second in the same gate on its first run, and two 'dead' branches turned out to be unobservable rather than unexercised"
metadata:
  node_type: memory
  type: project
---

Slice Z step 5 (2026-08-27) closed the slice: `tests/slice_z_dispatch.rs`, **8 gates, 1.22 s**,
plus a `CensusZ` counter pair in `src`. Nothing in the file reads a golden, so regenerating the
oracle cannot make any of it pass or fail. The blind-spot list was MEASURED (step 3's injections,
re-run at step 4 with the oracle) and came to exactly the four the pre-registration named.

**The leading lesson is a chain of two.** Mutating my own gates (11 mutations) found that
`assert_eq!(r_none, 0.0)` could not see the branch it named: `gains` returns a **central
difference**, and the central difference of ANY constant is zero, so the assertion proves the
empty-`caps` arm returns a CONSTANT and never that the constant is `0.0`.

**Then the bar written to keep that honest failed on its first run.** The repair was
`assert!(r_accel != 0.0)` — *if the accel arm is dormant too, `r_none == 0` is a property of the
DIFFERENCING, not of the branch*. It measured `r_accel = 0` **exactly**: the exhibit used an accel
schedule with a 10 % margin, whose cap sits ABOVE the scheduled fuel at a mid-ramp point, so
`max(0, mf_sched − cap)` clamped to zero on both sides of the difference. **The arm the gate
existed to exhibit was being exhibited DORMANT**, and the only assertion covering it passed solely
because the *reference* arm was non-zero.

**How to apply:**
- **A did-it-move bar must itself be checked that it CAN fire**, or it is one more thing that reads
  as coverage. Write the bar, then confirm it goes red on the dormant case before trusting it.
- **When a reader returns a DERIVATIVE, no assertion on it can pin a constant.** Ask what the
  instrument can physically distinguish before naming the gate after a literal.
- **A mutation harness is code too.** One row here bound an unused local — a mutation that does not
  mutate, whose "survives" said nothing. Re-spell and re-run.
- **Ask of each dead branch a SECOND question the pre-registration did not: can it change a value
  at all?** Two of five "dead" arms here are **unobservable**, not merely unexercised — deleting
  them changes no output on any input. `window`'s `P == 0` guard is inert in Rust (`2π/0.0` is
  already `+inf`) and load-bearing only in Python, where the division RAISES; `sign_changes`'s
  `peak <= 0` return is inert in BOTH. No gate can pin those; only a counter could, and a counter
  on a value-inert branch tests the source text. State the VALUE the port owes and say so.
- **Count what no value can show.** Which of two interchangeable solvers ran is invisible by
  construction (0 of 64 gates, 0 of 35 335 keys), so a thread-local counter in `src` is the gate —
  and it needs BOTH halves (`inline > 0` and `jfp == 0`), because a one-sided version passes if the
  march never runs at all.
- **A discard needs a witness one rung down.** "The lag was ignored" and "the lag never arrived"
  look identical from outside; running the same channel where the carrier IS read is what separates
  them.

Related: [[rust-port-slice-z-step3]], [[rust-port-slice-z-step4]], [[rust-port-slice-w-step5]],
[[rust-port-slice-y-step5]], [[rust-port-ported-test-vacuity]], [[rust-port-slice-v-step4]].
