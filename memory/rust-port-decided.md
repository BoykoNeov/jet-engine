---
name: rust-port-decided
description: "The project is being rewritten in Rust; phases 0-1 are done and green, phase 2+ needs fresh authorisation"
metadata: 
  node_type: memory
  type: project
  originSessionId: 454e5108-5b41-4abd-b607-eac9932757b5
  modified: 2026-08-12T06:22:04.548Z
---

On 2026-08-12 the user decided to rewrite the whole project in Rust. Python survives only as a
**single-use oracle** (deleted at phase 8) plus **one small matplotlib script for the chart** —
the rule the user gave is "the ENGINE is pure Rust", so everything computational, including the
station tables, is Rust.

The plan is `docs/plans/todo-rust-port.md` — read it rather than re-deriving. Three decisions
are already locked: bit-exactness **option B** (Python as oracle, then re-anchor to Rust);
`main.py` **split** (Rust prints tables + emits plot JSON, Python draws only); and **phases 0–1
only**, then stop.

**Phases 0 and 1 are COMPLETE, green, committed and pushed** (`rust/`, commits 101e3be and
f8a77d1). Phase 2 onward is **not authorised** — ask before starting it.

**Why:** the user deliberately scoped an early exit at phase 1 because that is where the
arithmetic risk concentrates. Treating "the plan exists" as authorisation to keep building
would spend a large budget on an unapproved branch.

**How to apply:** if asked to continue the port, confirm which phase. The architecture question
is settled and should not be re-litigated: a `const Hooks` table of function pointers per rung,
NOT compile-time generics (measured — see [[rust-port-ladder-architecture]]) and NOT a collapse
into one engine with flags (it would lose the ability to run superseded rungs, which the user
explicitly wants). Related: [[rust-port-arithmetic-is-pypy]], [[windows-tooling-file-hazards]].
