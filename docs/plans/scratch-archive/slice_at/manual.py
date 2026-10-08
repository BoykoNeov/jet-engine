# Hand-read ledger decisions for rows the matcher could not settle (or settled weakly).
# key: (py_file, py_test) -> (status, "rust_fn[;rust_fn]" (file-qualified if not own file), note)
# Statuses: PORTED LOOPED SPLIT MERGED NARROWED CORRECTED DECLARED-NONPORT RE-ANCHORED RETIRED
#           PORTED-IN-AT GAP
M = {}


def d(f, t, status, rust="", note=""):
    M[("test_" + f, "test_" + t)] = (status, rust, note)


# ---- rung 22
d("rung22", "reduce_primary_diagnostic_bit_identical", "GAP", "",
  "Rust reduce compares spatial=None vs mixing-only; never runs a SPATIAL call and checks the primary ei_no/x_no_mix")
d("rung22", "no_C_opt_knob_it_is_derived", "DECLARED-NONPORT", "",
  "rung22.rs header: an unknown struct field is a compile error in Rust; the derivation is gated instead (c_opt_is_the_derived_closed_form_and_the_argmin_tracks_it)")
d("rung22", "derived_floor_sits_below_the_hump_peak", "GAP", "",
  "the claim is quoted in the_emissions_minimum_at_c_opt_is_only_local's comment but never asserted (no hump-peak scan)")
d("rung22", "grid_converged", "GAP", "", "no ny=nz in {32,48,64} convergence check anywhere in rung22.rs")
d("rung22", "positivity_guards", "PORTED", "the_config_rejects_non_positive_geometry", "")
d("rung22", "rich_primary_required", "PORTED", "a_lean_primary_has_no_rql_geometry", "")
d("rung22", "group_collapse_gmin_geometry_independent", "PORTED", "the_minimum_value_is_geometry_independent", "")
d("rung22", "k_p_sets_C_opt_and_the_collapse_is_robust", "PORTED", "a_larger_k_p_moves_c_opt_down_and_the_argmin_follows", "")
d("rung22", "C_opt_is_an_output_matching_the_closed_form", "PORTED", "c_opt_is_the_derived_closed_form_and_the_argmin_tracks_it", "")
# ---- rung 24
d("rung24", "g_identical_to_rung22_by_construction", "CORRECTED",
  "the_width_equals_rung_22s_to_ROUNDING_and_not_to_the_bit",
  "Python's 'exact by construction' measured FALSE (rounding only); Rust gates both halves. Grid J{4,16,100}xny{16,20,32} vs Python J{4,16,64}")
d("rung24", "production_width_matches_spatial_pdf", "GAP", "",
  "no zoned_nox-level g_spatial_local vs g_spatial comparison; the field-level twin is the_width_equals_rung_22s_to_ROUNDING_and_not_to_the_bit")
d("rung24", "g_below_two_stream_ceiling", "GAP", "", "no g_ceiling bound anywhere in rung24.rs (3 Python cases J=4,16,64)")
d("rung24", "f_shape_is_independent_of_tau_mix", "CORRECTED", "f_is_independent_of_tau_mix_to_rounding_but_not_to_the_bit",
  "same J{4,16,64}; Python's bit-identity measured FALSE, Rust gates rounding + not-bit")
d("rung24", "tau_scales_linearly_in_tau_mix", "NARROWED", "the_dwell_scales_linearly_in_tau_mix",
  "Python J{4,16}; Rust J=16 only, through the public zoned_nox wiring (C_e halved)")
d("rung24", "f_shape_is_u_shaped_with_minimum_at_c_opt", "PORTED", "f_is_u_shaped_with_its_minimum_at_c_opt", "")
d("rung24", "ei_stays_monotone_the_emissions_optimum_is_not_recovered", "MERGED", "the_split_F_turns_but_the_emissions_do_not",
  "monotone <EI> asserted window by window on J_COARSE")
d("rung24", "does_not_claim_the_emissions_global_min_location", "MERGED", "the_split_F_turns_but_the_emissions_do_not",
  "F argmin at 16 and EI argmin != F argmin, off one sweep")
d("rung24", "scale_swamps_shape_quantified", "PORTED", "the_scale_swamps_the_shape_quantified", "")
d("rung24", "local_rate_moves_ei_only_modestly_vs_rung23", "GAP", "", "no rung-24 vs rung-23 <EI> bound (|e24/e23-1|<0.10) in rung24.rs")
d("rung24", "reduce_spatial_local_none_is_prior_path", "PORTED", "reduce_none_leaves_the_prior_path_untouched", "")
# ---- rung 23
d("rung23", "production_g_matches_spatialpdf", "PORTED", "production_reports_the_same_width_as_rung_22_at_a_matched_grid",
  "tightened to bit equality")
d("rung23", "terminal_field_reproduces_rung22", "PORTED", "the_terminal_field_reproduces_rung_22_BIT_for_bit",
  "tightened to bit equality; J{4,16,100} x ny{16,20,32}")
d("rung23", "correlation_adds_no_at_design_point", "PORTED", "the_correlation_adds_no_and_the_instrument_is_alive",
  "the Python's excess-field identity line is not transcribed; a dead-instrument bar is added")
d("rung23", "correlation_concentrated_under_penetration", "GAP", "",
  "no r(J=4) > r(J=16) > 1 comparison; Rust reads corr_ratio at J=16 only")
d("rung23", "g_below_two_stream_ceiling", "NARROWED", "the_width_stays_below_the_two_stream_ceiling_and_the_clamp_stays_dormant",
  "Python J{1,16,400}; Rust J{4,16,64} (the two extremes not visited)")
