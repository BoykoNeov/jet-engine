---
name: rust-port-status
description: "Where the Rust port stands — phases done, the slice currently in flight, and the numbers each closed slice landed on"
metadata: 
  node_type: memory
  type: project
  originSessionId: 57f6e146-1b56-46c8-a924-d490be2f24f2
  modified: 2026-08-28T09:14:08.040Z
---

The living status line for the Rust port. **This file is the only place the running tally lives**;
`MEMORY.md` carries a one-line pointer to it, because the tally grows every step and an index line
that grows is not an index line. Update THIS file at each step, not the index.

**Decided 2026-08-12.** Plan: `docs/plans/todo-rust-port.md`. A new PHASE needs authorisation;
slices inside an authorised phase are free. See [[rust-port-decided]].

## Phases

* **0–5 DONE.**
* **PHASE 6** authorised 2026-08-17, **COMPLETE 2026-08-20** with slice U's five steps.
* **PHASE 7** authorised 2026-08-20. Slices V, W, X, Y, Z, AA, **AB complete**; **AC (rungs 70+71) PRE-REGISTERED 2026-08-28** (§ 5.27, thirteen probes) — step 1 not started.

## Phase-7 slices, as they closed

| slice | rungs | landed |
|---|---|---|
| V–Y | 57–65 | slice Y closed 2026-08-27, all five steps |
| Z | 66+67 | 2026-08-27 — both marches + 20 method bodies, **38 ported gates** green first run at **20.0×** PyPy, **35 335 oracle keys** bit-exact on both interpreters (CPython exempt on 8 named keys), **8 dispatch gates** closing the four measured blind spots |
| AA | 68 | 2026-08-27 — `three_loop.rs` 2 302 lines, **47 gates**, **12 084 oracle keys** bit-exact vs PyPy (4 named CPython exemptions), full gate **124 blocks / 1 199 passed / 0 failed** |
| AB | 69 | **2026-08-28, all five steps** — `reference_split.rs` 1 686 lines, 25 ported gates, **15 957 oracle keys** bit-exact vs PyPy (194 named CPython exemptions), and **14 dispatch gates** over TEN cells, every one observable; full gate **129 binaries / 1 258 passed / 0 failed** |

## Slice AC — pre-registered, seven steps priced

Rungs 70 + 71, `docs/plans/todo-rust-port.md` § 5.27. **1 605 Python lines (2.27x slice AB),
57 collected tests (22 slow), ZERO cells added, 5 swaps over two rungs = 5 distinct function
pointers and NO new table field, 6 reduce arms all bit-for-bit by dispatch.** Rust estimated
3 400-3 800 lines. **SEVEN steps, pre-registered as P7** — the port and the gates each split by
rung, because five steps on a 2.27x slice would be habit rather than a measurement. Nine
predictions P1-P9.

**The plan's own column said 1 cell (`split_gains`) and it was wrong**: rung 80's same-named
method has an incompatible signature, so it is a NAME REUSED, not an override. The phase-wide
sweep of all 358 override pairs found one more — `_legs` (63 -> 77), a SHIPPED slice-W cell,
booked to slice AH. See [[rust-port-slice-ac-preflight]].

## Slice AB, step by step

1. **Step 1** — `src/reference_split.rs` 519 lines, 13 cell gates; nine swapped bodies panic.
2. **Step 2** — the nine bodies + six readers, 1 641 Rust lines from 713 Python (2.30×).
3. **Step 3** — the **25 ported gates** of `tests/rung69.rs` (851 lines, green first run); full gate
   **127 binaries / 1 241 passed / 0 failed**. A ten-injection sweep caught 9; the one hole, the
   root finder's Newton budget, was settleable only by step 4's oracle.
4. **Step 4** — the oracle: `dump_slice_ab.py` + `slice_ab_oracle.rs`, **15 957 keys per arm**,
   `Rust ≡ PyPy` on all of them after ONE fix (`py_half` — Python's `0.5 * z` is a complex product,
   not a scaling). CPython arm exempt on **194 named keys, two causes**. See
   [[rust-port-slice-ab-step4]].
5. **Step 5** — `tests/slice_ab_dispatch.rs`, **725 lines, 14 gates**: each of the ten cells swapped
   for its PARENT's function pointer (P2's letter), plus one declared counterfeit for `with_ref`.
   All ten observable, the count EMITTED. **Two of the pre-flight's four "breaks by PANIC" break by
   VALUE** — `triple_rig` and `manifold_v`, because rung 68's bodies for those two read no field at
   all. See [[rust-port-slice-ab-step5]].
