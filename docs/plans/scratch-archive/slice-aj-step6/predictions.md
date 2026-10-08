# Slice AJ step 6 — predictions, written BEFORE any golden is compared

Written 2026-09-30 while the PyPy golden was still running. Nothing compared yet.

## Settled BEFORE this file (not predictions)

* The per-reading `x**2` vs `x*x` census (my first CPython design) is DEAD: one ordinary march
  (`_threshold_scan`, r = 0.35, tau_f = 0.05, phi arm) on CPython 3.14.3 evaluates `eta_c_at`
  1 291 982 times and 26 of them differ from the multiply spelling (`eta_t_at` 55 242, 0 differ).
  Every reading would be flagged, so it predicts nothing. Replaced by the REVERSE run (below).
* § 5.34 (ix) P2's CPython half (*"bit-exact on CPython except `authority_mask`'s `c0`/`c1`"*) is
  ALREADY REFUTED by step 4 (§ 5.34.4 (d)): 27 lines at two readings. Scored as refuted, not re-run.

## P-A — PyPy

Rust ≡ the PyPy golden on EVERY line (path AND token, in order), first run after the shared
flatteners compile. Steps 2–4's four binaries stay green after their converters move into
`tests/slice_aj_flat/mod.rs` (they re-verify the shared module against their own goldens).

## P-B — the REVERSE run (CPython, every literal `** 2` in `turbojet/` rewritten `x*x` at import,
## `builtins.sum` a naive left fold, installed before `turbojet` imports)

Byte-identical to the PyPy golden — EVERY line, the `_interp.sum_probe` sentinel included (it
reads 0.0 under the naive fold, as PyPy's). Any surviving difference names a mechanism other than
the square spelling and the sum — falsified by one line.

## P-C — the plain CPython golden vs PyPy

1. `p_scan_p4` and `p_secant_iter_v4` (step 4's readings, verbatim) DIFFER, and at exactly step
   4's 27 values: 24 `summands` values at ladder points 4 and 5 of `p_scan_p4`, and the second
   trace entry's `F`, `g` plus `final_g` of `p_secant_iter_v4`.
2. `p_mask` / `i_mask`: some `c0` / `c1` lines differ (the `_charpoly4` sum path, § 5.34 (v)).
3. Other readings: COUNT NOT PREDICTED (the square flips are ~26 per march; which ones a march
   amplifies is not predictable from a census). Recorded, frozen into the gate by name.
4. **Every differing line is a FLOAT token (`f:`)** — no `i:`, `b:`, `s:`, `n`, `len:` or `keys:`
   line differs; no count, label, void string or structure moves. Falsified by one line.
5. The sentinel differs (1.0 vs 0.0) — by construction.

## P-D — the incidence arm is a CATCHER (injections, after the gate is green)

* J1: `scan_cells`' `split_march(..., kw.inc)` → `false`. KILLS the oracle gate; moves every
  `i_*` reading that marches through `scan_cells` (i_law, i_scan, i_read, i_shape, i_secant,
  i_edge_lo/hi, i_ladder, i_lattice, i_plant, i_classify) and NO `p_*` reading. Also: no other
  AJ binary fails (all their calls pass inc = false) — i.e. before this step J1 was invisible.
* J2: `authority_clock`'s `split_march(..., inc)` → `false`. Moves `i_clock` ONLY.
* J3: `authority_mask`'s → `false`. Moves `i_mask` ONLY.

## Closing gates — predicted before launch (after the last code edit)

cargo test --release --no-fail-fast: 182 result blocks / 1 884 passed / 0 failed (181/1 879 + the
one new binary's 5). pytest (python.exe -m pytest): 1 387 passed.

## C6 (advisor's check) — predicted before the injection

`residual_shape`'s ladder respelled `lo + i*step`: the two spellings are BIT-IDENTICAL at every
point of `p_shape_n4` ([0.016, 0.024], n = 4) and `i_shape` ([0.004, 0.05], n = 4) — computed.
So the mutant SURVIVES the oracle (0 readings move); "p_shape_n4 reaches C6" is FALSE.

## SCORED (2026-09-30)
- Closing gates: cargo 182 blocks / 1 884 passed / 0 failed, exit 0; pytest 1 387 passed (32:05), exit 0. EXACT.
- C6: mutant applied to corrector_law.rs:208, `every_token` gate PASSED (174 s) — SURVIVES, as predicted.
  Restored by checkout, blob 16002890ca247e460a2ea98692c20cc3a5c8b13a; full slice_aj_oracle 5/5 after.
