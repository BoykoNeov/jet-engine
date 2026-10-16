//! The step check (plan § 12.10 / rung 95 § 4) at the FAST end of the flight box: the march at 0.02 against 0.005 for
//! every knob limit set by a step check at the design flight. Fast = dense AND hot (dnu/ds ∝ δ·√θ).
use turbojet::burner_hold::Hold;
use turbojet::fuel_transient::FuelPoint;
use turbojet::governor as gv;
use turbojet::sandbox_controls::*;
use turbojet::atmosphere::standard;

fn march(sv: &ControlsSolver, ds: f64) -> Vec<FuelPoint> {
    let nu0 = (sv.start.nu_lp, sv.start.nu_hp);
    if let Some(g) = &sv.governor {
        return gv::march(&sv.core.fuel, &sv.flight, g, &sv.limiters(), None, nu0, sv.settings.s_end(), ds).0;
    }
    let sched = sv.settings.schedule(sv.mf_lo, sv.mf_hi);
    sv.core.fuel.integrate_fuel(&sv.flight, sched, nu0, sv.settings.s_end(), ds, &sv.limiters())
}

fn main() {
    let d = ControlsSettings::defaults();
    let fast = d.ramp; let _ = fast;
    let cases: Vec<(&str, ControlsSettings)> = vec![
        ("bare default", d),
        ("bare ramp 0.02", ControlsSettings { ramp: 0.02, ..d }),
        ("redline", ControlsSettings { redline_on: true, ..d }),
        ("gov lag 0.02", ControlsSettings { redline_on: true, gov_lag_on: true, gov_lag: 0.02, ..d }),
        ("release 0.02/0.02 floor", ControlsSettings { floor_on: true, release_on: true, tau_att: 0.02, tau_rel: 0.02, ..d }),
        ("accel + release", ControlsSettings { accel_on: true, release_on: true, tau_att: 0.02, tau_rel: 0.1, ..d }),
        ("slow valve 0.02", ControlsSettings { lever: LeverChoice::Bleed, bleed_mode: BleedMode::Valve, valve_lag_on: true, valve_tau: 0.02, ..d }),
        ("slow valve 0.02 close own 0.02", ControlsSettings { lever: LeverChoice::Bleed, bleed_mode: BleedMode::Valve, valve_lag_on: true, valve_tau: 0.05, valve_close_own: true, valve_tau_close: 0.02, ..d }),
        ("gov speed rate 0.04", ControlsSettings { govern: true, hold: Hold::HpSpeed, speed_law: SpeedLaw::Rate, gov_tau: 0.04, ..d }),
        ("gov speed prop 50", ControlsSettings { govern: true, hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: 50.0, ..d }),
        ("gov thrust", ControlsSettings { govern: true, hold: Hold::Thrust, ..d }),
        ("gov n1 rate 0.04", ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Rate, gov_tau: 0.04, ..d }),
        ("gov n1 prop 50", ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Prop, gov_gain: 50.0, ..d }),
        ("gov epr", ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::Epr, ..d }),
    ];
    let (t_sl, p_sl) = standard(0.0);
    let flights = [("design", d.t0, d.p0), ("SL std", t_sl, p_sl), ("SL +40", t_sl + 40.0, p_sl), ("SL -40", t_sl - 40.0, p_sl),
                   ("15 km", standard(15000.0).0, standard(15000.0).1)];
    for (name, c) in &cases {
        for shape in [0usize, 3] {
            for (fname, t0, p0) in flights {
                let s = ControlsSettings { t0, p0, shape, ..*c };
                let sv = match build(&s) { Ok(sv) => sv, Err(e) => { println!("{name:32} {:10} {fname:7}: REFUSED {}", CONTROL_SHAPES[shape], &e.0[..e.0.len().min(90)]); continue; } };
                let a = march(&sv, 0.02);
                let b = march(&sv, 0.005);
                let n = a.len().min((b.len() + 3) / 4);
                let (mut dphi, mut dt) = (0.0f64, 0.0f64);
                for k in 0..n {
                    let (x, y) = (&a[k], &b[4 * k]);
                    dphi = dphi.max((x.phi_lp - y.phi_lp).abs()).max((x.phi_hp - y.phi_hp).abs());
                    dt = dt.max((x.tt4 - y.tt4).abs());
                }
                let pk = |v: &[FuelPoint]| v.iter().map(|x| x.tt4).fold(0.0, f64::max);
                println!("{name:32} {:10} {fname:7}: pts {}/{} fine {}  max dphi {:.1e}  max dTt4 {:.2} K  peak {:.1} vs {:.1}",
                         CONTROL_SHAPES[shape], a.len(), s.expected_points(), b.len(), dphi, dt, pk(&a), pk(&b));
            }
        }
    }
}
