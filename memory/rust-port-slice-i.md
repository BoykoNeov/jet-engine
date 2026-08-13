---
name: rust-port-slice-i
description: "Slice I (rungs 31/33) — a bare `except` means the question is REACHABILITY, not which sites you listed; and a count without its grid is not a measurement"
metadata: 
  node_type: memory
  type: project
  originSessionId: 17530476-3625-4041-acc2-76744b135c6e
  modified: 2026-08-13T09:50:32.691Z
---

Phase 5 slice I (the off-design matcher) shipped 3951/3951 bit-exact. Three process lessons,
all about the SCOPE of a claim rather than about its content.

**A bare `except AssertionError` makes the question REACHABILITY, not enumeration.** The
pre-registered fallible-path design named three assert *sites* and decided, from a dump of
8,289 calls, that one of them never needed to be fallible. The dump instrumented
`freeze_equilibrium`. But the burner's equilibrium branch reaches the same solver by a
*different route* that never passes through it — so the sample was blind to that route by
construction, and the raises it missed were on cells that go on to return a real answer. Same
shape as [[rust-port-oracle-cannot-see-a-missing-gate]], one level down: the instrument was
placed where the hypothesis was, not where the control flow is.

**Why:** the advisor caught this before any code existed, from reading the Python's `except`
clause rather than my list of sites. The list was the artefact; the `try` scope was the fact.

**How to apply:** when porting anything a caller CATCHES, derive the fallible set from what the
`try` block can reach, then measure both edges. Two guards here were reachable and left as
panics on evidence (0 fires in 111,775; and 6 fires in 225,410 that provably abort their cell
rather than being marched past — established as a *superset* argument, not as an absence).
Make every conversion an additive `try_` twin whose panicking original delegates, so no
already-gated caller moves.

**A COUNT WITHOUT ITS GRID IS NOT A MEASUREMENT.** The pre-registration quoted "930 low / 616
high raises" without recording the sweep behind it, so it could not be reproduced, could not be
ported as a prediction, and had to be re-measured (550 / 46 on the same 3×6×7 shape). The grid
now lives in the oracle file beside the counts. This is the fourth distinct way this port has
been bitten by a claim whose scope was left implicit — see also [[rust-port-guessed-census-bars]]
and [[rust-port-phase5-preflight]].

**A TOLERANCE AND A COUNT ARE DIFFERENT CLAIMS.** Both Python suites state rung 31's pin and
rung 33's inversion as tolerances, which cannot separate "holds" from "nearly holds" and puts
"constant" and "varies" in different currencies. Counted over distinct `f64` bit patterns they
become one statement: the calorically-perfect choked case is **1 pattern across 26 cells**, its
subsonic case **4 across 4**, the reacting gas collapses nowhere. Gate BOTH halves together —
the constant half alone passes on a solver that stopped responding to its inputs, and the
varying half alone passes on a sweep too narrow to resolve anything. Same instinct as
[[rust-port-shape-keys]] and [[rust-port-location-keys-refute]].

**Also settled here, and load-bearing for phase 6:** the virtual hook for `solve_turbine`
shipped WITH the slice rather than being retrofitted, because rung 34 overrides it from the
next phase — the requirement [[rust-port-phase5-preflight]] discovered by widening its census.
The gate exercises the hook by substituting a counting wrapper, so a bypassed hook fails loudly
instead of silently returning the parent's answer.
