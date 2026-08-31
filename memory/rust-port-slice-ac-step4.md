---
name: rust-port-slice-ac-step4
description: "Slice AC (rungs 70/71) step 4 — both injections that survived the ported gates survive the PYTHON gates too, so the holes were inherited, not introduced"
metadata: 
  node_type: memory
  type: project
  originSessionId: 91b1a022-a14e-4f96-8d1a-7f37b7670370
  modified: 2026-08-31T12:45:08.316Z
---

Slice AC step 4 ported rung 70's gates: `rust/tests/rung70.rs`, **754 lines, 27 gates**, green on
the first run in 5.75 s. Plan record at § 5.27.4.

**THE STEP'S RESULT IS A DISTINCTION THE SWEEP ALONE CANNOT DRAW.** Ten injections into
`src/cross_split.rs`, two binaries each: **8 caught, 2 missed.** The instinct is to book a miss as
a hole in the ported gates. So the same two changes were made in **Python** and
`pytest tests/test_rung70.py` was run on each — **27 passed both times.** The port did not
introduce those blindnesses, it **inherited** them, and that changes what to do: tightening the
Rust gate with a bar Python does not carry would make the two suites disagree about what the rung
claims. **When an injection survives a ported gate, run it against the SOURCE suite before calling
it a port defect** — the answer decides whether you are looking at a translation error or at a
property the original never pinned either.

**AND A MISS IS WORTH NOTHING UNTIL THE INJECTION IS SHOWN ABLE TO MOVE SOMETHING.** Both were
measured with a throwaway probe that drove the SHIPPED readers (never a re-spelling) under each
injection, then was deleted:

* the clock-grid destructuring swap moves **25 of 38 printed lines** — one arm's `c1` 0.13145 →
  0.07688, another's row count 9 → 8 — and all four gates still pass because all four assert a
  STRUCTURAL property (`zeros == [1]`, `c0 < 1e-9`, `c1 > 1e-2`) true at *any* admissible clock
  triple. The gates pin the SHAPE, never the LOCATION.
* the joint-window predicate widened from `and` to `or` moves the window **61 → 341 points** and
  `joint_fraction` **0.179 → exactly 1.0**, and passes because both bars are **one-sided lower
  bounds** — a union satisfies a lower bound on an intersection maximally.

**THE COUNT DID NOT RECONCILE ON THE FIRST READING AND THE EXPLANATION HAD TO BE MEASURED.**
`grep -c '#\[test\]'` says 28, `cargo test` runs 27; the 28th is inside a doc comment. Had it been
a real attribute behind a `#[cfg]` the file would ship a silently dead gate with the totals still
adding up. The Python↔Rust mapping was then done **name by name** — 27 pair 1:1 in order, 0 added,
0 collapsed, 1 body substituted — because "two collapsing into one plus one added" also sums to 27.

**A PROCESS FAILURE WORTH MORE THAN THE STEP.** The session opened with two LIVE injections in the
working tree (one Rust, one Python) left by a sweep whose driver *announced* "source restored"
without checking. They read as ordinary edits; one of them made a shipped function momentarily
self-contradictory and cost a real detour before the cause was found. The repaired driver takes its
backup with `git show HEAD:…` (never `cp` from the tree, which a re-entry would poison with a live
injection) and **proves** the restore with `git diff --quiet`, logging FAILED if it does not hold.

**AND THE CLOSE-OUT CAUGHT THE SAME SHAPE IN THE PROJECT'S OWN CLIPPY CHECK.**
`cargo clippy --all-targets` aborts on the lib's one deliberate `eq_op` error
(`src/stator_transient.rs:2757`), and that abort takes every dependent target with it — so it has
**never linted a single `tests/*.rs` file in this crate**. Every past *"zero clippy findings in
this slice's files"* that named a TEST file came from a command that could not open it. Re-run as
`cargo clippy --all-targets -- -A clippy::eq_op` it reaches **24 test targets and finds 48
warnings**, one of them in this step's own new file. Only that one was fixed; the other 47 are
disclosed and booked, because editing nine other slices' shipped test files is not this step's
scope. **`--all-targets` is not all targets while anything it depends on fails to compile.**

**Why:** all four are one shape — a conclusion drawn from an instrument nobody asked to
demonstrate its own power (a miss with no movement measured, a count with no mapping, a restore
with no diff, a lint with no target).

**How to apply:** make every instrument prove it can see before believing what it reports. Run a
surviving injection against the source suite. Reconcile counts by NAME. And never let a harness
that mutates tracked files announce its own cleanup — have it prove it, or the next session finds
the mutation and cannot tell it from intent.

Related: [[rust-port-slice-ac-step3]], [[rust-port-slice-ac-step2]], [[rust-port-slice-ac-step1]],
[[rust-port-slice-w-step3]], [[rust-port-slice-w-step5]], [[rust-port-shape-keys]],
[[rust-port-guessed-census-bars]], [[windows-tooling-file-hazards]].
