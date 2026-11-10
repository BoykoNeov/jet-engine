//! Baselines for the step check: the same step comparison (DS against DS/4, largest flow-coefficient gap) for
//! (a) rung 95's governed march with the INSTANT schedule / floor (the page's route today), and (b) the prototype at a
//! slow attack (0.2/0.4) — per case, flight and law. Usage: base <sched|floor|both>
use rung107_probe::*;
use turbojet::burner_hold::Hold;
use turbojet::fuel_transient::AsymmetricLag;
use turbojet::governor as gv;
use turbojet::sandbox_controls::{build, ControlsSettings, SpeedLaw, ThrustBy, DS};

fn main() {
    let case: String = std::env::args().nth(1).unwrap_or("sched".into());
    for (fname, t0, p0, m0) in [("design", 250.0, 50_000.0, 0.85), ("thin", 220.0, 20_000.0, 0.85), ("dense", 288.15, 101_325.0, 0.3)] {
        let base0 = ControlsSettings { t0, p0, m0, govern: true, ..ControlsSettings::defaults() };
        let base = match case.as_str() {
            "sched" => ControlsSettings { accel_on: true, ..base0 },
            "floor" => ControlsSettings { floor_on: true, shape: 1, ..base0 },
            "both" => ControlsSettings { accel_on: true, floor_on: true, redline_on: true, ..base0 },
            _ => panic!("case"),
        };
        let rho = base.density_ratio();
        let laws: Vec<(&str, ControlsSettings)> = vec![
            ("thrust", ControlsSettings { hold: Hold::Thrust, ..base }),
            ("EPR", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::Epr, ..base }),
            ("HP rate .1", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Rate, gov_tau: 0.1 / rho.min(1.0), ..base }),
            ("HP prop 10", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: 10.0 / rho.max(1.0), ..base }),
            ("N1 rate .15", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Rate, gov_tau: 0.15 / rho.min(1.0), ..base }),
            ("N1 prop 7", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Prop, gov_gain: 7.0 / rho.max(1.0), ..base }),
        ];
        for (lname, s) in &laws {
            let sv = match build(s) {
                Ok(v) => v,
                Err(e) => {
                    println!("{case} {fname} {lname}: REFUSED {}", e.0);
                    continue;
                }
            };
            let g = sv.governor.as_ref().unwrap();
            let nu0 = (sv.start.nu_lp, sv.start.nu_hp);
            let r95 = |ds: f64| gv::march(&sv.core.fuel, &sv.flight, g, &sv.limiters(), None, nu0, s.s_end(), ds).0;
            let (x, y) = (r95(DS), r95(DS / 4.0));
            let n = x.len().min((y.len() + 3) / 4);
            let d95 = (0..n).map(|k| (x[k].phi_lp - y[4 * k].phi_lp).abs().max((x[k].phi_hp - y[4 * k].phi_hp).abs())).fold(0.0, f64::max);
            let mut out = Vec::new();
            for (ta, tr) in [(0.1, 0.4), (0.2, 0.4), (0.4, 0.4)] {
                let lag = AsymmetricLag::new(ta, tr);
                let (x, _) = march(&sv, lag, false, DS);
                let (y, _) = march(&sv, lag, false, DS / 4.0);
                let n = x.len().min((y.len() + 3) / 4);
                let dm = (0..n).map(|k| (x[k].phi_lp - y[4 * k].phi_lp).abs().max((x[k].phi_hp - y[4 * k].phi_hp).abs())).fold(0.0, f64::max);
                out.push(format!("{ta}/{tr}:{dm:.1e}"));
            }
            println!("{case} {fname} (density {rho:.2}) {lname:12}: rung95 instant {d95:.2e} ({} pts); lagged {}", x.len(), out.join(" "));
        }
    }
    // (c) the governor off: rung 52's own open route, the same comparison
    for (fname, t0, p0, m0) in [("design", 250.0, 50_000.0, 0.85), ("thin", 220.0, 20_000.0, 0.85), ("dense", 288.15, 101_325.0, 0.3)] {
        let base0 = ControlsSettings { t0, p0, m0, ..ControlsSettings::defaults() };
        let s = match case.as_str() {
            "sched" => ControlsSettings { accel_on: true, ..base0 },
            "floor" => ControlsSettings { floor_on: true, shape: 1, ..base0 },
            _ => ControlsSettings { accel_on: true, floor_on: true, redline_on: true, ..base0 },
        };
        let sv = build(&s).unwrap();
        let mut out = Vec::new();
        for (ta, tr) in [(0.02, 0.1), (0.02, 0.8), (0.1, 0.4)] {
            let lag = AsymmetricLag::new(ta, tr);
            let (x, _) = march(&sv, lag, true, DS);
            let (y, _) = march(&sv, lag, true, DS / 4.0);
            let n = x.len().min((y.len() + 3) / 4);
            let dm = (0..n).map(|k| (x[k].phi_lp - y[4 * k].phi_lp).abs().max((x[k].phi_hp - y[4 * k].phi_hp).abs())).fold(0.0, f64::max);
            out.push(format!("{ta}/{tr}:{dm:.1e}"));
        }
        println!("{case} {fname} OPEN (rung 52, governor off): {}", out.join(" "));
    }
}
