//! **The phase-8 pre-repair: the sonic-throat bracket `assert` is an `Abort` at every layer.**
//!
//! Python's `_sonic_throat` / `_sonic_throat_bisect` raise `AssertionError`, and **48 of the 56**
//! `except AssertionError` blocks in `engine.py` reach them by name (an over-approximating
//! name-level call graph, plan § 8.0); the other 8 call a callback their caller passes in, which
//! a name graph cannot follow, and are treated as reaching. Until this repair Rust's spelling was an `assert!` — a panic that
//! unwound past every `Result<_, Abort>` chain the port built to model those catches. The repair
//! added fallible twins and converted every call site reachable from a `Result` chain.
//!
//! These gates pin each converted LAYER on its own, so a regression is located rather than merely
//! detected: the leaf ([`try_sonic_throat`] on both gas branches), [`try_choked_mfp`], the nozzle
//! ([`Nozzle::try_apply`]'s rung-30 choke), and both turbine-solve hook bodies. The end-to-end cell
//! — a marcher that `break`s on the `Abort` and returns Python's empty trajectory — is
//! `rung46.rs::the_sonic_bracket_assert_is_catchable_and_the_march_returns_pythons_empty_trajectory`.
//!
//! **Every expected MESSAGE was measured on PyPy at the repair**, not transcribed: three Python
//! readers store `str(e)` in their output (`joint_ic_corners` `[:120]`, `demand_law` `[:200]`,
//! `windup_law` `[:240]`), so the Rust text must match byte-for-byte. And the infallible spellings
//! must still PANIC with that same text, which is what the callers no Python `except` sits above
//! rely on.

use std::panic::{catch_unwind, AssertUnwindSafe};
use turbojet::components::{choked_mfp, try_choked_mfp, try_sonic_throat, Nozzle};
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{FlowState, Gas, GasSpec};
use turbojet::matcher::{try_r31_solve_turbine, OffDesignMatcher};
use turbojet::spool::{try_r34_solve_turbine, R34};

const CPG_MSG: &str = "CPG sonic-throat root outside the physical bracket";
const TPG_MSG: &str = "sonic-throat bracket does not straddle M=1";

