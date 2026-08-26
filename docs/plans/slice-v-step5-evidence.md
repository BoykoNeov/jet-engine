# Slice V step 5 — the measured evidence

Emitted, not typed — by `rust/oracle/emit_slice_v_step5_evidence.py`, which RUNS the gate
and parses its own stdout. The narrative and every conclusion drawn from these tables live
in `docs/plans/todo-rust-port.md` § 5.20 step 5; this file is the data behind them.

**How to regenerate.**

```text
.venv\Scripts\python.exe rust\oracle\inject_slice_v.py --dispatch
.venv\Scripts\python.exe rust\oracle\inject_slice_v.py --self
.venv\Scripts\python.exe rust\oracle\emit_slice_v_step5_evidence.py
```

## 1. The gate

`cargo test --release --test slice_v_dispatch` — **ok, 6 passed, 0 failed, 0 ignored** (exit 0).

**P5, the headline.** `margin_min_lp` on the `both` arming, clean against the SAME
object with the arming scoped to the close:

```text
clean   4.62341253470772706e-2   (the committed golden `A/both/tsm/margin_min_lp`)
scoped  3.90998666812636397e-2
moved   15.430721 %          npts 61 -> 62
```

**The negative control.** `const_lp`: **2388** wrapped calls, **0 of 9** readings
moved. A constant setting is applied by the constructor and never passes through
`_arm`, so scoping `_arm` cannot reach it — § 5.20 (ii)'s exact-zero row, live.

## 2. § 5.20 (ii)'s six channels, both columns computed live

| arming | key | clean | scoped | moved | miss | bar | used |
|---|---|---|---|---|---|---|---|
| `lp_only` | `sm/SM_lp` | 6.08030847104e-2 | 5.79867858796e-2 | 4.6318 % | 3.91e-13 | 5e-12 | 8 % |
| `lp_only` | `tsm/margin_min_lp` | 1.14002036880e-1 | 1.13471510970e-1 | 0.4654 % | 3.00e-11 | 5e-10 | 6 % |
| `hp_only` | `tsm/margin_min_lp` | 9.23212214544e-2 | 8.51827788141e-2 | 7.7322 % | 4.12e-12 | 5e-12 | 82 % |
| `hp_only` | `sm/SM_hp` | 4.40493450112e-1 | 4.30113120026e-1 | 2.3565 % | 2.58e-11 | 5e-9 | 1 % |
| `both` | `sm/SM_lp` | 6.08737996237e-2 | 5.79867858796e-2 | 4.7426 % | 3.91e-13 | 5e-12 | 8 % |
| `both` | `tsm/margin_min_lp` | 4.62341253471e-2 | 3.90998666813e-2 | 15.4307 % | 1.26e-12 | 5e-12 | 25 % |

The clean column is asserted against `oracle/slice_v_pypy.tsv` **by key**; the scoped
column against § 5.20 (ii)'s printed value at half a unit in its own last printed place.

## 3. The six injections against the manufactured gate

`inject_slice_v.py --dispatch` — the step-3/4 harness with a third mode. Predictions were
written before the run; both columns are here so a wrong one cannot be quietly dropped.

**A `MISSED` row here is not a coverage failure.** I4 (`erosion`) and I5 (the incidence
lever) are not in this file's reader chain and it builds no incidence floor — the ORACLE
covers both (14 keys and a PANIC-BEFORE-COMPARE). A carrier gate that fired on everything
would not be measuring the carrier; `const_lp`'s exact zero is the same statement from the
other side.

**`the_hand_built_machine_is_the_shipped_one` is green under every row, and that is**
**correct.** It is an equality between two objects built the same way, so a `src`-side bug
moves both sides together. Its detector is a divergence between the CONSTRUCTOR and the
hand-build — which no injection in this set manufactures.

