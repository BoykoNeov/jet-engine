//! RUNG 109 — THE TURBOFAN. Anchor: `docs/plans/rung109-anchor-turbofan.md`; spec: `docs/rung109-spec.md`.
//!
//! The CONSTRUCTION gates of anchor § 2 (no credit): bypass ratio 0 is the turbojet bit for bit, continuity, Mattingly
//! Example 7-6 through the shipped model, Mattingly's equations to 1e-12, the cold nozzle against the hot one, the
//! refusals. Then § 3's 276 pre-registered cells, scored with every number printed
//! (`cargo test --release --test rung109 -- --nocapture`).

use std::panic::{catch_unwind, AssertUnwindSafe};
use turbojet::components::Nozzle;
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{FlowState, Gas, GasSpec};
use turbojet::turbofan::{best_pi_f, build_turbofan, is_refusal, try_cold_nozzle, try_cold_sonic_throat, Fan,
                         TurbofanResult, NO_CORE_PRESSURE};

fn real_losses() -> Losses {
    Losses { pi_d: 0.97, eta_c: 0.88, eta_b: 0.99, pi_b: 0.96, eta_t: 0.90, eta_m: 0.99, pi_n: 0.98,
             ..Losses::default() }
}
fn design_flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }
fn sls() -> FlightCondition { FlightCondition::new(288.15, 101_325.0, 0.0) }
fn gases() -> [(&'static str, fn() -> Gas); 5] {
    [("perfect", Gas::default), ("thermally_perfect", Gas::thermally_perfect), ("reacting", Gas::reacting),
     ("fork_b", Gas::reacting_forkb), ("equilibrium", Gas::reacting_equilibrium)]
}
fn fan(alpha: f64, pi_f: f64) -> Fan { Fan { alpha, pi_f, ..Fan::default() } }

// --- § 2: α = 0 IS the turbojet ----------------------------------------------------------------------------------

#[test]
fn bypass_ratio_zero_is_the_turbojet_bit_for_bit() {
    let mut n = 0;
    for (name, g) in gases() {
        for fl in [design_flight(), sls()] {
            for mode in 0..3 {
                let mut l = real_losses();
                match mode {
                    1 => l.p_exit = Some(1.3 * fl.p0),
                    2 => l.nozzle_convergent = true,
                    _ => {}
                }
                let jet = build_turbojet(g(), 10.0, 1500.0, fl.p0, l).run(&fl, 20.0);
                let tf = build_turbofan(g(), 10.0, 1500.0, fl.p0, l, Fan::default()).run(&fl, 20.0);
                let (a, b) = (&jet.performance, &tf.core.performance);
                assert!(a.specific_thrust == b.specific_thrust && a.tsfc == b.tsfc && a.eta_thermal == b.eta_thermal
                        && a.eta_propulsive == b.eta_propulsive && a.eta_overall == b.eta_overall
                        && a.eta_brayton == b.eta_brayton, "{name} M{} mode {mode}: performance moved", fl.m0);
                assert_eq!(jet.stations.len(), tf.core.stations.len());
                for ((la, sa), (lb, sb)) in jet.stations.iter().zip(&tf.core.stations) {
                    assert!(la == lb && sa.tt == sb.tt && sa.pt == sb.pt && sa.far == sb.far && sa.mdot == sb.mdot);
                }
                assert!(jet.v9 == tf.core.v9 && jet.t9 == tf.core.t9 && jet.p9 == tf.core.p9 && tf.fan.is_none());
                assert!(tf.core_thrust == a.specific_thrust && tf.fan_thrust == 0.0);
                n += 1;
            }
        }
    }
    assert_eq!(n, 30);
}

#[test]
fn a_vanishing_fan_is_continuous_with_the_turbojet() {
    for (name, g) in gases() {
        for fl in [design_flight(), sls()] {
            let jet = build_turbojet(g(), 10.0, 1500.0, fl.p0, real_losses()).run(&fl, 1.0);
            let tf = build_turbofan(g(), 10.0, 1500.0, fl.p0, real_losses(), fan(1e-9, 1.6)).run(&fl, 1.0);
            let rel = |x: f64, y: f64| ((x - y) / y).abs();
            let (a, b) = (&jet.performance, &tf.core.performance);
            assert!(rel(b.specific_thrust, a.specific_thrust) < 1e-6, "{name}: thrust");
            assert!(rel(b.tsfc, a.tsfc) < 1e-6, "{name}: tsfc");
            assert!(rel(b.eta_thermal, a.eta_thermal) < 1e-6, "{name}: thermal efficiency");
            let s19 = tf.fan.unwrap().s13;
            assert!(s19.mdot == 1e-9 * 1.0, "bypass mass is alpha times the core's, exactly");
        }
    }
}

// --- § 2: Mattingly, Elements of Gas Turbine Propulsion (1996), Example 7-6 -------------------------------------------

const BTU_LBM_R: f64 = 4186.8;
const BTU_LBM: f64 = 2326.0;
const LBF_PER: f64 = 9.80665;        // 1 lbf/(lbm/s) in N·s/kg
const S_PER: f64 = 2.832_545e-5;     // 1 (lbm/h)/lbf in kg/(N·s)

struct Ex { gc: f64, cpc: f64, rc: f64, gt: f64, cpt: f64, rt: f64, hpr: f64 }
fn ex() -> Ex {
    let (gc, gt) = (1.4f64, 1.33f64);
    let (cpc, cpt) = (0.240 * BTU_LBM_R, 0.276 * BTU_LBM_R);
    Ex { gc, cpc, rc: (gc - 1.0) / gc * cpc, gt, cpt, rt: (gt - 1.0) / gt * cpt, hpr: 18_400.0 * BTU_LBM }
}
fn ex_gas() -> Gas {
    let m = ex();
    Gas::new(GasSpec { gamma_c: m.gc, cp_c: m.cpc, r_c: m.rc, gamma_t: m.gt, cp_t: m.cpt, r_t: m.rt, hpr: m.hpr,
                       ..GasSpec::default() })
}

/// One textbook case: inputs (Example 7-6's names) and the model run through `build_turbofan`.
#[derive(Clone, Copy)]
struct Case { t0: f64, m0: f64, pi_d: f64, pi_c: f64, pi_f: f64, alpha: f64, tt4: f64, p0p9: f64, p0p19: f64 }
const P0: f64 = 20_000.0;
fn ex76() -> Case {
    Case { t0: 390.0 / 1.8, m0: 0.8, pi_d: 0.99, pi_c: 36.0, pi_f: 1.7, alpha: 8.0, tt4: 3000.0 / 1.8, p0p9: 0.9,
           p0p19: 0.9 }
}
fn model(c: Case) -> TurbofanResult {
    let l = Losses { pi_d: c.pi_d, eta_c: 1.0, e_c: Some(0.90), eta_b: 0.99, pi_b: 0.96, eta_t: 1.0, e_t: Some(0.89),
                     eta_m: 0.99, pi_n: 0.99, p_exit: Some(P0 / c.p0p9), nozzle_convergent: false };
    assert_eq!(c.p0p9, c.p0p19, "one exit pressure for both nozzles (the fan nozzle follows the core's mode)");
    let f = Fan { alpha: c.alpha, pi_f: c.pi_f, eta_f: 1.0, e_f: Some(0.89), pi_fn: 0.99 };
    build_turbofan(ex_gas(), c.pi_c, c.tt4, P0, l, f).run(&FlightCondition::new(c.t0, P0, c.m0), 1.0)
}
/// Mattingly's equations (7-52a … af), SI: `(f, τt, F/ṁ0, S, η_T)`.
fn mattingly(c: Case) -> (f64, f64, f64, f64, f64) {
    let m = ex();
    let (pi_b, pi_n, pi_fn, ec, ef, et, eta_b, eta_m) = (0.96, 0.99, 0.99, 0.90, 0.89, 0.89, 0.99, 0.99);
    let a0 = (m.gc * m.rc * c.t0).sqrt();
    let tr = 1.0 + (m.gc - 1.0) / 2.0 * c.m0 * c.m0;
    let pr = tr.powf(m.gc / (m.gc - 1.0));
    let tl = m.cpt * c.tt4 / (m.cpc * c.t0);
    let tc = c.pi_c.powf((m.gc - 1.0) / (m.gc * ec));
    let tf = c.pi_f.powf((m.gc - 1.0) / (m.gc * ef));
    let f = (tl - tr * tc) / (m.hpr * eta_b / (m.cpc * c.t0) - tl);
    let tt = 1.0 - 1.0 / (eta_m * (1.0 + f)) * tr / tl * (tc - 1.0 + c.alpha * (tf - 1.0));
    let pit = tt.powf(m.gt / ((m.gt - 1.0) * et));
    let pt9p9 = c.p0p9 * pr * c.pi_d * c.pi_c * pi_b * pit * pi_n;
    let m9 = (2.0 / (m.gt - 1.0) * (pt9p9.powf((m.gt - 1.0) / m.gt) - 1.0)).sqrt();
    let t9t0 = tl * tt / pt9p9.powf((m.gt - 1.0) / m.gt) * m.cpc / m.cpt;
    let v9a0 = m9 * (m.gt * m.rt * t9t0 / (m.gc * m.rc)).sqrt();
    let pt19p19 = c.p0p19 * pr * c.pi_d * c.pi_f * pi_fn;
    let m19 = (2.0 / (m.gc - 1.0) * (pt19p19.powf((m.gc - 1.0) / m.gc) - 1.0)).sqrt();
    let t19t0 = tr * tf / pt19p19.powf((m.gc - 1.0) / m.gc);
    let v19a0 = m19 * t19t0.sqrt();
    let core = (1.0 + f) * v9a0 - c.m0 + (1.0 + f) * m.rt * t9t0 / (m.rc * v9a0) * (1.0 - c.p0p9) / m.gc;
    let fan = v19a0 - c.m0 + t19t0 / v19a0 * (1.0 - c.p0p19) / m.gc;
    let st = a0 / (1.0 + c.alpha) * (core + c.alpha * fan);
    let s = f / ((1.0 + c.alpha) * st);
    let ke = (1.0 + f) * v9a0 * v9a0 + c.alpha * v19a0 * v19a0 - (1.0 + c.alpha) * c.m0 * c.m0;
    (f, tt, st, s, a0 * a0 * ke / (2.0 * f * m.hpr))
}

#[test]
fn mattingly_example_7_6_through_the_shipped_model() {
    let c = ex76();
    let r = model(c);
    let fs = r.fan.unwrap();
    let a0 = (1.4 * ex().rc * c.t0).sqrt();
    let tt4 = c.tt4;
    let rows: [(&str, f64, f64); 17] = [
        ("tau_c", r.core.station("3").tt / r.core.station("2").tt, 3.119),
        ("tau_f", fs.s13.tt / r.core.station("2").tt, 1.1857),
        ("f", r.f(), 0.02868),
        ("tau_t", r.core.station("5").tt / tt4, 0.54866),
        ("pi_t", r.core.station("5").pt / r.core.station("4").pt, 0.06599),
        ("M9", r.core.m9, 1.394), ("T9/T0", r.core.t9 / c.t0, 3.196), ("V9/a0", r.core.v9 / a0, 2.427),
        ("M19", fs.exit.m9, 1.154), ("T19/T0", fs.exit.t9 / c.t0, 1.0561), ("V19/a0", fs.exit.v9 / a0, 1.186),
        ("F/m0", r.core.performance.specific_thrust / LBF_PER, 18.02),
        ("S", r.core.performance.tsfc / S_PER, 0.6366),
        ("FR", r.core_thrust / r.fan_thrust, 3.988),
        ("eta_T", r.core.performance.eta_thermal, 0.4098),
        ("eta_O", r.core.performance.eta_overall, 0.3060),
        ("eta_P", r.core.performance.eta_propulsive, 0.7465),
    ];
    for (k, m, b) in rows {
        let d = (m / b - 1.0).abs();
        println!("{k:8} model {m:12.6} book {b:10} diff {:.4} %", 100.0 * d);
        assert!(d < 1e-3, "{k}: model {m} vs book {b}");
    }
}

#[test]
fn mattinglys_equations_to_machine_precision() {
    let second = Case { t0: 288.15, m0: 0.0, pi_d: 0.99, pi_c: 25.0, pi_f: 1.5, alpha: 5.0, tt4: 1600.0, p0p9: 1.0,
                        p0p19: 1.0 };
    for c in [ex76(), Case { p0p9: 1.0, p0p19: 1.0, ..ex76() }, second] {
        let r = model(c);
        let (f, tt, st, s, eta_t) = mattingly(c);
        let rel = |x: f64, y: f64| ((x - y) / y).abs();
        let p = &r.core.performance;
        assert!(rel(r.f(), f) < 1e-12, "f {} vs {f}", r.f());
        assert!(rel(r.core.station("5").tt / c.tt4, tt) < 1e-12, "tau_t");
        assert!(rel(p.specific_thrust, st) < 1e-12, "F/m0 {} vs {st}", p.specific_thrust);
        assert!(rel(p.tsfc, s) < 1e-12, "S");
        assert!(rel(p.eta_thermal, eta_t) < 1e-12, "eta_T {} vs {eta_t}", p.eta_thermal);
    }
}

// --- § 2: the cold nozzle ------------------------------------------------------------------------------------------

#[test]
fn the_cold_nozzle_is_the_hot_one_on_a_single_perfect_gas() {
    let gas = Gas::default();          // cold section = hot section
    assert!(gas.cold_is_cpg() && gas.hot_is_cpg());
    let s = FlowState { tt: 700.0, pt: 300_000.0, mdot: 3.0, far: 0.0 };
    for nz in [Nozzle::new(101_325.0, 0.98, None), Nozzle::new(101_325.0, 0.98, Some(150_000.0)),
               Nozzle::convergent(101_325.0, 0.98), Nozzle::convergent(200_000.0, 0.98)] {
        let hot = nz.try_apply(&s, &gas).unwrap();
        let cold = try_cold_nozzle(&nz, &s, &gas).unwrap();
        assert!(hot.t9 == cold.t9 && hot.v9 == cold.v9 && hot.m9 == cold.m9 && hot.p9 == cold.p9
                && hot.state.pt == cold.state.pt, "{nz:?}");
    }
    // The convergent throat on a self-consistent perfect gas IS the closed form T* = 2 Tt/(γ+1).
    let (g, cp) = (1.4f64, 1004.5);
    let gas = Gas::new(GasSpec { gamma_c: g, cp_c: cp, r_c: (g - 1.0) / g * cp, gamma_t: g, cp_t: cp,
                                 r_t: (g - 1.0) / g * cp, ..GasSpec::default() });
    let (tstar, pstar, _) = try_cold_sonic_throat(&gas, 600.0, 200_000.0).unwrap();
    assert!(((tstar - 2.0 * 600.0 / (g + 1.0)) / tstar).abs() < 1e-12);
    assert!(((pstar - 200_000.0 * (2.0 / (g + 1.0)).powf(g / (g - 1.0))) / pstar).abs() < 1e-12);
}

#[test]
fn the_fan_s_walls_are_refused_in_words() {
    // Too much fan for the turbine: alpha 10 at pi_f 3.
    let e = build_turbofan(Gas::thermally_perfect(), 10.0, 1500.0, 50_000.0, real_losses(), fan(10.0, 3.0))
        .try_run(&design_flight(), 1.0).unwrap_err();
    assert_eq!(e.0, NO_CORE_PRESSURE);
    assert!(is_refusal(&e));
    // Standing still, a fan too weak to cover the inlet and duct losses.
    let e = build_turbofan(Gas::thermally_perfect(), 10.0, 1500.0, 101_325.0, real_losses(), fan(4.0, 1.02))
        .try_run(&sls(), 1.0).unwrap_err();
    assert!(e.0.starts_with("The fan's pressure rise (1.020) does not cover"), "{}", e.0);
    assert!(is_refusal(&e));
}

// --- § 3: the predictions ------------------------------------------------------------------------------------------

struct Tally { hit: usize, miss: usize, vacuous: usize }
impl Tally {
    fn score(&mut self, ok: Option<bool>) -> &'static str {
        match ok { Some(true) => { self.hit += 1; "HIT" } Some(false) => { self.miss += 1; "MISS" }
                   None => { self.vacuous += 1; "VACUOUS" } }
    }
}

