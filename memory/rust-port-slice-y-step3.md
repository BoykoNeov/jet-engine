---
name: rust-port-slice-y-step3
description: "Slice Y step 3 — pointer identity on a Rust `const` is a compiler property, not a machine one; and the phase's `slow` bill finally measured at 16x"
metadata: 
  node_type: memory
  type: project
  originSessionId: cc4ae781-acf8-4650-a2a9-99b2cb27665e
  modified: 2026-08-26T18:45:10.726Z
---

Rung 65's 21 Python gates ported plus one the Python suite has no reason to own: Python's class
inherits its constructor and flips a class constant, Rust has to RE-SPELL the ten-assert chain with
one assert relaxed, and dropping any of the other nine leaves every value key green. That gate
drives each assert individually and pins the count.

**`std::ptr::eq` ON A `const` PROVES NOTHING.** The one gate that failed on first run asserted that
a sibling machine's hook table was rung 65's, by comparing `&R65` pointers. A `const` is inlined at
every use, so each `&R65` is a fresh promotion — the assertion tests the optimiser, not the object.
`fn_addr_eq` on the individual swapped CELLS is both reliable and stronger: it says *which bodies*
the sibling carries rather than which struct it points at.

**THE `slow` BILL, MEASURED RATHER THAN ESTIMATED.** Phase 7 has 263 `slow`-marked gates and the
plan had been deferring the cost question. This suite is 9 of 21 marked `slow` (42.9 %): PyPy runs
21 gates in 49.2 s, the Rust port runs 22 in 3.0 s — **16x**, same box, single-process both sides.
No `#[ignore]` re-introduced; the marker records a cost that does not survive the port.

**Why:** both are cases where the obvious spelling of an assertion is satisfied by something other
than the property it names — the recurring shape of this whole port.

**How to apply:** never assert pointer identity on a `const`; compare the function pointers. And
when a rule says "re-introduce a cost marker only against a measured number", take the measurement
at the first suite where it could matter, not the last.

Related: [[rust-port-slice-y-step1]], [[rust-port-slice-y-step2]], [[test-suite-speed-policy]].
