//! Reachability 2: (a) the flat-LP + floor stops — does rung 95's own governed march stop at the same point (no lag
//! involved)?; (b) the fast ramp (0.16 τ, redline 1440) the first probe's 0.15 could not run; (c) the lean-wall count
//! of the APPLIED fuel over a wider sweep (every law, both lags' extremes, several throttle pairs and redlines).
use rung106_probe::*;
use turbojet::burner_hold::Hold;
use turbojet::sandbox_controls::{build, ControlsSettings, SpeedLaw, ThrustBy, CONTROL_SHAPES, DS};

fn laws(d: ControlsSettings) -> Vec<(&'static str, ControlsSettings)> {
    vec![
        ("Tt4", ControlsSettings { hold: Hold::BurnerTemp, ..d }),
        ("thrust", ControlsSettings { hold: Hold::Thrust, ..d }),
        ("HP rate .1", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Rate, gov_tau: 0.1, ..d }),
        ("HP prop 20", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: 20.0, ..d }),
        ("EPR", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::Epr, ..d }),
        ("N1 rate", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Rate, gov_tau: 0.2, ..d }),
        ("N1 prop", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Prop, gov_gain: 5.0, ..d }),
    ]
}

fn main() {
    let d = ControlsSettings { govern: true, redline_on: true, ..ControlsSettings::defaults() };
    println!("(a) flat-LP + floor: rung 95's own march");
    for (n, s) in laws(ControlsSettings { shape: 3, floor_on: true, ..d }) {
        let sv = build(&s).unwrap();
        let (pts, _) = sv.march_marked();
        let (recs, stop) = march(&sv, s.redline, 0.2, DS);
        println!("  {n:10}: r95 {} pts, lagged {} pts; lagged stop: {}", pts.len(), recs.len(), stop.unwrap_or_default());
    }
    println!("(b) fast ramp 0.16, redline 1440");
    for shape in 0..4 {
        for (n, s) in laws(ControlsSettings { shape, ramp: 0.16, redline: 1440.0, ..d }) {
            let Ok(sv) = build(&s) else { println!("  refused"); continue };
            for tau in [0.02, 0.2, 0.8] {
                let (recs, stop) = march(&sv, s.redline, tau, DS);
                let la = recs.iter().filter(|r| r.lean_applied).count();
                println!("  {:10} {n:10} tau {tau}: {}/{} pts, applied-lean {la}{}", CONTROL_SHAPES[shape], recs.len(),
                         s.expected_points(), stop.map(|m| format!(" STOP {m}")).unwrap_or_default());
            }
        }
    }
    println!("(c) wider sweep: applied-lean counts");
    let (mut runs, mut lean_runs, mut stops) = (0, 0, 0);
    for (from, to) in [(700.0, 1400.0), (1000.0, 1600.0), (800.0, 1700.0), (1450.0, 1000.0), (1700.0, 900.0)] {
        for redline in [1300.0, 1480.0, 1600.0] {
            for ramp in [0.1, 0.5, 2.0] {
                for shape in 0..4 {
                    for (n, s) in laws(ControlsSettings { shape, from, to, redline, ramp, ..d }) {
                        let Ok(sv) = build(&s) else { continue };
                        for tau in [0.02, 0.8] {
                            let (recs, stop) = march(&sv, s.redline, tau, DS);
                            runs += 1;
                            let la = recs.iter().filter(|r| r.lean_applied).count();
                            if la > 0 {
                                lean_runs += 1;
                                println!("  LEAN {from}->{to} rl {redline} ramp {ramp} {} {n} tau {tau}: {la} pts", CONTROL_SHAPES[shape]);
                            }
                            if let Some(m) = stop {
                                stops += 1;
                                if stops <= 40 { println!("  stop {from}->{to} rl {redline} ramp {ramp} {} {n} tau {tau}: {m}", CONTROL_SHAPES[shape]); }
                            }
                        }
                    }
                }
            }
        }
    }
    println!("  {runs} runs, {lean_runs} with an applied fuel at the lean wall, {stops} stopped early");
}