/// One cell's measurements: TSFC and best fan per bypass ratio, and the jet-speed ratio at the best fan.
struct Cell { tsfc: Vec<Option<f64>>, best: Vec<Option<(f64, bool)>>, vratio: Vec<Option<f64>>, st_core: Vec<Option<f64>> }
const ALPHAS: [f64; 6] = [0.0, 1.0, 2.0, 4.0, 8.0, 12.0];

fn measure(g: fn() -> Gas, fl: FlightCondition, pi_c: f64, tt4: f64, convergent: bool, fixed_pf: Option<&[Option<(f64, bool)>]>) -> Option<Cell> {
    let mut l = real_losses();
    l.nozzle_convergent = convergent;
    let tf = build_turbofan(g(), pi_c, tt4, fl.p0, l, Fan::default());
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let jet = catch_unwind(AssertUnwindSafe(|| tf.run(&fl, 1.0))).ok();
    let core = catch_unwind(AssertUnwindSafe(|| tf.core_to_4(&fl, 1.0))).ok();
    std::panic::set_hook(prev);
    let (jet, core) = (jet?, core?);
    let mut cell = Cell { tsfc: vec![Some(jet.core.performance.tsfc)], best: vec![None],
                          vratio: vec![None], st_core: vec![Some(jet.thrust_per_core())] };
    for (k, &a) in ALPHAS.iter().enumerate().skip(1) {
        let fan_a = Fan { alpha: a, ..Fan::default() };
        let pick = match fixed_pf {
            Some(bs) => bs[k].map(|(p, top)| (p, top)),
            None => best_pi_f(&tf, &core, fan_a).map(|b| (b.pi_f, b.at_top || b.at_wall)),
        };
        let run = pick.and_then(|(p, _)| tf.try_finish(&core, Fan { pi_f: p, ..fan_a }).ok());
        cell.best.push(pick);
        cell.tsfc.push(run.as_ref().map(|r| r.core.performance.tsfc));
        cell.vratio.push(run.as_ref().map(|r| r.fan.unwrap().exit.v9 / r.core.v9));
        cell.st_core.push(run.as_ref().map(|r| r.thrust_per_core()));
    }
    Some(cell)
}

