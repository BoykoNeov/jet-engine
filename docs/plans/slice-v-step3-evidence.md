# Slice V step 3 — the measured evidence

Emitted, not typed. The narrative and every conclusion drawn from these tables live in
`docs/plans/todo-rust-port.md` § 5.20 step 3; this file is the data behind them.

**How to regenerate.** Copy `rust/oracle/slice_v_probe.rs.keep` to
`rust/tests/slice_v_probe.rs` and run
`cargo test --release --test slice_v_probe -- --nocapture`. It is kept OUT of `tests/`
as a `.keep` because it has no assertions, and a no-assertion `#[test]` is a vacuous
gate that would also break the crate's `0 ignored` line by adding a 60th test that can
never fail. `rust/oracle/inject_slice_v.py` drives it: for each named injection it
patches `rust/src/stator_transient.rs`, re-runs the probe for the DID-IT-MOVE column,
then runs the four ported suites — and restores the source in a `finally`. Pass
injection names as arguments to run a subset.

## The injection table

The probe emits **342 keys: 302 GATE-VISIBLE** (quantities some ported gate compares)
**and 40 WITNESS** (`W/…` — the live map's `vsv` after a march, `arm`'s dispatch
counters on an HP-scheduled machine no suite builds, the steady `surge_margin`).
**They are counted separately on purpose.** A witness key moving proves the injection
is LIVE; it says nothing about whether any gate can see it. Without that column the
first pass reported `moved 0` for I1 and I2, which is indistinguishable from an
injection that never applied — see § 5.20 step 3 finding 2.

| injection | gate-visible moved | witness moved | worst rel | caught | failing gates |
|---|---|---|---|---|---|
| `I1_local_armed_core` | **0** / 302 | 9 / 40 | 1 | **0** / 59 | **none** |
| `I2_hp_arm_dropped` | **0** / 302 | 15 / 40 | 1 | **0** / 59 | **none** |
| `I3_smooth_shape_cubed` | **78** / 302 | 21 / 40 | 4.1 | **4** / 59 | test_p3_p4_credit_decomposition<br>test_p5_the_phi_leg_is_not_composable_at_a_fixed_set_point<br>test_p3_the_knee_sweep_is_monotone_in_the_commanded_setting<br>test_p4_the_decomposition_is_clocked_but_the_delivered_credit_is_not |
| `I4_erosion_inverted` | **5** / 302 | 0 / 40 | 3.89 | **5** / 59 | test_p2_the_margin_itself_swings_far_more_than_the_credit_tilted<br>test_p1_erosion_is_r_invariant_and_matches_rung53_closed_form_primary<br>test_p1_erosion_is_r_invariant_and_matches_rung53_closed_form_tilted<br>test_p2_the_margin_itself_swings_far_more_than_the_credit_primary<br>test_reduce_rung57_readers_untouched |
| `I5_incidence_lever_sign` | **5** / 302 | 0 / 40 | 1.8e+14 | **5** / 59 | test_cycle_untouched_by_rung60_bit_for_bit_rung6<br>test_floor_composite_refuses_a_feedforward_leg<br>test_p1_the_third_regime_carries_no_armed_cell_dynamics_either<br>test_p5_the_timing_half_survives_because_a_time_has_no_wall<br>test_p1_a_floor_pins_its_own_coordinate_so_the_composite_is_a_tautology |
| `I6_arm_reads_the_wrong_shaft` | **79** / 302 | 12 / 40 | 12.1 | **4** / 59 | test_schedule_is_not_a_margin_lever_beside_a_constant<br>test_p3_p4_credit_decomposition<br>test_p3_the_knee_sweep_is_monotone_in_the_commanded_setting<br>test_p4_the_decomposition_is_clocked_but_the_delivered_credit_is_not |

`I1` is the LOCAL-ARMED-CORE carrier bug — the shape a natural Rust port takes, and the
one § 5.20 (ii) measured in Python. `I2` deletes `arm`'s whole HP branch. Both are
invisible to all 302 gate-visible readings and to all 59 gates.

## The bar margins — 63 inequalities, got vs bar

