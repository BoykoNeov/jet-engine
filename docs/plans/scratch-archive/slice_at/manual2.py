from manual import d

# ---- rungs 12-23 weak-name pairs, each read side by side
d("rung13", "convexity_jump_lean_and_sign_reversal_at_stoich", "PORTED", "the_convexity_jump_reverses_sign_at_a_stoichiometric_mean", "")
d("rung13", "mixingpdf_positivity_guards", "PORTED", "mixing_pdf_positivity_guards", "")
d("rung14", "freeze_composition_in_eq_branch_is_frozen_bitforbit", "CORRECTED", "the_expansion_body_consults_its_composition_function",
  "rung14.rs header item 3: after the Rust factorisation the equality compares a function with itself; kept as the SETUP of an arm that must MOVE under a different pool")
d("rung15", "stoich_mean_sign_reversal_the_discriminator", "PORTED", "term2_reverses_sign_at_a_stoichiometric_mean", "")
d("rung15", "quenchpdf_positivity_guards", "PORTED", "quench_pdf_positivity_guards", "")
d("rung16", "zoned_nox_matches_ei16_helper", "DECLARED-NONPORT", "the_pocket_bank_is_independent_of_the_segregation",
  "rung16.rs header: in Rust the helper and production are the same call; the bank/integration split is gated instead")
d("rung16", "sublinear_dwell_the_mechanism", "PORTED", "the_per_pocket_dwell_is_sublinear", "")
d("rung16", "far_flank_erosion_vs_rung15", "PORTED", "the_far_flank_erodes_against_rung15", "same J{144,225,400,625}, same 0.93 bar")
d("rung16", "clamp_dormant_over_pockets", "PORTED", "the_clamp_stays_dormant_over_every_pocket", "")
d("rung16", "pocketquenchpdf_positivity_guards", "PORTED", "pocket_quench_pdf_positivity_guards", "")
d("rung17", "identity_is_witnessed_not_a_test", "DECLARED-NONPORT", "", "rung17.rs header: vacuity case #6")
d("rung17", "requires_both_configs", "DECLARED-NONPORT", "", "rung17.rs header: vacuity case #7")
d("rung18", "spatial_coverage_omega_is_needed_for_the_optimum", "PORTED", "only_the_spatial_coverage_produces_an_interior_optimum", "")
d("rung18", "derived_ceiling_from_phi_p", "PORTED", "the_ceiling_is_derived_from_the_primary_richness", "")
d("rung18", "rich_primary_required_for_ceiling", "PORTED", "a_primary_leaner_than_the_mean_has_no_ceiling", "")
d("rung18", "requires_mixing", "PORTED", "the_transported_closure_requires_a_mixing_config", "")
d("rung18", "cycle_untouched", "PORTED", "the_cycle_is_untouched_by_a_transported_call", "")
d("rung21", "shape_preserved_optimum_at_jopt", "PORTED", "the_lift_preserves_the_shape_and_the_optimum", "")
d("rung22", "cycle_untouched", "PORTED", "the_cycle_is_untouched_a_pure_diagnostic", "")
d("rung23", "cycle_untouched", "PORTED", "the_cycle_is_untouched_a_pure_diagnostic", "")
d("rung12", "clamp_dormancy_persists_over_j_sweep", "PORTED", "clamp_dormancy_persists_over_both_streams", "")
d("rung13", "optimum_is_at_holdeman_c_opt_and_shifts_as_H_over_S_squared", "PORTED",
  "the_optimum_sits_at_the_holdeman_group_and_shifts_as_h_over_s_squared", "four spacings where the Python samples two")
# ---- rungs 29-33
d("rung29", "the_inversion_ratio_is_not_energy", "PORTED", "the_super_equilibrium_ratio_is_anti_correlated_with_the_energy", "")
d("rung29", "direction_recombination_reheats", "PORTED", "the_shifting_exit_is_warmer_and_at_higher_pressure", "")
d("rung30", "reduce_subcritical_is_full_expansion", "PORTED", "subcritical_convergent_is_the_default_nozzle_bit_for_bit", "")
d("rung30", "solver_reproduces_cpg_closed_form", "PORTED", "the_cpg_throat_reproduces_the_textbook_critical_ratios", "")
d("rung30", "solver_m9_is_one_on_reacting_gas", "PORTED", "the_reacting_throat_exits_at_mach_one", "")
d("rung30", "verdict_design_point_chokes_and_costs_thrust", "PORTED", "it_chokes_at_design_and_the_pressure_term_rescues_most_of_the_deficit", "")
d("rung30", "direction_pressure_term_partially_cancels_momentum_loss", "MERGED", "it_chokes_at_design_and_the_pressure_term_rescues_most_of_the_deficit", "")
d("rung33", "cycle_untouched_and_map_out_of_scope", "SPLIT", "gate7_cycle_untouched;rung32::rung33_gate7_second_half_map_does_not_inherit_subsonic",
  "rung33.rs header: the map half is a claim about rung 32 and lives in rung32.rs")