#[test]
fn predictions_section_3_scored() {
    let flights = [("standing SLS", sls()), ("3 km M0.5", FlightCondition::new(268.65, 70_120.0, 0.5)),
                   ("11 km M0.8", FlightCondition::new(216.65, 22_632.0, 0.8))];
    let gs: [(&str, fn() -> Gas, bool); 4] = [("perfect", Gas::default, false),
        ("thermally_perfect", Gas::thermally_perfect, true), ("reacting", Gas::reacting, true),
        ("equilibrium", Gas::reacting_equilibrium, true)];
    let mut t = [Tally { hit: 0, miss: 0, vacuous: 0 }, Tally { hit: 0, miss: 0, vacuous: 0 },
                 Tally { hit: 0, miss: 0, vacuous: 0 }, Tally { hit: 0, miss: 0, vacuous: 0 },
                 Tally { hit: 0, miss: 0, vacuous: 0 }, Tally { hit: 0, miss: 0, vacuous: 0 }];
    let f = |x: Option<f64>| x.map_or("—".to_string(), |v| format!("{v:.4}"));
    for (fname, fl) in flights {
        for pi_c in [15.0, 40.0] {
            for tt4 in [1400.0, 1700.0] {
                for (gname, g, scored) in gs {
                    let cell = measure(g, fl, pi_c, tt4, false, None);
                    let tag = if scored { "" } else { " (reported, not scored)" };
                    println!("\n== {fname}, pi_c {pi_c}, Tt4 {tt4}, {gname}{tag}");
                    let Some(c) = cell else {
                        println!("   the main cycle refuses: VACUOUS");
                        if scored { for k in 0..4 { t[k].score(None); } t[3].vacuous += 3; }
                        continue;
                    };
                    for (k, &a) in ALPHAS.iter().enumerate() {
                        println!("   alpha {a:4}: best pi_f {}  TSFC {} g/(kN.s)  V19/V9 {}  thrust/core {}",
                                 c.best[k].map_or("—".into(), |(p, top)| format!("{p:.4}{}", if top { " (edge)" } else { "" })),
                                 f(c.tsfc[k].map(|x| x * 1e6)), f(c.vratio[k]), f(c.st_core[k]));
                    }
                    let ts = &c.tsfc;
                    // P1: strictly falling 0 -> 1 -> 2 -> 4.
                    let p1 = (|| Some(ts[1]? < ts[0]? && ts[2]? < ts[1]? && ts[3]? < ts[2]?))();
                    // P2: the 8 -> 12 fall is less than a third of the 2 -> 4 fall (a rise counts as less).
                    let p2 = (|| Some(ts[4]? - ts[5]? < (ts[2]? - ts[3]?) / 3.0))();
                    // P3: best pi_f strictly falling through 2, 4, 8, 12.
                    let b = |k: usize| c.best[k].map(|x| x.0);
                    let p3 = (|| Some(b(3)? < b(2)? && b(4)? < b(3)? && b(5)? < b(4)?))();
                    if scored {
                        let (s1, s2, s3) = (t[0].score(p1), t[1].score(p2), t[2].score(p3));
                        print!("   P1 {s1}  P2 {s2}  P3 {s3}  P4");
                        for k in 2..6 {
                            let p4 = c.vratio[k].map(|v| (0.70..=0.88).contains(&v));
                            print!(" {}", t[3].score(p4));
                        }
                        println!();
                    }
                    // P6: standing still, alpha 8's thrust from the same core is 2.0-3.0 x the turbojet's.
                    if fname == "standing SLS" && scored {
                        let ratio = (|| Some(c.st_core[4]? / c.st_core[0]?))();
                        let p6 = ratio.map(|r| (2.0..=3.0).contains(&r));
                        println!("   P6 thrust ratio {}  {}", f(ratio), t[5].score(p6));
                    }
                    // P5: at 11 km, alpha 8 at the expanded best: convergent penalty below the turbojet's, and < 3 %.
                    if fname == "11 km M0.8" && scored {
                        let conv = measure(g, fl, pi_c, tt4, true, Some(&c.best));
                        let pen = |k: usize| (|| Some(conv.as_ref()?.tsfc[k]? / c.tsfc[k]? - 1.0))();
                        let (pj, pf) = (pen(0), pen(4));
                        let p5 = (|| Some(pf? < pj? && pf? < 0.03))();
                        println!("   P5 convergent penalty: turbojet {}  alpha 8 {}  {}", f(pj), f(pf), t[4].score(p5));
                    }
                }
            }
        }
    }
    let names = ["P1", "P2", "P3", "P4", "P5", "P6"];
    println!();
    for (k, n) in names.iter().enumerate() {
        println!("{n}: {} HIT, {} MISS, {} VACUOUS", t[k].hit, t[k].miss, t[k].vacuous);
    }
    let total: usize = t.iter().map(|x| x.hit + x.miss + x.vacuous).sum();
    assert_eq!(total, 276, "the anchor pre-registered 276 cells");
    // The score as first run (2026-11-19), frozen: 256/276. The misses are findings (docs/rung109-spec.md § 4):
    // P2's 12 are every standing-still cell (the named risk), P4's 8 are a best fan above the knob's top (6) and a
    // jet-speed ratio of 0.699 at bypass 12 (2).
    let got: Vec<(usize, usize, usize)> = t.iter().map(|x| (x.hit, x.miss, x.vacuous)).collect();
    assert_eq!(got, vec![(36, 0, 0), (24, 12, 0), (36, 0, 0), (136, 8, 0), (12, 0, 0), (12, 0, 0)]);
}
