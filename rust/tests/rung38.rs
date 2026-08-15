//! RUNG 38 — TWO-SPOOL MATCHING: the triangular cascade (no simultaneous solve).
//!
//! Port of `tests/test_rung38.py`, gate for gate. Its six gates:
//!
//!   1. REDUCE — `lp_disabled` IS an `OffDesignMatcher` **by construction** (exact dispatch, not
//!      a knob-to-zero limit): the two-spool machinery is never entered. In Rust it cannot BE
//!      entered — the two variants hold different types — so the gate additionally asserts the
//!      dispatch arm, which is the part Python can only assert about the numbers.
//!   2. NON-TAUTOLOGICAL — an INDEPENDENT bare-math CPG cascade (no `Gas`/`Component`/matcher
//!      calls: its own closed-form stagnation relations, its own bisection, its own fixed point)
//!      reproduces the shipped solver's `(pi_lpc, pi_hpc, tau_hpt, tau_lpt)` across a throttle
//!      sweep; both `tau`s are `Tt4`-INDEPENDENT on CPG, and (the rung-31 gate-5 mirror) DO
//!      drift on the reacting gas over the same window — isolating the CPG constancy as the
//!      gas-model effect rather than as an artefact of the sweep.
//!   3. THE FINDING — no 2×2 solve: each compressor's OWN efficiency is a terminal leaf.
//!      `eta_hpc` moves `pi_hpc` and leaves `pi_lpc` BIT-FOR-BIT unchanged; `eta_lpc` the
//!      mirror. CONTRAST: the two turbine efficiencies move BOTH, so this is not a claim that
//!      the spools do not talk.
//!   4. SCOPE GUARD — throttling into nozzle unchoke raises the documented scope error.
//!   5. PHYSICALITY / DIRECTION — reduces to design at the design point; hotter pumps harder.
//!   6. CYCLE UNTOUCHED — the default single-spool design path is bit-for-bit rung 6.

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{Gas, GasSpec};
use turbojet::matcher::OffDesignMatcher;
use turbojet::two_spool::{build_two_spool_turbojet, Matched, TwoSpoolLosses, TwoSpoolMatcher};

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

/// The single-spool design the `lp_disabled` path is handed — its compressor plays the HPC role.
fn single_design() -> turbojet::engine::Engine {
    build_turbojet(Gas::reacting_equilibrium(), PI_HPC, TT4, 50_000.0, Losses {
        pi_d: 0.97, eta_c: 0.88, eta_b: 0.99, pi_b: 0.96, eta_t: 0.92, eta_m: 0.99,
        pi_n: 0.98, nozzle_convergent: true, ..Losses::default()
    })
}

/// The SELF-CONSISTENT CPG dual gas: `R_t = (γ−1)/γ·cp_t` EXACTLY — rung 31's recipe, and gate 2
/// depends on it, because the bare-math reference below uses the perfect-gas relation directly.
fn cpg_gas() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: 286.9,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn matcher(gas: Gas) -> TwoSpoolMatcher {
    let design = build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, 50_000.0, real());
    TwoSpoolMatcher::new(design, flight(), 1.0)
}

fn reacting_matcher() -> TwoSpoolMatcher {
    matcher(Gas::reacting_equilibrium())
}

// --------------------------------------------------------------------------------- gate 1
#[test]
fn gate1_reduce_lp_disabled_is_offdesign_matcher() {
    let plain = OffDesignMatcher::new(single_design(), flight(), 1.0);
    let degenerate = TwoSpoolMatcher::lp_disabled(single_design(), flight(), 1.0);

    // THE PART PYTHON CANNOT ASSERT: "exact dispatch, not a knob-to-zero limit" is a TYPE fact
    // here. A degenerate matcher holds an `OffDesignMatcher` and no two-spool field, so the
    // cascade is unreachable rather than merely unentered.
    assert!(matches!(degenerate, TwoSpoolMatcher::Degenerate(_)));

    for tt4 in [1500.0, 1300.0, 900.0] {
        let a = plain.match_point(&flight(), tt4);
        let Matched::Single(b) = degenerate.match_point(&flight(), tt4) else {
            panic!("lp_disabled must return a single-spool result");
        };
        assert_eq!(a.pi_c.to_bits(), b.pi_c.to_bits(),
                   "lp_disabled must reproduce OffDesignMatcher BIT-FOR-BIT at Tt4={tt4}");
        assert_eq!(a.mdot_air.to_bits(), b.mdot_air.to_bits());
        assert_eq!(a.thrust.to_bits(), b.thrust.to_bits());
        assert_eq!(a.tau_t.to_bits(), b.tau_t.to_bits());
    }
}

