//! Reported, not scored: per reading on the default accel, four maps — peak Tt4, time to 95 % of the TRUE thrust step,
//! the HP shaft's creep (time for its gap to the final speed to fall below 1e-3), min phi both spools.
use turbojet::burner_hold::Hold;
use turbojet::sandbox_controls::{controls, ControlsSettings, ControlsSolver, SpeedLaw, ThrustBy, CONTROL_SHAPES};
fn main() {
    hp_leads();
    let d = ControlsSettings { govern: true, hold: Hold::Thrust, ..ControlsSettings::defaults() };
    for shape in 0..4 {
        for (name, s) in [("open", ControlsSettings { govern: false, ..d }), ("true", d), ("EPR", ControlsSettings { thrust_by: ThrustBy::Epr, ..d }),
                          ("N1 rate .1", ControlsSettings { thrust_by: ThrustBy::N1, ..d }),
                          ("N1 prop 20", ControlsSettings { thrust_by: ThrustBy::N1, speed_law: SpeedLaw::Prop, ..d })] {
            let s = ControlsSettings { shape, settle: 4.0, ..s };
            let (sv, o) = controls(&s).unwrap();
            let p = &o.points;
            let th: Vec<f64> = p.iter().map(|x| sv.read(x).unwrap().thrust).collect();
            let (f0, f1) = (ControlsSolver::thrust(&sv.start), ControlsSolver::thrust(&sv.end));
            let x = f0 + 0.95 * (f1 - f0);
            let t95 = (1..th.len()).find(|&k| th[k - 1] < x && th[k] >= x).map(|k| p[k - 1].s + (x - th[k - 1]) / (th[k] - th[k - 1]) * (p[k].s - p[k - 1].s));
            let creep = p.iter().rposition(|q| (q.nu_hp - sv.end.nu_hp).abs() > 1e-3).map(|k| p[k].s);
            let over = th.iter().map(|t| (t - f1) / (f1 - f0)).fold(f64::NEG_INFINITY, f64::max);
            let pk = p.iter().map(|q| q.tt4).fold(f64::NEG_INFINITY, f64::max);
            let lp = p.iter().map(|q| q.phi_lp).fold(f64::INFINITY, f64::min);
            let hp = p.iter().map(|q| q.phi_hp).fold(f64::INFINITY, f64::min);
            println!("{:10} {name:10} peak Tt4 {pk:7.1}  t95 {:>6}  thrust overshoot {:+.3} of step  HP gap>1e-3 until s={:>5}  min phi LP {lp:.4} HP {hp:.4}",
                     CONTROL_SHAPES[shape], t95.map_or("never".into(), |t| format!("{t:.3}")), over, creep.map_or("-".into(), |c| format!("{c:.2}")));
        }
    }
}

/// H3's named condition, MEASURED: on the fan-speed accel (rate 0.1), does the HP shaft lead its steady speed at the same
/// LP speed? Steady (nu_L, nu_H) from 25 equilibria between the ends, linear between them.
pub fn hp_leads() {
    use turbojet::sandbox_controls::CONTROL_SHAPES;
    let d = ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::N1, ..ControlsSettings::defaults() };
    for (dir, from, to) in [("accel", 1000.0, 1400.0), ("decel", 1400.0, 1000.0)] {
        for shape in 0..4 {
            let s = ControlsSettings { shape, from, to, ..d };
            let (sv, o) = controls(&s).unwrap();
            let mut line: Vec<(f64, f64)> = (0..25).map(|k| {
                let t = sv.start.tt4 + (sv.end.tt4 - sv.start.tt4) * k as f64 / 24.0;
                let i = sv.core.fuel.inner.try_equilibrium(&sv.flight, t, None).unwrap().0;
                (i.nu_lp, i.nu_hp)
            }).collect();
            line.sort_by(|a, b| a.0.total_cmp(&b.0));
            let at = |x: f64| { let k = line.windows(2).position(|w| x <= w[1].0).unwrap_or(line.len() - 2).min(line.len() - 2);
                                let (a, b) = (line[k], line[k + 1]); a.1 + (b.1 - a.1) * (x - a.0) / (b.0 - a.0) };
            let lead: Vec<f64> = o.points.iter().filter(|p| p.s > 0.0 && p.s <= s.ramp + 1e-9).map(|p| p.nu_hp - at(p.nu_lp)).collect();
            let (lo, hi) = lead.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(l, h), &x| (l.min(x), h.max(x)));
            println!("HP lead {dir} {:10}: nu_H - nu_H,steady(nu_L) over the ramp {lo:+.4}..{hi:+.4}", CONTROL_SHAPES[shape]);
        }
    }
}
