
#### 5.34.3 STEP 3 — RUNG 82's NINE READERS, AND **A VOID's ORDER OF CHECKS IS VISIBLE ONLY WHERE BOTH BRACKET ENDS ARE EMPTY — AND AT `r = 1.0` THE DEFAULT BRACKET's TOP END IS NOT: RUNG 82's *"NO FOUR-LOOP POINT AT ALL"* HOLDS AT THE `tau_f` ITS SUITE CHECKS, NOT AT `0.30`** (2026-09-29)

`scan_cells`, `hats`, `threshold_scan`, `scan`, `bisect`, `threshold_law`, `threshold_row`,
`threshold_reference` and `threshold_terms` in `rust/src/threshold_law.rs`, and
`rust/tests/slice_aj_threshold.rs` — **12 gates, every returned value bit for bit AND in Python's
own key order**, against `rust/oracle/probe_slice_aj_step3.py`'s output
(`slice_aj_step3_pypy.tsv`, PyPy), step 2's flat `path <TAB> token` format with one addition: a
NaN is written `f:nan` (its sign and payload are invisible to Python code, so not pinned). All
single-definer: **no table, no builder, P6 holds** (`TripleHooks` at 18). The first eleven gates
were green on the first run, with no warnings. Rung 81's `dict_put`, `key_of`, `authority_of`,
`b_max_of` and `lag_of` became `pub(crate)` — one copy each.

#### (a) THE PORT

* **Python's `**kw` is `ScanKw`** (every `_threshold_scan` argument but `tau_f`); the two `key`
  lambdas are `key_fuel` and `key_residual`, passed as `Fn(&ThresholdScan) -> bool`.
* **Four functions return different dict SHAPES and each shape is its own variant** — `Bisect`
  (four: V1 at an end, V3, V1 mid-bisection, ok), `ThresholdRow`, `ThresholdReference`,
  `TermsAt` (two each). `void` is the FIRST key of a void dict and the FIFTH of `_bisect`'s ok
  one; the flattening pins each order.
