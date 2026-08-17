---
name: rust-port-slice-n-step3
description: "Slice N step 3 (rung 55/56 matcher) — a carrier claim verified on one hook said nothing about the next hook's, and a constant measured DEAD still had its spelling wrong"
metadata: 
  node_type: memory
  type: project
  originSessionId: 2448703c-b841-42a6-a3c2-9f82686dab08
  modified: 2026-08-17T10:45:39.702Z
---

Slice N step 3 of the Rust port shipped `StageStackCore` (rungs 55/56's matcher) on 2026-08-17.
Two process lessons, both about claims that were verified on the wrong object.

**A carrier claim checked on ONE hook says nothing about the next hook's.** Step 1's whole point
was that reading a method body tells you what state it READS, not what the state's CARRIER costs
— and the plan then made exactly that mistake again, one level down. It worked out where
`at_setting`'s state lives and never asked where the *efficiency-loop* hook's state has to live.
That hook's `self` is the INNER core, which cannot see the outer object, so a fourth gated-code
edit landed at step 3 in a file step 1 never opened. The plan's "step 1 is the whole revert unit"
row was refuted; the narrower prediction it was written from survived. **When a slice states a
lesson about one hook, re-run it over every other hook in the same slice before shipping.**

**A constant measured DEAD still has to be SPELLED right.** The earlier slice measured a
bisection cap never reached and concluded "the shadow is not live", then hard-coded the literal —
in the exact spelling the pre-flight had explicitly forbidden — and attributed the shadow to the
wrong rung on the way past. Two different senses of *live*: a cap that is never hit still decides
which constant the body NAMES, and the wrong name is wrong wherever the grid later moves. Same
shape as [[rust-port-slice-n-step2]]'s *a dead guard's threshold is worth more than its count*.

Also: the exhaustive-`match` arity pin from the previous slice FIRED (compile error before any new
gate was written) — which is the only way a reader ever learns such a pin was load-bearing rather
than decorative. And the dispatch gate's two halves were both INJECTED-and-watched-to-fail rather
than trusted, because the pointer clauses are structurally blind to the failure that matters
(a sibling rebuilt without its stacks). See [[rust-port-location-keys-refute]] and
[[rust-port-ported-test-vacuity]].
