//! RUNG 39 — TWO-SPOOL + COMPONENT MAPS: the cascade acquires a DIRECTION.
//!
//! Port of `tests/test_rung39.py`, gate for gate. Its ten gates:
//!
//!   1. REDUCE — FLAT maps on both spools reproduce rung 38 BIT-FOR-BIT, and the efficiencies
//!      really are the design ones (the map is inert, not merely small).
//!   2. REDUCE (the ladder) — `lp_disabled` dispatch: flat → rung 31, shaped → rung 32. With
//!      gate 1 that closes all four rungs through one dispatch.
//!   3. NON-TAUTOLOGICAL — an INDEPENDENT bare-math CPG two-spool MAP cascade: its own
//!      closed-form thermodynamics, its own turbine and speed-line bisections, and efficiency
//!      fixed points by DAMPED SUBSTITUTION rather than the shipped secant. Two code paths, one
//!      operating point.
//!   4. FINDING A — THE ASYMMETRY. `eta_LPC` leaves `pi_HPC` BIT-FOR-BIT unchanged (the (†)
//!      cancellation) while `eta_HPC` MOVES `pi_LPC`, negatively. CONTRAST: the turbine
//!      efficiencies move BOTH — so this is not "the spools don't talk".
//!   5. FINDING A — the weak back-arrow: a TURBINE map DOES open the closed leaf, ≥50× weaker.
//!      The RATIO is disclaimed; only the sign and the order of magnitude are gated.
//!   6. FINDING B1 — `slip == 1` on CPG + flat maps is a STRUCTURAL identity: exact at every
//!      throttle AND under a deliberately forced `f`, because `(1+f)` cancels in `N_L/N_H`.
//!   7. FINDING B2 — the rung-31-gate-5 MIRROR: the SAME flat maps break the identity on the
//!      variable-`cp` gases, and on the SAME CPG gas the MAP channel is the larger of the two.
//!   8. FINDING B3 — direction: `N_L/N_H` falls monotonically with throttle, ≥3 shape pairs.
//!      Magnitude DISCLAIMED; only the sign is gated.
//!   9. SCOPE GUARD — nozzle unchoke raises the documented rung-38 scope error.
//!  10. CYCLE UNTOUCHED — the default single-spool design path is bit-for-bit rung 6.

use turbojet::components::ram_recovery;
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::{ComponentMap, MapMatcher};
use turbojet::matcher::OffDesignMatcher;
use turbojet::two_spool::{build_two_spool_turbojet, counters, CascadeMap, MatchedMap,
                          TwoSpoolLosses, TwoSpoolMapMatcher, TwoSpoolMatcher};

const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

fn real() -> TwoSpoolLosses {
    TwoSpoolLosses {
        pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96,
        eta_hpt: 0.92, eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98,
        p_exit: None, nozzle_convergent: true,
    }
}

fn cpg_gas() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: 286.9,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn single_design() -> turbojet::engine::Engine {
    build_turbojet(Gas::reacting_equilibrium(), PI_HPC, TT4, 50_000.0, Losses {
        pi_d: 0.97, eta_c: 0.88, eta_b: 0.99, pi_b: 0.96, eta_t: 0.92, eta_m: 0.99,
        pi_n: 0.98, nozzle_convergent: true, ..Losses::default()
    })
}

fn mm(gas: Gas, map_lp: ComponentMap, map_hp: ComponentMap) -> TwoSpoolMapMatcher {
    let design = build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, 50_000.0, real());
    TwoSpoolMapMatcher::new(design, flight(), 1.0, map_lp, map_hp)
}

fn flat_mm(gas: Gas) -> TwoSpoolMapMatcher {
    mm(gas, ComponentMap::flat(), ComponentMap::flat())
}

/// Rung 39's OWN compressor-island-only shape pairs (`a_t = 0`) — the CLEAN structural test for
/// finding A. Copied from `tests/test_rung39.py::SHAPES_C`; note `l`, which is why slice K put
/// rung 34's linear loading slope on the Rust `ComponentMap` (§ 5.7 (a)).
fn shapes_c() -> Vec<(&'static str, ComponentMap, ComponentMap)> {
    let f = ComponentMap::flat();
    vec![
        ("flow_dom", ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f },
                     ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f }),
        ("press_dom", ComponentMap { a: 0.05, b: 0.20, sigma: 0.1, l: 1.0, ..f },
                      ComponentMap { a: 0.05, b: 0.20, sigma: 0.1, l: 1.0, ..f }),
        ("tilted", ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..f },
                   ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..f }),
        ("mixed", ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f },
                  ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..f }),
    ]
}

