---
name: rust-port-decided
description: "The project is being rewritten in Rust; phases 0-4 DONE, phase 5 needs FRESH authorisation (it contains the rung-61 diamond)"
metadata: 
  node_type: memory
  type: project
  originSessionId: 454e5108-5b41-4abd-b607-eac9932757b5
  modified: 2026-08-17T17:49:46.416Z
---

On 2026-08-12 the user decided to rewrite the whole project in Rust. Python survives only as a
**single-use oracle** (deleted at phase 8) plus **one small matplotlib script for the chart** —
the rule the user gave is "the ENGINE is pure Rust", so everything computational, including the
station tables, is Rust.

**Restated by the user on 2026-09-07, unprompted: getting rid of Python is an AIM of the port,
and it includes ALL the Python TESTS.** So the per-slice habit of porting each rung's gates
1:1 into `rust/tests/rungN.rs` is not a nicety — it is the only thing that makes phase 8's
"full suite green on Rust alone" reachable. Two consequences to hold: a Rust gate that leans on
a Python-produced artefact is only portable if that artefact is a COMMITTED golden (the dump
SCRIPTS are Python and die); and any Python test with no Rust counterpart is an undeclared
phase-8 blocker, so coverage of the source suite is a running obligation, not a final audit.

The plan is `docs/plans/todo-rust-port.md` — read it rather than re-deriving. Three decisions
are already locked: `main.py` **split** (Rust prints tables + emits plot JSON, Python draws
only); the `const Hooks` ladder architecture; and **stop-and-re-decide at each phase boundary**.
The bit-exactness fork was decided as **option B** (tolerance) but phases 0–2 are held to
**bit-equality** instead, because it was measured achievable — see [[rust-port-power-spelling]]
for why that revision happened and what would justify falling back.

**Phases 0–5 are COMPLETE, green, committed and pushed** — phase 3 shipped as five slices A–E,
phase 4 as three (F/G/H), **phase 5 as seven (I·J·K·L·M·N·O), finished 2026-08-17**. Every oracle
is 100 % bit-exact against PyPy. No further authorisation is needed *inside* an authorised phase.
**PHASE 6 (the 15 transient rungs) was AUTHORISED 2026-08-17**; its pre-flight is
[[rust-port-phase6-preflight]] and its slices are P–U. **The next re-decide point is before
phase 7.**

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
