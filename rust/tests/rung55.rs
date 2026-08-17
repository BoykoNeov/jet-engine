//! RUNG 55 — THE STAGE STACK: a POSITIONAL lever buys relief from the part it does not move.
//!
//! Port of `tests/test_rung55.py`, gate for gate. Its ten gate groups:
//!
//!   1. REDUCE — an IDENTITY at `K = 1`: no stack object is built, both efficiency loops are the
//!      INHERITED ones, so every matched field is bit-identical to rung 53/54's at a MOVED
//!      stator and on both gases. `StageStack::solve_n` itself dispatches at `K = 1`.
//!   2. THE DERIVED KINEMATICS — `phi_1` IS the face `phi = m/n` exactly (so rungs 36–53 were
//!      reading the front stage all along), and the design ladder is exact for every `K` and
//!      every split (the stack does NOT re-design the engine). Plus the free rung-2b check.
//!   3. THE NON-TAUTOLOGY GATE — the marched stack does DIFFERENT work than the lumped law at
//!      the same `(m, n)`: exactly `0.0` at `K = 1`, non-zero and deepening beyond it.
//!   4. P1 — the RUNNING LINE MOVES: `n` RISES and `phi` FALLS, monotonically with throttle
//!      depth, on every shape; thrust and `pi_c` barely move (paid in SHAFT SPEED).
//!   5. P4 — one machine, two opposite failures: the LP FRONT stage is the worst incidence in
//!      the machine while the HP REAR stage runs ABOVE design `phi`.
//!   6. P5 — `K` is a RESOLUTION: the shift grows with `K` but its increments SHRINK.
//!   7. P6 — the disclosed WORK SPLIT does not carry any verdict.
//!   8. SCOPE, ASSERTED — **OWED TO PHASE 6**, see [`slice_n_deferrals`] item 6.
//!   9. P3 — THE HEADLINE: the front-row lever's cost FACTORISES, and the row count has an
//!      INTERIOR optimum.
//!  10. CYCLE UNTOUCHED.
//!
//! Python marks five of these `@pytest.mark.slow`; the markers are not carried over, for the
//! reason `rung53.rs`'s header gives — the marker records a COST that did not survive the port.
//!
//! # The two places this file is NOT a transcription
//!
//! * **`test_capacity_style_guards_reject_nonsense` SPLITS**, because `#[should_panic]` is
//!   per-test and one of its three refusals is unrepresentable in Rust. Booked in
//!   [`slice_n_deferrals`], so the name diff against Python reads as a factorisation and not as
//!   a gap.
//! * **`test_p3_row_count_has_an_interior_optimum`'s cost clause is STRENGTHENED**, because the
//!   source's is vacuous — see that gate's own note.

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::stage::{take_census, CapProfile, Split, StageStack, StageStackCore,
                      StageStackCoreSpec, StageStackSpec};
use turbojet::stator::VariableStatorCore;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;
const FLOOR: f64 = 0.55;
const THROTTLE: [f64; 4] = [1500.0, 1200.0, 1000.0, 800.0];
const ALL_SHAPES: [&str; 5] = ["flow/press", "press/flow", "tilted", "steep", "flat-eta"];

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
    let (gc, cc, gt, ct) = (1.4f64, 1004.0f64, 1.3f64, 1239.0f64);
    Gas::new(GasSpec {
        gamma_c: gc, cp_c: cc, r_c: (gc - 1.0) / gc * cc,
        gamma_t: gt, cp_t: ct, r_t: (gt - 1.0) / gt * ct,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn design(gas: Gas) -> TwoSpoolEngine {
    build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, 50_000.0, real())
}

/// Rung 53/54/55's five disclosed shapes, verbatim — `phi_surge` armed, NO throat model (that
/// is rung 56's file).
fn maps(name: &str) -> (ComponentMap, ComponentMap) {
    let f = ComponentMap::flat();
    let (l, h) = match name {
        "flow/press" => (ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f },
                         ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..f }),
        "press/flow" => (ComponentMap { a: 0.05, b: 0.20, sigma: 0.1, l: 1.0, ..f },
                         ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f }),
        "tilted"     => (ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..f },
                         ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..f }),
        "steep"      => (ComponentMap { a: 0.25, b: 0.12, sigma: 0.3, l: 1.2, ..f },
                         ComponentMap { a: 0.25, b: 0.12, sigma: 0.3, l: 1.2, ..f }),
        "flat-eta"   => (ComponentMap { sigma: 0.1, l: 0.7, ..f },
                         ComponentMap { sigma: 0.1, l: 1.0, ..f }),
        other => panic!("unknown shape {other}"),
    };
    (l.with_phi_surge(FLOOR), h.with_phi_surge(FLOOR))
}

/// Python's `_sm`, with its eight defaults.
#[allow(clippy::too_many_arguments)]
fn sm(d: TwoSpoolEngine, shape: &str, k_lp: usize, k_hp: usize, vl: f64, vh: f64, split: Split,
      vs_lp: Option<usize>, vs_hp: Option<usize>) -> StageStackCore {
    let (ml, mh) = maps(shape);
    StageStackCore::new(StageStackCoreSpec {
        vsv_lp: vl, vsv_hp: vh, k_lp, k_hp, split,
        vsv_stages_lp: vs_lp, vsv_stages_hp: vs_hp,
        ..StageStackCoreSpec::new(d, flight(), 1.0, ml, mh)
    })
}

/// The common call — `flow/press`, `dT`, the lumped lever.
fn sm8(d: TwoSpoolEngine, k: usize) -> StageStackCore {
    sm(d, "flow/press", k, k, 0.0, 0.0, Split::DT, None, None)
}

/// The 19 matched fields the reduce is asserted over, as raw bits.
fn stack_fields(m: &StageStackCore, tt4: f64) -> Vec<(&'static str, u64)> {
    named(m.match_point(&flight(), tt4))
}

fn stator_fields(m: &VariableStatorCore, tt4: f64) -> Vec<(&'static str, u64)> {
    named(m.core.match_point(&flight(), tt4))
}

