# Slice AJ pre-flight — predictions, written BEFORE each probe runs

Date 2026-09-29. Rungs 81–84 (`AuthorityClockTransient` … `StaircaseLawTransient`).

## Already measured before this file (probe 1, AST census — `p1_census.out`)
- 24 methods over 4 classes, every one single-definer in the MRO; 0 ADD, 0 SWAP.
  (`_scan` has a second definer, `VariableStatorMatcher`, which is NOT an ancestor — a name reuse
  across unrelated hierarchies, not a cell.)
- Sizing: 1 271 total / 24 methods / 653 body (81: 193, 82: 259, 83: 78, 84: 123).

## P-M: the booked mutation (delete `SplitWallTransient.at_lever` in a COPY)
- M1 (positive control): the mutated copy is the module loaded; `'at_lever' in
  SplitWallTransient.__dict__` is False; `StaircaseLawTransient.at_lever` resolves to rung 79's.
- M2: under the mutation, every rig built at rungs 80–84 is a `StateCoordinateTransient` by class
  (control: `SplitWallTransient`) — the rebuild is entered on every reader that marches.
- M3: the suites `test_rung80.py`…`test_rung84.py` are ALL GREEN under the mutation. Reason:
  `SplitWallTransient._shared_rig` re-writes `_sm_air` onto the rig it gets back, and every
  rung-80–84 reader reads the rig only through rung ≤ 80 methods that are not re-defined at 80
  except `at_lever`/`_shared_rig` (Rust step 7 measured this through rung 80).
- M4: the kernel dumps r80, r81, r81m, r82, r82r, r82t are BIT-IDENTICAL control vs mutated
  (every key, exact repr). Kernels r83 / r84 do NOT exist (the booking named them).
- M5: no rung-81+ method is ever called on a rig (a machine built by `at_lever`); every rung-81+
  call's receiver is the top-level machine the test built.
- Uncertainty: a rig that calls `m.at_lever(...)` AGAIN (a rig-of-a-rig) would, under the
  mutation, lose `_sm_air` a second time, because rung 79's `at_lever` does not carry it —
  if any reader does that, M3/M4 break. I do not know of one; I predict none.

## P-D: runtime dispatch census (control run)
- D1: the (definer, method) set reached from test_rung81–84 contains ONLY methods of rungs ≤ 84
  already ported to Rust (no un-ported inherited function), and `_quad_gains_at` resolves to rung
  72's (single definer? — to be checked).
- D2: the only rung-81+ methods not reached by the suites: none predicted.

## P-A: arithmetic surface
- A1: `round(x, 6)` / `round(s, 9)` via the crate's format-and-parse helpers agree bit for bit with
  Python on every argument the suites produce (0 mismatches).
- A2: `round(edge/ds)` needs ties-to-even; whether any argument is an exact .5 tie: predicted NONE
  (edge is on grid ⇒ ratio near an integer).
- A3: `"%g"` void messages: 4 sites; the crate has NO `%g` formatter (grep) ⇒ new helper.
- A4: the "9 places cannot merge two march points" claim holds (0 merges).
- A5: `authority_mask`'s `rate = sum(1.0/t ...)` and `_charpoly4` are the only float sums on the
  path; the CPython exemption is pre-registered as those paths (the fingerprint says r81/r82* are
  exact across interpreters; r81m carries an absolute leg).

## P-V: voids / refusals
- V: every V-code in rungs 82–84 is reachable on some shipped call; the suites assert only a
  subset. Predicted counts to be tabulated, not guessed.
