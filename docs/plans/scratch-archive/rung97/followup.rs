//! Rung 97 follow-up (advisor): at frozen shaft speeds, do thrust, EPR and both accelerations still rise strictly with
//! fuel where the nozzle is unchoked (and across the unchoke boundary)? And rung 96's G2 at unchoked instants: equal
//! EPR => equal gross thrust? Plus, at the six H1 ends: face airflow, pt5/p0 and specific thrust, new vs old.
use turbojet::atmosphere::standard;
use turbojet::inferred_thrust as inf;
use turbojet::matcher::Branch;
use turbojet::sandbox_controls::{build, ControlsSettings, ControlsSolver, GeneratorMode};

fn main() {
    let (t, p) = standard(0.0);
    let w = ControlsSettings { t0: t + 40.0, p0: p, from: 700.0, to: 1400.0, ..ControlsSettings::defaults() };
    let mut worst_g2 = 0.0f64; let mut g2_pairs = 0;
    for (name, s) in [("plain", w), ("generator LP", ControlsSettings { generator: GeneratorMode::Lp, ..w })] {
        for shape in 0..3 {
            let sv = build(&ControlsSettings { shape, ..s }).unwrap();
            let (fl, core) = (sv.flight, &sv.core.fuel);
            let (_, pt2, v0) = core.inner.inlet(&fl);
            let (mut samples, mut viol, mut unch_samples, mut crossing_sweeps) = (0, [0usize; 4], 0, 0);
            for pt in sv.march().iter().step_by(2) {
                // 41 fuel flows from 0.5x to 2x the point's fuel, log-spaced; keep the evaluable ones
                let mut prev: Option<[f64; 4]> = None; let mut branches = (false, false);
                for k in 0..41 {
                    let mf = pt.mf * (0.5f64).powf(1.0 - 2.0 * k as f64 / 40.0);
                    let Ok(fi) = core.try_instant_fuel(&fl, pt.nu_lp, pt.nu_hp, mf) else { prev = None; continue };
                    let i = &fi.base;
                    if i.branch == Branch::Choked { branches.0 = true } else { branches.1 = true; unch_samples += 1 }
                    let q = [ControlsSolver::thrust(i), inf::epr(i, pt2), i.phi_lp_dot, i.phi_hp_dot];
                    if let Some(pv) = prev { samples += 1; for j in 0..4 { if !(q[j] > pv[j]) { viol[j] += 1 } } }
                    prev = Some(q);
                }
                if branches.0 && branches.1 { crossing_sweeps += 1 }
            }
            println!("{name} shape {shape}: {samples} fuel steps ({unch_samples} unchoked samples, {crossing_sweeps} sweeps crossing the boundary); non-rising: thrust {} EPR {} dnuL {} dnuH {}", viol[0], viol[1], viol[2], viol[3]);
            // G2 pairs: two unchoked instants at different speeds, matched in EPR by fuel; compare gross thrust
            let un: Vec<_> = sv.march().into_iter().filter(|x| x.branch != Branch::Choked).collect();
            for w2 in un.windows(2) {
                let (a, b) = (&w2[0], &w2[1]);
                let ia = core.try_instant_fuel(&fl, a.nu_lp, a.nu_hp, a.mf).unwrap().base;
                let target = inf::epr(&ia, pt2);
                let e = |mf: f64| core.try_instant_fuel(&fl, b.nu_lp, b.nu_hp, mf).map(|x| inf::epr(&x.base, pt2) - target);
                let (mut lo, mut hi) = (b.mf * 0.7, b.mf * 1.3);
                let (Ok(el), Ok(eh)) = (e(lo), e(hi)) else { continue };
                if el * eh > 0.0 { continue }
                for _ in 0..100 { let m = 0.5 * (lo + hi); if e(m).unwrap() * el > 0.0 { lo = m } else { hi = m } }
                let ib = core.try_instant_fuel(&fl, b.nu_lp, b.nu_hp, 0.5 * (lo + hi)).unwrap().base;
                if ib.branch == Branch::Choked { continue }
                let gross = |i: &turbojet::two_spool_transient::Instant2| ControlsSolver::thrust(i) + v0 * inf::face(i);
                worst_g2 = worst_g2.max((gross(&ib) / gross(&ia) - 1.0).abs()); g2_pairs += 1;
            }
        }
    }
    println!("rung 96 G2 at unchoked instants: {g2_pairs} pairs, worst |gross thrust ratio - 1| at equal EPR {worst_g2:.2e} (choked: ~1e-15)");
    // the H1 ends: what makes the thrust higher
    for (lab, dt) in [("+40 K", 40.0), ("ISA", 0.0)] {
        for shape in 0..3 {
            let sv = build(&ControlsSettings { shape, t0: t + dt, p0: p, from: 700.0, ..ControlsSettings::defaults() }).unwrap();
            let core = &sv.core.fuel.inner; let fl = sv.flight;
            core.inner.base.lp_pinned.set(true);
            let o = core.try_equilibrium(&fl, 700.0, None).unwrap().0;
            core.inner.base.lp_pinned.set(false);
            let n = &sv.start;
            let npr = |i: &turbojet::two_spool_transient::Instant2| i.pi_lpt * i.pi_hpt * i.close.pt4 / fl.p0;
            println!("{lab} shape {shape}: face air {:.5} vs {:.5} ({:+.2} %) | pt5/p0 {:.4} vs {:.4} | thrust per face air {:.2} vs {:.2} ({:+.2} %) | thrust {:+.2} %",
                inf::face(n), inf::face(&o), (inf::face(n) / inf::face(&o) - 1.0) * 100.0, npr(n), npr(&o),
                ControlsSolver::thrust(n) / inf::face(n), ControlsSolver::thrust(&o) / inf::face(&o),
                (ControlsSolver::thrust(n) / inf::face(n) / (ControlsSolver::thrust(&o) / inf::face(&o)) - 1.0) * 100.0,
                (ControlsSolver::thrust(n) / ControlsSolver::thrust(&o) - 1.0) * 100.0);
        }
    }
}