// --------------------------------------------------------------------------------- gate 2
/// The bare-math CPG reference: closed-form stagnation, its own bisection, its own `f` fixed
/// point. **No `Gas`, `Component` or matcher call appears inside it** — that is what makes gate 2
/// non-tautological rather than a re-run of the code under test.
fn bare_cascade(m: &turbojet::two_spool::TwoSpoolCore, gas: &Gas, tt4: f64)
    -> (f64, f64, f64, f64) {
    let (gamma_c, gamma_t) = (1.4, 1.3);
    let (cp_c, cp_t, hpr) = (1004.0, 1239.0, 42.8e6);
    let gc = (gamma_c - 1.0) / gamma_c;
    let gt = (gamma_t - 1.0) / gamma_t;
    let (eta_lpc, eta_hpc, eta_hpt, eta_lpt) = (0.90, 0.88, 0.92, 0.90);
    let (eta_m, eta_b, pi_n) = (0.99, 0.99, 0.98);
    let _ = gas;

    let stag = 1.0 + 0.5 * (gamma_c - 1.0) * 0.85f64.powi(2);
    let tt2 = 250.0 * stag;

    // pi_t/sqrt(tau_t) = area_ratio, tau_t = 1 - eta_t*(1 - pi_t^gt): the CPG closed form,
    // because MFP* is Tt-INDEPENDENT for a CPG gas, so the area ratio ALONE is the target.
    let bisect = |area_ratio: f64, eta_t: f64| -> (f64, f64) {
        let tau_of = |pi_t: f64| 1.0 - eta_t * (1.0 - pi_t.powf(gt));
        let resid = |pi_t: f64| pi_t / tau_of(pi_t).sqrt() - area_ratio;
        let (mut lo, mut hi) = (0.02, 0.999);
        let (mut flo, fhi) = (resid(lo), resid(hi));
        assert!(flo < 0.0 && 0.0 < fhi);
        for _ in 0..200 {
            let mid = 0.5 * (lo + hi);
            let fm = resid(mid);
            if flo * fm <= 0.0 { hi = mid; } else { lo = mid; flo = fm; }
            if hi - lo < 1e-14 { break; }
        }
        let pi_t = 0.5 * (lo + hi);
        (pi_t, tau_of(pi_t))
    };

    let (_, tau_hpt) = bisect(m.a4 / m.a45, eta_hpt);
    let (_, tau_lpt) = bisect(m.a45 / (m.a8 * pi_n), eta_lpt);
    let tt45 = tt4 * tau_hpt;
    let tt5 = tt45 * tau_lpt;

    let (mut f, mut pi_lpc, mut pi_hpc) = (m.f_design, f64::NAN, f64::NAN);
    for _ in 0..60 {
        let dh_lpt = eta_m * (1.0 + f) * cp_t * (tt45 - tt5);
        let tt25 = tt2 + dh_lpt / cp_c;
        let tt25s = tt2 + eta_lpc * (tt25 - tt2);
        pi_lpc = (tt25s / tt2).powf(1.0 / gc);

        let dh_hpt = eta_m * (1.0 + f) * cp_t * (tt4 - tt45);
        let tt3 = tt25 + dh_hpt / cp_c;
        let tt3s = tt25 + eta_hpc * (tt3 - tt25);
        pi_hpc = (tt3s / tt25).powf(1.0 / gc);

        let h4 = cp_t * tt4;
        let f_new = (h4 - cp_c * tt3) / (eta_b * hpr - h4);
        if (f_new - f).abs() < 1e-14 {
            break;
        }
        f = f_new;
    }
    (pi_lpc, pi_hpc, tau_hpt, tau_lpt)
}

