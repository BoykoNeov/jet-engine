---
name: rust-port-slice-ad-step4
description: "Slice AD step 4 (rung 72's 28 ported gates) — a self-test gated on a BOUND is blind to a defect that moves 8 of its own 10 numbers, and a count can be satisfied by an accident where a bijection cannot"
metadata: 
  node_type: memory
  type: project
  originSessionId: 4383ee3a-684d-432c-a75f-5f1b613e7af7
  modified: 2026-09-01T10:45:26.941Z
---

Slice AD step 4 shipped `rust/tests/rung72.rs` — 949 lines, 28 gates, green on the first run.
See [[rust-port-status]] for the tally; details in `docs/plans/todo-rust-port.md` § 5.28.4.

**The lesson: a gate whose bar is a BOUND cannot see a defect that moves the numbers INSIDE the
bound — including when the gate is the rung's own self-test, written for exactly that class of
error.** Rung 72 ships `charpoly_selftest` because the polynomial builder once returned a wrong
answer with plausible downstream numbers. I attacked the other half of the same chain — deleted
the `|a3|` term from the root finder's start scale, the term that wins the max on 1 068 of
1 068 shipped calls. It moves **8 of the self-test's 10 keys** and 18 more across two readers,
**26 keys of 3 216**, and **all 28 gates pass — in Rust AND in Python.** The bars are one-sided
(`< 1e-9`), a different start converges to the same roots, and the residuals move in the last
digits without crossing anything. The two keys that do NOT move are the only two computed from a
coefficient rather than from a root, which is what turns the reading into a mechanism.

**Why:** the instinct after "10 injections, 7 caught" is to widen a gate until the misses die. Here
that is impossible without inventing a bar nobody measured. The right seat for a last-digit defect
is the BIT-EXACT oracle, so the miss is booked forward rather than papered over — and the same
argument says which misses are NOT holes at all.

**How to apply:**
- **Score a miss by what it MOVES, not by whether a test went red.** Re-drive a preserved
  value dump under the injection and diff by key. Three outcomes, not two: *moves keys and nothing
  catches it* = a real hole, book it to the oracle; *moves nothing* = an unobservable edit, a
  stronger statement than "missed"; *moves keys nothing reads* = a defence with no reader.
  Rung 72's three misses were one of each — 26 keys, 0 keys, 3 keys.
- **Then ask whether the SOURCE misses it too.** Apply the same edit to the Python and run its
  suite. Both real holes here passed 28 Python gates, so they are INHERITED, not introduced by the
  port — a different and much smaller liability.
- **Disclose a margin, never the word "unreachable".** The inert injection widened a strict
  interval to a closed one; the measured distance to the nearer endpoint was 3.5e−2 against a
  0.1-wide interval, over 165 of 165 eligible points.
- **Reconcile a ported test count by BIJECTION, not by counting.** `grep -c '#[test]'` said 30
  where `cargo test -- --list` said 28 — the third instance in the phase, and this time the two
  extras were in my own header sentence *documenting the trap*. Map every Rust name to a collected
  Python node id and assert 0 unmapped / 0 extra / 0 collisions. A count can be satisfied by an
  accident; a bijection cannot.
- **Read every reader argument off the SOURCE's `def` line, not off the test module's constants.**
  Rung 72's five readers default to three different step sizes, two different strides and three
  different clock grids, and the shipped tests pass none of them. Copying the module's `DS` into
  all five would have moved every number without failing anything —
  [[rust-port-slice-ac-step6]]'s `every = 40`-vs-`10` defect, pre-empted by tabulating the five
  signatures before writing a call.
- The miss-probe's own first run printed `BASELINE keys: 0` because it parsed stdout from a program
  that writes a file. Its see-check caught it — [[rust-port-slice-w-step3]] again.
- **And the STEP's own gate number was nearly read off a run that had not finished** — 125
  result blocks summing to 1 335 passed, both plausible, no error text anywhere, because a
  truncated `cargo test` log is a valid prefix of a good one. It exited 0 twelve minutes later
  at 137 / 1 393. Count the lines that ANNOUNCE a target and require them to equal the result
  blocks; a sum cannot detect a missing block. [[windows-tooling-file-hazards]] hazard 5.

Related: [[rust-port-slice-ac-step4]] (the same "the gates pin SHAPE, not LOCATION" result one
slice earlier), [[rust-port-slice-ad-step3]], [[rust-port-slice-aa-steps2345]] (defence with no
reader), [[rust-port-slice-ab-step3]] (run the sweep over ALL the slice's binaries — three here).
