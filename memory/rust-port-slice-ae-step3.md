---
name: rust-port-slice-ae-step3
description: "Slice AE step 3 — I made one needle two-sided and left its own CONTROL one-sided, on a file whose comments warn against the exact string; and a pre-registered `0 of 27` was falsified by the shipped Python suite it was written about"
metadata: 
  node_type: memory
  type: project
  originSessionId: 8a9664b2-e9e5-45ee-ab13-00346a218abd
  modified: 2026-09-03T08:25:23.102Z
---

Slice AE step 3 (rung 73, the 27 ported gates — `rust/tests/rung73.rs`, 1 073 lines, LF).
Plan § 5.29.3. See [[rust-port-status]], [[rust-port-slice-ae-step2]].

**THE LESSON: a control is an instrument too, and I gave it the treatment I had just decided was
not good enough for the thing it controls.** The gate ports Python's
`src.count("g_own + req - clip") == 1`. Two things do not transfer — the Rust body spells it
`(g_own + req) - clip`, and the port's own doc comment QUOTES that string, so a file-wide count
reads 2 where `inspect.getsource(<class>)` had a class-sized denominator. I split code lines from
comment lines and asserted BOTH. Then, one line later, I wrote the control — *"the rearrangement
`req + (g_own - clip)` is not written"* — as a **bare file-wide `== 0`**. It read **2**: both in
prose, at `:309` (why the association is pinned) and `:324` (`// Do not rewrite as …`, sitting
beside the expression). **`:324` was already in a grep output I had read and labelled DOC.** The
repair is strictly stronger than the `== 0` it replaces: the CODE count says the rearrangement is
not written, and the PROSE count of 2 says the counter can find the string at all — which a needle
absent everywhere can never demonstrate.

**Why to apply it:** the reflex is to give the *subject* of a gate the careful treatment and the
*control* a one-liner. But a control's whole job is to prove the instrument can see, and a bare
zero on an absent needle proves nothing — it is indistinguishable from a broken matcher. Ask of
every control the question you asked of the needle.

## Three more, all from measuring a prediction instead of citing it

* **P5 (`0 of 27` ported gates catch a folded float-identity branch) is FALSIFIED at 6, and
  falsified in PYTHON too.** Its premise — invisible to every RELATIVE bar — is true; the
  conclusion does not follow, because four of the bars are EXACT EQUALITIES and
  `tests/test_rung73.py`'s own docstring says why two paragraphs above them. The minimal fold is
  caught by **the same 6 of 27 in both languages, name for name including the parametrize split**
  — a bijection between the catch sets, which says more about the port's fidelity than "green"
  does. **A prediction can be wrong about the suite it was written about, not about the port.**
* **My first P5 number was 7 and the seventh was a COMMENT** — the sweep's replacement deleted two
  comment lines beside the branch, and one "catch" was the source-count gate firing on the
  `// Do not rewrite as …` line. Re-run minimally: 6. **Score a mutation confined to the thing you
  mean to mutate.**
* **P1's CONCLUSION and its REASON are two different injections, and only one is measurable here.**
  It says omitting rung 73's `integrate_fuel` asserts "passes all 27 … because no shipped rung-73
  test calls a rung-69 reader". Deleting both asserts dies at **2 of 27 in Rust and the same 2 in
  Python** (the two `refuses_…` gates drive them by hand and never touch a rung-69 reader). The
  narrower injection P1 names — asserts PRESENT but unreachable — is step 5's. Scoring them as one
  would have reported "P1 falsified" having measured something P1 does not describe.

## The two seats, and the one discharged by git rather than re-driven

Step 2 pre-registered M17–M21 for "both seats" here. Step 3's whole diff is ONE new test file
(`git diff --stat -- rust/src turbojet` empty), so the VALUE seat's answer is a function of
(shipped source, mutation) and both are byte-identical to step 2's — **premise verified by git
identity, not argued**, which is more than step 2 could offer for its own CRLF transfer since it
had no commit to compare against. Rebuilding a ~500-line dumper to reproduce a number that cannot
have changed is the weaker move. The NEW seat is the gate seat, 15 → 42.

**All five value-only rows are now CAUGHT (6/2/1/1/2 of 27), so there are ZERO misses** and step
2's *"the one-of-fifteen is not a hole, it is this step's shape"* is discharged. The third clause
— re-score a miss against Python's own 27 — therefore has no subject among M17–M21 and is recorded
as inapplicable **with the reason**; it is exercised twice instead, on P5 and P1a. A **declared
control** (the `reference` cell re-aimed at rung 72) fires 13 of 27, and a baseline run precedes
the sweep, so a report of nothing-but-misses is distinguishable from a broken runner.

## Smaller, kept because each was measured rather than assumed

* **`grep` says 28 `#[test]`, `cargo` runs 27** — the third instance in the phase, and **the first
  where the file's own header named the trap before the run**. The 28th is inside this file's
  sentence about parametrized tests landing "as two `#[test]` functions apiece".
* **Python's `psrc.count("self._reference(") == 4` ports as 1 SEAT and 4 CALLS.** The port hoists
  Python's four call sites into one closure (`core_ref`), which holds the single dispatch through
  the cell and is called at exactly the same four places. The port gains a bar Python cannot state.
* **P6 CONFIRMED by reading the shipped strings first.** Python's two inherited-refusal needles
  discriminate nothing (`"FORCED release"` reaches nine classes back to rung 43); the Rust messages
  open `"rung-72: …"`, so the ported gates name the OWNING rung — and assert by `fn_addr_eq` that
  the call went through rung 73's table, which is what makes "its own refusals ran first" a
  measurement.
* **`match=r"rung-73.*origin"` is a regex meeting a substring matcher.** Split into two literals
  **plus an ORDER assertion**, which is the `.*`'s actual content.
* **The first line-ending reading of the step was wrong**: Git Bash `grep -c $'\r'` reported every
  line CRLF in all seven files; the byte-level census disagreed and reproduced § 5.29.2 (f)'s
  after-state exactly (167 → 168 files, 146 → 147 LF, 21 CRLF unchanged). See
  [[windows-tooling-file-hazards]].
* The sweep is derived from `mutate_step2b.py` and never from `mutate_step2.py`, which § 5.29.2 (g)
  named as *"exactly what step 3 would copy"*.
