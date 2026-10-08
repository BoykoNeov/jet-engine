
#### 5.34.2 STEP 2 — RUNG 81's FIVE READERS, AND **THE TWO FUEL-CLOCK LOOKUPS THE PORT KEEPS APART ARE ONE FUNCTION ON EVERY `demand` POINT: RUNG 74's ARGUMENT SWAP IS EXACTLY THE CHANGE OF VARIABLES, SO NO VALUE GATE CAN SEE WHICH ONE A READER CALLS** (2026-09-29)

`central`, `criterion_at`, `authority_clock`, `tau_f_inert` and `authority_mask` in
`rust/src/authority_clock.rs` (815 lines), and `rust/tests/slice_aj_clock.rs` — **5 gates, every
returned value bit for bit AND in Python's own key order**, against
`rust/oracle/probe_slice_aj_step2.py`'s output (`slice_aj_step2_pypy.tsv`, 16 119 lines, PyPy).
All single-definer: **no table, no builder, P6 holds** (`TripleHooks` at 18). Green on the first
run, no warnings; 23 s in Rust against 72 s on PyPy for the same five readings.
`split_wall::py_max2` became `pub(crate)` — one copy of Python's two-argument `max`.

#### (a) THE GATE's FORMAT, AND WHY IT CHANGED FROM AI STEP 4's

AI step 4 spliced word vectors into the test as constants. Rung 81's `clock` fixture alone scores
**1 054 interior cells** × 10 fields, so the probe writes a flat TSV instead — `path <TAB> token`,
a dict as `keys:N` then its items in insertion order, a list as `len:N`, a float as its IEEE bits —
and the Rust test writes the same lines from its own structs and compares them one for one. That
pins more than values: a field missing, extra, renamed or out of Python's order fails AT ITS PATH.
Every reading also reads the caller's rig back (`lag_coord`, `_sm_air = None`): `split_march`'s
scope restored the knob.

#### (b) THE FIVE READINGS, AND WHAT EACH IS FOR

| reading | settings | why |
|---|---|---|
| `clock` | `test_rung81.py`'s fixture | 36 grid rows, 1 054 cells, the six shared-wall control rows |
| `mask` | `test_rung81.py`'s fixture | the mirror cell + rung 80's matched clocks, stride 1, 80 interior cells |
| `clock_latched` | `coords=("demand-latched",)`, `tau_fs=(0.05, 0.20)`, `tau_govs=(0.05,)` | DEMAND-shaped points through `_criterion_at`'s ELSE branch — it tests the coordinate's NAME. Predicted and measured labels disagree on most points here (`fuel` predicted 26, measured 140), so the branch choice is loud |
| `clock_single` | one `tau_f` (0.20), `clip` only | `_tau_f_inert`'s `None` column, and an EMPTY `control_clip_shared` (no control row meets `tau_f == 0.05`) |
| `mask_clip` | `coord="clip"`, `every=2` | the stride, and a VACUOUS mask (every cell governor-held) |

**Branch census, reached:** `worst_miss` `None` and `Some`; `_tau_f_inert` `None` and `Some`;
`control_all_gov` both; `vacuous` both; `predicted` `fuel` and `gov`; `fuel_side` empty and with
all three labels; `control_clip_shared` empty and full. **UNREACHED on this rig, recorded rather
than implied by a green sweep**: a riding point at a trajectory END (`n_edge > 0` — 0 on every
row, as `test_rung81.py` gates), an arrested row (`n_invalid > 0`), a skipped gains point
(`switch`/`regime`), a `tie`/`dormant` label in a cell, a quartic with other than ONE zero,
`ever_two_authorities`, a row with no interior cell (`agreement = None`), `all_fuel`, and
`_central`'s `i = 0` wrap to `traj[-1]` (Python's negative index, ported, never called).

#### (c) THE PORT

* **`_criterion_at` branches on `coord == "demand"`, never on the point's variant** — the advisor's
  first trap, and J1 below shows it is caught only by the latched reading.
* **`id(p)` → `riding4_idx`** (step 1's plumbing), ends counted as `n_edge` exactly where Python
  counts them; `census` and `n_riding4` over EVERY riding point, `cells` over interior ones.
* **`authority_mask` is `split_gains`' shape with five differences kept**: stride 1, ONE `cyc`
  field (two-argument `max` of the two left-to-right products), per-authority folds from `0.0`
  with Python's `max`, `abs(mask_leak)` REFUSING `None` (Python's `TypeError`) instead of
  `py_max_opt`, and the aggregates over the ALIVE arms. Gains through the RIG's table under
  `ShareScope("max")`.
* **Python's shapes throughout**: insertion-ordered census / `by_authority` / `fuel_cells` /
  `fuel_side` / `tau_f_inert` (a repeated key keeps its first position, last value), stable sorts,
  `zip`'s truncation in `_tau_f_inert`, the chained `tau_f == tau_gov == 0.05`, `ccensus` the LAST
  `tf == 0.05` row, `rate` a naive left fold.