/// The self-consistent CPG gas `test_rung46.py::_cpg_gas` builds.
fn cpg_gas() -> Gas {
    let (gc, cc, gt, ct) = (1.4f64, 1004.0f64, 1.3f64, 1239.0f64);
    Gas::new(GasSpec {
        gamma_c: gc, cp_c: cc, r_c: (gc - 1.0) / gc * cc,
        gamma_t: gt, cp_t: ct, r_t: (gt - 1.0) / gt * ct,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn matcher_with(hooks: Option<&'static turbojet::matcher::MatcherHooks>) -> OffDesignMatcher {
    let losses = Losses {
        pi_d: 0.97, eta_c: 0.88, eta_b: 0.99, pi_b: 0.96, eta_t: 0.90, eta_m: 0.99, pi_n: 0.98,
        nozzle_convergent: true, ..Losses::default()
    };
    let eng = build_turbojet(cpg_gas(), 10.0, 1500.0, 50_000.0, losses);
    let fl = FlightCondition::new(250.0, 50_000.0, 0.85);
    match hooks {
        None => OffDesignMatcher::new(eng, fl, 1.0),
        Some(h) => OffDesignMatcher::with_hooks(eng, fl, 1.0, h),
    }
}

fn panic_text(r: std::thread::Result<impl Sized>) -> String {
    match r {
        Ok(_) => "<no panic>".into(),
        Err(e) => e.downcast_ref::<String>().cloned()
            .or_else(|| e.downcast_ref::<&str>().map(|s| (*s).to_string()))
            .unwrap_or_else(|| "<non-string panic>".into()),
    }
}

/// The LEAF, on both branches. The CPG closed form refuses on the `Tt` rung 46's march hands it
/// (−3414.48 K) and on a NaN; the TPG/reacting bisection refuses a NaN with ITS OWN message —
/// two asserts, two messages, and Python's two strings are what each branch must carry.
#[test]
fn the_leaf_refuses_with_pythons_message_on_both_gas_branches() {
    let (cpg, tpg) = (cpg_gas(), Gas::thermally_perfect());
    for tt in [-3414.4849172987224, f64::NAN] {
        assert_eq!(try_choked_mfp(&cpg, tt, 0.02).expect_err("CPG must refuse").0, CPG_MSG);
    }
    assert_eq!(try_choked_mfp(&tpg, f64::NAN, 0.02).expect_err("TPG must refuse").0, TPG_MSG);
    assert_eq!(try_sonic_throat(&tpg, f64::NAN, 1e5, 0.02).expect_err("TPG must refuse").0,
               TPG_MSG);
    // The passing path: PyPy's `choked_mfp(cpg, 1000.0, 0.02)` repr, bit-for-bit.
    let v = try_choked_mfp(&cpg, 1000.0, 0.02).expect("a physical Tt passes");
    assert_eq!(v.to_bits(), 0.039461357122401314f64.to_bits());
}

/// The infallible spelling still PANICS, with the same text — the contract every caller with no
/// Python `except` above it relies on (a design capture, rung 31/32/38's `match`).
#[test]
fn the_infallible_wrapper_still_panics_with_the_same_message() {
    let cpg = cpg_gas();
    // No hook suppression: a process-global `set_hook` races the other gates in this binary (the
    // hazard `demand_coordinate.rs` names). The default hook prints one line; no value depends on it.
    let r = catch_unwind(AssertUnwindSafe(|| choked_mfp(&cpg, f64::NAN, 0.02)));
    assert_eq!(panic_text(r), CPG_MSG);
}

/// The NOZZLE layer: rung 30's convergent choke inside [`Nozzle::try_apply`] called the panicking
/// spelling before the repair — inside the very function whose doc says EVERY assert in the body
/// is an `Abort`, because rung 33's bracket march writes a bare `except AssertionError`. PyPy:
/// `Nozzle(p_ambient=5e4, pi_n=0.98, convergent=True).apply(FlowState(nan, 3e5, 1.0, 0.02), cpg)`
/// raises the CPG bracket message, so no earlier check in the body intercepts a NaN first.
#[test]
fn the_convergent_nozzle_returns_the_abort() {
    let st = FlowState { tt: f64::NAN, pt: 3e5, mdot: 1.0, far: 0.02 };
    let e = Nozzle::convergent(5e4, 0.98).try_apply(&st, &cpg_gas())
        .expect_err("the choke solve must refuse a NaN Tt");
    assert_eq!(e.0, CPG_MSG);
}

/// The TURBINE-SOLVE HOOK, both bodies. Each reads `choked_mfp` at station 4 first and again inside
/// its residual; before the repair the hook's type was infallible, so `try_instant_tail` (rung 34)
/// and `try_plenum_state` (rung 37) reached a panic from inside their `Result` chains. PyPy's
/// rung-31 `_solve_turbine(cpg, nan, 0.02)` raises the CPG message; and its passing value at
/// `Tt4 = 1400` is pinned bit-for-bit, so the conversion is shown not to move the arithmetic.
#[test]
fn both_turbine_solve_bodies_return_the_abort_and_their_passing_path_is_unmoved() {
    let (m31, m34) = (matcher_with(None), matcher_with(Some(&R34)));
    let gas = cpg_gas();
    assert_eq!(try_r31_solve_turbine(&m31, &gas, f64::NAN, 0.02, None).expect_err("r31").0, CPG_MSG);
    assert_eq!(try_r34_solve_turbine(&m34, &gas, f64::NAN, 0.02, None).expect_err("r34").0, CPG_MSG);
    // Through the HOOK, so the dispatch itself is on the fallible path.
    assert_eq!(m34.try_solve_turbine(&gas, f64::NAN, 0.02, None).expect_err("hook").0, CPG_MSG);

    let (pi_t, tau_t, tt5) = m31.try_solve_turbine(&gas, 1400.0, 0.02, None).expect("passes");
    assert_eq!((pi_t.to_bits(), tau_t.to_bits(), tt5.to_bits()),
               (0.4278783551930661f64.to_bits(), 0.8398822059717187f64.to_bits(),
                1175.8350883604062f64.to_bits()),
               "rung 31's turbine solve at Tt4=1400 must equal PyPy's repr bit-for-bit");
}
