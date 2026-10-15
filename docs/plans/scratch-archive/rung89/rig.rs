// Rung 89 probe rig (scratch): rung 88's rig verbatim.
use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::FuelPoint;
use turbojet::gas::{Gas, GasSpec};
use turbojet::lagged_bleed::build_lagged_bleed;
use turbojet::lagged_incidence::trapz;
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient, StatorArm, StatorSchedule};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

pub const REAL: TwoSpoolLosses = TwoSpoolLosses { pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true };
pub fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }
pub fn cpg() -> Gas { Gas::new(GasSpec { gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
    gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0, hpr: 42.8e6, ..GasSpec::default() }) }
pub fn lp_map() -> ComponentMap { ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }.with_phi_surge(0.55) }
pub fn hp_map() -> ComponentMap { ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }.with_phi_surge(0.55) }
pub fn tilt_map() -> ComponentMap { ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..ComponentMap::flat() }.with_phi_surge(0.55) }
pub fn design() -> TwoSpoolEngine { build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL) }
pub fn bare(lp: ComponentMap, hp: ComponentMap) -> ScheduledStatorCore {
    match build_lagged_bleed(design(), flight(), 1.0, Some(lp), Some(hp), 1.0, &LeverArm::default()) {
        ScheduledStatorTransient::Full(c) => c, _ => unreachable!() } }
pub const BMAX: f64 = 0.10;
pub fn stat() -> LeverArm { LeverArm::stator(StatorArm::scheduled_lp(StatorSchedule::new(0.20, 0.65))) }
pub fn phi_of(sm: f64) -> f64 { 0.55 * (1.0 + sm) }
pub const TAUS: [f64; 5] = [0.01, 0.02, 0.05, 0.1, 0.2];
/// Rung 87/88's 19 scored cells: (shape, r, sm).
pub const CELLS: [(&str, f64, f64); 19] = [
    ("shaped",0.25,0.34),("shaped",0.25,0.38),("shaped",0.25,0.42),("shaped",0.5,0.42),("shaped",0.5,0.46),("shaped",0.5,0.50),
    ("shaped",1.0,0.42),("shaped",1.0,0.46),("shaped",1.0,0.50),("shaped",1.0,0.54),("tilted",0.25,0.34),("tilted",0.25,0.38),
    ("tilted",0.25,0.42),("tilted",0.5,0.42),("tilted",0.5,0.46),("tilted",0.5,0.50),("tilted",1.0,0.46),("tilted",1.0,0.50),("tilted",1.0,0.54)];
pub fn thrust(t: &[FuelPoint], n: usize) -> f64 { trapz(t, n, |p| p.sp_thrust * p.mdot_air) }
