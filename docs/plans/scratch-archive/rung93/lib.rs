//! Shared rig-A setup for the rung-93 probes (rungs 42/90/91/92's rig A).
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

pub fn real() -> TwoSpoolLosses {
    TwoSpoolLosses { pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96,
                     eta_hpt: 0.92, eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None,
                     nozzle_convergent: true }
}
pub fn cpg() -> Gas {
    let (cp_c, cp_t) = (1004.0, 1239.0);
    Gas::new(GasSpec { gamma_c: 1.4, cp_c, r_c: 0.4 / 1.4 * cp_c, gamma_t: 1.3, cp_t, r_t: 0.3 / 1.3 * cp_t,
                       hpr: 42.8e6, ..GasSpec::default() })
}
pub fn design_at(tt4: f64) -> TwoSpoolEngine { build_two_spool_turbojet(cpg(), 3.0, 6.0, tt4, 50_000.0, real()) }
pub fn design() -> TwoSpoolEngine { design_at(1500.0) }
pub fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }
pub fn maps() -> Vec<(&'static str, ComponentMap, ComponentMap)> {
    vec![
        ("flat", ComponentMap::flat(), ComponentMap::flat()),
        ("flow/press", ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() },
                       ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }),
    ]
}

/// Gauntner (1980) NASA TM-81453, the bare correlation: coolant/gas = FACTOR*0.022*(phi/(1-phi))^1.25.
pub fn gauntner_w(factor: f64, phi: f64) -> f64 { factor * 0.022 * (phi / (1.0 - phi)).powf(1.25) }
/// Degrees Rankine to kelvin.
pub fn r_to_k(r: f64) -> f64 { r * 5.0 / 9.0 }
