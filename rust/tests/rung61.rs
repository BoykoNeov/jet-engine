//! RUNG 61 — STATOR + BLEED: **a compensating lever buys back the COORDINATE, not the BILL.**
//!
//! Ports `tests/test_rung61.py`'s 18 gates. Where a gate is STRENGTHENED past its source the
//! reason is written at the gate, never left to the diff.
//!
//! Gates, as `docs/rung61-spec.md` § Verification gates names them:
//!
//!  1. **REDUCE — TWO-AXIS.** `(0,0) ⇒ 39`, `(v,0) ⇒ 53`, `(0,b) ⇒ 42`, bit-for-bit, on the fast
//!     gas AND the reacting one.
//!  2. **THE TWO TRAPS** — the stators really move under the MRO, and `at_setting` carries the
//!     valve. Both failures would be plausible numbers with no exception.
//!  3. **THE HEADLINE** — at `b*` the φ-debit is fully bought back while ≥ 70 % of the overspeed
//!     SURVIVES; at `v = 0.30` the compensated point OVERSPEEDS the bare stator.
//!  4. **THE MECHANISM** — the compensated point is MORE unloaded than the stator-only one, and
//!     `psi_comp` matches the closed form (a PLUMBING check, not a finding).
//!  5. **RUNG 60's TAUTOLOGY, third route** — `dM_i = v` and `dM_phi = v·φ_s0²/(1+v·φ_s0)`,
//!     exactly, throttle-invariant.
//!  6. **THE SEAM AS POSED, REFUTED** — the valve SHRINKS the stator's authority; and
//!     artifact-free, the four-cell credit interaction is < 3 % of the credit sum.
//!  7. **SPOOL-DEPENDENCE** — `b*_LP` at every throttle, `b*_HP` at none.
//!  8. **THE PRICE COLLAPSE** on `(1+l)`, PLUS the negative control that keeps the ceiling
//!     un-claimed.
//!  9. **RUNG 53 CORRECTED** — its exact per-spool zero reproduced, then broken by the pair.
//! 10. **CYCLE UNTOUCHED.**

use turbojet::bleed::TwoSpoolBleedMatcher;
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::stator::VariableStatorCore;
use turbojet::stator_bleed::{Compensating, StatorBleedCore, Target};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses,
                          TwoSpoolMapCore};

const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;
const FLOOR: f64 = 0.55;

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

/// Python's `_cpg_gas` — `(g - 1.0)/g * cp`. See `slice_o_oracle.rs` for why the spelling is
/// load-bearing and not cosmetic.
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

/// Python's `LP` / `HP`.
fn lp_hp() -> (ComponentMap, ComponentMap) {
    let f = ComponentMap::flat();
    (ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f }.with_phi_surge(FLOOR),
     ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..f }.with_phi_surge(FLOOR))
}

/// Python's five disclosed `SHAPES`, BARE (the caller arms them).
fn bare_shape(name: &str) -> (ComponentMap, ComponentMap) {
    let f = ComponentMap::flat();
    match name {
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
    }
}

const SHAPES: [&str; 5] = ["flow/press", "press/flow", "tilted", "steep", "flat-eta"];

/// Python's `_sb`.
fn sb(vl: f64, vh: f64, b: f64) -> StatorBleedCore {
    let (l, h) = lp_hp();
    StatorBleedCore::new(design(cpg_gas()), flight(), 1.0, l, h, vl, vh, b)
}

/// Python's `_shaped`.
fn shaped(name: &str, vl: f64, b: f64) -> StatorBleedCore {
    let (ml, mh) = bare_shape(name);
    StatorBleedCore::new(design(cpg_gas()), flight(), 1.0,
                         ml.with_phi_surge(FLOOR), mh.with_phi_surge(FLOOR), vl, 0.0, b)
}

/// Python's `FIELDS`, as raw bit patterns — a reduce gate compares BITS, never a tolerance.
fn fields(r: &turbojet::two_spool::TwoSpoolMapResult) -> Vec<(&'static str, u64)> {
    vec![
        ("pi_lpc", r.base.pi_lpc.to_bits()), ("pi_hpc", r.base.pi_hpc.to_bits()),
        ("n_lp", r.n_lp.to_bits()), ("n_hp", r.n_hp.to_bits()),
        ("phi_lp", r.phi_lp.to_bits()), ("phi_hp", r.phi_hp.to_bits()),
        ("slip", r.slip.to_bits()), ("eta_lpc", r.eta_lpc.to_bits()),
        ("eta_hpc", r.eta_hpc.to_bits()), ("eta_hpt", r.eta_hpt.to_bits()),
        ("eta_lpt", r.eta_lpt.to_bits()), ("tau_lpc", r.base.tau_lpc.to_bits()),
        ("tau_hpc", r.base.tau_hpc.to_bits()), ("tau_hpt", r.base.tau_hpt.to_bits()),
        ("tau_lpt", r.base.tau_lpt.to_bits()), ("mdot_air", r.base.mdot_air.to_bits()),
        ("thrust", r.base.thrust.to_bits()),
    ]
}

