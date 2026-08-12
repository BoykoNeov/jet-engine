//! RUNG 3 — variable cp(T), the thermally-perfect gas.
//!
//! Ported from `tests/test_variable_cp.py`. Six gates
//! (`docs/rung3-variable-cp.md` § Verification gates), in priority order:
//!
//! 1. REDUCE-TO-IDEAL (load-bearing) — a CPG `Gas::default()` reproduces the rung-1/2/2b tables
//!    TO THE DIGIT. Owned by the EXISTING suites (`rung1.rs`, `rung2.rs`, `rung2b.rs`), which
//!    stay green untouched; a one-line guard here documents it.
//! 2. ROUND-TRIP INVERSES — `T_from_h(h(T)) == T` and `T_from_pr(pr(T)) == T` to ~1e-9, plus
//!    monotonicity of h and pr across the working range (a standing assert, too).
//! 3. DISCRIMINATING CPG-VS-INTEGRAL CHECK, run DUAL-SECTION — two distinct flat-cp polynomials
//!    through the integral path reproduce the rung-2 dual-cp turbojet to ~3e-4 (NOT 1e-9):
//!    proof the integral path is genuinely `pr = exp(phi/R)` AND that cold/hot are not confused
//!    (a routing bug would blow the gap wide open).
//! 4. AIR-TABLE ISENTROPIC ANCHOR — isentropic compression of air, pi = 10 from 300 K, lands at
//!    the gas-table ~574 K (vs the calorically-perfect 579 K).
//! 5. EXTERNAL MACHINERY ANCHORS (sourced, `docs/plans/rung3-anchor-cengel.md`) — Cengel 9-89
//!    (T2s, T4s, cycle eta_th) and Mattingly Ex 2.7/2.8 (compression, nozzle), to ~0.15 %.
//!    Topology caveat: Cengel is a POWER cycle, so these anchor the property + process
//!    MACHINERY, tested directly on the gas, not `build_turbojet`.
//! 6. DIRECTIONAL / GAS-TABLE EFFECT — TPG losses move thrust/TSFC the right way, and TPG
//!    compression lands COOLER than CPG at the same design point (cp rises with T).

use turbojet::components::ram_recovery;
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{powp, Gas, GasSpec};

fn close(actual: f64, expected: f64) -> bool { close_rel(actual, expected, 1.5e-3) }

fn close_rel(actual: f64, expected: f64, rel: f64) -> bool {
    (actual - expected).abs() <= rel * expected.abs()
}

/// A constant-cp polynomial (`A_low == A_high == cp/R`): a TPG section whose cp(T) happens to be
/// flat, used to exercise the integral path against a known answer.
fn flat(cp: f64, r: f64) -> ([f64; 5], [f64; 5]) {
    ([cp / r, 0.0, 0.0, 0.0, 0.0], [cp / r, 0.0, 0.0, 0.0, 0.0])
}

fn flight_r1() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }
fn flight_matt() -> FlightCondition { FlightCondition::new(216.7, 50_000.0, 2.0) }

fn matt_common() -> Losses {
    Losses {
        pi_d: 0.95 * ram_recovery(2.0), eta_c: 0.8641, eta_b: 0.98, pi_b: 0.94,
        eta_t: 0.9099, eta_m: 0.99, pi_n: 0.96, p_exit: Some(50_000.0 / 0.5),
        ..Losses::default()
    }
}

// --- Gate 1: reduce-to-ideal still holds (owned by the other suites) -----------------------

/// A CPG gas through the rung-3 code path still reproduces the rung-1 table.
///
/// The full gate lives in `rung1.rs`/`rung2.rs`/`rung2b.rs` (untouched); this is a fast guard
/// that the rung-3 machinery did not disturb the CPG branch.
#[test]
fn reduce_to_ideal_guard() {
    let r = build_turbojet(Gas::default(), 10.0, 1500.0, flight_r1().p0, Losses::default())
        .run(&flight_r1(), 1.0);
    assert!(close(r.station("3").tt, 552.4));
    assert!(close(r.station("5").tt, 1239.7));
    assert!(close(r.performance.specific_thrust, 816.6));
    assert!(close(r.m9, 2.033));
}

// --- Gate 2: round-trip inverses + monotonicity --------------------------------------------