* **The advisor's faithfulness list, all honoured before the first run**: `tau_hat_min` is
  `min(hat)`, not the argmin's `hat`; the argmin is the FIRST minimum (strict `<`); the `*_bind`
  lookups match `c.s == arg.s` by float equality over ALL cells (Python's own comparison); kappa
  is over all cells behind `tau_f > 0.0`, via `two_spool::round6`; `_bisect` scans both ends
  before testing either and re-scans at `b` on the ok branch; the row's void reason is `meas`'s
  first; `kappa_seen`/`all_kappa_pure` run over ALL rows, the rest over live ones; the walls
  iterate `sorted(phi_lims)` at a hard-coded `tau_gov = 0.05` while the field returns the caller's
  order; truthiness (`if a[…]`, `pred not in (None, 0.0)`) and `is not None` (`gap_falls`,
  `ratio_rises`) are ported as two different tests; a `None` on P5's `b` side is `TypeError`, a
  zero denominator `ZeroDivisionError` (`py_div`).
* **`_scan` is `threshold_law::scan`**, called fully qualified through the module's own path.
* **The V3 message goes through `py_g`** — and the default bracket CANNOT show it: `%g` and
  Rust's `{}` both print `[0.004, 0.3]`. The advisor's catch; `bisect_v3_g` runs `lo =
  0.0123456789` (`%g`: `0.0123457`), and K5 below is killed by that reading ALONE.

#### (b) THE READINGS

| reading | settings | why |
|---|---|---|
| `law`, `ref`, `terms` | `test_rung82.py`'s three fixtures, verbatim | the shipped values |
| `scan_ctl`, `scan_v1` | the suite's two direct `_threshold_scan` calls | rung 81's cell; the EMPTY window (every Option `None`, `kappa = []`) |
| `bisect_v3_hi`, `bisect_v3_lo` | `test_V3_censors_the_wall_on_both_sides`, verbatim | V3 `below` / `above` |
| `bisect_v3_g` | V3 at `lo = 0.0123456789` | the `%g` call site |
| `bisect_v1` | `r = 1.0` on `[0.004, 0.05]` | added AFTER the sweep — (d) |
| `law_fine` | `rs = (0.25, 1.0)`, `ds_fine = 0.0025` | the step-control leg (`ds_stable = True`) and a VOID row |
| `ref_void` | `r = 1.0` | `threshold_reference`'s void shape |
| `terms_void` | `phi_lims = (0.760, 0.750)`, `tau_govs = (0.05,)`, `n_bisect = 4` | sorted iteration vs as-passed field, a void `at`, `n_void = 1`, `p4 = p5 = None` |

**UNREACHED on this rig, recorded rather than implied by a green sweep:** the **`nan`** of
`tau_star_eff` — it needs a kappa-IMPURE reference, and kappa is `[3.0]` on all 45 marches of a
search over `r ∈ {0.2…0.9}`, `tau_f ∈ [0.004, 0.3]`, both walls, both governor clocks and a halved
`ds` (rung 82's own E5: every riding point is in RELEASE); `n_slope_excluded > 0` (0 on the same
45); `_bisect`'s mid-bisection V1; a row whose measured bisection is ok while the fixed point's
voids; `ds_stable = False`; a reference with no hats (`fwd = None`); `grows_* = None`; and
`tau_f = 0.0`'s empty kappa.

#### (c) THE INJECTION SWEEP — 22 injections, predicted in writing first (`W:\temp\claude\slice-aj-step3\predictions.md`), `--no-fail-fast`, **21 of 22 verdicts right**

| # | injection | result |
|---|---|---|
| K1 | `tau_hat_min := arg.hat` | SURVIVED — predicted: kappa pure ⇒ `tau_eff` constant over hats |
| K2 | argmin keeps the LAST minimum | SURVIVED — no tie |
| K3 | kappa over hats, not cells | SURVIVED — no slope-excluded cell |
| K4 | `at_hi` cached from the loop, no fresh scan | SURVIVED — the march is deterministic |
| K5 | V3 message with `{}` | **KILLED — `bisect_v3_g` ONLY** (predicted) |
| K6 | `gap_falls` by truthiness | SURVIVED — no gap is exactly 0 |
| K7 | walls iterate unsorted | **KILLED — `terms_void` ONLY** |
| K8 | `phi_lims` returned sorted | **KILLED — `terms_void` ONLY** |
| K9 | void reason fix-first | SURVIVED — both void with the same V1 string |
| K10 | `all_kappa_pure` over live rows | **KILLED — `law_fine` ONLY** |
| K11 | an unrun step control reads as passed | **KILLED — `law` ONLY** |
| K12 | the fix bisection skipped when meas voids | SURVIVED — fix is read only on an ok meas |
| K13 | forward reading ignores kappa | **KILLED — `law`, `ref`, `law_fine`** |
| K14 | `ratio_bind` without `slope_r != 0` | SURVIVED |
| K15 | `_grows` sorts by `err` | **KILLED — `ref` ONLY** |
| K16 | straddle tested BEFORE the window | **SURVIVED — predicted KILLED (`law_fine`, `ref_void`). THE MISPREDICTION — (d)** |
| K17 | mid-bisection V1 returns the original bracket | SURVIVED — unreached |
| K18 | P4's `pred` off the FIRST live gov row | **KILLED — `terms` ONLY** |
| K19 | P5's `d_lag` by `is not None` | SURVIVED |
| K20 | walls at `tau_govs[0]` | **KILLED — `terms` ONLY** |
| K21 | `scan_cells` keeps trajectory ENDS | SURVIVED — no riding point at an end |
| K22 | `n_fuel` over cells, not `ride` | SURVIVED — same |

Eleven survivors are IDENTITIES on this plant (K1–K4, K6, K9, K12, K14, K19, K21, K22 — each
written down with its reason before it ran) and one is unreached (K17): none is a gate seat for
step 7.

#### (d) THE FINDING — **THE MISPREDICTION WAS A SCOPE ERROR IN RUNG 82's OWN DOCSTRING, INHERITED**

K16 was predicted killed on the reasoning *"at `r = 1.0` the window is empty, so the key is false
at BOTH ends and the reversed order returns V3 instead of V1"*. It survived. Measured in Python
(`W:\temp\claude\slice-aj-step3\k16_check.py`): at `r = 1.0` the windows are EMPTY at `tau_f =
0.004` and `0.05` — and **OPEN at `0.30`, 14 riding points, all fuel**. So on the default bracket
V1 fires off the LOW end alone; the reversed order finds `key(s_hi)` TRUE, no straddle failure,
falls through to the window test and returns the same V1. The prediction's premise was
`engine.py:22329`'s *"Anchor E1: at `r >= 1.0` there is no four-loop point at all"* — which
`test_rung82.py` checks at `tau_f = 0.05` only, and which is false at `0.30`. **The port's
misprediction and the docstring's over-claim are the same sentence**, read at a `tau_f` nobody
checked. The Python is not edited (a port step does not re-scope a shipped rung's prose); the
Rust `bisect` doc records it. Repair: `bisect_v1` — `[0.004, 0.05]` at `r = 1.0`, BOTH ends empty,
the only bracket that sees the order — added and the probe RERUN WHOLE (the other eleven readings reproduced line for line, 892 of 892), and K16
re-run against it: **KILLED by `bisect_v1` ALONE** — and the corrected tally is still 21 of 22 predicted, the miss recorded rather than re-scored.

#### (e) P1, READ EARLY AND NOT SCORED

Rung 82 has 259 Python body lines and landed at 1 004 Rust lines — ≈ 3.9 per body line (rung 81:
4.2; AI: 4.1). Its per-line share of P1 (259 / 653 × 2 700) is ≈ 1 070; per-field (× 5 700) ≈
2 260. **Rung 82 tracks the per-LINE model too.**

#### (f) BOOKKEEPING

Citation guard re-blessed 55/361/218 → **56/373/229**: eleven new anchors, each read by hand
against the printed line, none wrong at birth. Gate: full `cargo test --release` + full `pytest` —
the numbers are in the commit.

**Next: step 4 — rungs 83 + 84** (`corrector_law.rs`, `staircase_law.rs`; ten methods).
