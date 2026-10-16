//! Reachability only. (1) EPR and the LP acceleration rise with fuel at frozen speeds (violation counts) + walls;
//! (2) the gross-thrust identity: two states with the SAME EPR give the same gross thrust? (relative difference —
//! decides gate vs prediction); nozzle branch; (3) the steady table monotone in thrust, EPR, nu_L;
//! (4) prototype-governed marches: ran to the end?, clamp counts, which leg bound.
use jet_rung96_probe::*;
use turbojet::sandbox_controls::{build, ControlsSettings, ControlsSolver, CONTROL_SHAPES};
use turbojet::matcher::Branch;

fn main() {
    for (dir, from, to) in [("accel", 1000.0, 1400.0), ("decel", 1400.0, 1000.0)] {
        for shape in 0..4 {
            let s = ControlsSettings { shape, from, to, ..ControlsSettings::defaults() };
            let sv = build(&s).unwrap();
            let pts = sv.march();
            let (mut viol, mut checked, mut subsonic) = ([0usize; 2], 0usize, 0usize);
            let mut samples: Vec<(f64, f64, f64, f64)> = Vec::new(); // (a, b, mf, epr) for the identity
            for p in pts.iter().step_by(5) {
                let Some((lo, hi)) = range(&sv, p.nu_lp, p.nu_hp, p.mf) else { continue };
                let mut prev: Option<[f64; 2]> = None;
                for k in 0..=40 {
                    let mf = lo * (hi / lo).powf(k as f64 / 40.0);
                    let Ok(i) = inst(&sv, p.nu_lp, p.nu_hp, mf) else { continue };
                    if i.base.branch != Branch::Choked { subsonic += 1 }
                    let v = [epr(&sv, &i.base), i.base.phi_lp_dot];
                    if let Some(pv) = prev { for j in 0..2 { if !(v[j] > pv[j]) { viol[j] += 1 } } }
                    prev = Some(v);
                    checked += 1;
                    if k % 10 == 5 { samples.push((p.nu_lp, p.nu_hp, mf, v[0])) }
                }
            }
            // (2) identity: for pairs of samples, find the fuel at the second state giving the first's EPR.
            let (mut worst, mut pairs, mut nohit) = (0.0f64, 0usize, 0usize);
            for (x, y) in samples.iter().zip(samples.iter().rev()).take(samples.len() / 2) {
                let ia = inst(&sv, x.0, x.1, x.2).unwrap();
                let (a, b) = (y.0, y.1);
                let Some((lo, hi)) = range(&sv, a, b, y.2) else { nohit += 1; continue };
                let r = |mf: f64| epr(&sv, &inst(&sv, a, b, mf).unwrap().base) - x.3;
                if r(lo) > 0.0 || r(hi) < 0.0 { nohit += 1; continue }
                let (mut u, mut w) = (lo, hi);
                for _ in 0..200 { if w - u <= 1e-15 * w { break } let m = 0.5 * (u + w); if r(m) < 0.0 { u = m } else { w = m } }
                let ib = inst(&sv, a, b, 0.5 * (u + w)).unwrap();
                let (ga, gb) = (gross(&sv, &ia.base), gross(&sv, &ib.base));
                worst = worst.max(((ga - gb) / ga).abs());
                pairs += 1;
            }
            println!("{dir} {:10} monotone: {checked} samples; non-rising EPR {} / LP accel {}; non-choked {subsonic}; \
                      identity: {pairs} pairs (no-hit {nohit}), worst |gross diff|/gross at equal EPR {worst:.2e}",
                     CONTROL_SHAPES[shape], viol[0], viol[1]);
            // (3) the steady table
            let rows = table(&sv, 13);
            let rising = |j: usize| rows.windows(2).all(|w| w[1][j] > w[0][j]);
            println!("    steady table (13 rows, sorted by thrust): EPR rising {}, nu_L rising {}, fuel rising {}",
                     rising(1), rising(2), rising(3));
        }
    }
    // (4) prototype-governed marches
    let base = ControlsSettings::defaults();
    let cases: Vec<(&str, ControlsSettings)> = vec![
        ("accel default", base),
        ("accel +redline", ControlsSettings { redline_on: true, ..base }),
        ("accel +sched", ControlsSettings { accel_on: true, ..base }),
        ("accel ramp 2.0", ControlsSettings { ramp: 2.0, ..base }),
        ("decel 1400->1000", ControlsSettings { from: 1400.0, to: 1000.0, ..base }),
        ("decel ramp 2.0", ControlsSettings { from: 1400.0, to: 1000.0, ramp: 2.0, ..base }),
    ];
    for (name, s) in cases {
        for shape in [0usize, 3] {
            let s = ControlsSettings { shape, ..s };
            let sv = build(&s).unwrap();
            let c = Ctx::new(&sv);
            for g in [Gov::Thrust, Gov::Epr, Gov::N1Rate(0.1), Gov::N1Rate(0.5), Gov::N1Prop(20.0)] {
                let (recs, stop) = march(&c, g, ds());
                let n_exp = s.expected_points();
                let cl = |k: Clamp| recs.iter().filter(|r| r.clamp == k).count();
                let l = |k: usize| recs.iter().filter(|r| r.leg == k).count();
                let ramp: Vec<&Rec> = recs.iter().filter(|r| r.s <= s.ramp + 1e-9).collect();
                let cr = |k: Clamp| ramp.iter().filter(|r| r.clamp == k).count();
                let nonchoked = recs.iter().filter(|r| r.i.base.branch != Branch::Choked).count();
                println!("{name:18} {:10} {g:?}: {}/{} pts{}; root/rich/lean {}/{}/{} (ramp {}/{}/{} of {}); legs none/redline/sched/floor {}/{}/{}/{}; non-choked {nonchoked}",
                         CONTROL_SHAPES[shape], recs.len(), n_exp, stop.map(|m| format!(" STOP: {m}")).unwrap_or_default(),
                         cl(Clamp::Root), cl(Clamp::Rich), cl(Clamp::Lean), cr(Clamp::Root), cr(Clamp::Rich), cr(Clamp::Lean), ramp.len(),
                         l(0), l(1), l(2), l(3));
            }
            let _ = ControlsSolver::thrust(&sv.start);
        }
    }
}
