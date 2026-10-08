from manual import d, M

d("numeric_fingerprint", "golden_file_declares_its_provenance", "PORTED", "fingerprint::cpython_golden_declares_its_provenance", "")
d("numeric_fingerprint", "every_kernel_has_a_disclosed_tolerance", "PORTED", "fingerprint::tolerance_tables_match_the_module",
  "the transcribed TOL/ABS_TOL tables equal the module's, kernel by kernel")
d("numeric_fingerprint", "every_kernel_is_actually_GATED", "PORTED", "fingerprint::coverage",
  "all 45 kernels ported in module order, each with its own gate! line")
for f, t, r in [("rung31", "cycle_untouched", "gate3_cycle_untouched"),
                ("rung32", "cycle_untouched", "gate2_cycle_untouched"),
                ("rung38", "cycle_untouched", "gate6_cycle_untouched"),
                ("rung39", "cycle_untouched_rung6", "gate10_cycle_untouched_rung6"),
                ("rung40", "cycle_untouched_rung6", "gate8_cycle_untouched_rung6"),
                ("rung61", "cycle_untouched_bit_for_bit_rung6", "gate10_cycle_untouched_bit_for_bit_rung6")]:
    d(f, t, "PORTED", r, "")
# The two gap ports this slice writes (rust fn names fixed when the files land).
d("claude_md_reference", "claude_md_within_byte_budget", "PORTED-IN-AT", "claude_md_reference::claude_md_within_byte_budget", "")
d("claude_md_reference", "claude_md_within_line_budget", "PORTED-IN-AT", "claude_md_reference::claude_md_within_line_budget", "")
for t in ("rung49_level_monotonicity_holds", "rate_inversion_cutting_fuel_steepens_the_descent",
          "the_two_halves_carry_opposite_signs", "the_arresting_bracket_has_no_root"):
    d("phi_rate_limiter_negative", t, "PORTED-IN-AT", "phi_rate_limiter_negative::" + t, "")
d("rung41", "cycle_untouched_rung6_bit_for_bit", "NARROWED", "gate1c_cycle_untouched_rung6",
  "rung41.rs header row 3: the bit-for-bit halves port, armed with the two-spool map diagnostics; the interleaved SpoolTransient.surge_margin_channels arm is deferred and no later gate interleaves it (Rust diagnostics take &self, so it cannot perturb the run)")
# ---- slice AT's own gap ports (overrides the GAP / NARROWED rows above)
_AT = "ported at slice AT from this ledger"
d("rung22", "reduce_primary_diagnostic_bit_identical", "PORTED-IN-AT", "a_spatial_call_leaves_the_primary_diagnostic_bit_identical", _AT)
d("rung22", "derived_floor_sits_below_the_hump_peak", "PORTED-IN-AT", "the_derived_floor_sits_just_below_the_hump_peak", _AT + "; Python grids ny=40, n_bell=48, scan n_quad=200")
d("rung22", "grid_converged", "PORTED-IN-AT", "the_located_optimum_is_grid_converged", _AT + "; ny=nz in {32,48,64}, 81-point log sweep")
d("rung23", "correlation_concentrated_under_penetration", "PORTED-IN-AT", "the_correlation_is_concentrated_under_penetration", _AT + "; Python grids")
d("rung23", "g_below_two_stream_ceiling", "PORTED-IN-AT", "the_ceiling_and_the_clamp_hold_at_the_pythons_own_jets",
  _AT + "; J{1,16,400} as the Python (the merged house gate visits J{4,16,64})")
d("rung23", "clamp_dormant_at_station4", "PORTED-IN-AT", "the_ceiling_and_the_clamp_hold_at_the_pythons_own_jets",
  _AT + "; J{4,16,100} as the Python")
d("rung24", "production_width_matches_spatial_pdf", "PORTED-IN-AT", "the_production_width_matches_rung_22s_through_zoned_nox", _AT)
d("rung24", "g_below_two_stream_ceiling", "PORTED-IN-AT", "the_width_stays_below_the_two_stream_ceiling", _AT + "; J{4,16,64}")
d("rung24", "local_rate_moves_ei_only_modestly_vs_rung23", "PORTED-IN-AT", "the_local_rate_moves_ei_only_modestly_against_rung_23", _AT + "; J{4,16}")
d("rung24", "tau_scales_linearly_in_tau_mix", "PORTED-IN-AT", "every_cells_dwell_scales_linearly_in_tau_mix",
  _AT + "; per cell at J{4,16}; the_dwell_scales_linearly_in_tau_mix keeps the production reading at J=16")
d("rung27", "guards", "SPLIT", "guard_length_must_be_positive;guard_nstep_below_100_is_refused;guard_rate_scale_must_be_positive;guard_requires_the_equilibrium_gas;guard_rejects_back_pressure_above_total",
  "all five Python arms; the L=0 arm added at slice AT")
d("rung28", "guards", "SPLIT", "guard_length_must_be_positive;guard_nstep_below_100_is_refused;guard_rate_scale_must_be_positive;guard_pool_rate_scale_must_be_positive;guard_requires_the_equilibrium_gas;guard_rejects_back_pressure_above_total",
  "all six Python arms; the L=0 and rate_scale=0 arms added at slice AT")
d("rung28", "depletion_unbounded_heat_release_saturates", "PORTED-IN-AT", "depletion_is_unbounded_while_the_heat_release_saturates", _AT + "; the six-decade sweep")
d("rung28", "net_is_deeper_frozen_across_the_band", "PORTED-IN-AT", "the_net_is_deeper_frozen_across_the_band", _AT + "; the Python's six-point BAND")
d("rung28", "both_channels_are_real_and_oppose", "MERGED", "the_two_channels_oppose_and_the_net_is_still_deeper_frozen",
  "Python Tt4{1800,2200,2400}; Rust {1800,2200,2300}; the net verdict at 2400 is the_net_is_deeper_frozen_across_the_band")
_GAP = "ported at slice AT: one of the plan's two named gap ports (§ 8.1 (v))"
d("claude_md_reference", "claude_md_within_byte_budget", "PORTED-IN-AT", "claude_md_reference::claude_md_within_byte_budget", _GAP)
d("claude_md_reference", "claude_md_within_line_budget", "PORTED-IN-AT", "claude_md_reference::claude_md_within_line_budget", _GAP)
for t in ("rung49_level_monotonicity_holds", "rate_inversion_cutting_fuel_steepens_the_descent",
          "the_two_halves_carry_opposite_signs", "the_arresting_bracket_has_no_root"):
    d("phi_rate_limiter_negative", t, "PORTED-IN-AT", "phi_rate_limiter_negative::" + t,
      _GAP + "; the walk's coverage is pinned to the Python's measured 14/16/14/16 by the declared addition the_walk_is_cut_where_the_python_cut_it")
d("rung28", "no_arrives_sub_equilibrium_yet_the_surrogate_still_bounds", "MERGED",
  "no_arrives_sub_equilibrium_and_leaves_super_equilibrium;the_surrogate_bounds_the_rate_along_the_path",
  "the arrival half and the bound half are two Rust gates")
d("rung30", "direction_pressure_term_partially_cancels_momentum_loss", "MERGED",
  "it_chokes_at_design_and_the_pressure_term_rescues_most_of_the_deficit",
  "the verdict and the direction are one Rust gate: momentum falls, the pressure term recovers > half")
