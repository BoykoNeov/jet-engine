---
name: rust-port-slice-ag-step1
description: "Slice AG step 1 — a gate written to prove a defect is invisible failed twice, and each failure was the finding"
metadata: 
  node_type: memory
  type: project
  originSessionId: 60dbc86b-d568-4baf-a62c-5d595a6e8060
  modified: 2026-09-06T16:15:48.366Z
---

Slice AG step 1 (plan § 5.31.1): rungs 75/76's plumbing — `src/anti_windup.rs` + `src/sensed_cap.rs`,
eight re-aimed pointers, **zero new table fields**, and `tests/slice_ag_cells.rs` (15 gates).

**THE PROCESS LESSON: when a gate you wrote to demonstrate a defect FAILS, read the failure as a
measurement before you fix the gate.** Both of this step's own gate failures were facts.

* I wrote *a missed re-aim is silent on the march* and it failed on the `applied` reference,
  because rung 74 has **no plant** there — so the missing device hits rung 74's joint-IC refusal
  and the defect is **LOUD**. Under `sched` rung 74 does have a plant and the same defect is
  **bit-for-bit silent**. **Whether a forgotten swap is loud is a property of the cell you happen
  to drive, not of the defect.**
* Then the positive control failed: with the cell correctly aimed, `Tt4` is **bit-identical**.
  That is rung 75's own headline — the device sits in the MASKED leg's law and `min`-select hides
  it from the plant — so a control on the output reads an **exact zero** and calls the correct
  cell inert. Re-aimed onto the two masked STATES (`w_fuel`/`w_gov`), it passes.
  [[rust-port-slice-t-step1]]'s *an EXACT ZERO blinds its own gate*, one slice on.

**Why this step needed an unusual instrument at all: `0 ADD`.** Every step 1 from slice AA to AF
widened the hook table, so a forgotten cell was a compile error. Here a forgotten re-aim compiles
and both parent bodies return exactly this slice's reduce-arm answer (`None` / `Ok(None)`), so
every reduce gate in the crate keeps passing. The replacement is **function-pointer identity in
both directions** — swapped slots must differ, inherited slots must be equal — which is also the
only witness for a *stray* re-aim (rung 76 silently disarming rung 75's device).

**Three inherited claims measured false or imprecise:**

* **`_with_ic_cap` does not exist.** Two shipped Rust doc comments named it as rung 75's writer of
  `_ic_cap`; the actual write is a bare `try/finally` inside `contraction_law`. It matters because
  the crate's carrier rule is *dispatch the setter iff a later rung overrides `_with_*` to write a
  different field* — an invented `_with_*` name invites a cell for a write Python makes by plain
  assignment, which is slice AF step 6's four-site defect. Slice AD's *a shipped block documents a
  method with ZERO definitions*, one slice on.
* **`max(1e-9, x)` on a NaN `x` returns `1e-9` in Python, not `x`.** Three `1e-9f64.max(·)` sites
  were filed as *unmeasured* cells on the opposite claim. Python seeds the fold at argument 0 and
  replaces only on a strict comparison, so a NaN propagates **only from argument 0** — and Rust
  discards it from either side. Census over `engine.py`: 800 `max`/`min` calls, 268 n-ary, **103
  literal-first** (exactly faithful under `lit.max(x)`) and 165 expression-first (where the
  spelling is a decision). All three sites are decided, and so are the crate's ~25 others.
* **A derived bound is not its own rounded decimal.** `2*ds/(2 - ds*sum(1/tau_i))` is
  `0.0062499999999999995` in **both** languages, one ULP below the `0.00625` rung 75's docstring
  quotes — which is exactly why the constant is spelled as the derivation.

**A FOURTH, and it is the step's leading finding: A LINE CITATION IS A CLAIM WITH AN EXPIRY DATE.**
The port cites the Python it ports by line. Swept properly — 50 sites over 9 files, in TWO forms
(`engine.py:N`, and the bare `` `N` `` that a pair's second site is always written in) — **18 were
stale**: 13 pushed `+2` by `a592a0d` (slice AF's own documentation-decay guard commit, which edited
`engine.py` on its way past), 1 by `+14`, and 4 that a first regex for `engine.py:` could not see at
all. **The under-count was my own instrument's population error, one step after the pre-flight wrote
that lesson down about a shipped comment** — and one of the 18 was written by THIS step, copied out
of the comment it was correcting. Repaired all 18, and shipped
`tests/test_rust_line_citations.py` + a blessed anchor file: it detects DRIFT (the ground moved under
a verified citation) and says so with the new line number, and it CANNOT detect wrongness, which the
file says out loud. Two exemptions, both citations quoted as history, both created by the same
commit.

Two smaller ones from the same sweep: `19022` beside the invented `_with_ic_cap` **was the right
line** under its own day's numbering, so the defect was a name on a correctly-located line; and the
`max(1e-9, x)` correction's own first draft filed `applied_demand`'s gate in the wrong half of its
own census. The one expression-first `1e-9` fold in the whole package is rung 76's `_c_at`, which
lands at **step 4 of this same slice** — pre-registered there rather than met as a habit.

See [[rust-port-slice-ag-preflight]], [[rust-port-status]].