fn solved(c: &Compensating) -> f64 {
    c.b_star().unwrap_or_else(|| panic!("expected a b*, got {:?}", c.reason()))
}

// =========================================================================================
// GATE 1 — REDUCE: TWO-AXIS, and stronger than either parent's alone
// =========================================================================================

fn reduce_on(gas: Gas) {
    let (l, h) = lp_hp();
    for tt4 in [1500.0f64, 1200.0] {
        // (0,0) => rung 39
        let d = design(gas.clone());
        let a = TwoSpoolMapCore::new(d.clone(), flight(), 1.0, l, h).match_point(&flight(), tt4);
        let b = StatorBleedCore::new(d, flight(), 1.0, l, h, 0.0, 0.0, 0.0)
            .core.core.match_point(&flight(), tt4);
        for ((k, x), (_, y)) in fields(&a).into_iter().zip(fields(&b)) {
            assert_eq!(x, y, "(0,0) {k} at Tt4={tt4}");
        }
        // (v,0) => rung 53
        let d = design(gas.clone());
        let a = VariableStatorCore::new(d.clone(), flight(), 1.0, l, h, 0.15, 0.0)
            .core.match_point(&flight(), tt4);
        let b = StatorBleedCore::new(d, flight(), 1.0, l, h, 0.15, 0.0, 0.0)
            .core.core.match_point(&flight(), tt4);
        for ((k, x), (_, y)) in fields(&a).into_iter().zip(fields(&b)) {
            assert_eq!(x, y, "(v,0) {k} at Tt4={tt4}");
        }
        // (0,b) => rung 42
        let d = design(gas.clone());
        let a = TwoSpoolBleedMatcher::new(d.clone(), flight(), 1.0, l, h, 0.08)
            .match_point(&flight(), tt4).base;
        let b = StatorBleedCore::new(d, flight(), 1.0, l, h, 0.0, 0.0, 0.08)
            .core.core.match_point(&flight(), tt4);
        for ((k, x), (_, y)) in fields(&a).into_iter().zip(fields(&b)) {
            assert_eq!(x, y, "(0,b) {k} at Tt4={tt4}");
        }
    }
}

#[test]
fn gate1_reduce_two_axis_bit_for_bit_fast() {
    reduce_on(Gas::thermally_perfect());
}

#[test]
fn gate1_reduce_two_axis_bit_for_bit_reacting() {
    reduce_on(Gas::reacting_equilibrium());
}

/// Rung 53's IDENTITY reduce survives the new class: at `v = 0` the stored maps ARE the maps
/// passed in, so there is still no rung-53 code path to skip.
///
/// **THE `is` HALF PORTS WEAKER AND THE `match` HALF PORTS STRONGER**, and both are stated.
/// [`ComponentMap`] is `Copy`, so Python's `m.map_lp is LP` has no Rust counterpart and becomes a
/// field-wise `==` (rung 53's own gate says the same). But
/// `StatorBleedMatcher.match is TwoSpoolBleedMatcher.match` becomes raw **fn-pointer equality**
/// against the `R42` table entry — a compile-time-shaped fact Python can only assert at runtime.
#[test]
fn gate1_reduce_map_objects_still_identical_at_design_setting() {
    let (l, h) = lp_hp();
    let m = sb(0.0, 0.0, 0.10);
    assert_eq!(m.core.core.map_lp, l);
    assert_eq!(m.core.core.map_hp, h);
    assert_eq!(m.core.map_lp_design, l);
    assert_eq!(m.core.map_hp_design, h);
    assert_eq!(m.core.core.map_lp.vsv, 0.0);

    // …and `match` resolves to rung 42's, which at b = 0 forwards to rung 39's.
    assert_eq!(m.core.core.hooks.try_match_point as usize,
               turbojet::bleed::R42.try_match_point as usize,
               "rung 61's inner table is not rung 42's, so its readers do not see the valve");
}

// =========================================================================================
// GATE 2 — THE TWO SILENT-FAILURE TRAPS
// =========================================================================================

