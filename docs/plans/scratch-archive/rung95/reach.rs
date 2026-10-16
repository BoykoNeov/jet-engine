//! Reachability only: (1) thrust / Tt4 / HP speed-rate rise with fuel at frozen speeds (violation counts), and where
//! the solvable range ends (fuel-air ratio at each wall); (2) prototype-governed marches: ran to the end?, how many
//! recorded points the governor sat on its root / the rich wall / the lean wall, and which leg bound how often.
use jet_rung95_probe::*;
use turbojet::sandbox_controls::{build, ControlsSettings, CONTROL_SHAPES};

fn main() {
    // (1) monotonicity at states along the existing open-loop march (default slam, each shape)
    for shape in 0..4 {
        let s = ControlsSettings { shape, ..ControlsSettings::defaults() };
        let sv = build(&s).unwrap();
        let pts = sv.march();
        let (mut viol, mut checked) = ([0usize; 3], 0usize);
        let (mut f_lo_min, mut f_lo_max, mut f_hi_min, mut f_hi_max) = (1.0f64, 0.0f64, 1.0f64, 0.0f64);
        for p in pts.iter().step_by(5) {
            let Some((lo, hi)) = range(&sv, p.nu_lp, p.nu_hp, p.mf) else { continue };
            let fl = inst(&sv, p.nu_lp, p.nu_hp, lo).unwrap().base.close.f;
            let fh = inst(&sv, p.nu_lp, p.nu_hp, hi).unwrap().base.close.f;
            f_lo_min = f_lo_min.min(fl); f_lo_max = f_lo_max.max(fl);
            f_hi_min = f_hi_min.min(fh); f_hi_max = f_hi_max.max(fh);
            let mut prev: Option<[f64; 3]> = None;
            for k in 0..=40 {
                let mf = lo * (hi / lo).powf(k as f64 / 40.0);
                let Ok(i) = inst(&sv, p.nu_lp, p.nu_hp, mf) else { continue };
                let v = [q(Gov::Thrust, &i), q(Gov::Tt4, &i), q(Gov::SpeedRate(1.0), &i)];
                if let Some(pv) = prev { for j in 0..3 { if !(v[j] > pv[j]) { viol[j] += 1 } } }
                prev = Some(v);
                checked += 1;
            }
        }
        println!("{:10} monotone check: {checked} samples; non-rising steps thrust {} / Tt4 {} / HP rate {}; lean wall f {:.5}..{:.5}, rich wall f {:.5}..{:.5}",
                 CONTROL_SHAPES[shape], viol[0], viol[1], viol[2], f_lo_min, f_lo_max, f_hi_min, f_hi_max);
    }
    // (2) prototype-governed marches
    let base = ControlsSettings::defaults();
    let cases: Vec<(&str, ControlsSettings)> = vec![
        ("accel default", base),
        ("accel +redline", ControlsSettings { redline_on: true, ..base }),
        ("accel +sched", ControlsSettings { accel_on: true, ..base }),
        ("accel +redline+sched", ControlsSettings { redline_on: true, accel_on: true, ..base }),
        ("accel ramp 2.0", ControlsSettings { ramp: 2.0, ..base }),
        ("decel 1400->1000", ControlsSettings { from: 1400.0, to: 1000.0, ..base }),
        ("decel ramp 2.0", ControlsSettings { from: 1400.0, to: 1000.0, ramp: 2.0, ..base }),
    ];
    for (name, s) in cases {
        for shape in [0usize, 3] {
            let s = ControlsSettings { shape, ..s };
            let sv = build(&s).unwrap();
            for g in [Gov::Tt4, Gov::Thrust, Gov::SpeedRate(0.1), Gov::SpeedRate(0.5)] {
                let (recs, stop) = march(&sv, g, ds());
                let n_exp = s.expected_points();
                let c = |k: Clamp| recs.iter().filter(|r| r.clamp == k).count();
                let l = |k: usize| recs.iter().filter(|r| r.leg == k).count();
                // the ramp only (where the demand moves)
                let ramp: Vec<&Rec> = recs.iter().filter(|r| r.s <= s.ramp + 1e-9).collect();
                let cr = |k: Clamp| ramp.iter().filter(|r| r.clamp == k).count();
                println!("{name:22} {:10} {g:?}: {}/{} pts{}; root/rich/lean {}/{}/{} (ramp {}/{}/{} of {}); legs none/redline/sched/floor {}/{}/{}/{}",
                         CONTROL_SHAPES[shape], recs.len(), n_exp, stop.map(|m| format!(" STOP: {m}")).unwrap_or_default(),
                         c(Clamp::Root), c(Clamp::Rich), c(Clamp::Lean), cr(Clamp::Root), cr(Clamp::Rich), cr(Clamp::Lean), ramp.len(),
                         l(0), l(1), l(2), l(3));
            }
        }
    }
}
