---
name: rust-port-ladder-architecture
description: "The 28-rung ladder ports as a const table of fn pointers per rung — settled by a spike, do not re-litigate"
metadata: 
  node_type: memory
  type: project
  originSessionId: 454e5108-5b41-4abd-b607-eac9932757b5
  modified: 2026-08-12T06:22:33.526Z
---

Python's 28-deep transient inheritance chain becomes **one `const Hooks` table of function
pointers per rung**, with `const R64: Hooks = Hooks { close_fuel: r64_close_fuel, ..R63 };` —
so a rung *is* the fields it replaces, and toggling is one `match` at the top of the program
rather than a branch per timestep. A depth-28 spike settled this with measurement, not argument
(`M:\claud_projects\temp\rust-spike`, its `RESULTS.md`).

The decisive number: the hook redefined **most** often (`at_lever`, 18×) fires **once** per
march, while the only hot one (`_close_fuel`, 62,670 calls) is redefined 5×. Lean execution and
28 toggleable histories are not in tension — they occupy disjoint code. Dispatch costs 0.5 % at
real rates, inside the drift check's own 0.1 %.

**Why compile-time generics lost:** the arrangement that drops the leaf type parameter
**compiled and returned a silently different number** (0.018 % off). Python resolves
`self._instant_fuel` inside an old rung's body to the *leaf's* override; generics resolve it to
the parent's. A function-pointer table cannot make that mistake — the leaf's table is the
argument. **Why copy-forward lost:** `_close_fuel` passes `super()._close_fuel` *into* a solver
as a value, so copying would duplicate the parent's body inside the child.

**How to apply:** two traps are already paid for. Never `#[inline(always)]` down the ladder — a
rung's fallback sits at two call sites, so at depth 28 that is 2²⁷ expansions (the first spike
build ran >10 min in codegen and was killed). And `at_lever`/`_shared_rig` (26 overrides that
exist only to copy fields forward, a trap the docstrings say the project hit 18 times) collapse
to `Config { vsv_lp, ..self.cfg }` — forgetting a field stops being expressible.
Related: [[rust-port-decided]].