/// The converged `(Tt2, pt2, f, pt4)` of a matched point — rung 38 gate 3's ISOLATION PROTOCOL,
/// so the outer `f` loop's own (separately disclosed) cross-talk cannot confound a perturbation.
fn fixed_inputs(m: &TwoSpoolMapMatcher, tt4: f64) -> (f64, f64, f64, f64) {
    let od = m.match_point(&flight(), tt4).two();
    let c = &m.core().base;
    let (state0, _) = c.freestream_for(&flight());
    let tt2 = state0.tt;
    let pt2 = c.pi_d_max * ram_recovery(flight().m0) * state0.pt;
    let f = od.base.station("4").far;
    let pt4 = c.pi_b * od.base.pi_hpc * od.base.pi_lpc * pt2;
    (tt2, pt2, f, pt4)
}

/// Re-run the cascade with ONE captured efficiency lowered by `delta`, then restore it.
fn perturbed(m: &mut TwoSpoolMapMatcher, wgas: &Gas, tt2: f64, pt2: f64, tt4: f64, f: f64,
             which: &str, delta: f64) -> CascadeMap {
    let core = m.core_mut();
    let old = match which {
        "eta_lpc" => core.base.eta_lpc,
        "eta_hpc" => core.base.eta_hpc,
        "eta_hpt" => core.base.eta_hpt,
        "eta_lpt" => core.base.eta_lpt,
        _ => unreachable!(),
    };
    match which {
        "eta_lpc" => core.base.eta_lpc = old - delta,
        "eta_hpc" => core.base.eta_hpc = old - delta,
        "eta_hpt" => core.base.eta_hpt = old - delta,
        "eta_lpt" => core.base.eta_lpt = old - delta,
        _ => unreachable!(),
    }
    let out = m.core().cascade_map(wgas, tt2, pt2, tt4, f);
    let core = m.core_mut();
    match which {
        "eta_lpc" => core.base.eta_lpc = old,
        "eta_hpc" => core.base.eta_hpc = old,
        "eta_hpt" => core.base.eta_hpt = old,
        "eta_lpt" => core.base.eta_lpt = old,
        _ => unreachable!(),
    }
    out
}

// --------------------------------------------------------------------------------- gate 1
#[test]
fn gate1_reduce_flat_maps_is_rung38() {
    let design38 = build_two_spool_turbojet(Gas::reacting_equilibrium(), PI_LPC, PI_HPC, TT4,
                                            50_000.0, real());
    let r38 = TwoSpoolMatcher::new(design38, flight(), 1.0);
    let r39 = flat_mm(Gas::reacting_equilibrium());

    for tt4 in [1500.0, 1300.0, 1100.0, 900.0] {
        let a = r38.match_point(&flight(), tt4).two();
        let b = r39.match_point(&flight(), tt4).two();
        // The flat map holds every eta at design and the remaining arithmetic is rung 38's with
        // two independent sub-expressions reordered — which lands BIT-FOR-BIT. It only does so
        // because both efficiency loops CHECK the residual before calling the secant; a
        // do-while shape would converge to the same place and break this line (§ 5.7).
        assert_eq!(a.pi_lpc.to_bits(), b.base.pi_lpc.to_bits(), "pi_lpc at Tt4={tt4}");
        assert_eq!(a.pi_hpc.to_bits(), b.base.pi_hpc.to_bits(), "pi_hpc at Tt4={tt4}");
        assert_eq!(a.tau_hpt.to_bits(), b.base.tau_hpt.to_bits());
        assert_eq!(a.tau_lpt.to_bits(), b.base.tau_lpt.to_bits());
        assert_eq!(a.mdot_air.to_bits(), b.base.mdot_air.to_bits());
        assert_eq!(a.thrust.to_bits(), b.base.thrust.to_bits());
        // ...and the efficiencies really are the design ones — the map is INERT, not small.
        assert_eq!(b.eta_lpc.to_bits(), 0.90f64.to_bits());
        assert_eq!(b.eta_hpc.to_bits(), 0.88f64.to_bits());
        assert_eq!(b.eta_hpt.to_bits(), 0.92f64.to_bits());
        assert_eq!(b.eta_lpt.to_bits(), 0.90f64.to_bits());
    }
}

