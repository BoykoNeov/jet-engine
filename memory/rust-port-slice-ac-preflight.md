---
name: rust-port-slice-ac-preflight
description: "Slice AC's pre-flight — the phase's cell predicate is by NAME, so seven slices of correctly-emitted columns could not tell an override from a name reused, and AC's only claimed cell is one"
metadata: 
  node_type: memory
  type: project
  originSessionId: 67ab7e3b-ddec-4b0d-822d-77d5d6d5435f
  modified: 2026-08-28T16:58:42.852Z
---

Slice AC (rungs 70/71, `CrossSplitTransient` + `FullSplitTransient`) was pre-registered on
2026-08-28 off fourteen probes, written to `docs/plans/todo-rust-port.md` § 5.27.

**THE PHASE'S CELL PREDICATE IS BY NAME AND HAS NEVER CHECKED SUBSTITUTABILITY.** Every slice
since § 5.19 emits its "cells ADDED" column with the predicate *new here AND overridden above*,
and every slice has been right — seven in a row, name for name. Slice AC's column said one cell,
`split_gains`. It is not a cell: rung 80's same-named method drops four parameters and adds five,
and rung 70's **own inherited caller** proves it live — `rung67_control` on a rung-80 machine
raises `TypeError: split_gains() got 3 unexpected keyword arguments`, against a control on a
rung-70 machine that returns. Two different functions sharing a name cannot share a `fn` pointer,
so **slice AC adds ZERO cells**, `TripleHooks` stays ten fields, and the `E0063` tripwire slice AB
deliberately left *addressed to slice AC by name* will never fire here.

I found it by starting to write step 1 — reaching for rung 80's signature to type the new field.

**A predicate that is wrong emitted seven times reads like seven confirmations.** The measurements
were all correct; the question was wrong. That is [[rust-port-phase7-preflight]]'s own lesson one
level further out: there, a section checked *defined exactly once* and never *overridden at least
once*; here, seven probes checked *overridden at least once* and never *by a body that could stand
in*. The repair is the predicate — **a cell is a name that is overridden AND substitutable** — and
a phase-wide sweep of all 358 override pairs then found exactly one more: `_legs`, a SHIPPED cell
from slice W whose rung-77 overrider takes a disjoint parameter list (booked to slice AH).

**Why:** the port's whole architecture is a `const` table of function pointers, so "is this a
cell" is really "can one caller dispatch between these two bodies". Name equality is a proxy that
holds until a rung reuses a name — and a ladder 31 classes deep will reuse names.

**How to apply:** before adding a field to a shared table, read BOTH bodies' signatures and, where
a caller exists, run the parent's caller against the child. **And repair the comments the old
predicate wrote, in the same pass** — two shipped Rust doc comments stated it as fact
(`slice_ab_cells.rs`'s tripwire named slice AC as its eleventh field's author; `LeverHooks::legs`
gave *"Overridden at rung 77"* as its ONLY reason). The tripwire's addressee was deliberately
replaced with NO slice rather than with the next letter: the next letter comes from the same
column, so naming it would repeat the error one addressee over. Three more things this pre-flight
wants carried:

* **The probe that repairs a predicate needs its own predicate checked first.** My sweep's first
  writing called any two ladder classes an override pair (so siblings appeared as overrides — the
  scoping error the phase-6 pre-flight already recorded, made a third time) and compared parameter
  NAMES (so a pure rename read as incompatible). Twelve "non-substitutable" names became two.
* **An impossible pair of numbers in one row is the cheapest self-check there is, and it only works
  if both are printed.** Two separate probes emitted `MAX NESTING DEPTH` of 158 and 37 beside an
  `OVERWRITE` of 0 — a global counter summing every machine. Per instance it is 1. Same artifact
  `probe_ab11` recorded at slice AA.
* **A docstring saying "single process" is not `-n 0`.** A probe printed fifteen zeros because
  `pytest.ini` carries `-n auto`, and the same run failed a shipped gate by wrapping a method that
  `inspect.getsource` reads — so an unlucky reading would have booked the probe's own damage as a
  finding.

The slice's own second headline, kept because it decides a gate shape: swapping rung 68's
`_triple_laws` into rung 70's slot makes the reader return **successfully with zero rows**, so
every value key agrees and a value-diff dispatch gate passes on an empty table. *A cell whose
output is a SAMPLE can break by changing the sample's SIZE rather than its values.*

Related: [[rust-port-slice-ab-step5]], [[rust-port-ported-test-vacuity]],
[[rust-port-guessed-census-bars]], [[rust-port-slice-k]], [[run-tests-below-normal]].
