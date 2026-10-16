---
name: rung94-burner-hold
description: "Rung 94 (each throttle holds thrust or HP speed, burner floats; metal cap switch) process lessons — a pre-freeze check that prints the claim's own quantity pre-computes it; a grid-only search floor misses a near-fold root"
metadata:
  node_type: memory
  type: project
  originSessionId: 76bf8353-8e43-4b1e-a44d-8d50c63caa37
  modified: 2026-10-16T06:55:05.029Z
---

Rung 94 (2026-10-16): `burner_hold.rs` (search with a bisected floor, refusals naming the bound, metal cap), Controls
ends hold thrust/HP speed (`hold`, `hold_cap`). Anchor `5924f6a`. H1, H4 HIT; H2, H3 true but KNOWN before the freeze.
Decisions: [[rung93-94-decisions]].

**Lesson 1 — a pre-freeze check must print only what it was asked to check.** The advisor asked for a second-order
check of the 700 K cells and a reachability (solve/refuse) check. My `check2` also printed the hold-minus-fixed
flow-coefficient shifts at 1000/1500 K — which WAS claim H2 (and gave H3's signs). A side agent caught it after scoring;
the spec relabelled them "known before the freeze, no credit".
**How to apply:** a reachability / pre-freeze probe prints solve-or-refuse and the bracket quantities only; never the
quantity a registered claim is about. Same family as [[instrument-fed-by-what-it-certifies]].

**Lesson 2 — near a fold, a grid floor overshoots the root.** An HP 0.10 solves from 974 K (80 N) but not 950 K; a 50 K
grid would start at 1000 K (104 N), above an 89 N target, and refuse a root that exists. Bisect the floor. (Rung 93's
knob bug again: [[rung93-metal-temperature]].)

Also: a refusal case guessed by hand (LP 0.05 at the 1000 K start) was reachable; measure the bound before choosing a
refusal test case.