fn named(o: turbojet::two_spool::TwoSpoolMapResult) -> Vec<(&'static str, u64)> {
    vec![
        ("pi_lpc", o.base.pi_lpc.to_bits()), ("pi_hpc", o.base.pi_hpc.to_bits()),
        ("n_lp", o.n_lp.to_bits()), ("n_hp", o.n_hp.to_bits()),
        ("phi_lp", o.phi_lp.to_bits()), ("phi_hp", o.phi_hp.to_bits()),
        ("slip", o.slip.to_bits()),
        ("eta_lpc", o.eta_lpc.to_bits()), ("eta_hpc", o.eta_hpc.to_bits()),
        ("eta_hpt", o.eta_hpt.to_bits()), ("eta_lpt", o.eta_lpt.to_bits()),
        ("tau_lpc", o.base.tau_lpc.to_bits()), ("tau_hpc", o.base.tau_hpc.to_bits()),
        ("tau_hpt", o.base.tau_hpt.to_bits()), ("tau_lpt", o.base.tau_lpt.to_bits()),
        ("mdot_air", o.base.mdot_air.to_bits()), ("thrust", o.base.thrust.to_bits()),
        ("N_lp_ratio", o.n_lp_ratio.to_bits()), ("N_hp_ratio", o.n_hp_ratio.to_bits()),
    ]
}

/// The design references a hand-built stack needs — `(cmap, tau_d, pi_d, eta_d)` on the LP.
fn lp_design_point(m: &StageStackCore) -> (ComponentMap, f64, f64, f64) {
    (maps("flow/press").0, m.core.core.tau_lpc_d,
     m.core.core.base.pi_lpc_design, m.core.core.base.eta_lpc)
}

// ==========================================================================================
// GATE 1 — REDUCE: an IDENTITY at K = 1
// ==========================================================================================

/// THE SPINE. At `K = 1` no stack object exists, both efficiency loops are the INHERITED rung-39
/// ones, and there is no rung-55 code path to skip — so this is an identity, not a tolerance.
/// Checked at a MOVED stator so it cannot be passing by way of rung 53's own `v == 0` early
/// returns.
#[test]
fn test_reduce_k1_is_bit_for_bit_rung53() {
    for (vl, vh) in [(0.0, 0.0), (0.30, 0.0), (0.0, 0.15), (0.20, 0.10)] {
        let d = design(cpg_gas());
        let (ml, mh) = maps("flow/press");
        let r53 = VariableStatorCore::new(d.clone(), flight(), 1.0, ml, mh, vl, vh);
        let st = sm(d, "flow/press", 1, 1, vl, vh, Split::DT, None, None);
        assert!(st.stack_of(Spool::Lp).is_none() && st.stack_of(Spool::Hp).is_none(),
                "rung 55 must not build a stack object at K = 1 — the reduce is an identity");
        for t in THROTTLE {
            for (got, want) in stack_fields(&st, t).iter().zip(stator_fields(&r53, t).iter()) {
                assert_eq!(got.1, want.1,
                           "rung-55 K=1 reduce broken on {} at Tt4={t}, vsv=({vl},{vh})", got.0);
            }
        }
    }
}

/// **§ 5.10 P5.** Even a HAND-BUILT one-stage stack is bit-for-bit: `StageStack::solve_n`
/// DISPATCHES to rung 32's own `ComponentMap::solve_n`, so it is the same code and not merely
/// the same algebra.
///
/// Step 4's FINDING 3 sharpened this and it is worth restating where the gate lives: the
/// dispatch is a property of the OBJECT and is unreachable through the MATCHER, which builds no
/// stack at `K = 1` at all (`solve_n_k1` = 0 on all three oracle arms). So this gate needs the
/// hand-built stack — a matcher-driven version would be vacuous.
///
/// # AND THE VALUE HALF IS VACUOUS TOO — MEASURED, AND IT IS THE SOURCE'S GATE THAT IS WEAK
///
/// Python's whole assertion is `stack.solve_n(...) == cmap.solve_n(...)`, and its docstring says
/// that equality shows *"it is the same code and not merely the same algebra"*. **It does not.**
/// The `K = 1` dispatch was deleted in `stage.rs` (`if false && self.k == 1`) and this gate still
/// passed, bit-for-bit, on all three points — because the fall-through bisects the SAME bracket
/// `[0.1, 2.0]` to the SAME `1e-14`, and its residual `tau_of − tau_c` differs from the map's
/// `psi*n^2 − target` by a POSITIVE affine factor, which a bisection reading only signs cannot
/// see. Same lo/hi sequence, same `0.5*(lo+hi)`, same bits.
///
/// So the dispatch is gated STRUCTURALLY, off `stage.rs`'s own census: a dispatched call runs
/// **zero** stack bisection passes and **zero** marches, and a fallen-through one runs 48 and
/// 51. That is the discriminator, and it was written only after the value half was watched to
/// fail to be one. *A documented gate that doesn't exist*, caught in the source rather than in
/// the port.
///
/// # ONE FRAGILITY, WRITTEN DOWN BECAUSE IT IS INVISIBLE FROM THE CALL
///
/// `take_census` READS AND RESETS a thread-local, so this gate is correct only while it is the
/// **only** census consumer in this binary. `cargo test` runs the gates in one file on several
/// threads; a second reader in `rung55.rs` would take tallies this one produced and vice versa,
/// and the failure would look like a physics disagreement rather than a harness one. If a census
/// read is ever needed elsewhere here, the two must be serialised — do not simply add one.
#[test]
fn test_reduce_stack_object_dispatches_at_k1() {
    let m = sm8(design(cpg_gas()), 8);       // a matcher only, to read the design point
    let (cmap, tau_d, pi_d, eta_d) = lp_design_point(&m);
    let stack = StageStack::new(StageStackSpec::new(1, cmap, tau_d, pi_d, eta_d));
    let _ = take_census();                   // discard the construction's tallies
    for (mm, tau) in [(1.0, tau_d), (0.73, 1.3255), (0.46, 1.2150)] {
        assert_eq!(stack.solve_n(mm, tau, eta_d).to_bits(),
                   cmap.solve_n(mm, tau, tau_d).to_bits(),
                   "K=1 must agree with the map's solver (m={mm}, tau={tau})");
    }
    let c = take_census();
    assert_eq!(c.solve_n_calls, 3, "three calls entered the stack's solver");
    assert_eq!(c.solve_n_passes, 0,
               "…and NONE of them ran the stack's own bisection: at K = 1 it must RETURN \
                ComponentMap::solve_n's answer, not reproduce it (got {} passes)",
               c.solve_n_passes);
    assert_eq!(c.marches, 0,
               "…and the stack marched nothing, which is the same statement from the other side");
    assert!((stack.e_d - eta_d).abs() <= 1e-12,
            "at K = 1 the per-stage efficiency IS the lumped one (the inversion is the identity)");
}

