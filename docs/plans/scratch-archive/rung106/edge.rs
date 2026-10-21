//! Rung 106, the user's "limit each law": the step check of the governed lagged march over the lag 0.1–1.0 (step 0.02)
//! at three flights, for one law at one candidate edge (scaled by the density as the page scales its own limits: a
//! rate's time constant divided by min(1, density), a gain divided by max(1, density)).
//! Usage: edge <hp_rate|lp_rate|hp_prop|lp_prop|thrust|epr> <value at the design flight>
use turbojet::burner_hold::Hold;
use turbojet::governor_lag::{self as gl, ClipLag, Demand};
use turbojet::sandbox_controls::{build, ControlsSettings, SpeedLaw, ThrustBy, DS};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let (law, v): (&str, f64) = (&a[1], a[2].parse().unwrap());
    let mut overall = 0.0f64;
    for (name, t0, p0, m0) in [("design", 250.0, 50_000.0, 0.85), ("thin", 220.0, 20_000.0, 0.85), ("dense", 288.15, 101_325.0, 0.3)] {
        let base = ControlsSettings { t0, p0, m0, govern: true, redline_on: true, gov_lag_on: true, ..ControlsSettings::defaults() };
        let rho = base.density_ratio();
        let (tau, gain) = (v / rho.min(1.0), v / rho.max(1.0));
        let s0 = match law {
            "hp_rate" => ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Rate, gov_tau: tau, ..base },
            "hp_prop" => ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: gain, ..base },
            "lp_rate" => ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Rate, gov_tau: tau, ..base },
            "lp_prop" => ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Prop, gov_gain: gain, ..base },
            "thrust" => ControlsSettings { hold: Hold::Thrust, ..base },
            "epr" => ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::Epr, ..base },
            _ => panic!("law"),
        };
        let (mut worst, mut at, mut stops) = (0.0f64, 0.0f64, 0usize);
        let mut over = Vec::new();
        for k in 0..=45 {
            let lag = 0.1 + 0.02 * k as f64;
            let s = ControlsSettings { gov_lag: lag, ..s0 };
            let sv = match build(&ControlsSettings { gov_lag_on: false, ..s }) { Ok(x) => x, Err(e) => { println!("{name}: refused {}", e.0); break } };
            let g = sv.governor.as_ref().unwrap();
            let nu0 = (sv.start.nu_lp, sv.start.nu_hp);
            let run = |ds: f64| gl::march(&sv.core.fuel, &sv.flight, Demand::Governor(g), ClipLag { tt4_max: s.redline, tau: lag },
                                          &sv.lag_limiters(), None, nu0, s.s_end(), ds).0;
            let (x, y) = (run(DS), run(DS / 4.0));
            if x.len() < s.expected_points() { stops += 1; }
            let n = x.len().min((y.len() + 3) / 4);
            let dm = (0..n).map(|k| (x[k].phi_lp - y[4 * k].phi_lp).abs().max((x[k].phi_hp - y[4 * k].phi_hp).abs())).fold(0.0, f64::max);
            if dm > worst { worst = dm; at = lag; }
            if dm >= 1e-4 { over.push(format!("{lag:.2}:{dm:.1e}")); }
        }
        overall = overall.max(worst);
        println!("{law} {v} {name} (density {rho:.2}): worst {worst:.2e} at lag {at:.2}; stops {stops}; over 1e-4: [{}]", over.join(" "));
    }
    println!("{law} {v}: OVERALL {overall:.2e} {}", if overall < 1e-4 { "CLEAN" } else { "ROUGH" });
}
