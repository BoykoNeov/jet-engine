//! Rung 31 — OFF-DESIGN MATCHING: the operating point becomes an OUTPUT.
//!
//! Port of `tests/test_rung31.py` (phase 5 slice I). Gates, named in `docs/rung31-spec.md`
//! § Verification gates:
//!
//! 1. **REDUCE TO DESIGN (the spine)** — the matching solver at the design flight + `Tt4`
//!    reproduces the design run's `pi_c`, stations, `mdot` and thrust. Reduce BY CONSTRUCTION:
//!    `A4`/`A8` are captured from that run, the compressor inverse is the exact inverse of
//!    `Compressor::apply`, and the `f` fixed point starts converged.
//! 2. **THE SOLVER IS RIGHT (non-tautological)** — on a calorically-perfect gas the matching
//!    solve reproduces Mattingly's closed-form referencing: `tau_t`, `pi_t` exactly constant
//!    across the throttle sweep, `pi_c = [1+eta_c(tau_c-1)]^(gc/(gc-1))`, and the
//!    `Tt4/(tau_r T0)` slaving factor constant. Without this, gate 1 exercises only the design
//!    point itself.
//! 3. **CYCLE UNTOUCHED** — the default specified-`pi_c` design path is bit-for-bit rung 6.
//! 4. **THE VERDICT / RUNNING LINE** — throttle down and `pi_c`, `mdot`, thrust fall together;
//!    the nozzle-unchoke `Tt4` bounds the branch and is flagged, not lied about.
//! 5. **THE DRIFT (the finding)** — on the reacting gas `tau_t` is NOT constant along the
//!    sweep while on the CPG gas it is constant to machine precision; the drift is the
//!    variable-`cp` physics, and its kill test separates the `gamma(T)` curve from composition.
//! 6. **DIRECTION** — hotter `Tt4` => higher `pi_c`, `mdot`, thrust.
//!
//! **ONE GATE IS NOT PORTED, AND THAT IS A FINDING RATHER THAN AN OMISSION** — see
//! [`gate3_cycle_untouched`]. It is the FOURTH instance of a pattern the port keeps meeting:
//! the source's test guards something Rust's type system already guarantees, so a faithful
//! transcription would be a green test that measures nothing.

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{powp, Gas, GasSpec};
use turbojet::matcher::{Branch, OffDesignMatcher};

const PI_C: f64 = 10.0;
const TT4: f64 = 1500.0;
const ETA_C: f64 = 0.88;

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

/// The rung-2 real-component losses, with rung 30's convergent nozzle — which rung 31 REQUIRES,
/// because `A8` is the throat area of a convergent nozzle and without one there is no such area.
fn real() -> Losses {
    Losses {
        pi_d: 0.97, eta_c: ETA_C, eta_b: 0.99, pi_b: 0.96,
        eta_t: 0.90, eta_m: 0.99, pi_n: 0.98,
        nozzle_convergent: true,
        ..Losses::default()
    }
}

/// The losses WITHOUT the convergent nozzle — the default design path gate 3 checks.
fn real_plain() -> Losses {
    Losses { nozzle_convergent: false, ..real() }
}

/// A SELF-CONSISTENT CPG dual gas: `R_t = (γ−1)/γ·cp_t` EXACTLY, so the sonic-throat SOLVER
/// equals the closed form and gate 2 compares two code paths rather than one with itself.
///
/// **Deliberately not slice H's `cpg()`**, which rounds `R_t` to 285.9. That rounding is
/// harmless where slice H used it and fatal here.
fn cpg_gas() -> Gas {
    let (g, cp) = (1.3, 1239.0);
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: 286.9,
        gamma_t: g, cp_t: cp, r_t: (g - 1.0) / g * cp,
        hpr: 42.8e6,
        ..GasSpec::default()
    })
}

fn matcher(gas: Gas) -> OffDesignMatcher {
    OffDesignMatcher::new(
        build_turbojet(gas, PI_C, TT4, 50_000.0, real()), flight(), 1.0)
}