/// GATE 2b — A FREE CONSISTENCY CHECK ON THE WHOLE CONSTRUCTION.
///
/// Nothing in the stack was told about polytropic efficiency: it is handed an ISENTROPIC design
/// point and a stage count. Yet the derived per-stage efficiency comes out ABOVE the lumped one
/// (the REHEAT effect) and converges, first order, on rung 2b's `e_c = ln(pi_d)/(kc*ln(tau_d))`.
/// The stack therefore INTERPOLATES rung 2 (`K = 1`, isentropic) to rung 2b (`K -> inf`,
/// polytropic), and rung 2b's shipped `eta_c < e_c` ordering falls out rather than being imposed.
#[test]
fn test_stack_reproduces_rung2b_polytropic_efficiency() {
    let kc = 1.4 / 0.4;
    let m = sm8(design(cpg_gas()), 8);
    let (cmap, tau_d, pi_d, eta_d) = lp_design_point(&m);
    let e_poly = pi_d.ln() / (kc * tau_d.ln());
    assert!(e_poly > eta_d, "rung 2b's own ordering: eta_c < e_c for a compressor");

    let mut errs = Vec::new();
    for k in [1usize, 2, 4, 8, 16, 32] {         // every step a DOUBLING
        let s = StageStack::new(StageStackSpec {
            kc, ..StageStackSpec::new(k, cmap, tau_d, pi_d, eta_d) });
        if k == 1 {
            assert!((s.e_d - eta_d).abs() <= 1e-12);
        } else {
            assert!(eta_d < s.e_d && s.e_d < e_poly, "K={k}: e_d must sit BETWEEN the two rungs");
        }
        errs.push(e_poly - s.e_d);
    }
    for w in errs.windows(2) {
        assert!(w[1] < w[0], "the approach to e_c must be monotone: {errs:?}");
    }
    for w in errs[1..].windows(2) {
        let r = w[1] / w[0];
        assert!(r > 0.35 && r < 0.65, "first-order convergence to e_c expected, got {r:.3}");
    }
}

/// The identity is a property of the CODE PATH, not of the gas — so it must hold on the
/// production reacting-equilibrium gas too (rung 53/54's both-gases discipline).
#[test]
fn test_reduce_k1_on_the_reacting_equilibrium_gas() {
    let d = design(Gas::reacting_equilibrium());
    let (ml, mh) = maps("flow/press");
    let r53 = VariableStatorCore::new(d.clone(), flight(), 1.0, ml, mh, 0.20, 0.0);
    let st = sm(d, "flow/press", 1, 1, 0.20, 0.0, Split::DT, None, None);
    for t in [1500.0, 1200.0] {
        for (got, want) in stack_fields(&st, t).iter().zip(stator_fields(&r53, t).iter()) {
            assert_eq!(got.1, want.1, "rung-55 reacting-gas reduce broken on {}", got.0);
        }
    }
}

// ==========================================================================================
// GATE 2 — THE DERIVED KINEMATICS
// ==========================================================================================

/// THE STACK DOES NOT RE-DESIGN THE ENGINE (rung 42/53's design-capture discipline). At the
/// design point every `phi_k = 1`, every `n_k = 1`, and the march returns `tau_d` — exactly, for
/// any resolution and any disclosed work split.
///
/// **The pressure-ladder clause reads `varpi_d`, which IS `ladder_p(theta_d, e_d)`** — the
/// constructor's own assignment, so this is Python's `stack._ladder_p(...)` call and not a
/// weaker stand-in. What it compares against is the SHIPPED design `pi_d`, which is the half
/// that makes it a check at all.
#[test]
fn test_design_ladder_is_exact_for_every_k_and_split() {
    for split in [Split::DT, Split::Tau] {
        for k in [2usize, 4, 8, 16] {
            let m = sm(design(cpg_gas()), "flow/press", k, k, 0.0, 0.0, split, None, None);
            for spool in [Spool::Lp, Spool::Hp] {
                let stack = m.stack_of(spool).expect("K > 1 builds a stack");
                let (tau_d, eta_d) = match spool {
                    Spool::Lp => (m.core.core.tau_lpc_d, m.core.core.base.eta_lpc),
                    Spool::Hp => (m.core.core.tau_hpc_d, m.core.core.base.eta_hpc),
                };
                let r = stack.march(1.0, 1.0, eta_d);
                assert!((r.tau - tau_d).abs() <= 1e-12 * tau_d.abs(),
                        "K={k} split={split:?}: design march must return tau_d exactly");
                assert_eq!(r.clamped, 0);
                for (i, (phi, nk)) in r.phis.iter().zip(r.n_ks.iter()).enumerate() {
                    assert!((phi - 1.0).abs() <= 1e-12, "stage {i} phi != 1 at design");
                    assert!((nk - 1.0).abs() <= 1e-12, "stage {i} n_k != 1 at design");
                }
                // the per-stage efficiency reproduces the SHIPPED design pi (no new constant)
                let last = *stack.varpi_d.last().expect("the ladder has K+1 entries");
                assert!((last - stack.pi_d).abs() <= 1e-10 * stack.pi_d.abs(),
                        "the pressure ladder must land on pi_d: {last} vs {}", stack.pi_d);
                assert!(stack.e_d > eta_d,
                        "the REHEAT effect: a resolved stack's per-stage eta sits ABOVE the \
                         lumped one");
            }
        }
    }
}