/// `T_from_h(h(T)) == T` and `T_from_pr(pr(T)) == T` to ~1e-9; h and pr strictly increasing.
#[test]
fn roundtrip_inverses_and_monotonicity() {
    let g = Gas::thermally_perfect();
    let cold_ts = [200.0, 250.0, 300.0, 500.0, 800.0, 1000.0, 1240.0, 1300.0];
    let hot_ts = [800.0, 1000.0, 1240.0, 1500.0, 1800.0, 2000.0];

    for t in cold_ts {
        assert!(close_rel(g.t_from_h_c(g.h_c(t)), t, 1e-9), "cold h round-trip at {t}");
        assert!(close_rel(g.t_from_pr_c(g.pr_c(t)), t, 1e-9), "cold pr round-trip at {t}");
    }
    for t in hot_ts {
        assert!(close_rel(g.t_from_h_t(g.h_t(t, 0.0), 0.0), t, 1e-9), "hot h round-trip at {t}");
        assert!(close_rel(g.t_from_pr_t(g.pr_t(t, 0.0), 0.0), t, 1e-9), "hot pr round-trip at {t}");
    }

    // Monotonicity (cp > 0 => h, pr strictly increasing): the well-posedness the inverses rely
    // on, checked across the join at 1000 K.
    for w in cold_ts.windows(2) {
        assert!(g.h_c(w[1]) > g.h_c(w[0]) && g.pr_c(w[1]) > g.pr_c(w[0]),
                "cold not monotone on [{},{}]", w[0], w[1]);
    }
    for w in hot_ts.windows(2) {
        assert!(g.h_t(w[1], 0.0) > g.h_t(w[0], 0.0) && g.pr_t(w[1], 0.0) > g.pr_t(w[0], 0.0),
                "hot not monotone on [{},{}]", w[0], w[1]);
    }
}

// --- Gate 3: dual-section discriminating CPG-vs-integral check -----------------------------

/// Flat-cp polynomials through the integral path reproduce the CPG turbojet to ~3e-4 — NOT
/// 1e-9, and NOT wildly off.
///
/// The two sections carry DISTINCT flats (cold 1004/286.9, hot 1239/285.9 — the rung-2 dual
/// gas), so a routing bug that called `pr_c` where `pr_t` belongs would swap a 0.286 exponent
/// for a 0.231 one and blow the gap to tens of percent. The measured gaps (Tt3 ~2e-4, F/m
/// ~1.4e-4) prove BOTH: the integral path genuinely uses `R/cp(T)` (gap > 1e-9), and cold/hot
/// are routed correctly (gap << 1 %).
#[test]
fn discriminating_dual_section_integral_path() {
    let cpg = Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: 286.9, gamma_t: 1.3, cp_t: 1239.0, r_t: 285.9,
        hpr: 42.8e6, ..GasSpec::default()
    });
    let flat_tpg = Gas::new(GasSpec {
        r_c: 286.9, r_t: 285.9, hpr: 42.8e6,
        cp_c_coeffs: Some(flat(1004.0, 286.9)), cp_t_coeffs: Some(flat(1239.0, 285.9)),
        ..GasSpec::default()
    });

    let rc = build_turbojet(cpg, 10.0, 1800.0, flight_matt().p0, matt_common())
        .run(&flight_matt(), 1.0);
    let rt = build_turbojet(flat_tpg, 10.0, 1800.0, flight_matt().p0, matt_common())
        .run(&flight_matt(), 1.0);

    let gap_tt3 = (rt.station("3").tt - rc.station("3").tt).abs() / rc.station("3").tt;
    let gap_f = (rt.performance.specific_thrust - rc.performance.specific_thrust).abs()
        / rc.performance.specific_thrust;

    // The integral path is genuinely R/cp-based: the gap must EXIST, else it secretly uses
    // (gamma-1)/gamma and would match to 1e-9.
    assert!(gap_tt3 > 3e-5, "Tt3 gap {gap_tt3:.1e} too small — integral path not exercised?");
    // ...but the routing is correct and the gap is the small rounded-R residual.
    assert!(gap_tt3 < 1e-3, "Tt3 gap {gap_tt3:.1e} too large — cold/hot section confusion?");
    assert!(gap_f < 1e-3, "F/m gap {gap_f:.1e} too large — section confusion?");
}

// --- Gate 4: air-table isentropic anchor (~574 K) ------------------------------------------

/// Isentropic compression of air, pi = 10 from 300 K, lands at the gas-table ~574 K.
///
/// Datum-independent (a pr ratio), so immune to the table's enthalpy/entropy datum. The
/// calorically-perfect answer is `300*10^0.2857 = 579.2 K` — the ~5 K shortfall IS the
/// variable-cp effect (cp rises with T, so less temperature for the same work).
#[test]
fn air_table_isentropic_anchor() {
    let g = Gas::thermally_perfect();
    let t2 = g.t_from_pr_c(g.pr_c(300.0) * 10.0);
    assert!(close_rel(t2, 574.1, 2e-3), "air-table T2: {t2}");        // gas table ~574.1
    // `10.0 ** 0.2857` is a libm pow in Python, so it is `powp` here — see gas.rs's rule.
    assert!(t2 < 300.0 * powp(10.0, 0.2857), "variable cp must land below the CPG 579 K");
}