// --------------------------------------------------------------------------------- gate 2
#[test]
fn gate2_reduce_lp_disabled_ladder() {
    let flat_deg = TwoSpoolMapMatcher::lp_disabled(single_design(), flight(), 1.0,
                                                   ComponentMap::flat());
    let r31 = OffDesignMatcher::new(single_design(), flight(), 1.0);
    // Python's `ComponentMap.surge_pressure()` — NOT the same shape as rung 32's
    // `pressure_dominated`, which is why it is spelled out.
    let shaped = ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, a_t: 0.02,
                                ..ComponentMap::flat() };
    let shp_deg = TwoSpoolMapMatcher::lp_disabled(single_design(), flight(), 1.0, shaped);
    let r32 = MapMatcher::new(single_design(), flight(), 1.0, shaped);

    assert!(matches!(flat_deg, TwoSpoolMapMatcher::Degenerate(_)));

    for tt4 in [1500.0, 1300.0, 1000.0] {
        let a = r31.match_point(&flight(), tt4);
        let MatchedMap::Single(b) = flat_deg.match_point(&flight(), tt4) else {
            panic!("lp_disabled must dispatch");
        };
        assert_eq!(a.pi_c.to_bits(), b.base.pi_c.to_bits());
        assert_eq!(a.mdot_air.to_bits(), b.base.mdot_air.to_bits());
        assert_eq!(a.thrust.to_bits(), b.base.thrust.to_bits());

        let p = r32.match_point(&flight(), tt4);
        let MatchedMap::Single(q) = shp_deg.match_point(&flight(), tt4) else {
            panic!("lp_disabled must dispatch");
        };
        assert_eq!(p.base.pi_c.to_bits(), q.base.pi_c.to_bits());
        assert_eq!(p.eta_c.to_bits(), q.eta_c.to_bits());
        assert_eq!(p.n_ratio.to_bits(), q.n_ratio.to_bits());
        assert_eq!(p.base.thrust.to_bits(), q.base.thrust.to_bits());
    }
}