/// THE CROSS-RUNG RESULT, before any measurement: `phi_1 = m/n` EXACTLY, so the face flow
/// coefficient every rung since 32 reads IS the front stage's. Rungs 36–53 were reading the
/// binding stage all along — a BOUNDING in rung 53's style, not a refutation.
#[test]
fn test_front_stage_phi_is_the_face_phi() {
    for k in [4usize, 8] {
        let m = sm8(design(cpg_gas()), k);
        for t in THROTTLE {
            let r = m.stage_margin(&flight(), t);
            for spool in [Spool::Lp, Spool::Hp] {
                let s = r.spool(spool);
                assert!((s.stages[0].phi - s.phi_face).abs() <= 1e-13 * s.phi_face.abs(),
                        "{spool:?} stage-0 phi must BE the face phi at Tt4={t}");
                assert!((s.phi_face - s.m / s.n).abs() <= 1e-13 * (s.m / s.n).abs());
            }
        }
    }
}

/// The style guards, half of them. Python's single `test_capacity_style_guards_reject_nonsense`
/// makes THREE refusals; `#[should_panic]` is per-test, so it splits — and its middle refusal
/// (`split="equal-psi"`) is UNREPRESENTABLE here. See [`slice_n_deferrals`] item 1.
#[test]
#[should_panic(expected = "needs K >= 1 stages")]
fn test_style_guard_rejects_zero_stages() {
    let (cmap, _) = maps("flow/press");
    let _ = StageStack::new(StageStackSpec::new(0, cmap, 1.4, 3.0, 0.9));
}

/// The other half — `vsv_stages` past the stage count. HALF unrepresentable: `usize` carries
/// Python's `0 <= vsv_stages`, and only the `<= K` clause survives as a runtime check.
#[test]
#[should_panic(expected = "vsv_stages must be in [0, K=4]")]
fn test_style_guard_rejects_more_stator_rows_than_stages() {
    let (cmap, _) = maps("flow/press");
    let _ = StageStack::new(StageStackSpec {
        vsv_stages: Some(5), ..StageStackSpec::new(4, cmap, 1.4, 3.0, 0.9) });
}

// ==========================================================================================
// GATE 3 — THE NON-TAUTOLOGY GATE (the reason this is a rung)
// ==========================================================================================

/// WITHOUT THIS THE RUNG IS A RE-READ. The front-to-rear spread alone is a functional of the
/// `(tau_c, pi_c)` rung 39 already solves. The content is the FEEDBACK: with a per-stage
/// `psi(phi_k)` the machine's work is no longer `psi(phi_face)*n^2`, so the stack MOVES the
/// running line. Exactly zero at `K = 1`; negative (the stack is WEAKER) and deepening beyond it.
#[test]
fn test_marched_work_differs_from_lumped_and_grows_with_throttle_depth() {
    let d = design(cpg_gas());
    let flat = sm8(d.clone(), 1);
    for t in THROTTLE {
        let g = flat.work_gap(&flight(), t);
        for spool in [Spool::Lp, Spool::Hp] {
            assert_eq!(g.spool(spool).gap, 0.0, "K=1 march IS the lumped law, exactly");
        }
    }

    let m8 = sm8(d, 8);
    let mut prev = [0.0f64; 2];
    for t in THROTTLE {                          // descending throttle
        let g = m8.work_gap(&flight(), t);
        for (i, spool) in [Spool::Lp, Spool::Hp].into_iter().enumerate() {
            let frac = g.spool(spool).gap_frac;
            if t == 1500.0 {
                assert!(frac.abs() < 1e-12, "at design the stack does the design work exactly");
            } else {
                assert!(frac < 0.0,
                        "the marched stack must be WEAKER than the lumped law ({spool:?}, \
                         Tt4={t}): got {frac:+.4e}");
                assert!(frac < prev[i],
                        "the gap must DEEPEN with throttle depth ({spool:?}, Tt4={t})");
            }
            prev[i] = frac;
        }
    }
    // the HP carries the bigger pressure ratio, hence the bigger density mismatch
    let g = m8.work_gap(&flight(), 800.0);
    assert!(g.hp.gap_frac < g.lp.gap_frac && g.lp.gap_frac < -0.05);
}

// ==========================================================================================
// GATE 4 — P1: the RUNNING LINE MOVES (n up, phi down), paid in SHAFT SPEED
// ==========================================================================================

/// P1, pre-registered and HIT on sign + monotonicity (the LEVEL was predicted 5–15 % and
/// measured 2.7–4.2 % — scored a miss in the anchor). A weaker stack must be run FASTER to do
/// the pinned work, so `n` RISES and the front stage's `phi` FALLS.
#[test]
fn test_p1_running_line_shift_sign_and_monotonicity() {
    for shape in ALL_SHAPES {
        let m8 = sm(design(cpg_gas()), shape, 8, 8, 0.0, 0.0, Split::DT, None, None);
        let rows = m8.running_line_shift(&flight(), &THROTTLE);
        assert!(rows[0].lp.d_n.abs() < 1e-9 && rows[0].lp.d_phi.abs() < 1e-9,
                "the design point must not move (the stack is design-consistent)");
        for spool in [Spool::Lp, Spool::Hp] {
            let dn: Vec<f64> = rows[1..].iter().map(|r| r.spool(spool).d_n).collect();
            let dphi: Vec<f64> = rows[1..].iter().map(|r| r.spool(spool).d_phi).collect();
            assert!(dn.iter().all(|&x| x > 0.0), "{shape}/{spool:?}: n must RISE");
            assert!(dphi.iter().all(|&x| x < 0.0), "{shape}/{spool:?}: phi must FALL");
            assert!(dn.windows(2).all(|w| w[0] <= w[1]),
                    "{shape}/{spool:?}: the shift must deepen with throttle: {dn:?}");
            assert!(dphi.windows(2).all(|w| w[0] >= w[1]), "{shape}/{spool:?}: {dphi:?}");
        }
    }
}

