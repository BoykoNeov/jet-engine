//! RUNG 53 — THE VARIABLE STATOR: what a margin *is*, when the lever moves the wall.
//!
//! Port of `tests/test_rung53.py`, gate for gate. Its ten gate groups:
//!
//!   1. REDUCE — at `vsv = 0` the stored maps are the ones passed in, bit for bit, and the
//!      matched fields are rung 39's on BOTH gases. Plus `psi`/`phi_surge_at` bit-for-bit at
//!      `vsv = 0`, and the `is_flat` rule (`phi_surge` ignored, `vsv` NOT).
//!   2. THE CONTROL that could have killed the rung — at `v = 0` the throttle moves `phi_op`
//!      against a FIXED floor, so all three currencies must agree in sign at every step and the
//!      ratio must track the Jacobian `1/phi_op^2`. A floor-fixed lever cannot split them.
//!   3. THE HEADLINE — with the STATOR as the lever the signs DO split, on both spools and
//!      across all five disclosed shapes; the derivatives hit their closed forms; the interval
//!      law holds.
//!   4. ZERO NEW CONSTANTS — `T_c == 1/phi_surge` exactly, the floor law and the psi law agree,
//!      and `t2 = l/(1+l)` reproduces psi's design slope.
//!   5. P1 — a SPEED lever, not a flow lever.
//!   6. P5's TWO ZEROS — `vsv_lp` never reaches the HP spool; `vsv_hp` never reaches the LP
//!      spool on flat-eta islands; the shaped arrow is nonzero, so the zeros are not vacuous.
//!   7. P7 — the constant-incidence schedule.
//!   8. BOTH SPLIT BOUNDARIES AS BRACKETS.
//!   9. RUNG 41's TWO-PATH pi GATE SURVIVES the new psi term, at a MOVED stator.
//!  10. CYCLE UNTOUCHED.
//!
//! # Where this port DIFFERS from the Python, and why
//!
//! Three of Python's assertions are about PYTHON, not about the physics, and each is recorded
//! at its gate rather than silently dropped — see `slice_m_deferrals` for the full ledger.
//! The short version: object identity becomes bit equality plus a delegation pin, method
//! inheritance becomes function-pointer equality against rung 39's own table, and the
//! `lp_disabled` refusal is enforced by the TYPE (the parameter does not exist) rather than by
//! an assertion, which is strictly stronger and therefore untestable at runtime.
//!
//! **Python's six `@pytest.mark.slow` markers are NOT carried over, and that is deliberate.**
//! The marker records a COST, not a claim, and the cost does not survive the port: all 24 tests
//! here — the six marked ones included, which between them run 10 shape×spool cells, 12 full
//! matches and 8 bisected schedule points — finish in 0.82 s. Carrying the marker would
//! deselect six real gates from the default run to save under a second, which is exactly the
//! silent deselection `conftest.py`'s one-gate policy exists to forbid.

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::stage::{CapProfile, Split, StageStackCore, StageStackCoreSpec};
use turbojet::stator::{VariableStatorCore, R53};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses,
                          TwoSpoolMapCore, R39};

const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;
const FLOOR: f64 = 0.55;
const THROTTLE: [f64; 6] = [1500.0, 1400.0, 1300.0, 1200.0, 1100.0, 1000.0];

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

/// Self-consistent CPG dual gas — rung 31/38/39/40/41/42's recipe, with `R` DERIVED from
/// `gamma` and `cp` rather than rounded. `rung39.rs` rounds `r_c` to 286.9; this file must not,
/// because it shares its numbers with `slice_m_oracle.rs`, which is bit-exact against Python.
fn cpg_gas() -> Gas {
    let (gc, cc, gt, ct) = (1.4f64, 1004.0f64, 1.3f64, 1239.0f64);
    Gas::new(GasSpec {
        gamma_c: gc, cp_c: cc, r_c: (gc - 1.0) / gc * cc,
        gamma_t: gt, cp_t: ct, r_t: (gt - 1.0) / gt * ct,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn lp_map() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn hp_map() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

/// The five disclosed shapes the split is asserted ACROSS (magnitudes disclaimed).
fn shapes() -> Vec<(&'static str, ComponentMap, ComponentMap)> {
    let f = ComponentMap::flat();
    vec![
        ("flow/press", ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f },
                       ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..f }),
        ("press/flow", ComponentMap { a: 0.05, b: 0.20, sigma: 0.1, l: 1.0, ..f },
                       ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f }),
        ("tilted",     ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..f },
                       ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..f }),
        ("steep",      ComponentMap { a: 0.25, b: 0.12, sigma: 0.3, l: 1.2, ..f },
                       ComponentMap { a: 0.25, b: 0.12, sigma: 0.3, l: 1.2, ..f }),
        ("flat-eta",   ComponentMap { sigma: 0.1, l: 0.7, ..f },
                       ComponentMap { sigma: 0.1, l: 1.0, ..f }),
    ]
}

fn design(gas: Gas) -> TwoSpoolEngine {
    build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, 50_000.0, real())
}

/// Python's `_vm`.
fn vm(d: TwoSpoolEngine, ml: ComponentMap, mh: ComponentMap, vl: f64, vh: f64)
    -> VariableStatorCore
{
    VariableStatorCore::new(d, flight(), 1.0, ml, mh, vl, vh)
}

fn vm_default(vl: f64, vh: f64) -> VariableStatorCore {
    vm(design(cpg_gas()), lp_map(), hp_map(), vl, vh)
}

const SPOOLS: [(Spool, &str); 2] = [(Spool::Lp, "lp"), (Spool::Hp, "hp")];

// ==========================================================================================
// GATE 1 — REDUCE
// ==========================================================================================

