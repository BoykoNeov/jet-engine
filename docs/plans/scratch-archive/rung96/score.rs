//! Rung 96 — the SHIPPED device scored: § 2 gates (reported numbers), § 3 H1–H4, the reported quantities, the step
//! check by law, the cost.
use std::time::Instant as Clock;
use turbojet::fuel_transient::FuelPoint;
use turbojet::governor as gv;
use turbojet::inferred_thrust as inf;
use turbojet::burner_hold::Hold;
use turbojet::sandbox_controls::{build, ControlsSettings, ControlsSolver, GeneratorMode, SpeedLaw, ThrustBy,
                                 CONTROL_SHAPES, DS};

fn run(s: &ControlsSettings) -> (ControlsSolver, Vec<FuelPoint>, Vec<gv::GovMark>) {
    let sv = build(s).unwrap_or_else(|e| panic!("{}", e.0));
    let (p, m) = sv.march_marked();
    (sv, p, m)
}

fn gaps(sv: &ControlsSolver, p: &[FuelPoint], ramp: f64) -> Vec<(f64, f64, f64)> {
    // (s, F_true - F_hat, F_true)
    p.iter().filter(|x| x.s > 0.0 && x.s <= ramp + 1e-9).map(|x| {
        let r = sv.read(x).unwrap();
        (x.s, r.thrust - r.inferred.unwrap(), r.thrust)
    }).collect()
}