/// Like rung 53's stator, the stack is thrust-neutral: it moves SPEED. On a flat efficiency
/// island `pi_c` cannot move at all, which isolates the channel exactly.
#[test]
fn test_p1_is_paid_in_shaft_speed_not_performance() {
    let d = design(cpg_gas());
    for r in sm8(d.clone(), 8).running_line_shift(&flight(), &THROTTLE) {
        assert!(r.d_thrust.abs() < 0.01, "thrust must barely move");
        assert!(r.lp.d_pi.abs() < 0.005);
        assert!(r.lp.d_n.abs() > 3.0 * r.d_thrust.abs() || r.tt4 == 1500.0);
    }
    let flat = sm(d, "flat-eta", 8, 8, 0.0, 0.0, Split::DT, None, None);
    for r in flat.running_line_shift(&flight(), &THROTTLE) {
        assert!(r.lp.d_pi.abs() <= 1e-12,
                "on a flat island the stack cannot touch pi_c AT ALL — it is a pure speed lever");
    }
}

// ==========================================================================================
// GATE 5 — P4: one machine, two OPPOSITE failures
// ==========================================================================================

/// P4, pre-registered and HIT. The smallest incidence margin in the machine is the LP's FRONT
/// stage; the largest excursion on the HP is its REAR stage, running ABOVE design `phi` (toward
/// choke / negative incidence). A lumped block has ONE `phi` and can represent neither end.
#[test]
fn test_p4_front_stalls_while_the_rear_chokes() {
    let m8 = sm8(design(cpg_gas()), 8);
    let at_design = m8.stage_margin(&flight(), 1500.0);
    for spool in [Spool::Lp, Spool::Hp] {
        assert!(at_design.spool(spool).rear_excess.abs() <= 1e-12);
    }

    let r = m8.stage_margin(&flight(), 800.0);
    let (lp, hp) = (&r.lp, &r.hp);
    assert!(lp.worst == 0 && hp.worst == 0, "the FRONT stage stalls first on both spools");
    assert!(lp.m_i_worst < hp.m_i_worst,
            "the LP front stage is the worst incidence in the whole machine (rung 41's split)");
    assert!(hp.phi_rear > 1.10, "the HP REAR stage must run ABOVE design phi — toward choke");
    assert!(lp.phi_front < 0.75 && 0.75 < lp.phi_rear,
            "the LP spans the design point front-to-rear");
    for spool in [Spool::Lp, Spool::Hp] {
        let s = r.spool(spool);
        let phis: Vec<f64> = s.stages.iter().map(|x| x.phi).collect();
        assert!(phis.windows(2).all(|w| w[0] <= w[1]),
                "{spool:?}: phi must rise MONOTONICALLY front to rear: {phis:?}");
        assert!(s.rear_excess > 0.30);
    }
}

// ==========================================================================================
// GATE 6 — P5: K is a RESOLUTION, not a knob
// ==========================================================================================

/// P5, pre-registered and HIT with room to spare: the shift GROWS with `K` but its INCREMENTS
/// SHRINK — and in fact halve as `K` doubles (first-order convergence), so the stack has a
/// well-defined continuum limit and no verdict rides on a particular `K`.
#[test]
fn test_p5_shift_converges_in_k() {
    let d = design(cpg_gas());
    for t in [1200.0, 1000.0, 800.0] {
        let vals: Vec<f64> = [1usize, 2, 4, 8, 16].iter()
            .map(|&k| sm8(d.clone(), k).running_line_shift(&flight(), &[t])[0].lp.d_phi)
            .collect();
        let incr: Vec<f64> = vals.windows(2).map(|w| (w[1] - w[0]).abs()).collect();
        assert!(incr.iter().all(|&x| x > 0.0));
        assert!(incr.windows(2).all(|w| w[0] >= w[1]),
                "the K-increments must SHRINK at Tt4={t}: {incr:?}");
        for w in incr[1..].windows(2) {          // halving, within 25 %
            let r = w[1] / w[0];
            assert!(r > 0.35 && r < 0.65, "first-order convergence expected, got {r:.3}");
        }
    }
}

// ==========================================================================================
// GATE 7 — P6: the disclosed WORK SPLIT carries no verdict
// ==========================================================================================

