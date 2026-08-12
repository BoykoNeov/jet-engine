---
name: rust-port-decided
description: "The project is being rewritten in Rust; phases 0-4 DONE, phase 5 needs FRESH authorisation (it contains the rung-61 diamond)"
metadata: 
  node_type: memory
  type: project
  originSessionId: 454e5108-5b41-4abd-b607-eac9932757b5
  modified: 2026-08-12T17:30:05.620Z
---

On 2026-08-12 the user decided to rewrite the whole project in Rust. Python survives only as a
**single-use oracle** (deleted at phase 8) plus **one small matplotlib script for the chart** —
the rule the user gave is "the ENGINE is pure Rust", so everything computational, including the
station tables, is Rust.

The plan is `docs/plans/todo-rust-port.md` — read it rather than re-deriving. Three decisions
are already locked: `main.py` **split** (Rust prints tables + emits plot JSON, Python draws
only); the `const Hooks` ladder architecture; and **stop-and-re-decide at each phase boundary**.
The bit-exactness fork was decided as **option B** (tolerance) but phases 0–2 are held to
**bit-equality** instead, because it was measured achievable — see [[rust-port-power-spelling]]
for why that revision happened and what would justify falling back.

**Phases 0–4 are COMPLETE, green, committed and pushed** (`rust/`, 377 tests) — phase 3 shipped
as five slices A–E, phase 4 as three dependency slices F (25/26), G (27/28), H (29/30). Every
oracle is 100 % bit-exact against PyPy. No further authorisation is needed *inside* an authorised
phase. **The next re-decide point is NOW: phase 5 needs fresh authorisation**, and it is the one
that contains the rung-61 multiple-inheritance diamond.

**Slicing by dependency is now the established shape of a phase**, and the slices are where the
findings come from: five oracles at 100 % bit-exact and every finding produced instead by
sweeping past the source's own gates.

**Why:** the user scoped early exits because that is where the arithmetic risk concentrates,
and authorises phases one at a time. Treating "the plan exists" as authorisation to keep
building would spend a large budget on an unapproved branch — but a slice inside an authorised
phase is not a new branch, and stopping to ask between them would be friction, not caution.

**How to apply:** if asked to continue the port, confirm which phase. The architecture question
is settled and should not be re-litigated: a `const Hooks` table of function pointers per rung,
NOT compile-time generics (measured — see [[rust-port-ladder-architecture]]) and NOT a collapse
into one engine with flags (it would lose the ability to run superseded rungs, which the user
explicitly wants). Related: [[rust-port-arithmetic-is-pypy]], [[rust-port-shape-keys]],
[[windows-tooling-file-hazards]].