fn main() {
    let d = ControlsSettings { govern: true, hold: Hold::Thrust, ..ControlsSettings::defaults() };
    let epr = ControlsSettings { thrust_by: ThrustBy::Epr, ..d };
    let n1 = ControlsSettings { thrust_by: ThrustBy::N1, ..d };
    let dirs = [("accel", 1000.0, 1400.0), ("decel", 1400.0, 1000.0)];

    println!("== H1 (EPR) / H3 (N1 rate 0.1): sign of F_true - F_hat over 0 < s <= ramp");
    for (name, base) in [("H1 EPR", epr), ("H3 N1", n1)] {
        for (dir, from, to) in dirs {
            for shape in 0..4 {
                let s = ControlsSettings { shape, from, to, ..base };
                let (sv, p, _) = run(&s);
                let g = gaps(&sv, &p, s.ramp);
                let step = (ControlsSolver::thrust(&sv.end) - ControlsSolver::thrust(&sv.start)).abs();
                let (lo, hi) = g.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(l, h), x| (l.min(x.1), h.max(x.1)));
                let want_pos = dir == "accel";
                let hit = g.iter().all(|x| if want_pos { x.1 > 0.0 } else { x.1 < 0.0 });
                let (smax, gmax) = g.iter().fold((0.0, 0.0f64), |(sm, gm), x| if x.1.abs() > gm { (x.0, x.1.abs()) } else { (sm, gm) });
                println!("{name} {dir} {:10} {} pts {}/{}: gap/step {:+.4}..{:+.4}; largest |gap| {:.4} of the step at s={smax:.2}  => {}",
                         CONTROL_SHAPES[shape], g.len(), p.len(), s.expected_points(), lo / step, hi / step, gmax / step,
                         if hit { "HIT" } else { "MISS" });
            }
        }
    }

    println!("== H2: EPR vs true-thrust governor, accel: peak Tt4, min phi_LP, min phi_HP (interior rule)");
    for shape in 0..4 {
        let (_, pe, _) = run(&ControlsSettings { shape, ..epr });
        let (_, pt, _) = run(&ControlsSettings { shape, ..d });
        let peak = |p: &[FuelPoint]| p.iter().map(|x| x.tt4).fold(f64::NEG_INFINITY, f64::max);
        let argmin = |p: &[FuelPoint], f: &dyn Fn(&FuelPoint) -> f64| p.iter().enumerate()
            .fold((0usize, f64::INFINITY), |(k, m), (j, x)| if f(x) < m { (j, f(x)) } else { (k, m) });
        let (ke, le) = argmin(&pe, &|x| x.phi_lp);
        let (kt, lt) = argmin(&pt, &|x| x.phi_lp);
        let (khe, he) = argmin(&pe, &|x| x.phi_hp);
        let (kht, ht) = argmin(&pt, &|x| x.phi_hp);
        let cell = |a: f64, b: f64, ka: usize, kb: usize| if ka == 0 || kb == 0 { "VACUOUS" } else if a < b { "HIT" } else { "MISS" };
        println!("{:10} peak Tt4 {:.1} vs {:.1} {} | min phi_LP {:.4}@{ke} vs {:.4}@{kt} {} | min phi_HP {:.4}@{khe} vs {:.4}@{kht} {}",
                 CONTROL_SHAPES[shape], peak(&pe), peak(&pt), if peak(&pe) > peak(&pt) { "HIT" } else { "MISS" },
                 le, lt, cell(le, lt, ke, kt), he, ht, cell(he, ht, khe, kht));
    }

    println!("== H4: EPR, clean table, generator power 0.02: end thrust vs target");
    for gm in [GeneratorMode::Hp, GeneratorMode::Lp] {
        for shape in 0..4 {
            let s = ControlsSettings { shape, generator: gm, ..epr };
            match build(&s) {
                Err(e) => println!("{gm:?} {:10} VACUOUS (refused: {})", CONTROL_SHAPES[shape], e.0),
                Ok(sv) => for (end, h) in [("start", sv.hold_start), ("end", sv.hold_end)] {
                    let h = h.unwrap();
                    println!("{gm:?} {:10} {end}: target {:.3} achieved {:.3} ({:+.3e} rel)  => {}", CONTROL_SHAPES[shape],
                             h.target, h.achieved, h.achieved / h.target - 1.0, if h.achieved > h.target { "HIT" } else { "MISS" });
                },
            }
        }
    }

    println!("== reported: N1 calibration error (clean table, generator 0.02), and the prop law's droop");
    for gm in [GeneratorMode::Hp, GeneratorMode::Lp] {
        for shape in [0usize, 3] {
            let s = ControlsSettings { shape, generator: gm, ..n1 };
            match build(&s) {
                Err(e) => println!("{gm:?} {:10} refused: {}", CONTROL_SHAPES[shape], e.0),
                Ok(sv) => {
                    let (h0, h1) = (sv.hold_start.unwrap(), sv.hold_end.unwrap());
                    println!("{gm:?} {:10} N1 ends: {:+.3e} / {:+.3e} rel", CONTROL_SHAPES[shape], h0.achieved / h0.target - 1.0,
                             h1.achieved / h1.target - 1.0);
                    let sp = ControlsSettings { speed_law: SpeedLaw::Prop, settle: 8.0, ..s };
                    let (sv, p, _) = run(&sp);
                    let l = p.last().unwrap();
                    let g = sv.governor.as_ref().unwrap();
                    println!("    prop K20 settle 8: nu_L {:.6} vs demand {:.6} (droop {:+.3e}); points {}/{}", l.nu_lp,
                             g.demand(l.s), l.nu_lp - g.demand(l.s), p.len(), sp.expected_points());
                }
            }
        }
    }

    println!("== gate: decomposition F_true - F_hat = V0 (face_hat - face), EPR; worst residual / thrust");
    let mut worst = 0.0f64;
    for s in [epr, ControlsSettings { from: 1400.0, to: 1000.0, ..epr }, ControlsSettings { generator: GeneratorMode::Hp, ..epr },
              ControlsSettings { generator: GeneratorMode::Lp, table_clean: false, ..epr }, ControlsSettings { cool_vane: 0.05, cool_blade: 0.03, ..epr },
              ControlsSettings { redline_on: true, accel_on: true, ..epr },
              ControlsSettings { lever: turbojet::sandbox_controls::LeverChoice::Bleed, ..epr },
              ControlsSettings { customer: turbojet::sandbox_controls::CustomerMode::Share, ..epr }] {
        for shape in 0..4 {
            let s = ControlsSettings { shape, ..s };
            let (sv, p, _) = run(&s);
            let t = sv.governor.as_ref().unwrap().infer.as_ref().unwrap();
            for x in &p {
                let r = sv.read(x).unwrap();
                let i = sv.reread(x).unwrap();
                let e = inf::epr(&i.base, t.pt2);
                let res = (r.thrust - r.inferred.unwrap()) - t.v0 * (t.face_at_epr(e) - inf::face(&i.base));
                worst = worst.max((res / r.thrust).abs());
            }
        }
    }
    println!("worst {worst:.2e} (bar 1e-12)");

    println!("== gate: generator off => table_clean inert; loaded table ends = rung 94 ends");
    for shape in 0..4 {
        let (_, a, _) = run(&ControlsSettings { shape, ..epr });
        let (_, b, _) = run(&ControlsSettings { shape, table_clean: false, ..epr });
        let same = a.len() == b.len() && a.iter().zip(&b).all(|(x, y)| x.nu_lp.to_bits() == y.nu_lp.to_bits() && x.mf.to_bits() == y.mf.to_bits());
        let s94 = ControlsSettings { shape, generator: GeneratorMode::Hp, hold: Hold::Thrust, ..ControlsSettings::defaults() };
        let sv94 = build(&s94).unwrap();
        let svl = build(&ControlsSettings { shape, generator: GeneratorMode::Hp, table_clean: false, ..epr }).unwrap();
        println!("{:10} clean-off inert bitwise {same}; loaded ends == rung 94's {} {}", CONTROL_SHAPES[shape],
                 sv94.start == svl.start, sv94.end == svl.end);
    }

    println!("== gate: hold identity + fixed point (settle 8)");
    for (name, base) in [("EPR", epr), ("N1 rate", n1), ("EPR clean gen HP", ControlsSettings { generator: GeneratorMode::Hp, ..epr }),
                         ("N1 clean gen LP", ControlsSettings { generator: GeneratorMode::Lp, ..n1 })] {
        let s = ControlsSettings { settle: 8.0, ..base };
        let (sv, p, m) = run(&s);
        let g = sv.governor.as_ref().unwrap();
        let mut hold = 0.0f64;
        for (x, mk) in p.iter().zip(&m) {
            if mk.clamp != gv::Clamp::Root || mk.leg != gv::Leg::None { continue }
            let i = sv.reread(x).unwrap();
            let q = gv::held_by(g, &sv.core.fuel, &i.base);
            let want = match g.law { gv::Law::LpRate { tau } => (g.demand(x.s) - x.nu_lp) / tau, _ => g.demand(x.s) };
            hold = hold.max(((q - want) / want.abs().max(1e-3)).abs());
        }
        let l = p.last().unwrap();
        println!("{name:18} hold identity worst {hold:.1e}; settled |dnu| {:.1e} {:.1e}; thrust vs end {:.1e} rel",
                 (l.nu_lp - sv.end.nu_lp).abs(), (l.nu_hp - sv.end.nu_hp).abs(),
                 sv.read(l).unwrap().thrust / ControlsSolver::thrust(&sv.end) - 1.0);
    }

    println!("== step check: 0.02 vs 0.005, max |dphi| over the run (LP laws), accel and decel");
    for (dir, from, to) in dirs {
        for law in [("EPR", ControlsSettings { ..epr }), ("rate 0.02", ControlsSettings { gov_tau: 0.02, ..n1 }),
                    ("rate 0.04", ControlsSettings { gov_tau: 0.04, ..n1 }), ("rate 0.1", n1),
                    ("prop 20", ControlsSettings { speed_law: SpeedLaw::Prop, ..n1 }),
                    ("prop 50", ControlsSettings { speed_law: SpeedLaw::Prop, gov_gain: 50.0, ..n1 }),
                    ("prop 100", ControlsSettings { speed_law: SpeedLaw::Prop, gov_gain: 100.0, ..n1 })] {
            for shape in [0usize, 3] {
                let s = ControlsSettings { shape, from, to, ..law.1 };
                let Ok(sv) = build(&s) else { println!("{dir} {} refused", law.0); continue };
                let g = sv.governor.as_ref().unwrap();
                let nu0 = (sv.start.nu_lp, sv.start.nu_hp);
                let lim = sv.limiters();
                let (a, ma) = gv::march(&sv.core.fuel, &sv.flight, g, &lim, None, nu0, s.s_end(), DS);
                let (b, _) = gv::march(&sv.core.fuel, &sv.flight, g, &lim, None, nu0, s.s_end(), DS / 4.0);
                let mut dmax = 0.0f64;
                for (k, x) in a.iter().enumerate() {
                    if let Some(y) = b.get(4 * k) { dmax = dmax.max((x.phi_lp - y.phi_lp).abs().max((x.phi_hp - y.phi_hp).abs())) }
                }
                let lean = ma.iter().filter(|m| m.clamp == gv::Clamp::Lean).count();
                println!("{dir} {:10} {:10} pts {}/{} max dphi {dmax:.1e} (lean pts {lean})", law.0, CONTROL_SHAPES[shape], a.len(), b.len());
            }
        }
    }

    println!("== cost");
    for (name, s) in [("open", ControlsSettings::defaults()), ("true", d), ("EPR", epr), ("N1", n1),
                      ("EPR +redline+sched", ControlsSettings { redline_on: true, accel_on: true, ..epr }),
                      ("EPR clean gen HP", ControlsSettings { generator: GeneratorMode::Hp, ..epr })] {
        let c = Clock::now();
        let (_, p, _) = run(&s);
        println!("{name:20} {:.3} s ({} pts)", c.elapsed().as_secs_f64(), p.len());
    }
}