`slack` is `(bar-got)/|bar|` for a `<` bar and `(got-bar)/|bar|` for a `>` one: how far
the reading sits on the passing side, as a fraction of the bar. Tightest first. A bar
passing at 5x or more is a SIGN or ORDER-OF-MAGNITUDE test, not a magnitude one — which
is what the Python docstrings say those gates are for; the slack is not a defect.

| gate | quantity | got | op | bar | slack |
|---|---|---|---|---|---|
| r58 p3 | `sched v_ratio` | 1.11648 | `>` | 1.1 | +0.015 |
| r60 p3 | `gap_phi_bands` | 1.05298 | `>` | 1 | +0.053 |
| r57 cur | `m_phi shut<bare` | 0.175386 | `<` | 0.185448 | +0.0543 |
| r58 p4 | `delivered spread < bare` | 0.0652001 | `<` | 0.0704443 | +0.0744 |
| r57 cur | `nu0 shut>bare` | 0.828164 | `>` | 0.755741 | +0.0958 |
| r57 p4 | `max self_cancel` | 0.896315 | `<` | 1 | +0.104 |
| r58 p6 | `const leg-cost drift` | 0.000826051 | `<` | 0.000955108 | +0.135 |
| r57 cur | `m_i shut>bare` | 0.527606 | `>` | 0.458467 | +0.151 |
| r58 p3 | `predicted/interaction sched` | 0.824127 | `>` | 0.7 | +0.177 |
| r59 p1 | `lp_const d_abscissa` | 7.75544e-13 | `<` | 1e-12 | +0.224 |
| r57 p3 | `max share_start` | 0.270505 | `<` | 0.35 | +0.227 |
| r58 p4 | `credit_bare spread` | 0.0704443 | `<` | 0.1 | +0.296 |
| r58 p6 | `sched leg-cost drift` | 0.000261781 | `<` | 0.000382043 | +0.315 |
| r57 p5b | `eta-island gap` | 0.000319635 | `<` | 0.000480421 | +0.335 |
| r60 p4 | `credit spread` | 0.00929048 | `<` | 0.015 | +0.381 |
| r58 p3 | `predicted/interaction const` | 1.00553 | `>` | 0.7 | +0.436 |
| r60 p3 | `gap_m_bands` | 0.0436297 | `<` | 0.1 | +0.564 |
| r58 p3 | `const share (floor)` | 0.00862091 | `<` | 0.02 | +0.569 |
| r59 p1 | `hp_const chain worst` | 4.00324e-13 | `<` | 1e-12 | +0.6 |
| r60 p4 | `excursion spread` | 3.20847 | `>` | 2 | +0.604 |
| r57 p1 | `|erosion-cf|/cf @r2` | 0.0391893 | `<` | 0.1 | +0.608 |
| r57 p1 | `|erosion-cf|/cf @r1` | 0.0389107 | `<` | 0.1 | +0.611 |
| r57 p1 | `|erosion-cf|/cf @r3` | 0.0380132 | `<` | 0.1 | +0.62 |
| r57 p1 | `|erosion-cf|/cf @r4` | 0.0367973 | `<` | 0.1 | +0.632 |
| r58 p2 | `|rel_dormant|` | 0.00162673 | `<` | 0.005 | +0.675 |
| r58 p2 | `|rel_limited|` | 0.00162552 | `<` | 0.005 | +0.675 |
| r59 p1 | `hp_const d_ordinate` | 2.98101e-13 | `<` | 1e-12 | +0.702 |
| r59 p1 | `lp_sched chain worst` | 2.73506e-13 | `<` | 1e-12 | +0.726 |
| r59 p1 | `lp_const chain worst` | 2.68148e-13 | `<` | 1e-12 | +0.732 |
| r57 p2 | `bare swing` | 0.520332 | `>` | 0.3 | +0.734 |
| r57 p1 | `|erosion-cf|/cf @r0` | 0.0225738 | `<` | 0.1 | +0.774 |
| r57 p1 | `erosion spread` | 0.0104616 | `<` | 0.05 | +0.791 |
| r59 p1 | `lp_const d_ordinate` | 1.94696e-13 | `<` | 1e-12 | +0.805 |
| r59 p1 | `lp_sched d_abscissa` | 1.56777e-13 | `<` | 1e-12 | +0.843 |
| r59 p1 | `lp_sched d_ordinate` | 1.23334e-13 | `<` | 1e-12 | +0.877 |
| r60 p5 | `|d_s_eng|/s_eng 0` | 0.950667 | `>` | 0.5 | +0.901 |
| r57 lever | `|g-c| vs .25|g|` | 0.00122802 | `<` | 0.0131804 | +0.907 |
| r58 p2 | `share` | 0.0990056 | `>` | 0.05 | +0.98 |
| r59 p1b | `lp_const |delta_match|` | 3.21965e-15 | `<` | 1e-12 | +0.997 |
| r60 p1 | `|credit_fuel| inc 1` | 8.88178e-16 | `<` | 1e-12 | +0.999 |
| r60 p1 | `|credit_fuel| inc 0` | 5.55112e-16 | `<` | 1e-12 | +0.999 |
| r59 p1b | `lp_sched |delta_match|` | 4.44089e-16 | `<` | 1e-12 | +1 |
| r60 p1 | `|credit_fuel| inc 2` | 2.22045e-16 | `<` | 1e-12 | +1 |
| r59 p2 | `|abscissa_share-1|` | 1.82077e-14 | `<` | 1e-06 | +1 |
| r59 p1 | `hp d_abscissa` | 0.066898 | `>` | 0.03 | +1.23 |
| r58 p5 | `bare_min - sched_first` | 0.0226253 | `>` | 0.01 | +1.26 |
| r58 p3 | `sched vs 5x const` | 0.0990056 | `>` | 0.0431045 | +1.3 |
| r60 p3 | `ratio` | 24.1345 | `>` | 10 | +1.41 |
| r58 p2 | `share vs 20x dormant` | 0.0990056 | `>` | 0.0325345 | +2.04 |
| r60 p4 | `ratio` | 345.351 | `>` | 100 | +2.45 |
| r58 p4 | `max share vs 10x min` | 0.143632 | `>` | 0.0297688 | +3.82 |
| r57 p2 | `swing/10*spread` | 0.520332 | `>` | 0.104616 | +3.97 |
| r60 p5 | `|d_s_eng|/s_eng 1` | 3.15883 | `>` | 0.5 | +5.32 |
| r57 p5 | `|d_phi_hp| flat` | 0.00928879 | `>` | 0.001 | +8.29 |
| r57 p5 | `|d_phi_hp| shaped` | 0.00960843 | `>` | 0.001 | +8.61 |
| r57 p5 | `|d_Tt25| shaped` | 13.0497 | `>` | 1 | +12 |
| r57 p5 | `|d_Tt25| flat` | 13.3659 | `>` | 1 | +12.4 |
| r59 p2 | `delta_match (hp)` | 0.0153088 | `>` | 0.001 | +14.3 |
| r60 p5 | `|d_s_eng|/s_eng 2` | 12.0011 | `>` | 0.5 | +23 |
| r59 p3 | `|int_bare| vs 5x|int_matched|` | 0.0152588 | `>` | 0.000249858 | +60.1 |
| r57 p5 | `|hp d_phi_lp| flat` | 0.117907 | `>` | 0.001 | +117 |
| r57 p5 | `|hp d_phi_lp| shaped` | 0.127796 | `>` | 0.001 | +127 |
| r57 p4 | `min self_cancel` | 0.754068 | `>` | 0 | +inf |

## The two non-strict orderings, measured

`test_p3_p4_credit_decomposition` ports Python's `xs == sorted(xs, reverse=True)` as
`>=`, faithfully — an inert sequence satisfies it. Both are in fact strict:

| sequence | `r` = 0.1 / 0.25 / 0.5 / 1.0 / 2.0 | smallest adjacent gap |
|---|---|---|
| `share_start` | +0.270505 +0.119577 +0.029678 −0.029814 −0.068085 | 3.83e-2 (11.3 % of range) |
| `self_cancel` | +0.896315 +0.802907 +0.768883 +0.756340 +0.754068 | 2.27e-3 (1.6 % of range) |