// --------------------------------------------------------------------------------- gate 3
#[test]
fn gate3_independent_cpg_map_cascade() {
    let (gamma_c, gamma_t) = (1.4, 1.3);
    let (cp_c, cp_t, hpr) = (1004.0, 1239.0, 42.8e6);
    let (gc, gt) = ((gamma_c - 1.0) / gamma_c, (gamma_t - 1.0) / gamma_t);
    let (e_lpc0, e_hpc0) = (0.90, 0.88);
    let (eta_hpt, eta_lpt) = (0.92, 0.90);
    let (eta_m, eta_b, pi_n) = (0.99, 0.99, 0.98);
    let (_, map_lp, map_hp) = shapes_c().into_iter().find(|(n, _, _)| *n == "mixed").unwrap();
    let m = mm(cpg_gas(), map_lp, map_hp);

    // Freestream + the design point, in closed form — no Gas/Component/ComponentMap call.
    let stag = 1.0 + 0.5 * (gamma_c - 1.0) * 0.85f64.powi(2);
    let tt2 = 250.0 * stag;
    let tt25_d = tt2 * (1.0 + (PI_LPC.powf(gc) - 1.0) / e_lpc0);
    let tt3_d = tt25_d * (1.0 + (PI_HPC.powf(gc) - 1.0) / e_hpc0);
    let f_d = (cp_t * TT4 - cp_c * tt3_d) / (eta_b * hpr - cp_t * TT4);
    let (tau_lpc_d, tau_hpc_d) = (tt25_d / tt2, tt3_d / tt25_d);

    let bisect = |fna: &dyn Fn(f64) -> f64, mut lo: f64, mut hi: f64| -> f64 {
        let mut flo = fna(lo);
        assert!(flo * fna(hi) < 0.0, "bare bracket fails");
        for _ in 0..300 {
            let mid = 0.5 * (lo + hi);
            let fm = fna(mid);
            if flo * fm <= 0.0 { hi = mid; } else { lo = mid; flo = fm; }
            if hi - lo < 1e-14 { break; }
        }
        0.5 * (lo + hi)
    };
    let turbine = |area_ratio: f64, eta_t: f64| -> f64 {
        let tau = move |p: f64| 1.0 - eta_t * (1.0 - p.powf(gt));
        let pi_t = bisect(&|p: f64| p / tau(p).sqrt() - area_ratio, 0.02, 0.999);
        tau(pi_t)
    };
    let psi = |cm: &ComponentMap, phi: f64| {
        1.0 - cm.sigma * (phi - 1.0).powi(2) - cm.l * (phi - 1.0)
    };
    let solve_n = |cm: &ComponentMap, mflow: f64, tau_c: f64, tau_c_d: f64| -> f64 {
        let target = (tau_c - 1.0) / (tau_c_d - 1.0);
        bisect(&|n: f64| psi(cm, mflow / n) * n * n - target, 0.1, 2.0)
    };
    let eta_at = |cm: &ComponentMap, base: f64, phi: f64, n: f64| {
        base - cm.a * (phi - 1.0).powi(2) - cm.b * (n - 1.0).powi(2)
            - cm.c * (phi - 1.0) * (n - 1.0)
    };

    let c = &m.core().base;
    let (area_hp, area_lp) = (c.a4 / c.a45, c.a45 / (c.a8 * pi_n));

    let bare = |tt4: f64| -> (f64, f64, f64, f64, f64, f64) {
        let tau_hpt = turbine(area_hp, eta_hpt);
        let tt45 = tt4 * tau_hpt;
        let tau_lpt = turbine(area_lp, eta_lpt);
        let tt5 = tt45 * tau_lpt;

        let (mut f, mut pi_lpc, mut pi_hpc) = (f_d, f64::NAN, f64::NAN);
        let (mut e_l, mut e_h, mut n_l, mut n_h) = (e_lpc0, e_hpc0, f64::NAN, f64::NAN);
        for _ in 0..300 {
            let tt25 = tt2 + eta_m * (1.0 + f) * cp_t * (tt45 - tt5) / cp_c;
            let tt3 = tt25 + eta_m * (1.0 + f) * cp_t * (tt4 - tt45) / cp_c;

            // HP efficiency fixed point — CLOSED (no LP quantity anywhere), by (†). MFP* and
            // A4*pi_b cancel against the design normalisation on a CPG gas. Solved by DAMPED
            // SUBSTITUTION: a genuinely different iteration from the shipped secant.
            e_h = e_hpc0;
            for _ in 0..600 {
                pi_hpc = (1.0 + e_h * (tt3 / tt25 - 1.0)).powf(1.0 / gc);
                let m_h = (pi_hpc / PI_HPC) * (tt25 / tt25_d).sqrt()
                    * (TT4 / tt4).sqrt() * (1.0 + f_d) / (1.0 + f);
                n_h = solve_n(&map_hp, m_h, tt3 / tt25, tau_hpc_d);
                let tgt = eta_at(&map_hp, e_hpc0, m_h / n_h, n_h);
                if (tgt - e_h).abs() < 1e-15 { break; }
                e_h += 0.5 * (tgt - e_h);
            }

            // LP efficiency fixed point — carries pi_HPC, by (‡).
            e_l = e_lpc0;
            for _ in 0..600 {
                pi_lpc = (1.0 + e_l * (tt25 / tt2 - 1.0)).powf(1.0 / gc);
                let m_l = (pi_hpc * pi_lpc / (PI_HPC * PI_LPC))
                    * (TT4 / tt4).sqrt() * (1.0 + f_d) / (1.0 + f);
                n_l = solve_n(&map_lp, m_l, tt25 / tt2, tau_lpc_d);
                let tgt = eta_at(&map_lp, e_lpc0, m_l / n_l, n_l);
                if (tgt - e_l).abs() < 1e-15 { break; }
                e_l += 0.5 * (tgt - e_l);
            }

            let f_new = (cp_t * tt4 - cp_c * tt3) / (eta_b * hpr - cp_t * tt4);
            if (f_new - f).abs() < 1e-14 { break; }
            f = f_new;
        }
        (pi_lpc, pi_hpc, e_l, e_h, n_l, n_h)
    };

    for tt4 in [1500.0, 1300.0, 1100.0, 1000.0] {
        let (pl, ph, el, eh, nl, nh) = bare(tt4);
        let od = m.match_point(&flight(), tt4).two();
        assert!((od.base.pi_lpc - pl).abs() < 1e-8 * pl, "{tt4}: {} vs {pl}", od.base.pi_lpc);
        assert!((od.base.pi_hpc - ph).abs() < 1e-8 * ph, "{tt4}: {} vs {ph}", od.base.pi_hpc);
        assert!((od.eta_lpc - el).abs() < 1e-9, "{tt4}: {} vs {el}", od.eta_lpc);
        assert!((od.eta_hpc - eh).abs() < 1e-9, "{tt4}: {} vs {eh}", od.eta_hpc);
        assert!((od.n_lp - nl).abs() < 1e-8, "{tt4}: {} vs {nl}", od.n_lp);
        assert!((od.n_hp - nh).abs() < 1e-8, "{tt4}: {} vs {nh}", od.n_hp);
    }
}