/// P6, pre-registered band `< 25 %` and HIT by an order of magnitude. The split is rung 54's
/// "disclosed level" pattern: the KINEMATICS are derived, the split is disclosed, and the
/// verdict is asserted across it.
///
/// # THE SOURCE'S GATE IS ONE-SIDED, AND MY OWN DETECTOR IS WHAT FOUND IT
///
/// Every assertion Python makes here is an upper bound — `|y − x| / |x| < 0.25`, `worst == 0` on
/// both, `|Δrear_excess| / rear_excess < 0.25` — and **every one of them is satisfied at
/// `x == y`.** So the gate cannot distinguish *"the split is disclosed and the verdict does not
/// ride on it"*, which is P6's claim, from *"the split does nothing at all"*, which would make
/// the whole disclosed choice dead code. Measured: `Split::Tau`'s ladder arm was collapsed onto
/// `Split::DT` in `stage.rs` and **this gate passed**, while rung 56's `P4` gate — whose claim
/// is the opposite sign — failed at once.
///
/// The `assert_ne!` below closes it, on the in-repo precedent that already exists for exactly
/// this shape: step 4's `the_capacity_profile_cannot_reach_any_rung_55_reading` carries a
/// `moved > 0` clause *"so it cannot become a comparison of two identical objects"*, and Python's
/// own rung-56 P4 gate ends with `!=`. **A "nothing rides on this knob" gate is vacuous unless
/// something else says the knob is live** — and the two rungs' gates are two-sided only when
/// read together, which is not how a suite is read.
#[test]
fn test_p6_verdicts_survive_the_work_split() {
    let d = design(cpg_gas());
    let a = sm(d.clone(), "flow/press", 8, 8, 0.0, 0.0, Split::DT, None, None);
    let b = sm(d, "flow/press", 8, 8, 0.0, 0.0, Split::Tau, None, None);
    for t in [1200.0, 1000.0, 800.0] {
        let ra = a.running_line_shift(&flight(), &[t])[0];
        let rb = b.running_line_shift(&flight(), &[t])[0];
        for spool in [Spool::Lp, Spool::Hp] {
            let (x, y) = (ra.spool(spool).d_phi, rb.spool(spool).d_phi);
            assert!(x < 0.0 && y < 0.0);
            assert!((y - x).abs() / x.abs() < 0.25,
                    "{spool:?} d_phi split-sensitive at Tt4={t}");
            // …AND THE SPLIT IS LIVE, which no upper bound above can say. See the note.
            assert_ne!(x.to_bits(), y.to_bits(),
                       "the two work splits produced the IDENTICAL d_phi on {spool:?} at \
                        Tt4={t} — P6 says the verdict does not ride on the split, not that the \
                        split is dead");
        }
        let sa = a.stage_margin(&flight(), t);
        let sb = b.stage_margin(&flight(), t);
        assert!(sa.lp.worst == 0 && sb.lp.worst == 0);
        assert!((sb.hp.rear_excess - sa.hp.rear_excess).abs() / sa.hp.rear_excess < 0.25);
        assert_ne!(sa.hp.rear_excess.to_bits(), sb.hp.rear_excess.to_bits(),
                   "…and the same hole on the rear_excess half");
    }
}

// ==========================================================================================
// GATE 9 — P3: THE HEADLINE (the factorisation, and the interior row-count optimum)
// ==========================================================================================

/// THE HEADLINE. Holding the front stage's design incidence with a FRONT-ROW-ONLY stator costs a
/// small fraction of rung 53's whole-machine lever, and the collapse FACTORISES:
///
/// ```text
///     dN_ratio = (1/K) x (v*_front / v*_lumped)
/// ```
///
/// to within 5 % across an 8x range in `K`. The `1/K` leg was pre-registered; the SETTING leg was
/// not, and it is why P3's level was scored a miss. A front-only lever does not fight its own
/// speed rise.
#[test]
fn test_p3_front_row_lever_cost_factorises() {
    let t = 1000.0;
    let d = design(cpg_gas());
    let (ml, mh) = maps("flow/press");
    let r53 = VariableStatorCore::new(d.clone(), flight(), 1.0, ml, mh, 0.0, 0.0);
    let row53 = r53.incidence_schedule(&flight(), &[t], Spool::Lp, 4.0)[0];
    let b53 = r53.at_setting(0.0, 0.0).core.match_point(&flight(), t);
    let s53 = r53.at_setting(row53.vsv_star, 0.0).core.match_point(&flight(), t);
    let dn53 = (s53.n_lp_ratio - b53.n_lp_ratio) / b53.n_lp_ratio;
    assert!(dn53 > 0.60, "rung 53's lumped lever must be expensive (bare-at-throttle reference)");

    let mut prev_v: Option<f64> = None;
    let mut dn = f64::NAN;
    for k in [2usize, 4, 8, 16] {
        let m = sm(d.clone(), "flow/press", k, 8, 0.0, 0.0, Split::DT, Some(1), None);
        let r = m.stage_incidence_schedule(&flight(), &[t], Spool::Lp, 0, 4.0)[0];
        assert!(r.reached, "the front-row schedule must EXIST at K={k}");
        let bare = m.at_setting(0.0, 0.0).match_point(&flight(), t);
        let sib = m.at_setting(r.vsv_star, 0.0).match_point(&flight(), t);
        dn = (sib.n_lp_ratio - bare.n_lp_ratio) / bare.n_lp_ratio;
        let v_ratio = r.vsv_star / row53.vsv_star;
        let want = v_ratio / k as f64;
        assert!((dn / dn53 - want).abs() <= 0.05 * want.abs(),
                "K={k}: the cost must factorise as (1/K)x(v* ratio): {} vs {want}", dn / dn53);
        assert!(v_ratio < 0.40, "the front-only lever needs a much SMALLER setting too");
        if let Some(p) = prev_v {                // v* SATURATES while the penalty keeps falling
            assert!(r.vsv_star < p);
        }
        prev_v = Some(r.vsv_star);
    }
    assert!(dn < 0.03, "at K = 16 the front-row lever must be nearly free in shaft speed");
}

