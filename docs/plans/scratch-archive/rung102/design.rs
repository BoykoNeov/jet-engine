// Rung 102 pre-anchor probe: an afterburner on the design-point turbojet, beside build_turbojet.
use turbojet::components::{choked_mfp, ram_recovery, Nozzle};
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{FlowState, Gas};

fn losses() -> Losses {
    Losses { pi_d: 0.97, eta_c: 0.88, eta_b: 0.99, pi_b: 0.96, eta_t: 0.90, eta_m: 0.99, pi_n: 0.98,
             ..Losses::default() }
}

// Non-equilibrium: fixed point on (1+f5) h_t(Tt5,f5) + eta_ab (f7-f5) hPR = (1+f7) h_t(Tt7,f7).
fn ab_far(gas: &Gas, tt5: f64, f5: f64, tt7: f64, eta_ab: f64) -> Result<f64, String> {
    let h5 = gas.h_t(tt5, f5);
    let mut f = f5;
    for _ in 0..200 {
        let h7 = gas.h_t(tt7, f);
        let fnew = f5 + (1.0 + f5) * (h7 - h5) / (eta_ab * gas.hpr() - h7);
        if (fnew - f).abs() <= 1e-13 * fnew.abs().max(1e-12) { return Ok(fnew); }
        f = fnew;
    }
    Err("no fixed point".into())
}

// Equilibrium: scale-B absolute balance per mol air, reactant = the station-4 FROZEN mixture at Tt5.
fn ab_far_eq(gas: &Gas, comp4: &[(&'static str, f64)], tt5: f64, f5: f64, tt7: f64, pt7: f64, eta_ab: f64)
    -> Result<f64, String>
{
    let react0 = gas.h_products_abs_b(comp4, tt5);
    let n5 = gas.n_fuel_per_air(f5);
    let (mut lo, mut hi) = (f5, gas.f_stoich_lean() * (1.0 - 1e-6));
    let (mut f, mut res, mut prod) = (f5, 0.0, 1.0);
    for _ in 0..200 {
        f = 0.5 * (lo + hi);
        let comp = gas.try_equilibrium_composition(f, tt7, pt7).map_err(|e| e.0)?;
        let nab = gas.n_fuel_per_air(f) - n5;
        prod = gas.h_products_abs_b(&comp, tt7);
        res = react0 + nab * gas.hf_fuel_molar() - prod - (1.0 - eta_ab) * nab * gas.lhv_molar();
        if hi - lo <= 1e-13 * f { break; }
        if res < 0.0 { lo = f } else { hi = f }
    }
    if res.abs() > 1e-6 * prod.abs() {
        return Err(format!("balance open at f={f:.6} (res {res:.3e}; lo {lo:.6} hi {hi:.6})"));
    }
    Ok(f)
}

fn mk_gas(name: &str) -> Gas {
    match name {
        "cpg" => Gas::default(),
        "tpg" => Gas::thermally_perfect(),
        "reacting" => Gas::reacting(),
        _ => Gas::reacting_equilibrium(),
    }
}

struct Out { st: f64, tsfc: f64, f: f64, a8: f64, v9: f64 }

fn nozzle_out(gas: &Gas, s: &FlowState, v0: f64, p0: f64, pi_n: f64) -> Out {
    let ex = Nozzle::new(p0, pi_n, None).apply(s, gas);
    let ftot = s.far;
    let st = (1.0 + ftot) * ex.v9 - v0;
    let a8 = (1.0 + ftot) * s.tt.sqrt() / (choked_mfp(gas, s.tt, s.far) * pi_n * s.pt);
    Out { st, tsfc: ftot / st, f: ftot, a8, v9: ex.v9 }
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let flights = [("take-off M0.2", FlightCondition::new(288.15, 101_325.0, 0.2)),
                   ("design M0.85", FlightCondition::new(250.0, 50_000.0, 0.85)),
                   ("M2 11km", FlightCondition::new(216.65, 22_632.0, 2.0))];
    let (pi_ab, eta_ab) = (0.95, 0.95);
    for gname in ["reacting", "equilibrium"] {
        println!("=== gas {gname}  (pi_c 10, Tt4 1500, pi_AB {pi_ab}, eta_AB {eta_ab}, fully expanded) ===");
        for (fname, fl) in flights {
            let mut l = losses();
            l.pi_d = 0.97 * ram_recovery(fl.m0);
            let eng = build_turbojet(mk_gas(gname), 10.0, 1500.0, fl.p0, l);
            let r = eng.run(&fl, 1.0);
            let s4 = *r.station("4"); let s5 = *r.station("5");
            let dry = nozzle_out(&eng.gas, &s5, r.v0, fl.p0, l.pi_n);
            println!("  {fname}: dry ST {:7.1} N.s/kg  TSFC {:.3} g/kN.s  f {:.5}  Tt5 {:.1} K  V9 {:.1}  (CLI ST {:.1})",
                     dry.st, dry.tsfc * 1e6, dry.f, s5.tt, dry.v9, r.performance.specific_thrust);
            for tt7 in [s5.tt, s5.tt + 1.0, 1800.0, 2000.0, 2200.0, 2300.0] {
                let pt7 = pi_ab * s5.pt;
                let res = if eng.gas.is_equilibrium() {
                    let comp4 = eng.gas.freeze_equilibrium(s4.far, s4.tt, s4.pt);
                    ab_far_eq(&eng.gas, &comp4, s5.tt, s5.far, tt7, pt7, eta_ab).map(|f| {
                        let g2 = Gas::reacting_equilibrium();
                        g2.freeze_equilibrium(f, tt7, pt7);
                        (f, g2)
                    })
                } else {
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| ab_far(&eng.gas, s5.tt, s5.far, tt7, eta_ab))).unwrap_or_else(|e| Err(e.downcast_ref::<String>().cloned().unwrap_or_default())).map(|f| (f, eng.gas.clone()))
                };
                match res {
                    Ok((f7, g7)) => {
                        let s7 = FlowState { tt: tt7, pt: pt7, mdot: 1.0 + f7, far: f7 };
                        let w = nozzle_out(&g7, &s7, r.v0, fl.p0, l.pi_n);
                        println!("    Tt7 {:6.1}: f {:.5} (phi {:.3})  ST {:7.1} (x{:.3})  TSFC {:.3} (x{:.3})  A8 x{:.4}  [sqrt(Tt7/Tt5)/pi_AB {:.4}]",
                                 tt7, f7, f7 / eng.gas.f_stoich_lean(), w.st, w.st / dry.st, w.tsfc * 1e6, w.tsfc / dry.tsfc,
                                 w.a8 / dry.a8, (tt7 / s5.tt).sqrt() / pi_ab);
                    }
                    Err(e) => println!("    Tt7 {:6.1}: REFUSED {e}", tt7),
                }
            }
        }
    }
}
