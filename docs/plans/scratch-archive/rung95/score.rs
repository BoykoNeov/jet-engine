//! Rung 95 scoring — the SHIPPED device (`turbojet::governor` through the Controls view). Gates (§ 2) and H1–H6 (§ 3).
use std::time::Instant;
use turbojet::burner_hold::Hold;
use turbojet::fuel_transient::FuelPoint;
use turbojet::governor as gv;
use turbojet::sandbox_controls::{build, controls, ControlsSettings, ControlsSolver, GeneratorMode, SpeedLaw, CONTROL_SHAPES, DS};

fn thrust_of(sv: &ControlsSolver, p: &FuelPoint) -> f64 { gv::thrust(&sv.reread(p).unwrap().base) }
fn argmax(v: &[f64]) -> usize { (0..v.len()).fold(0, |m, k| if v[k] > v[m] { k } else { m }) }
fn argmin(v: &[f64]) -> usize { (0..v.len()).fold(0, |m, k| if v[k] < v[m] { k } else { m }) }
/// First time a rising series crosses `x`, linear between points.
fn cross(s: &[f64], y: &[f64], x: f64) -> Option<f64> {
    (1..y.len()).find(|&k| y[k - 1] < x && y[k] >= x).map(|k| s[k - 1] + (x - y[k - 1]) / (y[k] - y[k - 1]) * (s[k] - s[k - 1]))
}

