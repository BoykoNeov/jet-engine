---
name: rust-port-slice-ah-step7
description: "Slice AH step 7 (CLOSED) — a OnceLock serialises a COMPUTATION, not a process-global counter: another gate in the same binary marched a gauged machine and 838+528=1366 hits landed on a rung-77 baseline"
metadata:
  type: project
---

Slice AH step 7, 2026-09-26: `rust/tests/slice_ah_dispatch.rs`, 8 gates; the slice CLOSES
(plan § 5.32.7). Rows landed as predicted; P2's channel confirmed by source mutation (13
failures, all pointer comparisons, oracle's 29 288 values unmoved), its "no ported gate" clause
refuted (the two ported `isinstance` gates fail).

**The lesson: a lock protects the resource you wrap it around.** The advisor prescribed caching
the matrix in a `OnceLock` "to remove the GAUGE_HITS race". That removed races between readers of
the MATRIX, not between the matrix and every other writer of the process-global COUNTER. A
separate gate in the same binary marched a gauged machine on its own thread, and the rung-77
baseline (a machine with NO gauged branch) read 838 + 528 = 1 366 hits: exactly one gauged
march. The sum is what diagnosed it. Fixed with one `Mutex` taken by the matrix and by every gate
that drives a gauged march.

**Why:** cargo runs a binary's tests on parallel threads; any global counter is shared by all of
them, whatever caching the reader does.
**How to apply:** when a test reads a process-global counter, enumerate every gate IN THAT BINARY
that can WRITE it, and lock them all — and when a count comes back impossible, check whether it
equals one known unit of some other test's work.

Also measured: `cap_fuel` is entered at ONE of eight seats (only the DEMAND march calls it;
seven readers stand on the CLIP `_ledger_march`), so its row's silences are UNREACHABILITY while
`shared_rig`'s identical row is REDUNDANCY. I and the advisor both predicted "entered everywhere".
And a forgotten `at_lever:` line leaves the private body dead, which rustc WARNS about: a free
tripwire the "no E0063" census did not count. See [[rust-port-slice-ah-step6b-oracle]],
[[rust-port-slice-ag-step7]].
