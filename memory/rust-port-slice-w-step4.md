---
name: rust-port-slice-w-step4
description: "Slice W step 4 — the cross-interpreter arm caught a divergence in Python's own sum(), and the four keys that looked like a port defect were a predicate I had renamed myself"
metadata: 
  node_type: memory
  type: project
  originSessionId: 75ae92c3-7944-4f18-bafd-48b3783bfa66
  modified: 2026-08-26T14:09:13.308Z
---

Slice W step 4 of the Rust port (rungs 62/63, the oracle), 2026-08-26. `dump_slice_w.py` +
`slice_w_oracle.rs`: **9 422 keys bit-exact against PyPy AND CPython 3.14**, ten sections, zero
tolerance tiers. Five process lessons, four of which cost a run.

**A CROSS-INTERPRETER ARM CAN FAIL FOR A REASON THAT IS NEITHER THE PORT NOR A TOLERANCE.** Seven
keys disagreed between PyPy and CPython. Cause, measured: **CPython 3.12+ uses Neumaier
COMPENSATED summation in `sum()` for floats and PyPy's is naive left-to-right**, so
`sum(vals)/len(vals)` over a constant value returns exactly `0.1` on one and three ULPs below it
on the other. Not a port defect — the crate matches PyPy, the interpreter every golden comes off —
and no shipped gate reads the affected value. **How to apply:** when the second arm disagrees,
ask whether the DIFFERENCE IS IN THE LANGUAGE before touching either side; then carry it as a
declared RULE with a liveness check (the rule must cover a pinned number of keys AND at least one
drift must land inside it, or a suppression that suppresses nothing goes stale unnoticed). It also
falsified a claim in the shipped Rust's own comment — [[rust-port-slice-l-step3]], caught for the
first time by an INTERPRETER arm rather than by a reader.

**WHEN A PORT DELIBERATELY RENAMES A PREDICATE, AN ORACLE KEY MUST NAME WHICH ONE IT ASKS FOR.**
Four keys flipped `rust 1 vs cpython 0`. Python's `_is_armed()` is SCHEDULED-only; the Rust
renamed the composite guard `_is_armed() or vsv_lp or vsv_hp` to `is_armed()` and Python's method
to `is_scheduled()`. Both are right and the key was asking two different questions. The
instructive part is the arithmetic: **over a hundred such keys agreed and exactly four flipped**,
because the two predicates coincide on every input except one shape — a constant setting with no
schedule — that existed in the file only in a sub-section added for another reason. The agreeing
hundred is what made the four read as a defect. Both predicates now have their own key.

**A CHECKLIST ITEM CAN NAME A MACHINE THE READERS REFUSE.** The plan booked "run all eight
inherited readers on a bleed-armed machine". Six of the eight assert a STATOR arming, so as
written the section would have raised six times and carried two readers while reading like eight.
Fixed by arming both devices and **recording the six refusals as their own keys** (with the two
non-refusals beside them, so the row is a measurement). A second refusal shape turned up the same
way: one reader trips its own clamp audit at the default margin — recorded as a key, not tuned
away.

**FIVE OF SEVENTEEN COUNTERS WERE ZEROS NOBODY HAD LOOKED AT.** The census workload builds no
siblings, so every sibling-constructor counter read 0 on all five armings — a dead counter and an
untaken path are the same character. A SECOND workload was added on which all five are non-zero,
and the gate asserts both directions. Related: **the Python wrappers had to move from the INSTANCE
to the CLASS**, because the Rust counters are a thread-local global that fires on siblings too; an
instance patch is invisible on a workload that builds none and wrong by a factor on one that does.

**TWO RUNS LOST TO PLUMBING, AND NEITHER FAILURE NAMED A KEY.** A `2>&1` merged the dump's stderr
key-count line onto the END of the last data line, and a signed `-1` went into a `key<TAB>u64`
file. Both surfaced as `ParseIntError { kind: InvalidDigit }` from the LOADER — before any
comparison exists, so the message names nothing. **How to apply:** never redirect a dump's
diagnostic stream into its data file, and make the loader's error print the offending LINE.

See [[rust-port-slice-w]], [[rust-port-slice-w-step3]], [[rust-port-slice-w-step5]],
[[rust-port-decided]].