/// TRAP: rung 42's constructor forwards no `vsv`, so a co-operative chain would leave the stators
/// at the design setting and report plausible WRONG numbers.
#[test]
fn gate2_trap_stators_actually_move_under_the_mro() {
    let m = sb(0.20, 0.05, 0.10);
    assert_eq!(m.core.core.map_lp.vsv, 0.20);
    assert_eq!(m.core.core.map_hp.vsv, 0.05);
    assert_eq!(m.bleed(), 0.10);
    // the design references are still captured at v = 0 (rung 53's construction discipline)
    assert_eq!(m.core.map_lp_design.vsv, 0.0);
    assert_eq!(m.core.map_hp_design.vsv, 0.0);
    // …and the moved stator is actually LIVE in the solve
    let bare = sb(0.0, 0.0, 0.0).core.core.match_point(&flight(), TT4);
    let moved = sb(0.20, 0.0, 0.0).core.core.match_point(&flight(), TT4);
    assert_ne!(moved.phi_lp.to_bits(), bare.phi_lp.to_bits());
}

/// TRAP: every rung-53/54 instrument routes through `at_setting`, and rung 53's version would
/// silently drop the valve.
///
/// **STRENGTHENED PAST THE SOURCE.** Python asserts `isinstance(sib, StatorBleedMatcher)`; in
/// Rust the sibling's TYPE is `VariableStatorCore` either way, so an `isinstance` port would be
/// vacuous — the discriminant is the TABLE POINTER and the carried state, which is what is
/// asserted here. That is the *a ported test can go VACUOUS* failure caught rather than shipped.
#[test]
fn gate2_trap_at_setting_carries_the_bleed() {
    let m = sb(0.0, 0.0, 0.12);
    let sib = m.at_setting(0.05, 0.0);
    assert_eq!(sib.bleed(), 0.12);
    assert_eq!(sib.core.vsv_lp, 0.05);

    // THROUGH THE TABLE — the copy rung 53's own readers reach.
    let dispatched = m.core.at_setting(0.05, 0.0);
    assert_eq!(dispatched.core.bleed, 0.12, "the dispatched sibling ran rung 53's body");
    assert_eq!(dispatched.core.hooks.try_match_point as usize,
               turbojet::bleed::R42.try_match_point as usize);

    // …so the instruments that route through it see the BLED machine.
    let rows = m.core.stator_sweep(&flight(), TT4, &[0.0, 0.05], Spool::Lp);
    let unbled = sb(0.0, 0.0, 0.0).core.stator_sweep(&flight(), TT4, &[0.0, 0.05], Spool::Lp);
    assert_ne!(rows[0].lp.phi_op.to_bits(), unbled[0].lp.phi_op.to_bits());
}

// =========================================================================================
// GATE 3 — THE HEADLINE: the debit goes, the bill stays
// =========================================================================================

/// `b*` removes the WHOLE φ-debit, and ≥ 70 % of the stator's overspeed survives it.
///
/// **THE BAR'S HEADROOM IS MEASURED, NOT ASSUMED**: worst retention over the five cells is
/// **0.73339** against the 0.70 bar the source sets. Recorded so a later change that erodes it to
/// 0.71 is visible as erosion rather than as a pass.
#[test]
fn gate3_headline_overspeed_survives_compensation() {
    for (tt4, v) in [(1500.0f64, 0.10f64), (1500.0, 0.20), (1300.0, 0.20),
                     (1100.0, 0.20), (1100.0, 0.30)] {
        let c = sb(0.0, 0.0, 0.0).compensated_point(&flight(), tt4, v, Spool::Lp);
        let k = c.comp.unwrap_or_else(|| panic!("no b* at Tt4={tt4}, v={v}: {:?}", c.reason));
        // the coordinate IS bought back
        assert!((k.phi_comp - c.phi_bare).abs() <= 1e-10);
        // the stator really spent something
        assert!(c.phi_stator < c.phi_bare - 1e-3);
        // the BILL is not
        assert!(k.dn_stator > 0.0 && k.dn_comp > 0.0);
        let retained = k.dn_comp / k.dn_stator;
        assert!(retained >= 0.70, "retention {retained:.3} at Tt4={tt4}, v={v}");
        // …and rung 42's own thrust bill is now on top of it
        assert!(k.d_f_comp < k.d_f_stator - 0.02);
    }
}

/// THE strongest single number: at `v = 0.30` the COMPENSATED machine overspeeds the
/// UNcompensated one, so undoing the lever is strictly worse than leaving it alone. The SIGN is
/// the claim, not the level.
#[test]
fn gate3_headline_crossover_compensation_is_strictly_worse() {
    let c = sb(0.0, 0.0, 0.0).compensated_point(&flight(), 1500.0, 0.30, Spool::Lp);
    let k = c.comp.expect("b* exists at v = 0.30");
    assert!(k.dn_comp > k.dn_stator,
            "crossover absent: comp {:+.5} vs stator {:+.5}", k.dn_comp, k.dn_stator);
}