/// A POSITIONAL LEVER PAYS FOR THE ROWS IT MOVES OUT OF THE ROWS IT DOES NOT — through the shaft
/// speed every stage shares. So relief in the row count is not monotone: it peaks at 3–4 rows of
/// 8 and then REVERSES, ending WORSE than bare. The first object in this project whose optimum
/// is a COUNT.
///
/// # The scan step, and the one shipped-source constant a test reaches inside
///
/// Python sets `m._V_SCAN = 0.01` here. That is the ONLY instance-level constant override in
/// rungs 53–56 (`grep '\._[A-Z_]* *='` over the four suites returns it and one *assertion* on
/// `_V_STEP`), and it MOVES the answer: at the default 0.05 the bracket lands elsewhere. Rust's
/// associated const cannot be overridden, so [`StageStackCore::with_v_scan`] was added — the
/// slice's FIFTH gated-code edit, landing at STEP 5 in a file steps 1 and 3 had both finished
/// with. The step table called step 1 *"ALL changes to already-gated code"*; step 3 already
/// refuted that once (`TwoSpoolMapCore`'s two stack fields), and this is the second time, from a
/// direction neither looked: **a suite can reach into a constant, so porting the CODE does not
/// bound the edits the TESTS force.**
///
/// # The cost clause is STRENGTHENED, because the source's is vacuous
///
/// Python asserts `cost == dict(sorted(cost.items())) or [...] == sorted([...])`. The first
/// disjunct compares a dict to a re-ordered copy of ITSELF and `dict.__eq__` ignores order — so
/// it is `True` for ANY cost curve, the `or` short-circuits, and the monotonicity it looks like
/// it gates is never evaluated. Measured on PyPy before porting: the costs ARE ascending
/// (0.0230, 0.0529, 0.0931, 0.1509, 0.2432, 0.4309), so the live half is asserted here alone.
/// *A ported test can go VACUOUS* — this time the vacuity was already in the source.
#[test]
fn test_p3_row_count_has_an_interior_optimum() {
    let (t, k) = (1000.0, 8usize);
    let d = design(cpg_gas());
    let base = sm(d.clone(), "flow/press", k, k, 0.0, 0.0, Split::DT, Some(1), None);
    let mi_bare = base.at_setting(0.0, 0.0).stage_margin(&flight(), t).lp.m_i_worst;

    let (mut relief, mut cost) = (Vec::new(), Vec::new());
    for rows in 1usize..=6 {
        let m = sm(d.clone(), "flow/press", k, k, 0.0, 0.0, Split::DT, Some(rows), None)
            .with_v_scan(0.01);
        let r = m.stage_incidence_schedule(&flight(), &[t], Spool::Lp, 0, 4.0)[0];
        assert!(r.reached, "rows={rows}: the schedule must exist");
        let sib = m.at_setting(r.vsv_star, 0.0);
        let s = sib.stage_margin(&flight(), t);
        let b = base.at_setting(0.0, 0.0).match_point(&flight(), t);
        relief.push((s.lp.m_i_worst - mi_bare) / mi_bare);
        cost.push((sib.match_point(&flight(), t).n_lp_ratio - b.n_lp_ratio) / b.n_lp_ratio);
        // the worst stage is PROMOTED into the rows the stator does not move
        assert!(s.lp.worst >= rows.min(k - 1) || s.lp.worst > 0);
    }

    let peak = (0..6).max_by(|&a, &b| relief[a].total_cmp(&relief[b])).expect("six rows");
    assert!(peak == 2 || peak == 3, "relief must peak at 3-4 rows of 8, got {relief:?}");
    assert!(relief[5] < 0.0 && 0.0 < relief[0],
            "moving too many rows must end WORSE than bare (the reversal)");
    assert!(relief[4] < relief[3], "the fall past the peak must be smooth, not a jump");
    // cost climbs monotonically while relief turns over — TWO currencies, TWO optima
    assert!(cost.windows(2).all(|w| w[0] <= w[1]), "the cost must climb in rows: {cost:?}");
    let ppc: Vec<f64> = relief.iter().zip(cost.iter()).map(|(r, c)| r / c).collect();
    let best = (0..6).max_by(|&a, &b| ppc[a].total_cmp(&ppc[b])).expect("six rows");
    assert_eq!(best, 0,
               "relief PER UNIT SPEED is cheapest at ONE row — a different optimum: {ppc:?}");
}

/// THE FIFTH GATED-CODE EDIT, GATED — because the gate above **cannot see it**.
///
/// [`StageStackCore::with_v_scan`] was added at step 5 so the row-count experiment runs at the
/// scan step the source runs it at. The obvious place to gate it is that experiment, and the
/// obvious gate is vacuous: **measured, the whole gate passes unchanged at the default step.**
/// So the edit ships with nothing reading it as a claim unless something else does.
///
/// Measured on PyPy, `rows = 1..6` at `Tt4 = 1000`, `K = 8`, front-row lever:
///
/// ```text
///     v* at 0.01   0x1.6a19e5f8b8522p-2 …   RELIEF  0x1.867f37d1b88f3p-4 …
///     v* at 0.05   0x1.6a19e5f8b999ap-2 …   RELIEF  0x1.867f37d1b8b57p-4 …
/// ```
///
/// — the roots differ from about the 11th decimal, which is a real difference (the bisection
/// stops on `INC_TOL = 1e-12` in the RESIDUAL, so a different bracket lands on a different
/// root) and is orders below every bar the experiment asserts. Two consequences, both recorded
/// rather than smoothed over:
///
/// * The edit is justified by FAITHFULNESS, not by a failing gate — the port must run the
///   source's experiment, not a neighbouring one that happens to agree. *A dead guard's
///   threshold is worth more than its count*, third instance in this slice, now on a knob whose
///   deadness is a VERDICT's and not a value's.
/// * The verdict is therefore ROBUST to the scan step, which is a stronger statement about the
///   interior optimum than the source makes — its own note says the finer scan was added
///   because a coarse one "could have been a bracket artifact". It was not.
#[test]
fn test_the_scan_step_is_an_instance_value_and_it_moves_the_root() {
    let d = design(cpg_gas());
    let build = || sm(d.clone(), "flow/press", 8, 8, 0.0, 0.0, Split::DT, Some(3), None);
    let star = |m: StageStackCore| {
        let r = m.stage_incidence_schedule(&flight(), &[1000.0], Spool::Lp, 0, 4.0)[0];
        assert!(r.reached);
        r.vsv_star
    };
    let coarse = star(build());
    let fine = star(build().with_v_scan(0.01));
    // LIVE: the field reaches the scan at all.
    assert_ne!(coarse.to_bits(), fine.to_bits(),
               "with_v_scan must reach the scan — an ignored field would make the row-count \
                experiment a different one from Python's while reading as the same");
    // …and the default IS the constant, so an un-built matcher is Python's class attribute.
    assert_eq!(star(build().with_v_scan(StageStackCore::V_SCAN)).to_bits(), coarse.to_bits());
    // …and the move is small enough that no verdict in this file rides on it.
    assert!((fine - coarse).abs() < 1e-9 && (fine - coarse) != 0.0,
            "the two roots must differ, and by far less than any bar: {coarse} vs {fine}");
}

