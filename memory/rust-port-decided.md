---
name: rust-port-decided
description: "The project is being rewritten in Rust; phases 0-2 done, phase 3 authorised and running in SLICES, phase 5 needs fresh authorisation"
metadata: 
  node_type: memory
  type: project
  originSessionId: 454e5108-5b41-4abd-b607-eac9932757b5
  modified: 2026-08-12T08:12:17.150Z
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

**Phases 0, 1 and 2 are COMPLETE, green, committed and pushed** (`rust/`). **Phase 3 was
authorised on 2026-08-12 and runs in SLICES** — it is the port's largest phase (2,745 source
lines, 204 tests, eight mutually-exclusive mixing closures), so it ships one green gate at a
time rather than as one landing. **Slice A (rungs 7/8/9/19) is done**; remaining: the
finite-rate quench (10–12), the PDF family (13/15/16/18), the nozzle strand (14/17), the
spatial fields (20–24). No further authorisation is needed *inside* phase 3. The next
re-decide point is **before phase 5**, which contains the rung-61 multiple-inheritance diamond.

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