// =========================================================================================
// GATE 4 — THE MECHANISM: the φ-debit was carrying a rebate
// =========================================================================================

/// The compensated point is MORE unloaded than the stator-only point, because the stator's
/// φ-drop raised `base(φ)` and restoring φ gives that rebate back.
#[test]
fn gate4_mechanism_compensation_forfeits_the_loading_rebate() {
    let (lp, _) = lp_hp();
    let m = sb(0.0, 0.0, 0.0);
    let (tt4, v) = (1500.0f64, 0.20f64);
    let bs = solved(&m.compensating_bleed(&flight(), tt4, v, Spool::Lp, Target::Phi));

    let cell = |vv: f64, bb: f64| {
        let sib = m.at_point(vv, 0.0, bb);
        let od = sib.core.core.match_point(&flight(), tt4);
        (od.phi_lp, sib.core.core.map_lp.psi(od.phi_lp))
    };
    let (bare, stator, comp) = (cell(0.0, 0.0), cell(v, 0.0), cell(v, bs));

    // the REBATE: base(φ) rises as the stator drops φ — the map's own loading law
    let base = |p: f64| 1.0 - lp.sigma * (p - 1.0).powi(2) - lp.l * (p - 1.0);
    assert!(base(stator.0) > base(bare.0) + 1e-3);
    // …and forfeiting it leaves the compensated point MORE unloaded than the stator alone
    assert!(comp.1 < stator.1 && stator.1 < bare.1,
            "psi ordering broken: comp {} stator {} bare {}", comp.1, stator.1, bare.1);
}

/// `psi_comp == base(φ_bare) − v(1+l)·φ_bare`. **A PLUMBING CHECK, NOT A FINDING** — it is `psi`
/// evaluated at a KNOWN argument, gated only to prove `at_point` composes the two levers onto one
/// map and one cascade correctly.
#[test]
fn gate4_psi_closed_form_is_a_plumbing_check() {
    for name in SHAPES {
        let (ml, _) = bare_shape(name);
        let m = shaped(name, 0.0, 0.0);
        let (tt4, v) = (1500.0f64, 0.20f64);
        let phi_b = m.at_point(0.0, 0.0, 0.0).core.core.match_point(&flight(), tt4).phi_lp;
        let bs = solved(&m.compensating_bleed(&flight(), tt4, v, Spool::Lp, Target::Phi));
        let sib = m.at_point(v, 0.0, bs);
        let psi_meas = sib.core.core.map_lp.psi(
            sib.core.core.match_point(&flight(), tt4).phi_lp);
        let base = 1.0 - ml.sigma * (phi_b - 1.0).powi(2) - ml.l * (phi_b - 1.0);
        assert!((psi_meas - (base - v * (1.0 + ml.l) * phi_b)).abs() <= 1e-10, "{name}");
    }
}

// =========================================================================================
// GATE 5 — RUNG 60's TAUTOLOGY, reached by a THIRD route (an identity, NOT a finding)
// =========================================================================================

/// Restoring φ (rather than PINNING it, as rung 60's floor does) hands back the SAME published
/// value: `dM_i = v` exactly. So rung 60's tautology needs no floor at all — only restoration.
#[test]
fn gate5_rung60_tautology_third_route() {
    for tt4 in [1500.0f64, 1300.0, 1100.0] {
        for v in [0.10f64, 0.20, 0.30] {
            let c = sb(0.0, 0.0, 0.0).compensated_point(&flight(), tt4, v, Spool::Lp);
            let k = c.comp.unwrap_or_else(|| panic!("no b* at Tt4={tt4}, v={v}"));
            assert!((k.d_m_i - v).abs() <= 1e-10, "dM_i at Tt4={tt4}, v={v}");
            assert!((k.d_m_phi - v * FLOOR * FLOOR / (1.0 + v * FLOOR)).abs() <= 1e-10,
                    "dM_phi at Tt4={tt4}, v={v}");
        }
    }
}