* **`py_float_str`** for the `f"{c}@{tg}"` keys — Python's `str(float)` on a DECLARED domain
  (`[1e-4, 1e16)` and zero, where only the `.0` suffix differs from `Display`), panicking outside
  it; `py_repr`'s precedent. Gated by the keys `demand@0.02`, `@0.05`, `@0.2`, `demand-latched@0.05`.
* **Every refusal is Python's exception**: `KeyError` for a key the point lacks,
  `AttributeError` for `m.bleed_lim.b_max` / `lag.tau` on `None`, `TypeError` for `abs(None)`.

#### (d) THE INJECTION SWEEP — 16 injections, predicted in writing first (`W:\temp\claude\slice-aj-step2\predictions.md`), `--no-fail-fast`, **16 of 16 verdicts right, 0 mechanisms wrong**

| # | injection | result |
|---|---|---|
| J1 | `_criterion_at` branches on the `Demand` VARIANT | **KILLED** — `clock_latched` ONLY (predicted) |
| J2a | `authority_mask` uses `demand_tau(cap_fuel, w_fuel)` under `demand` | SURVIVED — predicted: the identity in (e) |
| J2b | `_criterion_at`'s demand branch uses `lag.tau(required_fuel, g_fuel)` | SURVIVED — predicted: the same identity |
| J3 | the mask ignores `every` | **KILLED** — `mask_clip` only |
| J4 | mask aggregates over ALL arms, not `alive` | SURVIVED — every arm valid |
| J5 | clip `lag_gap` in the demand form | **KILLED** — clock, latched, single |
| J6 | `census` drops trajectory ends | SURVIVED — `n_edge = 0` everywhere |
| J7 | the control passes the caller's `phi_air` | **KILLED** — clock, latched, single |
| J8 | `fuel_side` unsorted | **KILLED** — clock (`slow, fast, matched` by insertion) |
| J9 | `n_differing` summed, not maxed | **KILLED** — clock; the latched column has two rows, so sum = max there (predicted) |
| J10 | `ctl` drops the chained `tau_gov == 0.05` | **KILLED** — clock (`control_n_riding4` 1 → 3 rows) |
| J11 | the mask reads the CALLER's table | SURVIVED — predicted; step 7's P4 |
| J12 | `rate` summed in reverse | SURVIVED — one zero per quartic, far from the bar |
| J13 | the mask's `lag.tau` arguments swapped | **KILLED** — mask, mask_clip (`c0`/`c1`) |
| J14 | `predicted` with `<=` | SURVIVED — no exact tie |
| J15 | `ccensus` = the FIRST `tf == 0.05` row | SURVIVED — 0.05 appears once |

#### (e) THE FINDING — **THE "TWO LOOKUPS" ARE ONE FUNCTION ON A `demand` POINT**

The advisor flagged, before writing, that `_criterion_at`'s demand branch reads rung 74's
argument-swapped `_demand_tau(lag, cap_fuel, w_fuel)` while `authority_mask` reads
`lag.tau(required_fuel, g_fuel)` in either coordinate, and that merging them *"would look like a
fix"*. Both directions of the merge (J2a, J2b) SURVIVED, as predicted — and not by luck of the
grid. On a `demand` point `required_fuel = mf_sched - cap_fuel` and `g_fuel = mf_sched - w_fuel`
(the unfloored projections `fuel_transient.rs`'s `Demand` doc names), so
`required > g  ⇔  cap < w`, which is precisely the comparison `demand_tau`'s swap performs. The
swap IS the change of variables; the two spellings can differ only where `mf_sched - cap` and
`mf_sched - w` round to the same float while `cap ≠ w`, which no point here does. So the port
keeps Python's two spellings as a matter of FORM, the module header now says why no gate pins the
choice, and step 7's injection list should not spend a seat on it. (Rung 74's own § 0.4 gate —
attack vs release on a known point — is a different claim and stands: it tests the swap against
the UNSWAPPED call on the SAME arguments.)

#### (f) P1, READ EARLY AND NOT SCORED

Rung 81 has 193 Python body lines and landed at **815** Rust lines — ≈ 4.2 per body line, against
AI's 2 195 / 530 ≈ 4.1. Its share of P1's per-line figure (193 / 653 × 2 700) is ≈ 800; of the
per-field figure (× 5 700), ≈ 1 685. **Rung 81 tracks the per-LINE model**, although it is the
densest rung in returned structure: the struct declarations did not add a line per key on top.

#### (g) BOOKKEEPING

Citation guard re-blessed 54/348/208 → **55/361/218**: ten new anchors, each read by hand against
the printed line, none wrong at birth. Gate: full `cargo test --release` + full `pytest` — the
numbers are in the commit.

**Next: step 3 — rung 82's nine readers** in `threshold_law.rs`.
