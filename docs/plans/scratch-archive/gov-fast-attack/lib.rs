//! Probe: the governed realistic release on a FINER step, through the shipped march (`governor_release::march`).
use turbojet::fuel_transient::FuelPoint;
use turbojet::governor_lag as gl;
use turbojet::governor_release as gr;
use turbojet::sandbox_controls::ControlsSolver;
use turbojet::two_spool_transient::Instant2;

/// The shipped march at step `ds` (records every step).
pub fn march(sv: &ControlsSolver, ds: f64) -> Vec<FuelPoint> {
    let g = sv.governor.as_ref().unwrap();
    let over = |i: &Instant2| sv.metal_over(i);
    let metal: Option<&dyn Fn(&Instant2) -> f64> = if sv.settings.metal_leg() { Some(&over) } else { None };
    gr::march(&sv.core.fuel, &sv.flight, gl::Demand::Governor(g), &sv.limiters(), metal,
              (sv.start.nu_lp, sv.start.nu_hp), sv.settings.s_end(), ds).0
}

/// Largest flow-coefficient gap between run `x` (step DS/kx) and `y` (step DS/ky), on the DS grid.
pub fn gap(x: &[FuelPoint], kx: usize, y: &[FuelPoint], ky: usize) -> f64 {
    if x.is_empty() || y.is_empty() {
        return f64::NAN;
    }
    let n = ((x.len() - 1) / kx).min((y.len() - 1) / ky) + 1;
    (0..n).map(|i| (x[kx * i].phi_lp - y[ky * i].phi_lp).abs().max((x[kx * i].phi_hp - y[ky * i].phi_hp).abs()))
        .fold(0.0, f64::max)
}