d("rung41", "rung36_verdict_survives_but_its_mechanism_is_corrected", "PORTED",
  "rung36::rung41_deferred_the_verdict_survives_while_its_mechanism_is_corrected", "rung41.rs header row 12")
d("rung55", "capacity_style_guards_reject_nonsense", "SPLIT", "test_style_guard_rejects_zero_stages;test_style_guard_rejects_more_stator_rows_than_stages",
  "2 of 3 arms; the third (an unknown split name) is unrepresentable as an enum, rung55.rs slice_n_deferrals item 1")
d("rung55", "cycle_untouched_transient_ladder_is_bit_for_bit_unstacked", "PORTED", "rung45::rung55_item5_transient_ladder_is_bit_for_bit_unstacked",
  "rung55.rs slice_n_deferrals item 5: discharged at slice S step 3")
# ---- rungs 62/63: one Rust fn per pytest case
_R3 = ("r025", "r050", "r100")
for t, fns in [
    ("headline_both_loop_factors_have_their_sign_and_neither_reverses", ["headline_both_loop_factors_" + x for x in ("0900", "1100", "1300", "1500")]),
    ("headline_the_two_schedules_land_on_OPPOSITE_SIDES_of_one", ["headline_two_schedules_opposite_sides_" + x for x in _R3]),
    ("headline_the_loop_is_witnessed_in_the_COMMANDED_SETTING", ["headline_loop_witnessed_in_commanded_setting_" + x for x in ("r025", "r100")]),
    ("headline_survives_the_schedule_SHAPE", ["headline_survives_shape_linear", "headline_survives_shape_smooth"]),
    ("second_finding_a_bleed_SCHEDULE_triples_the_stators_surrender", ["second_finding_bleed_triples_surrender_" + x for x in _R3]),
    ("second_finding_is_the_LOOP_and_not_the_LEVEL", ["second_finding_loop_not_level_" + x for x in _R3]),
    ("corrects_rung61_credits_are_sub_additive_on_the_transient", [f"corrects_rung61_credits_sub_additive_{s}_{r}" for s in ("shaped", "tilted") for r in _R3]),
    ("corrects_rung61_adverse_SPEED_cost_interaction_survives", [f"corrects_rung61_adverse_speed_{s}_{r}" for s in ("shaped", "tilted") for r in _R3]),
]:
    d("rung62", t, "SPLIT", ";".join(fns), "one Rust fn per pytest case")
for t, fns in [
    ("choked_A4_control_holds_for_every_lever", ["choked_a4_control_holds_for_" + x for x in ("bleed_const", "bleed_sched", "stator_sched")]),
    ("the_bleed_retimes_the_leg_where_the_stator_does_not", [f"bleed_retimes_the_leg_{s}_{r}" for s in ("shaped", "tilted") for r in _R3]),
    ("every_march_stays_on_the_choked_branch_with_a_leg_armed", ["every_march_stays_choked_" + x for x in ("bare", "bleed_const", "bleed_sched")]),
]:
    d("rung63", t, "SPLIT", ";".join(fns), "one Rust fn per pytest case")
d("rung64", "every_march_stays_on_the_choked_branch", "LOOPED", "every_march_stays_on_the_choked_branch",
  "the same four lever arms in one loop (read by hand)")
for t in ("throttle_cannot_split_the_currencies", "split_is_shape_robust", "speed_lever_off_design_stays_within_the_registered_bands"):
    d("rung53", t, "LOOPED", "test_" + t, "loops SPOOLS (lp, hp) and, where parametrized, shapes() (read by hand)")
d("rung72", "charpoly_selftest", "PORTED", "charpoly_selftest_is_clean_on_both_matrices", "")
d("rung81", "reduce_is_bit_for_bit_rung80", "CORRECTED", "reduce_the_reader_reads_split_marchs_own_march",
  "rung81.rs header row 1: SUBSTITUTED (a self-comparison in Rust becomes an argument-threading check)")
d("rung82", "reduce_is_bit_for_bit_rung81", "CORRECTED", "reduce_the_scan_reads_split_marchs_own_march",
  "rung82.rs header row 1: SUBSTITUTED")