/// `dM_phi` takes the SAME value at every throttle (it is pure geometry), and both identities
/// hold on the flat-eta island where the maps carry no shaping at all.
#[test]
fn gate5_tautology_is_throttle_invariant_and_survives_flat_eta() {
    let vals: Vec<f64> = [1500.0f64, 1300.0, 1100.0].iter()
        .map(|&t| sb(0.0, 0.0, 0.0).compensated_point(&flight(), t, 0.20, Spool::Lp)
             .comp.expect("b*").d_m_phi)
        .collect();
    let (lo, hi) = (vals.iter().cloned().fold(f64::INFINITY, f64::min),
                    vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
    assert!(hi - lo <= 1e-10, "{vals:?}");
    let c = shaped("flat-eta", 0.0, 0.0)
        .compensated_point(&flight(), 1500.0, 0.20, Spool::Lp).comp.expect("b*");
    assert!((c.d_m_i - 0.20).abs() <= 1e-10);
    assert!((c.d_m_phi - vals[0]).abs() <= 1e-10);
}

// =========================================================================================
// GATE 6 — THE SEAM AS POSED, REFUTED
// =========================================================================================

/// *"The bleed takes over where the stator's authority ends"* predicts the ceiling is INDIFFERENT
/// to the valve. It is not: the valve PRE-SPENDS the incidence budget, so the stator's remaining
/// authority SHRINKS. **This rung's own prediction had the sign the other way** and is scored a
/// miss in the anchor.
#[test]
fn gate6_seam_as_posed_valve_shrinks_the_stators_authority() {
    for tt4 in [1500.0f64, 1000.0] {
        let rows = sb(0.0, 0.0, 0.0)
            .authority_with_bleed(&flight(), tt4, &[0.0, 0.05, 0.10, 0.15], Spool::Lp);
        let edges: Vec<f64> = rows.iter().map(|r| r.v_edge).collect();
        let spans: Vec<f64> = rows.iter().map(|r| r.span).collect();
        let zeros: Vec<f64> = rows.iter().map(|r| r.m_i_0).collect();
        assert!(edges.windows(2).all(|w| w[0] >= w[1]) && edges[0] > edges[edges.len() - 1],
                "Tt4={tt4}: {edges:?}");
        assert!(spans.windows(2).all(|w| w[0] > w[1]), "Tt4={tt4}: {spans:?}");
        // the valve PRE-SPENDS
        assert!(zeros.windows(2).all(|w| w[0] < w[1]), "Tt4={tt4}: {zeros:?}");
    }
}

/// The load-bearing version of the seam's refutation — **no reliance on rung 53's `solve_n`
/// artifact edge.** The two levers are SUBSTITUTES on one incidence budget: the four-cell
/// interaction is under 3 % of the credit sum on every shape.
///
/// **MEASURED WORST: 0.01686 against the 0.03 bar** — a factor of 1.8, so the bar is real but not
/// tight. Recorded rather than tightened: 0.03 is the SOURCE's number and re-fitting a bar to the
/// port's own measurement is how a gate stops testing the claim it names.
#[test]
fn gate6_credits_superpose_artifact_free() {
    for name in SHAPES {
        let m = shaped(name, 0.0, 0.0);
        for tt4 in [1500.0f64, 1200.0] {
            for (v, b) in [(0.10f64, 0.05f64), (0.20, 0.10)] {
                let m_i = |vv: f64, bb: f64| {
                    m.at_point(vv, 0.0, bb).core.stator_margin(&flight(), tt4).lp.m_i
                };
                let base = m_i(0.0, 0.0);
                let (cs, cb) = (m_i(v, 0.0) - base, m_i(0.0, b) - base);
                let inter = (m_i(v, b) - base) - cs - cb;
                assert!(cs > 0.0 && cb > 0.0, "{name}: credits must both be positive");
                assert!(inter.abs() / (cs + cb) < 0.03,
                        "{name} {tt4} {v},{b}: {inter:+.5}");
            }
        }
    }
}

// =========================================================================================
// GATE 7 — SPOOL-DEPENDENCE: the two levers do not span the same space
// =========================================================================================

/// `b*_LP` exists at every throttle; `b*_HP` at NONE of them. Rung 53's stator acts on either
/// spool, rung 42's valve on one, so a stator debit is compensable only where the two overlap.
#[test]
fn gate7_compensability_is_spool_dependent() {
    let rows = sb(0.0, 0.0, 0.0)
        .compensability(&flight(), &[1500.0, 1300.0, 1100.0, 900.0], 0.20);
    assert!(rows.len() >= 4);
    assert!(rows.iter().all(|r| matches!(r.b_lp, Some(b) if b > 0.0 && b < 0.45)));
    assert!(rows.iter().all(|r| r.b_hp.is_none()));
    assert!(rows.iter().all(|r| r.why_hp == Some("valve authority exhausted (b >= cap)")));
    // b*_LP falls monotonically as power falls — the LP branch is well-behaved exactly where
    // the HP one does not exist.
    let bs: Vec<f64> = rows.iter().map(|r| r.b_lp.unwrap()).collect();
    assert!(bs.windows(2).all(|w| w[0] > w[1]), "{bs:?}");
}

/// The anchor predicted a DIVERGENCE toward `π*`. Measured: uniformly unavailable, by a
/// throttle-INVARIANT shortfall. The mechanism was right, the shape was wrong — asserted so the
/// corrected statement is the gated one.
#[test]
fn gate7_hp_shortfall_is_throttle_invariant_not_a_pi_star_divergence() {
    let m = sb(0.0, 0.0, 0.0);
    let mut short = Vec::new();
    for tt4 in [1500.0f64, 1300.0, 1100.0, 900.0] {
        let c = m.compensating_bleed(&flight(), tt4, 0.20, Spool::Hp, Target::Phi);
        assert!(c.b_star().is_none(), "the HP spool must NOT compensate at Tt4={tt4}");
        let resid_last = c.resid_last().expect("the exhausted branch carries resid_last");
        let spent = m.at_point(0.0, 0.20, 0.0).core.stator_margin(&flight(), tt4).hp.phi_op
            - c.goal();
        let returned = resid_last - spent;
        assert!(spent < 0.0 && returned > 0.0, "Tt4={tt4}: spent {spent} returned {returned}");
        assert!(spent.abs() > 3.0 * returned, "short by <3x at Tt4={tt4}");
        short.push(resid_last.abs());
    }
    let mean = short.iter().sum::<f64>() / short.len() as f64;
    let (lo, hi) = (short.iter().cloned().fold(f64::INFINITY, f64::min),
                    short.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
    assert!((hi - lo) / mean < 0.10, "shortfall is not throttle-invariant: {short:?}");
}

// =========================================================================================
// GATE 8 — THE PRICE COLLAPSE, and the control that keeps it honest
// =========================================================================================

/// `b*/[v(1+l)]` is the same number across five shapes whose `l` spans 0.7 → 1.2, so the price's
/// ENTIRE shape-dependence is the map's own loading slope. The coefficient itself rides on `v`
/// and the throttle and is disclaimed.
#[test]
fn gate8_price_collapses_on_the_loading_slope() {
    for tt4 in [1500.0f64, 1200.0] {
        let mut vals = Vec::new();
        for name in SHAPES {
            let l = bare_shape(name).0.l;
            let bs = solved(&shaped(name, 0.0, 0.0)
                .compensating_bleed(&flight(), tt4, 0.20, Spool::Lp, Target::Phi));
            vals.push(bs / (0.20 * (1.0 + l)));
        }
        let mean = vals.iter().sum::<f64>() / vals.len() as f64;
        let (lo, hi) = (vals.iter().cloned().fold(f64::INFINITY, f64::min),
                        vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
        let spread = (hi - lo) / mean;
        assert!(spread < 0.05, "Tt4={tt4}: {vals:?} spread {spread:.3}");
    }
}

/// **THE NEGATIVE CONTROL.** The compensable range looked like a derived ceiling scaling as
/// `1/(1+l)`. It is not: `_B_CAP` is this rung's OWN constant, and moving it moves the ceiling.
/// Gated so the un-publishable claim cannot creep back in.
///
/// This is the gate that needs [`StatorBleedCore::with_b_cap`] — the per-instance rebind of a
/// class constant Python spells `m._B_CAP = cap`.
#[test]
fn gate8_the_ceiling_is_cap_dependent_and_therefore_not_claimed() {
    let last_ok = |cap: f64| {
        let m = sb(0.0, 0.0, 0.0).with_b_cap(cap);
        let (mut ok, mut v) = (0.0f64, 0.10f64);
        while v < 0.8 {
            if m.compensating_bleed(&flight(), 1500.0, v, Spool::Lp, Target::Phi)
                .b_star().is_none() {
                return ok;
            }
            ok = v;
            v = ((v + 0.05) * 10_000.0).round() / 10_000.0;   // Python's round(v + 0.05, 4)
        }
        ok
    };
    let (lo, hi) = (last_ok(0.35), last_ok(0.45));
    assert!(hi > lo, "the ceiling does not track the cap: {lo} vs {hi}");
}

/// *"Restore the point"* and *"restore the reported margin"* are different instructions — the
/// stator moved the floor between them. The gap grows with `v` and is throttle-INVARIANT while
/// each price separately moves a lot.
#[test]
fn gate8_price_split_two_loci() {
    let mut gaps: Vec<(f64, Vec<f64>)> = Vec::new();
    for tt4 in [1500.0f64, 1200.0] {
        let rows = sb(0.0, 0.0, 0.0)
            .price_split(&flight(), tt4, &[0.10, 0.20, 0.30], Spool::Lp);
        for r in &rows {
            let (bp, bm) = (r.b_phi.expect("b_phi"), r.b_m_phi.expect("b_m_phi"));
            assert!(bp > bm && bm > 0.0, "Tt4={tt4} v={}: {bp} vs {bm}", r.vsv);
        }
        let g: Vec<f64> = rows.iter().map(|r| r.gap.expect("gap")).collect();
        assert!(g.windows(2).all(|w| w[0] < w[1]), "gap does not grow with v: {g:?}");
        gaps.push((tt4, g));
    }
    for (a, b) in gaps[0].1.iter().zip(gaps[1].1.iter()) {
        assert!((a - b).abs() / a < 0.02, "gap is not throttle-invariant: {a} vs {b}");
    }
}

// =========================================================================================
// GATE 9 — RUNG 53 CORRECTED, with its own control; and the cost machine-zero
// =========================================================================================

/// Rung 53's P5: `vsv_lp` leaves `phi_HP` BIT-IDENTICAL, and its inter-spool arrow is
/// η-mediated so a flat-eta island switches it off. Both still hold for the lever ALONE
/// (reproduced here, so the correction is NOT vacuous) — and neither survives the pair, because
/// the only lever that buys the LP debit back reaches the HP through the shared `Tt25` ENERGY
/// channel, which no flat map can switch off.
#[test]
fn gate9_rung53_per_spool_cleanliness_lost_under_composition() {
    for name in ["flow/press", "flat-eta"] {
        let m = shaped(name, 0.0, 0.0);
        let (tt4, v) = (1500.0f64, 0.20f64);
        let r0 = m.at_point(0.0, 0.0, 0.0).core.stator_margin(&flight(), tt4).hp.phi_op;
        let rv = m.at_point(v, 0.0, 0.0).core.stator_margin(&flight(), tt4).hp.phi_op;
        // THE CONTROL — rung 53's exact zero, reproduced on the bit
        assert_eq!(rv.to_bits(), r0.to_bits(), "{name}: rung 53's exact zero is gone");
        let c = m.compensated_point(&flight(), tt4, v, Spool::Lp).comp.expect("b*");
        // …broken by the PAIR, even flat
        assert!(c.d_phi_other_comp.abs() > 1e-3, "{name}");
    }
}

/// **RUNG 53's P1 SHARPENED, and the control that stopped this rung overclaiming.** Rung 53
/// reported the stator thrust-neutral as a TOLERANCE. With the efficiency island switched off it
/// is a MACHINE ZERO: the stator's own thrust effect is exactly `0.0` while `n` moves > 4 %. So
/// the whole of the stator's thrust cost is the η island, and it is a PURE speed lever there.
#[test]
fn gate9_rung53_p1_thrust_neutrality_is_exact_on_a_flat_eta_island() {
    let flat = shaped("flat-eta", 0.0, 0.0);
    for tt4 in [1500.0f64, 1200.0] {
        let f00 = flat.at_point(0.0, 0.0, 0.0).core.core.match_point(&flight(), tt4);
        for v in [0.10f64, 0.20, 0.30] {
            let od = flat.at_point(v, 0.0, 0.0).core.core.match_point(&flight(), tt4);
            assert_eq!(od.base.thrust.to_bits(), f00.base.thrust.to_bits(),
                       "v={v} at Tt4={tt4}: thrust is not a MACHINE zero");
            assert!(od.n_lp / f00.n_lp - 1.0 > 0.04, "the speed bill vanished at v={v}");
        }
    }
}

/// The real cost interaction: the pair always costs MORE shaft speed than the sum of its parts —
/// including on the flat-eta island, where the thrust interaction is trivially zero but the speed
/// one is not.
#[test]
fn gate9_cost_interaction_speed_is_adverse_everywhere() {
    let flat = shaped("flat-eta", 0.0, 0.0);
    for tt4 in [1500.0f64, 1200.0] {
        for (v, b) in [(0.10f64, 0.05f64), (0.20, 0.10)] {
            let cell = |vv: f64, bb: f64| {
                let od = flat.at_point(vv, 0.0, bb).core.core.match_point(&flight(), tt4);
                (od.base.thrust, od.n_lp)
            };
            let ((f00, n00), (fv0, nv0)) = (cell(0.0, 0.0), cell(v, 0.0));
            let ((f0b, n0b), (fvb, nvb)) = (cell(0.0, b), cell(v, b));
            let i_f = (fvb / f00) - (fv0 / f00) - (f0b / f00) + 1.0;
            let i_n = (nvb / n00) - (nv0 / n00) - (n0b / n00) + 1.0;
            assert_eq!(i_f, 0.0, "a COROLLARY of the gate above, not a claim");
            assert!(i_n > 1e-4, "{i_n}");
        }
    }
    let sh = shaped("flow/press", 0.0, 0.0);
    for (v, b) in [(0.10f64, 0.05f64), (0.20, 0.10), (0.30, 0.10)] {
        let cell = |vv: f64, bb: f64| {
            let od = sh.at_point(vv, 0.0, bb).core.core.match_point(&flight(), 1500.0);
            (od.base.thrust, od.n_lp)
        };
        let ((f00, n00), (fv0, nv0)) = (cell(0.0, 0.0), cell(v, 0.0));
        let ((f0b, n0b), (fvb, nvb)) = (cell(0.0, b), cell(v, b));
        assert!((fvb / f00) - (fv0 / f00) - (f0b / f00) + 1.0 > 0.0);
        assert!((nvb / n00) - (nv0 / n00) - (n0b / n00) + 1.0 > 0.0);
    }
}

// =========================================================================================
// GATE 10 — CYCLE UNTOUCHED
// =========================================================================================

/// The default single-spool design run never sees a stator or a valve.
#[test]
fn gate10_cycle_untouched_bit_for_bit_rung6() {
    // Python filters `REAL` to the five keys `build_turbojet` shares — no `eta_c`/`eta_t`, which
    // is why this differs from rung 53's otherwise identical gate.
    let eng = build_turbojet(Gas::reacting_equilibrium(), 10.0, 1600.0, 50_000.0,
                             Losses { pi_d: 0.97, eta_b: 0.99, pi_b: 0.96, eta_m: 0.99,
                                      pi_n: 0.98, ..Losses::default() });
    let r = eng.run(&flight(), 1.0);
    assert!(r.performance.specific_thrust > 0.0 && r.performance.tsfc > 0.0);
    let r2 = eng.run(&flight(), 1.0);
    assert_eq!(r2.performance.specific_thrust.to_bits(),
               r.performance.specific_thrust.to_bits());
}

// =========================================================================================
// THE LEDGER — what slice O did NOT port, and why
// =========================================================================================

/// **Every deferral slice O leaves behind, in one place.** The `slice_j` / `slice_l` / `slice_m` /
/// `slice_n` precedent, fifth use: a deferral recorded in a test is findable; one recorded in a
/// commit message is not.
#[test]
fn slice_o_deferrals() {
    // 1. `lp_disabled`. Python's rung-61 constructor takes it and asserts
    //    `not (lp_disabled and bleed != 0.0)`. UNREPRESENTABLE here: `VariableStatorCore` holds a
    //    `TwoSpoolMapCore` directly, not the degenerate-or-not enum, so there is no lp_disabled
    //    rung-61 object for the guard to reject. § 5.11's P7 — held.
    //
    // 2. `assert target in ("phi", "m_phi")` and `assert spool in self._SPOOLS`. Both
    //    unrepresentable: `Target` and `Spool` are enums. Slice N's `Split`/`CapProfile`
    //    precedent, third use.
    //
    // 3. `isinstance(sib, StatorBleedMatcher)` in Python's trap gate. A sibling from
    //    `at_setting` is a `VariableStatorCore` in Rust whichever table built it, so an
    //    `isinstance` port would compare a type to itself. Replaced — not dropped — by the
    //    TABLE-POINTER and carried-state assertions in
    //    `gate2_trap_at_setting_carries_the_bleed`, which is strictly stronger.
    //
    // 4. Python's `@pytest.mark.slow` on 14 of the 18 gates. NOT ported: the crate has no slow
    //    tier and the whole rung 61 suite runs in seconds. Slice M's rule, unchanged — port the
    //    gate, drop the marker, and re-introduce `#[ignore]` only against a MEASURED cost.
    //
    // 5. `stator_margin`'s floor assert (`phi_s < phi_op`). Python's `_feasible` would SWALLOW
    //    it; Rust's `try_stator_margin` makes only the MATCH fallible, so it stays a panic.
    //    Measured absent: 0 firings in slice M's 560 calls (§ 5.9 (iii)), and slice O's
    //    1 760-cell wide sweep found only TWO refusal classes in 756 refusals — the speed-line
    //    bracket and the choked envelope — neither of them the floor. Recorded, not hidden.
    //
    // 6. **DISCHARGED, NOT DEFERRED:** slice L's `lp_eta_loop_bleed` panic site. See
    //    `slice_o_oracle.rs`'s `lp_bleed_aborts` bar and `bleed.rs`'s module note.
}