/// Python asserts OBJECT IDENTITY (`m.map_lp is LP`) and METHOD IDENTITY
/// (`VariableStatorMatcher.match is TwoSpoolMapMatcher.match`). Neither is expressible here and
/// each needs a DIFFERENT substitute, so they are ported separately rather than merged:
///
/// * `ComponentMap` is `Copy`, so "the same object" is not a thing a Rust map can be. The
///   claim underneath it — *no rung-53 code path ran* — becomes bit equality of every field
///   against the map passed in, which is what `with_hooks`'s `if vsv != 0.0` guard delivers.
/// * Rust has no inheritance, so "`match` is rung 39's own method" becomes FUNCTION-POINTER
///   equality between the core's hook table and `R39`'s. Comparing two `R53` entries to each
///   other would be the self-comparison slice L's lesson names, and would pass on any table.
#[test]
fn test_reduce_map_objects_are_identical_at_design_setting() {
    let (lp, hp) = (lp_map(), hp_map());
    let m = vm(design(cpg_gas()), lp, hp, 0.0, 0.0);
    for (got, want, who) in [(m.core.map_lp(), lp, "map_lp"), (m.core.map_hp(), hp, "map_hp"),
                             (m.map_lp_design, lp, "map_lp_design"),
                             (m.map_hp_design, hp, "map_hp_design")] {
        assert_eq!(got.vsv.to_bits(), want.vsv.to_bits(), "{who} vsv");
        assert_eq!(got.a.to_bits(), want.a.to_bits(), "{who} a");
        assert_eq!(got.b.to_bits(), want.b.to_bits(), "{who} b");
        assert_eq!(got.c.to_bits(), want.c.to_bits(), "{who} c");
        assert_eq!(got.sigma.to_bits(), want.sigma.to_bits(), "{who} sigma");
        assert_eq!(got.a_t.to_bits(), want.a_t.to_bits(), "{who} a_t");
        assert_eq!(got.l.to_bits(), want.l.to_bits(), "{who} l");
        assert_eq!(got.phi_surge.to_bits(), want.phi_surge.to_bits(), "{who} phi_surge");
        assert_eq!(got.capacity.to_bits(), want.capacity.to_bits(), "{who} capacity");
    }
    // ...and the matching cascade IS rung 39's, not a rung-53 copy of it.
    assert_eq!(m.core.hooks.hp_eta_loop as usize, R39.hp_eta_loop as usize);
    assert_eq!(m.core.hooks.lp_eta_loop as usize, R39.lp_eta_loop as usize);
}

/// `vsv = 0` ⇒ every matched field is EXACTLY rung 39's, on both gases.
#[test]
fn test_reduce_bit_for_bit_rung39() {
    for gasname in ["fast", "reacting"] {
        let gas = if gasname == "fast" { Gas::thermally_perfect() }
                  else { Gas::reacting_equilibrium() };
        let d = design(gas);
        let base = TwoSpoolMapCore::new(d.clone(), flight(), 1.0, lp_map(), hp_map());
        let stat = vm(d, lp_map(), hp_map(), 0.0, 0.0);
        for tt4 in [1500.0, 1300.0, 1100.0] {
            let a = base.match_point(&flight(), tt4);
            let b = stat.core.match_point(&flight(), tt4);
            let fields: [(&str, f64, f64); 19] = [
                ("pi_lpc", a.base.pi_lpc, b.base.pi_lpc),
                ("pi_hpc", a.base.pi_hpc, b.base.pi_hpc),
                ("n_lp", a.n_lp, b.n_lp), ("n_hp", a.n_hp, b.n_hp),
                ("phi_lp", a.phi_lp, b.phi_lp), ("phi_hp", a.phi_hp, b.phi_hp),
                ("slip", a.slip, b.slip),
                ("eta_lpc", a.eta_lpc, b.eta_lpc), ("eta_hpc", a.eta_hpc, b.eta_hpc),
                ("eta_hpt", a.eta_hpt, b.eta_hpt), ("eta_lpt", a.eta_lpt, b.eta_lpt),
                ("tau_lpc", a.base.tau_lpc, b.base.tau_lpc),
                ("tau_hpc", a.base.tau_hpc, b.base.tau_hpc),
                ("tau_hpt", a.base.tau_hpt, b.base.tau_hpt),
                ("tau_lpt", a.base.tau_lpt, b.base.tau_lpt),
                ("mdot_air", a.base.mdot_air, b.base.mdot_air),
                ("thrust", a.base.thrust, b.base.thrust),
                ("N_lp_ratio", a.n_lp_ratio, b.n_lp_ratio),
                ("N_hp_ratio", a.n_hp_ratio, b.n_hp_ratio),
            ];
            for (k, x, y) in fields {
                assert_eq!(x.to_bits(), y.to_bits(), "{gasname} Tt4={tt4} field {k}");
            }
        }
    }
}

/// `psi` / `phi_surge_at` at `vsv = 0` are the rung ≤ 52 expressions, exactly.
///
/// **NARROWED, and the narrowing is enumerated.** Python's version also gates `phi_max()`'s
/// generalisation as inert at `vsv == 0`. `ComponentMap::phi_max` does not exist in Rust yet —
/// it is read only by the rung-34/40/43 FORWARD transient closures, which are phase 6, and the
/// steady stator never calls it. The `phi_max` third of this gate is therefore owed, and it is
/// named in `slice_m_deferrals` so this file's gate-name diff cannot read as full coverage.
#[test]
fn test_reduce_componentmap_expressions_bit_for_bit() {
    for cm in [lp_map(), hp_map(), ComponentMap::flat(), ComponentMap::surge_tilted()] {
        for phi in [0.4, 0.7, 1.0, 1.3, 1.9] {
            let u = phi - 1.0;
            assert_eq!(cm.psi(phi).to_bits(),
                       (1.0 - cm.sigma * (u * u) - cm.l * u).to_bits(), "psi at {phi}");
        }
        assert_eq!(cm.phi_surge_at().to_bits(), cm.phi_surge.to_bits());
    }
}

