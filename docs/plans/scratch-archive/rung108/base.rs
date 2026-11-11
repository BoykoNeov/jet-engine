//! Baseline roughness at one step on the sweep's governed settings: rung 95 (instant redline) and rung 106's single
//! lag (0.1, 0.2), each k=1 against k=4 — to tell the two-speed clip's roughness from the route's own.
use rung108_probe::*;
use std::sync::Mutex;
use turbojet::burner_hold::Hold;
use turbojet::fuel_transient::FuelPoint;
use turbojet::governor as gv;
use turbojet::sandbox_controls::{build, ControlsSettings, ControlsSolver, SpeedLaw, ThrustBy, DS};
use turbojet::two_spool_transient::Instant2;

fn inst(sv: &ControlsSolver, ds: f64) -> Vec<FuelPoint> {
    let over = |i: &Instant2| sv.metal_over(i);
    let metal: Option<&dyn Fn(&Instant2) -> f64> = if sv.settings.metal_leg() { Some(&over) } else { None };
    let lim = turbojet::fuel_transient::FuelLimiters { tau_gov: None, ..sv.limiters() };
    gv::march(&sv.core.fuel, &sv.flight, sv.governor.as_ref().unwrap(), &lim, metal, (sv.start.nu_lp, sv.start.nu_hp), sv.settings.s_end(), ds).0
}
fn gapf(x: &[FuelPoint], kx: usize, y: &[FuelPoint], ky: usize) -> f64 {
    let n = ((x.len() - 1) / kx).min((y.len() - 1) / ky) + 1;
    (0..n).map(|i| (x[kx * i].phi_lp - y[ky * i].phi_lp).abs().max((x[kx * i].phi_hp - y[ky * i].phi_hp).abs())).fold(0.0, f64::max)
}
fn main() {
    let mut v = Vec::new();
    for case in ["red", "sched", "floor"] {
        for (fname, t0, p0, m0) in [("design", 250.0, 50_000.0, 0.85), ("thin", 220.0, 20_000.0, 0.85), ("dense", 288.15, 101_325.0, 0.3)] {
            let base0 = ControlsSettings { t0, p0, m0, redline_on: true, gov_lag_on: true, gov_lag: 0.2, govern: true, ..ControlsSettings::defaults() };
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
            ] { v.push((format!("{case} {fname} {l}"), s)); }
        }
    }
    let jobs = Mutex::new(v);
    std::thread::scope(|sc| {
        for _ in 0..8 {
            sc.spawn(|| loop {
                let Some((name, s)) = jobs.lock().unwrap().pop() else { break };
                let Ok(sv) = build(&s) else { println!("{name}: REFUSED"); continue };
                let i = gapf(&inst(&sv, DS), 1, &inst(&sv, DS / 4.0), 4);
                let l1 = gap(&march(&sv, 0.1, 0.1, DS), 1, &march(&sv, 0.1, 0.1, DS / 4.0), 4);
                let l2 = gap(&march(&sv, 0.2, 0.2, DS), 1, &march(&sv, 0.2, 0.2, DS / 4.0), 4);
                println!("{name} instant={i:.1e} lag0.1={l1:.1e} lag0.2={l2:.1e}");
            });
        }
    });
}
