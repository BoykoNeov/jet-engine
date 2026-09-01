---
name: rust-port-slice-ad-step2
description: "A convention that a new enum variant breaks the build held at 7 of 20 sites, and my own gate compared the plant against the function that produced it"
metadata: 
  node_type: memory
  type: project
  originSessionId: f0d438fb-b4ba-467d-b0e5-f1af8995bcd2
  modified: 2026-08-31T20:25:04.543Z
---

Slice AD step 2 (rung 72 Rust port) shipped the six-state march, `PointExtra::Shared` (30 keys),
the `Authority` label, nineteen widened reader sites, and `rust/tests/slice_ad_march.rs` —
**13 gates**, **10 of 10 mutations killed on the second run**. Plan § 5.28.2.

**THE LESSON: a "the compiler will catch it" convention is a claim about every site, so measure how
many sites actually have it.** `cross_extra`'s doc says the arms are spelled out *so the NEXT
`PointExtra` variant breaks the build here*. Adding a variant measured it: **7 of 20 `match .extra`
sites in `src` are exhaustive; 13 carry a `_ =>` wildcard.** The compiler stopped at 6. The other 13
compiled silently and would have given a rung-72 trajectory a default (3 sites), a **silent drop
inside a FILTER** (3 sites — the quietest, because the reader then computes over an EMPTY set and
reports perfect tracking), or a refusal Python does not raise (7 sites).

**AND THE PROBE THAT COUNTED THEM WAS WRONG TWICE, BOTH TIMES LOW** — same site, a `match` written
on ONE line: first the regex anchored `_ =>` to a line start, then the repaired version scanned a
body starting at the line *after* the `match`, which for a single-line match is empty. An instrument
that undercounts silent fallbacks is the exact defect it was built to find.

**MY OWN GATE COMPARED THE PLANT WITH ITSELF, AND ONLY THE MUTATION SWEEP SAW IT.** The authority
gate asserted `recorded_label == authority(g_fuel, g_gov)` — the label against the very function
that produced it — so inverting `gf > gr` inside that function passed, and the counting half
survived because inverting a bijection just exchanges two non-zero counts. **Rung 72's own spec
names this pattern and says the only defence that works is a gate that fails when the two laws are
the same one**; I reproduced it in a gate written to check that rung. Spell the expectation out in
the test; never route it through the function under test.

**Three more, all measured before writing:**
- Every `self.X` the march reaches (26) classified first. One flag, `_instant_fuel` — **SIBLINGS,
  not an override**, and my predicate was `defined >= 2` with no MRO check: the same defect
  [[rust-port-phase7-preflight]] records the phase-5 census making.
- The two `except AssertionError: break` arms are **reachable** (3 of 38 marches end early) and
  **four distinct (raiser, message) pairs** fire, so a `Result` collapsing them loses information.
- A bar of mine failed for the physics being right: `sum` marches 84 points where `max` marches 341
  — **in Python too**. Rewritten as a same-run comparison instead of a guessed absolute.

**How to apply:** when a comment says the compiler will catch the next case, add the next case and
count which sites go red. And when gating a recorded label, derive the expectation independently in
the test — a gate that calls the function under test to build its own expectation cannot fail.
