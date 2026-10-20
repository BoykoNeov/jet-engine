// Probe 2: every slam that stops at its FIRST step (the slam_floors grid: five variants, six flights, four maps,
// starts 400..1300 K up to the first full slam) — is the start's own burner mixture leaner than the closure's lean
// wall (0.004)? If not, is the start's root within one scan step (0.04) below the wall's flow (a scan miss)?
use turbojet::sandbox_controls::{controls, ControlsSettings, CustomerMode, LeverChoice, CONTROL_SHAPES};
fn main() {
    let d = ControlsSettings::defaults();
    let (t_sl, p_sl) = turbojet::atmosphere::standard(0.0);
    let (mut n0, mut lean, mut miss, mut other, mut mid) = (0, 0, 0, 0, 0);
    let mut fmax_first_fail: f64 = 0.0;
    let mut fmin_ok: f64 = 1.0;
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
                let mut tt4 = 400.0;
                let mut line = String::new();
                while tt4 <= 1300.0 + 1e-9 {
                    if let Ok((sv, o)) = controls(&ControlsSettings { from: tt4, ..s }) {
                        let f = sv.start.close.f;
                        let m = sv.start.close.m_lp;
                        if o.points.len() == s.expected_points() { fmin_ok = fmin_ok.min(f); break; }
                        if o.points.is_empty() {
                            n0 += 1;
                            fmax_first_fail = fmax_first_fail.max(f);
                            let hi0 = m * f / 0.004;
                            let lo = (m * f / 0.065).max(0.02);
                            // last scan point below the root
                            let kk = ((m - lo) / 0.04).floor();
                            let next = lo + (kk + 1.0) * 0.04;
                            let tag = if f < 0.004 { lean += 1; "L" } else if next >= hi0 { miss += 1; "M" } else { other += 1; "?" };
                            line += &format!(" {tt4:.0}:{tag}{:.4}", f);
                        } else {
                            mid += 1;
                            let lf = o.points.last().unwrap().f;
                            line += &format!(" {tt4:.0}:mid{}pts,f={:.4}", o.points.len(), lf);
                        }
                    }
                    tt4 += 10.0;
                }
                println!("{variant} | {name} | {shape}:{line}");
            }
        }
    }
    println!("first-step stops {n0}: start leaner than 0.004 {lean}, root within one scan step of the wall {miss}, other {other}; mid-slam stops {mid}");
    println!("richest start that stopped at step 1: f={fmax_first_fail:.5}; leanest start whose whole slam ran: f={fmin_ok:.5}");
}
