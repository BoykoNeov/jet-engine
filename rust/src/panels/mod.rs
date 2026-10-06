//! **`main.py`, ported byte for byte** — the design-point run and one teaching panel per rung
//! (phase 8; `docs/plans/todo-rust-port.md` § 8.1).
//!
//! Every panel is a function writing into a [`Printer`], in the order `main.py`'s `main()` calls
//! it, and [`PANELS`] is that order. The gate (`tests/cli_golden.rs`) holds each panel to EXACT
//! equality with its own segment of the PyPy stdout golden (`rust/oracle/main_stdout.txt`, cut at
//! the byte offsets in `rust/oracle/main_segments.tsv`), so a panel can be neither short, long,
//! nor out of place. The binary (`src/main.rs`) runs [`PANELS`] in order and writes each panel's
//! text to stdout as it finishes.
//!
//! **Porting rules for a panel.** The Python is presentation code, but it is not arithmetic-free:
//! keep each expression's operation ORDER exactly (`100 * a / tot` is `(100*a)/tot`), spell a
//! non-square power with [`crate::gas::powp`] and a square as a multiply, sum in the source's
//! iteration order, and format through [`crate::pyf!`] / [`crate::pct!`] with the spec copied
//! verbatim. The byte gate catches a miss, but a slice late.

use crate::engine::{build_turbojet, EngineResult, FlightCondition, Losses};
use crate::gas::Gas;
use crate::pyfmt::Printer;

pub mod actuator;
pub mod airflow;
pub mod blade_speed;
pub mod cascades;
pub mod cycle;
pub mod limiters;
pub mod marches;
pub mod mixing;
pub mod nox;
pub mod offdesign;
pub mod readers;
pub mod schedules;
pub mod twospool;

/// The chart DATA the CLI writes to the working directory, as `main.py` wrote its PNG there.
/// `plot_ts_diagram.py` draws it (slice AR); the last line names this file, not a PNG.
pub const TS_DIAGRAM_JSON: &str = "ts_diagram.json";
/// `PI_C` — the design compressor pressure ratio (the rung-1 validation case).
pub const PI_C: f64 = 10.0;
/// `TT4` — the design turbine-inlet temperature, K.
pub const TT4: f64 = 1500.0;

/// `FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)`.
pub fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

/// `REAL_LOSSES` — one gas, fully expanded, so the only difference from the ideal run is the
/// entropy each component generates.
pub fn real_losses() -> Losses {
    Losses { pi_d: 0.97, eta_c: 0.88, eta_b: 0.99, pi_b: 0.96, eta_t: 0.90, eta_m: 0.99, pi_n: 0.98,
             ..Losses::default() }
}

/// What `main()` computes before its first print and hands to the panels that need it.
pub struct Design {
    pub flight: FlightCondition,
    /// `gas = Gas()` — the single cold-air-standard gas of the design-point comparison.
    pub gas: Gas,
    pub ideal: EngineResult,
    pub real: EngineResult,
}

impl Design {
    pub fn new() -> Self {
        let flight = flight();
        let gas = Gas::default();
        let ideal = build_turbojet(gas.clone(), PI_C, TT4, flight.p0, Losses::default()).run(&flight, 1.0);
        let real = build_turbojet(gas.clone(), PI_C, TT4, flight.p0, real_losses()).run(&flight, 1.0);
        Design { flight, gas, ideal, real }
    }
}

impl Default for Design {
    fn default() -> Self { Design::new() }
}

/// One `main()` step: a `print_*` call, or a line `main()` prints itself.
pub type Panel = fn(&mut Printer, &Design);

/// **The panels BORN IN RUST** — rung 85 on. They have no segment in the Python golden, so each
/// is held instead to its own capture in `rust/oracle/rust_owned/<name>.txt`, written ONLY by
/// `cargo run --release -- panel <name> --write`.
///
/// **What that capture certifies, and what it cannot.** It was taken from the very code it
/// checks, so a pass means "the text has not moved since the capture" — a CHANGE detector, never
/// a correctness oracle. A panel's numbers are right only by its rung's own tests
/// (`tests/rungN.rs`), held to values that did NOT come from this code.
///
/// Declared here by NAME, not inferred from "absent from the Python segments": inferring would let
/// a mistyped Python panel name pass as a new panel against a fresh capture of itself. Each name
/// must sit in [`PANELS`] exactly once, in the contiguous block just before the chart line, and in
/// no Python segment (`tests/cli_golden.rs`).
pub const RUST_OWNED: &[&str] = &["print_blade_speed_table"];