#[test]
fn gate2_cpg_independent_cascade() {
    let gas = cpg_gas();
    let m = matcher(cpg_gas());
    let c = m.core();

    for tt4 in [1500.0, 1300.0, 1100.0, 1000.0] {
        let (pi_lpc_ref, pi_hpc_ref, tau_hpt_ref, tau_lpt_ref) = bare_cascade(c, &gas, tt4);
        let od = m.match_point(&flight(), tt4).two();
        assert!((od.pi_lpc - pi_lpc_ref).abs() < 1e-8 * pi_lpc_ref,
                "pi_lpc {} vs bare-math {pi_lpc_ref}", od.pi_lpc);
        assert!((od.pi_hpc - pi_hpc_ref).abs() < 1e-8 * pi_hpc_ref,
                "pi_hpc {} vs bare-math {pi_hpc_ref}", od.pi_hpc);
        assert!((od.tau_hpt - tau_hpt_ref).abs() < 1e-9);
        assert!((od.tau_lpt - tau_lpt_ref).abs() < 1e-9);
    }

    // Both tau's are Tt4-INDEPENDENT on CPG — the MFP-constant structural fact.
    let rows: Vec<_> = [1500.0, 1300.0, 1100.0, 1000.0].iter()
        .map(|&t| m.match_point(&flight(), t).two()).collect();
    for od in &rows {
        assert!((od.tau_hpt - rows[0].tau_hpt).abs() < 1e-9);
        assert!((od.tau_lpt - rows[0].tau_lpt).abs() < 1e-9);
    }

    // THE MIRROR (rung 31's gate 5, doubled): the reacting gas DOES drift over the SAME choked
    // window, which is what isolates the constancy above as a gas-model statement.
    let mr = reacting_matcher();
    let hot = mr.match_point(&flight(), 1500.0).two();
    let cold = mr.match_point(&flight(), 650.0).two();
    let drift_hpt = (hot.tau_hpt - cold.tau_hpt).abs() / hot.tau_hpt;
    let drift_lpt = (hot.tau_lpt - cold.tau_lpt).abs() / hot.tau_lpt;
    assert!(drift_hpt > 0.02, "reacting tau_HPT should drift >2%: {drift_hpt:.4}");
    assert!(drift_lpt > 0.01, "reacting tau_LPT should drift >1%: {drift_lpt:.4}");
}