// --------------------------------------------------------------------------------- gate 4
#[test]
fn gate4_finding_a_the_asymmetry() {
    let d = 0.01;
    for (gas_name, make_gas) in [("cpg", (|| cpg_gas()) as fn() -> Gas),
                                 ("reacting", Gas::reacting_equilibrium)] {
        let all = shapes_c();
        let sel: Vec<_> = if gas_name == "cpg" {
            all.clone()
        } else {
            all.iter().filter(|(n, _, _)| *n == "mixed").cloned().collect()
        };
        for (name, map_lp, map_hp) in sel {
            assert!(map_lp.a_t == 0.0 && map_hp.a_t == 0.0,
                    "gate 4 is the a_t = 0 STRUCTURAL test");
            for tt4 in [1400.0, 1200.0, 1000.0] {
                let mut m = mm(make_gas(), map_lp, map_hp);
                let (tt2, pt2, f, pt4) = fixed_inputs(&m, tt4);
                let owned = m.core().base.working_gas(f, tt4, pt4);
                let wgas = match owned {
                    Some(g) => g,
                    None => m.core().gas().clone(),
                };
                let base = m.core().cascade_map(&wgas, tt2, pt2, tt4, f);

                // THE LEAF THAT SURVIVES: eta_LPC cannot reach pi_HPC. Bit-for-bit.
                let ql = perturbed(&mut m, &wgas, tt2, pt2, tt4, f, "eta_lpc", d);
                assert_eq!(ql.c.pi_hpc.to_bits(), base.c.pi_hpc.to_bits(),
                           "{gas_name}/{name}/{tt4}: eta_LPC reached pi_HPC");
                assert_eq!(ql.c.tt3.to_bits(), base.c.tt3.to_bits());
                assert_eq!(ql.c.tt25.to_bits(), base.c.tt25.to_bits());
                assert_ne!(ql.c.pi_lpc.to_bits(), base.c.pi_lpc.to_bits(),
                           "eta_LPC must move its OWN ratio");

                // THE ARROW THE MAP OPENS: eta_HPC DOES reach pi_LPC, negatively.
                let qh = perturbed(&mut m, &wgas, tt2, pt2, tt4, f, "eta_hpc", d);
                let arrow = qh.c.pi_lpc / base.c.pi_lpc - 1.0;
                assert!(arrow < 0.0, "{gas_name}/{name}/{tt4}: arrow {arrow}");
                assert!(arrow.abs() > 1e-5, "{gas_name}/{name}/{tt4}: arrow {arrow}");

                // CONTRAST: the turbine/energy-path parameters move BOTH ratios.
                for attr in ["eta_hpt", "eta_lpt"] {
                    let q = perturbed(&mut m, &wgas, tt2, pt2, tt4, f, attr, d);
                    assert_ne!(q.c.pi_lpc.to_bits(), base.c.pi_lpc.to_bits(),
                               "{gas_name}/{name}/{tt4}/{attr}");
                    assert_ne!(q.c.pi_hpc.to_bits(), base.c.pi_hpc.to_bits(),
                               "{gas_name}/{name}/{tt4}/{attr}");
                }
            }
        }
    }
}

