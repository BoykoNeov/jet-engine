//! The advisor's check on G4: the SAME sweep as reach2 (c) — every configuration — with the OPEN ramp as the demand
//! (the prototype ≡ rung 47, check47). Counts the open runs whose applied fuel reaches the lean wall, and lists them.
use rung106_probe::*;
use turbojet::burner_hold::Hold;
use turbojet::sandbox_controls::{build, ControlsSettings, SpeedLaw, ThrustBy, CONTROL_SHAPES, DS};

fn main() {
    OPEN.store(true, std::sync::atomic::Ordering::Relaxed);
    let d = ControlsSettings { govern: true, redline_on: true, ..ControlsSettings::defaults() };
    let laws = |d: ControlsSettings| vec![
        ("Tt4", ControlsSettings { hold: Hold::BurnerTemp, ..d }),
        ("thrust", ControlsSettings { hold: Hold::Thrust, ..d }),
        ("HP rate .1", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Rate, gov_tau: 0.1, ..d }),
        ("HP prop 20", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: 20.0, ..d }),
        ("EPR", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::Epr, ..d }),
        ("N1 rate", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Rate, gov_tau: 0.2, ..d }),
        ("N1 prop", ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Prop, gov_gain: 5.0, ..d }),
    ];
    let (mut runs, mut lean_runs, mut stops) = (0, 0, 0);
    for (from, to) in [(700.0, 1400.0), (1000.0, 1600.0), (800.0, 1700.0), (1450.0, 1000.0), (1700.0, 900.0)] {
        for redline in [1300.0, 1480.0, 1600.0] {
            for ramp in [0.1, 0.5, 2.0] {
                for shape in 0..4 {
                    // the open ramp ignores the law, but the hold moves the ENDS (rung 94's targets) — keep every law's ends
                    for (n, s) in laws(ControlsSettings { shape, from, to, redline, ramp, ..d }) {
                        let Ok(sv) = build(&s) else { continue };
                        for tau in [0.02, 0.8] {
                            let (recs, stop) = march(&sv, s.redline, tau, DS);
                            runs += 1;
                            let la = recs.iter().filter(|r| r.lean_applied).count();
                            if la > 0 {
                                lean_runs += 1;
                                println!("  OPEN LEAN {from}->{to} rl {redline} ramp {ramp} {} {n} tau {tau}: {la} pts", CONTROL_SHAPES[shape]);
                            }
                            if let Some(m) = stop {
                                stops += 1;
                                println!("  stop {from}->{to} rl {redline} ramp {ramp} {} {n} tau {tau}: {m}", CONTROL_SHAPES[shape]);
                            }
                        }
                    }
                }
            }
        }
    }
    println!("  {runs} open runs, {lean_runs} with an applied fuel at the lean wall, {stops} stopped early");
}
