---
name: rust-port-slice-ac-step7
description: "Four shipped doc comments claimed an alias is louder than a spread when neither is loud at all, and the injection every reader launders needed a declared carrier to be observable"
metadata: 
  node_type: memory
  type: project
  originSessionId: f0d438fb-b4ba-467d-b0e5-f1af8995bcd2
  modified: 2026-08-31T18:14:15.208Z
---

Slice AC step 7 (rungs 70/71 Rust port) shipped `rust/tests/slice_ac_dispatch.rs` — **9 gates**,
five function pointers over five swaps — plus four corrected doc comments. All nine green on the
first run, and **6 of 6 mutations of this file's own gates were killed**. **Slice AC is closed.**

**THE LESSON: when a comment gives a REASON for how something is written, the reason is a testable
claim — test it.** `R70_TWO`, `R70_STATOR`, `R71_TWO`, `R71_STATOR` each said they were *named
rather than reached through a `..` spread* so the NEXT field added to that struct would not be
silent. All four are whole-const **aliases**, and an alias is exactly as silent as a spread —
neither emits a diagnostic when the struct grows. `R66_TWO`, the precedent all four cite, is
itself an alias. Measured by adding a probe field to each of the five hook structs and counting
`E0063` sites: **5 of 5 `TripleHooks` consts are loud** (they spell every field out) against
**2 of 11, 2 of 11, 2 of 11 and 3 of 10** for the other four types. The crate had a width gate for
one of five table types and four comments asserting it had five. Corrected, and pinned by a
tripwire that CAN fail — exhaustive destructuring, **not** `ptr::eq` on a `const` (a reference to a
const is a fresh temporary; [[rust-port-slice-aa-step1]]). My own first draft had that `ptr::eq`
inside a `||` whose other arm made it unfalsifiable: *a gate that cannot fail hiding inside one
that can.*

**AND THE INJECTION EVERY READER LAUNDERS.** Every rung-70/71 reader opens by calling a `*_rig`,
which calls `core.at_lever(…)`, whose body rebuilds through the cascade builder — installing the
**shipped** table. So a `TripleHooks` or `FuelTransientHooks` injection into a core never reaches
the first row: `split_gains` on an injected core comes back **bit-identical**, seven rows and all.
In Python the same patch survives, because `_triple_laws` is a method on the CLASS and the sibling
is the same class. **So the pre-flight's "seen by 1 of 6 readers" is a PYTHON number that does not
transfer**, and scoring the cell through a reader would have reported UNOBSERVABLE for a reason
about the builder. Each swap is scored where it is dispatched, and `triple_laws` gets a DECLARED
carrier with its own control (mutation m6 proves that control can fail).

**Two habits that paid, both from earlier steps of this same slice:** the sample-emptying gate is
`assert_eq!` on **both** endpoints (7 rows → 0, and the mirror 0 → 7 skipped), because
[[rust-port-slice-ac-step6]] found three ported gates blind to exactly that shape through one-sided
lower bounds; and the mutation script asserts each anchor is present before `str.replace`, because
step 6's patch script printed `patched` having matched nothing.

**How to apply:** treat a "written this way because X" comment as a claim with a cheap experiment
behind it — for a struct-width claim, add a field and read which sites fail to compile. And before
scoring a hook-table injection through a reader, check whether the reader REBUILDS its machine
first; if it does, the injection is laundered and the honest score is a direct call or a declared
carrier, never a convenient reader.