// ------------------------------------------------------------------------------- gate 1
/// GATE 1 — matching AT the design condition reproduces the design run.
#[test]
fn gate1_reduce_to_design() {
    let m = matcher(Gas::reacting_equilibrium());
    let od = m.match_point(&flight(), TT4);
    assert!((od.pi_c - PI_C).abs() < 1e-8, "pi_c did not reduce to design: {}", od.pi_c);
    assert!((od.mdot_ratio - 1.0).abs() < 1e-8, "mdot did not reduce: {}", od.mdot_ratio);
    let ref_perf = m.reference.performance.specific_thrust;
    assert!((od.performance.specific_thrust - ref_perf).abs() < 1e-6);
    for k in ["2", "3", "4", "5"] {
        let (a, b) = (od.station(k), m.reference.station(k));
        assert!((a.tt - b.tt).abs() < 1e-6 * b.tt, "station {k} Tt drifted");
        assert!((a.pt - b.pt).abs() < 1e-6 * b.pt, "station {k} pt drifted");
    }
    assert!((od.station("4").far - m.reference.station("4").far).abs() < 1e-9);
    // The design reference itself is the choked-convergent (rung-30) point.
    assert!(od.nozzle_choked && (od.m9 - 1.0).abs() < 1e-6);
}

// ------------------------------------------------------------------------------- gate 2
/// GATE 2 — on a CPG gas the matching solve == Mattingly's closed-form referencing.
///
/// This is what makes gate 1 non-tautological: gate 1 checks the solver at the ONE point its
/// hardware was captured from, where reduce is by construction. Here the solver is compared
/// against textbook algebra at four points it has never seen.
#[test]
fn gate2_cpg_closed_form_referencing() {
    let m = matcher(cpg_gas());
    let gc = (1.4 - 1.0) / 1.4;
    let m0 = flight().m0;
    let tau_r = 1.0 + 0.2 * (m0 * m0);         // the SQUARE is a product; see `lib.rs`'s rule
    let rows: Vec<_> = [1500.0, 1300.0, 1100.0, 1000.0]
        .iter().map(|&t| m.match_point(&flight(), t)).collect();

    // (a) tau_t, pi_t EXACTLY constant — the choked-turbine + choked-nozzle pin, on CPG.
    let (tau_t0, pi_t0) = (rows[0].tau_t, rows[0].pi_t);
    for od in &rows {
        assert!((od.tau_t - tau_t0).abs() < 1e-9,
                "CPG tau_t not constant: {} vs {tau_t0}", od.tau_t);
        assert!((od.pi_t - pi_t0).abs() < 1e-9,
                "CPG pi_t not constant: {} vs {pi_t0}", od.pi_t);
    }

    // (b) the Tt4/(tau_r T0) slaving factor is constant.
    let slave = |od: &turbojet::matcher::OffDesignResult| -> f64 {
        let f = od.station("4").far;
        (od.tau_c - 1.0) / ((1.0 + f) * od.tt4 / (tau_r * flight().t0))
    };
    let s0 = slave(&rows[0]);
    for od in &rows {
        assert!((slave(od) - s0).abs() < 1e-9 * s0, "CPG slaving factor not constant");
    }

    // (c) pi_c == the closed-form compressor map.
    for od in &rows {
        let closed = powp(1.0 + ETA_C * (od.tau_c - 1.0), 1.0 / gc);
        assert!((od.pi_c - closed).abs() < 1e-9 * closed, "pi_c != closed-form compressor map");
    }
}