// --------------------------------------------------------------------------------- gate 5
#[test]
fn gate5_finding_a_weak_back_arrow() {
    let d = 0.01;
    for (name, ml, mh) in shapes_c() {
        let map_lp = ComponentMap { a_t: 0.02, ..ml };
        let map_hp = ComponentMap { a_t: 0.02, ..mh };
        for tt4 in [1400.0, 1200.0, 1000.0] {
            let mut m = mm(cpg_gas(), map_lp, map_hp);
            let (tt2, pt2, f, pt4) = fixed_inputs(&m, tt4);
            let owned = m.core().base.working_gas(f, tt4, pt4);
            let wgas = match owned { Some(g) => g, None => m.core().gas().clone() };
            let base = m.core().cascade_map(&wgas, tt2, pt2, tt4, f);

            let ql = perturbed(&mut m, &wgas, tt2, pt2, tt4, f, "eta_lpc", d);
            let qh = perturbed(&mut m, &wgas, tt2, pt2, tt4, f, "eta_hpc", d);
            let back = (ql.c.pi_hpc / base.c.pi_hpc - 1.0).abs();
            let arrow = (qh.c.pi_lpc / base.c.pi_lpc - 1.0).abs();
            // The turbine map must OPEN the leaf — a strictly nonzero back-arrow — and the
            // forward one must still dominate. The 50x bar is LOOSE against a measured
            // 119x-548x, and loose because the ratio rides on the representative a_t = 0.02,
            // not because 50 is physically meaningful.
            assert!(back > 0.0, "{name}/{tt4}: a turbine map must OPEN the closed leaf");
            assert!(arrow > 50.0 * back, "{name}/{tt4}: arrow {arrow} vs back {back}");
        }
    }
}

// --------------------------------------------------------------------------------- gate 6
#[test]
fn gate6_finding_b1_slip_identity_is_structural() {
    let m = flat_mm(cpg_gas());
    for tt4 in [1500.0, 1300.0, 1100.0, 900.0] {
        let od = m.match_point(&flight(), tt4).two();
        assert!((od.slip - 1.0).abs() < 1e-9, "{tt4}: slip {}", od.slip);
        assert!((od.n_lp_ratio - od.n_hp_ratio).abs() < 1e-9 * od.n_hp_ratio);
    }

    // The (1+f) cancellation, exercised DIRECTLY: force f far off its solved value. Both shaft
    // works are eta_m*(1+f)*cp_t*Tt4*[pure geometry], so (1+f) AND Tt4 cancel in N_L/N_H — the
    // identity is f- and Tt4-independent, not a design-point coincidence.
    let (tt2, pt2, f, pt4) = fixed_inputs(&m, 1200.0);
    for f_forced in [0.5 * f, f, 2.0 * f, 4.0 * f] {
        let owned = m.core().base.working_gas(f_forced, 1200.0, pt4);
        let wgas = match owned { Some(g) => g, None => m.core().gas().clone() };
        let c = m.core().cascade_map(&wgas, tt2, pt2, 1200.0, f_forced);
        assert!((c.slip - 1.0).abs() < 1e-9, "f={f_forced}: slip {}", c.slip);
    }
}

// --------------------------------------------------------------------------------- gate 7
#[test]
fn gate7_finding_b2_mirror_and_map_dominance() {
    const COLD: f64 = 900.0;
    let cpg_flat = flat_mm(cpg_gas()).match_point(&flight(), COLD).two().slip;
    assert!((cpg_flat - 1.0).abs() < 1e-9);

    // MIRROR: the variable-cp gases drift on the SAME flat maps.
    let mut gas_channel: f64 = 0.0;
    for (name, g) in [("tpg", Gas::thermally_perfect()),
                      ("eq", Gas::reacting_equilibrium())] {
        let slip = flat_mm(g).match_point(&flight(), COLD).two().slip;
        let drift = (1.0 - slip).abs();
        assert!(drift > 5e-3, "{name}: slip {slip}");
        gas_channel = gas_channel.max(drift);
    }

    // DOMINANCE: on the SAME CPG gas, the map channel is the larger of the two.
    let (_, map_lp, map_hp) = shapes_c().into_iter().find(|(n, _, _)| *n == "mixed").unwrap();
    let map_channel = (1.0 - mm(cpg_gas(), map_lp, map_hp)
        .match_point(&flight(), COLD).two().slip).abs();
    assert!(map_channel > gas_channel, "map {map_channel} vs gas {gas_channel}");
}

// --------------------------------------------------------------------------------- gate 8
#[test]
fn gate8_finding_b3_slip_direction_shape_robust() {
    let grid = [1500.0, 1300.0, 1100.0, 900.0];
    for (name, ml, mh) in shapes_c() {
        let m = mm(cpg_gas(), ComponentMap { a_t: 0.02, ..ml },
                   ComponentMap { a_t: 0.02, ..mh });
        let slips: Vec<f64> = grid.iter()
            .map(|&t| m.match_point(&flight(), t).two().slip).collect();
        assert!((slips[0] - 1.0).abs() < 1e-9, "{name}: design is the datum, got {}", slips[0]);
        for i in 0..slips.len() - 1 {
            assert!(slips[i + 1] < slips[i], "{name}: {slips:?}");
        }
    }
}

