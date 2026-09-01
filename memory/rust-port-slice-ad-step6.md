---
name: rust-port-slice-ad-step6
description: "Slice AD step 6 (rung 72's dispatch gates, slice closed) — a first definer still has a parent pointer because the parent slot carries a REFUSAL, so the pre-flight cited a precedent's exception where its rule applied; and my own new header typed nine tests beside the ten that disprove it"
metadata: 
  node_type: memory
  type: project
  originSessionId: 5ffd5501-1ff4-460b-9641-4d20ca45158e
  modified: 2026-09-01T14:00:12.731Z
---

Slice AD step 6 shipped `rust/tests/slice_ad_dispatch.rs` — **three injections across ten tests**,
no source file touched, all ten green on the first run. **Slice AD closes**, and with it rung 72.
See [[rust-port-status]] for the tally; details in `docs/plans/todo-rust-port.md` § 5.28.6.

**The lesson: "this rung is the FIRST definer" does not imply "there is no parent pointer to
install".** The pre-flight reasoned from the first to the second and concluded all three of the
slice's cells needed a hand-written sentinel — slice AB's declared *"THE ONE INJECTION THAT IS NOT
A PARENT POINTER"*. Measured, the parent table holds a pointer in **all three** slots: the shared
refusal, the same address in `NO_TRIPLE`, `R68_TRIPLE` and `R71_TRIPLE`. A first definer has no
parent **body**; it still has a parent **pointer**, because the refusal table's whole design is
that every slot is occupied by something that refuses. **So the pre-flight cited the precedent's
EXCEPTION where the precedent's RULE applied** — AB's own `parent_swap!(P_WITH_REF, with_ref)` does
exactly this, on a cell whose parent slot is also a refusal. AB's exception is the *second*
injection into that same cell, and it exists only because a refusal is a loud break and cannot
expose a quiet one.

**Why it matters beyond bookkeeping:** a counterfeit's observability is a property of the body *I*
wrote, so a gate on it can be satisfied by having written a satisfiable body. A shipped constant's
is not.

**How to apply:**
- **Before writing a sentinel, look in the parent slot.** Ask what the ancestor table actually
  holds, not whether an ancestor *implements* the cell. Where a `NO_*` refusal exists, the
  parent-pointer injection exists too, and it is strictly better evidence.
- **Score a precedent by which of its parts you are invoking.** Citing a prior slice's *declared
  exception* is a much stronger claim than citing its rule, and it went unchallenged here from the
  pre-flight to the last step because nobody re-opened the file it named. [[rust-port-slice-ab-step5]]
- **Run the seat matrix WHOLE, not on the diagonal.** Three cells x six seats is 18 readings —
  **7 panics, 11 silences** — and the pre-flight's seat table names 3 of them. Running only the
  diagonal gives three panics that are entirely consistent with the other fifteen seats being quiet
  because the injection never took. Each cell raising *somewhere* in its own row is what makes
  every silence in that row a fact about the PATH. The gate refuses to report a silence for a cell
  silent at all six seats.
- **The eleven silences were two different mechanisms and read identically.** Ten are laundering
  (the rig's third line rebuilds through the cascade builder and re-installs the shipped tables);
  the eleventh is a path that never calls the cell at all. *Did it panic* cannot tell them apart —
  only the other seat can. Extends [[rust-port-slice-ac-step7]]'s laundering finding, whose column
  covers one of the two.
- **With a refusal injection, "silent" needs three assertions before it means laundered:** the
  reader COMPLETES, its reading is BIT-IDENTICAL to shipped, and the shipped reading is
  NON-TRIVIAL. AC's version asserted the identity alone; a refusal makes the completion half
  load-bearing, because *no panic* is equally what "never reached" looks like.
- **Score a cell that many readers dispatch as a CENSUS.** `shared_rig` has five readers; a
  one-reader gate passes on a crate where four fifths of the dispatch is inlined.
- **And I typed NINE tests in a header sitting above the TEN that disprove it** — the same number
  pre-registered as the run's prediction, counted from memory while looking at the functions.
  Caught by the runner's `running 10 tests`, not by re-reading. Step 5's own close-out lesson
  (*re-measure every testable sentence in your own new header*) reproduced one step later by the
  person who wrote it down. **The repair is not a corrected number: the count is now read off the
  file's own source (`include_str!` on itself) and pinned**, so adding a test fails there instead
  of leaving a stale figure in prose. A count that describes an artifact from memory is a claim; a
  count read out of the artifact is a gate. [[rust-port-slice-z-step1]]
- **A prediction that comes true and is never marked is the quiet half of a bad ledger.** AD's
  six-step count held and was scored; its own cited precedent — AC's "SEVEN steps" — also held and
  no line in AC's ledger ever said so.
- **And I asserted that absence from two regexes, one of which matched a DIFFERENT slice's P7.**
  Caught in review, then established properly by reading the ledger block: AC's P7 bullet is one
  sentence and the next line starts P8, where P6/P8/P9 all carry continuation lines. **An absence
  is established by reading the neighbourhood, never by a pattern that fails to match** — a grep
  for the tag would have returned the same verdict for the wrong reason. Same shape as the
  nine-vs-ten miss, one document over: I re-measured the TEST FILE's header and not my own longer
  close-out text. **Re-measure every new text you wrote this step, not just the one you remember
  writing.**

Related: [[rust-port-slice-ad-step5]] (the step this closes out from), [[rust-port-slice-ad-preflight]]
(where the § (vi) reasoning and P6 were written), [[rust-port-slice-ab-step5]] (the dispatch-gate
precedent, its rule and its exception), [[rust-port-slice-ac-step7]] (laundering, first measured).