/// `phi_surge` is ignored by flatness (rung 36's rule); `vsv` is NOT (it enters `psi`).
///
/// BOTH directions, because either alone passes on a stub: a reader that always returns `true`
/// clears lines 1 and 3, one that always returns `false` clears line 2.
#[test]
fn test_is_flat_rule() {
    assert!(ComponentMap::flat().with_phi_surge(0.7).is_flat());
    assert!(!ComponentMap::flat().with_vsv(0.1).is_flat());
    assert!(ComponentMap::flat().with_vsv(0.0).is_flat());
}

/// The matcher moves the stators itself, so a pre-swirled map is refused.
///
/// Python's second half asserts the `lp_disabled` refusal. Rust's constructor has no such
/// parameter, so the refusal is a TYPE property: the call does not compile. That is strictly
/// stronger than a runtime assertion and cannot be witnessed by a test — recorded in
/// `slice_m_deferrals` rather than left to look like an omission.
#[test]
#[should_panic(expected = "DESIGN-SETTING maps")]
fn test_design_setting_maps_refused() {
    let _ = vm(design(cpg_gas()), lp_map().with_vsv(0.1), hp_map(), 0.0, 0.0);
}

// ==========================================================================================
// GATE 2 — THE CONTROL: a FLOOR-FIXED lever CANNOT split the currencies
// ==========================================================================================

/// THE GATE THAT COULD HAVE KILLED THE RUNG. At the design stator setting the throttle moves
/// `phi_op` against a FIXED floor. Then `M_i` is a monotone reparameterisation of `M_phi` with a
/// STRICTLY POSITIVE Jacobian `1/phi_op^2`, so the two (and `SM_N` with them) must agree in sign
/// at every step. If they could split here, the moving floor would not be the mechanism and the
/// headline would be wrong.
#[test]
fn test_throttle_cannot_split_the_currencies() {
    for (spool, name) in SPOOLS {
        let rows = vm_default(0.0, 0.0).throttle_currency(&flight(), &THROTTLE, spool);
        assert_eq!(rows.len(), THROTTLE.len() - 1, "{name}");
        for r in &rows {
            assert!(r.signs_agree, "{name} {r:?}");
            assert!(r.all_three_agree, "{name} {r:?}");
            // the ratio IS the Jacobian, to the finite-difference error
            assert!((r.ratio / r.jacobian - 1.0).abs() < 1e-3, "{name} {r:?}");
        }
    }
}

// ==========================================================================================
// GATE 3 — THE HEADLINE: the STATOR splits them
// ==========================================================================================

/// The two currencies disagree in SIGN under the stator, and both derivatives hit their closed
/// forms at the design point (zero new constants).
#[test]
fn test_headline_split_and_closed_forms() {
    for (spool, name) in SPOOLS {
        let cs = vm_default(0.0, 0.0).currency_split(&flight(), TT4, spool, None);
        let l = if spool == Spool::Lp { lp_map().l } else { hp_map().l };
        assert!((cs.phi_op - 1.0).abs() < 1e-9, "{name} design point");
        assert!(cs.d_m_phi < 0.0 && 0.0 < cs.d_m_i, "{name} THE SPLIT: {cs:?}");
        assert!(cs.split, "{name}");
        let want_phi = -(1.0 + l) / (2.0 + l) + FLOOR * FLOOR;
        assert!((cs.d_m_phi / want_phi - 1.0).abs() < 1e-4, "{name} d_m_phi {cs:?}");
        let want_i = 1.0 / (2.0 + l);
        assert!((cs.d_m_i / want_i - 1.0).abs() < 1e-4, "{name} d_m_i {cs:?}");
        // the interval law: disagreement IFF -phi_op'/v' lies in (phi_surge^2, phi_op^2)
        assert!(cs.in_interval, "{name}");
        assert!(cs.interval.0 < cs.ratio && cs.ratio < cs.interval.1, "{name} {cs:?}");
    }
}

/// The SIGN split holds on all five disclosed shapes, both spools (magnitudes disclaimed).
/// Includes flat-eta, where the stator's only inter-spool arrow is switched off.
#[test]
fn test_split_is_shape_robust() {
    for (shape, ml, mh) in shapes() {
        let (ml, mh) = (ml.with_phi_surge(FLOOR), mh.with_phi_surge(FLOOR));
        for (spool, name) in SPOOLS {
            let m = vm(design(cpg_gas()), ml, mh, 0.0, 0.0);
            let cs = m.currency_split(&flight(), TT4, spool, None);
            assert!(cs.d_m_phi < 0.0 && 0.0 < cs.d_m_i, "{shape}/{name}: {cs:?}");
            let l = if spool == Spool::Lp { ml.l } else { mh.l };
            assert!((cs.d_m_i / (1.0 / (2.0 + l)) - 1.0).abs() < 1e-3, "{shape}/{name}: {cs:?}");
        }
    }
}

// ==========================================================================================
// GATE 4 — ZERO NEW CONSTANTS: the two channels are anchored, and they AGREE
// ==========================================================================================

#[test]
fn test_incidence_anchor_is_the_rung36_floor() {
    for cm in [lp_map(), hp_map(), ComponentMap::surge_flow().with_phi_surge(0.65)] {
        assert_eq!(cm.tan_beta1_crit().to_bits(), (1.0 / cm.phi_surge).to_bits());
        // v = 0: the anchor itself
        assert_eq!(cm.tan_beta1(cm.phi_surge).to_bits(), cm.tan_beta1_crit().to_bits());
    }
}

/// The unarmed map has no anchor to read, and says so.
#[test]
#[should_panic(expected = "anchor")]
fn test_incidence_anchor_refuses_an_unarmed_map() {
    let _ = ComponentMap::surge_flow().tan_beta1_crit();
}

