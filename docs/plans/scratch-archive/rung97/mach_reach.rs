//! Mach reach on the Controls view (after rung 97): the default move per map and Mach, design air; unchoked counts.
use turbojet::matcher::Branch;
use turbojet::sandbox_controls::{controls, ControlsSettings, ControlsSolver};

fn main() {
    let d = ControlsSettings::defaults();
    for shape in 0..4 {
        for m0 in [0.05, 0.2, 0.4, 0.6, 0.85, 1.0, 1.2, 1.5, 1.8, 2.2, 2.6, 3.0, 3.5] {
            let s = ControlsSettings { shape, m0, ..d };
            match controls(&s) {
                Err(e) => println!("shape {shape} M {m0:>4}: REFUSED {}", e.0.chars().take(150).collect::<String>()),
                Ok((sv, o)) => println!("shape {shape} M {m0:>4}: sigma {:.3}  nuL {:.3}->{:.3} nuH {:.3}->{:.3}  F {:.1}->{:.1}  pts {}/{}  peakTt4 {:.0}  stop {}  unchoked {}",
                    s.density_ratio(), sv.start.nu_lp, sv.end.nu_lp, sv.start.nu_hp, sv.end.nu_hp,
                    ControlsSolver::thrust(&sv.start), ControlsSolver::thrust(&sv.end), o.points.len(), s.expected_points(),
                    o.points.iter().map(|x| x.tt4).fold(0.0, f64::max), o.stop.is_some(),
                    o.points.iter().filter(|x| x.branch != Branch::Choked).count()),
            }
        }
    }
}