d("rung23", "clamp_dormant_at_station4", "MERGED", "the_width_stays_below_the_two_stream_ceiling_and_the_clamp_stays_dormant",
  "Python J{4,16,100}; Rust J{4,16,64}, and also at five C_e in the_correlation_sign_is_one_signed_across_tau_mix")
d("rung23", "helper_matches_production", "DECLARED-NONPORT", "",
  "rung23.rs header: a function compared with itself in Rust; replaced by a_constant_spectrum_reproduces_the_scalar_path_bit_for_bit and the_matched_mean_arm_is_rung_16s_closure_at_a_derived_scalar")
d("rung23", "no_c_opt_knob", "PORTED", "c_opt_is_derived_here_too_no_knob", "the TypeError half is a compile error in Rust (rung22.rs header)")
d("rung23", "config_positivity", "PORTED", "the_config_and_the_field_reject_bad_input", "")
d("rung23", "reduce_spatial_dwell_none_is_prior_path", "PORTED", "reduce_none_leaves_the_prior_path_untouched", "")
d("rung23", "requires_mixing", "PORTED", "spatial_dwell_requires_a_mixing_config", "")
# ---- rung 20
d("rung20", "super_eq_o_now_combines_with_ideal_bell_closures", "PORTED", "rung21::the_hybrid_is_resolved_and_every_closure_combines",
  "rung20.rs header: gate 6's ideal-bell half discharged by slice C; rung21's test runs pdf, pdf_quench and transported with super_eq_o and must not panic")
d("rung20", "super_eq_o_combines_with_quench_closures", "PORTED", "super_eq_o_combines_with_unmixedness", "")
# ---- rungs 25-28
d("rung25", "guards", "SPLIT", "guard_da_zero_is_refused;guard_da_negative_is_refused;guard_nstep_below_100_is_refused;guard_requires_the_equilibrium_gas;guard_rejects_back_pressure_above_total",
  "all five Python arms, one Rust test each")
d("rung26", "rung25_neighbor_untouched", "DECLARED-NONPORT", "the_two_diagnostics_share_their_references_bit_for_bit",
  "rung26.rs header item 2: both methods take &self, so the Python's mutation check cannot fail in Rust; the shared-reference check replaces it")
d("rung26", "guards", "SPLIT", "guard_length_must_be_positive;guard_nstep_below_100_is_refused;guard_rate_scale_must_be_positive;guard_requires_the_equilibrium_gas;guard_rejects_back_pressure_above_total",
  "all five Python arms")
d("rung26", "entropy_production_nonneg", "PORTED", "entropy_production_is_monotone_and_positive_where_the_clock_runs", "")
d("rung27", "neighbors_untouched", "DECLARED-NONPORT", "the_diagnostics_share_their_references_bit_for_bit",
  "rung27.rs header item 2 (rung26.rs's &self reason)")
d("rung27", "guards", "NARROWED", "guard_nstep_below_100_is_refused;guard_rate_scale_must_be_positive;guard_requires_the_equilibrium_gas;guard_rejects_back_pressure_above_total",
  "4 of the Python's 5 arms; the L=0 config guard has no Rust test")
d("rung27", "clamp_scales_linearly_with_no_level", "PORTED", "the_clamp_scales_with_the_no_level_exactly_frozen_and_affine_when_relaxing", "")
d("rung28", "rung26_27_untouched_across_a_rung28_call", "DECLARED-NONPORT", "",
  "the &self reason of rung26.rs/rung27.rs header item 2 (rung28.rs does not restate it)")
d("rung28", "trajectory_recorder_is_a_pure_observer", "PORTED", "rung26::the_record_observer_is_bit_for_bit_pure",
  "the recorder belongs to rung 26's marcher, and its purity gate lives there")
d("rung28", "uncoupled_is_rung27_bit_for_bit", "PORTED", "uncoupled_is_rung27_through_the_public_method", "")
d("rung28", "entry_da_is_path_independent_and_frozen_everywhere", "PORTED", "the_entry_clock_is_path_independent", "")
d("rung28", "both_channels_are_real_and_oppose", "MERGED", "the_two_channels_oppose_and_the_net_is_still_deeper_frozen",
  "Python Tt4{1800,2200,2400}; Rust {1800,2200,2300}")
d("rung28", "net_is_deeper_frozen_across_the_band", "NARROWED", "the_two_channels_oppose_and_the_net_is_still_deeper_frozen",
  "Python BAND {1500,1650,1800,2000,2200,2400}; Rust {1800,2200,2300}")
d("rung28", "opposing_channel_grows_monotonically_with_tt4", "PORTED", "the_channel_ratio_rises_with_tt4", "")
d("rung28", "depletion_unbounded_heat_release_saturates", "GAP", "depletion_wins_decisively_at_a_faster_pool;depletion_wins_at_every_tt4_in_the_limit",
  "Rust compares base vs pool_rate_scale=1e6 only; the 6-scale monotone sweep and the heat-release SATURATION (bounded <1.5, last decade <1e-3) are not asserted")
d("rung28", "no_arrives_sub_equilibrium_yet_the_surrogate_still_bounds", "MERGED", "no_arrives_sub_equilibrium_and_leaves_super_equilibrium;the_surrogate_bounds_the_rate_along_the_path", "")
d("rung28", "guards", "NARROWED", "guard_nstep_below_100_is_refused;guard_pool_rate_scale_must_be_positive;guard_requires_the_equilibrium_gas;guard_rejects_back_pressure_above_total",
  "the L=0 and rate_scale=0 config arms have no Rust test")
