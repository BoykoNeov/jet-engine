---
name: rust-port-slice-ag-step6
description: "Slice AG step 6 (rungs 75/76 ported gates + oracle) — a pre-registered exemption named the right cause and sized it by the wrong property, and a census that grew was read as coverage"
metadata: 
  node_type: memory
  type: project
  modified: 2026-09-07T11:28:43.278Z
  originSessionId: 0ce3e683-3c6d-45e1-aa9c-28d47c8ad3ce
---

Slice AG step 6 of 7 shipped in three commits (`0aedb9d`, `af59e3e`, `61f4dc7`, 2026-09-07):
the widened citation guard, `rung75.rs` + `rung76.rs`, and the cross-language oracle.
Detail is in `M:\claud_projects\jet engine\docs\plans\todo-rust-port.md` § 5.31.6.

**THE LESSON — a prediction can name the right CAUSE and size it by the wrong PROPERTY.**
P2 was pre-registered before the CPython arm ran: the PyPy/CPython exemption would be
exactly the two `sum()` calls that add a **341-long** trajectory, CPython 3.12+ being
Neumaier-compensated where PyPy and Rust fold naively. Measured: **406 of 38 100 keys
differ.** The 8 keys P2 named all differ — the cause is right. 398 more differ because
`_charpoly4` is built on two `sum()` calls of **FOUR TERMS**. Compensated summation parts
company with a naive fold as soon as the addends have **mixed magnitudes**; `n` was never
the criterion, and the prediction had reasoned about length.

**The second half, which is the reusable one: blast radius is set by SHARING, not SIZE.**
The suspected 341-term sum sits in one reader and moved 8 keys. The unsuspected 4-term one
sits in a **shared static helper**, so it reached every reader reporting a determinant —
398 keys, three sections, six readers. **Ask what SHARES the code, not what is biggest.**

**Why:** an exemption list read off the observed diff would have passed and taught nothing.
Writing it as the CAUSAL PATH (`came out of _charpoly4`, or out of a trajectory `sum()`)
plus **asserted counts** (398 and 8) means an exemption that stops matching what it exempts
fails. Cf. [[instrument-fed-by-what-it-certifies]], [[rust-port-guessed-census-bars]].

**How to apply:** before pre-registering a cross-interpreter exemption, grep the readers for
`sum(` and follow the SHARED helpers, not just the long loops. When scoring, report the
cause and the scope separately — one can be right while the other is wrong.

**A census that grows is not evidence that coverage grew.** Commit (a) widened the citation
guard from one root to two (24 of 47 sites stale in the unwatched one). Commit (c) found a
**third** — `rust/oracle`, the Python dumpers. Adding it with a `#` marker moved the site
count 139 → 140, one of the four that are there; the other three sit in a module
**docstring**, which no marker-based scanner reaches. Unchecked, +1 would have been read as
coverage of the directory. **The two numbers move together only if you check.** Same defect
class as (a), one step later, same file, same author — see [[claude-md-is-a-reference]] for
why that gets written down twice rather than quietly fixed.

Three instrument defects the oracle found in itself: a helper that both BUILDS a key and
EMITS one is only safe at the depth its emission belongs to (`arms()` called inside three
inner loops); a float census that classified its population by **pattern-matching key names**
over-counted by 173 (replaced by a measurement at the emission site); and the `neg_zero` /
`nan` counters now push `-0.0` and a NaN through the real emitter at every run so a `0`
means *measured none*, not *never fired* — [[rust-port-slice-w-step3]]'s lesson applied
before a failure instead of after one. See also [[rust-port-status]].