/// `main()`'s calls, in order, each named as the golden's segment file names it (a `print_*`
/// function's name, or `main:<what>` for text `main()` prints inline).
pub const PANELS: &[(&str, Panel)] = &[
    ("print_station_table", cycle::station_table_ideal),
    ("print_station_table", cycle::station_table_real),
    ("main:losses_cost", cycle::losses_cost),
    ("print_polytropic_table", cycle::polytropic_table),
    ("print_variable_cp_table", cycle::variable_cp_table),
    ("print_reacting_table", cycle::reacting_table),
    ("print_forkb_table", cycle::forkb_table),
    ("print_equilibrium_table", cycle::equilibrium_table),
    // ---- slice AL: rungs 7–24, the NOx / mixing strand
    ("print_nox_table", nox::nox_table),
    ("print_zoning_table", nox::zoning_table),
    ("print_rql_table", nox::rql_table),
    ("print_finite_quench_table", nox::finite_quench_table),
    ("print_jet_mixing_table", nox::jet_mixing_table),
    ("print_unmixedness_table", nox::unmixedness_table),
    ("print_mixing_pdf_table", nox::mixing_pdf_table),
    ("print_nozzle_flow_table", nox::nozzle_flow_table),
    ("print_pdf_quench_table", mixing::pdf_quench_table),
    ("print_pocket_quench_table", mixing::pocket_quench_table),
    ("print_exhaust_clamp_table", mixing::exhaust_clamp_table),
    ("print_transported_variance_table", mixing::transported_variance_table),
    ("print_super_eq_prompt_table", mixing::super_eq_prompt_table),
    ("print_super_eq_quench_table", mixing::super_eq_quench_table),
    ("print_ideal_bell_lift_table", mixing::ideal_bell_lift_table),
    ("print_spatial_pdf_table", mixing::spatial_pdf_table),
    ("print_dwell_spectrum_table", mixing::dwell_spectrum_table),
    ("print_local_mixing_table", mixing::local_mixing_table),
    // ---- slice AM: rungs 25–37, the marches and the single-spool off-design ladder
    ("print_finite_rate_nozzle_table", marches::finite_rate_nozzle_table),
    ("print_freeze_out_nozzle_table", marches::freeze_out_nozzle_table),
    ("print_no_freeze_out_table", marches::no_freeze_out_table),
    ("print_coupled_no_march_table", marches::coupled_no_march_table),
    ("print_shifting_turbine_table", marches::shifting_turbine_table),
    ("print_choked_nozzle_table", marches::choked_nozzle_table),
    ("print_offdesign_table", offdesign::offdesign_table),
    ("print_component_map_table", offdesign::component_map_table),
    ("print_subsonic_matching_table", offdesign::subsonic_matching_table),
    ("print_spool_transient_table", offdesign::spool_transient_table),
    ("print_fuel_metering_table", offdesign::fuel_metering_table),
    ("print_surge_line_table", offdesign::surge_line_table),
    ("print_combustor_dynamics_table", offdesign::combustor_dynamics_table),
    // ---- slice AN: rungs 38–45, the two-spool family
    ("print_two_spool_matching_table", twospool::two_spool_matching_table),
    ("print_two_spool_map_table", twospool::two_spool_map_table),
    ("print_two_shaft_transient_table", twospool::two_shaft_transient_table),
    ("print_two_spool_surge_table", twospool::two_spool_surge_table),
    ("print_interstage_bleed_table", twospool::interstage_bleed_table),
    ("print_two_shaft_fuel_table", twospool::two_shaft_fuel_table),
    ("print_transient_surge_table", twospool::transient_surge_table),
    ("print_transient_fuel_surge_table", twospool::transient_fuel_surge_table),
    // ---- slice AO: rungs 46–52, the fuel-side limiter family
    ("print_topping_governor_table", limiters::topping_governor_table),
    ("print_lagged_governor_table", limiters::lagged_governor_table),
    ("print_accel_schedule_table", limiters::accel_schedule_table),
    ("print_phi_limiter_table", limiters::phi_limiter_table),
    ("print_release_edge_table", limiters::release_edge_table),
    ("print_release_rate_table", limiters::release_rate_table),
    ("print_asymmetric_lag_table", limiters::asymmetric_lag_table),
    // ---- slice AP: rungs 53–63, the airflow levers and the schedules
    ("print_variable_stator_table", airflow::variable_stator_table),
    ("print_throat_capacity_table", airflow::throat_capacity_table),
    ("print_stage_stack_table", airflow::stage_stack_table),
    ("print_per_row_capacity_table", airflow::per_row_capacity_table),
    ("print_stator_schedule_table", schedules::stator_schedule_table),
    ("print_composite_minselect_table", schedules::composite_minselect_table),
    ("print_matched_floor_table", schedules::matched_floor_table),
    ("print_stator_bleed_table", schedules::stator_bleed_table),
    ("print_bleed_schedule_table", schedules::bleed_schedule_table),
    ("print_fuel_bleed_table", schedules::fuel_bleed_table),
    // ---- slice AQ: rungs 64–84, the valve, the cascades, the splits and the reader-only rungs
    ("print_bleed_limiter_table", cascades::bleed_limiter_table),
    ("print_lagged_valve_table", cascades::lagged_valve_table),
    ("print_two_lag_cascade_table", cascades::two_lag_cascade_table),
    ("print_cascade_a_table", cascades::cascade_a_table),
    ("print_three_loop_table", cascades::three_loop_table),
    ("print_reference_split_table", cascades::reference_split_table),
    ("print_cross_split_table", cascades::cross_split_table),
    ("print_full_split_table", cascades::full_split_table),
    ("print_shared_actuator_table", actuator::shared_actuator_table),
    ("print_applied_reference_table", actuator::applied_reference_table),
    ("print_demand_coordinate_table", actuator::demand_coordinate_table),
    ("print_anti_windup_table", actuator::anti_windup_table),
    ("print_sensed_cap_table", actuator::sensed_cap_table),
    ("print_stiffness_ledger_table", actuator::stiffness_ledger_table),
    ("print_residual_gauge_table", readers::residual_gauge_table),
    ("print_state_coordinate_table", readers::state_coordinate_table),
    ("print_split_wall_table", readers::split_wall_table),
    ("print_authority_clock_table", readers::authority_clock_table),
    ("print_threshold_law_table", readers::threshold_law_table),
    ("print_corrector_law_table", readers::corrector_law_table),
    ("print_staircase_law_table", readers::staircase_law_table),
    // ---- born in Rust: rung 85 on, each held to its own capture (RUST_OWNED)
    ("print_blade_speed_table", blade_speed::blade_speed_table),
    // ---- slice AR: the chart's line (the data itself is written by `src/main.rs`)
    ("plot_ts_diagram", cycle::ts_diagram_line),
];