/// Rung 53 conceded its schedule numbers were "model-bound". Resolved into stages, the ALL-ROWS
/// schedule is not merely expensive — below `Tt4 ~ 1300` it is UNREACHABLE, the scan running
/// into the speed-line bracket at `v ~ 2.1–2.4`. (Rung 54 found the same object ceasing to exist
/// under the throat, by a different mechanism: two independent ceilings.)
///
/// This is § 5.10 (ii)'s census in its smallest form — the non-reached rows are the LUMPED
/// lever's, and § 5.10 (i)'s frame table says every one of them is a `try_solve_n` abort caught
/// by the scan.
#[test]
fn test_p3_all_rows_schedule_ceases_to_exist_deep_off_design() {
    let m = sm(design(cpg_gas()), "flow/press", 8, 8, 0.0, 0.0, Split::DT, Some(8), None);
    let rows = m.stage_incidence_schedule(
        &flight(), &[1500.0, 1300.0, 1100.0, 1000.0], Spool::Lp, 0, 4.0);
    let reached: Vec<bool> = rows.iter().map(|r| r.reached).collect();
    assert!(reached[0] && reached[1], "the all-rows schedule must still exist near design");
    assert!(!reached[2] && !reached[3],
            "the all-rows schedule must CEASE TO EXIST deep off design");
}

// ==========================================================================================
// GATE 10 — THE CYCLE IS UNTOUCHED
// ==========================================================================================

/// The project's standing gate: rung 55 is reached through a separate entry point, so the
/// default single-spool design run must be bit-for-bit what it was.
#[test]
fn test_cycle_untouched_default_design_run_is_bit_for_bit_rung6() {
    let eng = build_turbojet(Gas::reacting_equilibrium(), 10.0, 1600.0, 50_000.0, Losses {
        pi_d: 0.97, eta_c: 0.90, eta_b: 0.99, pi_b: 0.96, eta_t: 0.92, eta_m: 0.99,
        pi_n: 0.98, ..Losses::default()
    });
    let a = eng.run(&flight(), 1.0);
    let b = eng.run(&flight(), 1.0);
    assert_eq!(a.performance.specific_thrust.to_bits(),
               b.performance.specific_thrust.to_bits());
    for st in ["2", "3", "4", "5", "9"] {
        assert_eq!(a.station(st).tt.to_bits(), b.station(st).tt.to_bits());
        assert_eq!(a.station(st).pt.to_bits(), b.station(st).pt.to_bits());
    }
}

// ==========================================================================================
// THE LEDGER — what rungs 55/56 do NOT port, and why
// ==========================================================================================

/// The slice-N deferral ledger, taken over from `slice_n_smoke.rs::slice_n_deferrals_so_far`.
///
/// It lives here rather than in that file because step 5's suites are where a reader compares
/// Python's gate names against Rust's; every entry below is a name that WILL be missing from
/// that diff, with the reason it is missing. *A deferral filed against the wrong cause is a
/// deferral nobody can discharge*, and one filed nowhere is worse.
#[test]
fn slice_n_deferrals() {
    // 1. `assert split in ("dT", "tau")` and `assert cap_profile in ("derived", "uniform")` —
    //    UNREPRESENTABLE. Both are enums, so there is no invalid value to construct and no
    //    runtime check to port. The type-level refusal is strictly stronger (rung 53's
    //    `lp_disabled` precedent, § 5.10 P10). This is why Python's
    //    `test_capacity_style_guards_reject_nonsense` becomes TWO Rust gates and not three, and
    //    rung 56's `test_uniform_profile_is_the_disclosed_alternative` TWO and not three.
    let _ = (Split::DT, Split::Tau, CapProfile::Derived, CapProfile::Uniform);

    // 2. `assert 0 <= vsv_stages` — HALF unrepresentable. `usize` carries the lower bound; the
    //    `<= K` half is live and gated by `test_style_guard_rejects_more_stator_rows_than_stages`.

    // 3. Rung 55's `lp_disabled` refusal — no such parameter exists in Rust, so
    //    `assert not (lp_disabled and K > 1)` has nothing to witness (§ 5.10 P10).

    // 4. `assert spool in self._SPOOLS` — THREE instances (`_stack_of`, `throat_walk`,
    //    `stage_incidence_schedule`), all UNREPRESENTABLE: `Spool` is a two-variant enum.
    let _ = (Spool::Lp, Spool::Hp);

    // 5. Rung 55's `test_cycle_untouched_transient_ladder_is_bit_for_bit_unstacked` — **OWED TO
    //    PHASE 6**, and this is the one deferral in the ledger that is a whole GATE rather than
    //    an assert. It runs a rung-43 fuel transient twice on the same hardware — once before a
    //    stack is live, once after — and demands the two point lists compare `==`; its content
    //    is ABSENCE OF LEAKAGE from the stack into the transient closures. What makes it
    //    unportable is that `TwoSpoolFuelTransient` does not exist in Rust yet, NOT `phi_max`,
    //    which is where the first draft filed it. The assertion, quoted so phase 6 does not
    //    re-derive it:
    //
    //        before == after   over [(s, nu_lp, nu_hp, Tt4)], with a K=8 stacked matcher
    //                          matched at Tt4 = 1000 between the two runs
    //
    // 6. `ComponentMap.phi_max` — still booked to PHASE 6, unchanged from slice M's ledger and
    //    NOT re-opened here.

    // 7. Rung 56's `_M_of_nu` range guard is LATENT-ONLY on § 5.10's grid (worst `nu^2` is 2.7 %
    //    of the limit) and has NO Python gate at all. Rust gates it anyway —
    //    `rung56.rs::test_total_referenced_mach_guard_is_latent_not_absent` — which is a gate
    //    ADDED rather than deferred, and it is listed here so the name diff is symmetric in both
    //    directions.
}