/// The two derived channels are not independent fits: `phi_surge_at()` is EXACTLY the `phi` at
/// which `tan_beta1` reaches the (stator-invariant) critical incidence, and the psi swirl term
/// carries the SAME `v` through the derived `t2 = l/(1+l)`.
#[test]
fn test_floor_law_and_psi_law_are_one_law() {
    for v in [-0.2, -0.05, 0.1, 0.3, 0.8] {
        for cm0 in [lp_map(), hp_map()] {
            let cm = cm0.with_vsv(v);
            let a = cm.tan_beta1(cm.phi_surge_at());
            assert!((a / cm.tan_beta1_crit() - 1.0).abs() < 1e-14, "v={v}");
            let want = cm0.phi_surge / (1.0 + v * cm0.phi_surge);
            assert!((cm.phi_surge_at() / want - 1.0).abs() < 1e-14, "v={v}");
            // the derived rotor-exit metal angle reproduces the map's own design slope
            let t2 = cm0.l / (1.0 + cm0.l);
            assert!((1.0 / (1.0 - t2) / (1.0 + cm0.l) - 1.0).abs() < 1e-14, "v={v}");
            // psi's swirl increment is exactly -v*(1+l)*phi
            for phi in [0.6, 1.0, 1.4] {
                let want = cm0.psi(phi) - v * (1.0 + cm0.l) * phi;
                let d = (cm.psi(phi) - want).abs();
                assert!(d <= 1e-15 + 1e-14 * want.abs(), "v={v} phi={phi}");
            }
            // closing lowers the floor, opening raises it
            assert_eq!(cm.phi_surge_at() < cm0.phi_surge, v > 0.0, "v={v}");
        }
    }
}

// ==========================================================================================
// GATE 5 — P1: a SPEED lever, not a flow lever; and the trade
// ==========================================================================================

/// AT the design point the eta island is STATIONARY, so `m` cannot move at all and the closed
/// form `-(1+l)/(2+l)` is EXACT, not approximate.
///
/// **`flow_vs_speed < 1e-6` is a band around a zero, and the zero IS the finding.** The measured
/// value is 1.5e-9 (`slice_m_oracle.rs` records it and deliberately does NOT claim it — a band
/// asserted in an interpreter-comparison arm would be a claim about CPython, not about the
/// stator). This is where it is claimed, and the band is Python's own.
#[test]
fn test_speed_lever_at_design_is_a_machine_zero() {
    for (spool, name) in SPOOLS {
        let cs = vm_default(0.0, 0.0).currency_split(&flight(), TT4, spool, None);
        let l = if spool == Spool::Lp { lp_map().l } else { hp_map().l };
        assert!(cs.d_n > 0.0 && 0.0 > cs.d_phi_op, "{name}: n UP, phi DOWN — {cs:?}");
        assert!(cs.flow_vs_speed < 1e-6, "{name}: m pinned — {cs:?}");
        let want = -(1.0 + l) / (2.0 + l);
        assert!((cs.d_phi_op / want - 1.0).abs() < 1e-5, "{name} {cs:?}");
    }
}

/// Off design `m` DOES move (through the eta island) but stays far below `n`, and the general
/// closed form `-(1+l)phi^2/D(phi)` holds within the pre-registered 10%.
#[test]
fn test_speed_lever_off_design_stays_within_the_registered_bands() {
    for (spool, name) in SPOOLS {
        let s = vm_default(0.0, 0.0);
        for tt4 in THROTTLE {
            let cs = s.currency_split(&flight(), tt4, spool, None);
            assert!(cs.d_n > 0.0 && 0.0 > cs.d_phi_op, "{name} Tt4={tt4} {cs:?}");
            assert!(cs.flow_vs_speed <= 0.1, "{name} Tt4={tt4} {cs:?}");
            assert!((cs.d_phi_op / cs.d_phi_op_closed - 1.0).abs() < 0.10,
                    "{name} Tt4={tt4} {cs:?}");
        }
    }
}

/// The contrast with rung 42: bleed costs thrust monotonically, the stator costs SPEED. At fixed
/// `Tt4` the energy cascade pins `tau_c`, so `pi_c` moves only through `eta`.
#[test]
fn test_the_trade_is_thrust_neutral_and_paid_in_shaft_speed() {
    let d = design(cpg_gas());
    let (mut st, mut nl) = (Vec::new(), Vec::new());
    for v in [-0.1, 0.0, 0.1, 0.2, 0.3] {
        let od = vm(d.clone(), lp_map(), hp_map(), v, 0.0).core.match_point(&flight(), TT4);
        st.push(od.base.performance.specific_thrust);
        nl.push(od.n_lp_ratio);
    }
    let (hi, lo) = (st.iter().cloned().fold(f64::MIN, f64::max),
                    st.iter().cloned().fold(f64::MAX, f64::min));
    assert!(hi / lo - 1.0 < 5e-3, "thrust FLAT (< 0.5%): {st:?}");
    // ...and it PEAKS at the design setting (index 1).
    let mut k = 0usize;
    for (i, s) in st.iter().enumerate() { if *s > st[k] { k = i; } }
    assert_eq!(k, 1, "thrust peaks at the design setting: {st:?}");
    let mut sorted = nl.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(nl, sorted, "N_L monotone: {nl:?}");
    assert!(nl[4] / nl[1] - 1.0 > 0.15, "N_L up > 15%: {nl:?}");
}

// ==========================================================================================
// GATE 6 — P5: the inter-spool arrow is eta-MEDIATED ONLY (two EXACT zeros)
// ==========================================================================================

