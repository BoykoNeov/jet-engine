---
name: rust-port-slice-z-step3
description: "Slice Z step 3 — the injection census reported a 0/0 baseline and called all eight injections invisible, including its own controls; and a gate's own doc comment claimed a coverage it did not have"
metadata:
  node_type: memory
  type: project
---

Slice Z step 3 (2026-08-27) ported `test_rung66.py` (15 gates) and `test_rung67.py` (23) to
`rust/tests/rung66.rs` and `rung67.rs`. Both green on the first run: **4.56 s against PyPy's
91.07 s, a measured 20.0×**, so no `slow` marker was introduced.

**The leading lesson is about the INSTRUMENT, not the port.** The injection harness returned a
clean, plausible, entirely worthless table — all eight injections invisible, **including the two
CONTROLS chosen precisely because a ported gate exists for them**. `cargo test` prints
`Running tests\<name>.rs` on **stderr** and `test result: … N passed; M failed` on **stdout**, so
parsing `stdout + stderr` puts every result line BEFORE the target line that names it: the
parser's "which target am I in" variable was `None` throughout, every target was recorded
`0 passed / 0 failed`, and `sum(failed) == 0` for every run.

**Why it was caught:** the harness echoed its baseline, and `{'rung66': (0, 0), …}` beside a suite
just watched go 15/15 was the only line in the run that disagreed with itself.

**How to apply:**
- **A census must clear a COUNTED baseline before it is allowed to conclude anything.** The repair
  is a bar, not a fix: one target per `cargo test`, parse stdout alone, and refuse to run unless
  the baseline reads the exact expected per-target counts.
- **Never mix a subprocess's stdout and stderr when the parse is POSITIONAL.** Two streams are
  interleaved by the OS, not by the program.
- **Run every injection TWICE** — a LIVENESS build with a `panic!` marker at the site, then the
  SEMANTIC edit. A marker that never fires makes an injection *provably* invisible rather than
  merely unnoticed, which is a much stronger result than "no gate went red".
- **A control that stays green is the alarm.** Two of the eight were written to be caught; both
  reading "invisible" was the signal the harness was broken.

**Two measurements that inverted predictions:**
- `violation`'s documented dropped straddling cell IS real (accumulated `s` lands at
  `5.00000000000000222e-1` at every grid) and IS dropped — but `phi_lim − phi_lp` is `−5.2565e-3`
  at both its ends, so `max(0, ·)` clamps the added area to **exactly `0.0`, bit for bit**. Not
  approximately immaterial: identically zero, and by the CLAMP rather than by decay.
- Re-spelling `zeta` as `sqrt(1/(1+|P|))` instead of `1/sqrt(1+|P|)` differs by one ulp at 5 of the
  8 `P` values the suite evaluates and **nothing saw it**, because the ported gate's own bar is
  `< 1e-15` against a `~1.1e-16` gap. That **falsified a claim I had written into that gate's own
  doc comment** — the concession offered in place of admitting it was a self-comparison. Corrected
  in place. *A gate labelled as PARTLY vacuous can have a vacuous surviving half too.*

Related: [[rust-port-slice-z-step2]], [[rust-port-slice-w-step3]], [[rust-port-ported-test-vacuity]],
[[rust-port-guessed-census-bars]].