| injection | predicted | measured | predicted n | measured n | failing gates |
|---|---|---|---|---|---|
| `I1_local_armed_core` | CAUGHT | **CAUGHT** | 4 | 4 | `the_six_channels_of_the_plan_reproduce_live_on_both_sides`, `the_local_armed_core_breaks_rung_57s_own_currency`, `the_scoped_port_reads_a_scheduled_machine_as_an_unstatored_one`, `the_remarching_reader_is_immune_and_that_is_a_call_order_property` |
| `I2_hp_arm_dropped` | CAUGHT | **CAUGHT** ⚠ | 3 | 4 | `the_local_armed_core_breaks_rung_57s_own_currency`, `the_six_channels_of_the_plan_reproduce_live_on_both_sides`, `the_scoped_port_reads_a_scheduled_machine_as_an_unstatored_one`, `the_remarching_reader_is_immune_and_that_is_a_call_order_property` |
| `I3_smooth_shape_cubed` | CAUGHT | **CAUGHT** | 2 | 2 | `the_six_channels_of_the_plan_reproduce_live_on_both_sides`, `the_local_armed_core_breaks_rung_57s_own_currency` |
| `I4_erosion_inverted` | MISSED | **MISSED** | 0 | 0 | — |
| `I5_incidence_lever_sign` | MISSED | **MISSED** | 0 | 0 | — |
| `I6_arm_reads_the_wrong_shaft` | CAUGHT | **CAUGHT** | 2 | 2 | `the_six_channels_of_the_plan_reproduce_live_on_both_sides`, `the_local_armed_core_breaks_rung_57s_own_currency` |

## 3b. The gate's OWN manufactured bug, mutated

Every row above patches `rust/src/` — it asks *does the gate notice when the PORT is
wrong?* It says nothing about the mutation the gate itself carries. A wrapper that restored
only ONE of the two maps is a PARTIAL carrier bug in the INSTRUMENT, and the four
difference-asserting gates could still have passed at their pinned values. `--self` patches
the TEST FILE instead:

| mutation of the wrapper | status | gates failing |
|---|---|---|
| `S1_partial_carrier_both_wrappers` | **CAUGHT** | 4 of 6: `the_local_armed_core_breaks_rung_57s_own_currency`, `the_six_channels_of_the_plan_reproduce_live_on_both_sides`, `the_scoped_port_reads_a_scheduled_machine_as_an_unstatored_one`, `the_remarching_reader_is_immune_and_that_is_a_call_order_property` |
| `S2_partial_carrier_fuel_wrapper_only` | **CAUGHT** | 4 of 6: `the_local_armed_core_breaks_rung_57s_own_currency`, `the_six_channels_of_the_plan_reproduce_live_on_both_sides`, `the_scoped_port_reads_a_scheduled_machine_as_an_unstatored_one`, `the_remarching_reader_is_immune_and_that_is_a_call_order_property` |

Both fire **4 of 6** — the same four as I1, and the narrow variant (HP restore dropped in
the FUEL wrapper alone) is not confined to the HP armings, because the two spools are
coupled through the shaft state. That is step 3's I2 finding recurring on the instrument
rather than on the port.

## 4. Which gates need the golden, and which do not

A manufactured gate earns its place only where it says something a golden comparison
cannot. Recorded per test rather than claimed in prose:

| `#[test]` | reads `slice_v_pypy.tsv` |
|---|---|
| `a_constant_setting_is_the_negative_control_and_the_wrapper_did_run` | **no** |
| `the_hand_built_machine_is_the_shipped_one` | **no** |
| `the_local_armed_core_breaks_rung_57s_own_currency` | yes |
| `the_remarching_reader_is_immune_and_that_is_a_call_order_property` | **no** |
| `the_scoped_port_reads_a_scheduled_machine_as_an_unstatored_one` | **no** |
| `the_six_channels_of_the_plan_reproduce_live_on_both_sides` | yes |

The four that read nothing on disk are the ones that survive a regenerated golden.