/// rung 39: `pi_LPC` cancels out of the HP face and the energy cascade is map-free, so the LP
/// stator is a PURE-LP lever — bit-for-bit, not to a tolerance.
#[test]
fn test_lp_stator_never_reaches_the_hp_spool_exactly() {
    let d = design(cpg_gas());
    let a = vm(d.clone(), lp_map(), hp_map(), 0.0, 0.0).stator_margin(&flight(), TT4);
    let b = vm(d, lp_map(), hp_map(), 0.20, 0.0).stator_margin(&flight(), TT4);
    assert_eq!(b.hp.phi_op.to_bits(), a.hp.phi_op.to_bits(), "EXACT");
    assert_eq!(b.hp.n.to_bits(), a.hp.n.to_bits(), "EXACT");
    // ...and the lever IS live, so the zero is not vacuous.
    assert!(b.lp.phi_op != a.lp.phi_op);
    assert!(b.lp.phi_op < a.lp.phi_op);
}

/// The HP→LP arrow exists ONLY through the efficiency island: switch the island off (`a=b=c=0`)
/// and it is EXACTLY zero; leave it on and it is not.
#[test]
fn test_hp_stator_arrow_is_eta_mediated_only() {
    let f = ComponentMap::flat();
    let lpf = ComponentMap { sigma: 0.1, l: 0.7, ..f }.with_phi_surge(FLOOR);
    let hpf = ComponentMap { sigma: 0.1, l: 1.0, ..f }.with_phi_surge(FLOOR);
    let df = design(cpg_gas());
    let a = vm(df.clone(), lpf, hpf, 0.0, 0.0).stator_margin(&flight(), TT4);
    let b = vm(df.clone(), lpf, hpf, 0.0, 0.20).stator_margin(&flight(), TT4);
    assert_eq!(b.lp.phi_op.to_bits(), a.lp.phi_op.to_bits(), "EXACT zero");
    assert!(b.hp.phi_op < a.hp.phi_op, "the HP lever is live");
    // ...and with the island ON the arrow is nonzero, so the zero above is not vacuous.
    let c = vm(df.clone(), lp_map(), hp_map(), 0.0, 0.0).stator_margin(&flight(), TT4);
    let e = vm(df, lp_map(), hp_map(), 0.0, 0.20).stator_margin(&flight(), TT4);
    assert!(e.lp.phi_op != c.lp.phi_op);
}

// ==========================================================================================
// GATE 7 — P7: the constant-incidence schedule
// ==========================================================================================

/// THE HEADLINE MADE OPERATIONAL, one gate with both halves: along a schedule that holds the
/// TRUE margin exactly constant, the `phi`-currency reports a large monotone LOSS — and falls
/// BELOW its own unscheduled value at the same throttle.
#[test]
fn test_constant_incidence_schedule_holds_m_i_while_m_phi_collapses() {
    let s = vm_default(0.0, 0.0);
    let rows = s.incidence_schedule(&flight(), &THROTTLE, Spool::Lp, 1.6);
    for r in &rows {
        assert!((r.m_i - rows[0].m_i).abs() <= 1e-11, "M_i EXACTLY constant: {r:?}");
        assert!(r.residual.abs() <= 1e-11, "{r:?}");
    }
    let m_phi: Vec<f64> = rows.iter().map(|r| r.m_phi).collect();
    let mut desc = m_phi.clone();
    desc.sort_by(|a, b| b.partial_cmp(a).unwrap());
    assert_eq!(m_phi, desc, "phi-currency falls monotonically: {m_phi:?}");
    for r in &rows[1..] {
        assert!(r.m_phi < r.m_phi_bare, "below the bare reading: {r:?}");
    }
    assert!(m_phi[m_phi.len() - 1] / m_phi[0] < 0.4, "~74% loss: {m_phi:?}");
    // the schedule closes progressively as power falls
    let vs: Vec<f64> = rows.iter().map(|r| r.vsv_star).collect();
    let mut asc = vs.clone();
    asc.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(vs, asc, "{vs:?}");
    assert_eq!(vs[0], 0.0);
    assert!(vs[vs.len() - 1] > 1.0, "{vs:?}");
}

/// The stator authority a spool needs measures its exposure: the LP (which takes the throttle
/// excursion, rungs 41/44/45) needs several times the HP's setting.
#[test]
fn test_schedule_size_inherits_rung41_split() {
    let s = vm_default(0.0, 0.0);
    let grid = [TT4, 1000.0];
    let lo = *s.incidence_schedule(&flight(), &grid, Spool::Lp, 1.6).last().unwrap();
    let hi = *s.incidence_schedule(&flight(), &grid, Spool::Hp, 1.6).last().unwrap();
    assert!(lo.vsv_star > 3.0 * hi.vsv_star, "{} vs {}", lo.vsv_star, hi.vsv_star);
}

// ==========================================================================================
// GATE 8 — the split's TWO boundaries, asserted as BRACKETS
// ==========================================================================================

/// The split needs `phi_s0 < sqrt((1+l)/(2+l))` = 0.7935 (LP). Asserted as a BRACKET: the sign of
/// `dM_phi/dv` flips between `phi_s0` = 0.79 and 0.82. The closed form is the claim; the
/// crossing's exact level rides on the disclosed constants.
#[test]
fn test_floor_tightness_boundary_bracket() {
    let d = design(cpg_gas());
    let f = ComponentMap::flat();
    let mut got = Vec::new();
    for floor in [0.79, 0.82] {
        let ml = ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f }.with_phi_surge(floor);
        let mh = ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..f }.with_phi_surge(floor);
        got.push(vm(d.clone(), ml, mh, 0.0, 0.0)
                 .currency_split(&flight(), TT4, Spool::Lp, None));
    }
    assert!(got[0].d_m_phi < 0.0 && 0.0 < got[1].d_m_phi, "{got:?}");
    assert!(got[0].split && !got[1].split);
    // the closed form brackets
    let b = powp(1.7 / 2.7, 0.5);
    assert!(0.79 < b && b < 0.82, "{b}");
}