// ------------------------------------------------------------------------------- gate 3
/// GATE 3 — the default design path is unchanged by rung 31 (bit-for-bit rung 6).
///
/// **HALF OF THE SOURCE'S GATE IS NOT PORTED, and the reason is the point.** Python also
/// asserts that *building a matcher does not perturb a later default run* — a real hazard
/// there, because `OffDesignMatcher.__init__` runs a design cycle on a gas object other code
/// may still hold, and an equilibrium gas carries a frozen station-4 mixture. In Rust
/// `build_turbojet` MOVES the gas and `OffDesignMatcher::new` moves the engine, so no second
/// holder can exist and the transcription would assert something the borrow checker has
/// already made unrepresentable.
///
/// That is the **fourth** instance of one pattern — with rung 16's cached helper, rung 23's
/// `test_helper_matches_production` and rung 22's `TypeError`-on-unknown-field. The rule the
/// port applies: *ask what a ported test could still FAIL for in the new code.* Here, nothing;
/// so what remains is the half that CAN still fail — the absolute rung-6 anchors.
#[test]
fn gate3_cycle_untouched() {
    let r = build_turbojet(Gas::reacting_equilibrium(), PI_C, TT4, 50_000.0, real_plain())
        .run(&flight(), 1.0);
    // Rung-6 anchors: the ideal (fully expanded) design specific thrust ~798, M9 supersonic.
    assert!((r.performance.specific_thrust - 798.37).abs() < 0.5,
            "{}", r.performance.specific_thrust);
    assert!(r.m9 > 1.8 && (r.p9 - 50_000.0).abs() < 1e-6);   // default nozzle: fully expanded

    // The half that still means something: a run AFTER a matcher has been built and used is
    // unchanged. It cannot fail through a shared gas (there is none) — but it can still fail
    // if the matcher left global state behind, which is a claim worth one line.
    let m = matcher(Gas::reacting_equilibrium());
    let _ = m.match_point(&flight(), TT4);
    let r2 = build_turbojet(Gas::reacting_equilibrium(), PI_C, TT4, 50_000.0, real_plain())
        .run(&flight(), 1.0);
    assert!((r2.performance.specific_thrust - r.performance.specific_thrust).abs() < 1e-9);
}

// ----------------------------------------------------------------------------- gate 4/6
/// GATE 4/6 — the running line is monotone; hotter pumps harder; unchoke is flagged.
///
/// `#[ignore]` is the port's spelling of `@pytest.mark.slow`: run it with
/// `cargo test -- --ignored`, exactly as the Python's `-m slow` opts in by typing.
#[test]
#[ignore]
fn gate4_running_line_and_direction() {
    let m = matcher(Gas::reacting_equilibrium());
    let ods: Vec<_> = [1500.0, 1300.0, 1100.0, 900.0]
        .iter().map(|&t| m.match_point(&flight(), t)).collect();
    // Monotone: throttle up => higher pi_c, mdot, thrust (the compressor is slaved to the line).
    for w in ods.windows(2) {
        assert!(w[0].pi_c > w[1].pi_c, "pi_c must fall as Tt4 falls");
        assert!(w[0].mdot_ratio > w[1].mdot_ratio, "mdot must fall as Tt4 falls");
        assert!(w[0].thrust > w[1].thrust, "thrust must fall as Tt4 falls");
    }
    assert!(ods.iter().all(|od| od.nozzle_choked), "all of these are on the choked branch");
    // Throttled far enough the nozzle unchokes — flagged, not lied about.
    let deep = m.match_point(&flight(), 550.0);
    assert!(!deep.nozzle_choked, "deep throttle should unchoke the nozzle (branch boundary)");
}

// ------------------------------------------------------------------------------- gate 5
/// GATE 5 — the reacting `tau_t` DRIFTS along the throttle sweep; CPG holds it constant.
#[test]
fn gate5_tau_t_drift_is_the_finding() {
    let m = matcher(Gas::reacting_equilibrium());
    let hot = m.match_point(&flight(), 1500.0).tau_t;
    let cold = m.match_point(&flight(), 800.0).tau_t;
    let drift = (hot - cold).abs() / hot;
    assert!(drift > 0.02,
            "reacting tau_t should drift >2% over a 2:1 throttle, got {drift:.4}");

    // CPG: constant to machine precision — which isolates the drift as variable-cp physics.
    let mc = matcher(cpg_gas());
    let hot_c = mc.match_point(&flight(), 1500.0).tau_t;
    let cold_c = mc.match_point(&flight(), 800.0).tau_t;
    assert!((hot_c - cold_c).abs() < 1e-9,
            "CPG tau_t must be constant, drift {:.2e}", (hot_c - cold_c).abs());
}