// --------------------------------------------------------------------------------- gate 3
#[test]
fn gate3_triangularity_is_the_finding() {
    let mut m = reacting_matcher();
    let (state0, _) = m.core().freestream_for(&flight());
    let (tt2, pt2) = (state0.tt, m.core().pi_d_max * state0.pt);
    let f = 0.02;
    let pt4 = m.core().pi_b * m.core().pi_hpc_design * m.core().pi_lpc_design * pt2;
    // `working_gas` returns `Some` here because the gas is the equilibrium one, and it is
    // UNWRAPPED rather than cloned: a `Gas` clone does NOT carry the frozen station-4 mixture,
    // so `owned.clone()` would hand the cascade an unfrozen gas and panic in the hot section.
    // Shipped code never hits this because it uses `owned.as_ref()`; a test that needs the gas
    // to outlive a `&mut` borrow of the matcher is the one place the difference shows.
    let wgas = m.core().working_gas(f, TT4, pt4)
        .expect("the reacting-equilibrium gas yields an owned working gas");

    let base = m.core().cascade(&wgas, tt2, TT4, f);

    // eta_hpc: step 4's OWN pressure-inversion leaf. It must not reach pi_lpc, nor Tt25.
    let keep = m.core().eta_hpc;
    m.core_mut().eta_hpc = 0.55;
    let c = m.core().cascade(&wgas, tt2, TT4, f);
    assert_eq!(c.pi_lpc.to_bits(), base.pi_lpc.to_bits(),
               "pi_lpc must be BIT-FOR-BIT unchanged by eta_hpc");
    assert_eq!(c.tt25.to_bits(), base.tt25.to_bits());
    assert_ne!(c.pi_hpc.to_bits(), base.pi_hpc.to_bits(),
               "pi_hpc SHOULD move with its own eta_hpc");
    m.core_mut().eta_hpc = keep;

    // eta_lpc: step 3's OWN leaf. A dead end for the HP spool.
    let keep = m.core().eta_lpc;
    m.core_mut().eta_lpc = 0.55;
    let c = m.core().cascade(&wgas, tt2, TT4, f);
    assert_ne!(c.pi_lpc.to_bits(), base.pi_lpc.to_bits(),
               "pi_lpc SHOULD move with its own eta_lpc");
    assert_eq!(c.pi_hpc.to_bits(), base.pi_hpc.to_bits(),
               "pi_hpc must be BIT-FOR-BIT unchanged by eta_lpc — a dead end for the HP spool");
    m.core_mut().eta_lpc = keep;

    // THE CONTRAST: the turbine efficiencies are ENERGY-path parameters — they shape Tt45/Tt5,
    // so they legitimately move BOTH ratios. This is why "no 2x2 solve" is not "the spools
    // don't talk", and without it gate 3 would be quotable as the stronger, false claim.
    let keep = m.core().eta_hpt;
    m.core_mut().eta_hpt = 0.70;
    let c = m.core().cascade(&wgas, tt2, TT4, f);
    assert_ne!(c.pi_lpc.to_bits(), base.pi_lpc.to_bits(),
               "eta_hpt SHOULD move pi_lpc via Tt45");
    assert_ne!(c.pi_hpc.to_bits(), base.pi_hpc.to_bits());
    m.core_mut().eta_hpt = keep;

    let keep = m.core().eta_lpt;
    m.core_mut().eta_lpt = 0.70;
    let c = m.core().cascade(&wgas, tt2, TT4, f);
    assert_ne!(c.pi_lpc.to_bits(), base.pi_lpc.to_bits());
    assert_ne!(c.pi_hpc.to_bits(), base.pi_hpc.to_bits(),
               "eta_lpt SHOULD move pi_hpc via Tt25");
    m.core_mut().eta_lpt = keep;
}

/// **THE STRUCTURAL HALF OF GATE 3, WHICH ONLY THE PORT CAN STATE.** Rung 39's `hp_eta_loop` is
/// a FREE function; `eta_lpc` and `pi_lpc` are not among its parameters, so the closure of the
/// HP leaf is a scope fact and not a numerical one. This test exists to make the claim FAIL
/// LOUDLY if someone later widens that signature to take `&TwoSpoolMapCore` for convenience:
/// the call below compiles only while the parameter list is scalars.
#[test]
fn gate3_structural_the_hp_leaf_takes_no_lp_quantity() {
    let m = matcher(cpg_gas());
    let c = m.core();
    let (state0, _) = c.freestream_for(&flight());
    let (tt2, pt2) = (state0.tt, c.pi_d_max * state0.pt);
    let f = 0.02;
    let pt4 = c.pi_b * c.pi_hpc_design * c.pi_lpc_design * pt2;
    let owned = c.working_gas(f, TT4, pt4);
    let wgas = owned.unwrap_or_else(|| c.gas().clone());
    let cc = c.cascade(&wgas, tt2, TT4, f);
    let mfp4 = turbojet::components::choked_mfp(&wgas, TT4, f);

    // Every argument is either a gas, a geometry constant, a burner constant, or an HP-side
    // number. There is no LP efficiency and no LP pressure ratio to pass.
    let hp = turbojet::two_spool::hp_eta_loop_closed(
        &wgas, TT4, f, cc.tt25, cc.tt3, mfp4, &turbojet::map::ComponentMap::flat(),
        c.eta_hpc, c.a4, c.pi_b,
        // `powp(tt, 0.5)`, NOT `tt.sqrt()` — the distinction slice I pre-registered as its P4
        // because a tolerance hides it. Nothing here would notice the difference, which is
        // exactly why the wrong spelling must not sit in new gated code for slice L to copy.
        turbojet::gas::powp(m.core().reference.station("25").tt, 0.5)
            / m.core().reference.station("25").pt,
        cc.tt3 / cc.tt25);
    // `.expect` and not `.unwrap()`: slice M made this loop fallible (rung 54's `_scan` catches
    // its `solve_n` bracket), and this gate is about the SIGNATURE — an `Err` here would mean the
    // HP leaf stopped closing on a flat map, which is a different failure from the one the
    // assertion below describes.
    let hp = hp.expect("the HP leaf closes on a flat map");
    assert!(hp.eta.is_finite() && hp.pi > 1.0);
}

