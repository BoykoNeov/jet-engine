// Probe 3 (built against HEAD and against the throwaway lean-wall-lowered worktree): per variant, flight and map, the
// lowest start that builds, the lowest whose whole default slam finishes, and how the slams between stop (points, the
// last point's mixture); plus, at every start whose slam takes a first step, how far that first point is from the
// start's own steady state (mixture, LP-face flow).
use turbojet::sandbox_controls::{controls, ControlsSettings, CustomerMode, LeverChoice, CONTROL_SHAPES};
fn main() {
    let d = ControlsSettings::defaults();
    let (t_sl, p_sl) = turbojet::atmosphere::standard(0.0);
    let (mut first0, mut rich, mut other, mut worst_df) = (0, 0, 0, 0.0f64);
    for variant in ["plain", "bleed lever", "stator lever", "customer", "cooling"] {
        for (name, t0, p0, m0) in [("design", d.t0, d.p0, d.m0), ("M1.6", d.t0, d.p0, 1.6), ("M2", d.t0, d.p0, 2.0),
                                   ("M2.6", d.t0, d.p0, 2.6), ("SL std", t_sl, p_sl, 0.85), ("SL+40", t_sl + 40.0, p_sl, 0.85)] {
            for (k, shape) in CONTROL_SHAPES.iter().enumerate() {
                let mut s = ControlsSettings { shape: k, t0, p0, m0, ..d };
                match variant {
                    "bleed lever" => s.lever = LeverChoice::Bleed,
                    "stator lever" => s.lever = LeverChoice::Stator,
                    "customer" => s.customer = CustomerMode::Share,
                    "cooling" => { s.cool_vane = 0.03; s.cool_blade = 0.02; }
                    _ => {}
                }
                let (mut built, mut full) = (None, None);
                let mut line = String::new();
                let mut tt4 = 400.0;
                while tt4 <= 1300.0 + 1e-9 {
                    if let Ok((sv, o)) = controls(&ControlsSettings { from: tt4, ..s }) {
                        built.get_or_insert(tt4);
                        if let Some(p) = o.points.first() {
                            let df = (p.f - sv.start.close.f).abs() / sv.start.close.f;
                            worst_df = worst_df.max(df);
                        }
                        if o.points.len() == s.expected_points() { full = Some(tt4); break; }
                        let lf = o.points.last().map(|p| p.f).unwrap_or(f64::NAN);
                        if o.points.is_empty() { first0 += 1; } else if lf > 0.055 { rich += 1; } else { other += 1; }
                        line += &format!(" {tt4:.0}:{}p,f={:.4}", o.points.len(), lf);
                    }
                    tt4 += 10.0;
                }
                println!("{variant} | {name} | {shape}: builds {built:?}, whole slam {full:?};{line}");
            }
        }
    }
    println!("stops below the whole-slam floor: at the first step {first0}, ending rich (last f > 0.055) {rich}, other {other}");
    println!("first point vs the start's own mixture, worst relative difference: {worst_df:.3e}");
}
