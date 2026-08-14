---
name: rust-port-slice-l-step1
description: "Slice L step 1 — fallibility is adjudicated per CALL SITE, not per function; and a broken message can fail a gate that is correct"
metadata: 
  node_type: memory
  type: project
  originSessionId: f43910a0-ec86-4640-9d09-6a96a1cfe30c
  modified: 2026-08-14T11:32:04.441Z
---

Rust port, phase 5 slice L (rungs 41 + 42), step 1 shipped 2026-08-14 as commit `ff784ed`.
Rung 41 skips failing points with `except AssertionError`, so every assert reachable inside
`match` had to become an inspectable `Abort` — measured on a 147-cell grid, seven sites firing
and everything else zero. Full handoff state, the count table, and the ordered remaining work
are in `docs/plans/todo-rust-port.md` § 5.8.

Two lessons, both about instruments rather than physics:

**1. The zero-firing rule is per CALL SITE, not per function.** `t_from_h` and `t_from_pr_t`
both reach the same `solve`. Through `t_from_pr_t` it raises 36 times; through `t_from_h`, zero.
So `solve` gained a fallible twin while `t_from_h` kept its `assert!` — the same function, two
verdicts. Had I adjudicated per function I would either have made a path fallible that can
never fail (a gate measuring nothing) or left the live path un-catchable. This is also what
overturned § 5.4 (i)'s earlier "solve stays a panic" decision: a *measurement* overturned it,
which is the only thing allowed to.

**Why:** the crate's rule is reachability from inside a caught scope, and reachability is a
property of the path, not of the callee.

**How to apply:** when deciding fallibility, record the innermost raising frame *and* the chain
above it, then adjudicate each (site, chain) pair separately. See
[[rust-port-oracle-cannot-see-a-missing-gate]] — a bit-exact oracle says nothing about which
paths a guard actually covers.

**2. A gate failing does not mean the gate is wrong.** `rung39.rs` gate 9 broke, and I spent a
debug harness proving the new `Abort` behaviour was correct before noticing why: I had written
the `format!` string literal across lines without `\` continuations, so the message carried 18
literal spaces mid-sentence and `contains("OUT OF SCOPE")` no longer matched. The gate was
right; the *message* was broken. I had already reached for "the gate catches a panic and the
guard is now an `Abort`" as the explanation — plausible, and wrong.

**Why:** a refactor makes the refactor the obvious suspect, so a mundane co-located typo gets
attributed to the interesting change.

**How to apply:** before concluding a gate needs repair, print the actual value it compared.
`repr`/`{:?}` on the string, not the reasoning about it. Related:
[[windows-tooling-file-hazards]], [[rust-port-power-spelling]] — both are defects a plausible
story explained away.

**Process note:** permission prompts in this session came from `cd "…" && cmd` compounds in the
Bash tool, which cannot be statically checked on Windows. Bare commands from the already-correct
working directory are auto-approved. Never write a `cd`-compound here.