// --- Gate 5: external machinery anchors (Cengel 9-89, Mattingly 2.7/2.8) -------------------

/// Cengel 9-89 (air, Table A-17): the rung-3 station-3/5 substate machinery.
///
/// A POWER cycle (the turbine expands the full pi), so per the topology caveat this is tested
/// on the GAS directly, not through `build_turbojet`. `T2s`/`T4s` are the exact rung-3
/// compressor/turbine substate equations on a single air section; the cycle `eta_th` is the
/// same h-difference energetics (`eta_c = 0.83`, `eta_t = 0.87`, `r_p = 10`).
#[test]
fn cengel_9_89_machinery() {
    let g = Gas::thermally_perfect();
    let t2s = g.t_from_pr_c(g.pr_c(295.0) * 10.0);
    let t4s = g.t_from_pr_c(g.pr_c(1240.0) / 10.0);
    assert!(close(t2s, 564.9), "Cengel T2s: {t2s}");
    assert!(close(t4s, 689.6), "Cengel T4s: {t4s}");

    let (eta_c, eta_t) = (0.83, 0.87);
    let (h1, h3) = (g.h_c(295.0), g.h_c(1240.0));
    let h2 = h1 + (g.h_c(t2s) - h1) / eta_c;
    let h4 = h3 - eta_t * (h3 - g.h_c(t4s));
    let eta_th = ((h3 - h4) - (h2 - h1)) / (h3 - h2);
    assert!(close_rel(eta_th, 0.3013, 2e-3), "Cengel cycle eta_th: {eta_th}");
}

/// Mattingly's OWN variable-cp examples (his Eq 2.53-2.58 gas-table method).
///
/// 2.7 = isentropic compression (the station-3 substate). 2.8 = an isentropic nozzle: it
/// exercises the station-9 pair TOGETHER — V2 from the ENTHALPY split and P2/P1 from the pr
/// ratio — and varies gamma over a wide range, covering the M9 blind spot that the flat-cp
/// gate 3 cannot see.
#[test]
fn mattingly_2_7_2_8_machinery() {
    let g = Gas::thermally_perfect();
    // Ex 2.7: 293.15 K, pi = 15 -> 627.57 K.
    assert!(close(g.t_from_pr_c(g.pr_c(293.15) * 15.0), 627.57), "Mattingly 2.7");

    // Ex 2.8: 3000 R, dh = 179.74 Btu/lbm -> 2377.7 R, P2/P1 = 0.3757.
    let (btu_lbm, r_to_k) = (2326.0, 1.0 / 1.8);
    let t1 = 3000.0 * r_to_k;
    let t2 = g.t_from_h_c(g.h_c(t1) - 179.74 * btu_lbm);
    let p_ratio = g.pr_c(t2) / g.pr_c(t1);                       // = P2/P1 (a pr ratio)
    assert!(close(t2 / r_to_k, 2377.7), "Mattingly 2.8 T2: {} R", t2 / r_to_k);
    assert!(close_rel(p_ratio, 0.3757, 2e-3), "Mattingly 2.8 P2/P1: {p_ratio}");
}

// --- Gate 6: directional + the gas-table effect --------------------------------------------

/// Losses move thrust/TSFC the right way on the TPG gas; and TPG compression lands COOLER than
/// CPG at the same design point (the gas-table effect).
#[test]
fn tpg_directional_and_gas_table_effect() {
    let ideal = build_turbojet(Gas::thermally_perfect(), 10.0, 1500.0, flight_r1().p0,
                               Losses::default()).run(&flight_r1(), 1.0);
    let lossy = build_turbojet(Gas::thermally_perfect(), 10.0, 1500.0, flight_r1().p0, Losses {
        pi_d: 0.95, eta_c: 0.88, eta_b: 0.99, pi_b: 0.95, eta_t: 0.90, eta_m: 0.99, pi_n: 0.98,
        ..Losses::default()
    }).run(&flight_r1(), 1.0);
    assert!(lossy.performance.specific_thrust < ideal.performance.specific_thrust,
            "losses cut thrust");
    assert!(lossy.performance.tsfc > ideal.performance.tsfc, "losses raise TSFC");

    // Gas-table effect: at the same pi_c, variable-cp compression reaches a LOWER Tt3 than
    // constant-cp — cp climbs with T, so the same pressure work is a smaller temperature rise.
    let cpg = build_turbojet(Gas::default(), 10.0, 1500.0, flight_r1().p0, Losses::default())
        .run(&flight_r1(), 1.0);
    assert!(ideal.station("3").tt < cpg.station("3").tt, "TPG compression must land cooler");
}