fn main() {
    let d = ControlsSettings::defaults();
    let gov = |s: ControlsSettings| ControlsSettings { govern: true, ..s };
    println!("== § 2 GATES ==");
    for shape in 0..4 {
        let name = CONTROL_SHAPES[shape];
        // G-Tt4: governed burner temperature vs rung 40's temperature-commanded march on the same machine
        let s = gov(ControlsSettings { shape, ..d });
        let sv = build(&s).unwrap();
        let pts = sv.march();
        let g = sv.governor.as_ref().unwrap();
        let tp = sv.core.fuel.inner.integrate(&sv.flight, |x: f64| g.demand(x), (sv.start.nu_lp, sv.start.nu_hp), s.s_end(), DS);
        let dmax = pts.iter().zip(&tp).map(|(a, b)| (a.nu_lp - b.nu_lp).abs().max((a.nu_hp - b.nu_hp).abs())).fold(0.0, f64::max);
        println!("{name:10} G-Tt4: {} vs {} points, max |d nu| {dmax:.3e} (bar 1e-8)", pts.len(), tp.len());
        // G-fixed point + identity, thrust (no generator; HP generator 0.02) and speed rate, settle 8
        for (lbl, s) in [("thrust", ControlsSettings { hold: Hold::Thrust, settle: 8.0, ..gov(ControlsSettings { shape, ..d }) }),
                         ("thrust HPgen", ControlsSettings { hold: Hold::Thrust, settle: 8.0, generator: GeneratorMode::Hp, ..gov(ControlsSettings { shape, ..d }) }),
                         ("speed rate", ControlsSettings { hold: Hold::HpSpeed, settle: 8.0, ..gov(ControlsSettings { shape, ..d }) }),
                         ("speed prop", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, settle: 8.0, ..gov(ControlsSettings { shape, ..d }) })] {
            let (sv, o) = controls(&s).unwrap();
            let l = o.points.last().unwrap();
            let fe = gv::thrust(&sv.end);
            let g = sv.governor.as_ref().unwrap();
            let mut idmax = 0.0f64;
            for (p, m) in o.points.iter().zip(&o.marks) {
                if m.clamp != gv::Clamp::Root || m.leg != gv::Leg::None { continue }
                let i = sv.reread(p).unwrap().base;
                let e = match g.law {
                    gv::Law::Thrust => (gv::thrust(&i) - g.demand(p.s)).abs() / g.demand(p.s),
                    gv::Law::SpeedRate { tau } => (i.phi_hp_dot - (g.demand(p.s) - p.nu_hp) / tau).abs(),
                    _ => 0.0,
                };
                idmax = idmax.max(e);
            }
            println!("{name:10} G-fixed {lbl:12}: {}/{} pts, end dthrust/F {:.2e}, d nu_L {:.2e}, d nu_H {:.2e}; identity max {idmax:.2e}",
                     o.points.len(), s.expected_points(), (thrust_of(&sv, l) - fe) / fe, l.nu_lp - sv.end.nu_lp, l.nu_hp - sv.end.nu_hp);
        }
    }
    // step check + timing
    for (lbl, s) in [("thrust", ControlsSettings { hold: Hold::Thrust, ..gov(d) }),
                     ("rate 0.1", ControlsSettings { hold: Hold::HpSpeed, ..gov(d) }),
                     ("prop 20", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, ..gov(d) }),
                     ("prop 200", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: 200.0, ..gov(d) }),
                     ("prop 100", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: 100.0, ..gov(d) }),
                     ("prop 50", ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: 50.0, ..gov(d) })] {
        let sv = build(&s).unwrap();
        let g = sv.governor.as_ref().unwrap();
        let (a, _) = gv::march(&sv.core.fuel, &sv.flight, g, &sv.limiters(), None, (sv.start.nu_lp, sv.start.nu_hp), s.s_end(), DS);
        let (b, _) = gv::march(&sv.core.fuel, &sv.flight, g, &sv.limiters(), None, (sv.start.nu_lp, sv.start.nu_hp), s.s_end(), 0.005);
        let dm = a.iter().enumerate().filter_map(|(k, p)| b.get(4 * k).map(|q| (p.phi_lp - q.phi_lp).abs().max((p.phi_hp - q.phi_hp).abs()))).fold(0.0, f64::max);
        println!("step check {lbl:9}: {} / {} pts, max |d phi| 0.02 vs 0.005 = {dm:.2e}", a.len(), b.len());
    }
    for (lbl, s) in [("open, redline+sched", ControlsSettings { redline_on: true, accel_on: true, ..d }),
                     ("thrust, redline+sched", ControlsSettings { hold: Hold::Thrust, redline_on: true, accel_on: true, ..gov(d) }),
                     ("thrust, none", ControlsSettings { hold: Hold::Thrust, ..gov(d) })] {
        let t = Instant::now();
        let (_, o) = controls(&s).unwrap();
        println!("timing {lbl:22}: {:.3} s ({} pts)", t.elapsed().as_secs_f64(), o.points.len());
    }

    println!("\n== § 3 PREDICTIONS ==");
    for shape in 0..4 {
        let name = CONTROL_SHAPES[shape];
        for (dir, from, to) in [("accel", 1000.0, 1400.0), ("decel", 1400.0, 1000.0)] {
            let base = ControlsSettings { shape, from, to, hold: Hold::Thrust, ..d };
            let (_, op) = controls(&base).unwrap();
            let (_, gp) = controls(&gov(base)).unwrap();
            let idx: Vec<usize> = (1..op.points.len()).filter(|&k| op.points[k].s > 0.0 && op.points[k].s <= base.ramp + 1e-9).collect();
            let more = idx.iter().filter(|&&k| gp.points[k].mf > op.points[k].mf).count();
            let less = idx.iter().filter(|&&k| gp.points[k].mf < op.points[k].mf).count();
            let wmin = idx.iter().map(|&k| gp.points[k].mf / op.points[k].mf - 1.0).fold(f64::INFINITY, f64::min);
            let wmax = idx.iter().map(|&k| gp.points[k].mf / op.points[k].mf - 1.0).fold(f64::NEG_INFINITY, f64::max);
            println!("{name:10} H1 {dir}: governed fuel more/less than ramp at {more}/{less} of {} ramp points; ratio-1 range {wmin:+.4}..{wmax:+.4}",
                     idx.len());
            if dir == "accel" {
                let col = |o: &[FuelPoint], f: fn(&FuelPoint) -> f64| o.iter().map(f).collect::<Vec<f64>>();
                let (ot, gt) = (col(&op.points, |p| p.tt4), col(&gp.points, |p| p.tt4));
                let (ol, gl) = (col(&op.points, |p| p.phi_lp), col(&gp.points, |p| p.phi_lp));
                let (oh, gh) = (col(&op.points, |p| p.phi_hp), col(&gp.points, |p| p.phi_hp));
                println!("{name:10} H2: peak Tt4 gov {:.2} (#{}) vs open {:.2} (#{}); min phi_LP gov {:.5} (#{}) vs open {:.5} (#{}); min phi_HP gov {:.5} (#{}) vs open {:.5} (#{})",
                         gt[argmax(&gt)], argmax(&gt), ot[argmax(&ot)], argmax(&ot), gl[argmin(&gl)], argmin(&gl), ol[argmin(&ol)], argmin(&ol),
                         gh[argmin(&gh)], argmin(&gh), oh[argmin(&oh)], argmin(&oh));
                println!("{name:10} H3: governed peak Tt4 at index {} (ramp end = {}), clamps {}",
                         argmax(&gt), (base.ramp / DS).round(), gp.marks.iter().filter(|m| m.clamp != gv::Clamp::Root).count());
            }
        }
        // H4
        let mut taus = Vec::new();
        for (from, to) in [(990.0, 1000.0), (1390.0, 1400.0)] {
            let s = ControlsSettings { shape, from, to, ramp: 0.02, settle: 3.0, hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop,
                                       gov_gain: 20.0, ..gov(d) };
            let sv = build(&s).unwrap();
            let g = sv.governor.as_ref().unwrap();
            let mut row = Vec::new();
            for ds in [0.005, 0.02] {
                let (p, _) = gv::march(&sv.core.fuel, &sv.flight, g, &sv.limiters(), None, (sv.start.nu_lp, sv.start.nu_hp), s.s_end(), ds);
                let k0 = (0.02 / ds).round() as usize;
                let e: Vec<f64> = p.iter().map(|q| g.hi - q.nu_hp).collect();
                let e0 = e[k0];
                let k = (k0 + 1..e.len()).find(|&k| e[k] <= e0 / std::f64::consts::E);
                let t = k.map(|k| {
                    let (l0, l1) = (e[k - 1].ln(), e[k].ln());
                    let x = e0.ln() - 1.0;
                    p[k - 1].s + (x - l0) / (l1 - l0) * (p[k].s - p[k - 1].s) - p[k0].s
                });
                row.push(t);
            }
            println!("{name:10} H4 {from}->{to}: 1/e time after ramp, ds 0.005 {:?}, ds 0.02 {:?}", row[0], row[1]);
            taus.push(row[0]);
        }
        if let (Some(a), Some(b)) = (taus[0], taus[1]) { println!("{name:10} H4 ratio tau(1000)/tau(1400) = {:.3} (estimate 1.47)", a / b) }
        // H5
        let red = ControlsSettings { shape, hold: Hold::Thrust, redline_on: true, ..d };
        let (svo, oo) = controls(&red).unwrap();
        let (svg, og) = controls(&gov(red)).unwrap();
        let (f0, f1) = (gv::thrust(&svo.start), gv::thrust(&svo.end));
        let x = f0 + 0.95 * (f1 - f0);
        let ts = |sv: &ControlsSolver, o: &[FuelPoint]| cross(&o.iter().map(|p| p.s).collect::<Vec<_>>(),
                                                                &o.iter().map(|p| thrust_of(sv, p)).collect::<Vec<_>>(), x);
        println!("{name:10} H5: 95% thrust at s = governed {:?} vs open {:?} (redline 1480)", ts(&svg, &og.points), ts(&svo, &oo.points));
        // H6
        let peaks: Vec<f64> = [0.05, 0.1, 0.2, 0.5].iter().map(|&t| {
            let (_, o) = controls(&ControlsSettings { shape, hold: Hold::HpSpeed, gov_tau: t, ..gov(d) }).unwrap();
            o.points.iter().map(|p| p.tt4).fold(f64::NEG_INFINITY, f64::max)
        }).collect();
        println!("{name:10} H6: peak Tt4 at tau_N 0.05/0.1/0.2/0.5 = {:.2} / {:.2} / {:.2} / {:.2}", peaks[0], peaks[1], peaks[2], peaks[3]);
    }
}