// --------------------------------------------------------------------------------- gate 4
#[test]
#[should_panic(expected = "OUT OF SCOPE")]
fn gate4_nozzle_unchoke_is_out_of_scope() {
    reacting_matcher().match_point(&flight(), 600.0);
}

// --------------------------------------------------------------------------------- gate 5
#[test]
fn gate5_physicality_and_direction() {
    let design = build_two_spool_turbojet(Gas::reacting_equilibrium(), PI_LPC, PI_HPC, TT4,
                                          50_000.0, real());
    let reference = design.run(&flight(), 1.0);
    let m = reacting_matcher();

    let od = m.match_point(&flight(), TT4).two();
    assert!((od.pi_lpc - PI_LPC).abs() < 1e-6, "pi_lpc did not reduce to design: {}", od.pi_lpc);
    assert!((od.pi_hpc - PI_HPC).abs() < 1e-6, "pi_hpc did not reduce to design: {}", od.pi_hpc);
    assert!((od.mdot_ratio - 1.0).abs() < 1e-6);
    assert!((od.performance.specific_thrust - reference.performance.specific_thrust).abs()
                < 1e-4);

    let hot = m.match_point(&flight(), 1500.0).two();
    let cold = m.match_point(&flight(), 1100.0).two();
    assert!(hot.pi_lpc > cold.pi_lpc && hot.pi_hpc > cold.pi_hpc);
    assert!(hot.mdot_ratio > cold.mdot_ratio);
    assert!(hot.thrust > cold.thrust);
    for od in [&hot, &cold] {
        assert!(od.pi_lpc > 1.0 && od.pi_hpc > 1.0);
        assert!(0.0 < od.tau_hpt && od.tau_hpt < 1.0);
        assert!(0.0 < od.tau_lpt && od.tau_lpt < 1.0);
        assert!(od.station("4").pt > od.station("25").pt);
        assert!(od.station("25").pt > od.station("2").pt);
    }
}

// --------------------------------------------------------------------------------- gate 6
#[test]
fn gate6_cycle_untouched() {
    let plain = || build_turbojet(Gas::reacting_equilibrium(), 10.0, TT4, 50_000.0, Losses {
        pi_d: 0.97, eta_c: 0.88, eta_b: 0.99, pi_b: 0.96, eta_t: 0.90, eta_m: 0.99,
        pi_n: 0.98, ..Losses::default()
    });
    let r = plain().run(&flight(), 1.0);
    assert!((r.performance.specific_thrust - 798.37).abs() < 0.5,
            "{}", r.performance.specific_thrust);
    assert!(r.m9 > 1.8 && (r.p9 - 50_000.0).abs() < 1e-6);

    // Building a two-spool design AND matching on it must not perturb the single-spool default.
    let _ = matcher(Gas::reacting_equilibrium());
    let r2 = plain().run(&flight(), 1.0);
    assert!((r2.performance.specific_thrust - r.performance.specific_thrust).abs() < 1e-9);
}