// --------------------------------------------------------------------------------- gate 9
#[test]
fn gate9_scope_guard_unchoke_raises() {
    let (_, map_lp, map_hp) = shapes_c().into_iter().find(|(n, _, _)| *n == "mixed").unwrap();
    let m = mm(cpg_gas(), map_lp, map_hp);
    let hush = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let mut raised = false;
    for tt4 in [700.0, 650.0, 600.0, 550.0, 500.0, 450.0] {
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
            || m.match_point(&flight(), tt4)));
        if let Err(e) = r {
            let msg = e.downcast_ref::<String>().cloned()
                .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_default();
            if msg.contains("OUT OF SCOPE") {
                raised = true;
                break;
            }
            break;   // a DIFFERENT failure first: that is not this gate passing
        }
    }
    std::panic::set_hook(hush);
    assert!(raised, "deep throttle must raise the rung-38 'OUT OF SCOPE' unchoke error");
}

// --------------------------------------------------------------------------------- gate 10
#[test]
fn gate10_cycle_untouched_rung6() {
    let plain = || build_turbojet(Gas::reacting_equilibrium(), 10.0, TT4, 50_000.0, Losses {
        pi_d: 0.97, eta_c: 0.88, eta_b: 0.99, pi_b: 0.96, eta_t: 0.92, eta_m: 0.99,
        pi_n: 0.98, ..Losses::default()
    });
    let before = plain().run(&flight(), 1.0);

    let (_, map_lp, map_hp) = shapes_c().into_iter().find(|(n, _, _)| *n == "mixed").unwrap();
    let _ = mm(cpg_gas(), map_lp, map_hp).match_point(&flight(), 1200.0);

    let after = plain().run(&flight(), 1.0);
    assert_eq!(before.performance.specific_thrust.to_bits(),
               after.performance.specific_thrust.to_bits());
    assert_eq!(before.performance.tsfc.to_bits(), after.performance.tsfc.to_bits());
    for (label, s) in &before.stations {
        let t = after.station(label);
        assert_eq!(s.tt.to_bits(), t.tt.to_bits(), "station {label} Tt");
        assert_eq!(s.pt.to_bits(), t.pt.to_bits(), "station {label} pt");
    }
}

// ------------------------------------------------------------------- the check-first shape
/// **THE `hp_passes_min` / `lp_passes_min` CLAIM, PINNED ABSOLUTELY — NOT AGAINST PYTHON.**
///
/// The oracle carries these two keys, but a comparison gate only asserts *Rust agrees with
/// Python*: if BOTH sides ran the efficiency loop `do`-while — secant first, residual tested
/// after — both would dump `1`, the comparison would pass clean, and the shape claim would be
/// unwitnessed. A bit-equality gate is blind to an assumption the two sides SHARE, so the claim
/// needs an absolute bar of its own.
///
/// The claim: on a flat map the efficiency residual is already met on entry, so the loop returns
/// having called the secant ZERO times. `0` is only reachable check-first.
///
/// The second half is the vacuity guard. A counter wedged at its initial value would satisfy the
/// first half for the wrong reason, so a shaped map must drive the same counters ABOVE zero.
#[test]
fn the_efficiency_loops_test_before_they_step() {
    counters::reset();
    let _ = flat_mm(cpg_gas()).match_point(&flight(), 1200.0);
    assert_eq!(counters::hp_passes_min(), 0,
               "the HP efficiency loop took a secant step on a FLAT map, where the residual is \
                met on entry — that is a `do`-while shape, and the oracle's key-vs-key \
                comparison cannot see it if the Python side has it too");
    assert_eq!(counters::lp_passes_min(), 0,
               "the LP efficiency loop took a secant step on a FLAT map — see above");

    // ... and the counters are not simply stuck at zero.
    counters::reset();
    let (_, map_lp, map_hp) = shapes_c().into_iter().find(|(n, _, _)| *n == "tilted").unwrap();
    let _ = mm(cpg_gas(), map_lp, map_hp).match_point(&flight(), 1200.0);
    assert!(counters::hp_passes_max() > 0 && counters::lp_passes_max() > 0,
            "a SHAPED map drove neither efficiency loop off zero ({} / {}) — the counters are \
             not measuring the loop at all, so the flat-map zeros above prove nothing",
            counters::hp_passes_max(), counters::lp_passes_max());
}
