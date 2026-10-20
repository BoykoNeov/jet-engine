// Rung 104 probe P14 (advisor 2026-10-20): are the control rungs' failures numerical dust? March Controls runs on the
// perfect gas under the pin and under the ellipse; per step: the branch (choked?) and the largest relative difference in
// nu_L, nu_H, Tt4 and the applied fuel.
use turbojet::matcher::Branch;
use turbojet::sandbox_controls::{controls, BleedMode, ControlsSettings, LeverChoice};
use turbojet::turbine_law::TurbineLaw;

fn main() {
    let d = ControlsSettings::defaults();
    let cases: Vec<(&str, ControlsSettings)> = vec![
        ("default", d),
        ("stator", ControlsSettings { lever: LeverChoice::Stator, ..d }),
        ("bleed valve", ControlsSettings { lever: LeverChoice::Bleed, bleed_mode: BleedMode::Valve, ..d }),
        ("both + valve + floor", ControlsSettings { lever: LeverChoice::Both, bleed_mode: BleedMode::Valve,
                                                    floor_on: true, floor_phi: 0.759, ..d }),
        ("redline + lag", ControlsSettings { redline_on: true, gov_lag_on: true, ..d }),
    ];
    for (name, s) in cases {
        let (_, a) = controls(&ControlsSettings { turbine: TurbineLaw::Pin, ..s }).unwrap();
        let (_, b) = controls(&ControlsSettings { turbine: TurbineLaw::Ellipse, ..s }).unwrap();
        let unchoked = b.points.iter().filter(|p| p.branch != Branch::Choked).count();
        let mut worst = [0.0f64; 4];
        let mut at = [0usize; 4];
        for (i, (p, q)) in a.points.iter().zip(&b.points).enumerate() {
            for (k, (x, y)) in [(p.nu_lp, q.nu_lp), (p.nu_hp, q.nu_hp), (p.tt4, q.tt4), (p.mf, q.mf)].into_iter().enumerate() {
                let r = (x / y - 1.0).abs();
                if r > worst[k] { worst[k] = r; at[k] = i; }
            }
        }
        println!("{name:22} points {}/{} unchoked {unchoked}  worst rel diff  nuL {:.2e}@{}  nuH {:.2e}@{}  Tt4 {:.2e}@{}  mf {:.2e}@{}",
                 a.points.len(), b.points.len(), worst[0], at[0], worst[1], at[1], worst[2], at[2], worst[3], at[3]);
    }
}