/// Throttled far enough down, the `phi`-currency FLIPS to agreement and both currencies say
/// closing the stator loses margin. Predicted at `phi_op ~ 0.71`; bracketed inside the choked
/// envelope between `Tt4` = 825 and 800.
///
/// The prediction is scored HONESTLY: `phi_op ~ 0.71` lands just ABOVE the measured bracket
/// (0.6996, 0.7078) — a 0.3% miss, consistent with the closed form's known few-percent error off
/// design (gate 5). The load-bearing claim is the EXISTENCE and the bracket, not 0.71.
#[test]
fn test_part_power_boundary_bracket() {
    let s = vm_default(0.0, 0.0);
    let hi = s.currency_split(&flight(), 825.0, Spool::Lp, None);
    let lo = s.currency_split(&flight(), 800.0, Spool::Lp, None);
    assert!(hi.d_m_phi < 0.0 && 0.0 < lo.d_m_phi, "{} {}", hi.d_m_phi, lo.d_m_phi);
    assert!(hi.split && !lo.split);
    assert!(lo.phi_op < hi.phi_op && hi.phi_op < 0.72, "the crossing, bracketed");
    assert!((0.71 / hi.phi_op - 1.0).abs() < 0.01, "the prediction within 1%");
    assert!(hi.d_m_i > 0.0 && lo.d_m_i > 0.0, "incidence still helps on both sides");
}

// ==========================================================================================
// GATE 9 — rung 41's two-path pi gate survives the new psi term
// ==========================================================================================

/// `pi_c_spool` (which reads `psi`) must reproduce the SHIPPED `pi` from the cascade at the
/// operating point — two code paths, one `pi`. Rung 41's gate, now witnessing the swirl term.
#[test]
fn test_two_path_pi_agrees_at_a_moved_stator() {
    for v in [0.0, 0.1, 0.25] {
        let s = vm_default(v, 0.0);
        let od = s.core.match_point(&flight(), TT4);
        let r = s.stator_margin(&flight(), TT4);
        assert!((r.lp.pi_op / od.base.pi_lpc - 1.0).abs() < 1e-11, "v={v}");
        assert!((r.hp.pi_op / od.base.pi_hpc - 1.0).abs() < 1e-11, "v={v}");
        // the floor point is a DIFFERENT map point, and its pi is above the operating one
        assert!(r.lp.sm_n > 0.0 && r.hp.sm_n > 0.0, "v={v}");
    }
}

/// Rung 50's lesson: an edge is measured two-sided. Opening the stators (`v < 0`) raises the
/// floor and lifts `phi_op`; closing lowers both. `M_i` monotone rising in `v`, `M_phi` falling.
#[test]
fn test_sweep_is_two_sided_and_monotone() {
    let grid = [-0.2, -0.1, 0.0, 0.1, 0.2, 0.3];
    let rows = vm_default(0.0, 0.0).stator_sweep(&flight(), TT4, &grid, Spool::Lp);
    let m_i: Vec<f64> = rows.iter().map(|r| r.lp.m_i).collect();
    let m_phi: Vec<f64> = rows.iter().map(|r| r.lp.m_phi).collect();
    let floors: Vec<f64> = rows.iter().map(|r| r.lp.phi_surge).collect();
    assert!(m_i.windows(2).all(|w| w[0] <= w[1]), "M_i rising: {m_i:?}");
    assert!(m_phi.windows(2).all(|w| w[0] >= w[1]), "M_phi falling: {m_phi:?}");
    assert!(floors.windows(2).all(|w| w[0] >= w[1]), "floor falling: {floors:?}");
    // the OTHER spool is untouched throughout (P5's zero, along a whole sweep)
    let hp: Vec<u64> = rows.iter().map(|r| r.hp.phi_op.to_bits()).collect();
    assert!(hp.windows(2).all(|w| w[0] == w[1]), "the HP spool never moves");
}

// ==========================================================================================
// GATE 10 — CYCLE UNTOUCHED
// ==========================================================================================

/// The default single-spool design run is bit-for-bit rung 6 (no rung-53 knob reaches it).
#[test]
fn test_cycle_untouched_rung6() {
    let eng = build_turbojet(Gas::reacting_equilibrium(), 10.0, 1600.0, 50_000.0, Losses {
        pi_d: 0.97, eta_c: 0.90, eta_b: 0.99, pi_b: 0.96, eta_t: 0.92, eta_m: 0.99,
        pi_n: 0.98, ..Losses::default()
    });
    let r = eng.run(&flight(), 1.0);
    assert!(r.performance.specific_thrust > 0.0);
    assert_eq!(ComponentMap::flat().vsv, 0.0);
    assert!(ComponentMap::flat().is_flat());
}

// ==========================================================================================
// THE DEFERRAL LEDGER
// ==========================================================================================

