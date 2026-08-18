---
name: rust-port-slice-r-step3
description: "Slice R step 3 (rung 44's 6 gates + slice L's last deferral) — an invariance gate is satisfied BEST by deleting the variable it varies, and an injection harness needs its own baseline to pass"
metadata: 
  node_type: memory
  type: project
  originSessionId: 90ca045f-9f35-4f45-8ace-04b125ccc0a7
  modified: 2026-08-18T13:28:11.982Z
---

Phase 6 slice R step 3 (`tests/test_rung44.py` → `rust/tests/rung44.rs`, **607** lines,
counted after the last edit) shipped 2026-08-18: **9 `#[test]` fns for an 8-function source,
9 run / 0 failed**, no `src/` edit — verified by an empty `git diff` on `src/` AFTER the harnesses,
which had written to that file thirteen times. Crate **709 run / 0 failed**. The ninth is slice L's LAST deferral, `test_reduce_transient_untouched_by_surge_line_bit_for_bit`,
discharged into `rung44.rs` (not `rung40.rs`) because what it gates — a surge line left UNREAD by a
transient — is rung 44's subject; `rung41.rs`'s roster goes **11 → 12** and the slice-L IOU is
CLOSED. Step 4 (the oracle) is all that remains of slice R.

**AN INVARIANCE GATE IS SATISFIED BEST BY DELETING THE VARIABLE IT VARIES.** Rung 44's headline is
that the surge excursion is `rho`-INVARIANT, and the gate asserts a spread under 5 % over a 25×
sweep. **Deleting `rho` from the shipped marcher outright sends that spread to exactly ZERO — it
passes the bar more comfortably than the truth does.** Invisible to all nine gates in the file, and
to the bare-math reference's own copy of the same leg; what caught the shipped one was a gate a rung
away, aimed at something else (`rung40.rs`'s marched-threshold scope gate). **A gate certifying a
variable POWERLESS cannot distinguish powerless from ABSENT, so it needs a second bar saying the
variable is READ** — both legs now carry `assert!(lo < hi)` beside the spread, and re-injection puts
each defect back on the gate whose claim it breaks. Generalises [[rust-port-slice-r-step2]]: there
the question was WHICH assertion discriminates, here it is whether the assertion's own DIRECTION
can. Related: [[rust-port-ported-test-vacuity]].

**AN INJECTION HARNESS NEEDS ITS OWN BASELINE TO PASS BEFORE ANY ROW IT PRINTS IS EVIDENCE.** Mine
produced two false readings before a true one. First it reported every defect as a compile error —
the detector matched cargo's own `error: test failed` line. Then, fixed, it reported a FAILING
baseline and three defects landing on a gate they do not touch: the revert used `mv "$f.bak" "$f"`,
which restores the backup's mtime, so cargo did not rebuild and each injection ran against the
PREVIOUS one's binary. `cp` + `touch` on revert fixes it. Three of the rows I nearly wrote up were
pure carry-over from the row above.

**A SELF-COMPARISON CANNOT BE VALUE-CHECKED, SO THE REDUCE GATES NEED THEIR OWN INJECTION.** Six of
the nine gates were both bit-checked against PyPy and injection-sized; the other three (the two
reduce gates and the discharged one) compare two runs of the SAME code, so no probe can reach them
and only an injection can. Making the forward closure read `phi_surge` at `1e-9` fails both the
reduce gate and the discharge test. Sorting the gates by WHAT VALIDATES EACH is what exposed the
hole — a green suite hides which of its members nothing is behind. The advisor found this one, and
also that my "no `src/` edit" claim was unverified after the harnesses ran.

**A SIGN GATE NEEDS ITS VALUES CHECKED SOMEWHERE ELSE.** Every assertion in this file is a sign, an
ordering, a monotonicity or a spread — not one pins a number, so a port wrong by a few per cent
passes all nine. A throwaway probe on both sides dumped the **49 floats and 11 discrete flags** the
gates read; the two diff to nothing but `True`/`true`. Worth doing BEFORE believing a suite that
goes green first try — [[rust-port-slice-n-step2]]'s lesson, from the other side.

**AND TWO REPORTED FIELDS ARE READ BY NO RUNG GATE IN EITHER LANGUAGE.** `min_phi_lp/hp` and
`s_lp/s_hp` are emitted by `phi_excursion` and asserted by nothing; corrupting them is invisible to
all 17 rung tests and caught only by step 1's value dump. That IS the right architecture — a dump
is what covers reported-but-unasserted fields — but "the rung suites pass" would otherwise read as
covering the whole record. Detail: `docs/plans/todo-rust-port.md` § 5.15 step 3.
