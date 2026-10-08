// Solver-tolerance audit, Rust era: rung 55's two stacked-efficiency secants.
// Sweeps gas x shape x K x stator x Tt4, records each loop's residual history up to the
// shipped exit and 60 secant steps PAST it (the true floor), and every raise.
use std::panic::{catch_unwind, AssertUnwindSafe};
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::stage::{probe_take, Split, StageStackCore, StageStackCoreSpec};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolLosses};

fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }
fn real() -> TwoSpoolLosses {
    TwoSpoolLosses { pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96,
        eta_hpt: 0.92, eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None,
        nozzle_convergent: true }
}
fn cpg() -> Gas {
    let (gc, cc, gt, ct) = (1.4f64, 1004.0f64, 1.3f64, 1239.0f64);
    Gas::new(GasSpec { gamma_c: gc, cp_c: cc, r_c: (gc - 1.0) / gc * cc, gamma_t: gt, cp_t: ct,
        r_t: (gt - 1.0) / gt * ct, hpr: 42.8e6, ..GasSpec::default() })
}
fn maps(name: &str) -> (ComponentMap, ComponentMap) {
    let f = ComponentMap::flat();
    let (l, h) = match name {
        "flow/press" => (ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f },
                         ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..f }),
        "press/flow" => (ComponentMap { a: 0.05, b: 0.20, sigma: 0.1, l: 1.0, ..f },
                         ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f }),
        "tilted" => (ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..f },
                     ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..f }),
        "steep" => (ComponentMap { a: 0.25, b: 0.12, sigma: 0.3, l: 1.2, ..f },
                    ComponentMap { a: 0.25, b: 0.12, sigma: 0.3, l: 1.2, ..f }),
        _ => (ComponentMap { sigma: 0.1, l: 0.7, ..f }, ComponentMap { sigma: 0.1, l: 1.0, ..f }),
    };
    (l.with_phi_surge(0.55), h.with_phi_surge(0.55))
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let shapes = ["flow/press", "press/flow", "tilted", "steep", "flat-eta"];
    println!("gas\tshape\tK\tvsv\tTt4\toutcome\tspool\texit_it\texit_r\tfloor\ttail_maxlast20\tmax_it");
    for (gname, mk) in [("cpg", cpg as fn() -> Gas), ("re", Gas::reacting_equilibrium as fn() -> Gas)] {
        let d = build_two_spool_turbojet(mk(), 3.0, 6.0, 1500.0, 50_000.0, real());
        for shape in shapes {
            for k in [2usize, 3, 4, 6] {
                for (vl, vh) in [(0.0, 0.0), (0.20, 0.10)] {
                    let (ml, mh) = maps(shape);
                    let st = StageStackCore::new(StageStackCoreSpec {
                        vsv_lp: vl, vsv_hp: vh, k_lp: k, k_hp: k, split: Split::DT,
                        vsv_stages_lp: None, vsv_stages_hp: None,
                        ..StageStackCoreSpec::new(d.clone(), flight(), 1.0, ml, mh)
                    });
                    let mut t = 800.0;
                    while t <= 1550.0 + 1e-9 {
                        let _ = probe_take();
                        let out = catch_unwind(AssertUnwindSafe(|| st.core.core.try_match_point(&flight(), t)));
                        let outcome = match &out { Ok(Ok(_)) => "ok", Ok(Err(_)) => "abort", Err(_) => "PANIC" };
                        let recs = probe_take();
                        if recs.is_empty() {
                            println!("{gname}\t{shape}\t{k}\t{vl}/{vh}\t{t}\t{outcome}\t-\t-\t-\t-\t-\t-");
                        }
                        for r in recs {
                            let exit_r = r.hist.last().copied().unwrap_or(f64::NAN).abs();
                            let floor = r.tail.iter().fold(exit_r, |a, x| a.min(x.abs()));
                            let last: Vec<f64> = r.tail.iter().rev().take(20).map(|x| x.abs()).collect();
                            let tmax = last.iter().fold(0.0f64, |a, x| a.max(*x));
                            let o = if r.raised { "RAISE" } else { outcome };
                            println!("{gname}\t{shape}\t{k}\t{vl}/{vh}\t{t}\t{o}\t{}\t{}\t{exit_r:.3e}\t{floor:.3e}\t{tmax:.3e}\t{}",
                                     r.spool, r.hist.len(), r.hist.len());
                        }
                        t += 5.0;
                    }
                }
            }
        }
    }
}
