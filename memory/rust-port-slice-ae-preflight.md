---
name: rust-port-slice-ae-preflight
description: "Slice AE pre-flight — a prior sweep SAW both name reuses, named them, and cleared them on the wrong property; and four of my own probes were void before anything was written down"
metadata: 
  node_type: memory
  type: project
  originSessionId: 9e916a88-d011-4715-b32a-cf1123e2800b
  modified: 2026-09-01T15:28:10.095Z
---

Slice AE (rung 73, `AppliedReferenceTransient`), `docs/plans/todo-rust-port.md` § 5.29, ten
probes in `M:\claud_projects\temp\rust-ae\`.

**THE LESSON: a sweep that finds a hazard, puts it in a named class, and CLEARS it is more
dangerous than one that misses it.** Slice AC's phase-wide substitutability sweep found
`_with_ref` (69→73) and `_with_coord` (74→79), filed them under **RENAMED — "same arity, one
parameter renamed … the shipped port already holds these in ONE pointer, correctly"**, and moved
on. Its predicate compared **signatures**. Measured here: the two bodies write **disjoint
fields** (`_ref` vs `_ref_law`), and rung 69's own inherited caller **raises** on a rung-73
machine, with a passing control. It is `split_gains`'s name-reuse shape with the signature filed
off. A miss leaves an open question; a **cleared** hazard closes it, and everything downstream
then cites the clearance rather than re-measuring. Only the classification was wrong — AC's count
is fine, and saying so is part of the correction ([[rust-port-slice-ac-step6]]).

**Why it did not become a shipped bug:** the port had already reached the right structure from
the right observation (`reference_split.rs:279` dispatches the *setter* through the cell *because*
rung 73 writes a different field). **The plan and the code disagreed, in the plan's favour, and
nobody diffed them** — [[rust-port-phase7-preflight]]'s lesson with the two documents swapped.

**Four of my own instruments were void, and every one was caught by a control rather than by
review** — this is the part to carry forward:

* a reader census restricted to `self.NAME` scored a method at **0 readers** when it has **11
  call sites**, all `m.NAME` on a rebuilt machine — and AD's independent filter scored the same 0
  by a *different* mechanism, so two instruments agreeing is not corroboration;
* a field spy printed a value that was the **class default**, not a mutation (fixed with a
  sentinel no default can be);
* the behavioural reading built on it was **blind**: 0 keys moved under the wrong machine AND 0
  moved under a forced coordinate, so the zero was never evidence — verdict downgraded to
  UNDRIVEN and the claim withdrawn ([[rust-port-slice-w-step3]]);
* two probes ran with wrong keywords and raised on both arms, caught by an `assert calls > 0`
  bar inherited from AD.

**And a "seat exists" is not an observable.** AD booked a cell forward as unreachable; the seat
does exist here, but the row was only written after holding the machine fixed and swapping **only
the pointer** — 32 keys move, 70 vanish, and the rung's own headline number `F_r` goes
−1.000000000002735 → 0.0. Asking *does shipped code sit in the seat* is the weaker question;
*does a discriminating input exist* is the cell question.

**And two predictions written in one sitting were jointly impossible.** One said a new hook
field, the other said re-aiming the existing one is the defect -- but that slot has been
shipped since slice AB, so both could not hold. Re-reading never caught it; naming the object
they disagreed about did, and then one measurement settled it (0 reads of the parent field on
a child machine, against a liveness control at 1). **The check for a contradiction between two
claims is not proof-reading -- it is naming what they disagree about.** The same sitting also
ran a needle census over the 8 classes I expected to match and got two counts wrong; widened
to all 58, one needle went from 5 classes to 9 and reached back thirty rungs.

Related: [[rust-port-slice-ad-step6]], [[rust-port-status]], [[rust-port-guessed-census-bars]].
