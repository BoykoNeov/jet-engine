//! Plan § 47 probe: rung 106's SHIPPED governed lag (`governor_lag::march`) driven from a Controls request at any step.
use turbojet::burner_hold::Hold;
use turbojet::fuel_transient::FuelPoint;
use turbojet::governor_lag as gl;
use turbojet::sandbox_controls::{ControlsSettings, ControlsSolver, SpeedLaw, ThrustBy};
use turbojet::two_spool_transient::Instant2;

/// The view's own call (`march_marked`'s rung-106 arm) with the step as an argument.
pub fn lag_march(sv: &ControlsSolver, ds: f64) -> Vec<FuelPoint> {
    let over = |i: &Instant2| sv.metal_over(i);
    let metal: Option<&dyn Fn(&Instant2) -> f64> = if sv.settings.metal_leg() { Some(&over) } else { None };
    gl::march(&sv.core.fuel, &sv.flight, gl::Demand::Governor(sv.governor.as_ref().unwrap()), sv.clip_lag().unwrap(),
              &sv.lag_limiters(), metal, (sv.start.nu_lp, sv.start.nu_hp), sv.settings.s_end(), ds).0
}

/// Largest gap in either flow coefficient on the coarse run's points that both runs reach (`kx`, `ky` points per 0.02).
pub fn gap(x: &[FuelPoint], kx: usize, y: &[FuelPoint], ky: usize) -> f64 {
    if x.is_empty() || y.is_empty() { return f64::NAN; }
    let n = ((x.len() - 1) / kx).min((y.len() - 1) / ky) + 1;
    (0..n).map(|i| (x[kx * i].phi_lp - y[ky * i].phi_lp).abs().max((x[kx * i].phi_hp - y[ky * i].phi_hp).abs())).fold(0.0, f64::max)
}

/// The 63 governed settings of rung 107/108's probes: 3 cases × 3 flights × 7 holds/laws, each law at rung 106's edge.
pub fn settings(lag: f64) -> Vec<(String, ControlsSettings)> {
    let mut v = Vec::new();
    for case in ["red", "sched", "floor"] {
        for (fname, t0, p0, m0) in [("design", 250.0, 50_000.0, 0.85), ("thin", 220.0, 20_000.0, 0.85), ("dense", 288.15, 101_325.0, 0.3)] {
            let base0 = ControlsSettings { t0, p0, m0, redline_on: true, gov_lag_on: true, gov_lag: lag, govern: true, ..ControlsSettings::defaults() };
            let gb = match case { "red" => base0, "sched" => ControlsSettings { accel_on: true, ..base0 }, _ => ControlsSettings { floor_on: true, shape: 1, ..base0 } };
            let rho = gb.density_ratio();
            for (l, s) in [
                ("thrust", ControlsSettings { hold: Hold::Thrust, ..gb }),
                ("EPR", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::Epr, ..gb }),
                ("Tt4", ControlsSettings { hold: Hold::BurnerTemp, ..gb }),
                ("HPrate.1", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Rate, gov_tau: 0.1 / rho.min(1.0), ..gb }),
                ("HPprop10", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: 10.0 / rho.max(1.0), ..gb }),
                ("N1rate.15", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Rate, gov_tau: 0.15 / rho.min(1.0), ..gb }),
                ("N1prop7", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Prop, gov_gain: 7.0 / rho.max(1.0), ..gb }),
            ] { v.push((format!("{case} {fname} {l} lag{lag}"), s)); }
        }
    }
    v
}
