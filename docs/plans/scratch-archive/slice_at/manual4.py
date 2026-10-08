"""The 108 word-overlap pairs, re-checked by NUMBERS (numcheck.py): 68 carried every distinctive
Python number in the paired Rust body; the 40 that did not were read side by side. Most were
noise (rung numbers in messages, a constant named in Rust where Python wrote the literal). These
are the ones that were not."""
from manual import d

d("rung9", "soot_bound_guard", "SPLIT", "soot_bound_accepts_exactly_two;soot_bound_rejects_22;soot_bound_rejects_30",
  "the accepted bound and both refused arms, one Rust test each (the pair tier had found only the 3.0 arm)")
d("rung33", "envelope_monotone_and_subidle", "SPLIT", "gate5_envelope_monotone;gate5_subidle_is_reported_not_force_fit",
  "the monotone envelope and the SUB-IDLE refusal are two Rust gates")
d("rung27", "margin_narrows_with_tt4_no_crossing", "NARROWED", "the_margin_narrows_with_tt4_without_crossing",
  "Python Tt4{1500,1650,1800,2000,2200}; Rust {1500,1800,2200}, same four assertions (the end-to-end collapse bar reads the same endpoints)")
d("rung29", "reduce_to_prior_frozen_is_the_shipped_turbine", "NARROWED", "the_frozen_bound_is_the_shipped_turbine_bit_for_bit",
  "Python Tt4{1500,1800,2100,2400}; Rust {1500,2200}, and against an actual Turbine rather than the retyped two lines")
d("rung29", "work_limited_solver_agrees_with_the_closed_form", "PORTED", "the_solver_frozen_branch_agrees_with_the_closed_form",
  "Tt4{1500,2200} for the Python's {1500,2100}; the T5 bar tightened 1e-6 -> 1e-8")
d("rung29", "verdict_earned_at_design_bites_hot", "NARROWED", "the_freeze_is_earned_at_design_and_bites_hot",
  "Rust walks a five-point ladder 1300-2300 against the code's SHIFT_EARNED_TOL; the Python's absolute bars at 2400 K "
  "(design < 2e-4, hot > 1e-2, hot/design > 100) are not asserted")
d("rung29", "M0_helper_reproduces_the_certified_flight_anchor", "CORRECTED", "the_margin_helper_reproduces_the_certified_flight_anchor",
  "rung29.rs: one parameterised helper in Rust, so the Python's helper-vs-helper equality is a self-comparison; re-aimed at helper vs the main construction, anchor kept")
d("rung30", "cycle_untouched_default_nozzle", "NARROWED", "the_default_nozzle_path_is_untouched",
  "Rust asserts the default nozzle bit-equal to the design run; the Python's absolute rung-6 anchors are not re-asserted here "
  "(F/mdot 798.37 is, in rung31-34/38; the station values are the fingerprint's absolute gate)")
d("rung23", "correlation_sign_one_signed_across_tau_mix", "NARROWED", "the_correlation_sign_is_one_signed_across_tau_mix",
  "Python J{4,16} x tau_mix scale{0.2,1,5}; Rust J=16 x five C_e (tau_mix x0.2-x5). J=4 at scale 1 is gated by the_correlation_is_concentrated_under_penetration")
d("rung22", "j_opt_shifts_as_H_over_S_squared", "PORTED", "j_opt_shifts_exactly_as_h_over_s_squared",
  "on rung22.rs's coarse location grid: an EXACT grid ratio replaces the Python's 81-point sweep at 15 % (rung22.rs J_COARSE note)")
d("rung28", "both_channels_are_real_and_oppose", "MERGED", "the_two_channels_oppose_and_the_net_is_still_deeper_frozen",
  "Python Tt4{1800,2200,2400}; the Rust loop {1800,2200,2300,2400} (2400 added at slice AT) carries the same four assertions plus the net verdict")
d("rung29", "earned_at_design_is_M0_robust", "PORTED", "earned_at_design_is_m0_robust",
  "the 2e-4 design bar and the 1800/2200 K bracket at every M0 were missing from the Rust body; added at slice AT")