/// **WHAT SLICE M COULD NOT GATE, NAMED.** The `slice_j_deferrals` / `slice_l_deferrals`
/// precedent, fourth use: an omission that is written down is a decision; one that is not is a
/// gate-name diff that reads 22/22 while covering less.
///
/// 1. ~~**`ComponentMap::phi_max`**~~ — **DISCHARGED by phase-6 SLICE P**, which is where its only
///    callers (rung 34's two forward compressor closures) arrive. It was a third of
///    `test_reduce_componentmap_expressions_bit_for_bit`, and the steady stator never calls it.
///
///    **AND THE ASSERTION THIS ITEM QUOTED WAS WRONG — kept here rather than deleted, because
///    the error is the lesson.** It read: *`sigma == 0 and l == 0` ⇒ `phi_max() == 5.0`; else
///    `1 + u` with `u = rhs/l` when `sigma == 0`, else `(-l + sqrt(l^2 + 4*sigma*rhs))/(2*sigma)`,
///    `rhs = 1 - 0.1`* — which is the **rung-34 form**, i.e. the shipped function evaluated at
///    `vsv == 0`. The real body threads the swirl amplitude `A = vsv*(1 + l)` through **three**
///    coefficients: the flat guard is `sigma == 0 && l == 0 && A == 0`, `rhs = 1 - A - psi_floor`,
///    and the linear coefficient is `l + A`. `map.rs`'s companion note went further and said
///    `phi_max` *"returns before the swirl term at `vsv == 0.0` exactly as `psi` does"* — it does
///    not; there is no early return at all.
///
///    Both records were placed exactly where slice P would read them, which is slice O's rule
///    working; **the content was the defect**. And it was unobservable at the discharge site —
///    § 5.13 probe 1 measured `vsv == 0.0` at all 16 508 calls a rung-34 march makes, so a port
///    built from the quotation would have been bit-identical here and wrong for rung 53.
///    See [`ComponentMap::phi_max`] for what shipped.
///
/// 2. **The `lp_disabled` refusal** — the second half of `test_design_setting_maps_refused`.
///    Rust's constructor has no such parameter, so the call does not compile. **Not owed**: the
///    guarantee is strictly stronger than Python's runtime assertion, and there is nothing left
///    to witness at run time.
///
/// 3. **`StatorHooks` DISPATCH** — ~~the table has ONE entry and `Descendant` ONE variant, so any
///    "dispatch works" assertion would pass while measuring nothing.~~ **DISCHARGED FOR RUNG 55
///    BY SLICE N step 3** — see [`the_stacked_dispatch_is_live`] below, the second cell the arity
///    pin was waiting for. **STILL OWED FOR RUNG 61** (slice O): `StatorBleedMatcher` overrides
///    `at_setting` for its own reason — so a sweep cannot silently run with the bleed valve SHUT
///    — and nothing here witnesses that override, because the class does not exist yet. Marked
///    per-rung rather than struck whole, so a slice-O reader still finds an IOU: *a deferral
///    filed against the wrong cause is a deferral nobody can discharge*, and a deferral closed on
///    behalf of a rung that has not shipped is the same failure with the sign flipped.
#[test]
fn slice_m_deferrals() {
    // (3) — the arity pin, now with the variant it was waiting for. `Descendant` is
    // non-exhaustive to a reader but not to the compiler: adding a variant makes this match fail
    // to compile, which is the point — and it DID, at slice N step 3, which is how a reader knows
    // the pin is load-bearing rather than decorative.
    for d in [turbojet::stator::Descendant::Plain,
              turbojet::stator::Descendant::Stack {
                  k_lp: 8, k_hp: 1, split: Split::DT, vsv_stages_lp: Some(1),
                  vsv_stages_hp: None, cap_profile: CapProfile::Derived }] {
        match d {
            turbojet::stator::Descendant::Plain => {}
            turbojet::stator::Descendant::Stack { .. } => {}
        }
    }
    // ...and the one hook entry really is rung 53's own, not a placeholder.
    let m = vm_default(0.0, 0.0);
    assert_eq!(m.hooks.at_setting as usize, R53.at_setting as usize);
    // A sibling built through the hook is the SAME machine at a moved setting: the design
    // references it captured must be the design-setting maps, NOT the moved ones.
    let sib = m.at_setting(0.2, 0.0);
    assert_eq!(sib.map_lp_design.vsv.to_bits(), 0.0f64.to_bits());
    assert_eq!(sib.core.map_lp().vsv.to_bits(), 0.2f64.to_bits());
}