/// GATE 5 (kill test) — the drift's DRIVER is the `gamma_t(T)` curve, not the composition.
///
/// Three-gas ladder, drift measured over the CHOKED branch (1500 vs 800, both choked):
/// CPG (fixed gamma) -> 0; thermally-perfect (variable `cp(T)`, FROZEN composition) -> most of
/// the drift; reacting-equilibrium (variable cp AND composition) -> the full drift. Within one
/// operating point both throats carry the same frozen composition, so `R` cancels in
/// `MFP4/MFP9` and the residual IS a `gamma_t(T)`-curve effect.
#[test]
fn gate5_killtest_gamma_curve() {
    let drift = |gas: Gas| -> f64 {
        let m = matcher(gas);
        let h = m.match_point(&flight(), 1500.0).tau_t;
        let c = m.match_point(&flight(), 800.0).tau_t;      // both choked
        (h - c).abs() / h
    };
    let d_cpg = drift(cpg_gas());
    let d_tpg = drift(Gas::thermally_perfect());            // variable cp(T), frozen composition
    let d_react = drift(Gas::reacting_equilibrium());
    assert!(d_cpg < 1e-9, "CPG must not drift");
    assert!(d_react > 0.02, "reacting drift should exceed 2%");
    // The gamma(T) curve alone carries the MAJORITY of the drift (measured ~81 %).
    assert!(d_tpg > 0.6 * d_react,
            "gamma(T) curve should drive most of the drift: {:.2}", d_tpg / d_react);
    assert!(d_tpg < d_react, "composition adds a (minority) further contribution");
}

// -------------------------------------------------------------------------------- M0 axis
/// The flight-Mach trends: `pi_c` falls and `mdot` rises with `M0` (ram) at fixed `Tt4`.
#[test]
fn m0_ram_lapse() {
    let m = matcher(Gas::reacting_equilibrium());
    let lo = m.match_point(&FlightCondition::new(250.0, 50_000.0, 0.5), TT4);
    let hi = m.match_point(&FlightCondition::new(250.0, 50_000.0, 2.0), TT4);
    assert!(hi.pi_c < lo.pi_c, "pi_c must fall as flight Mach rises (ram raises Tt2)");
    assert!(hi.mdot_ratio > lo.mdot_ratio, "mdot must rise as flight Mach rises (higher pt4)");
}

// ------------------------------------------------------------- what the Python cannot state
/// The (★) pin is PURE GEOMETRY, stated as a count over BIT PATTERNS rather than a tolerance.
///
/// The Python's gate 2(a) asserts `|tau_t - tau_t0| < 1e-9` at four throttles. That is a
/// tolerance on a claim that is actually exact, and it cannot distinguish "the pin holds" from
/// "the pin nearly holds". Here the whole choked CPG sweep — throttle AND flight Mach — must
/// produce ONE bit pattern, and the reacting gas over the same sweep must produce a different
/// value at every point. The pairing is what makes it a measurement: without the second half,
/// a sweep too narrow to move anything would pass the first.
#[test]
fn the_pin_is_exact_not_merely_tight() {
    use std::collections::HashSet;
    let throttles = [1500.0, 1300.0, 1100.0, 1000.0, 900.0];
    let machs = [0.5, 0.85, 1.2];

    let patterns = |gas: Gas| -> HashSet<u64> {
        let m = matcher(gas);
        let mut out = HashSet::new();
        for &m0 in &machs {
            let fl = FlightCondition::new(250.0, 50_000.0, m0);
            for &t in &throttles {
                let od = m.match_point(&fl, t);
                assert_eq!(od.branch, Branch::Choked, "this sweep must stay choked");
                out.insert(od.tau_t.to_bits());
            }
        }
        out
    };
    let n_cells = throttles.len() * machs.len();
    assert_eq!(patterns(cpg_gas()).len(), 1,
               "rung 31: on CPG the choked tau_t is ONE value over all {n_cells} cells");
    assert_eq!(patterns(Gas::reacting_equilibrium()).len(), n_cells,
               "the reacting gas must collapse NOWHERE — else the sweep is the finding");
}