// --------------------------------------------------------------------------------- envelope
/// **THE `M0 = 0` COLUMN IS EXCLUDED BY A SOLVER ROUND-TRIP, NOT BY PHYSICS** — § 5.7 (f), and
/// slice I's grid never sampled it.
///
/// At `M0 = 0` the freestream's two-clause assert `Tt0 >= T0 && pt0 >= p0` fails on its FIRST
/// clause for the integral gases and passes entirely on the closed-form one, because
/// `t_from_h_c(h_c(250))` returns 249.999999999999943 — three ulps low — while the pressure
/// clause is exact. This is gated because a Rust round-trip landing on the OTHER side of
/// exactness would move an envelope boundary that no value comparison can see: the cell would
/// simply start matching.
#[test]
fn envelope_m0_zero_is_a_round_trip_not_a_physics_boundary() {
    for (name, gas) in [("tpg", Gas::thermally_perfect()),
                        ("eq", Gas::reacting_equilibrium())] {
        let round = gas.t_from_h_c(gas.h_c(250.0));
        assert!(round < 250.0, "{name}: the round-trip is exact, so the M0=0 envelope moved: \
                                {round:.17e}");
        assert!((round - 250.0).abs() < 1e-12, "{name}: {round:.17e} is not a last-bits miss");
        assert_eq!((gas.pr_c(round) / gas.pr_c(250.0)).to_bits(), 1.0f64.to_bits(),
                   "{name}: the PRESSURE clause is the exact one — if it stops being exact the \
                    abort's REASON changes even though the cell still aborts");
    }
    // The closed-form gas round-trips exactly, which is why its M0=0 cells get past the ram
    // check and abort later, in `score`.
    let cpg = cpg_gas();
    assert_eq!(cpg.t_from_h_c(cpg.h_c(250.0)).to_bits(), 250.0f64.to_bits());

    // And the consequence, at the matcher: two different abort messages, split by gas.
    let hush = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let at_rest = FlightCondition::new(250.0, 50_000.0, 0.0);
    let msg = |g: Gas| -> String {
        let m = matcher(g);
        let e = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
            || m.match_point(&at_rest, 1500.0))).unwrap_err();
        e.downcast_ref::<String>().cloned()
            .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default()
    };
    let (tpg, cpg_msg) = (msg(Gas::thermally_perfect()), msg(cpg_gas()));
    std::panic::set_hook(hush);
    assert!(tpg.contains("ram must not"), "tpg at M0=0 should abort on the ram clause: {tpg}");
    assert!(cpg_msg.contains("efficiency cascade"),
            "cpg at M0=0 gets PAST the ram clause and aborts in score: {cpg_msg}");
}

/// The `(★)` bisection's cost, gated at the noun the instrument reads (§ 5.7 (d), P3).
///
/// `ceil(log2(0.979 / 1e-13)) = 44` iterations; **+ 2** bracket-endpoint evaluations = 46
/// residual calls; **+ 1** more `tau_of` after the loop = **47**, which is what `tau_calls`
/// counts. § 5.6's P2 was corrected one slice ago for naming the wrong one of these three.
#[test]
fn the_star_bisection_costs_47_tau_of_calls_per_solve() {
    let m = matcher(cpg_gas());
    let c = m.core();
    c.tau_calls.set(0);
    turbojet::two_spool::counters::reset();
    let _ = m.match_point(&flight(), 1200.0).two();
    let solves = 2 * turbojet::two_spool::counters::cascade_calls();   // (★-HP) and (★-LP)
    let total = c.tau_calls.get();
    assert!(solves > 0);
    assert_eq!(total % solves, 0,
               "the cost SPREAD: {total} tau_of calls over {solves} solves");
    assert_eq!(total / solves, 47,
               "47 = 44 bisection iterations + 2 bracket endpoints + 1 final tau_of");
}