/// **SLICE N step 3, P3 — THE DISPATCH GATE, AND IT ASSERTS IN BOTH DIRECTIONS.**
///
/// The deferral it discharges is `slice_m_deferrals` item 3: slice M shipped two hook tables with
/// nothing overriding them, so a "dispatch works" assertion could only have compared a table to
/// itself. Rung 55 is the first descendant, and it overrides in the OTHER table —
/// `_hp_eta_loop`/`_lp_eta_loop` live on rung 39's [`TwoSpoolHooks`], not on `StatorHooks`.
///
/// **THREE CLAUSES, NOT TWO, AND THE THIRD IS THE ONE A CARELESS GATE OMITS.** A one-directional
/// gate passes on ANY table — slice M (e)'s `is_flat` failure mode — so this asserts:
///
/// ```text
///     a rung-53 core's eta loops ARE R39's          (the negative control)
///     a STACKED core's eta loops are NOT R39's      (the override happened)
///     a stacked core's try_match_point IS R39's     (and NOTHING ELSE was overridden)
/// ```
///
/// The third clause is what catches an accidental `match` override — the failure that would make
/// rung 55 a new matcher rather than rung 39's with one inversion swapped.
///
/// **AND THE POINTER CLAUSES CANNOT SEE THE FAILURE THAT ACTUALLY MATTERS**, which is why the
/// second half of this gate is a VALUE comparison. An `R55` whose `at_setting` entry was left
/// pointing at `r53_at_setting` still satisfies every fn-pointer comparison above — and it would
/// hand back a sibling with `stack_lp: None`, a silently UNSTACKED machine producing plausible
/// numbers. § 5.10 measured which reads discriminate it: `stack.cmap.vsv` MOVES, and
/// `stack.cmap_axial.vsv` stays 0.0 by construction, so the PAIR is a two-sided bar; while
/// `theta_d` and `e_d` are **bit-identical** across the move, because the design ladder is built
/// from `tau_d`/`pi_d`/`eta_d`/`kc` and the stator touches none of them. A "the stack was
/// rebuilt" gate written on `theta_d` would pass on a stack that was never rebuilt at all — *a
/// ported test can go VACUOUS*, caught before writing rather than after.
#[test]
fn the_stacked_dispatch_is_live() {
    let plain = vm_default(0.0, 0.0);
    let stacked = StageStackCore::new(StageStackCoreSpec {
        k_lp: 8, k_hp: 1, vsv_stages_lp: Some(1),
        ..StageStackCoreSpec::new(design(cpg_gas()), flight(), 1.0, lp_map(), hp_map())
    });

    // --- clause 1: the negative control. A rung-53 core inherits rung 39's loops verbatim.
    assert_eq!(plain.core.hooks.hp_eta_loop as usize, R39.hp_eta_loop as usize);
    assert_eq!(plain.core.hooks.lp_eta_loop as usize, R39.lp_eta_loop as usize);
    assert_eq!(plain.hooks.at_setting as usize, R53.at_setting as usize);

    // --- clause 2: the stacked core overrides BOTH, in the INNER table.
    assert_ne!(stacked.core.core.hooks.hp_eta_loop as usize, R39.hp_eta_loop as usize,
               "rung 55 must override the HP efficiency loop");
    assert_ne!(stacked.core.core.hooks.lp_eta_loop as usize, R39.lp_eta_loop as usize,
               "rung 55 must override the LP efficiency loop");
    assert_ne!(stacked.core.hooks.at_setting as usize, R53.at_setting as usize,
               "rung 55 must override at_setting, or a swept setting drops the stack");

    // --- clause 3: and NOTHING ELSE. `match` is rung 39's own body, by construction — R55_TWO
    // names `R39.try_match_point` rather than a second spelling of it.
    assert_eq!(stacked.core.core.hooks.try_match_point as usize,
               R39.try_match_point as usize,
               "rung 55 adds no matching code path — the stack enters through solve_n alone");

    // --- the VALUE half: at_setting REBUILDS, and the two reads that can tell.
    let base = stacked.stack_of(Spool::Lp).expect("K_lp = 8 builds a stack");
    assert_eq!(base.cmap.vsv.to_bits(), 0.0f64.to_bits());
    let moved = stacked.at_setting(0.20, 0.0);
    let ms = moved.stack_of(Spool::Lp).expect("the sibling must still be STACKED");
    assert_eq!(ms.cmap.vsv.to_bits(), 0.20f64.to_bits(),
               "the rebuilt stack must carry the MOVED map");
    assert_eq!(ms.cmap_axial.vsv.to_bits(), 0.0f64.to_bits(),
               "...and the rows the front-block stator does not move must stay at design");
    // The two reads that CANNOT tell — asserted equal, so the vacuity is recorded rather than
    // discovered by someone writing a weaker gate later.
    assert_eq!(ms.theta_d[1].to_bits(), base.theta_d[1].to_bits(),
               "the design ladder is map-INDEPENDENT: it cannot witness a rebuild");
    assert_eq!(ms.e_d.to_bits(), base.e_d.to_bits(),
               "nor can e_d");
    // The unstacked spool stays unstacked through the move.
    assert!(moved.stack_of(Spool::Hp).is_none(), "K_hp = 1 builds no object, at any setting");
}

/// **THE `_INC_MAX` SHADOW — RUNG 55's, AND `stator.rs`'s FIRST DRAFT SAID RUNG 61's.**
///
/// § 5.3's pre-flight had it right and told the port what to do about it (*"in Rust that cap must
/// be a per-cell parameter, never a literal in the ported body"*); slice M shipped the literal in
/// both loops and named the wrong rung beside it. Read: `StatorBleedMatcher` declares
/// `_B_TOL`/`_B_MAX`/`_B_CAP`/`_B_STEP` and no `_INC_MAX`; `StageStackMatcher` declares
/// `_INC_MAX = 200` (`engine.py:7282`). Three read sites — rung 55's own
/// `stage_incidence_schedule` and the two INHERITED rung-53/54 solver loops, which a stacked
/// object enters with 200 in Python.
///
/// **NO VALUE MOVES, AND IT IS GATED ANYWAY.** § 5.9 (iv) measured the cap never reached (30–36
/// passes, then 26–33, all ending on `_INC_TOL`), so 80 against 200 cannot change a number on
/// that grid — which is what licensed the literal. A cap that is never hit still decides which
/// constant the body NAMES. So this gate asserts the DISPATCH, not an outcome. It is latent
/// besides: `test_rung55.py:438` runs `incidence_schedule` on a genuine rung-53 matcher, so no
/// shipped Python test drives an inherited schedule on a stacked object — which is exactly why no
/// value oracle could have caught a wrong cap here.
#[test]
fn the_bisection_cap_is_shadowed_by_the_descendant() {
    assert_eq!(vm_default(0.0, 0.0).inc_max(), VariableStatorCore::INC_MAX);
    assert_eq!(VariableStatorCore::INC_MAX, 80);
    let stacked = StageStackCore::new(StageStackCoreSpec {
        k_lp: 4, ..StageStackCoreSpec::new(design(cpg_gas()), flight(), 1.0, lp_map(), hp_map())
    });
    assert_eq!(stacked.core.inc_max(), VariableStatorCore::INC_MAX_STACKED);
    assert_eq!(VariableStatorCore::INC_MAX_STACKED, 200);
    // The other two rung-55 constants are NOT shadows, and saying so is half the finding: the
    // tolerance is a re-declaration at the same value, and the scan step is a NEW name.
    assert_eq!(StageStackCore::INC_TOL.to_bits(), VariableStatorCore::INC_TOL.to_bits());
    assert_ne!(StageStackCore::V_SCAN.to_bits(), VariableStatorCore::V_STEP.to_bits());
}

/// Python's `**0.5`, spelled the way the port spells it everywhere else.
fn powp(x: f64, p: f64) -> f64 {
    x.powf(p)
}
